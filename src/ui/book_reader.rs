use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::game::book::get_book_def;
use crate::ui::input::UiState;

pub fn render_book_reader(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.book_reader.active {
        return;
    }

    let book_def = match get_book_def(&ui_state.book_reader.book_id) {
        Some(def) => def,
        None => return,
    };

    let area = f.area();
    let popup_area = Rect::new(
        area.width / 2 - 35,
        area.height / 2 - 15,
        70,
        30,
    );

    // Clear background
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        popup_area,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {} ", book_def.title),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::White));

    f.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Author
            Constraint::Min(1),     // Content
            Constraint::Length(1),  // Footer
        ])
        .split(inner_area);

    // Author
    f.render_widget(
        Paragraph::new(format!("By {}", book_def.author))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        chunks[0],
    );

    // Page Content
    if let Some(page) = book_def.pages.get(ui_state.book_reader.current_page) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),     // Text
                Constraint::Length(10), // Illustration (optional)
            ])
            .split(chunks[1]);

        // Text
        f.render_widget(
            Paragraph::new(page.text.clone())
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left),
            content_chunks[0],
        );

        // Illustration
        if let Some(art) = &page.illustration {
            f.render_widget(
                Paragraph::new(art.clone())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Cyan)),
                content_chunks[1],
            );
        }
    }

    // Footer (Page number and controls)
    let footer_text = format!(
        "Page {}/{} | <Left/Right> Turn Page | <Esc> Close",
        ui_state.book_reader.current_page + 1,
        book_def.pages.len()
    );
    
    f.render_widget(
        Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        chunks[2],
    );
}
