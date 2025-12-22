//! HUD rendering - side panel with stats, bottom panel with log

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, List, ListItem},
};
use crate::game::{GameState, MsgType, get_item_def, get_quest_def};
use crate::game::equipment::EquipSlot;
use super::theme::theme;

/// Unicode block characters for smooth health bars (8 levels)
const BAR_CHARS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

/// Get gradient color based on percentage using theme
fn health_color(pct: f32) -> Color {
    let t = theme();
    if pct > 0.5 { t.hp_high }
    else if pct > 0.25 { t.hp_mid }
    else { t.hp_low }
}

/// Render a gradient health bar with Unicode blocks
fn render_bar(current: i32, max: i32, width: usize) -> (String, Color) {
    let pct = if max > 0 { current.max(0) as f32 / max as f32 } else { 0.0 };
    let color = health_color(pct);
    let filled = pct * width as f32;
    let full_blocks = filled as usize;
    let partial = ((filled - full_blocks as f32) * 8.0) as usize;
    
    let mut bar = String::with_capacity(width);
    for _ in 0..full_blocks { bar.push(BAR_CHARS[8]); }
    if full_blocks < width { bar.push(BAR_CHARS[partial]); }
    while bar.chars().count() < width { bar.push(BAR_CHARS[0]); }
    (bar, color)
}

/// Render the side panel with player stats and equipment
pub fn render_side_panel(frame: &mut Frame, area: Rect, state: &GameState) {
    let block = Block::default().title(" Player ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),  // Stats (expanded for sanity)
            Constraint::Length(5),  // Status Effects
            Constraint::Length(9),  // Equipment
            Constraint::Min(3),     // Quests
        ])
        .split(inner);

    // Stats section with gradient bars
    let bar_width = 10;
    let (hp_bar, hp_color) = render_bar(state.player_hp, state.player_max_hp, bar_width);
    let (ap_bar, _) = render_bar(state.player_ap, state.player_max_ap, bar_width);
    let storm_color = if state.storm.turns_until <= 3 { Color::Red } 
        else if state.storm.turns_until <= 5 { Color::Yellow } 
        else { Color::Green };

    let stats = vec![
        Line::from(vec![
            Span::raw("HP "),
            Span::styled(hp_bar, Style::default().fg(hp_color)),
            Span::styled(format!(" {}/{}", state.player_hp, state.player_max_hp), Style::default().fg(hp_color)),
        ]),
        Line::from(vec![
            Span::raw("AP "),
            Span::styled(ap_bar, Style::default().fg(Color::Cyan)),
            Span::styled(format!(" {}/{}", state.player_ap, state.player_max_ap), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Sanity: "),
            Span::styled(format!("{}/{}", state.sanity.current_sanity, state.sanity.max_sanity), 
                Style::default().fg(if state.sanity.current_sanity > 60 { Color::Green } 
                    else if state.sanity.current_sanity > 30 { Color::Yellow } 
                    else { Color::Red })),
        ]),
        Line::from(vec![
            Span::raw("Lvl: "),
            Span::styled(format!("{}", state.player_level), Style::default().fg(Color::Yellow)),
            Span::raw(format!(" XP: {}", state.player_xp)),
        ]),
        Line::from(vec![
            Span::raw("Ref: "),
            Span::styled(format!("{}", state.refraction), Style::default().fg(Color::Magenta)),
            Span::raw(" Arm: "),
            Span::styled(format!("{}", state.player_armor), Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::raw("Scrip: "),
            Span::styled(format!("{}", state.salt_scrip), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Time: "),
            Span::styled(format!("{:02}:00", state.time_of_day), Style::default().fg(Color::White)),
            Span::raw(format!(" {:?}", state.weather)),
        ]),
        Line::from(vec![
            Span::raw("Storm: "),
            Span::styled(format!("{} turns", state.storm.turns_until), Style::default().fg(storm_color)),
        ]),
    ];
    frame.render_widget(Paragraph::new(stats), chunks[0]);

    // Status Effects section
    let mut status_lines = vec![Line::from(Span::styled("─Status─", Style::default().fg(Color::DarkGray)))];
    if state.status_effects.is_empty() {
        status_lines.push(Line::from(Span::styled("None", Style::default().fg(Color::DarkGray))));
    } else {
        for effect in &state.status_effects {
            status_lines.push(Line::from(vec![
                Span::styled(format!("{} ", effect.name), Style::default().fg(Color::Red)),
                Span::styled(format!("({})", effect.duration), Style::default().fg(Color::Yellow)),
            ]));
        }
    }
    frame.render_widget(Paragraph::new(status_lines), chunks[1]);

    // Equipment section
    let mut equip_lines = vec![Line::from(Span::styled("─Equipment─", Style::default().fg(Color::DarkGray)))];
    for slot in EquipSlot::all() {
        let item_name = state.equipment.get(*slot)
            .and_then(|id| get_item_def(id))
            .map(|d| d.name.as_str())
            .unwrap_or("-");
        equip_lines.push(Line::from(format!("{}: {}", slot.display_name(), item_name)));
    }
    frame.render_widget(Paragraph::new(equip_lines), chunks[2]);

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
    frame.render_widget(Paragraph::new(quest_lines), chunks[3]);
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

    // Message log with color-coded types
    let log_block = Block::default().title(" Log ").borders(Borders::ALL);
    let log_inner = log_block.inner(chunks[0]);
    frame.render_widget(log_block, chunks[0]);
    
    let t = theme();
    let msg_count = log_inner.height as usize;
    let total = state.messages.len();
    let messages: Vec<ListItem> = state.messages.iter()
        .rev()
        .take(msg_count)
        .rev()
        .enumerate()
        .map(|(i, m)| {
            let base_color = match m.msg_type {
                MsgType::Combat => t.msg_combat,
                MsgType::Loot => t.msg_loot,
                MsgType::Status => t.msg_status,
                MsgType::Dialogue => t.msg_dialogue,
                MsgType::System => t.msg_system,
                MsgType::Social => t.msg_system, // Use same color as system for now
            };
            // Fade older messages (dim if not recent)
            let age = total.saturating_sub(i + 1);
            let color = if age > 2 { t.msg_faded } else { base_color };
            ListItem::new(Span::styled(&m.text, Style::default().fg(color)))
        })
        .collect();
    frame.render_widget(List::new(messages), log_inner);

    // Hotkeys
    let hotkeys_block = Block::default().title(" Keys ").borders(Borders::ALL);
    let hotkeys = vec![
        "hjkl  Move",
        "i     Inventory",
        "c     Craft",
        "q     Quest log",
        "w     Wiki",
        "x     Look mode",
        "e     End turn",
        "o     Auto-explore",
        "Esc   Menu",
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

/// Render target enemy HUD (bottom left overlay)
pub fn render_target_hud(frame: &mut Frame, state: &GameState, target_idx: usize) {
    if target_idx >= state.enemies.len() {
        return;
    }
    let enemy = &state.enemies[target_idx];
    if enemy.hp <= 0 {
        return;
    }
    
    let def = enemy.def();
    let name = enemy.name();
    let max_hp = def.map(|d| d.max_hp).unwrap_or(10);
    let demeanor = format!("{:?}", enemy.demeanor()).to_lowercase();
    
    let bar_width = 12;
    let (hp_bar, hp_color) = render_bar(enemy.hp, max_hp, bar_width);
    
    let lines = vec![
        Line::from(Span::styled(name, Style::default().fg(Color::Red).bold())),
        Line::from(vec![
            Span::raw("HP "),
            Span::styled(hp_bar, Style::default().fg(hp_color)),
            Span::styled(format!(" {}/{}", enemy.hp, max_hp), Style::default().fg(hp_color)),
        ]),
        Line::from(Span::styled(format!("({})", demeanor), Style::default().fg(Color::DarkGray))),
    ];
    
    let area = frame.area();
    let width = 22u16;
    let height = 5u16;
    let x = area.width.saturating_sub(width + 1);
    let y = area.height.saturating_sub(height + 1);
    let hud_area = Rect::new(x, y, width, height);
    
    let block = Block::default()
        .title(" Target ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(ratatui::widgets::Clear, hud_area);
    frame.render_widget(Paragraph::new(lines).block(block), hud_area);
}
