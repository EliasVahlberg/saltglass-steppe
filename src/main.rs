use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::Alignment,
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::{stdout, Result};
use tui_rpg::{get_active_effects, get_enemy_effects, get_item_def, EffectContext, GameState, Tile, VisualEffect, MAP_HEIGHT};

const SAVE_FILE: &str = "savegame.ron";

/// UI-specific state, separate from game logic
struct UiState {
    look_mode: LookMode,
    frame_count: u64,
    show_controls: bool,
}

impl UiState {
    fn new() -> Self {
        Self {
            look_mode: LookMode { active: false, x: 0, y: 0 },
            frame_count: 0,
            show_controls: false,
        }
    }
    
    fn tick_frame(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}

struct LookMode {
    active: bool,
    x: i32,
    y: i32,
}

enum Action { Quit, Move(i32, i32), Save, Load, UseItem(usize), OpenControls, EnterLook, BreakWall(i32, i32), EndTurn, AutoExplore, RangedAttack(i32, i32), None }

fn handle_input(ui: &mut UiState) -> Result<Action> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if ui.look_mode.active {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => ui.look_mode.active = false,
                        KeyCode::Up | KeyCode::Char('k') => ui.look_mode.y -= 1,
                        KeyCode::Down | KeyCode::Char('j') => ui.look_mode.y += 1,
                        KeyCode::Left | KeyCode::Char('h') => ui.look_mode.x -= 1,
                        KeyCode::Right | KeyCode::Char('l') => ui.look_mode.x += 1,
                        KeyCode::Char('b') => {
                            let (x, y) = (ui.look_mode.x, ui.look_mode.y);
                            ui.look_mode.active = false;
                            return Ok(Action::BreakWall(x, y));
                        }
                        KeyCode::Char('f') => {
                            let (x, y) = (ui.look_mode.x, ui.look_mode.y);
                            ui.look_mode.active = false;
                            return Ok(Action::RangedAttack(x, y));
                        }
                        _ => {}
                    }
                    return Ok(Action::None);
                }
                return Ok(match key.code {
                    KeyCode::Char('q') => Action::Quit,
                    KeyCode::Char('S') => Action::Save,
                    KeyCode::Char('L') => Action::Load,
                    KeyCode::Char('x') => Action::EnterLook,
                    KeyCode::Char('e') => Action::EndTurn,
                    KeyCode::Char('o') => Action::AutoExplore,
                    KeyCode::Char('1') => Action::UseItem(0),
                    KeyCode::Char('2') => Action::UseItem(1),
                    KeyCode::Char('3') => Action::UseItem(2),
                    KeyCode::Up | KeyCode::Char('k') => Action::Move(0, -1),
                    KeyCode::Down | KeyCode::Char('j') => Action::Move(0, 1),
                    KeyCode::Left | KeyCode::Char('h') => Action::Move(-1, 0),
                    KeyCode::Right | KeyCode::Char('l') => Action::Move(1, 0),
                    KeyCode::Esc => Action::OpenControls,
                    _ => Action::None,
                });
            }
        }
    }
    Ok(Action::None)
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
        Action::None => {}
    }
    true
}

fn render(frame: &mut Frame, state: &GameState, ui: &UiState) {
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

    let desc_height = if ui.look_mode.active { 3u16 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(desc_height),
            Constraint::Min(MAP_HEIGHT as u16 + 2),
            Constraint::Length(9),
        ])
        .split(frame.area());

    // Look mode description box
    if ui.look_mode.active {
        let desc = state.describe_at(ui.look_mode.x, ui.look_mode.y);
        let block = Block::default().title(" Look (Esc/Enter to exit) ").borders(Borders::ALL);
        frame.render_widget(Paragraph::new(desc).block(block), chunks[0]);
    }

    let storm_color = if state.storm.turns_until <= 3 { Color::Red } 
        else if state.storm.turns_until <= 5 { Color::Yellow } 
        else { Color::Green };
    let title = Line::from(vec![
        " HP:".into(),
        Span::styled(format!("{}/{}", state.player_hp, state.player_max_hp), 
            Style::default().fg(if state.player_hp <= 5 { Color::Red } else { Color::Green })),
        " | Ref:".into(),
        Span::styled(format!("{}", state.refraction), Style::default().fg(Color::Cyan)),
        format!(" | Turn {} | Storm:", state.turn).into(),
        Span::styled(format!("{}", state.storm.turns_until), Style::default().fg(storm_color)),
        if state.adaptations.is_empty() { Span::raw("") } else {
            Span::styled(format!(" | {}", state.adaptations.iter().map(|a| a.name()).collect::<Vec<_>>().join(", ")),
                Style::default().fg(Color::Magenta))
        },
        " ".into(),
    ]);
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(chunks[1]);
    frame.render_widget(block, chunks[1]);

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
            } else if state.visible.contains(&idx) {
                let tile = &state.map.tiles[idx];
                let style = match tile {
                    Tile::Floor => Style::default().fg(Color::DarkGray),
                    Tile::Wall { .. } => Style::default().fg(Color::Gray),
                    Tile::Glass => Style::default().fg(Color::Cyan),
                };
                (tile.glyph(), style)
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

    let inv_str = if state.inventory.is_empty() {
        "Inventory: (empty)".to_string()
    } else {
        format!("Inventory: {}", state.inventory.iter().enumerate()
            .map(|(i, id)| {
                let name = get_item_def(id).map(|d| d.name.as_str()).unwrap_or("?");
                format!("[{}]{}", i + 1, name)
            })
            .collect::<Vec<_>>().join(" "))
    };
    let status = if state.player_hp <= 0 {
        "\n*** YOU DIED *** Press q to quit"
    } else {
        "\nMove: hjkl | Look: x | End turn: e | Auto-explore: o | Use: 1-3 | Save: S | Load: L | Quit: q"
    };
    let log_text = format!("{}\n{}{}", inv_str, state.messages.join("\n"), status);
    let log_block = Block::default().title(" Log ").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(log_text).block(log_block), chunks[2]);
}

enum MenuAction { NewGame, Quit, None }

fn handle_menu_input() -> Result<MenuAction> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(match key.code {
                    KeyCode::Char('n') | KeyCode::Enter => MenuAction::NewGame,
                    KeyCode::Char('q') | KeyCode::Esc => MenuAction::Quit,
                    _ => MenuAction::None,
                });
            }
        }
    }
    Ok(MenuAction::None)
}

fn render_menu(frame: &mut Frame, tick: u64) {
    let area = frame.area();
    let height = area.height as usize;
    let width = area.width as usize;
    
    // Slow tick for subtle animation
    let slow_tick = tick / 8;
    
    // Generate subtle sand background for entire screen
    let sand_chars = ['.', '·', ' ', ' ', ' ', '.', ' ', '·', ' ', ' '];
    let mut bg_lines: Vec<Line> = Vec::new();
    for y in 0..height {
        let mut row = String::new();
        for x in 0..width {
            let idx = ((x + y * 3 + slow_tick as usize) * 7) % sand_chars.len();
            row.push(sand_chars[idx]);
        }
        bg_lines.push(Line::from(Span::styled(row, Style::default().fg(Color::Rgb(60, 55, 45)))));
    }
    frame.render_widget(Paragraph::new(bg_lines), area);

    let title_art = vec![
        "░██████╗░█████╗░██╗░░░░░████████╗░██████╗░██╗░░░░░░█████╗░░██████╗░██████╗",
        "██╔════╝██╔══██╗██║░░░░░╚══██╔══╝██╔════╝░██║░░░░░██╔══██╗██╔════╝██╔════╝",
        "╚█████╗░███████║██║░░░░░░░░██║░░░██║░░██╗░██║░░░░░███████║╚█████╗░╚█████╗░",
        "░╚═══██╗██╔══██║██║░░░░░░░░██║░░░██║░░╚██╗██║░░░░░██╔══██║░╚═══██╗░╚═══██╗",
        "██████╔╝██║░░██║███████╗░░░██║░░░╚██████╔╝███████╗██║░░██║██████╔╝██████╔╝",
        "╚═════╝░╚═╝░░╚═╝╚══════╝░░░╚═╝░░░░╚═════╝░╚══════╝╚═╝░░╚═╝╚═════╝░╚═════╝░",
        "",
        "░██████╗████████╗███████╗██████╗░██████╗░███████╗",
        "██╔════╝╚══██╔══╝██╔════╝██╔══██╗██╔══██╗██╔════╝",
        "╚█████╗░░░░██║░░░█████╗░░██████╔╝██████╔╝█████╗░░",
        "░╚═══██╗░░░██║░░░██╔══╝░░██╔═══╝░██╔═══╝░██╔══╝░░",
        "██████╔╝░░░██║░░░███████╗██║░░░░░██║░░░░░███████╗",
        "╚═════╝░░░░╚═╝░░░╚══════╝╚═╝░░░░░╚═╝░░░░░╚══════╝",
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    
    for line in &title_art {
        lines.push(Line::from(Span::styled(*line, Style::default().fg(Color::Cyan))));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("A roguelike of glass storms and refraction", Style::default().fg(Color::DarkGray).italic())));
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ◆ ", Style::default().fg(Color::Cyan)),
        Span::raw("Press "),
        Span::styled("N", Style::default().fg(Color::Green).bold()),
        Span::raw(" to begin your journey"),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ◆ ", Style::default().fg(Color::Cyan)),
        Span::raw("Press "),
        Span::styled("Q", Style::default().fg(Color::Red).bold()),
        Span::raw(" to quit"),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("─────────────────────────────────────────────────────────────────────────", Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(Span::styled("  The storms have scoured the steppe for centuries.", Style::default().fg(Color::Yellow))));
    lines.push(Line::from(Span::styled("  Glass grows where flesh once walked.", Style::default().fg(Color::Yellow))));
    lines.push(Line::from(Span::styled("  You feel the refraction building in your veins...", Style::default().fg(Color::Magenta))));

    // Center the content
    let content_height = lines.len() as u16;
    let start_y = area.height.saturating_sub(content_height) / 2;
    let content_area = Rect::new(area.x, area.y + start_y, area.width, content_height);
    
    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, content_area);
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
            let action = handle_input(&mut ui)?;
            if !update(&mut state, action, &mut ui) { break; }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn render_controls(frame: &mut Frame) {
    let area = frame.area();
    let block = Block::default().title(" Controls ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("CONTROLS", Style::default().fg(Color::Yellow).bold())),
        Line::from(""),
        Line::from("  Movement:"),
        Line::from("    h/←  Move left"),
        Line::from("    j/↓  Move down"),
        Line::from("    k/↑  Move up"),
        Line::from("    l/→  Move right"),
        Line::from(""),
        Line::from("  Actions:"),
        Line::from("    x    Look at (examine tile)"),
        Line::from("    1-3  Use inventory item"),
        Line::from("    S    Save game"),
        Line::from("    L    Load game"),
        Line::from(""),
        Line::from("  Menu:"),
        Line::from("    Esc  Open this menu"),
        Line::from("    q    Quit game"),
        Line::from(""),
        Line::from(Span::styled("Press any key to return", Style::default().fg(Color::DarkGray))),
    ];
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}
