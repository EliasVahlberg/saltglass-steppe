//! Skills Menu — pannable 2D skill tree graph

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::collections::HashMap;

use crate::game::{
    skills::{
        SKILL_CATEGORIES, calculate_skill_cost, category_name, get_category_roots,
        get_skill_children, get_skill_def,
    },
    state::GameState,
};

// --- Layout constants ---
pub const NODE_W: i32 = 22; // chars per node (including box)
pub const NODE_H: i32 = 3; // rows per node
const COL_GAP: i32 = 6; // extra chars between columns
const ROW_GAP: i32 = 1; // extra rows between rows

// --- State ---

#[derive(Clone, Debug, Default)]
pub struct SkillsMenu {
    pub active: bool,
    pub category_idx: usize,
    pub cursor_id: String,
    pub pan_x: i32,
    pub pan_y: i32,
    /// Cached layout for current category: skill_id → (canvas_x, canvas_y)
    layout_cache: HashMap<String, (i32, i32)>,
    layout_category: Option<usize>,
}

impl SkillsMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.active = true;
        self.pan_x = 0;
        self.pan_y = 0;
        self.ensure_layout();
        if self.cursor_id.is_empty() {
            self.cursor_id = self.first_node_id();
        }
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn next_category(&mut self) {
        self.category_idx = (self.category_idx + 1) % SKILL_CATEGORIES.len();
        self.layout_category = None;
        self.pan_x = 0;
        self.pan_y = 0;
        self.ensure_layout();
        self.cursor_id = self.first_node_id();
    }

    pub fn prev_category(&mut self) {
        self.category_idx = if self.category_idx == 0 {
            SKILL_CATEGORIES.len() - 1
        } else {
            self.category_idx - 1
        };
        self.layout_category = None;
        self.pan_x = 0;
        self.pan_y = 0;
        self.ensure_layout();
        self.cursor_id = self.first_node_id();
    }

    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.pan_x = (self.pan_x + dx).max(0);
        self.pan_y = (self.pan_y + dy).max(0);
    }

    /// Move cursor to parent node
    pub fn cursor_left(&mut self) {
        if let Some(def) = get_skill_def(&self.cursor_id)
            && let Some(parent_id) = &def.tree_parent
        {
            self.cursor_id = parent_id.clone();
            self.scroll_to_cursor();
        }
    }

    /// Move cursor to first child
    pub fn cursor_right(&mut self) {
        let children = get_skill_children(&self.cursor_id);
        if let Some(first) = children.first() {
            self.cursor_id = first.id.clone();
            self.scroll_to_cursor();
        }
    }

    /// Move cursor to previous sibling (same parent, lower canvas Y)
    pub fn cursor_up(&mut self) {
        if let Some(sibling) = self.adjacent_sibling(-1) {
            self.cursor_id = sibling;
            self.scroll_to_cursor();
        }
    }

    /// Move cursor to next sibling (same parent, higher canvas Y)
    pub fn cursor_down(&mut self) {
        if let Some(sibling) = self.adjacent_sibling(1) {
            self.cursor_id = sibling;
            self.scroll_to_cursor();
        }
    }

    fn adjacent_sibling(&self, direction: i32) -> Option<String> {
        let parent_id = get_skill_def(&self.cursor_id)
            .and_then(|d| d.tree_parent.as_deref())
            .map(|s| s.to_string());

        // Collect siblings (same parent)
        let siblings: Vec<String> = if let Some(ref pid) = parent_id {
            get_skill_children(pid)
                .iter()
                .map(|d| d.id.clone())
                .collect()
        } else {
            // Root nodes — siblings are other roots in this category
            let cat = &SKILL_CATEGORIES[self.category_idx];
            get_category_roots(cat)
                .iter()
                .map(|d| d.id.clone())
                .collect()
        };

        // Sort by canvas Y
        let mut sorted: Vec<(String, i32)> = siblings
            .iter()
            .filter_map(|id| self.layout_cache.get(id).map(|&(_, y)| (id.clone(), y)))
            .collect();
        sorted.sort_by_key(|(_, y)| *y);

        let pos = sorted.iter().position(|(id, _)| id == &self.cursor_id)?;
        let next_pos = pos as i32 + direction;
        if next_pos < 0 || next_pos >= sorted.len() as i32 {
            return None;
        }
        Some(sorted[next_pos as usize].0.clone())
    }

    fn scroll_to_cursor(&mut self) {
        // Will be called with viewport dimensions during render; for now just ensure layout
        self.ensure_layout();
    }

    fn first_node_id(&self) -> String {
        let cat = &SKILL_CATEGORIES[self.category_idx];
        get_category_roots(cat)
            .first()
            .map(|d| d.id.clone())
            .unwrap_or_default()
    }

    fn ensure_layout(&mut self) {
        if self.layout_category == Some(self.category_idx) {
            return;
        }
        self.layout_cache.clear();
        let cat = &SKILL_CATEGORIES[self.category_idx];
        let roots = get_category_roots(cat);
        let mut row_counter: HashMap<i32, i32> = HashMap::new(); // col → next row slot
        for root in roots {
            assign_positions(
                root.id.as_str(),
                0,
                &mut row_counter,
                &mut self.layout_cache,
            );
        }
        self.layout_category = Some(self.category_idx);
    }

    /// Adjust pan so cursor node is visible within the given viewport size
    pub fn ensure_cursor_visible(&mut self, viewport_w: i32, viewport_h: i32) {
        if let Some(&(cx, cy)) = self.layout_cache.get(&self.cursor_id) {
            let margin_x = NODE_W;
            let margin_y = NODE_H + ROW_GAP;
            if cx < self.pan_x + margin_x {
                self.pan_x = (cx - margin_x).max(0);
            } else if cx + NODE_W > self.pan_x + viewport_w - margin_x {
                self.pan_x = cx + NODE_W - viewport_w + margin_x;
            }
            if cy < self.pan_y + margin_y {
                self.pan_y = (cy - margin_y).max(0);
            } else if cy + NODE_H > self.pan_y + viewport_h - margin_y {
                self.pan_y = cy + NODE_H - viewport_h + margin_y;
            }
        }
    }

    pub fn upgrade_selected(&self, state: &mut GameState) -> Result<(), String> {
        if self.cursor_id.is_empty() {
            return Err("No skill selected".to_string());
        }
        state.player.skills.upgrade_skill(&self.cursor_id)
    }
}

/// Recursively assign canvas positions depth-first
fn assign_positions(
    skill_id: &str,
    depth: i32,
    row_counter: &mut HashMap<i32, i32>,
    layout: &mut HashMap<String, (i32, i32)>,
) {
    let row = *row_counter.get(&depth).unwrap_or(&0);
    *row_counter.entry(depth).or_insert(0) += 1;

    let x = depth * (NODE_W + COL_GAP);
    let y = row * (NODE_H + ROW_GAP);
    layout.insert(skill_id.to_string(), (x, y));

    for child in get_skill_children(skill_id) {
        assign_positions(&child.id, depth + 1, row_counter, layout);
    }
}

// --- Rendering ---

pub fn render_skills_menu(f: &mut Frame, game_state: &GameState, menu: &mut SkillsMenu) {
    let size = f.area();
    let popup = centered_rect(90, 88, size);
    f.render_widget(Clear, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(popup);

    // Header
    let cat = &SKILL_CATEGORIES[menu.category_idx];
    let sp = game_state.player.skills.skill_points;
    let header_text = format!(
        " {} ─── {} SP available  [{}/{}]",
        category_name(cat),
        sp,
        menu.category_idx + 1,
        SKILL_CATEGORIES.len()
    );
    f.render_widget(
        Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title("Skill Tree"))
            .style(Style::default().fg(Color::Yellow)),
        chunks[0],
    );

    // Content: tree canvas (left 68%) + detail panel (right 32%)
    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(chunks[1]);

    let viewport_w = content[0].width as i32 - 2; // subtract borders
    let viewport_h = content[0].height as i32 - 2;

    menu.ensure_layout();
    menu.ensure_cursor_visible(viewport_w, viewport_h);

    render_tree_canvas(f, game_state, menu, content[0]);
    render_detail_panel(f, game_state, menu, content[1]);

    // Footer
    f.render_widget(
        Paragraph::new(
            " ←→↑↓: Move cursor  HJKL: Pan  Tab/Shift+Tab: Category  Enter: Upgrade  Esc: Close",
        )
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray)),
        chunks[2],
    );
}

fn render_tree_canvas(f: &mut Frame, game_state: &GameState, menu: &SkillsMenu, area: Rect) {
    let inner_x = area.x + 1;
    let inner_y = area.y + 1;
    let vw = area.width.saturating_sub(2) as i32;
    let vh = area.height.saturating_sub(2) as i32;

    // Build a styled canvas buffer: (char, Style)
    let buf_w = vw as usize;
    let buf_h = vh as usize;
    let blank = (' ', Style::default());
    let mut canvas: Vec<Vec<(char, Style)>> = vec![vec![blank; buf_w]; buf_h];

    let cat = &SKILL_CATEGORIES[menu.category_idx];

    // Draw connections first (behind nodes)
    for (skill_id, &(cx, cy)) in &menu.layout_cache {
        let children = get_skill_children(skill_id);
        if children.is_empty() {
            continue;
        }

        let node_right_x = cx + NODE_W; // right edge of parent node
        let parent_mid_y = cy + NODE_H / 2;

        for (i, child) in children.iter().enumerate() {
            if let Some(&(child_x, child_y)) = menu.layout_cache.get(&child.id) {
                let child_mid_y = child_y + NODE_H / 2;

                // Horizontal line from parent right edge to child left edge
                let line_start_x = node_right_x;
                let line_end_x = child_x;
                let connector_x = line_start_x + COL_GAP / 2;

                // Draw horizontal segment from parent to connector column
                for x in line_start_x..connector_x {
                    draw_char(
                        &mut canvas,
                        x - menu.pan_x,
                        parent_mid_y - menu.pan_y,
                        '─',
                        Style::default().fg(Color::DarkGray),
                        vw,
                        vh,
                    );
                }
                // Vertical segment if needed
                if parent_mid_y != child_mid_y {
                    let y_min = parent_mid_y.min(child_mid_y);
                    let y_max = parent_mid_y.max(child_mid_y);
                    for y in y_min..=y_max {
                        draw_char(
                            &mut canvas,
                            connector_x - menu.pan_x,
                            y - menu.pan_y,
                            '│',
                            Style::default().fg(Color::DarkGray),
                            vw,
                            vh,
                        );
                    }
                    // Corner chars
                    let corner = if i == 0 { '├' } else { '└' };
                    draw_char(
                        &mut canvas,
                        connector_x - menu.pan_x,
                        child_mid_y - menu.pan_y,
                        corner,
                        Style::default().fg(Color::DarkGray),
                        vw,
                        vh,
                    );
                }
                // Horizontal segment from connector to child
                for x in connector_x..line_end_x {
                    draw_char(
                        &mut canvas,
                        x - menu.pan_x,
                        child_mid_y - menu.pan_y,
                        '─',
                        Style::default().fg(Color::DarkGray),
                        vw,
                        vh,
                    );
                }
            }
        }
    }

    // Draw nodes
    for skill_id in get_category_roots(cat)
        .iter()
        .flat_map(|r| collect_subtree(r.id.as_str()))
    {
        if let Some(def) = get_skill_def(&skill_id)
            && let Some(&(cx, cy)) = menu.layout_cache.get(&def.id)
        {
            let level = game_state.player.skills.get_skill_level(&def.id);
            let can_up = game_state.player.skills.can_upgrade_skill(&def.id).is_ok();
            let is_cursor = def.id == menu.cursor_id;

            let node_style = if is_cursor {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if def.blocked {
                Style::default().fg(Color::Red)
            } else if level >= def.max_level {
                Style::default().fg(Color::Yellow)
            } else if can_up {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let suffix = if def.blocked {
                "✗"
            } else if def.active {
                "★"
            } else {
                ""
            };
            let label = format!("{}{}", def.name, suffix);
            let label_truncated = if label.len() > (NODE_W - 4) as usize {
                format!("{}…", &label[..(NODE_W - 5) as usize])
            } else {
                label
            };
            let level_str = format!("{}/{}", level, def.max_level);

            // Top border
            draw_str(
                &mut canvas,
                cx - menu.pan_x,
                cy - menu.pan_y,
                &format!("┌{:─<width$}┐", "", width = (NODE_W - 2) as usize),
                node_style,
                vw,
                vh,
            );
            // Label row
            draw_str(
                &mut canvas,
                cx - menu.pan_x,
                cy + 1 - menu.pan_y,
                &format!(
                    "│{:<width$}│",
                    label_truncated,
                    width = (NODE_W - 2) as usize
                ),
                node_style,
                vw,
                vh,
            );
            // Level row
            draw_str(
                &mut canvas,
                cx - menu.pan_x,
                cy + 2 - menu.pan_y,
                &format!("└{:─<width$}┘", level_str, width = (NODE_W - 2) as usize),
                node_style,
                vw,
                vh,
            );
        }
    }

    // Blit canvas to frame buffer
    let frame_buf = f.buffer_mut();
    for (row, canvas_row) in canvas.iter().enumerate() {
        for (col, &(ch, style)) in canvas_row.iter().enumerate() {
            let sx = inner_x + col as u16;
            let sy = inner_y + row as u16;
            if sx < area.x + area.width && sy < area.y + area.height {
                let cell = frame_buf.cell_mut((sx, sy)).unwrap();
                cell.set_char(ch);
                cell.set_style(style);
            }
        }
    }

    // Draw border on top
    f.render_widget(Block::default().borders(Borders::ALL).title("Tree"), area);
}

fn draw_char(
    canvas: &mut [Vec<(char, Style)>],
    x: i32,
    y: i32,
    ch: char,
    style: Style,
    w: i32,
    h: i32,
) {
    if x >= 0 && y >= 0 && x < w && y < h {
        canvas[y as usize][x as usize] = (ch, style);
    }
}

fn draw_str(
    canvas: &mut [Vec<(char, Style)>],
    x: i32,
    y: i32,
    s: &str,
    style: Style,
    w: i32,
    h: i32,
) {
    for (i, ch) in s.chars().enumerate() {
        draw_char(canvas, x + i as i32, y, ch, style, w, h);
    }
}

fn collect_subtree(root_id: &str) -> Vec<String> {
    let mut result = Vec::new();
    result.push(root_id.to_string());
    for child in get_skill_children(root_id) {
        result.extend(collect_subtree(&child.id));
    }
    result
}

fn render_detail_panel(f: &mut Frame, game_state: &GameState, menu: &SkillsMenu, area: Rect) {
    let content = if let Some(def) = get_skill_def(&menu.cursor_id) {
        let level = game_state.player.skills.get_skill_level(&def.id);
        let sp = game_state.player.skills.skill_points;
        let cost = calculate_skill_cost(&def.id, level);
        let upgrade = game_state.player.skills.can_upgrade_skill(&def.id);

        let mut lines = vec![
            Line::from(Span::styled(
                def.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(def.description.clone()),
            Line::from(""),
            Line::from(format!("Level: {}/{}", level, def.max_level)),
            Line::from(format!("Cost:  {} SP  (have {})", cost, sp)),
        ];

        if def.blocked {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "⚠ Blocked — system not yet implemented",
                Style::default().fg(Color::Red),
            )));
        } else {
            match upgrade {
                Ok(()) => {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "[Enter] Upgrade",
                        Style::default().fg(Color::Green),
                    )));
                }
                Err(ref e) => {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!("✗ {}", e),
                        Style::default().fg(Color::Red),
                    )));
                }
            }
        }

        // Prerequisites
        if !def.prerequisites.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Prerequisites:",
                Style::default().fg(Color::DarkGray),
            )));
            for prereq in &def.prerequisites {
                let have = game_state.player.skills.get_skill_level(&prereq.skill_id);
                let met = have >= prereq.required_level;
                let icon = if met { "✓" } else { "✗" };
                let color = if met { Color::Green } else { Color::Red };
                let name = get_skill_def(&prereq.skill_id)
                    .map(|d| d.name.as_str())
                    .unwrap_or(&prereq.skill_id);
                lines.push(Line::from(Span::styled(
                    format!("  {} {} Lv.{}", icon, name, prereq.required_level),
                    Style::default().fg(color),
                )));
            }
        }

        // Passive effects summary
        if !def.passive_effects.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Passives:",
                Style::default().fg(Color::DarkGray),
            )));
            for eff in &def.passive_effects {
                lines.push(Line::from(format!(
                    "  +{:.0}% {} per level",
                    eff.value_per_level * 100.0,
                    eff.effect_type.replace('_', " ")
                )));
            }
        }

        // Abilities unlocked by this skill
        let unlocked_abilities: Vec<_> = crate::game::skills::all_ability_ids()
            .into_iter()
            .filter_map(|id| crate::game::skills::get_ability_def(id))
            .filter(|a| a.required_skill == def.id)
            .collect();
        if !unlocked_abilities.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Unlocks:",
                Style::default().fg(Color::DarkGray),
            )));
            for ab in unlocked_abilities {
                let have = game_state.player.skills.unlocked_abilities.contains(&ab.id);
                let color = if have { Color::Yellow } else { Color::White };
                lines.push(Line::from(Span::styled(
                    format!("  ★ {}", ab.name),
                    Style::default().fg(color),
                )));
            }
        }

        lines
    } else {
        vec![Line::from("No skill selected")]
    };

    f.render_widget(
        Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}
