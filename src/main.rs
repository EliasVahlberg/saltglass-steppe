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
use saltglass_steppe::game::save;
use saltglass_steppe::satellite::SatelliteApp;
use saltglass_steppe::ui::{
    Action, MainMenuState, MenuAction, UiState, handle_menu_input, render_book_reader,
    render_bottom_panel, render_chest_ui, render_controls, render_crafting_menu,
    render_crystal_menu, render_damage_numbers, render_death_screen, render_debug_console,
    render_debug_menu, render_dialog_box, render_faction_menu, render_inventory_menu,
    render_issue_reporter, render_light_menu, render_menu, render_pause_menu, render_psychic_menu,
    render_quest_log, render_side_panel, render_skills_menu, render_target_hud, render_void_menu,
    render_wiki,
};
use saltglass_steppe::{GameState, Renderer, get_item_def};
use std::io::{Result, stdout};

mod session;
use session::{SessionOutcome, run_game_session};

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
        Action::Save => {
            if state.debug.test_mode {
                state.log("Cannot save in test mode.");
            } else {
                state.world.saved_on_world_map = ui.world_map_view.open;
                match save::save_game(state) {
                    Ok(path) => state.log(format!(
                        "Game saved: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    )),
                    Err(e) => state.log(format!("Save failed: {}", e)),
                }
            }
        }
        Action::Load => match save::list_saves().into_iter().next() {
            Some(info) => match save::load_game(&info.path) {
                Ok(loaded) => {
                    *state = loaded;
                    ui.world_map_view.open = state.world.saved_on_world_map;
                    state.log("Game loaded.");
                }
                Err(e) => state.log(format!("Load failed: {}", e)),
            },
            None => state.log("No saves found."),
        },
        Action::UseItem(idx) => {
            if state.player.hp > 0 {
                state.dispatch(saltglass_steppe::game::effects::Command::UseItem { index: idx });
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
                state.dispatch(saltglass_steppe::game::effects::Command::Move { dx, dy });
            }
        }
        Action::EndTurn => {
            if state.player.hp > 0 {
                state.end_turn();
            }
        }
        Action::Wait => {
            if state.player.hp > 0 {
                state.dispatch(saltglass_steppe::game::effects::Command::Wait);
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
                state.dispatch(saltglass_steppe::game::effects::Command::RangedAttack {
                    target_x: x,
                    target_y: y,
                });
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
            if let Some(interface) = &mut ui.trade_menu.interface
                && let Some(item) = interface.available_items.get(idx)
            {
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
                        state.emit(saltglass_steppe::event::GameEvent::TradeCompleted {
                            npc_id: trader_id,
                        });
                    }
                    Err(e) => state.log(e),
                }
            }
        }
        Action::TradeSell(idx) => {
            if let Some(interface) = &ui.trade_menu.interface
                && let Some(item_id) = state.player.inventory.get(idx)
            {
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
                        state.emit(saltglass_steppe::event::GameEvent::TradeCompleted {
                            npc_id: trader_id,
                        });
                    }
                    Err(e) => state.log(e),
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
            if let Some(idx) = ui.inventory_menu.selected_inv_index()
                && idx < state.player.inventory.len()
                && let Some(def) = get_item_def(&state.player.inventory[idx])
                && let Some(slot_str) = &def.equip_slot
                && let Ok(_slot) = slot_str.parse::<saltglass_steppe::EquipSlot>()
            {
                state.dispatch(saltglass_steppe::game::effects::Command::Equip {
                    inv_idx: idx,
                    slot: slot_str.to_string(),
                });
            }
        }
        Action::UnequipSelected => {
            if let Some(slot) = ui.inventory_menu.selected_equip_slot() {
                state.dispatch(saltglass_steppe::game::effects::Command::Unequip {
                    slot: format!("{:?}", slot).to_lowercase(),
                });
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
            if let Some(&chest_idx) = state.spatial.chest_positions.get(&(state.player.x, state.player.y)) {
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
                state.dispatch(saltglass_steppe::game::effects::Command::Interact {
                    x: state.player.x,
                    y: state.player.y,
                });
            }
        }
        Action::Examine(_, _) => {
            if state.player.hp > 0 {
                state.dispatch(saltglass_steppe::game::effects::Command::Examine {
                    x: state.player.x,
                    y: state.player.y,
                });
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
            // Void abilities are display-only; activation not yet implemented
        }
        Action::OpenSkillsMenu => {
            ui.skills_menu.open();
        }
        Action::UsePsychicAbility(ability_id) => {
            state.dispatch(saltglass_steppe::game::effects::Command::UsePsychic {
                ability_id: ability_id.clone(),
            });
        }
        Action::RangedAttackMode => {
            // TODO: Implement ranged attack mode
        }
        Action::TargetMode => {
            // TODO: Implement targeting mode
        }
        Action::OpenWorldMap => {
            ui.world_map_view
                .toggle(state.world.world_x, state.world.world_y);
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
    if !ui.aria_interface.response_text.is_empty() {
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
        render_skills_menu(frame, state, &mut ui.skills_menu);
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
        if ui.pause_menu.active {
            render_pause_menu(frame, ui.pause_menu.selected_index);
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
        let popup_width = area.width.clamp(20, 60);
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
        frame.render_widget(Paragraph::new(all_lines).block(block), popup_area);
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
    use saltglass_steppe::ipc::IpcServer;
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
            return Err(std::io::Error::other(e.to_string()));
        }
    };

    let mut menu_state = MainMenuState::new();
    'main: loop {
        // Main menu loop
        let mut menu_tick: u64 = 0;
        let (class_id, seed, player_name) = loop {
            terminal.draw(|f| render_menu(f, menu_tick, &menu_state))?;
            menu_tick = menu_tick.wrapping_add(1);
            match handle_menu_input(&mut menu_state)? {
                MenuAction::NewGame {
                    class_id: class,
                    name,
                } => {
                    let seed = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    break (class, seed, name);
                }
                MenuAction::NewGameWithSeed {
                    seed,
                    class_id: class,
                    name,
                } => {
                    break (class, seed, name);
                }
                MenuAction::Controls => {
                    // Show controls screen
                    loop {
                        terminal.draw(render_controls)?;
                        if event::poll(std::time::Duration::from_millis(16))?
                            && let Event::Key(key) = event::read()?
                            && key.kind == KeyEventKind::Press
                        {
                            break;
                        }
                    }
                }
                MenuAction::Quit => {
                    disable_raw_mode()?;
                    stdout().execute(LeaveAlternateScreen)?;
                    return Ok(());
                }
                MenuAction::TileTest(cfg) => {
                    let params = cfg.to_tile_params();
                    let seed = params.seed;
                    let mut state = GameState::new_with_class(seed, "wanderer");
                    state.debug.test_mode = true;
                    state.load_test_tile(params);
                    if let SessionOutcome::Quit =
                        run_game_session(&mut terminal, &mut renderer, state, |_, _| {})?
                    {
                        break 'main;
                    }
                    continue 'main;
                }
                MenuAction::LoadGame(path) => {
                    match save::load_game(&path) {
                        Ok(loaded_state) => {
                            menu_state.load_error = None;
                            if let SessionOutcome::Quit = run_game_session(
                                &mut terminal,
                                &mut renderer,
                                loaded_state,
                                |_, _| {},
                            )? {
                                break 'main;
                            }
                            menu_state.save_entries = save::list_saves();
                        }
                        Err(e) => {
                            menu_state.save_list = true;
                            menu_state.save_entries = save::list_saves();
                            menu_state.load_error = Some(e);
                        }
                    }
                    continue 'main;
                }
                MenuAction::None => {}
            }
        };

        // Create game with selected class and seed
        let mut state = GameState::new_with_class(seed, &class_id);
        state.player.name = player_name;
        let outcome = run_game_session(&mut terminal, &mut renderer, state, |state, _ui| {
            use saltglass_steppe::ipc::IpcMessage;
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
                turn: state.turn,
                storm_countdown: state.world.storm.turns_until as i32,
                adaptations,
                god_view: state.debug.god_view,
                phase_mode: state.debug.phase,
            });
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
            .filter_map(|(slot, item)| item.as_ref().map(|i| format!("{}: {}", slot, i)))
            .collect();
            let _ = ipc_server.send_message(IpcMessage::InventoryUpdate {
                items: state.player.inventory.clone(),
                equipped: equipped_items,
            });
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
                god_view: state.debug.god_view,
                phase_mode: state.debug.phase,
            });
            while let Some(message) = ipc_server.try_recv_message() {
                if let IpcMessage::Command { action } = message {
                    state.debug_command(&action);
                }
            }
        })?;
        if let SessionOutcome::Quit = outcome {
            break 'main;
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
