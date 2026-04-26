//! Adaptation choice UI — shown when player reaches a refraction threshold.
//! Presents 3 adaptation options; player must choose one (Escape disabled).

use crate::game::{GameState, adaptation::get_adaptation_def};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Default)]
pub struct AdaptationChoiceMenu {
    pub active: bool,
    pub selected: usize,
    /// The adaptation IDs being offered.
    pub choices: Vec<String>,
}

impl AdaptationChoiceMenu {
    pub fn open(&mut self, choices: Vec<String>) {
        self.choices = choices;
        self.selected = 0;
        self.active = true;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.choices.clear();
    }

    pub fn navigate(&mut self, delta: i32) {
        if self.choices.is_empty() { return; }
        let n = self.choices.len();
        self.selected = ((self.selected as i32 + delta).rem_euclid(n as i32)) as usize;
    }

    /// Returns the chosen adaptation ID and closes the menu.
    pub fn confirm(&mut self) -> Option<String> {
        if !self.active || self.choices.is_empty() { return None; }
        let id = self.choices.get(self.selected).cloned();
        self.close();
        id
    }
}

pub fn render_adaptation_choice(
    frame: &mut Frame,
    menu: &AdaptationChoiceMenu,
    _state: &GameState,
) {
    if !menu.active || menu.choices.is_empty() { return; }

    let area = frame.area();

    // Dim overlay
    let overlay = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(Clear, area);
    frame.render_widget(overlay, area);

    // Outer panel
    let panel_width = (area.width).min(90);
    let panel_height = 24u16;
    let panel = Rect {
        x: area.x + (area.width.saturating_sub(panel_width)) / 2,
        y: area.y + (area.height.saturating_sub(panel_height)) / 2,
        width: panel_width,
        height: panel_height,
    };

    let outer = Block::default()
        .title(" ⬡ Your body is ready to change ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(outer, panel);

    let subtitle = Paragraph::new("Choose one adaptation. This choice is permanent.")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(subtitle, Rect { x: panel.x + 1, y: panel.y + 2, width: panel.width - 2, height: 1 });

    // Card layout — divide inner width into N equal columns
    let n = menu.choices.len().max(1);
    let inner_width = panel.width.saturating_sub(4);
    let card_width = (inner_width / n as u16).saturating_sub(1);
    let card_height = panel_height - 7;

    for (i, id) in menu.choices.iter().enumerate() {
        let card_x = panel.x + 2 + i as u16 * (card_width + 1);
        let card_y = panel.y + 4;
        let card_area = Rect { x: card_x, y: card_y, width: card_width, height: card_height };

        let selected = i == menu.selected;
        let border_style = if selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let def = get_adaptation_def(id);
        let name = def.map(|d| d.name.as_str()).unwrap_or(id.as_str());
        let category = def
            .and_then(|d| d.category.as_ref())
            .map(|c| c.display_name())
            .unwrap_or("Unknown");
        let tier = def.map(|d| d.tier).unwrap_or(1);
        let visibility = def
            .and_then(|d| d.faction_visibility.as_ref())
            .map(|v| v.display_name())
            .unwrap_or("Low");
        let description = def.map(|d| d.description.as_str()).unwrap_or("");

        let title = if selected {
            format!(" ► {} ", name)
        } else {
            format!("   {}  ", name)
        };

        let card_block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(border_style);
        frame.render_widget(card_block, card_area);

        // Category + tier line
        let meta = format!("[{}] Tier {}", category, tier);
        let meta_style = if selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };
        let meta_widget = Paragraph::new(meta).style(meta_style).alignment(Alignment::Center);
        frame.render_widget(meta_widget, Rect { x: card_x + 1, y: card_y + 2, width: card_width - 2, height: 1 });

        // Description
        let desc_widget = Paragraph::new(description)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        frame.render_widget(desc_widget, Rect {
            x: card_x + 1,
            y: card_y + 4,
            width: card_width - 2,
            height: card_height.saturating_sub(7),
        });

        // Visibility line at bottom of card
        let vis_color = match visibility {
            "Low"      => Color::Green,
            "Moderate" => Color::Yellow,
            "High"     => Color::Red,
            "Extreme"  => Color::Magenta,
            _          => Color::Gray,
        };
        let vis_widget = Paragraph::new(format!("Visibility: {}", visibility))
            .style(Style::default().fg(vis_color))
            .alignment(Alignment::Center);
        frame.render_widget(vis_widget, Rect {
            x: card_x + 1,
            y: card_y + card_height - 3,
            width: card_width - 2,
            height: 1,
        });
    }

    // Controls hint
    let hint = Paragraph::new("← → Navigate    Enter Confirm")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, Rect {
        x: panel.x + 1,
        y: panel.y + panel_height - 2,
        width: panel.width - 2,
        height: 1,
    });
}
