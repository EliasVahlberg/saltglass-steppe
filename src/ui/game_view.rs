//! Game map rendering

use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use crate::{get_enemy_effects, get_light_def, GameState, Tile, VisualEffect};
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
    let inner = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), area.height.saturating_sub(2));
    let view_w = inner.width as i32;
    let view_h = inner.height as i32;
    let cam_x = (state.player_x - view_w / 2).max(0).min(state.map.width as i32 - view_w);
    let cam_y = (state.player_y - view_h / 2).max(0).min(state.map.height as i32 - view_h);
    
    for dn in &state.damage_numbers {
        let rise = (12 - dn.frames) / 6;
        let screen_x = (dn.x - cam_x) as i32;
        let screen_y = (dn.y - cam_y) as i32 - rise as i32;
        
        if screen_x >= 0 && screen_x < view_w && screen_y >= 0 && screen_y < view_h {
            let color = if dn.is_heal { Color::Green } else { t.msg_combat };
            let text = format!("{}", dn.value);
            let rect = Rect::new(inner.x + screen_x as u16, inner.y + screen_y as u16, text.len() as u16, 1);
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
    camera: (f32, f32),
) {
    let title = Line::from(format!(" Turn {} ", state.turn));
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate viewport using smooth camera position
    let view_w = inner.width as i32;
    let view_h = inner.height as i32;
    let half_w = view_w / 2;
    let half_h = view_h / 2;
    
    // Use smooth camera position, clamped to map bounds
    let cam_x = (camera.0 as i32 - half_w).max(0).min(state.map.width as i32 - view_w);
    let cam_y = (camera.1 as i32 - half_h).max(0).min(state.map.height as i32 - view_h);

    let mut lines: Vec<Line> = Vec::new();
    for vy in 0..view_h {
        let mut spans: Vec<Span> = Vec::new();
        let y = (cam_y + vy) as usize;
        for vx in 0..view_w {
            let x = (cam_x + vx) as usize;
            if x >= state.map.width || y >= state.map.height {
                spans.push(Span::raw(" "));
                continue;
            }
            let idx = state.map.idx(x as i32, y as i32);
            let is_look_cursor = look_cursor.map(|(lx, ly)| x as i32 == lx && y as i32 == ly).unwrap_or(false);
            
            // Check for projectile at this position
            if let Some(proj_char) = state.get_projectile_at(x as i32, y as i32) {
                let style = Style::default().fg(Color::Yellow).bold();
                let style = if is_look_cursor { style.bg(Color::White).fg(Color::Black) } else { style };
                spans.push(Span::styled(proj_char.to_string(), style));
                continue;
            }
            
            // Check for light beam at this position
            if let Some((beam_char, beam_type)) = state.get_beam_at(x as i32, y as i32) {
                let beam_color = match beam_type {
                    crate::game::state::BeamType::Laser => Color::Red,
                    crate::game::state::BeamType::Light => Color::Yellow,
                    crate::game::state::BeamType::Reflection => Color::Cyan,
                    crate::game::state::BeamType::Arrow => Color::Green,
                };
                let style = Style::default().fg(beam_color).bold();
                let style = if is_look_cursor { style.bg(Color::White).fg(Color::Black) } else { style };
                spans.push(Span::styled(beam_char.to_string(), style));
                continue;
            }
            
            let (ch, style) = render_tile(state, x, y, idx, player_effects, frame_count, state.debug_god_view);
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
    god_view: bool,
) -> (char, Style) {
    let t = theme();
    let visible = state.visible.contains(&idx) || god_view;
    let revealed = state.revealed.contains(&idx) || god_view;
    
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
            let status_color = match effect.id.as_str() {
                "burn" => Some(t.status_burning),
                "poison" => Some(t.status_poisoned),
                "bleed" => Some(t.status_bleeding),
                "stun" => Some(t.status_frozen),
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
                VisualEffect::Pulse { speed, color } => {
                    let phase = (frame_count / *speed as u64) % 4;
                    if phase < 2 {
                        let light = state.get_light_level(x as i32, y as i32);
                        style = style.fg(dim_color(*color, light));
                    }
                }
                VisualEffect::Wave { speed, color } => {
                    let wave_phase = ((frame_count / *speed as u64) + (x as u64 + y as u64)) % 6;
                    if wave_phase < 3 {
                        let light = state.get_light_level(x as i32, y as i32);
                        style = style.fg(dim_color(*color, light));
                    }
                }
                VisualEffect::Shimmer { speed, colors } => {
                    let color_idx = ((frame_count / *speed as u64) + (x as u64 ^ y as u64)) % colors.len() as u64;
                    let light = state.get_light_level(x as i32, y as i32);
                    style = style.fg(dim_color(colors[color_idx as usize], light));
                }
                VisualEffect::Rainbow { speed, colors } => {
                    let color_idx = (frame_count / *speed as u64) % colors.len() as u64;
                    let light = state.get_light_level(x as i32, y as i32);
                    style = style.fg(dim_color(colors[color_idx as usize], light));
                }
                VisualEffect::Fade { speed, color } => {
                    let fade_phase = (frame_count / *speed as u64) % 8;
                    if fade_phase < 4 {
                        let light = state.get_light_level(x as i32, y as i32);
                        style = style.fg(dim_color(*color, light));
                    }
                }
                VisualEffect::Drift { speed, color } => {
                    let drift_phase = ((frame_count / *speed as u64) + (x as u64 * 3 + y as u64 * 7)) % 10;
                    if drift_phase < 3 {
                        let light = state.get_light_level(x as i32, y as i32);
                        style = style.fg(dim_color(*color, light));
                    }
                }
                VisualEffect::HitFlash { .. } => {} // Handled above
            }
        }
        return ('@', style);
    }
    
    // Enemy
    if let Some(&ei) = state.enemy_positions.get(&(x as i32, y as i32)) {
        let e = &state.enemies[ei];
        if visible {
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
            let light = state.get_light_level(x as i32, y as i32);
            let mut style = Style::default().fg(dim_color(base_color, light));
            for effect in get_enemy_effects(&e.id) {
                match effect {
                    VisualEffect::Blink { speed, color } => {
                        if (frame_count / speed as u64) % 2 == 0 { 
                            style = style.fg(dim_color(color, light)); 
                        }
                    }
                    VisualEffect::Glow { color } => style = style.fg(dim_color(color, light)),
                    VisualEffect::Pulse { speed, color } => {
                        let phase = (frame_count / speed as u64) % 4;
                        if phase < 2 { 
                            style = style.fg(dim_color(color, light)); 
                        }
                    }
                    VisualEffect::Wave { speed, color } => {
                        let wave_phase = ((frame_count / speed as u64) + (x as u64 + y as u64)) % 6;
                        if wave_phase < 3 { 
                            style = style.fg(dim_color(color, light)); 
                        }
                    }
                    VisualEffect::Shimmer { speed, colors } => {
                        let color_idx = ((frame_count / speed as u64) + (x as u64 ^ y as u64)) % colors.len() as u64;
                        style = style.fg(dim_color(colors[color_idx as usize], light));
                    }
                    VisualEffect::Rainbow { speed, colors } => {
                        let color_idx = (frame_count / speed as u64) % colors.len() as u64;
                        style = style.fg(dim_color(colors[color_idx as usize], light));
                    }
                    VisualEffect::Fade { speed, color } => {
                        let fade_phase = (frame_count / speed as u64) % 8;
                        if fade_phase < 4 { 
                            style = style.fg(dim_color(color, light)); 
                        }
                    }
                    VisualEffect::Drift { speed, color } => {
                        let drift_phase = ((frame_count / speed as u64) + (x as u64 * 3 + y as u64 * 7)) % 10;
                        if drift_phase < 3 { 
                            style = style.fg(dim_color(color, light)); 
                        }
                    }
                    VisualEffect::HitFlash { .. } => {} // Handled above
                }
            }
            return (e.glyph(), style);
        } else if revealed {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // NPC
    if let Some(&ni) = state.npc_positions.get(&(x as i32, y as i32)) {
        let npc = &state.npcs[ni];
        if visible {
            let light = state.get_light_level(x as i32, y as i32);
            return (npc.glyph(), Style::default().fg(dim_color(Color::Green, light)).bold());
        } else if revealed {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // Item
    if let Some(items) = state.item_positions.get(&(x as i32, y as i32)) {
        let item = &state.items[items[0]];
        if visible {
            let light = state.get_light_level(x as i32, y as i32);
            return (item.glyph(), Style::default().fg(dim_color(Color::LightMagenta, light)));
        } else if revealed {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // Light source
    if let Some(ml) = state.map.lights.iter().find(|l| l.x == x as i32 && l.y == y as i32) {
        if visible {
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
        } else if revealed {
            return (state.map.tiles[idx].glyph(), Style::default().fg(Color::DarkGray));
        }
        return (' ', Style::default());
    }
    
    // Visible tile
    if visible {
        let tile = &state.map.tiles[idx];
        let light = state.get_light_level(x as i32, y as i32);
        
        // Check if this tile was changed by the last storm
        let is_storm_changed = state.storm_changed_tiles.contains(&idx);
        
        // Animated tiles based on animation_frame
        let (glyph, base_color) = match tile {
            Tile::Glass => {
                // Glass shimmers between cyan shades with shimmer overlay
                let phase = ((state.animation_frame / 4) + (x as u32 ^ y as u32)) % 3;
                let shimmer_phase = ((state.animation_frame / 2) + (x as u32 * 3 + y as u32 * 7)) % 6;
                
                let (glyph, color) = if shimmer_phase < 2 {
                    // Show shimmer overlay
                    ('≈', Color::LightCyan)
                } else {
                    // Normal glass appearance
                    let color = match phase {
                        0 => Color::Cyan,
                        1 => Color::LightCyan,
                        _ => Color::Rgb(100, 200, 220),
                    };
                    ('░', color)
                };
                (glyph, color)
            }
            Tile::Glare => {
                // Glare tiles pulse with bright light
                let pulse_phase = ((state.animation_frame / 3) + (x as u32 + y as u32)) % 4;
                let color = match pulse_phase {
                    0 => Color::White,
                    1 => Color::LightYellow,
                    2 => Color::Yellow,
                    _ => Color::Rgb(255, 255, 200),
                };
                ('░', color)
            }
            Tile::Floor { id: _ } => ('.', Color::DarkGray),
            Tile::Wall { .. } => ('#', Color::Gray),
            Tile::StairsDown => ('>', Color::Yellow),
            Tile::StairsUp => ('<', Color::Yellow),
            Tile::WorldExit => ('▣', Color::Green),
        };
        
        // Apply storm change highlighting
        let final_color = if is_storm_changed {
            Color::LightCyan // Highlight changed tiles
        } else {
            dim_color(base_color, light)
        };
        
        return (glyph, Style::default().fg(final_color));
    }
    
    // Revealed but not visible
    if revealed {
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
    let msg = " Press Esc to return to menu ";
    let msg_x = area.width.saturating_sub(msg.len() as u16) / 2;
    let msg_y = stats_y + stats.len() as u16 + 2;
    frame.render_widget(
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::White).bg(Color::Red))),
        Rect::new(msg_x, msg_y, msg.len() as u16, 1)
    );
}


/// Render the debug console overlay
pub fn render_debug_console(frame: &mut Frame, console: &super::input::DebugConsole) {
    let area = frame.area();
    
    // Make console take up about 1/3 of screen height and 2/3 of width
    let width = ((area.width as f32 * 0.66) as u16).min(area.width.saturating_sub(4));
    let height = ((area.height as f32 * 0.33) as u16).max(8).min(area.height.saturating_sub(4));
    
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let rect = Rect::new(x, y, width, height);
    
    // Clear just the console area with black background
    let clear_text = " ".repeat(width as usize);
    let clear_lines: Vec<Line> = (0..height).map(|_| Line::from(clear_text.clone())).collect();
    let clear_paragraph = Paragraph::new(clear_lines).style(Style::default().bg(Color::Black));
    frame.render_widget(clear_paragraph, rect);
    
    // Terminal-style block with title
    let block = Block::default()
        .title(" Debug Terminal ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Green));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    
    // Split into history area and input area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),      // History area
            Constraint::Length(3),   // Input + suggestions area
        ])
        .split(inner);
    
    // Render command history
    if !console.history.is_empty() {
        let history_height = chunks[0].height as usize;
        let start_idx = if console.history.len() > history_height {
            console.history.len() - history_height
        } else {
            0
        };
        
        let mut history_lines = Vec::new();
        for (i, cmd) in console.history[start_idx..].iter().enumerate() {
            let line_style = if Some(start_idx + i) == console.history_index {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Gray)
            };
            history_lines.push(Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Green)),
                Span::styled(cmd.clone(), line_style),
            ]));
        }
        
        let history_paragraph = Paragraph::new(history_lines)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(history_paragraph, chunks[0]);
    }
    
    // Split input area for prompt and suggestions
    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Input line
            Constraint::Min(1),      // Suggestions
        ])
        .split(chunks[1]);
    
    // Render current input with cursor
    let prompt = format!("> {}_", console.input);
    let input_paragraph = Paragraph::new(prompt)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(input_paragraph, input_chunks[0]);
    
    // Render suggestions if any
    if !console.suggestions.is_empty() && input_chunks[1].height > 0 {
        let mut suggestion_lines = Vec::new();
        let max_suggestions = input_chunks[1].height as usize;
        
        for (i, suggestion) in console.suggestions.iter().take(max_suggestions).enumerate() {
            let style = if i == console.suggestion_index {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            
            suggestion_lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(suggestion.clone(), style),
            ]));
        }
        
        let suggestions_paragraph = Paragraph::new(suggestion_lines);
        frame.render_widget(suggestions_paragraph, input_chunks[1]);
    }
}


/// Render dialog box overlay with word wrapping and typewriter effect
pub fn render_dialog_box(frame: &mut Frame, dialog: &super::input::DialogBox) {
    if !dialog.active { return; }
    
    let area = frame.area();
    let width = 60.min(area.width.saturating_sub(4));
    let height = 12.min(area.height.saturating_sub(4));
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let rect = Rect::new(x, y, width, height);
    
    // Clear the area first
    frame.render_widget(ratatui::widgets::Clear, rect);
    
    let title = format!(" {} ", dialog.speaker);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    
    // Word wrap the visible text
    let text = dialog.visible_text();
    let wrapped = textwrap::wrap(text, inner.width as usize);
    let lines: Vec<Line> = wrapped.iter().map(|s| Line::from(s.to_string())).collect();
    frame.render_widget(Paragraph::new(lines).style(Style::default().bg(Color::Black)), inner);
    
    // Page indicator and hint
    let page_info = format!(" [{}/{}] Enter=Next Esc=Close ", dialog.current_page + 1, dialog.pages.len());
    let info_len = page_info.len() as u16;
    let hint_x = rect.x + rect.width.saturating_sub(info_len + 1);
    let hint_y = rect.y + rect.height - 1;
    frame.render_widget(
        Paragraph::new(Span::styled(page_info, Style::default().fg(Color::DarkGray).bg(Color::Black))),
        Rect::new(hint_x, hint_y, info_len, 1)
    );
}
