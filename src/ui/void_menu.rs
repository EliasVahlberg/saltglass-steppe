use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use crate::game::state::GameState;
use crate::game::void_energy::VoidAbility;

#[derive(Default)]
pub struct VoidMenu {
    pub active: bool,
    pub selected_index: usize,
}

impl VoidMenu {
    pub fn toggle(&mut self) { self.active = !self.active; if self.active { self.selected_index = 0; } }
    pub fn close(&mut self) { self.active = false; }
    pub fn navigate(&mut self, delta: i32, max_items: usize) {
        if max_items == 0 { return; }
        self.selected_index = (self.selected_index as i32 + delta).rem_euclid(max_items as i32) as usize;
    }
}

pub fn render_void_menu(frame: &mut Frame, area: Rect, state: &GameState, menu: &VoidMenu) {
    let block = Block::default().title(" Void Energy (v) ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(inner);

    // Stats section
    let exposure_level = format!("{:?}", state.void_system.exposure_level());
    let stats_text = format!(
        "Exposure: {} ({})\nEnergy: {}/{}{}",
        state.void_system.void_exposure,
        exposure_level,
        state.void_system.void_energy,
        state.void_system.max_void_energy,
        if state.void_system.phase_walk_turns > 0 {
            format!("\nPhase Walk: {} turns remaining", state.void_system.phase_walk_turns)
        } else {
            String::new()
        }
    );
    let stats = Paragraph::new(stats_text);
    frame.render_widget(stats, chunks[0]);

    // Ability list
    if state.void_system.unlocked_abilities.is_empty() {
        let no_abilities = Paragraph::new("No void abilities unlocked yet.");
        frame.render_widget(no_abilities, chunks[1]);
    } else {
        let items: Vec<ListItem> = state.void_system.unlocked_abilities.iter().enumerate().map(|(i, ability)| {
            let name = match ability {
                VoidAbility::VoidStep => "Void Step",
                VoidAbility::RealityRend => "Reality Rend",
                VoidAbility::VoidShield => "Void Shield",
                VoidAbility::PhaseWalk => "Phase Walk",
                VoidAbility::VoidDrain => "Void Drain",
            };
            let cost = ability.energy_cost();
            let text = format!("{} ({} energy)", name, cost);
            let style = if state.void_system.void_energy >= cost {
                Style::default().fg(Color::Green)
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
        frame.render_widget(list, chunks[1]);
    }
}

pub fn get_selected_ability(menu: &VoidMenu, state: &GameState) -> Option<VoidAbility> {
    state.void_system.unlocked_abilities.get(menu.selected_index).copied()
}