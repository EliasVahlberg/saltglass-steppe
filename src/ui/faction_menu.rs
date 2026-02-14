use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use once_cell::sync::Lazy;
use serde::Deserialize;
use crate::game::state::GameState;

#[derive(Clone, Debug, Deserialize)]
struct FactionDef {
    id: String,
    name: String,
    description: String,
    #[allow(dead_code)]
    color: String,
}

#[derive(Clone, Debug, Deserialize)]
struct FactionsFile {
    factions: Vec<FactionDef>,
}

static FACTIONS: Lazy<FactionsFile> = Lazy::new(|| {
    let data = include_str!("../../data/factions.json");
    serde_json::from_str(data).expect("Failed to parse factions.json")
});

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
    FACTIONS.factions.len()
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

    let faction_items: Vec<ListItem> = FACTIONS.factions.iter().enumerate().map(|(i, faction)| {
        let reputation = state.get_reputation(&faction.id);
        let (status, color) = match reputation {
            r if r >= 50 => ("Allied", Color::Green),
            r if r >= 25 => ("Friendly", Color::Cyan),
            r if r >= -24 => ("Neutral", Color::White),
            r if r >= -49 => ("Distrusted", Color::Yellow),
            _ => ("Hostile", Color::Red),
        };
        
        let text = format!("{}: {} {}", faction.name, reputation, status);
        let style = if i == menu.selected_index {
            Style::default().bg(Color::DarkGray).fg(color)
        } else {
            Style::default().fg(color)
        };
        
        ListItem::new(text).style(style)
    }).collect();

    let list = List::new(faction_items);
    frame.render_widget(list, chunks[0]);

    if let Some(selected_faction) = FACTIONS.factions.get(menu.selected_index) {
        let description = Paragraph::new(selected_faction.description.as_str())
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(description, chunks[1]);
    }
}