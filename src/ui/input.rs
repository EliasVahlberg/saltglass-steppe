//! Input handling for game UI

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io::Result;
use crate::GameState;
use super::{InventoryMenu, QuestLogMenu, CraftingMenu, WikiMenu, MenuPanel, WorldMapView};
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
}

impl DebugConsole {
    pub fn toggle(&mut self) { self.active = !self.active; self.input.clear(); }
    pub fn push(&mut self, c: char) { self.input.push(c); }
    pub fn pop(&mut self) { self.input.pop(); }
    pub fn submit(&mut self) -> Option<String> {
        if self.input.is_empty() { return None; }
        let cmd = self.input.clone();
        self.history.push(cmd.clone());
        self.input.clear();
        Some(cmd)
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
    pub world_map_view: WorldMapView,
    pub target_enemy: Option<usize>,
    pub debug_console: DebugConsole,
}

/// Pause menu state
#[derive(Default)]
pub struct PauseMenu {
    pub active: bool,
    pub selected: usize,
}

impl PauseMenu {
    pub fn open(&mut self) { self.active = true; self.selected = 0; }
    pub fn close(&mut self) { self.active = false; }
    pub fn navigate(&mut self, dy: i32, count: usize) {
        self.selected = (self.selected as i32 + dy).rem_euclid(count as i32) as usize;
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
            world_map_view: WorldMapView::default(),
            target_enemy: None,
            debug_console: DebugConsole::default(),
        }
    }
    
    pub fn tick_frame(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
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
    AutoExplore,
    RangedAttack(i32, i32),
    OpenInventory,
    EquipSelected,
    UnequipSelected,
    OpenQuestLog,
    OpenCrafting,
    OpenWiki,
    OpenWorldMap,
    Craft,
    OpenPauseMenu,
    ReturnToMainMenu,
    SetTarget(i32, i32),
    UseStairs,
    DebugCommand(String),
    None,
}

/// Handle input and return the resulting action
pub fn handle_input(ui: &mut UiState, state: &GameState) -> Result<Action> {
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
        
        // Debug console input
        if ui.debug_console.active {
            return Ok(handle_debug_console_input(ui, key.code));
        }
        // Pause menu input
        if ui.pause_menu.active {
            return Ok(handle_pause_menu_input(ui, key.code));
        }
        // World map view input
        if ui.world_map_view.open {
            return Ok(handle_world_map_input(ui, state, key.code));
        }
        // Wiki menu input
        if ui.wiki_menu.active {
            return Ok(handle_wiki_input(ui, state, key.code));
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

fn handle_wiki_input(ui: &mut UiState, _state: &GameState, code: KeyCode) -> Action {
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

fn handle_quest_log_input(ui: &mut UiState, state: &GameState, code: KeyCode) -> Action {
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

fn handle_inventory_input(ui: &mut UiState, state: &GameState, code: KeyCode) -> Action {
    match code {
        KeyCode::Esc | KeyCode::Char('i') => ui.inventory_menu.close(),
        KeyCode::Char('j') | KeyCode::Down => ui.inventory_menu.navigate(1, state.inventory.len()),
        KeyCode::Char('k') | KeyCode::Up => ui.inventory_menu.navigate(-1, state.inventory.len()),
        KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left | KeyCode::Right => ui.inventory_menu.switch_panel(),
        KeyCode::Char('x') => ui.inventory_menu.inspect(&state.inventory, &state.equipment),
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
            let selected = ui.pause_menu.selected;
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
                return Action::DebugCommand(cmd);
            }
        }
        KeyCode::Backspace => { ui.debug_console.pop(); }
        KeyCode::Char(c) => ui.debug_console.push(c),
        _ => {}
    }
    Action::None
}

fn handle_world_map_input(ui: &mut UiState, state: &GameState, code: KeyCode) -> Action {
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
        _ => {}
    }
    Action::None
}

fn handle_game_input(ui: &mut UiState, code: KeyCode) -> Action {
    match code {
        KeyCode::Char('`') => { ui.debug_console.toggle(); Action::None }
        KeyCode::Char('S') => Action::Save,
        KeyCode::Char('L') => Action::Load,
        KeyCode::Char('x') => Action::EnterLook,
        KeyCode::Char('e') => Action::EndTurn,
        KeyCode::Char('o') => Action::AutoExplore,
        KeyCode::Char('i') => Action::OpenInventory,
        KeyCode::Char('q') => Action::OpenQuestLog,
        KeyCode::Char('c') => Action::OpenCrafting,
        KeyCode::Char('w') => Action::OpenWiki,
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
