//! HUD rendering - side panel with stats, bottom panel with log

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, List, ListItem},
};
use crate::game::{GameState, get_item_def, get_quest_def};
use crate::game::equipment::EquipSlot;

/// Render the side panel with player stats and equipment
pub fn render_side_panel(frame: &mut Frame, area: Rect, state: &GameState) {
    let block = Block::default().title(" Player ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Stats
            Constraint::Length(9),  // Equipment
            Constraint::Min(3),     // Quests
        ])
        .split(inner);

    // Stats section
    let hp_color = if state.player_hp <= 5 { Color::Red } 
        else if state.player_hp <= state.player_max_hp / 2 { Color::Yellow }
        else { Color::Green };
    let storm_color = if state.storm.turns_until <= 3 { Color::Red } 
        else if state.storm.turns_until <= 5 { Color::Yellow } 
        else { Color::Green };

    let stats = vec![
        Line::from(vec![
            Span::raw("HP: "),
            Span::styled(format!("{}/{}", state.player_hp, state.player_max_hp), Style::default().fg(hp_color)),
        ]),
        Line::from(vec![
            Span::raw("AP: "),
            Span::styled(format!("{}/{}", state.player_ap, state.player_max_ap), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Lvl: "),
            Span::styled(format!("{}", state.player_level), Style::default().fg(Color::Yellow)),
            Span::raw(format!(" XP: {}", state.player_xp)),
        ]),
        Line::from(vec![
            Span::raw("Ref: "),
            Span::styled(format!("{}", state.refraction), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::raw("Storm: "),
            Span::styled(format!("{} turns", state.storm.turns_until), Style::default().fg(storm_color)),
        ]),
    ];
    frame.render_widget(Paragraph::new(stats), chunks[0]);

    // Equipment section
    let mut equip_lines = vec![Line::from(Span::styled("─Equipment─", Style::default().fg(Color::DarkGray)))];
    for slot in EquipSlot::all() {
        let item_name = state.equipment.get(*slot)
            .and_then(|id| get_item_def(id))
            .map(|d| d.name.as_str())
            .unwrap_or("-");
        equip_lines.push(Line::from(format!("{}: {}", slot.display_name(), item_name)));
    }
    frame.render_widget(Paragraph::new(equip_lines), chunks[1]);

    // Active quests section
    let mut quest_lines = vec![Line::from(Span::styled("─Quests─", Style::default().fg(Color::DarkGray)))];
    if state.quest_log.active.is_empty() {
        quest_lines.push(Line::from(Span::styled("(none)", Style::default().fg(Color::DarkGray))));
    } else {
        for quest in state.quest_log.active.iter().take(3) {
            if let Some(def) = get_quest_def(&quest.quest_id) {
                let progress: usize = quest.objectives.iter().filter(|o| o.completed).count();
                let total = quest.objectives.len();
                quest_lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{} ({}/{})", def.name, progress, total)),
                ]));
            }
        }
    }
    frame.render_widget(Paragraph::new(quest_lines), chunks[2]);
}

/// Render the bottom panel with messages and hotkeys
pub fn render_bottom_panel(frame: &mut Frame, area: Rect, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Message log
    let log_block = Block::default().title(" Log ").borders(Borders::ALL);
    let log_inner = log_block.inner(chunks[0]);
    frame.render_widget(log_block, chunks[0]);
    
    let msg_count = log_inner.height as usize;
    let messages: Vec<ListItem> = state.messages.iter()
        .rev()
        .take(msg_count)
        .rev()
        .map(|m| ListItem::new(m.as_str()))
        .collect();
    frame.render_widget(List::new(messages), log_inner);

    // Hotkeys
    let hotkeys_block = Block::default().title(" Keys ").borders(Borders::ALL);
    let hotkeys = vec![
        "hjkl  Move",
        "i     Inventory",
        "c     Craft",
        "Q     Quest log",
        "x     Look mode",
        "e     End turn",
        "o     Auto-explore",
        "S/L   Save/Load",
        "q     Quit",
    ];
    let hotkey_lines: Vec<Line> = hotkeys.iter().map(|s| Line::from(*s)).collect();
    frame.render_widget(Paragraph::new(hotkey_lines).block(hotkeys_block), chunks[1]);
}

/// Render inventory bar (compact view)
pub fn render_inventory_bar(frame: &mut Frame, area: Rect, state: &GameState) {
    let items: Vec<Span> = state.inventory.iter().enumerate().take(9).map(|(i, id)| {
        let name = get_item_def(id).map(|d| d.name.as_str()).unwrap_or("?");
        Span::raw(format!("[{}]{} ", i + 1, name))
    }).collect();
    
    let line = if items.is_empty() {
        Line::from(Span::styled("Inventory: (empty)", Style::default().fg(Color::DarkGray)))
    } else {
        Line::from(items)
    };
    frame.render_widget(Paragraph::new(line), area);
}
