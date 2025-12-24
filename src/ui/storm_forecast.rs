use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crate::game::GameState;

/// Render the storm forecast panel showing upcoming storm information
pub fn render_storm_forecast(frame: &mut Frame, area: Rect, state: &GameState) {
    let block = Block::default()
        .title(" Storm Forecast ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let storm_color = if state.storm.turns_until <= 3 { 
        Color::Red 
    } else if state.storm.turns_until <= 5 { 
        Color::Yellow 
    } else { 
        Color::Green 
    };

    // Create intensity bar (5 blocks max)
    let intensity_blocks = state.storm.intensity.min(5);
    let mut intensity_bar = String::new();
    for i in 0..5 {
        if i < intensity_blocks {
            intensity_bar.push('█');
        } else {
            intensity_bar.push('░');
        }
    }

    let mut lines = vec![
        Line::from(vec![
            Span::raw("Turns: "),
            Span::styled(format!("{}", state.storm.turns_until), Style::default().fg(storm_color)),
        ]),
        Line::from(vec![
            Span::raw("Intensity: "),
            Span::styled(intensity_bar, Style::default().fg(storm_color)),
        ]),
    ];

    // Add edit types
    if !state.storm.edit_types.is_empty() {
        lines.push(Line::from(Span::styled("Edits:", Style::default().fg(Color::Cyan))));
        for edit_type in &state.storm.edit_types {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(edit_type.display_name(), Style::default().fg(Color::White)),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}
