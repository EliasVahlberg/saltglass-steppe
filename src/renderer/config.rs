//! Rendering configuration loaded from JSON

use serde::{Deserialize, Serialize};
use crate::renderer::particles::ParticleConfig;
use crate::renderer::animations::VisualAnimationConfig;
use ratatui::prelude::Color;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderConfig {
    pub colors: ColorConfig,
    pub lighting: LightingConfig,
    pub effects: EffectsConfig,
    pub animations: AnimationConfig,
    pub visual_animations: VisualAnimationConfig,
    pub rendering: RenderingConfig,
    pub performance: PerformanceConfig,
    pub particles: ParticleConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorConfig {
    pub entities: EntityColors,
    pub tiles: TileColors,
    pub lighting: LightingColors,
    pub ui: UiColors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityColors {
    pub player: PlayerColors,
    pub enemies: HashMap<String, String>,
    pub npcs: NpcColors,
    pub items: ItemColors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerColors {
    pub base: String,
    pub status_effects: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NpcColors {
    pub base: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemColors {
    pub base: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TileColors {
    pub floor: String,
    pub wall: String,
    pub glass: GlassColors,
    pub stairs_down: String,
    pub stairs_up: String,
    pub world_exit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlassColors {
    pub base: String,
    pub shimmer: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LightingColors {
    pub sources: HashMap<String, String>,
    pub ambient: u8,
    pub visibility_threshold: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiColors {
    pub revealed_tile: String,
    pub look_cursor: CursorColors,
    pub hit_flash: FlashColors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorColors {
    pub bg: String,
    pub fg: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FlashColors {
    pub bg: String,
    pub fg: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LightingConfig {
    pub enabled: bool,
    pub ambient_level: u8,
    pub visibility_threshold: u8,
    pub glare_threshold: u8,
    pub player_light: PlayerLightConfig,
    pub equipment_light_detection: bool,
    pub smooth_falloff: bool,
    pub recalculate_on_turn: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerLightConfig {
    pub enabled: bool,
    pub radius: i32,
    pub intensity: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EffectsConfig {
    pub enabled: bool,
    pub frame_rate: u64,
    pub max_effects_per_entity: usize,
    pub priority_order: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnimationConfig {
    pub glass_shimmer: GlassShimmerConfig,
    pub tile_animations: bool,
    pub entity_animations: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlassShimmerConfig {
    pub enabled: bool,
    pub speed: u64,
    pub colors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderingConfig {
    pub god_view_override: bool,
    pub smooth_camera: bool,
    pub damage_numbers: bool,
    pub status_effect_priority: Vec<String>,
    pub hit_flash_duration: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    pub target_fps: u32,
    pub viewport_culling: bool,
    pub frame_limiting: bool,
    pub optimization_level: String,
}

impl RenderConfig {
    /// Load configuration from JSON file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: RenderConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to JSON file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Convert string color name to ratatui Color
pub fn parse_color(color_str: &str) -> Color {
    match color_str {
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
        "Cyan" => Color::Cyan,
        "White" => Color::White,
        "Black" => Color::Black,
        "DarkGray" => Color::DarkGray,
        "LightRed" => Color::LightRed,
        "LightGreen" => Color::LightGreen,
        "LightYellow" => Color::LightYellow,
        "LightBlue" => Color::LightBlue,
        "LightMagenta" => Color::LightMagenta,
        "LightCyan" => Color::LightCyan,
        "Gray" => Color::Gray,
        s if s.starts_with("Rgb(") && s.ends_with(')') => {
            // Parse "Rgb(r, g, b)" format
            let inner = &s[4..s.len()-1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>(), parts[2].parse::<u8>()) {
                    return Color::Rgb(r, g, b);
                }
            }
            Color::White // fallback
        }
        _ => Color::White, // fallback for unknown colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("Red"), Color::Red);
        assert_eq!(parse_color("Rgb(255, 140, 0)"), Color::Rgb(255, 140, 0));
        assert_eq!(parse_color("Unknown"), Color::White);
    }

    #[test]
    fn test_config_serialization() {
        let config = RenderConfig {
            colors: ColorConfig {
                entities: EntityColors {
                    player: PlayerColors {
                        base: "Yellow".to_string(),
                        status_effects: HashMap::new(),
                    },
                    enemies: HashMap::new(),
                    npcs: NpcColors { base: "Green".to_string() },
                    items: ItemColors { base: "LightMagenta".to_string() },
                },
                tiles: TileColors {
                    floor: "DarkGray".to_string(),
                    wall: "Gray".to_string(),
                    glass: GlassColors {
                        base: "Cyan".to_string(),
                        shimmer: vec!["Cyan".to_string(), "LightCyan".to_string()],
                    },
                    stairs_down: "Yellow".to_string(),
                    stairs_up: "Yellow".to_string(),
                    world_exit: "Green".to_string(),
                },
                lighting: LightingColors {
                    sources: HashMap::new(),
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
                equipment_light_detection: true,
                smooth_falloff: true,
                recalculate_on_turn: true,
            },
            effects: EffectsConfig {
                enabled: true,
                frame_rate: 60,
                max_effects_per_entity: 3,
                priority_order: vec!["HitFlash".to_string(), "Blink".to_string()],
            },
            animations: AnimationConfig {
                glass_shimmer: GlassShimmerConfig {
                    enabled: true,
                    speed: 4,
                    colors: vec!["Cyan".to_string()],
                },
                tile_animations: true,
                entity_animations: true,
            },
            rendering: RenderingConfig {
                god_view_override: false,
                smooth_camera: true,
                damage_numbers: true,
                status_effect_priority: vec!["burn".to_string()],
                hit_flash_duration: 2,
            },
            performance: PerformanceConfig {
                target_fps: 60,
                viewport_culling: true,
                frame_limiting: true,
                optimization_level: "balanced".to_string(),
            },
            particles: ParticleConfig::default(),
            visual_animations: VisualAnimationConfig::default(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: RenderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.lighting.enabled, true);
    }
}
