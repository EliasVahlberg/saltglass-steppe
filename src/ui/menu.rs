//! Main menu and controls screen rendering

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::Alignment,
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use std::io::Result;

use super::input::PAUSE_OPTIONS;
use crate::game::{all_classes, MetaProgress};

/// Main menu action result
pub enum MenuAction {
    NewGame(String), // class_id
    LoadGame(String), // save file path
    Controls,
    Quit,
    None,
}

/// Main menu state
#[derive(Default)]
pub struct MainMenuState {
    pub selected: usize,
    pub class_select: bool,
    pub class_index: usize,
    pub meta: MetaProgress,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            meta: MetaProgress::load(),
            ..Default::default()
        }
    }
}

const MAIN_OPTIONS: &[&str] = &["New Game", "Controls", "Quit"];

/// Handle main menu input
pub fn handle_menu_input(state: &mut MainMenuState) -> Result<MenuAction> {
    if !event::poll(std::time::Duration::from_millis(16))? {
        return Ok(MenuAction::None);
    }
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(MenuAction::None);
        }
        
        if state.class_select {
            // Class selection mode
            let classes: Vec<_> = all_classes().iter()
                .filter(|c| state.meta.is_class_unlocked(&c.id))
                .collect();
            return Ok(match key.code {
                KeyCode::Esc => { state.class_select = false; MenuAction::None }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !classes.is_empty() {
                        state.class_index = (state.class_index + classes.len() - 1) % classes.len();
                    }
                    MenuAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !classes.is_empty() {
                        state.class_index = (state.class_index + 1) % classes.len();
                    }
                    MenuAction::None
                }
                KeyCode::Enter => {
                    if let Some(class) = classes.get(state.class_index) {
                        MenuAction::NewGame(class.id.clone())
                    } else {
                        MenuAction::None
                    }
                }
                _ => MenuAction::None,
            });
        }
        
        // Main menu mode
        return Ok(match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.selected = (state.selected + MAIN_OPTIONS.len() - 1) % MAIN_OPTIONS.len();
                MenuAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.selected = (state.selected + 1) % MAIN_OPTIONS.len();
                MenuAction::None
            }
            KeyCode::Enter => match state.selected {
                0 => { state.class_select = true; state.class_index = 0; MenuAction::None }
                1 => MenuAction::Controls,
                2 => MenuAction::Quit,
                _ => MenuAction::None,
            },
            KeyCode::Char('q') | KeyCode::Esc => MenuAction::Quit,
            _ => MenuAction::None,
        });
    }
    Ok(MenuAction::None)
}

/// Render the main menu screen
pub fn render_menu(frame: &mut Frame, tick: u64, state: &MainMenuState) {
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
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("A roguelike of glass storms and refraction", Style::default().fg(Color::DarkGray).italic())));
    lines.push(Line::from(""));
    
    // Menu options
    for (i, opt) in MAIN_OPTIONS.iter().enumerate() {
        let style = if i == state.selected && !state.class_select {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if i == state.selected && !state.class_select { "► " } else { "  " };
        lines.push(Line::from(Span::styled(format!("{}  {}", prefix, opt), style)));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("─────────────────────────────────────────────────────────────────────────", Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(Span::styled("  The storms have scoured the steppe for centuries.", Style::default().fg(Color::Yellow))));

    let content_height = lines.len() as u16;
    let start_y = area.height.saturating_sub(content_height) / 2;
    let content_area = Rect::new(area.x, area.y + start_y, area.width, content_height);
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), content_area);
    
    // Class selection overlay
    if state.class_select {
        render_class_select(frame, state);
    }
}

fn render_class_select(frame: &mut Frame, state: &MainMenuState) {
    let area = frame.area();
    let classes: Vec<_> = all_classes().iter()
        .filter(|c| state.meta.is_class_unlocked(&c.id))
        .collect();
    
    let width = 50u16.min(area.width - 4);
    let height = (classes.len() as u16 * 3 + 4).min(area.height - 4);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);
    
    frame.render_widget(Clear, popup);
    
    let inner_width = width.saturating_sub(6) as usize; // account for borders and indent
    let mut lines: Vec<Line> = Vec::new();
    for (i, class) in classes.iter().enumerate() {
        let style = if i == state.class_index {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default()
        };
        let prefix = if i == state.class_index { "► " } else { "  " };
        lines.push(Line::from(Span::styled(format!("{}{}", prefix, class.name), style)));
        // Word wrap the description
        for wrapped_line in textwrap::wrap(&class.description, inner_width) {
            lines.push(Line::from(Span::styled(format!("    {}", wrapped_line), Style::default().fg(Color::DarkGray))));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[Enter] Select  [Esc] Back", Style::default().fg(Color::DarkGray))));
    
    let block = Block::default()
        .title(" Select Class ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(Paragraph::new(lines).block(block), popup);
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
        Line::from("    o    Auto-explore"),
        Line::from("    e    End turn (wait)"),
        Line::from("    1-3  Use inventory item"),
        Line::from("    S    Save game"),
        Line::from("    L    Load game"),
        Line::from(""),
        Line::from("  Menus:"),
        Line::from("    i    Inventory"),
        Line::from("    c    Crafting"),
        Line::from("    q    Quest log"),
        Line::from("    Esc  Pause menu"),
        Line::from(""),
        Line::from(Span::styled("Press any key to return", Style::default().fg(Color::DarkGray))),
    ];
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}

/// Render the pause menu overlay
pub fn render_pause_menu(frame: &mut Frame, selected_index: usize) {
    let area = frame.area();
    
    // Centered popup
    let width = 24u16;
    let height = (PAUSE_OPTIONS.len() + 4) as u16;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);
    
    frame.render_widget(Clear, popup);
    
    let items: Vec<ListItem> = PAUSE_OPTIONS.iter().enumerate().map(|(i, opt)| {
        let style = if i == selected_index {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default()
        };
        ListItem::new(format!("  {}  ", opt)).style(style)
    }).collect();
    
    let block = Block::default()
        .title(" Paused ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(List::new(items).block(block), popup);
}
