//! Quest log UI - fullscreen quest display

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
};
use crate::game::{GameState, get_quest_def};

#[derive(Default)]
pub struct QuestLogMenu {
    pub active: bool,
    pub selected: usize,
}

impl QuestLogMenu {
    pub fn open(&mut self) {
        self.active = true;
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn navigate(&mut self, delta: i32, max: usize) {
        if max == 0 { return; }
        let new = self.selected as i32 + delta;
        self.selected = new.clamp(0, max.saturating_sub(1) as i32) as usize;
    }
}

pub fn render_quest_log(frame: &mut Frame, menu: &QuestLogMenu, state: &GameState) {
    let area = frame.area();
    let block = Block::default()
        .title(" Quest Log (Esc to close) ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(inner);

    // Quest list
    let mut items: Vec<ListItem> = Vec::new();
    
    // Active quests
    items.push(ListItem::new(Line::from(Span::styled("── Active ──", Style::default().fg(Color::Yellow).bold()))));
    for quest in &state.quest_log.active {
        if let Some(def) = get_quest_def(&quest.quest_id) {
            let progress: usize = quest.objectives.iter().filter(|o| o.completed).count();
            let total = quest.objectives.len();
            items.push(ListItem::new(format!("  {} ({}/{})", def.name, progress, total)));
        }
    }
    
    // Completed quests
    if !state.quest_log.completed.is_empty() {
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(Span::styled("── Completed ──", Style::default().fg(Color::Green).bold()))));
        for quest_id in &state.quest_log.completed {
            if let Some(def) = get_quest_def(quest_id) {
                items.push(ListItem::new(format!("  ✓ {}", def.name)));
            }
        }
    }

    let list_block = Block::default().title(" Quests ").borders(Borders::ALL);
    let mut list_state = ListState::default();
    list_state.select(Some(menu.selected));
    frame.render_stateful_widget(
        List::new(items).block(list_block).highlight_style(Style::default().bg(Color::DarkGray)),
        chunks[0],
        &mut list_state,
    );

    // Quest details
    let detail_block = Block::default().title(" Details ").borders(Borders::ALL);
    let detail_inner = detail_block.inner(chunks[1]);
    frame.render_widget(detail_block, chunks[1]);

    // Find selected quest
    let active_count = state.quest_log.active.len();
    let header_offset = 1; // "── Active ──" header
    
    if menu.selected > 0 && menu.selected <= active_count {
        let quest_idx = menu.selected - header_offset;
        if quest_idx < state.quest_log.active.len() {
            let quest = &state.quest_log.active[quest_idx];
            if let Some(def) = get_quest_def(&quest.quest_id) {
                let mut lines = vec![
                    Line::from(Span::styled(&def.name, Style::default().fg(Color::Yellow).bold())),
                    Line::from(""),
                    Line::from(&def.description[..]),
                    Line::from(""),
                    Line::from(Span::styled("Objectives:", Style::default().bold())),
                ];
                
                for (i, obj) in def.objectives.iter().enumerate() {
                    let progress = &quest.objectives[i];
                    let status = if progress.completed {
                        Span::styled("✓ ", Style::default().fg(Color::Green))
                    } else {
                        Span::styled("○ ", Style::default().fg(Color::DarkGray))
                    };
                    let text = format!("{} ({}/{})", obj.description, progress.current, progress.target);
                    lines.push(Line::from(vec![status, Span::raw(text)]));
                }
                
                if def.reward.xp > 0 || !def.reward.items.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("Rewards:", Style::default().bold())));
                    if def.reward.xp > 0 {
                        lines.push(Line::from(format!("  {} XP", def.reward.xp)));
                    }
                    for item_id in &def.reward.items {
                        lines.push(Line::from(format!("  {}", item_id)));
                    }
                }
                
                frame.render_widget(Paragraph::new(lines), detail_inner);
            }
        }
    }
}
