//! Lighting system for the renderer

use crate::GameState;
use super::config::RenderConfig;

/// Handles lighting calculations and light level queries
pub struct LightingRenderer {
    config: RenderConfig,
    light_map: Vec<u8>,
    needs_recalculation: bool,
}

impl LightingRenderer {
    pub fn new(config: &RenderConfig) -> Self {
        Self {
            config: config.clone(),
            light_map: Vec::new(),
            needs_recalculation: true,
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: &RenderConfig) {
        self.config = config.clone();
        self.needs_recalculation = true;
    }

    /// Calculate lighting for the entire map
    pub fn calculate_lighting(&mut self, state: &GameState) -> Vec<u8> {
        if !self.config.lighting.enabled {
            // Return full brightness if lighting is disabled
            return vec![255; (state.map.width * state.map.height) as usize];
        }

        // Check if we need to recalculate
        if !self.needs_recalculation && !self.config.lighting.recalculate_on_turn {
            return self.light_map.clone();
        }

        let map_size = (state.map.width * state.map.height) as usize;
        let mut light_map = vec![self.config.lighting.ambient_level; map_size];

        // Collect all light sources
        let mut light_sources = Vec::new();

        // Add player light if enabled (force for testing)
        light_sources.push(LightSource {
            x: state.player_x,
            y: state.player_y,
            radius: 16,  // Much larger radius
            intensity: 255,  // Maximum intensity
        });

        // Add equipment light sources if detection is enabled
        if self.config.lighting.equipment_light_detection {
            // Equipment is a struct, not a collection, so we need to check individual fields
            if let Some(ref weapon) = state.equipment.weapon {
                if let Some(light_radius) = self.get_item_light_radius(weapon) {
                    light_sources.push(LightSource {
                        x: state.player_x,
                        y: state.player_y,
                        radius: light_radius,
                        intensity: 150, // Equipment light intensity
                    });
                }
            }
            if let Some(ref ranged_weapon) = state.equipment.ranged_weapon {
                if let Some(light_radius) = self.get_item_light_radius(ranged_weapon) {
                    light_sources.push(LightSource {
                        x: state.player_x,
                        y: state.player_y,
                        radius: light_radius,
                        intensity: 150, // Equipment light intensity
                    });
                }
            }
        }

        // Add map light sources
        for map_light in &state.map.lights {
            if let Some(light_def) = crate::get_light_def(&map_light.id) {
                light_sources.push(LightSource {
                    x: map_light.x,
                    y: map_light.y,
                    radius: light_def.radius,
                    intensity: light_def.intensity,
                });
            }
        }

        // Calculate lighting from all sources
        for source in &light_sources {
            self.apply_light_source(&mut light_map, source, state.map.width, state.map.height);
        }

        self.light_map = light_map.clone();
        self.needs_recalculation = false;
        light_map
    }

    /// Apply a single light source to the light map
    fn apply_light_source(&self, light_map: &mut [u8], source: &LightSource, map_width: usize, map_height: usize) {
        for dy in -source.radius..=source.radius {
            for dx in -source.radius..=source.radius {
                let x = source.x + dx;
                let y = source.y + dy;
                
                if x < 0 || y < 0 || x >= map_width as i32 || y >= map_height as i32 {
                    continue;
                }

                let dist_sq = dx * dx + dy * dy;
                
                // Use larger radius for smoother gradients
                let extended_radius = source.radius + 4;
                let extended_radius_sq = extended_radius * extended_radius;
                
                if dist_sq <= extended_radius_sq {
                    let idx = y as usize * map_width + x as usize;
                    
                    // Very slow falloff for long range
                    let dist = (dist_sq as f32).sqrt();
                    // Ensure center tile (dist=0) gets full intensity
                    let falloff = if dist == 0.0 {
                        1.0
                    } else {
                        (1.0 - (dist / (extended_radius as f32 * 1.2))).max(0.0)
                    };
                    
                    let light_contribution = (source.intensity as f32 * falloff) as u8;
                    let new_light = (light_map[idx] as u16 + light_contribution as u16).min(255) as u8;
                    light_map[idx] = new_light;
                }
            }
        }
    }

    /// Get light level at specific coordinates
    pub fn get_light_level(&self, x: i32, y: i32, map_width: usize, map_height: usize) -> u8 {
        if x < 0 || y < 0 || x >= map_width as i32 || y >= map_height as i32 {
            return 0;
        }
        
        let idx = y as usize * map_width + x as usize;
        self.light_map.get(idx).copied().unwrap_or(self.config.lighting.ambient_level)
    }

    /// Check if a position is visible (above visibility threshold)
    pub fn is_visible(&self, x: i32, y: i32, map_width: usize, map_height: usize) -> bool {
        self.get_light_level(x, y, map_width, map_height) >= self.config.lighting.visibility_threshold
    }

    /// Check if a position has glare (too bright)
    pub fn has_glare(&self, x: i32, y: i32, map_width: usize, map_height: usize) -> bool {
        self.get_light_level(x, y, map_width, map_height) >= self.config.lighting.glare_threshold
    }

    /// Dim a color based on light level
    pub fn dim_color(&self, color: ratatui::prelude::Color, light_level: u8) -> ratatui::prelude::Color {
        if light_level >= 200 { 
            return color; 
        }
        
        let factor = light_level as f32 / 255.0;
        
        match color {
            ratatui::prelude::Color::Rgb(r, g, b) => ratatui::prelude::Color::Rgb(
                (r as f32 * factor) as u8,
                (g as f32 * factor) as u8,
                (b as f32 * factor) as u8,
            ),
            ratatui::prelude::Color::Gray | ratatui::prelude::Color::DarkGray => {
                if light_level < 100 { 
                    ratatui::prelude::Color::Black 
                } else { 
                    ratatui::prelude::Color::DarkGray 
                }
            },
            ratatui::prelude::Color::Cyan => {
                if light_level < 100 { 
                    ratatui::prelude::Color::DarkGray 
                } else { 
                    color 
                }
            },
            _ => {
                if light_level < 100 { 
                    ratatui::prelude::Color::DarkGray 
                } else { 
                    color 
                }
            }
        }
    }

    /// Mark that lighting needs recalculation
    pub fn mark_dirty(&mut self) {
        self.needs_recalculation = true;
    }

    /// Get item light radius for equipment detection
    fn get_item_light_radius(&self, item_id: &str) -> Option<i32> {
        // This should be loaded from item definitions or configuration
        match item_id {
            "torch" => Some(6),
            "lantern" => Some(8),
            "glowing_crystal" => Some(4),
            _ => None,
        }
    }
}

/// A light source with position and properties
#[derive(Debug, Clone)]
struct LightSource {
    x: i32,
    y: i32,
    radius: i32,
    intensity: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::config::*;

    fn create_test_config() -> RenderConfig {
        RenderConfig {
            colors: ColorConfig {
                entities: EntityColors {
                    player: PlayerColors {
                        base: "Yellow".to_string(),
                        status_effects: std::collections::HashMap::new(),
                    },
                    enemies: std::collections::HashMap::new(),
                    npcs: NpcColors { base: "Green".to_string() },
                    items: ItemColors { base: "LightMagenta".to_string() },
                },
                tiles: TileColors {
                    floor: "DarkGray".to_string(),
                    wall: "Gray".to_string(),
                    glass: GlassColors {
                        base: "Cyan".to_string(),
                        shimmer: vec!["Cyan".to_string()],
                    },
                    stairs_down: "Yellow".to_string(),
                    stairs_up: "Yellow".to_string(),
                    world_exit: "Green".to_string(),
                },
                lighting: LightingColors {
                    sources: std::collections::HashMap::new(),
                    ambient: 20,
                    visibility_threshold: 50,
                },
                ui: UiColors {
                    revealed_tile: "DarkGray".to_string(),
                    look_cursor: CursorColors {
                        bg: "White".to_string(),
                        fg: "Black".to_string(),
                    },
                    hit_flash: FlashColors {
                        bg: "Red".to_string(),
                        fg: "White".to_string(),
                    },
                },
            },
            lighting: LightingConfig {
                enabled: true,
                ambient_level: 20,
                visibility_threshold: 50,
                glare_threshold: 220,
                player_light: PlayerLightConfig {
                    enabled: true,
                    radius: 5,
                    intensity: 100,
                },
                equipment_light_detection: false,
                smooth_falloff: true,
                recalculate_on_turn: true,
            },
            effects: EffectsConfig {
                enabled: true,
                frame_rate: 60,
                max_effects_per_entity: 3,
                priority_order: vec![],
            },
            animations: AnimationConfig {
                glass_shimmer: GlassShimmerConfig {
                    enabled: true,
                    speed: 4,
                    colors: vec![],
                },
                tile_animations: true,
                entity_animations: true,
            },
            rendering: RenderingConfig {
                god_view_override: false,
                smooth_camera: true,
                damage_numbers: true,
                status_effect_priority: vec![],
                hit_flash_duration: 2,
            },
        }
    }

    #[test]
    fn test_lighting_disabled() {
        let mut config = create_test_config();
        config.lighting.enabled = false;
        
        let mut lighting = LightingRenderer::new(&config);
        // Create a minimal game state for testing
        // This would need to be adapted based on your actual GameState structure
    }

    #[test]
    fn test_dim_color() {
        let config = create_test_config();
        let lighting = LightingRenderer::new(&config);
        
        let bright_red = ratatui::prelude::Color::Red;
        let dimmed = lighting.dim_color(bright_red, 128);
        // Should be dimmed but still recognizable
        assert_ne!(dimmed, bright_red);
        
        let full_bright = lighting.dim_color(bright_red, 255);
        assert_eq!(full_bright, bright_red);
    }
}
