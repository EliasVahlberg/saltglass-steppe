//! Entity rendering system

use ratatui::prelude::*;
use crate::GameState;
use super::config::{RenderConfig, parse_color};

/// Handles rendering of all entities (player, enemies, NPCs, items)
pub struct EntityRenderer {
    config: RenderConfig,
}

impl EntityRenderer {
    pub fn new(config: &RenderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: &RenderConfig) {
        self.config = config.clone();
    }

    /// Render all entities and return spans for each screen position
    pub fn render_entities(
        &self,
        state: &GameState,
        light_map: &[u8],
        frame_count: u64,
        cam_x: i32,
        cam_y: i32,
        view_width: i32,
        view_height: i32,
    ) -> Vec<Vec<Option<Span<'_>>>> {
        let mut entity_spans = vec![vec![None; view_width as usize]; view_height as usize];

        // Render player
        if let Some((screen_x, screen_y)) = self.world_to_screen(state.player_x, state.player_y, cam_x, cam_y, view_width, view_height) {
            if let Some(span) = self.render_player(state, light_map, frame_count) {
                entity_spans[screen_y as usize][screen_x as usize] = Some(span);
            }
        }

        // Render enemies
        for (pos, &enemy_idx) in &state.enemy_positions {
            if let Some((screen_x, screen_y)) = self.world_to_screen(pos.0, pos.1, cam_x, cam_y, view_width, view_height) {
                if let Some(span) = self.render_enemy(state, enemy_idx, pos.0, pos.1, light_map, frame_count) {
                    entity_spans[screen_y as usize][screen_x as usize] = Some(span);
                }
            }
        }

        // Render NPCs
        for (pos, &npc_idx) in &state.npc_positions {
            if let Some((screen_x, screen_y)) = self.world_to_screen(pos.0, pos.1, cam_x, cam_y, view_width, view_height) {
                if let Some(span) = self.render_npc(state, npc_idx, pos.0, pos.1, light_map, frame_count) {
                    entity_spans[screen_y as usize][screen_x as usize] = Some(span);
                }
            }
        }

        // Render items
        for (pos, item_indices) in &state.item_positions {
            if let Some((screen_x, screen_y)) = self.world_to_screen(pos.0, pos.1, cam_x, cam_y, view_width, view_height) {
                if let Some(span) = self.render_item(state, item_indices[0], pos.0, pos.1, light_map, frame_count) {
                    entity_spans[screen_y as usize][screen_x as usize] = Some(span);
                }
            }
        }

        // Render light sources
        for map_light in &state.map.lights {
            if let Some((screen_x, screen_y)) = self.world_to_screen(map_light.x, map_light.y, cam_x, cam_y, view_width, view_height) {
                if let Some(span) = self.render_light_source(&map_light.id, map_light.x, map_light.y, light_map) {
                    entity_spans[screen_y as usize][screen_x as usize] = Some(span);
                }
            }
        }

        // Render projectiles
        for projectile_trail in &state.projectile_trails {
            if projectile_trail.current_idx < projectile_trail.path.len() {
                let (px, py) = projectile_trail.path[projectile_trail.current_idx];
                if let Some((screen_x, screen_y)) = self.world_to_screen(px, py, cam_x, cam_y, view_width, view_height) {
                    let span = Span::styled(
                        projectile_trail.char.to_string(),
                        Style::default().fg(Color::Yellow).bold()
                    );
                    entity_spans[screen_y as usize][screen_x as usize] = Some(span);
                }
            }
        }

        entity_spans
    }

    /// Render the player character
    fn render_player(&self, state: &GameState, light_map: &[u8], frame_count: u64) -> Option<Span<'_>> {
        let visible = state.visible.contains(&state.map.idx(state.player_x, state.player_y)) || state.debug_god_view;
        if !visible {
            return None;
        }

        let light_level = self.get_light_level(state.player_x, state.player_y, light_map, state.map.width, state.map.height);
        let base_color = parse_color(&self.config.colors.entities.player.base);
        let mut style = Style::default().fg(base_color).bold();

        // Apply hit flash
        if state.has_hit_flash(state.player_x, state.player_y) && (frame_count % 2 == 0) {
            return Some(Span::styled(
                "@",
                Style::default()
                    .fg(parse_color(&self.config.colors.ui.hit_flash.fg))
                    .bg(parse_color(&self.config.colors.ui.hit_flash.bg))
                    .bold()
            ));
        }

        // Apply status effect colors (priority order from config)
        for status_name in &self.config.rendering.status_effect_priority {
            if state.status_effects.iter().any(|e| e.id == *status_name) {
                if let Some(color_name) = self.config.colors.entities.player.status_effects.get(status_name) {
                    let status_color = parse_color(color_name);
                    // Blink effect for status
                    if (frame_count / 4) % 2 == 0 {
                        style = style.fg(status_color);
                    }
                    break; // Only show highest priority status
                }
            }
        }

        // Apply lighting
        style = style.fg(self.dim_color(style.fg.unwrap_or(base_color), light_level));

        Some(Span::styled("@", style))
    }

    /// Render an enemy
    fn render_enemy(&self, state: &GameState, enemy_idx: usize, x: i32, y: i32, light_map: &[u8], frame_count: u64) -> Option<Span<'_>> {
        let visible = state.visible.contains(&state.map.idx(x, y)) || state.debug_god_view;
        if !visible {
            return None;
        }

        let enemy = &state.enemies[enemy_idx];
        let light_level = self.get_light_level(x, y, light_map, state.map.width, state.map.height);

        // Apply hit flash
        if state.has_hit_flash(x, y) && (frame_count % 2 == 0) {
            return Some(Span::styled(
                enemy.glyph().to_string(),
                Style::default()
                    .fg(parse_color(&self.config.colors.ui.hit_flash.fg))
                    .bg(parse_color(&self.config.colors.ui.hit_flash.bg))
            ));
        }

        // Get enemy color from configuration
        let base_color = self.config.colors.entities.enemies
            .get(&enemy.id)
            .map(|c| parse_color(c))
            .unwrap_or_else(|| parse_color(&self.config.colors.entities.enemies.get("default").unwrap_or(&"Red".to_string())));

        let style = Style::default().fg(self.dim_color(base_color, light_level));

        Some(Span::styled(enemy.glyph().to_string(), style))
    }

    /// Render an NPC
    fn render_npc(&self, state: &GameState, npc_idx: usize, x: i32, y: i32, light_map: &[u8], _frame_count: u64) -> Option<Span<'_>> {
        let visible = state.visible.contains(&state.map.idx(x, y)) || state.debug_god_view;
        if !visible {
            return None;
        }

        let npc = &state.npcs[npc_idx];
        let light_level = self.get_light_level(x, y, light_map, state.map.width, state.map.height);
        let base_color = parse_color(&self.config.colors.entities.npcs.base);
        let style = Style::default().fg(self.dim_color(base_color, light_level)).bold();

        Some(Span::styled(npc.glyph().to_string(), style))
    }

    /// Render an item
    fn render_item(&self, state: &GameState, item_idx: usize, x: i32, y: i32, light_map: &[u8], _frame_count: u64) -> Option<Span<'_>> {
        let visible = state.visible.contains(&state.map.idx(x, y)) || state.debug_god_view;
        if !visible {
            return None;
        }

        let item = &state.items[item_idx];
        let light_level = self.get_light_level(x, y, light_map, state.map.width, state.map.height);
        let base_color = parse_color(&self.config.colors.entities.items.base);
        let style = Style::default().fg(self.dim_color(base_color, light_level));

        Some(Span::styled(item.glyph().to_string(), style))
    }

    /// Render a light source
    fn render_light_source(&self, light_id: &str, _x: i32, _y: i32, _light_map: &[u8]) -> Option<Span<'_>> {
        if let Some(light_def) = crate::get_light_def(light_id) {
            let glyph = light_def.glyph.chars().next().unwrap_or('*');
            
            // Get color from configuration or use default
            let color = self.config.colors.lighting.sources
                .get(light_id)
                .or_else(|| self.config.colors.lighting.sources.get("default"))
                .map(|c| parse_color(c))
                .unwrap_or(Color::Yellow);

            let style = Style::default().fg(color);
            Some(Span::styled(glyph.to_string(), style))
        } else {
            None
        }
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(&self, x: i32, y: i32, cam_x: i32, cam_y: i32, view_width: i32, view_height: i32) -> Option<(i32, i32)> {
        let screen_x = x - cam_x;
        let screen_y = y - cam_y;
        
        if screen_x >= 0 && screen_x < view_width && screen_y >= 0 && screen_y < view_height {
            Some((screen_x, screen_y))
        } else {
            None
        }
    }

    /// Get light level at position
    fn get_light_level(&self, x: i32, y: i32, light_map: &[u8], map_width: usize, map_height: usize) -> u8 {
        if x < 0 || y < 0 || x >= map_width as i32 || y >= map_height as i32 {
            return 0;
        }
        let idx = y as usize * map_width + x as usize;
        light_map.get(idx).copied().unwrap_or(self.config.lighting.ambient_level)
    }

    /// Dim a color based on light level
    fn dim_color(&self, color: Color, light_level: u8) -> Color {
        if light_level >= 200 { 
            return color; 
        }
        
        let factor = light_level as f32 / 255.0;
        
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(
                (r as f32 * factor) as u8,
                (g as f32 * factor) as u8,
                (b as f32 * factor) as u8,
            ),
            Color::Gray | Color::DarkGray => {
                if light_level < 100 { 
                    Color::Black 
                } else { 
                    Color::DarkGray 
                }
            },
            Color::Cyan => {
                if light_level < 100 { 
                    Color::DarkGray 
                } else { 
                    color 
                }
            },
            _ => {
                if light_level < 100 { 
                    Color::DarkGray 
                } else { 
                    color 
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen() {
        let config = RenderConfig {
            // ... minimal config for testing
            colors: super::super::config::ColorConfig {
                entities: super::super::config::EntityColors {
                    player: super::super::config::PlayerColors {
                        base: "Yellow".to_string(),
                        status_effects: std::collections::HashMap::new(),
                    },
                    enemies: std::collections::HashMap::new(),
                    npcs: super::super::config::NpcColors { base: "Green".to_string() },
                    items: super::super::config::ItemColors { base: "LightMagenta".to_string() },
                },
                tiles: super::super::config::TileColors {
                    floor: "DarkGray".to_string(),
                    wall: "Gray".to_string(),
                    glass: super::super::config::GlassColors {
                        base: "Cyan".to_string(),
                        shimmer: vec!["Cyan".to_string()],
                    },
                    stairs_down: "Yellow".to_string(),
                    stairs_up: "Yellow".to_string(),
                    world_exit: "Green".to_string(),
                },
                lighting: super::super::config::LightingColors {
                    sources: std::collections::HashMap::new(),
                    ambient: 20,
                    visibility_threshold: 50,
                },
                ui: super::super::config::UiColors {
                    revealed_tile: "DarkGray".to_string(),
                    look_cursor: super::super::config::CursorColors {
                        bg: "White".to_string(),
                        fg: "Black".to_string(),
                    },
                    hit_flash: super::super::config::FlashColors {
                        bg: "Red".to_string(),
                        fg: "White".to_string(),
                    },
                },
            },
            lighting: super::super::config::LightingConfig {
                enabled: true,
                ambient_level: 20,
                visibility_threshold: 50,
                glare_threshold: 220,
                player_light: super::super::config::PlayerLightConfig {
                    enabled: true,
                    radius: 5,
                    intensity: 100,
                },
                equipment_light_detection: false,
                smooth_falloff: true,
                recalculate_on_turn: true,
            },
            effects: super::super::config::EffectsConfig {
                enabled: true,
                frame_rate: 60,
                max_effects_per_entity: 3,
                priority_order: vec![],
            },
            animations: super::super::config::AnimationConfig {
                glass_shimmer: super::super::config::GlassShimmerConfig {
                    enabled: true,
                    speed: 4,
                    colors: vec![],
                },
                tile_animations: true,
                entity_animations: true,
            },
            rendering: super::super::config::RenderingConfig {
                god_view_override: false,
                smooth_camera: true,
                damage_numbers: true,
                status_effect_priority: vec!["burn".to_string(), "poison".to_string()],
                hit_flash_duration: 2,
            },
            performance: super::super::config::PerformanceConfig {
                target_fps: 60,
                viewport_culling: true,
                frame_limiting: true,
                optimization_level: "balanced".to_string(),
            },
            particles: crate::renderer::particles::ParticleConfig::default(),
            visual_animations: crate::renderer::animations::VisualAnimationConfig::default(),
        };
        
        let renderer = EntityRenderer::new(&config);
        
        // Test coordinate conversion
        assert_eq!(renderer.world_to_screen(5, 5, 0, 0, 10, 10), Some((5, 5)));
        assert_eq!(renderer.world_to_screen(-1, 5, 0, 0, 10, 10), None);
        assert_eq!(renderer.world_to_screen(15, 5, 0, 0, 10, 10), None);
    }
}
