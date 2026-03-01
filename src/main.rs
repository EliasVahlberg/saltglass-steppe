use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use saltglass_steppe::cli::{LaunchMode, parse_args};
use saltglass_steppe::satellite::SatelliteApp;
use saltglass_steppe::ui::{
    Action, MainMenuState, MenuAction, UiState, handle_input, handle_menu_input,
    render_book_reader, render_bottom_panel, render_chest_ui, render_controls,
    render_crafting_menu, render_damage_numbers, render_death_screen, render_debug_console,
    render_debug_menu, render_dialog_box, render_crystal_menu, render_faction_menu, render_inventory_menu, render_issue_reporter,
    render_light_menu, render_menu, render_pause_menu, render_psychic_menu, render_quest_log, render_side_panel,
    render_skills_menu, render_target_hud, render_void_menu, render_wiki,
};
use saltglass_steppe::{GameState, Renderer, get_item_def};
use saltglass_steppe::game::save;
use std::io::{Result, stdout};

const SAVE_FILE: &str = "savegame.ron";

fn update(state: &mut GameState, action: Action, ui: &mut UiState) -> Option<bool> {
    match action {
        Action::Quit => return Some(false),
        Action::ReturnToMainMenu => return None, // Signal to return to main menu
        Action::OpenPauseMenu => ui.pause_menu.open(),
        Action::OpenControls => ui.show_controls = true,
        Action::EnterLook => {
            ui.look_mode.active = true;
            ui.look_mode.x = state.player.x;
            ui.look_mode.y = state.player.y;
        }
        Action::BreakWall(x, y) => {
            if state.player.hp > 0 {
                state.try_break_wall(x, y);
            }
        }
        Action::Save => match save::save_game(state, SAVE_FILE) {
            Ok(_) => state.log("Game saved."),
            Err(e) => state.log(format!("Save failed: {}", e)),
        },
        Action::Load => match save::load_game(SAVE_FILE) {
            Ok(loaded) => {
                *state = loaded;
                state.log("Game loaded.");
            }
            Err(e) => state.log(format!("Load failed: {}", e)),
        },
        Action::UseItem(idx) => {
            if state.player.hp > 0 {
                state.use_item(idx);
                ui.inventory_menu.close();
            }
        }
        Action::Move(dx, dy) => {
            if state.player.hp > 0 {
                let new_x = state.player.x + dx;
                let new_y = state.player.y + dy;
                if let Some(ei) = state.enemy_at(new_x, new_y) {
                    ui.target_enemy = Some(ei);
                }
                state.try_move(dx, dy);
            }
        }
        Action::EndTurn => {
            if state.player.hp > 0 {
                state.end_turn();
            }
        }
        Action::Wait => {
            if state.player.hp > 0 {
                state.wait_turn();
            }
        }
        Action::AutoExplore => {
            if state.player.hp > 0 {
                state.auto_explore();
            }
        }
        Action::RangedAttack(x, y) => {
            if state.player.hp > 0 {
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
            if state.player.hp > 0 {
                // Check what tile we're standing on
                if let Some(tile) = state.world.map.get(state.player.x, state.player.y) {
                    match tile {
                        saltglass_steppe::Tile::StairsDown => {
                            state.enter_subterranean();
                        }
                        saltglass_steppe::Tile::StairsUp => {
                            state.exit_subterranean();
                        }
                        saltglass_steppe::Tile::WorldExit => {
                            // Simple world map travel - for now just show a message
                            // TODO: Add proper world map UI
                            state.log("Use arrow keys to choose direction, then press > again.");
                        }
                        _ => {
                            state.log("No stairs here.");
                        }
                    }
                }
            }
        }
        Action::TradeBuy(idx) => {
            if let Some(interface) = &mut ui.trade_menu.interface {
                if let Some(item) = interface.available_items.get(idx) {
                    let trader_id = interface.trader_id.clone();
                    use saltglass_steppe::trading::execute_trade;
                    match execute_trade(
                        interface,
                        &item.item_id.clone(),
                        1,
                        &mut state.player.salt_scrip,
                        &mut state.player.inventory,
                    ) {
                        Ok(msg) => {
                            state.log_typed(msg, saltglass_steppe::MsgType::Social);
                            state.emit(saltglass_steppe::event::GameEvent::TradeCompleted { npc_id: trader_id });
                        }
                        Err(e) => state.log(e),
                    }
                }
            }
        }
        Action::TradeSell(idx) => {
            if let Some(interface) = &ui.trade_menu.interface {
                if let Some(item_id) = state.player.inventory.get(idx) {
                    let trader_id = interface.trader_id.clone();
                    use saltglass_steppe::trading::execute_sell;
                    match execute_sell(
                        interface,
                        &item_id.clone(),
                        1,
                        &mut state.player.salt_scrip,
                        &mut state.player.inventory,
                    ) {
                        Ok(msg) => {
                            state.log_typed(msg, saltglass_steppe::MsgType::Social);
                            state.emit(saltglass_steppe::event::GameEvent::TradeCompleted { npc_id: trader_id });
                        }
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
                if idx < state.player.inventory.len() {
                    if let Some(def) = get_item_def(&state.player.inventory[idx]) {
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
            if let Some(&chest_idx) = state.chest_positions.get(&(state.player.x, state.player.y)) {
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
        Action::Interact(_, _) => {
            if state.player.hp > 0 {
                state.interact_at(state.player.x, state.player.y);
            }
        }
        Action::Examine(_, _) => {
            if state.player.hp > 0 {
                state.examine_at(state.player.x, state.player.y);
            }
        }
        Action::OpenWiki => {
            ui.wiki_menu.open();
        }
        Action::OpenPsychicMenu => {
            ui.psychic_menu.toggle();
        }
        Action::OpenFactionMenu => {
            ui.faction_menu.toggle();
        }
        Action::OpenVoidMenu => {
            ui.void_menu.toggle();
        }
        Action::OpenCrystalMenu => {
            ui.crystal_menu.toggle();
        }
        Action::OpenLightMenu => {
            ui.light_menu.toggle();
        }
        Action::UseVoidAbility => {
            if let Some(ability) = saltglass_steppe::ui::void_menu::get_selected_ability(&ui.void_menu, state) {
                state.player.void_system.use_ability(ability);
            }
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
            ui.world_map_view.toggle(state.world.world_x, state.world.world_y);
        }
        Action::WorldMapTravel(wx, wy) => {
            if state.player.hp > 0 && state.player.layer == 0 {
                // Block travel during encounters
                if state.world.encounter_state.is_some() {
                    state.log("You cannot travel while in an encounter!");
                } else {
                    state.travel_to_tile_safe(wx, wy);
                }
            }
        }
        Action::WorldMapMove(dx, dy) => {
            if state.player.hp > 0 && state.player.layer == 0 {
                let new_wx = (state.world.world_x as i32 + dx).clamp(0, 191) as usize;
                let new_wy = (state.world.world_y as i32 + dy).clamp(0, 63) as usize;
                
                if new_wx != state.world.world_x || new_wy != state.world.world_y {
                    // Clear path on manual movement
                    state.world.world_map_path.clear();
                    state.world.world_map_target = None;
                    
                    // Use fast worldmap movement
                    if let Some(encounter_msg) = state.move_on_world_map(new_wx, new_wy) {
                        // Encounter triggered - show popup and close worldmap
                        ui.dialog_box.show("Encounter!", &encounter_msg);
                        ui.world_map_view.open = false;
                    }
                }
            }
        }
        Action::WorldMapAutoMove => {
            if state.player.hp > 0 && state.player.layer == 0 {
                match state.move_along_path() {
                    Ok(true) => {
                        // Check if encounter triggered during auto-move
                        if state.world.encounter_state.is_some() {
                            ui.world_map_view.open = false;
                        }
                    }
                    Ok(false) => state.log("No path set."),
                    Err(msg) => state.log(&msg),
                }
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
    if ui.aria_interface.response_text.len() > 0 {
        ui.aria_interface.render(frame, frame.area());
        return;
    }
    if ui.trade_menu.active {
        use saltglass_steppe::ui::render_trade_menu;
        render_trade_menu(frame, &ui.trade_menu, state);
        return;
    }
    if ui.inventory_menu.active {
        render_inventory_menu(
            frame,
            &ui.inventory_menu,
            &state.player.inventory,
            &state.player.equipment,
        );
        return;
    }
    if let Some(ref mut chest_ui) = ui.chest_ui {
        if chest_ui.chest_index < state.world.chests.len() {
            render_chest_ui(
                frame,
                frame.area(),
                &state.world.chests[chest_ui.chest_index],
                &state.player.inventory,
                chest_ui,
            );
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
    if ui.faction_menu.active {
        render_faction_menu(frame, frame.area(), state, &ui.faction_menu);
        return;
    }
    if ui.void_menu.active {
        render_void_menu(frame, frame.area(), state, &ui.void_menu);
        return;
    }
    if ui.crystal_menu.active {
        render_crystal_menu(frame, frame.area(), state, &ui.crystal_menu);
        return;
    }
    if ui.light_menu.active {
        render_light_menu(frame, frame.area(), state, &ui.light_menu);
        return;
    }
    if ui.skills_menu.active {
        render_skills_menu(frame, state, &ui.skills_menu);
        return;
    }
    if ui.world_map_view.open {
        if let Some(ref world_map) = state.world.world_map {
            saltglass_steppe::ui::render_world_map(
                frame,
                frame.area(),
                world_map,
                state.world.world_x,
                state.world.world_y,
                &ui.world_map_view,
                state,
            );
        }
        return;
    }

    // Death screen
    if state.player.hp <= 0 {
        render_death_screen(frame, state);
        return;
    }

    // Main layout: side panel + game area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(state.world.map.width as u16 + 2),
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
        let block = Block::default()
            .title(" Look (Esc/Enter to exit) ")
            .borders(Borders::ALL);
        frame.render_widget(
            Paragraph::new(desc)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .block(block),
            game_chunks[0],
        );
    }

    // Render game map using new modular renderer
    let look_cursor = if ui.look_mode.active {
        Some((ui.look_mode.x, ui.look_mode.y))
    } else {
        None
    };
    renderer.render_game(
        frame,
        game_chunks[1],
        state,
        ui.frame_count,
        look_cursor,
        ui.debug_console.active,
    );
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

    // Tutorial overlay
    if let Some((_, ref text)) = ui.tutorial_message {
        let area = frame.area();
        let popup_width = (area.width.min(60)).max(20);
        let lines: Vec<Line> = textwrap::wrap(text, (popup_width - 4) as usize)
            .into_iter()
            .map(|l| Line::from(l.into_owned()))
            .collect();
        let popup_height = (lines.len() as u16 + 4).min(area.height);
        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);
        frame.render_widget(ratatui::widgets::Clear, popup_area);
        let block = Block::default()
            .title(" Tutorial ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));
        let mut all_lines = lines;
        all_lines.push(Line::from(""));
        all_lines.push(Line::from(Span::styled(
            "Press any key to dismiss",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(
            Paragraph::new(all_lines).block(block),
            popup_area,
        );
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
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unknown UI type",
        )),
    }
}

fn run_main_game() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Initialize IPC server
    use saltglass_steppe::ipc::{IpcMessage, IpcServer};
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };

    'main: loop {
        // Main menu loop
        let mut menu_state = MainMenuState::new();
        let mut menu_tick: u64 = 0;
        let (class_id, seed) = loop {
            terminal.draw(|f| render_menu(f, menu_tick, &menu_state))?;
            menu_tick = menu_tick.wrapping_add(1);
            match handle_menu_input(&mut menu_state)? {
                MenuAction::NewGame(class) => {
                    let seed = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    break (class, seed);
                }
                MenuAction::NewGameWithSeed(seed) => {
                    // Get the selected class
                    let classes: Vec<_> = saltglass_steppe::all_classes()
                        .iter()
                        .filter(|c| menu_state.meta.is_class_unlocked(&c.id))
                        .collect();
                    let class = classes.get(menu_state.class_index).unwrap().id.clone();
                    break (class, seed);
                }
                MenuAction::Controls => {
                    // Show controls screen
                    loop {
                        terminal.draw(render_controls)?;
                        if event::poll(std::time::Duration::from_millis(16))? {
                            if let Event::Key(key) = event::read()? {
                                if key.kind == KeyEventKind::Press {
                                    break;
                                }
                            }
                        }
                    }
                }
                MenuAction::Quit => {
                    disable_raw_mode()?;
                    stdout().execute(LeaveAlternateScreen)?;
                    return Ok(());
                }
                MenuAction::LoadGame(_) | MenuAction::None | MenuAction::TileTest(_) => {}
            }
        };

        // Create game with selected class and seed
        let mut state = GameState::new_with_class(seed, &class_id);
        let mut ui = UiState::new();
        // Initialize camera to player position
        ui.camera_x = state.player.x as f32;
        ui.camera_y = state.player.y as f32;

        // Initial tutorial check (game_start trigger)
        if let Some(msg) = state.get_next_tutorial_message() {
            ui.tutorial_message = Some(msg);
        }

        loop {
            // Only tick animations and updates if debug console is not active
            if !ui.debug_console.active {
                ui.tick_frame();
                state.world.visual_effects.tick_hit_flash();
                state.world.visual_effects.tick_damage_numbers();
                state.world.visual_effects.tick_projectile_trails();
                state.world.visual_effects.tick_light_beams();
                state.world.visual_effects.tick_animation();
                ui.update_camera(state.player.x, state.player.y);
                ui.dialog_box.tick(16); // ~60fps
            }

            // Check for pending dialogue from NPC interaction
            if let Some((speaker, text)) = state.pending_dialogue.take() {
                ui.dialog_box.show(&speaker, &text);
            }

            // Check for pending ARIA dialogue
            if let Some((text, options)) = state.pending_aria_dialogue.take() {
                ui.aria_interface.response_text = text;
                ui.aria_interface.options = options;
                ui.aria_interface.selected_option = 0;
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
                    use saltglass_steppe::trading::{calculate_area_tier, get_trade_interface};
                    let area_tier = calculate_area_tier(&state.world.enemies);
                    if let Some(interface) = get_trade_interface(
                        &trader_id,
                        area_tier,
                        &state.player.faction_reputation,
                        None, // Player faction not yet implemented
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
                if ei >= state.world.enemies.len() || state.world.enemies[ei].hp <= 0 {
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
                        // Check for tutorial messages after each action
                        if ui.tutorial_message.is_none() {
                            if let Some(msg) = state.get_next_tutorial_message() {
                                ui.tutorial_message = Some(msg);
                            }
                        }

                        // Send game state update to satellite terminals
                        let adaptations: Vec<String> = state
                            .player
                            .adaptations
                            .iter()
                            .map(|a| a.name().to_string())
                            .collect();

                        let _ = ipc_server.send_message(IpcMessage::GameState {
                            hp: state.player.hp,
                            max_hp: state.player.max_hp,
                            refraction: state.player.refraction as i32,
                            turn: state.turn as u32,
                            storm_countdown: state.world.storm.turns_until as i32,
                            adaptations,
                            god_view: state.debug_god_view,
                            phase_mode: state.debug_phase,
                        });

                        // Send inventory update
                        let equipped_items: Vec<String> = [
                            ("Weapon", &state.player.equipment.weapon),
                            ("Ranged", &state.player.equipment.ranged_weapon),
                            ("Head", &state.player.equipment.head),
                            ("Jacket", &state.player.equipment.jacket),
                            ("Pants", &state.player.equipment.pants),
                            ("Boots", &state.player.equipment.boots),
                            ("Gloves", &state.player.equipment.gloves),
                            ("L.Wrist", &state.player.equipment.left_wrist),
                            ("R.Wrist", &state.player.equipment.right_wrist),
                            ("Necklace", &state.player.equipment.necklace),
                            ("Accessory", &state.player.equipment.accessory),
                            ("Backpack", &state.player.equipment.backpack),
                        ]
                        .iter()
                        .filter_map(|(slot, item)| {
                            item.as_ref().map(|i| format!("{}: {}", slot, i))
                        })
                        .collect();

                        let _ = ipc_server.send_message(IpcMessage::InventoryUpdate {
                            items: state.player.inventory.clone(),
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
                        let tile_seed = state
                            .world
                            .world_map
                            .as_ref()
                            .map(|wm| wm.tile_seed(state.world.world_x, state.world.world_y))
                            .unwrap_or(0);
                        let _ = ipc_server.send_message(IpcMessage::DebugInfo {
                            player_pos: (state.player.x, state.player.y),
                            enemies_count: state.world.enemies.len(),
                            items_count: state.player.inventory.len(),
                            storm_intensity: state.world.storm.intensity as i32,
                            seed: state.seed,
                            tile_seed,
                            world_pos: (state.world.world_x, state.world.world_y),
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
                    None => break,              // Return to main menu
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
