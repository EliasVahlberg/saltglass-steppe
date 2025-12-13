use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::{stdout, Result};
use tui_rpg::{GameState, MAP_HEIGHT};

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

    // Adaptations display
    let adapt_str = if state.adaptations.is_empty() {
        String::new()
    } else {
        format!(" | {}", state.adaptations.iter().map(|a| a.name()).collect::<Vec<_>>().join(", "))
    };

    let title = format!(
        " HP:{}/{} | Ref:{} | Turn {} | Storm:{}{}",
        state.player_hp, state.player_max_hp, state.refraction,
        state.turn, state.storm.turns_until, adapt_str
    );
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    let mut map_str = String::new();
    for y in 0..state.map.height {
        for x in 0..state.map.width {
            let idx = state.map.idx(x as i32, y as i32);
            if x as i32 == state.player_x && y as i32 == state.player_y {
                map_str.push('@');
            } else if let Some(e) = state.enemies.iter().find(|e| e.x == x as i32 && e.y == y as i32 && e.hp > 0) {
                if state.visible.contains(&idx) { map_str.push(e.kind.glyph()); }
                else if state.revealed.contains(&idx) { map_str.push('~'); }
                else { map_str.push(' '); }
            } else if let Some(item) = state.items.iter().find(|i| i.x == x as i32 && i.y == y as i32) {
                if state.visible.contains(&idx) { map_str.push(item.kind.glyph()); }
                else if state.revealed.contains(&idx) { map_str.push('~'); }
                else { map_str.push(' '); }
            } else if state.visible.contains(&idx) {
                map_str.push(state.map.tiles[idx].glyph());
            } else if state.revealed.contains(&idx) {
                map_str.push('~');
            } else {
                map_str.push(' ');
            }
        }
        map_str.push('\n');
    }
    frame.render_widget(Paragraph::new(map_str), inner);

    // Inventory line
    let inv_str = if state.inventory.is_empty() {
        "Inventory: (empty)".to_string()
    } else {
        format!("Inventory: {}", state.inventory.iter().enumerate()
            .map(|(i, k)| format!("[{}]{}", i + 1, k.name()))
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

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

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
