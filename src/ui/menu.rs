//! Main menu and controls screen rendering

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::Alignment,
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::Result;

/// Main menu action result
pub enum MenuAction {
    NewGame,
    Quit,
    None,
}

/// Handle main menu input
pub fn handle_menu_input() -> Result<MenuAction> {
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

/// Render the main menu screen
pub fn render_menu(frame: &mut Frame, tick: u64) {
    let area = frame.area();
    let height = area.height as usize;
    let width = area.width as usize;
    let slow_tick = tick / 8;
    
    // Sand background
    let sand_chars = ['.', '·', ' ', ' ', ' ', '.', ' ', '·', ' ', ' '];
    let bg_lines: Vec<Line> = (0..height).map(|y| {
        let row: String = (0..width).map(|x| {
            sand_chars[((x + y * 3 + slow_tick as usize) * 7) % sand_chars.len()]
        }).collect();
        Line::from(Span::styled(row, Style::default().fg(Color::Rgb(60, 55, 45))))
    }).collect();
    frame.render_widget(Paragraph::new(bg_lines), area);

    let title_art = [
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

    let mut lines: Vec<Line> = vec![Line::from("")];
    lines.extend(title_art.iter().map(|l| Line::from(Span::styled(*l, Style::default().fg(Color::Cyan)))));
    lines.extend([
        Line::from(""),
        Line::from(Span::styled("A roguelike of glass storms and refraction", Style::default().fg(Color::DarkGray).italic())),
        Line::from(""), Line::from(""),
        Line::from(vec![
            Span::styled("  ◆ ", Style::default().fg(Color::Cyan)),
            Span::raw("Press "), Span::styled("N", Style::default().fg(Color::Green).bold()),
            Span::raw(" to begin your journey"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ◆ ", Style::default().fg(Color::Cyan)),
            Span::raw("Press "), Span::styled("Q", Style::default().fg(Color::Red).bold()),
            Span::raw(" to quit"),
        ]),
        Line::from(""), Line::from(""),
        Line::from(Span::styled("─────────────────────────────────────────────────────────────────────────", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("  The storms have scoured the steppe for centuries.", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled("  Glass grows where flesh once walked.", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled("  You feel the refraction building in your veins...", Style::default().fg(Color::Magenta))),
    ]);

    let content_height = lines.len() as u16;
    let start_y = area.height.saturating_sub(content_height) / 2;
    let content_area = Rect::new(area.x, area.y + start_y, area.width, content_height);
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), content_area);
}

/// Render the controls help screen
pub fn render_controls(frame: &mut Frame) {
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
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}
