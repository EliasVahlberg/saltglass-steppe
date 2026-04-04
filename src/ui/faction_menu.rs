use crate::game::faction;
use crate::game::state::GameState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

#[derive(Default)]
pub struct FactionMenu {
    pub active: bool,
    pub selected_index: usize,
}

impl FactionMenu {
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
        if max_items == 0 {
            return;
        }
        let new_index = (self.selected_index as i32 + delta).rem_euclid(max_items as i32) as usize;
        self.selected_index = new_index;
    }
}

pub fn faction_count() -> usize {
    faction::all_faction_ids().len()
}

pub fn render_faction_menu(frame: &mut Frame, area: Rect, state: &GameState, menu: &FactionMenu) {
    let block = Block::default()
        .title(" Faction Reputation (F) ")
        .borders(Borders::ALL);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let faction_ids = faction::all_faction_ids();
    let faction_items: Vec<ListItem> = faction_ids
        .iter()
        .enumerate()
        .map(|(i, faction_id)| {
            let faction_def = faction::get_faction(faction_id).expect("Faction not found");
            let reputation = state.get_reputation(faction_id);
            let standing = faction::get_standing(reputation);
            let color = faction::get_standing_color(reputation);

            let text = format!("{}: {} ({})", faction_def.name, reputation, standing);
            let style = if i == menu.selected_index {
                Style::default().bg(Color::DarkGray).fg(color)
            } else {
                Style::default().fg(color)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(faction_items);
    frame.render_widget(list, chunks[0]);

    if let Some(faction_id) = faction_ids.get(menu.selected_index)
        && let Some(faction_def) = faction::get_faction(faction_id)
    {
        let description = Paragraph::new(faction_def.description.as_str())
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(description, chunks[1]);
    }
}
