//! Game map rendering

use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use crate::{get_enemy_effects, get_light_def, GameState, Tile, VisualEffect};
use crate::game::status::StatusType;
use super::theme::theme;

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

/// Render floating damage numbers as overlay
pub fn render_damage_numbers(frame: &mut Frame, area: Rect, state: &GameState) {
    let t = theme();
    for dn in &state.damage_numbers {
        // Calculate screen position (offset by 1 for border)
        let screen_x = dn.x as u16 + 1;
        // Rise effect: move up as frames decrease (12 -> 0 means rise by ~1 cell)
        let rise = (12 - dn.frames) / 6;
        let screen_y = (dn.y as u16).saturating_sub(rise as u16) + 1;
        
        if screen_x < area.width && screen_y < area.height {
            let color = if dn.is_heal { Color::Green } else { t.msg_combat };
            let text = format!("{}", dn.value);
            let rect = Rect::new(area.x + screen_x, area.y + screen_y, text.len() as u16, 1);
            frame.render_widget(
                Paragraph::new(text).style(Style::default().fg(color).bold()),
                rect
            );
        }
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
            
            // Check for projectile at this position
            if let Some(proj_char) = state.get_projectile_at(x as i32, y as i32) {
                let style = Style::default().fg(Color::Yellow).bold();
                let style = if is_look_cursor { style.bg(Color::White).fg(Color::Black) } else { style };
                spans.push(Span::styled(proj_char.to_string(), style));
                continue;
            }
            
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
    let t = theme();
    
    // Check for hit flash at this position
    let has_flash = state.has_hit_flash(x as i32, y as i32);
    
    // Player
    if x as i32 == state.player_x && y as i32 == state.player_y {
        let mut style = Style::default().fg(Color::Yellow).bold();
        
        // Hit flash takes priority
        if has_flash && (frame_count % 2 == 0) {
            return ('@', Style::default().fg(Color::White).bg(Color::Red).bold());
        }
        
        // Apply status effect colors (priority: burn > poison > bleed)
        for effect in &state.status_effects {
            let status_color = match effect.effect_type {
                StatusType::Burn => Some(t.status_burning),
                StatusType::Poison => Some(t.status_poisoned),
                StatusType::Bleed => Some(t.status_bleeding),
                StatusType::Stun => Some(t.status_frozen),
                _ => None,
            };
            if let Some(color) = status_color {
                // Blink effect for status
                if (frame_count / 4) % 2 == 0 {
                    style = style.fg(color);
                }
                break; // Only show highest priority status
            }
        }
        
        for effect in player_effects {
            match effect {
                VisualEffect::Blink { speed, color } => {
                    if (frame_count / *speed as u64) % 2 == 0 {
                        style = style.fg(*color);
                    }
                }
                VisualEffect::Glow { color } => style = style.fg(*color),
                VisualEffect::HitFlash { .. } => {} // Handled above
            }
        }
        return ('@', style);
    }
    
    // Enemy
    if let Some(&ei) = state.enemy_positions.get(&(x as i32, y as i32)) {
        let e = &state.enemies[ei];
        if state.visible.contains(&idx) {
            // Hit flash takes priority
            if has_flash && (frame_count % 2 == 0) {
                return (e.glyph(), Style::default().fg(Color::White).bg(Color::Red));
            }
            
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
                    VisualEffect::HitFlash { .. } => {} // Handled above
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
            Tile::StairsDown => Color::Yellow,
            Tile::StairsUp => Color::Yellow,
            Tile::WorldExit => Color::Green,
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
pub fn render_death_screen(frame: &mut Frame, state: &GameState) {
    let area = frame.area();
    
    // Fill background with dark red
    let bg_lines: Vec<Line> = (0..area.height).map(|_| {
        Line::from(Span::styled(" ".repeat(area.width as usize), Style::default().bg(Color::Rgb(40, 0, 0))))
    }).collect();
    frame.render_widget(Paragraph::new(bg_lines), area);
    
    // ASCII skull art
    let skull = [
        "     ___________     ",
        "    /           \\    ",
        "   /  .       .  \\   ",
        "  |       _       |  ",
        "  |    \\     /    |  ",
        "   \\    '---'    /   ",
        "    \\___________/    ",
        "        | | |        ",
        "       _| | |_       ",
    ];
    
    let skull_y = area.height / 4;
    let skull_x = area.width.saturating_sub(21) / 2;
    for (i, line) in skull.iter().enumerate() {
        frame.render_widget(
            Paragraph::new(Span::styled(*line, Style::default().fg(Color::Red).bold())),
            Rect::new(skull_x, skull_y + i as u16, 21, 1)
        );
    }
    
    // "YOU DIED" text
    let title = "Y O U   D I E D";
    let title_x = area.width.saturating_sub(title.len() as u16) / 2;
    let title_y = skull_y + skull.len() as u16 + 2;
    frame.render_widget(
        Paragraph::new(Span::styled(title, Style::default().fg(Color::Red).bold())),
        Rect::new(title_x, title_y, title.len() as u16, 1)
    );
    
    // Stats summary
    let stats = [
        format!("Level: {}", state.player_level),
        format!("Turns Survived: {}", state.turn),
        format!("Refraction: {}", state.refraction),
        format!("Adaptations: {}", state.adaptations.len()),
    ];
    let stats_y = title_y + 3;
    for (i, stat) in stats.iter().enumerate() {
        let stat_x = area.width.saturating_sub(stat.len() as u16) / 2;
        frame.render_widget(
            Paragraph::new(Span::styled(stat.as_str(), Style::default().fg(Color::Gray))),
            Rect::new(stat_x, stats_y + i as u16, stat.len() as u16, 1)
        );
    }
    
    // Quit prompt
    let msg = " Press Q to quit ";
    let msg_x = area.width.saturating_sub(msg.len() as u16) / 2;
    let msg_y = stats_y + stats.len() as u16 + 2;
    frame.render_widget(
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::White).bg(Color::Red))),
        Rect::new(msg_x, msg_y, msg.len() as u16, 1)
    );
}
