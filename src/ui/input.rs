//! Input handling for game UI

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io::Result;
use crate::GameState;
use super::{InventoryMenu, QuestLogMenu, CraftingMenu, MenuPanel};
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

/// UI-specific state, separate from game logic
pub struct UiState {
    pub look_mode: LookMode,
    pub frame_count: u64,
    pub show_controls: bool,
    pub inventory_menu: InventoryMenu,
    pub quest_log: QuestLogMenu,
    pub crafting_menu: CraftingMenu,
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
    Craft,
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
        return Ok(handle_game_input(key.code));
    }
    Ok(Action::None)
}

fn handle_quest_log_input(ui: &mut UiState, state: &GameState, code: KeyCode) -> Action {
    let total = state.quest_log.active.len() + state.quest_log.completed.len() + 3;
    match code {
        KeyCode::Esc | KeyCode::Char('Q') => ui.quest_log.close(),
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
        _ => {}
    }
    Action::None
}

fn handle_game_input(code: KeyCode) -> Action {
    match code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('S') => Action::Save,
        KeyCode::Char('L') => Action::Load,
        KeyCode::Char('x') => Action::EnterLook,
        KeyCode::Char('e') => Action::EndTurn,
        KeyCode::Char('o') => Action::AutoExplore,
        KeyCode::Char('i') => Action::OpenInventory,
        KeyCode::Char('Q') => Action::OpenQuestLog,
        KeyCode::Char('c') => Action::OpenCrafting,
        KeyCode::Char('1') => Action::UseItem(0),
        KeyCode::Char('2') => Action::UseItem(1),
        KeyCode::Char('3') => Action::UseItem(2),
        KeyCode::Up | KeyCode::Char('k') => Action::Move(0, -1),
        KeyCode::Down | KeyCode::Char('j') => Action::Move(0, 1),
        KeyCode::Left | KeyCode::Char('h') => Action::Move(-1, 0),
        KeyCode::Right | KeyCode::Char('l') => Action::Move(1, 0),
        KeyCode::Esc => Action::OpenControls,
        _ => Action::None,
    }
}
