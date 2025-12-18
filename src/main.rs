use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use std::io::{stdout, Result};
use tui_rpg::{get_active_effects, get_enemy_effects, get_item_def, get_light_def, EffectContext, GameState, Tile, VisualEffect, MAP_HEIGHT};
use tui_rpg::ui::{render_inventory_menu, render_quest_log, render_crafting_menu, render_side_panel, render_bottom_panel, handle_input, Action, UiState, handle_menu_input, render_menu, render_controls, MenuAction};

const SAVE_FILE: &str = "savegame.ron";

/// Dim a color based on light level (0-255)
fn dim_color(color: Color, light: u8) -> Color {
    if light >= 200 { return color; }
    let factor = light as f32 / 255.0;
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        Color::Gray | Color::DarkGray => if light < 100 { Color::Black } else { Color::DarkGray },
        Color::Cyan => if light < 100 { Color::DarkGray } else { color },
        _ => if light < 100 { Color::DarkGray } else { color },
    }
}

fn update(state: &mut GameState, action: Action, ui: &mut UiState) -> bool {
    match action {
        Action::Quit => return false,
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
                state.try_move(dx, dy);
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
                state.try_ranged_attack(x, y);
            }
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
        Action::Craft => {
            if let Some(recipe_id) = ui.crafting_menu.selected_recipe_id() {
                state.craft(recipe_id);
            }
        }
        Action::None => {}
    }
    true
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

    // Build effect context
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
    
    // Add triggered effects
    for te in &state.triggered_effects {
        if let Some(effect) = tui_rpg::parse_effect(&te.effect) {
            player_effects.push(effect);
        }
    }
    
    // Death screen
    if state.player_hp <= 0 {
        let area = frame.area();
        let mut lines: Vec<Line> = Vec::new();
        let death_msg = "YOU DIED";
        for _ in 0..area.height {
            let mut row = String::new();
            while row.len() < area.width as usize {
                row.push_str(death_msg);
                row.push(' ');
            }
            lines.push(Line::from(Span::styled(row, Style::default().fg(Color::Red).bold())));
        }
        frame.render_widget(Paragraph::new(lines), area);
        let center_y = area.height / 2;
        let msg = " Press Q to quit ";
        let center_x = area.width.saturating_sub(msg.len() as u16) / 2;
        frame.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(Color::White).bg(Color::Red))),
            Rect::new(center_x, center_y, msg.len() as u16, 1)
        );
        return;
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

    let title = Line::from(vec![
        format!(" Turn {} ", state.turn).into(),
    ]);
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(game_chunks[1]);
    frame.render_widget(block, game_chunks[1]);

    let mut lines: Vec<Line> = Vec::new();
    for y in 0..state.map.height {
        let mut spans: Vec<Span> = Vec::new();
        for x in 0..state.map.width {
            let idx = state.map.idx(x as i32, y as i32);
            let is_look_cursor = ui.look_mode.active && x as i32 == ui.look_mode.x && y as i32 == ui.look_mode.y;
            let (ch, style) = if x as i32 == state.player_x && y as i32 == state.player_y {
                let mut base_style = Style::default().fg(Color::Yellow).bold();
                // Apply player effects
                for effect in &player_effects {
                    match effect {
                        VisualEffect::Blink { speed, color } => {
                            if (ui.frame_count / *speed as u64) % 2 == 0 {
                                base_style = base_style.fg(*color);
                            }
                        }
                        VisualEffect::Glow { color } => {
                            base_style = base_style.fg(*color);
                        }
                    }
                }
                ('@', base_style)
            } else if let Some(&ei) = state.enemy_positions.get(&(x as i32, y as i32)) {
                let e = &state.enemies[ei];
                if state.visible.contains(&idx) {
                    let base_color = match e.id.as_str() {
                        "mirage_hound" => Color::LightYellow,
                        "glass_beetle" => Color::Cyan,
                        "salt_mummy" => Color::White,
                        _ => Color::Red,
                    };
                    let mut style = Style::default().fg(base_color);
                    // Apply enemy effects
                    for effect in get_enemy_effects(&e.id) {
                        match effect {
                            VisualEffect::Blink { speed, color } => {
                                if (ui.frame_count / speed as u64) % 2 == 0 {
                                    style = style.fg(color);
                                }
                            }
                            VisualEffect::Glow { color } => {
                                style = style.fg(color);
                            }
                        }
                    }
                    (e.glyph(), style)
                } else if state.revealed.contains(&idx) {
                    // Show underlying tile, not enemy
                    let tile = &state.map.tiles[idx];
                    (tile.glyph(), Style::default().fg(Color::DarkGray))
                } else { (' ', Style::default()) }
            } else if let Some(&ni) = state.npc_positions.get(&(x as i32, y as i32)) {
                let npc = &state.npcs[ni];
                if state.visible.contains(&idx) {
                    (npc.glyph(), Style::default().fg(Color::Green).bold())
                } else if state.revealed.contains(&idx) {
                    let tile = &state.map.tiles[idx];
                    (tile.glyph(), Style::default().fg(Color::DarkGray))
                } else { (' ', Style::default()) }
            } else if state.item_positions.contains_key(&(x as i32, y as i32)) {
                let item = &state.items[state.item_positions[&(x as i32, y as i32)][0]];
                if state.visible.contains(&idx) {
                    (item.glyph(), Style::default().fg(Color::LightMagenta))
                } else if state.revealed.contains(&idx) {
                    let tile = &state.map.tiles[idx];
                    (tile.glyph(), Style::default().fg(Color::DarkGray))
                } else { (' ', Style::default()) }
            } else if let Some(ml) = state.map.lights.iter().find(|l| l.x == x as i32 && l.y == y as i32) {
                if state.visible.contains(&idx) {
                    let def = get_light_def(&ml.id);
                    let glyph = def.map(|d| d.glyph.chars().next().unwrap_or('*')).unwrap_or('*');
                    let color = match def.map(|d| d.color.as_str()) {
                        Some("orange") => Color::Rgb(255, 140, 0),
                        Some("yellow") => Color::Yellow,
                        Some("cyan") => Color::Cyan,
                        Some("red") => Color::Rgb(255, 80, 40),
                        _ => Color::Yellow,
                    };
                    (glyph, Style::default().fg(color))
                } else if state.revealed.contains(&idx) {
                    let tile = &state.map.tiles[idx];
                    (tile.glyph(), Style::default().fg(Color::DarkGray))
                } else { (' ', Style::default()) }
            } else if state.visible.contains(&idx) {
                let tile = &state.map.tiles[idx];
                let light = state.get_light_level(x as i32, y as i32);
                let base_color = match tile {
                    Tile::Floor => Color::DarkGray,
                    Tile::Wall { .. } => Color::Gray,
                    Tile::Glass => Color::Cyan,
                };
                (tile.glyph(), Style::default().fg(dim_color(base_color, light)))
            } else if state.revealed.contains(&idx) {
                // Show actual tile glyph in dark gray for explored-but-not-visible
                let tile = &state.map.tiles[idx];
                (tile.glyph(), Style::default().fg(Color::DarkGray))
            } else { (' ', Style::default()) };
            
            let style = if is_look_cursor { style.bg(Color::White).fg(Color::Black) } else { style };
            spans.push(Span::styled(ch.to_string(), style));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inner);

    // Bottom panel with log
    render_bottom_panel(frame, game_chunks[2], state);

    // Right side panel with stats
    render_side_panel(frame, main_chunks[1], state);
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Main menu loop
    let mut menu_tick: u64 = 0;
    loop {
        terminal.draw(|f| render_menu(f, menu_tick))?;
        menu_tick = menu_tick.wrapping_add(1);
        match handle_menu_input()? {
            MenuAction::NewGame => break,
            MenuAction::Quit => {
                disable_raw_mode()?;
                stdout().execute(LeaveAlternateScreen)?;
                return Ok(());
            }
            MenuAction::None => {}
        }
    }

    // Game loop
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let mut state = GameState::new(seed);
    let mut ui = UiState::new();

    loop {
        ui.tick_frame();
        
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
            if !update(&mut state, action, &mut ui) { break; }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
