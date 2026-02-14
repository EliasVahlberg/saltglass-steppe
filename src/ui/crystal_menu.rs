use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use crate::game::state::GameState;
use crate::game::crystal_resonance::CrystalFrequency;

#[derive(Default)]
pub struct CrystalMenu {
    pub active: bool,
    pub selected_index: usize,
}

impl CrystalMenu {
    pub fn toggle(&mut self) { self.active = !self.active; if self.active { self.selected_index = 0; } }
    pub fn close(&mut self) { self.active = false; }
    pub fn navigate(&mut self, delta: i32, max_items: usize) {
        if max_items == 0 { return; }
        self.selected_index = (self.selected_index as i32 + delta).rem_euclid(max_items as i32) as usize;
    }
}

pub fn render_crystal_menu(frame: &mut Frame, area: Rect, state: &GameState, menu: &CrystalMenu) {
    let block = Block::default()
        .title(" Crystal Resonance (V) ")
        .borders(Borders::ALL);
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(inner);
    
    // Stats section
    let stats_text = format!(
        "Energy: {}/{}\nFormations: {} nearby crystals\nHarmonics: {} active",
        state.player.crystal_system.resonance_energy,
        state.player.crystal_system.max_resonance_energy,
        state.player.crystal_system.crystal_formations.len(),
        state.player.crystal_system.active_harmonics.len()
    );
    let stats = Paragraph::new(stats_text);
    frame.render_widget(stats, chunks[0]);
    
    // Frequency list
    let frequencies = CrystalFrequency::all();
    let descriptions = ["Structural", "Psychic", "Healing", "Combat", "Chaotic"];
    let colors = [Color::Blue, Color::Magenta, Color::Green, Color::Red, Color::Yellow];
    
    let items: Vec<ListItem> = frequencies.iter().enumerate().map(|(i, freq)| {
        let level = state.player.crystal_system.frequency_attunement.get(freq).copied().unwrap_or(0);
        let text = format!("{}: Level {} ({})", freq.name(), level, descriptions[i]);
        let mut style = Style::default().fg(colors[i]);
        if i == menu.selected_index {
            style = style.bg(Color::DarkGray);
        }
        ListItem::new(text).style(style)
    }).collect();
    
    let list = List::new(items);
    frame.render_widget(list, chunks[1]);
}