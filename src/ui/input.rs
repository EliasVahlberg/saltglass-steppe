//! Input handling for game UI

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io::Result;
use crate::GameState;
use super::{InventoryMenu, QuestLogMenu, CraftingMenu, WikiMenu, TradeMenu, MenuPanel, WorldMapView, DebugMenu, IssueReporter, PsychicMenu, SkillsMenu};
use super::trade_menu::TradeMode;
use crate::all_recipe_ids;

/// Look mode cursor state
pub struct LookMode {
    pub active: bool,
    pub x: i32,
    pub y: i32,
}

impl Default for LookMode {
    fn default() -> Self {
        Self { active: false, x: 0, y: 0 }
    }
}

/// Debug console state
#[derive(Default)]
pub struct DebugConsole {
    pub active: bool,
    pub input: String,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub suggestions: Vec<String>,
    pub suggestion_index: usize,
}

impl DebugConsole {
    pub fn toggle(&mut self) { 
        self.active = !self.active; 
        if self.active {
            self.input.clear();
            self.history_index = None;
            self.update_suggestions();
        }
    }
    
    pub fn push(&mut self, c: char) { 
        self.input.push(c); 
        self.update_suggestions();
    }
    
    pub fn pop(&mut self) { 
        self.input.pop(); 
        self.update_suggestions();
    }
    
    pub fn submit(&mut self) -> Option<String> {
        if self.input.is_empty() { return None; }
        let cmd = self.input.clone();
        self.history.push(cmd.clone());
        if self.history.len() > 50 { // Keep last 50 commands
            self.history.remove(0);
        }
        self.input.clear();
        self.history_index = None;
        self.suggestions.clear();
        Some(cmd)
    }
    
    pub fn history_up(&mut self) {
        if self.history.is_empty() { return; }
        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
                self.input = self.history[self.history.len() - 1].clone();
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.input = self.history[idx - 1].clone();
            }
            _ => {}
        }
        self.update_suggestions();
    }
    
    pub fn history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input = self.history[idx + 1].clone();
            }
            Some(_) => {
                self.history_index = None;
                self.input.clear();
            }
            None => {}
        }
        self.update_suggestions();
    }
    
    pub fn accept_suggestion(&mut self) {
        if !self.suggestions.is_empty() && self.suggestion_index < self.suggestions.len() {
            self.input = self.suggestions[self.suggestion_index].clone();
            self.suggestions.clear();
        }
    }
    
    pub fn next_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.suggestion_index = (self.suggestion_index + 1) % self.suggestions.len();
        }
    }
    
    pub fn prev_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.suggestion_index = if self.suggestion_index == 0 {
                self.suggestions.len() - 1
            } else {
                self.suggestion_index - 1
            };
        }
    }
    
    fn update_suggestions(&mut self) {
        self.suggestions.clear();
        self.suggestion_index = 0;
        
        if self.input.is_empty() {
            return;
        }
        
        let input_lower = self.input.to_lowercase();
        let mut candidates = Vec::new();
        
        // Command suggestions
        let commands = [
            "show tile", "hide tile", "sturdy", "phase", "save_debug", "load_debug", 
            "list_debug", "debug_info", "run_des", "list_des", "help", "spawn",
            "terminals", "report_issue", "complete_quest", "list_quests", "interact", "examine", "collect_data"
        ];
        
        for cmd in &commands {
            if cmd.starts_with(&input_lower) {
                candidates.push(cmd.to_string());
            }
        }
        
        // Command-specific suggestions
        if self.input.contains(' ') {
            let parts: Vec<&str> = self.input.split_whitespace().collect();
            if let Some(command) = parts.first() {
                match *command {
                    "complete_quest" => {
                        use crate::game::quest::all_quest_ids;
                        for quest_id in all_quest_ids() {
                            let suggestion = format!("complete_quest {}", quest_id);
                            if parts.len() == 1 || quest_id.contains(&parts[1..].join(" ")) {
                                candidates.push(suggestion);
                            }
                        }
                    }
                    "interact" | "examine" => {
                        // These commands can target any entity type
                        let entities = ["enemy", "npc", "item", "chest"];
                        for entity in &entities {
                            let suggestion = format!("{} {}", command, entity);
                            if parts.len() == 1 || entity.contains(&parts[1..].join(" ")) {
                                candidates.push(suggestion);
                            }
                        }
                    }
                    _ => {
                        // No specific suggestions for other commands
                    }
                }
            }
        }
        
        candidates.sort();
        candidates.dedup();
        self.suggestions = candidates;
    }
}

/// Dialog box state for NPC conversations
#[derive(Default)]
pub struct DialogBox {
    pub active: bool,
    pub speaker: String,
    pub pages: Vec<String>,
    pub current_page: usize,
    pub chars_shown: usize,
    pub cpm: u32, // characters per minute
}

impl DialogBox {
    pub fn show(&mut self, speaker: &str, text: &str) {
        self.active = true;
        self.speaker = speaker.to_string();
        self.pages = text.split("</nextpage>").map(|s| s.trim().to_string()).collect();
        self.current_page = 0;
        self.chars_shown = 0;
        self.cpm = 1200; // default ~20 chars/sec
    }
    
    pub fn close(&mut self) {
        self.active = false;
    }
    
    pub fn next_page(&mut self) -> bool {
        let char_count = self.current_text().chars().count();
        // If text still revealing, show all
        if self.chars_shown < char_count {
            self.chars_shown = char_count;
            return true;
        }
        // Go to next page or close
        if self.current_page + 1 < self.pages.len() {
            self.current_page += 1;
            self.chars_shown = 0;
            true
        } else {
            self.close();
            false
        }
    }
    
    pub fn current_text(&self) -> &str {
        self.pages.get(self.current_page).map(|s| s.as_str()).unwrap_or("")
    }
    
    pub fn visible_text(&self) -> &str {
        let text = self.current_text();
        // chars_shown counts characters, not bytes - find the byte index
        let byte_end = text.char_indices()
            .nth(self.chars_shown)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        &text[..byte_end]
    }
    
    pub fn tick(&mut self, frame_ms: u64) {
        if !self.active { return; }
        let chars_per_ms = self.cpm as f64 / 60000.0;
        let chars_to_add = (chars_per_ms * frame_ms as f64).max(1.0) as usize;
        let char_count = self.current_text().chars().count();
        self.chars_shown = (self.chars_shown + chars_to_add).min(char_count);
    }
}

/// UI-specific state, separate from game logic
pub struct UiState {
    pub look_mode: LookMode,
    pub frame_count: u64,
    pub show_controls: bool,
    pub inventory_menu: InventoryMenu,
    pub quest_log: QuestLogMenu,
    pub crafting_menu: CraftingMenu,
    pub pause_menu: PauseMenu,
    pub wiki_menu: WikiMenu,
    pub trade_menu: TradeMenu,
    pub world_map_view: WorldMapView,
    pub target_enemy: Option<usize>,
    pub debug_console: DebugConsole,
    pub debug_menu: DebugMenu,
    pub issue_reporter: IssueReporter,
    pub psychic_menu: PsychicMenu,
    pub skills_menu: SkillsMenu,
    pub dialog_box: DialogBox,
    pub book_reader: BookReader,
    pub chest_ui: Option<super::ChestUI>,
    /// Smooth camera position (lerped toward player)
    pub camera_x: f32,
    pub camera_y: f32,
}

/// Book reader state
#[derive(Default)]
pub struct BookReader {
    pub active: bool,
    pub book_id: String,
    pub current_page: usize,
}

impl BookReader {
    pub fn open(&mut self, book_id: &str) {
        self.active = true;
        self.book_id = book_id.to_string();
        self.current_page = 0;
    }
    
    pub fn close(&mut self) {
        self.active = false;
    }
    
    pub fn next_page(&mut self, total_pages: usize) {
        if self.current_page + 1 < total_pages {
            self.current_page += 1;
        }
    }
    
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }
}

/// Pause menu state
#[derive(Default)]
pub struct PauseMenu {
    pub active: bool,
    pub selected_index: usize,
}

impl PauseMenu {
    pub fn open(&mut self) { self.active = true; self.selected_index = 0; }
    pub fn close(&mut self) { self.active = false; }
    pub fn navigate(&mut self, dy: i32, count: usize) {
        self.selected_index = (self.selected_index as i32 + dy).rem_euclid(count as i32) as usize;
    }
}

impl UiState {
    pub fn new() -> Self {
        Self {
            look_mode: LookMode::default(),
            frame_count: 0,
            show_controls: false,
            inventory_menu: InventoryMenu::default(),
            quest_log: QuestLogMenu::default(),
            crafting_menu: CraftingMenu::default(),
            pause_menu: PauseMenu::default(),
            wiki_menu: WikiMenu::default(),
            trade_menu: TradeMenu::default(),
            world_map_view: WorldMapView::default(),
            target_enemy: None,
            debug_console: DebugConsole::default(),
            debug_menu: DebugMenu::default(),
            issue_reporter: IssueReporter::default(),
            psychic_menu: PsychicMenu::default(),
            skills_menu: SkillsMenu::default(),
            dialog_box: DialogBox::default(),
            book_reader: BookReader::default(),
            chest_ui: None,
            camera_x: 0.0,
            camera_y: 0.0,
        }
    }
    
    pub fn tick_frame(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    /// Lerp camera toward target position
    pub fn update_camera(&mut self, target_x: i32, target_y: i32) {
        let lerp_speed = 0.2;
        self.camera_x += (target_x as f32 - self.camera_x) * lerp_speed;
        self.camera_y += (target_y as f32 - self.camera_y) * lerp_speed;
    }
}

/// Game actions that can be triggered by input
pub enum Action {
    Quit,
    Move(i32, i32),
    Save,
    Load,
    UseItem(usize),
    OpenControls,
    EnterLook,
    BreakWall(i32, i32),
    EndTurn,
    Wait,
    AutoExplore,
    RangedAttack(i32, i32),
    OpenInventory,
    EquipSelected,
    UnequipSelected,
    OpenQuestLog,
    OpenCrafting,
    OpenWiki,
    OpenPsychicMenu,
    OpenSkillsMenu,
    UsePsychicAbility(String),
    RangedAttackMode,
    TargetMode,
    OpenWorldMap,
    WorldMapTravel(usize, usize),
    Craft,
    TradeBuy(usize),
    TradeSell(usize),
    OpenPauseMenu,
    ReturnToMainMenu,
    SetTarget(i32, i32),
    UseStairs,
    DebugCommand(String),
    OpenDebugMenu,
    OpenIssueReporter,
    SubmitIssueReport,
    OpenChest(usize),
    ChestTransfer,
    CloseChest,
    Interact(i32, i32),
    Examine(i32, i32),
    None,
}

/// Handle input and return the resulting action
pub fn handle_input(ui: &mut UiState, state: &mut GameState) -> Result<Action> {
    if !event::poll(std::time::Duration::from_millis(16))? {
        return Ok(Action::None);
    }
    
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(Action::None);
        }
        
        // Death screen - only allow Esc to return to main menu
        if state.player_hp <= 0 {
            match key.code {
                KeyCode::Esc => return Ok(Action::ReturnToMainMenu),
                _ => return Ok(Action::None),
            }
        }
        
        // Dialog box input (highest priority when active)
        if ui.dialog_box.active {
            match key.code {
                KeyCode::Esc => ui.dialog_box.close(),
                KeyCode::Enter | KeyCode::Char(' ') => { ui.dialog_box.next_page(); }
                _ => {}
            }
            return Ok(Action::None);
        }
        
        // Debug console input
        if ui.debug_console.active {
            return Ok(handle_debug_console_input(ui, key.code));
        }
        
        // Issue reporter input
        if ui.issue_reporter.active {
            return Ok(handle_issue_reporter_input(ui, key.code));
        }
        
        // Psychic menu input
        if ui.psychic_menu.active {
            return Ok(handle_psychic_menu_input(ui, state, key.code));
        }
        // Skills menu input
        if ui.skills_menu.active {
            return Ok(handle_skills_menu_input(ui, state, key.code));
        }
        // Debug menu input
        if ui.debug_menu.active {
            return Ok(handle_debug_menu_input(ui, key.code));
        }
        
        // Pause menu input
        if ui.pause_menu.active {
            return Ok(handle_pause_menu_input(ui, key.code));
        }
        // World map view input
        if ui.world_map_view.open {
            return Ok(handle_world_map_input(ui, state, key.code));
        }
        // Book reader input
        if ui.book_reader.active {
            return Ok(handle_book_reader_input(ui, key.code));
        }
        // Wiki menu input
        if ui.wiki_menu.active {
            return Ok(handle_wiki_input(ui, state, key.code));
        }
        // Chest UI input
        if ui.chest_ui.is_some() {
            return Ok(handle_chest_input(ui, key.code));
        }
        // Trade menu input
        if ui.trade_menu.active {
            return Ok(handle_trade_input(ui, state, key.code));
        }
        // Quest log input
        if ui.quest_log.active {
            return Ok(handle_quest_log_input(ui, state, key.code));
        }
        // Crafting menu input
        if ui.crafting_menu.active {
            return Ok(handle_crafting_input(ui, key.code));
        }
        // Inventory menu input
        if ui.inventory_menu.active {
            return Ok(handle_inventory_input(ui, state, key.code));
        }
        // Look mode input
        if ui.look_mode.active {
            return Ok(handle_look_input(ui, key.code));
        }
        // Normal game input
        return Ok(handle_game_input(ui, key.code));
    }
    Ok(Action::None)
}

fn handle_book_reader_input(ui: &mut UiState, code: KeyCode) -> Action {
    use crate::game::book::get_book_def;
    
    let total_pages = match get_book_def(&ui.book_reader.book_id) {
        Some(def) => def.pages.len(),
        None => 0,
    };

    match code {
        KeyCode::Esc | KeyCode::Char('q') => ui.book_reader.close(),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => ui.book_reader.next_page(total_pages),
        KeyCode::Left | KeyCode::Char('h') => ui.book_reader.prev_page(),
        _ => {}
    }
    Action::None
}

fn handle_wiki_input(ui: &mut UiState, _state: &mut GameState, code: KeyCode) -> Action {
    use crate::game::{all_item_ids, all_enemy_ids};
    use crate::game::npc::all_npc_ids;
    use super::wiki::WikiTab;
    
    let count = match ui.wiki_menu.tab {
        WikiTab::Items => all_item_ids().len(),
        WikiTab::Enemies => all_enemy_ids().len(),
        WikiTab::NPCs => all_npc_ids().len(),
    };
    match code {
        KeyCode::Esc | KeyCode::Char('w') => ui.wiki_menu.close(),
        KeyCode::Tab | KeyCode::Char('l') => ui.wiki_menu.next_tab(),
        KeyCode::BackTab | KeyCode::Char('h') => ui.wiki_menu.prev_tab(),
        KeyCode::Char('j') | KeyCode::Down => ui.wiki_menu.navigate(1, count),
        KeyCode::Char('k') | KeyCode::Up => ui.wiki_menu.navigate(-1, count),
        _ => {}
    }
    Action::None
}

fn handle_quest_log_input(ui: &mut UiState, state: &mut GameState, code: KeyCode) -> Action {
    let total = state.quest_log.active.len() + state.quest_log.completed.len() + 3;
    match code {
        KeyCode::Esc | KeyCode::Char('q') => ui.quest_log.close(),
        KeyCode::Char('j') | KeyCode::Down => ui.quest_log.navigate(1, total),
        KeyCode::Char('k') | KeyCode::Up => ui.quest_log.navigate(-1, total),
        _ => {}
    }
    Action::None
}

fn handle_crafting_input(ui: &mut UiState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc | KeyCode::Char('c') => ui.crafting_menu.close(),
        KeyCode::Char('j') | KeyCode::Down => ui.crafting_menu.navigate(1, all_recipe_ids().len()),
        KeyCode::Char('k') | KeyCode::Up => ui.crafting_menu.navigate(-1, all_recipe_ids().len()),
        KeyCode::Enter => return Action::Craft,
        _ => {}
    }
    Action::None
}

fn handle_inventory_input(ui: &mut UiState, state: &mut GameState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc | KeyCode::Char('i') => ui.inventory_menu.close(),
        KeyCode::Char('j') | KeyCode::Down => ui.inventory_menu.navigate(1, state.inventory.len()),
        KeyCode::Char('k') | KeyCode::Up => ui.inventory_menu.navigate(-1, state.inventory.len()),
        KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left | KeyCode::Right => ui.inventory_menu.switch_panel(),
        KeyCode::Char('x') => ui.inventory_menu.inspect(&state.inventory, &state.equipment),
        KeyCode::Char('u') => {
            // Use selected item from inventory
            if ui.inventory_menu.panel == MenuPanel::Inventory && ui.inventory_menu.inspect_item.is_none() {
                if let Some(idx) = ui.inventory_menu.selected_inv_index() {
                    if idx < state.inventory.len() {
                        return Action::UseItem(idx);
                    }
                }
            }
        }
        KeyCode::Enter => {
            if ui.inventory_menu.inspect_item.is_some() {
                ui.inventory_menu.inspect_item = None;
            } else {
                return match ui.inventory_menu.panel {
                    MenuPanel::Inventory => Action::EquipSelected,
                    MenuPanel::Equipment => Action::UnequipSelected,
                };
            }
        }
        _ => {}
    }
    Action::None
}

fn handle_look_input(ui: &mut UiState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc | KeyCode::Enter => ui.look_mode.active = false,
        KeyCode::Up | KeyCode::Char('k') => ui.look_mode.y -= 1,
        KeyCode::Down | KeyCode::Char('j') => ui.look_mode.y += 1,
        KeyCode::Left | KeyCode::Char('h') => ui.look_mode.x -= 1,
        KeyCode::Right | KeyCode::Char('l') => ui.look_mode.x += 1,
        KeyCode::Char('b') => {
            let (x, y) = (ui.look_mode.x, ui.look_mode.y);
            ui.look_mode.active = false;
            return Action::BreakWall(x, y);
        }
        KeyCode::Char('f') => {
            let (x, y) = (ui.look_mode.x, ui.look_mode.y);
            ui.look_mode.active = false;
            return Action::RangedAttack(x, y);
        }
        KeyCode::Char('t') => {
            let (x, y) = (ui.look_mode.x, ui.look_mode.y);
            ui.look_mode.active = false;
            return Action::SetTarget(x, y);
        }
        _ => {}
    }
    Action::None
}

/// Pause menu options
pub const PAUSE_OPTIONS: &[&str] = &["Resume", "Save", "Controls", "Main Menu", "Quit"];

fn handle_pause_menu_input(ui: &mut UiState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc => { ui.pause_menu.close(); }
        KeyCode::Up | KeyCode::Char('k') => ui.pause_menu.navigate(-1, PAUSE_OPTIONS.len()),
        KeyCode::Down | KeyCode::Char('j') => ui.pause_menu.navigate(1, PAUSE_OPTIONS.len()),
        KeyCode::Enter => {
            let selected = ui.pause_menu.selected_index;
            ui.pause_menu.close();
            return match selected {
                0 => Action::None, // Resume
                1 => Action::Save,
                2 => Action::OpenControls,
                3 => Action::ReturnToMainMenu,
                4 => Action::Quit,
                _ => Action::None,
            };
        }
        _ => {}
    }
    Action::None
}

fn handle_debug_console_input(ui: &mut UiState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc | KeyCode::Char('`') => ui.debug_console.toggle(),
        KeyCode::Enter => {
            if let Some(cmd) = ui.debug_console.submit() {
                if cmd == "report_issue" {
                    ui.debug_console.toggle(); // Close debug console first
                    ui.issue_reporter.open();
                    return Action::None;
                }
                return Action::DebugCommand(cmd);
            }
        }
        KeyCode::Backspace => { ui.debug_console.pop(); }
        KeyCode::Up => { 
            if !ui.debug_console.suggestions.is_empty() {
                ui.debug_console.prev_suggestion();
            } else {
                ui.debug_console.history_up();
            }
        }
        KeyCode::Down => { 
            if !ui.debug_console.suggestions.is_empty() {
                ui.debug_console.next_suggestion();
            } else {
                ui.debug_console.history_down();
            }
        }
        KeyCode::Tab => { ui.debug_console.accept_suggestion(); }
        KeyCode::Right if !ui.debug_console.suggestions.is_empty() => { 
            ui.debug_console.next_suggestion(); 
        }
        KeyCode::Left if !ui.debug_console.suggestions.is_empty() => { 
            ui.debug_console.prev_suggestion(); 
        }
        KeyCode::Char(c) => ui.debug_console.push(c),
        _ => {}
    }
    Action::None
}

fn handle_world_map_input(ui: &mut UiState, state: &mut GameState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('M') => {
            ui.world_map_view.open = false;
        }
        KeyCode::Up | KeyCode::Char('k') => ui.world_map_view.move_cursor(0, -1),
        KeyCode::Down | KeyCode::Char('j') => ui.world_map_view.move_cursor(0, 1),
        KeyCode::Left | KeyCode::Char('h') => ui.world_map_view.move_cursor(-1, 0),
        KeyCode::Right | KeyCode::Char('l') => ui.world_map_view.move_cursor(1, 0),
        KeyCode::Char('c') => {
            // Center on player
            ui.world_map_view.cursor_x = state.world_x;
            ui.world_map_view.cursor_y = state.world_y;
        }
        KeyCode::Enter => {
            // Travel to cursor position
            let (wx, wy) = (ui.world_map_view.cursor_x, ui.world_map_view.cursor_y);
            ui.world_map_view.open = false;
            return Action::WorldMapTravel(wx, wy);
        }
        _ => {}
    }
    Action::None
}

fn handle_game_input(ui: &mut UiState, code: KeyCode) -> Action {
    match code {
        KeyCode::Char('`') => { ui.debug_console.toggle(); Action::None }
        KeyCode::F(12) => { ui.debug_menu.toggle(); Action::None }
        KeyCode::Char('S') => Action::Save,
        KeyCode::Char('L') => Action::Load,
        KeyCode::Char('x') => Action::EnterLook,
        KeyCode::Char('X') => Action::Examine(0, 0), // Will be set to player position in main loop
        KeyCode::Char('e') => Action::Wait,
        KeyCode::Char('E') => Action::Interact(0, 0), // Will be set to player position in main loop
        KeyCode::Char('o') => Action::AutoExplore,
        KeyCode::Char('i') => Action::OpenInventory,
        KeyCode::Char('q') => Action::OpenQuestLog,
        KeyCode::Char('c') => Action::OpenCrafting,
        KeyCode::Char('C') => Action::OpenChest(0), // Placeholder, will be handled in main loop
        KeyCode::Char('w') => Action::OpenWiki,
        KeyCode::Char('p') => Action::OpenPsychicMenu,
        KeyCode::Char('s') => Action::OpenSkillsMenu,
        KeyCode::Char('f') => Action::RangedAttackMode,
        KeyCode::Char('t') => Action::TargetMode,
        KeyCode::Char('m') | KeyCode::Char('M') => Action::OpenWorldMap,
        KeyCode::Char('<') | KeyCode::Char('>') => Action::UseStairs,
        KeyCode::Char('1') => Action::UseItem(0),
        KeyCode::Char('2') => Action::UseItem(1),
        KeyCode::Char('3') => Action::UseItem(2),
        KeyCode::Up | KeyCode::Char('k') => Action::Move(0, -1),
        KeyCode::Down | KeyCode::Char('j') => Action::Move(0, 1),
        KeyCode::Left | KeyCode::Char('h') => Action::Move(-1, 0),
        KeyCode::Right | KeyCode::Char('l') => Action::Move(1, 0),
        KeyCode::Esc => Action::OpenPauseMenu,
        _ => Action::None,
    }
}

fn handle_chest_input(ui: &mut UiState, key: KeyCode) -> Action {
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(ref mut chest_ui) = ui.chest_ui {
                chest_ui.move_selection(-1);
            }
            Action::None
        },
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(ref mut chest_ui) = ui.chest_ui {
                chest_ui.move_selection(1);
            }
            Action::None
        },
        KeyCode::Tab => {
            if let Some(ref mut chest_ui) = ui.chest_ui {
                chest_ui.switch_panel();
            }
            Action::None
        },
        KeyCode::Enter => Action::ChestTransfer,
        KeyCode::Esc => Action::CloseChest,
        _ => Action::None,
    }
}

fn handle_trade_input(ui: &mut UiState, state: &mut GameState, key: KeyCode) -> Action {
    let interface = match &ui.trade_menu.interface {
        Some(i) => i,
        None => return Action::None,
    };
    
    let list_len = match ui.trade_menu.mode {
        TradeMode::Buy => interface.available_items.len(),
        TradeMode::Sell => state.inventory.len(),
    };
    
    match key {
        KeyCode::Esc => {
            ui.trade_menu.close();
            Action::None
        },
        KeyCode::Tab => {
            ui.trade_menu.toggle_mode();
            Action::None
        },
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
            ui.trade_menu.navigate(-1, list_len);
            Action::None
        },
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
            ui.trade_menu.navigate(1, list_len);
            Action::None
        },
        KeyCode::Enter => {
            if list_len == 0 { return Action::None; }
            match ui.trade_menu.mode {
                TradeMode::Buy => Action::TradeBuy(ui.trade_menu.selected_index),
                TradeMode::Sell => Action::TradeSell(ui.trade_menu.selected_index),
            }
        },
        _ => Action::None,
    }
}
fn handle_debug_menu_input(ui: &mut UiState, code: KeyCode) -> Action {
    match code {
        KeyCode::F(12) | KeyCode::Esc => {
            ui.debug_menu.toggle();
            Action::None
        }
        KeyCode::Tab => {
            ui.debug_menu.next_tab();
            Action::None
        }
        KeyCode::BackTab => {
            ui.debug_menu.prev_tab();
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_issue_reporter_input(ui: &mut UiState, code: KeyCode) -> Action {
    use super::issue_reporter::IssueStep;
    
    match code {
        KeyCode::Esc => {
            ui.issue_reporter.close();
            Action::None
        }
        KeyCode::Enter => {
            match ui.issue_reporter.step {
                IssueStep::Steps => {
                    if !ui.issue_reporter.current_step.trim().is_empty() {
                        ui.issue_reporter.add_step();
                    } else {
                        ui.issue_reporter.next_step();
                    }
                    Action::None
                }
                IssueStep::Review => {
                    if ui.issue_reporter.is_complete() {
                        Action::SubmitIssueReport
                    } else {
                        Action::None
                    }
                }
                _ => {
                    ui.issue_reporter.next_step();
                    Action::None
                }
            }
        }
        KeyCode::Char('\n') if ui.issue_reporter.step == IssueStep::Steps => {
            ui.issue_reporter.next_step();
            Action::None
        }
        KeyCode::Backspace => {
            match ui.issue_reporter.step {
                IssueStep::Steps => {
                    if ui.issue_reporter.current_step.is_empty() {
                        ui.issue_reporter.remove_last_step();
                    } else {
                        ui.issue_reporter.pop_char();
                    }
                }
                IssueStep::Review => {
                    ui.issue_reporter.prev_step();
                }
                _ => {
                    ui.issue_reporter.pop_char();
                }
            }
            Action::None
        }
        KeyCode::Char(c) if matches!(ui.issue_reporter.step, IssueStep::Description | IssueStep::Steps | IssueStep::Expected | IssueStep::Actual) => {
            ui.issue_reporter.push_char(c);
            Action::None
        }
        KeyCode::Char(' ') if matches!(ui.issue_reporter.step, IssueStep::Severity | IssueStep::Category) => {
            match ui.issue_reporter.step {
                IssueStep::Severity => ui.issue_reporter.cycle_severity(),
                IssueStep::Category => ui.issue_reporter.cycle_category(),
                _ => {}
            }
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_psychic_menu_input(ui: &mut UiState, state: &mut GameState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc | KeyCode::Char('p') => {
            ui.psychic_menu.close();
            Action::None
        },
        KeyCode::Up | KeyCode::Char('k') => {
            ui.psychic_menu.navigate(-1, state.psychic.unlocked_abilities.len());
            Action::None
        },
        KeyCode::Down | KeyCode::Char('j') => {
            ui.psychic_menu.navigate(1, state.psychic.unlocked_abilities.len());
            Action::None
        },
        KeyCode::Enter => {
            if let Some(ability_id) = ui.psychic_menu.get_selected_ability(state) {
                ui.psychic_menu.close();
                Action::UsePsychicAbility(ability_id)
            } else {
                Action::None
            }
        },
        _ => Action::None,
    }
}

fn handle_skills_menu_input(ui: &mut UiState, state: &mut GameState, code: KeyCode) -> Action {
    use crate::game::skills::{get_skills_by_category, get_abilities_by_category};
    use super::skills_menu::SkillsMenuMode;
    
    match code {
        KeyCode::Esc | KeyCode::Char('s') => {
            ui.skills_menu.active = false;
            Action::None
        },
        KeyCode::Tab => {
            ui.skills_menu.toggle_mode();
            Action::None
        },
        KeyCode::Left | KeyCode::Char('h') => {
            ui.skills_menu.prev_category();
            Action::None
        },
        KeyCode::Right | KeyCode::Char('l') => {
            ui.skills_menu.next_category();
            Action::None
        },
        KeyCode::Up | KeyCode::Char('k') => {
            ui.skills_menu.navigate_up();
            Action::None
        },
        KeyCode::Down | KeyCode::Char('j') => {
            let max_items = match ui.skills_menu.mode {
                SkillsMenuMode::Skills => get_skills_by_category(&ui.skills_menu.selected_category).len(),
                SkillsMenuMode::Abilities => get_abilities_by_category(&ui.skills_menu.selected_category).len(),
            };
            ui.skills_menu.navigate_down(max_items);
            Action::None
        },
        KeyCode::Enter => {
            match ui.skills_menu.mode {
                SkillsMenuMode::Skills => {
                    match ui.skills_menu.upgrade_skill(state) {
                        Ok(()) => {
                            // Success message will be logged by the upgrade function
                        },
                        Err(e) => {
                            state.log(e);
                        }
                    }
                },
                SkillsMenuMode::Abilities => {
                    match ui.skills_menu.use_ability(state) {
                        Ok(()) => {
                            ui.skills_menu.active = false;
                        },
                        Err(e) => {
                            state.log(e);
                        }
                    }
                },
            }
            Action::None
        },
        _ => Action::None,
    }
}
