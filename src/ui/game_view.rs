//! Game map rendering

use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use crate::{get_enemy_effects, get_light_def, GameState, Tile, VisualEffect};

/// Dim a color based on light level (0-255)
pub fn dim_color(color: Color, light: u8) -> Color {
    if light >= 200 { return color; }
    let factor = light as f32 / 255.0;
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        Color::Gray | Color::DarkGray => if light < 100 { Color::Black } else { Color::DarkGray },
        Color::Cyan => if light < 100 { Color::DarkGray } else { color },
        _ => if light < 100 { Color::DarkGray } else { color },
    }
}

/// Render the game map
pub fn render_map(
    frame: &mut Frame,
    area: Rect,
    state: &GameState,
    player_effects: &[VisualEffect],
    frame_count: u64,
    look_cursor: Option<(i32, i32)>,
) {
    let title = Line::from(format!(" Turn {} ", state.turn));
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for y in 0..state.map.height {
        let mut spans: Vec<Span> = Vec::new();
        for x in 0..state.map.width {
            let idx = state.map.idx(x as i32, y as i32);
            let is_look_cursor = look_cursor.map(|(lx, ly)| x as i32 == lx && y as i32 == ly).unwrap_or(false);
            
            let (ch, style) = render_tile(state, x, y, idx, player_effects, frame_count);
            let style = if is_look_cursor { style.bg(Color::White).fg(Color::Black) } else { style };
            spans.push(Span::styled(ch.to_string(), style));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_tile(
    state: &GameState,
    x: usize,
    y: usize,
    idx: usize,
    player_effects: &[VisualEffect],
    frame_count: u64,
) -> (char, Style) {
    // Player
    if x as i32 == state.player_x && y as i32 == state.player_y {
        let mut style = Style::default().fg(Color::Yellow).bold();
        for effect in player_effects {
            match effect {
                VisualEffect::Blink { speed, color } => {
                    if (frame_count / *speed as u64) % 2 == 0 {
                        style = style.fg(*color);
                    }
                }
                VisualEffect::Glow { color } => style = style.fg(*color),
            }
        }
        return ('@', style);
    }
    
    // Enemy
    if let Some(&ei) = state.enemy_positions.get(&(x as i32, y as i32)) {
        let e = &state.enemies[ei];
        if state.visible.contains(&idx) {
            let base_color = match e.id.as_str() {
                "mirage_hound" => Color::LightYellow,
                "glass_beetle" => Color::Cyan,
                "salt_mummy" => Color::White,
                _ => Color::Red,
            };
            let mut style = Style::default().fg(base_color);
            for effect in get_enemy_effects(&e.id) {
                match effect {
                    VisualEffect::Blink { speed, color } => {
                        if (frame_count / speed as u64) % 2 == 0 { style = style.fg(color); }
                    }
                    VisualEffect::Glow { color } => style = style.fg(color),
                }
            }
            return (e.glyph(), style);
        } else if state.revealed.contains(&idx) {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // NPC
    if let Some(&ni) = state.npc_positions.get(&(x as i32, y as i32)) {
        let npc = &state.npcs[ni];
        if state.visible.contains(&idx) {
            return (npc.glyph(), Style::default().fg(Color::Green).bold());
        } else if state.revealed.contains(&idx) {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // Item
    if let Some(items) = state.item_positions.get(&(x as i32, y as i32)) {
        let item = &state.items[items[0]];
        if state.visible.contains(&idx) {
            return (item.glyph(), Style::default().fg(Color::LightMagenta));
        } else if state.revealed.contains(&idx) {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // Light source
    if let Some(ml) = state.map.lights.iter().find(|l| l.x == x as i32 && l.y == y as i32) {
        if state.visible.contains(&idx) {
            let def = get_light_def(&ml.id);
            let glyph = def.map(|d| d.glyph.chars().next().unwrap_or('*')).unwrap_or('*');
            let color = match def.map(|d| d.color.as_str()) {
                Some("orange") => Color::Rgb(255, 140, 0),
                Some("yellow") => Color::Yellow,
                Some("cyan") => Color::Cyan,
                Some("red") => Color::Rgb(255, 80, 40),
                _ => Color::Yellow,
            };
            return (glyph, Style::default().fg(color));
        } else if state.revealed.contains(&idx) {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // Visible tile
    if state.visible.contains(&idx) {
        let tile = &state.map.tiles[idx];
        let light = state.get_light_level(x as i32, y as i32);
        let base_color = match tile {
            Tile::Floor => Color::DarkGray,
            Tile::Wall { .. } => Color::Gray,
            Tile::Glass => Color::Cyan,
        };
        return (tile.glyph(), Style::default().fg(dim_color(base_color, light)));
    }
    
    // Revealed but not visible
    if state.revealed.contains(&idx) {
        return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
    }
    
    (' ', Style::default())
}

/// Render the death screen
pub fn render_death_screen(frame: &mut Frame) {
    let area = frame.area();
    let death_msg = "YOU DIED";
    let lines: Vec<Line> = (0..area.height).map(|_| {
        let mut row = String::new();
        while row.len() < area.width as usize {
            row.push_str(death_msg);
            row.push(' ');
        }
        Line::from(Span::styled(row, Style::default().fg(Color::Red).bold()))
    }).collect();
    frame.render_widget(Paragraph::new(lines), area);
    
    let center_y = area.height / 2;
    let msg = " Press Q to quit ";
    let center_x = area.width.saturating_sub(msg.len() as u16) / 2;
    frame.render_widget(
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::White).bg(Color::Red))),
        Rect::new(center_x, center_y, msg.len() as u16, 1)
    );
}
