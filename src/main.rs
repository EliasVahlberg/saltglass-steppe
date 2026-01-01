use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use std::io::{stdout, Result};
use saltglass_steppe::{get_item_def, GameState, Renderer};
use saltglass_steppe::ui::{render_inventory_menu, render_quest_log, render_crafting_menu, render_wiki, render_psychic_menu, render_skills_menu, render_side_panel, render_bottom_panel, render_target_hud, handle_input, Action, UiState, handle_menu_input, render_menu, render_controls, render_pause_menu, render_debug_console, render_debug_menu, render_issue_reporter, render_dialog_box, render_book_reader, render_chest_ui, MenuAction, MainMenuState, render_damage_numbers, render_death_screen};
use saltglass_steppe::cli::{parse_args, LaunchMode};
use saltglass_steppe::satellite::SatelliteApp;

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
                ui.inventory_menu.close();
            }
        }
        Action::Move(dx, dy) => {
            if state.player_hp > 0 {
                let new_x = state.player_x + dx;
                let new_y = state.player_y + dy;
                if let Some(ei) = state.enemy_at(new_x, new_y) {
                    ui.target_enemy = Some(ei);
                }
                state.try_move(dx, dy);
            }
        }
        Action::EndTurn => {
            if state.player_hp > 0 {
                state.end_turn();
            }
        }
        Action::Wait => {
            if state.player_hp > 0 {
                state.wait_turn();
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
                        saltglass_steppe::Tile::StairsDown => { state.enter_subterranean(); }
                        saltglass_steppe::Tile::StairsUp => { state.exit_subterranean(); }
                        saltglass_steppe::Tile::WorldExit => {
                            // Simple world map travel - for now just show a message
                            // TODO: Add proper world map UI
                            state.log("Use arrow keys to choose direction, then press > again.");
                        }
                        _ => { state.log("No stairs here."); }
                    }
                }
            }
        }
        Action::TradeBuy(idx) => {
            if let Some(interface) = &mut ui.trade_menu.interface {
                if let Some(item) = interface.available_items.get(idx) {
                    use saltglass_steppe::trading::execute_trade;
                    match execute_trade(interface, &item.item_id.clone(), 1, &mut state.salt_scrip, &mut state.inventory) {
                        Ok(msg) => state.log_typed(msg, saltglass_steppe::MsgType::Social),
                        Err(e) => state.log(e),
                    }
                }
            }
        }
        Action::TradeSell(idx) => {
            if let Some(interface) = &ui.trade_menu.interface {
                if let Some(item_id) = state.inventory.get(idx) {
                    use saltglass_steppe::trading::execute_sell;
                    match execute_sell(interface, &item_id.clone(), 1, &mut state.salt_scrip, &mut state.inventory) {
                        Ok(msg) => state.log_typed(msg, saltglass_steppe::MsgType::Social),
                        Err(e) => state.log(e),
                    }
                }
            }
        }
        Action::DebugCommand(cmd) => {
            state.debug_command(&cmd);
        }
        Action::SubmitIssueReport => {
            let report = state.create_issue_report(
                ui.issue_reporter.description.clone(),
                ui.issue_reporter.steps.clone(),
                ui.issue_reporter.expected.clone(),
                ui.issue_reporter.actual.clone(),
                ui.issue_reporter.severity.clone(),
                ui.issue_reporter.category.clone(),
            );
            match state.save_issue_report(&report) {
                Ok(_) => {
                    state.log(format!("Issue report saved: {}", report.id));
                    ui.issue_reporter.close();
                }
                Err(e) => state.log(format!("Failed to save issue report: {}", e)),
            }
        }
        Action::OpenDebugMenu => {
            ui.debug_menu.toggle();
        }
        Action::OpenIssueReporter => {
            ui.issue_reporter.open();
        }
        Action::OpenInventory => {
            ui.inventory_menu.open();
        }
        Action::EquipSelected => {
            if let Some(idx) = ui.inventory_menu.selected_inv_index() {
                if idx < state.inventory.len() {
                    if let Some(def) = get_item_def(&state.inventory[idx]) {
                        if let Some(slot_str) = &def.equip_slot {
                            if let Ok(slot) = slot_str.parse::<saltglass_steppe::EquipSlot>() {
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
        Action::OpenChest(_) => {
            // Check if player is standing on a chest
            if let Some(&chest_idx) = state.chest_positions.get(&(state.player_x, state.player_y)) {
                if state.open_chest(chest_idx) {
                    ui.chest_ui = Some(saltglass_steppe::ui::ChestUI::new(chest_idx));
                }
            } else {
                state.log("No chest here.");
            }
        }
        Action::ChestTransfer => {
            if let Some(ref chest_ui) = ui.chest_ui {
                let chest_index = chest_ui.chest_index;
                if let Some(chest_item_idx) = chest_ui.get_selected_chest_item() {
                    state.transfer_from_chest(chest_index, chest_item_idx);
                } else if let Some(inv_item_idx) = chest_ui.get_selected_inventory_item() {
                    state.transfer_to_chest(chest_index, inv_item_idx);
                }
            }
        }
        Action::CloseChest => {
            ui.chest_ui = None;
        }
        Action::OpenWiki => {
            ui.wiki_menu.open();
        }
        Action::OpenPsychicMenu => {
            ui.psychic_menu.toggle();
        }
        Action::OpenSkillsMenu => {
            ui.skills_menu.open();
        }
        Action::UsePsychicAbility(ability_id) => {
            state.use_psychic_ability(&ability_id);
        }
        Action::RangedAttackMode => {
            // TODO: Implement ranged attack mode
        }
        Action::TargetMode => {
            // TODO: Implement targeting mode
        }
        Action::OpenWorldMap => {
            ui.world_map_view.toggle(state.world_x, state.world_y);
        }
        Action::WorldMapTravel(wx, wy) => {
            if state.player_hp > 0 && state.layer == 0 {
                state.travel_to_tile_safe(wx, wy);
            }
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

fn render(frame: &mut Frame, state: &GameState, ui: &mut UiState, renderer: &mut Renderer) {
    // Fullscreen menus
    if ui.trade_menu.active {
        use saltglass_steppe::ui::render_trade_menu;
        render_trade_menu(frame, &ui.trade_menu, state);
        return;
    }
    if ui.inventory_menu.active {
        render_inventory_menu(frame, &ui.inventory_menu, &state.inventory, &state.equipment);
        return;
    }
    if let Some(ref mut chest_ui) = ui.chest_ui {
        if chest_ui.chest_index < state.chests.len() {
            render_chest_ui(frame, frame.area(), &state.chests[chest_ui.chest_index], &state.inventory, chest_ui);
        }
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
    if ui.psychic_menu.active {
        render_psychic_menu(frame, frame.area(), state, &ui.psychic_menu);
        return;
    }
    if ui.skills_menu.active {
        render_skills_menu(frame, state, &ui.skills_menu);
        return;
    }
    if ui.world_map_view.open {
        if let Some(ref world_map) = state.world_map {
            saltglass_steppe::ui::render_world_map(frame, frame.area(), world_map, state.world_x, state.world_y, &ui.world_map_view);
        }
        return;
    }

    // Death screen
    if state.player_hp <= 0 {
        render_death_screen(frame, state);
        return;
    }

    // Main layout: side panel + game area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(state.map.width as u16 + 2),
            Constraint::Min(22),
        ])
        .split(frame.area());

    // Left side: game area with look mode and map
    let desc_height = if ui.look_mode.active { 3u16 } else { 0 };
    let game_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(desc_height),
            Constraint::Min(20),
            Constraint::Length(7),
        ])
        .split(main_chunks[0]);

    // Look mode description box
    if ui.look_mode.active {
        let desc = state.describe_at(ui.look_mode.x, ui.look_mode.y);
        let block = Block::default().title(" Look (Esc/Enter to exit) ").borders(Borders::ALL);
        frame.render_widget(Paragraph::new(desc).wrap(ratatui::widgets::Wrap { trim: true }).block(block), game_chunks[0]);
    }

    // Render game map using new modular renderer
    let look_cursor = if ui.look_mode.active { Some((ui.look_mode.x, ui.look_mode.y)) } else { None };
    renderer.render_game(frame, game_chunks[1], state, ui.frame_count, look_cursor);
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
        render_pause_menu(frame, ui.pause_menu.selected_index);
    }
    
    // Debug console overlay
    if ui.debug_console.active {
        render_debug_console(frame, &ui.debug_console);
    }
    
    // Debug menu overlay
    if ui.debug_menu.active {
        render_debug_menu(frame, &ui.debug_menu, state);
    }
    
    // Issue reporter overlay
    if ui.issue_reporter.active {
        render_issue_reporter(frame, &ui.issue_reporter);
    }
    
    // Dialog box overlay (highest priority)
    render_dialog_box(frame, &ui.dialog_box);
    
    // Book reader overlay
    render_book_reader(frame, ui);
}

fn main() -> Result<()> {
    let launch_mode = parse_args();
    
    match launch_mode {
        LaunchMode::MainGame => run_main_game(),
        LaunchMode::LogUi => run_satellite_ui("log-ui"),
        LaunchMode::GameLogUi => run_satellite_ui("game-log-ui"),
        LaunchMode::StatusUi => run_satellite_ui("status-ui"),
        LaunchMode::InventoryUi => run_satellite_ui("inventory-ui"),
        LaunchMode::DebugUi => run_satellite_ui("debug-ui"),
    }
}

fn run_satellite_ui(ui_type: &str) -> Result<()> {
    let socket_path = "/tmp/saltglass-steppe.sock";
    let mut app = match SatelliteApp::new(socket_path) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to connect to main game: {}", e);
            eprintln!("Make sure the main game is running first.");
            return Err(e);
        }
    };
    
    match ui_type {
        "log-ui" => app.run_log_ui(),
        "game-log-ui" => app.run_game_log_ui(),
        "status-ui" => app.run_status_ui(),
        "inventory-ui" => app.run_inventory_ui(),
        "debug-ui" => app.run_debug_ui(),
        _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unknown UI type")),
    }
}

fn run_main_game() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Initialize IPC server
    use saltglass_steppe::ipc::{IpcServer, IpcMessage};
    let socket_path = "/tmp/saltglass-steppe.sock";
    let ipc_server = IpcServer::new(socket_path)?;
    ipc_server.start()?;
    let mut last_message_count = 0;

    // Initialize the new modular renderer
    let mut renderer = match Renderer::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize renderer: {}", e);
            disable_raw_mode()?;
            stdout().execute(LeaveAlternateScreen)?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    };

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
        // Initialize camera to player position
        ui.camera_x = state.player_x as f32;
        ui.camera_y = state.player_y as f32;

        loop {
            ui.tick_frame();
            state.tick_hit_flash();
            state.tick_damage_numbers();
            state.tick_projectile_trails();
            state.tick_light_beams();
            state.tick_animation();
            ui.update_camera(state.player_x, state.player_y);
            ui.dialog_box.tick(16); // ~60fps
            
            // Check for pending dialogue from NPC interaction
            if let Some((speaker, text)) = state.pending_dialogue.take() {
                ui.dialog_box.show(&speaker, &text);
            }
            
            // Check for pending book open
            if let Some(book_id) = state.pending_book_open.take() {
                ui.book_reader.open(&book_id);
            }
            
            // Check for pending trade (only if no dialog is active)
            if let Some(trader_id) = state.pending_trade.take() {
                if ui.dialog_box.active {
                    // Put the trade back if dialog is still active
                    state.pending_trade = Some(trader_id);
                } else {
                    use saltglass_steppe::trading::{get_trade_interface, calculate_area_tier};
                    let area_tier = calculate_area_tier(&state.enemies);
                    if let Some(interface) = get_trade_interface(
                        &trader_id, 
                        area_tier, 
                        &state.faction_reputation, 
                        None // Player faction not yet implemented
                    ) {
                        // Close other menus to ensure trade menu has focus
                        ui.inventory_menu.close();
                        ui.quest_log.close();
                        ui.crafting_menu.close();
                        ui.wiki_menu.close();
                        ui.pause_menu.close();
                        ui.trade_menu.open(trader_id, interface);
                    } else {
                        state.log("This merchant has nothing to trade.");
                    }
                }
            }
            
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
                terminal.draw(|frame| render(frame, &state, &mut ui, &mut renderer))?;
                let action = handle_input(&mut ui, &mut state)?;
                match update(&mut state, action, &mut ui) {
                    Some(true) => {
                        // Send game state update to satellite terminals
                        let adaptations: Vec<String> = state.adaptations.iter()
                            .map(|a| a.name().to_string())
                            .collect();
                        
                        let _ = ipc_server.send_message(IpcMessage::GameState {
                            hp: state.player_hp,
                            max_hp: state.player_max_hp,
                            refraction: state.refraction as i32,
                            turn: state.turn as u32,
                            storm_countdown: state.storm.turns_until as i32,
                            adaptations,
                            god_view: state.debug_god_view,
                            phase_mode: state.debug_phase,
                        });
                        
                        // Send inventory update
                        let equipped_items: Vec<String> = [
                            ("Weapon", &state.equipment.weapon),
                            ("Ranged", &state.equipment.ranged_weapon),
                            ("Head", &state.equipment.head),
                            ("Jacket", &state.equipment.jacket),
                            ("Pants", &state.equipment.pants),
                            ("Boots", &state.equipment.boots),
                            ("Gloves", &state.equipment.gloves),
                            ("L.Wrist", &state.equipment.left_wrist),
                            ("R.Wrist", &state.equipment.right_wrist),
                            ("Necklace", &state.equipment.necklace),
                            ("Accessory", &state.equipment.accessory),
                            ("Backpack", &state.equipment.backpack),
                        ]
                        .iter()
                        .filter_map(|(slot, item)| {
                            item.as_ref().map(|i| format!("{}: {}", slot, i))
                        })
                        .collect();
                        
                        let _ = ipc_server.send_message(IpcMessage::InventoryUpdate {
                            items: state.inventory.clone(),
                            equipped: equipped_items,
                        });
                        
                        // Send new log messages only
                        if state.messages.len() > last_message_count {
                            for message in &state.messages[last_message_count..] {
                                let _ = ipc_server.send_message(IpcMessage::LogEntry {
                                    message: message.text.clone(),
                                    msg_type: format!("{:?}", message.msg_type),
                                    turn: message.turn,
                                });
                            }
                            last_message_count = state.messages.len();
                        }
                        
                        // Send debug info update
                        let _ = ipc_server.send_message(IpcMessage::DebugInfo {
                            player_pos: (state.player_x, state.player_y),
                            enemies_count: state.enemies.len(),
                            items_count: state.inventory.len(),
                            storm_intensity: state.storm.intensity as i32,
                            seed: state.seed,
                            god_view: state.debug_god_view,
                            phase_mode: state.debug_phase,
                        });
                        
                        // Handle incoming commands from debug terminal
                        while let Some(message) = ipc_server.try_recv_message() {
                            if let IpcMessage::Command { action } = message {
                                state.debug_command(&action);
                            }
                        }
                    }
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
