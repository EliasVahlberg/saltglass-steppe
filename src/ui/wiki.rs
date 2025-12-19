//! Wiki/Codex menu for viewing game data

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
};

use crate::game::{all_item_ids, all_enemy_ids, get_item_def, get_enemy_def, get_npc_def};
use crate::game::npc::all_npc_ids;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum WikiTab { #[default] Items, Enemies, NPCs }

impl WikiTab {
    const ALL: [WikiTab; 3] = [WikiTab::Items, WikiTab::Enemies, WikiTab::NPCs];
    fn name(&self) -> &'static str {
        match self { WikiTab::Items => "Items", WikiTab::Enemies => "Enemies", WikiTab::NPCs => "NPCs" }
    }
}

#[derive(Default)]
pub struct WikiMenu {
    pub active: bool,
    pub tab: WikiTab,
    pub selected: usize,
    pub scroll: usize,
}

impl WikiMenu {
    pub fn open(&mut self) { self.active = true; self.selected = 0; self.scroll = 0; }
    pub fn close(&mut self) { self.active = false; }
    
    pub fn next_tab(&mut self) {
        let idx = WikiTab::ALL.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = WikiTab::ALL[(idx + 1) % WikiTab::ALL.len()];
        self.selected = 0; self.scroll = 0;
    }
    
    pub fn prev_tab(&mut self) {
        let idx = WikiTab::ALL.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = WikiTab::ALL[(idx + WikiTab::ALL.len() - 1) % WikiTab::ALL.len()];
        self.selected = 0; self.scroll = 0;
    }
    
    pub fn navigate(&mut self, dy: i32) {
        let count = self.entry_count();
        if count > 0 {
            self.selected = (self.selected as i32 + dy).rem_euclid(count as i32) as usize;
        }
    }
    
    fn entry_count(&self) -> usize {
        match self.tab {
            WikiTab::Items => all_item_ids().len(),
            WikiTab::Enemies => all_enemy_ids().len(),
            WikiTab::NPCs => all_npc_ids().len(),
        }
    }
}

pub fn render_wiki(frame: &mut Frame, menu: &WikiMenu) {
    let area = frame.area();
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);
    
    // Tabs
    let tabs = Tabs::new(WikiTab::ALL.iter().map(|t| t.name()))
        .select(WikiTab::ALL.iter().position(|t| *t == menu.tab).unwrap_or(0))
        .block(Block::default().title(" Wiki ").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).bold());
    frame.render_widget(tabs, chunks[0]);
    
    // Content: list on left, details on right
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);
    
    // Entry list
    let entries = get_entries(menu.tab);
    let items: Vec<ListItem> = entries.iter().enumerate().map(|(i, (name, _))| {
        let style = if i == menu.selected {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else { Style::default() };
        ListItem::new(format!(" {} ", name)).style(style)
    }).collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(list, content[0]);
    
    // Details panel
    let detail_text = entries.get(menu.selected)
        .map(|(_, desc)| desc.as_str())
        .unwrap_or("");
    
    let detail = Paragraph::new(detail_text)
        .wrap(Wrap { trim: true })
        .block(Block::default().title(" Details ").borders(Borders::ALL));
    frame.render_widget(detail, content[1]);
    
    // Help
    let help = " [Tab/h/l] Switch tab | [j/k] Navigate | [Esc/w] Close ";
    let help_area = Rect::new(area.x, area.y + area.height.saturating_sub(1), area.width, 1);
    frame.render_widget(Paragraph::new(help).style(Style::default().fg(Color::DarkGray)), help_area);
}

fn get_entries(tab: WikiTab) -> Vec<(String, String)> {
    match tab {
        WikiTab::Items => {
            all_item_ids().into_iter().filter_map(|id| {
                let def = get_item_def(id)?;
                let mut desc = format!("{}\n\n{}", def.name, def.description);
                if def.heal > 0 { desc.push_str(&format!("\n\nHeals: {} HP", def.heal)); }
                if def.armor_value > 0 { desc.push_str(&format!("\nArmor: +{}", def.armor_value)); }
                if def.usable { desc.push_str("\n\n[Consumable]"); }
                if def.equip_slot.is_some() { desc.push_str(&format!("\nSlot: {}", def.equip_slot.as_ref().unwrap())); }
                Some((def.name.clone(), desc))
            }).collect()
        }
        WikiTab::Enemies => {
            all_enemy_ids().into_iter().filter_map(|id| {
                let def = get_enemy_def(id)?;
                let desc = format!(
                    "{}\n\n{}\n\nHP: {}\nDamage: {}-{}\nArmor: {}\nXP: {}\nBehavior: {:?}",
                    def.name, def.description, def.max_hp, def.damage_min, def.damage_max,
                    def.armor, def.xp_value, def.demeanor
                );
                Some((def.name.clone(), desc))
            }).collect()
        }
        WikiTab::NPCs => {
            all_npc_ids().into_iter().filter_map(|id| {
                let def = get_npc_def(id)?;
                let desc = format!("{}\n\n{}", def.name, def.description);
                Some((def.name.clone(), desc))
            }).collect()
        }
    }
}
