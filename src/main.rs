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
use tui_rpg::{get_item_def, GameState, Tile, MAP_HEIGHT};

const SAVE_FILE: &str = "savegame.ron";

enum Action { Quit, Move(i32, i32), Save, Load, UseItem(usize), None }

fn handle_input() -> Result<Action> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(match key.code {
                    KeyCode::Char('q') => Action::Quit,
                    KeyCode::Char('S') => Action::Save,
                    KeyCode::Char('L') => Action::Load,
                    KeyCode::Char('1') => Action::UseItem(0),
                    KeyCode::Char('2') => Action::UseItem(1),
                    KeyCode::Char('3') => Action::UseItem(2),
                    KeyCode::Up | KeyCode::Char('k') => Action::Move(0, -1),
                    KeyCode::Down | KeyCode::Char('j') => Action::Move(0, 1),
                    KeyCode::Left | KeyCode::Char('h') => Action::Move(-1, 0),
                    KeyCode::Right | KeyCode::Char('l') => Action::Move(1, 0),
                    _ => Action::None,
                });
            }
        }
    }
    Ok(Action::None)
}

fn update(state: &mut GameState, action: Action) -> bool {
    match action {
        Action::Quit => return false,
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
        Action::None => {}
    }
    true
}

fn render(frame: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(MAP_HEIGHT as u16 + 2), Constraint::Length(9)])
        .split(frame.area());

    let storm_color = if state.storm.turns_until <= 3 { Color::Red } else { Color::Yellow };
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
    let inner = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    for y in 0..state.map.height {
        let mut spans: Vec<Span> = Vec::new();
        for x in 0..state.map.width {
            let idx = state.map.idx(x as i32, y as i32);
            let (ch, style) = if x as i32 == state.player_x && y as i32 == state.player_y {
                ('@', Style::default().fg(Color::Yellow).bold())
            } else if let Some(e) = state.enemies.iter().find(|e| e.x == x as i32 && e.y == y as i32 && e.hp > 0) {
                if state.visible.contains(&idx) {
                    let color = match e.id.as_str() {
                        "mirage_hound" => Color::LightYellow,
                        "glass_beetle" => Color::Cyan,
                        "salt_mummy" => Color::White,
                        _ => Color::Red,
                    };
                    (e.glyph(), Style::default().fg(color))
                } else if state.revealed.contains(&idx) {
                    ('~', Style::default().fg(Color::DarkGray))
                } else { (' ', Style::default()) }
            } else if let Some(item) = state.items.iter().find(|i| i.x == x as i32 && i.y == y as i32) {
                if state.visible.contains(&idx) {
                    (item.glyph(), Style::default().fg(Color::LightMagenta))
                } else if state.revealed.contains(&idx) {
                    ('~', Style::default().fg(Color::DarkGray))
                } else { (' ', Style::default()) }
            } else if state.visible.contains(&idx) {
                let tile = &state.map.tiles[idx];
                let style = match tile {
                    Tile::Floor => Style::default().fg(Color::DarkGray),
                    Tile::Wall => Style::default().fg(Color::Gray),
                    Tile::Glass => Style::default().fg(Color::Cyan),
                };
                (tile.glyph(), style)
            } else if state.revealed.contains(&idx) {
                ('~', Style::default().fg(Color::DarkGray))
            } else { (' ', Style::default()) };
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
        "\nMove: hjkl | Use: 1-3 | Save: S | Load: L | Quit: q"
    };
    let log_text = format!("{}\n{}{}", inv_str, state.messages.join("\n"), status);
    let log_block = Block::default().title(" Log ").borders(Borders::ALL);
    frame.render_widget(Paragraph::new(log_text).block(log_block), chunks[1]);
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

fn render_menu(frame: &mut Frame) {
    let area = frame.area();
    let block = Block::default().title(" Saltglass Steppe ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("SALTGLASS STEPPE", Style::default().fg(Color::Yellow).bold())),
        Line::from(""),
        Line::from("A roguelike of glass storms and refraction"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::raw("  ["),
            Span::styled("N", Style::default().fg(Color::Green)),
            Span::raw("] New Run"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  ["),
            Span::styled("Q", Style::default().fg(Color::Red)),
            Span::raw("] Quit"),
        ]),
    ];
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Main menu loop
    loop {
        terminal.draw(render_menu)?;
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

    loop {
        terminal.draw(|frame| render(frame, &state))?;
        if !update(&mut state, handle_input()?) { break; }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
