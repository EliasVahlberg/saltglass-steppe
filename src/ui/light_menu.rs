use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use crate::game::state::GameState;

#[derive(Default)]
pub struct LightMenu {
    pub active: bool,
    pub selected_index: usize,
}

impl LightMenu {
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
        self.selected_index = (self.selected_index as i32 + delta).rem_euclid(max_items as i32) as usize;
    }
}

pub fn render_light_menu(frame: &mut Frame, area: Rect, state: &GameState, menu: &LightMenu) {
    let block = Block::default()
        .title(" Light Manipulation (L) ")
        .borders(Borders::ALL);
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Energy bar
            Constraint::Length(4),  // Active beams
            Constraint::Min(0),     // Abilities
        ])
        .split(inner);
    
    // Light energy display
    let energy_text = format!(
        "Light Energy: {}\nSources: {} | Surfaces: {}",
        state.player.light_system.light_energy,
        state.player.light_system.light_sources.len(),
        state.player.light_system.refraction_surfaces.len()
    );
    let energy = Paragraph::new(energy_text);
    frame.render_widget(energy, chunks[0]);
    
    // Active beams display
    let beams_text = if state.player.light_system.active_beams.is_empty() {
        "No active beams".to_string()
    } else {
        format!("Active Beams: {}", state.player.light_system.active_beams.len())
    };
    let beams = Paragraph::new(beams_text);
    frame.render_widget(beams, chunks[1]);
    
    // Abilities list
    let abilities = [
        ("Create Beam", 10),
        ("Place Mirror", 15),
        ("Focus Light", 20),
        ("Absorb Light", 5),
    ];
    
    let items: Vec<ListItem> = abilities.iter().enumerate().map(|(i, (name, cost))| {
        let text = format!("{} ({} energy)", name, cost);
        let style = if state.player.light_system.light_energy >= *cost {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let style = if i == menu.selected_index {
            style.bg(Color::DarkGray)
        } else {
            style
        };
        ListItem::new(text).style(style)
    }).collect();
    
    let list = List::new(items);
    frame.render_widget(list, chunks[2]);
}