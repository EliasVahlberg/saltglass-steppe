//! Psychic abilities menu for combat

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use crate::game::{GameState, psychic::{get_ability_def, PsychicCategory}};

#[derive(Default)]
pub struct PsychicMenu {
    pub active: bool,
    pub selected_index: usize,
}

impl PsychicMenu {
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if self.active {
            self.selected_index = 0;
        }
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn navigate(&mut self, delta: i32, max_items: usize) {
        if max_items == 0 { return; }
        let new_index = (self.selected_index as i32 + delta).rem_euclid(max_items as i32) as usize;
        self.selected_index = new_index;
    }

    pub fn get_selected_ability(&self, state: &GameState) -> Option<String> {
        if self.selected_index < state.psychic.unlocked_abilities.len() {
            Some(state.psychic.unlocked_abilities[self.selected_index].clone())
        } else {
            None
        }
    }
}

fn render_psychic_menu_internal(frame: &mut Frame, area: Rect, state: &GameState, menu: &PsychicMenu) {
    if !menu.active { return; }

    let block = Block::default()
        .title(" Psychic Abilities ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left side: ability list
    let abilities: Vec<ListItem> = state.psychic.unlocked_abilities.iter()
        .enumerate()
        .map(|(i, id)| {
            let def = get_ability_def(id).unwrap();
            let on_cooldown = state.psychic.cooldowns.get(id).unwrap_or(&0) > &0;
            let can_afford = state.psychic.coherence >= def.coherence_cost;
            
            let style = if i == menu.selected_index {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if on_cooldown {
                Style::default().fg(Color::DarkGray)
            } else if !can_afford {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let cooldown_text = if on_cooldown {
                format!(" ({})", state.psychic.cooldowns.get(id).unwrap())
            } else {
                String::new()
            };

            ListItem::new(format!("{} [{}]{}", def.name, def.coherence_cost, cooldown_text))
                .style(style)
        })
        .collect();

    frame.render_widget(List::new(abilities), chunks[0]);

    // Right side: ability details
    if let Some(selected_id) = state.psychic.unlocked_abilities.get(menu.selected_index) {
        if let Some(def) = get_ability_def(selected_id) {
            let category_color = match def.category {
                PsychicCategory::Telepathy => Color::Magenta,
                PsychicCategory::Probability => Color::Yellow,
                PsychicCategory::Energy => Color::Red,
                PsychicCategory::Phasing => Color::Cyan,
                PsychicCategory::Temporal => Color::Green,
            };

            let details = vec![
                Line::from(vec![
                    Span::styled("Category: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:?}", def.category), Style::default().fg(category_color)),
                ]),
                Line::from(vec![
                    Span::styled("Cost: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{} Coherence", def.coherence_cost), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("Cooldown: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{} turns", def.cooldown), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(Span::styled(&def.description, Style::default().fg(Color::White))),
            ];

            frame.render_widget(Paragraph::new(details), chunks[1]);
        }
    }

    // Instructions at bottom
    let instructions = Paragraph::new("↑↓: Navigate | Enter: Use | Esc: Close")
        .style(Style::default().fg(Color::DarkGray));
    
    let bottom_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - 2,
        width: area.width - 2,
        height: 1,
    };
    frame.render_widget(instructions, bottom_area);
}

pub fn render_psychic_menu(frame: &mut Frame, area: Rect, state: &GameState, menu: &PsychicMenu) {
    render_psychic_menu_internal(frame, area, state, menu);
}
