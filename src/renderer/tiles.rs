//! Tile rendering system

use ratatui::prelude::*;
use crate::{GameState, Tile};
use super::config::{RenderConfig, parse_color};

/// Handles tile rendering with animations and lighting
pub struct TileRenderer {
    config: RenderConfig,
}

impl TileRenderer {
    pub fn new(config: &RenderConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: &RenderConfig) {
        self.config = config.clone();
    }

    /// Render all tiles and return spans for each screen position
    pub fn render_tiles(
        &self,
        state: &GameState,
        light_map: &[u8],
        frame_count: u64,
        cam_x: i32,
        cam_y: i32,
        view_width: i32,
        view_height: i32,
    ) -> Vec<Vec<Span<'_>>> {
        let mut tile_spans = Vec::new();

        for vy in 0..view_height {
            let mut row_spans = Vec::new();
            let y = cam_y + vy;
            
            for vx in 0..view_width {
                let x = cam_x + vx;
                
                if x < 0 || y < 0 || x >= state.map.width as i32 || y >= state.map.height as i32 {
                    // Out of bounds - render empty space
                    row_spans.push(Span::raw(" "));
                    continue;
                }

                let idx = state.map.idx(x, y);
                let light_level = self.get_light_level(x, y, light_map, state.map.width, state.map.height);
                let visible = light_level > 80 || state.debug_god_view;
                let revealed = state.revealed.contains(&idx) || state.debug_god_view;

                let span = if visible {
                    self.render_visible_tile(state, x, y, idx, light_map, frame_count)
                } else if revealed {
                    self.render_revealed_tile(state, idx)
                } else {
                    Span::raw(" ")
                };

                row_spans.push(span);
            }
            tile_spans.push(row_spans);
        }

        tile_spans
    }

    /// Render a visible tile with full lighting and animations
    fn render_visible_tile(&self, state: &GameState, x: i32, y: i32, idx: usize, light_map: &[u8], frame_count: u64) -> Span<'_> {
        let tile = &state.map.tiles[idx];
        let light_level = self.get_light_level(x, y, light_map, state.map.width, state.map.height);

        let (glyph, base_color) = self.get_tile_appearance(tile, x, y, frame_count, state.animation_frame);
        let final_color = self.dim_color(base_color, light_level);
        
        Span::styled(glyph.to_string(), Style::default().fg(final_color))
    }

    /// Render a revealed but not visible tile
    fn render_revealed_tile(&self, _state: &GameState, _idx: usize) -> Span<'_> {
        let tile = &_state.map.tiles[_idx];
        let glyph = tile.glyph();
        let color = parse_color(&self.config.colors.ui.revealed_tile);
        
        Span::styled(glyph.to_string(), Style::default().fg(color))
    }

    /// Get the appearance (glyph and color) for a tile
    fn get_tile_appearance(&self, tile: &Tile, x: i32, y: i32, _frame_count: u64, animation_frame: u32) -> (char, Color) {
        match tile {
            Tile::Glass => {
                if self.config.animations.glass_shimmer.enabled && self.config.animations.tile_animations {
                    // Glass shimmers between different shades
                    let phase = ((animation_frame / self.config.animations.glass_shimmer.speed as u32) + (x as u32 ^ y as u32)) % self.config.animations.glass_shimmer.colors.len() as u32;
                    let color_name = &self.config.animations.glass_shimmer.colors[phase as usize];
                    let color = parse_color(color_name);
                    ('░', color)
                } else {
                    let color = parse_color(&self.config.colors.tiles.glass.base);
                    ('░', color)
                }
            }
            Tile::Floor => {
                let color = parse_color(&self.config.colors.tiles.floor);
                ('.', color)
            }
            Tile::Wall { .. } => {
                let color = parse_color(&self.config.colors.tiles.wall);
                ('#', color)
            }
            Tile::StairsDown => {
                let color = parse_color(&self.config.colors.tiles.stairs_down);
                ('>', color)
            }
            Tile::StairsUp => {
                let color = parse_color(&self.config.colors.tiles.stairs_up);
                ('<', color)
            }
            Tile::WorldExit => {
                let color = parse_color(&self.config.colors.tiles.world_exit);
                ('▣', color)
            }
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

    /// Simple color dimming that's more visible
    fn simple_dim_color(&self, color: Color, light_level: u8) -> Color {
        match light_level {
            200..=255 => color,           // Bright: original color
            150..=199 => Color::Gray,     // Medium: gray
            100..=149 => Color::DarkGray, // Dim: dark gray
            50..=99 => Color::Black,      // Very dim: black
            _ => Color::Black,            // No light: black
        }
    }

    /// Dim a color based on light level
    fn dim_color(&self, color: Color, light_level: u8) -> Color {
        match color {
            Color::Rgb(r, g, b) => {
                let factor = light_level as f32 / 255.0;
                Color::Rgb(
                    (r as f32 * factor) as u8,
                    (g as f32 * factor) as u8,
                    (b as f32 * factor) as u8,
                )
            },
            Color::Gray => {
                if light_level >= 150 { 
                    Color::White 
                } else if light_level >= 100 { 
                    Color::Gray 
                } else if light_level >= 50 {
                    Color::DarkGray
                } else {
                    Color::Black
                }
            },
            Color::DarkGray => {
                // Fixed: DarkGray should brighten to White at high light levels
                if light_level >= 200 { 
                    Color::White    // Very bright: white
                } else if light_level >= 150 { 
                    Color::Gray     // Bright: gray
                } else if light_level >= 100 { 
                    Color::DarkGray // Medium: dark gray
                } else {
                    Color::DarkGray // Low: still dark gray (no black)
                }
            },
            _ => {
                if light_level >= 200 { 
                    color 
                } else if light_level >= 100 { 
                    color 
                } else { 
                    Color::DarkGray 
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_appearance() {
        let config = RenderConfig {
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
                        shimmer: vec!["Cyan".to_string(), "LightCyan".to_string()],
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
                    colors: vec!["Cyan".to_string(), "LightCyan".to_string()],
                },
                tile_animations: true,
                entity_animations: true,
            },
            rendering: super::super::config::RenderingConfig {
                god_view_override: false,
                smooth_camera: true,
                damage_numbers: true,
                status_effect_priority: vec![],
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
        
        let renderer = TileRenderer::new(&config);
        
        // Test floor tile
        let (glyph, _color) = renderer.get_tile_appearance(&Tile::Floor, 0, 0, 0, 0);
        assert_eq!(glyph, '.');
        
        // Test glass tile
        let (glyph, _color) = renderer.get_tile_appearance(&Tile::Glass, 0, 0, 0, 0);
        assert_eq!(glyph, '░');
    }
}
