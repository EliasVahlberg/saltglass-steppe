use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use std::io::{stdout, Result};
use tui_rpg::{get_active_effects, get_item_def, EffectContext, GameState, Tile, MAP_HEIGHT};
use tui_rpg::ui::{render_inventory_menu, render_quest_log, render_crafting_menu, render_wiki, render_side_panel, render_bottom_panel, render_target_hud, handle_input, Action, UiState, handle_menu_input, render_menu, render_controls, render_pause_menu, render_debug_console, MenuAction, MainMenuState, render_map, render_death_screen, render_damage_numbers};

const SAVE_FILE: &str = "savegame.ron";

fn update(state: &mut GameState, action: Action, ui: &mut UiState) -> Option<bool> {
    match action {
        Action::Quit => return Some(false),
        Action::ReturnToMainMenu => return None, // Signal to return to main menu
        Action::OpenPauseMenu => ui.pause_menu.open(),
        Action::OpenControls => ui.show_controls = true,
        Action::EnterLook => {
            ui.look_mode.active = true;
            ui.look_mode.x = state.player_x;
            ui.look_mode.y = state.player_y;
        }
        Action::BreakWall(x, y) => {
            if state.player_hp > 0 {
                state.try_break_wall(x, y);
            }
        }
        Action::Save => {
            match state.save(SAVE_FILE) {
                Ok(_) => state.log("Game saved."),
                Err(e) => state.log(format!("Save failed: {}", e)),
            }
        }
        Action::Load => {
            match GameState::load(SAVE_FILE) {
                Ok(loaded) => {
                    *state = loaded;
                    state.log("Game loaded.");
                }
                Err(e) => state.log(format!("Load failed: {}", e)),
            }
        }
        Action::UseItem(idx) => {
            if state.player_hp > 0 {
                state.use_item(idx);
            }
        }
        Action::Move(dx, dy) => {
            if state.player_hp > 0 {
                // Check if on world exit - travel to adjacent world tile
                if let Some(tile) = state.map.get(state.player_x, state.player_y) {
                    if *tile == tui_rpg::Tile::WorldExit && state.layer == 0 {
                        let new_wx = (state.world_x as i32 + dx).max(0) as usize;
                        let new_wy = (state.world_y as i32 + dy).max(0) as usize;
                        if new_wx != state.world_x || new_wy != state.world_y {
                            state.travel_to_tile(new_wx, new_wy);
                        }
                    } else {
                        let new_x = state.player_x + dx;
                        let new_y = state.player_y + dy;
                        if let Some(ei) = state.enemy_at(new_x, new_y) {
                            ui.target_enemy = Some(ei);
                        }
                        state.try_move(dx, dy);
                    }
                } else {
                    let new_x = state.player_x + dx;
                    let new_y = state.player_y + dy;
                    if let Some(ei) = state.enemy_at(new_x, new_y) {
                        ui.target_enemy = Some(ei);
                    }
                    state.try_move(dx, dy);
                }
            }
        }
        Action::EndTurn => {
            if state.player_hp > 0 {
                state.end_turn();
            }
        }
        Action::AutoExplore => {
            if state.player_hp > 0 {
                state.auto_explore();
            }
        }
        Action::RangedAttack(x, y) => {
            if state.player_hp > 0 {
                // Auto-target enemy when attacking
                if let Some(ei) = state.enemy_at(x, y) {
                    ui.target_enemy = Some(ei);
                }
                state.try_ranged_attack(x, y);
            }
        }
        Action::SetTarget(x, y) => {
            ui.target_enemy = state.enemy_at(x, y);
        }
        Action::UseStairs => {
            if state.player_hp > 0 {
                // Check what tile we're standing on
                if let Some(tile) = state.map.get(state.player_x, state.player_y) {
                    match tile {
                        tui_rpg::Tile::StairsDown => { state.enter_subterranean(); }
                        tui_rpg::Tile::StairsUp => { state.exit_subterranean(); }
                        tui_rpg::Tile::WorldExit => {
                            // Simple world map travel - for now just show a message
                            // TODO: Add proper world map UI
                            state.log("Use arrow keys to choose direction, then press > again.");
                        }
                        _ => { state.log("No stairs here."); }
                    }
                }
            }
        }
        Action::DebugCommand(cmd) => {
            state.debug_command(&cmd);
        }
        Action::OpenInventory => {
            ui.inventory_menu.open();
        }
        Action::EquipSelected => {
            if let Some(idx) = ui.inventory_menu.selected_inv_index() {
                if idx < state.inventory.len() {
                    if let Some(def) = get_item_def(&state.inventory[idx]) {
                        if let Some(slot_str) = &def.equip_slot {
                            if let Ok(slot) = slot_str.parse::<tui_rpg::EquipSlot>() {
                                state.equip_item(idx, slot);
                            }
                        }
                    }
                }
            }
        }
        Action::UnequipSelected => {
            if let Some(slot) = ui.inventory_menu.selected_equip_slot() {
                state.unequip_slot(slot);
            }
        }
        Action::OpenQuestLog => {
            ui.quest_log.open();
        }
        Action::OpenCrafting => {
            ui.crafting_menu.open();
        }
        Action::OpenWiki => {
            ui.wiki_menu.open();
        }
        Action::Craft => {
            if let Some(recipe_id) = ui.crafting_menu.selected_recipe_id() {
                state.craft(recipe_id);
            }
        }
        Action::None => {}
    }
    Some(true)
}

fn render(frame: &mut Frame, state: &GameState, ui: &UiState) {
    // Fullscreen menus
    if ui.inventory_menu.active {
        render_inventory_menu(frame, &ui.inventory_menu, &state.inventory, &state.equipment);
        return;
    }
    if ui.quest_log.active {
        render_quest_log(frame, &ui.quest_log, state);
        return;
    }
    if ui.crafting_menu.active {
        render_crafting_menu(frame, &ui.crafting_menu, state);
        return;
    }
    if ui.wiki_menu.active {
        render_wiki(frame, &ui.wiki_menu, &state.meta);
        return;
    }

    // Death screen
    if state.player_hp <= 0 {
        render_death_screen(frame, state);
        return;
    }

    // Build effect context for player visual effects
    let player_idx = state.map.idx(state.player_x, state.player_y);
    let on_glass = state.map.tiles.get(player_idx).map(|t| *t == Tile::Glass).unwrap_or(false);
    let effect_ctx = EffectContext {
        player_hp: state.player_hp,
        storm_turns: state.storm.turns_until,
        has_adaptation: !state.adaptations.is_empty(),
        on_glass,
        adaptations_hidden: state.adaptations_hidden_turns > 0,
    };
    let mut player_effects = get_active_effects(&effect_ctx, "player");
    for te in &state.triggered_effects {
        if let Some(effect) = tui_rpg::parse_effect(&te.effect) {
            player_effects.push(effect);
        }
    }

    // Main layout: side panel + game area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(state.map.width as u16 + 2),
            Constraint::Length(22),
        ])
        .split(frame.area());

    // Left side: game area with look mode and map
    let desc_height = if ui.look_mode.active { 3u16 } else { 0 };
    let game_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(desc_height),
            Constraint::Min(MAP_HEIGHT as u16 + 2),
            Constraint::Length(7),
        ])
        .split(main_chunks[0]);

    // Look mode description box
    if ui.look_mode.active {
        let desc = state.describe_at(ui.look_mode.x, ui.look_mode.y);
        let block = Block::default().title(" Look (Esc/Enter to exit) ").borders(Borders::ALL);
        frame.render_widget(Paragraph::new(desc).block(block), game_chunks[0]);
    }

    // Render game map
    let look_cursor = if ui.look_mode.active { Some((ui.look_mode.x, ui.look_mode.y)) } else { None };
    render_map(frame, game_chunks[1], state, &player_effects, ui.frame_count, look_cursor);
    render_damage_numbers(frame, game_chunks[1], state);

    // Bottom panel with log
    render_bottom_panel(frame, game_chunks[2], state);

    // Right side panel with stats
    render_side_panel(frame, main_chunks[1], state);
    
    // Target HUD (bottom left)
    if let Some(target_idx) = ui.target_enemy {
        render_target_hud(frame, state, target_idx);
    }
    
    // Pause menu overlay (rendered last)
    if ui.pause_menu.active {
        render_pause_menu(frame, ui.pause_menu.selected);
    }
    
    // Debug console overlay
    if ui.debug_console.active {
        render_debug_console(frame, &ui.debug_console);
    }
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    'main: loop {
        // Main menu loop
        let mut menu_state = MainMenuState::new();
        let mut menu_tick: u64 = 0;
        let class_id = loop {
            terminal.draw(|f| render_menu(f, menu_tick, &menu_state))?;
            menu_tick = menu_tick.wrapping_add(1);
            match handle_menu_input(&mut menu_state)? {
                MenuAction::NewGame(class) => break class,
                MenuAction::Controls => {
                    // Show controls screen
                    loop {
                        terminal.draw(render_controls)?;
                        if event::poll(std::time::Duration::from_millis(16))? {
                            if let Event::Key(key) = event::read()? {
                                if key.kind == KeyEventKind::Press { break; }
                            }
                        }
                    }
                }
                MenuAction::Quit => {
                    disable_raw_mode()?;
                    stdout().execute(LeaveAlternateScreen)?;
                    return Ok(());
                }
                MenuAction::LoadGame(_) | MenuAction::None => {}
            }
        };

        // Create game with selected class
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut state = GameState::new_with_class(seed, &class_id);
        let mut ui = UiState::new();

        loop {
            ui.tick_frame();
            state.tick_hit_flash();
            state.tick_damage_numbers();
            state.tick_projectile_trails();
            
            // Clear target if enemy is dead
            if let Some(ei) = ui.target_enemy {
                if ei >= state.enemies.len() || state.enemies[ei].hp <= 0 {
                    ui.target_enemy = None;
                }
            }
            
            if ui.show_controls {
                terminal.draw(render_controls)?;
                if event::poll(std::time::Duration::from_millis(16))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            ui.show_controls = false;
                        }
                    }
                }
            } else {
                terminal.draw(|frame| render(frame, &state, &ui))?;
                let action = handle_input(&mut ui, &state)?;
                match update(&mut state, action, &mut ui) {
                    Some(true) => {} // Continue game
                    Some(false) => break 'main, // Quit
                    None => break, // Return to main menu
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
