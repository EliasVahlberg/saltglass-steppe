//! Wiki/Codex menu for viewing game data

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
};

use crate::game::npc::all_npc_ids;
use crate::game::{
    MetaProgress, all_enemy_ids, all_item_ids, get_enemy_def, get_item_def, get_npc_def,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum WikiTab {
    #[default]
    Items,
    Enemies,
    NPCs,
}

impl WikiTab {
    const ALL: [WikiTab; 3] = [WikiTab::Items, WikiTab::Enemies, WikiTab::NPCs];
    fn name(&self) -> &'static str {
        match self {
            WikiTab::Items => "Items",
            WikiTab::Enemies => "Enemies",
            WikiTab::NPCs => "NPCs",
        }
    }
}

#[derive(Default)]
pub struct WikiMenu {
    pub active: bool,
    pub tab: WikiTab,
    pub selected: usize,
}

impl WikiMenu {
    pub fn open(&mut self) {
        self.active = true;
        self.selected = 0;
    }
    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn next_tab(&mut self) {
        let idx = WikiTab::ALL
            .iter()
            .position(|t| *t == self.tab)
            .unwrap_or(0);
        self.tab = WikiTab::ALL[(idx + 1) % WikiTab::ALL.len()];
        self.selected = 0;
    }

    pub fn prev_tab(&mut self) {
        let idx = WikiTab::ALL
            .iter()
            .position(|t| *t == self.tab)
            .unwrap_or(0);
        self.tab = WikiTab::ALL[(idx + WikiTab::ALL.len() - 1) % WikiTab::ALL.len()];
        self.selected = 0;
    }

    pub fn navigate(&mut self, dy: i32, count: usize) {
        if count > 0 {
            self.selected = (self.selected as i32 + dy).rem_euclid(count as i32) as usize;
        }
    }
}

pub fn render_wiki(frame: &mut Frame, menu: &WikiMenu, meta: &MetaProgress) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Tabs
    let tabs = Tabs::new(WikiTab::ALL.iter().map(|t| t.name()))
        .select(
            WikiTab::ALL
                .iter()
                .position(|t| *t == menu.tab)
                .unwrap_or(0),
        )
        .block(Block::default().title(" Wiki ").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).bold());
    frame.render_widget(tabs, chunks[0]);

    // Content
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    let entries = get_entries(menu.tab, meta);
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, (name, _, discovered))| {
            let style = if i == menu.selected {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else if !discovered {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            let display = if *discovered {
                name.clone()
            } else {
                "???".to_string()
            };
            ListItem::new(format!(" {} ", display)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));
    frame.render_widget(list, content[0]);

    // Details
    let detail_text = entries
        .get(menu.selected)
        .map(|(_, desc, discovered)| {
            if *discovered {
                desc.as_str()
            } else {
                "Not yet discovered."
            }
        })
        .unwrap_or("");

    let detail = Paragraph::new(detail_text)
        .wrap(Wrap { trim: true })
        .block(Block::default().title(" Details ").borders(Borders::ALL));
    frame.render_widget(detail, content[1]);

    let help = " [Tab/h/l] Switch tab | [j/k] Navigate | [Esc/w] Close ";
    let help_area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(1),
        area.width,
        1,
    );
    frame.render_widget(
        Paragraph::new(help).style(Style::default().fg(Color::DarkGray)),
        help_area,
    );
}

fn get_entries(tab: WikiTab, meta: &MetaProgress) -> Vec<(String, String, bool)> {
    match tab {
        WikiTab::Items => all_item_ids()
            .into_iter()
            .filter_map(|id| {
                let def = get_item_def(id)?;
                let discovered = meta.discovered_items.contains(id);
                let mut desc = format!("{}\n\n{}", def.name, def.description);
                if def.heal > 0 {
                    desc.push_str(&format!("\n\nHeals: {} HP", def.heal));
                }
                if def.armor_value > 0 {
                    desc.push_str(&format!("\nArmor: +{}", def.armor_value));
                }
                if def.usable {
                    desc.push_str("\n\n[Consumable]");
                }
                if def.equip_slot.is_some() {
                    desc.push_str(&format!("\nSlot: {}", def.equip_slot.as_ref().unwrap()));
                }
                Some((def.name.clone(), desc, discovered))
            })
            .collect(),
        WikiTab::Enemies => all_enemy_ids()
            .into_iter()
            .filter_map(|id| {
                let def = get_enemy_def(id)?;
                let discovered = meta.discovered_enemies.contains(id);
                let desc = format!(
                    "{}\n\n{}\n\nHP: {}\nDamage: {}-{}\nArmor: {}\nXP: {}\nBehavior: {:?}",
                    def.name,
                    def.description,
                    def.max_hp,
                    def.damage_min,
                    def.damage_max,
                    def.armor,
                    def.xp_value,
                    def.demeanor
                );
                Some((def.name.clone(), desc, discovered))
            })
            .collect(),
        WikiTab::NPCs => all_npc_ids()
            .into_iter()
            .filter_map(|id| {
                let def = get_npc_def(id)?;
                let discovered = meta.discovered_npcs.contains(id);
                let desc = format!("{}\n\n{}", def.name, def.description);
                Some((def.name.clone(), desc, discovered))
            })
            .collect(),
    }
}
