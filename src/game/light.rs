use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Light beam direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// Light beam properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightBeam {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
    pub intensity: u8,
    pub color: LightColor,
    pub range: u8,
}

/// Light colors with different properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightColor {
    White,
    Red,
    Blue,
    Green,
    Yellow,
    Violet,
}

/// Light source that can emit beams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSource {
    pub x: i32,
    pub y: i32,
    pub intensity: u8,
    pub color: LightColor,
    pub active: bool,
}

/// Refraction surface that can bend light
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefractionSurface {
    pub x: i32,
    pub y: i32,
    pub angle: u8,
    pub efficiency: f32,
}

/// Light manipulation system state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LightSystem {
    pub light_sources: Vec<LightSource>,
    pub refraction_surfaces: Vec<RefractionSurface>,
    pub active_beams: Vec<LightBeam>,
    pub light_energy: u32,
}

impl LightSystem {
    /// Update light system each turn
    pub fn update(&mut self, rng: &mut ChaCha8Rng) {
        // Decay beam intensity over time
        self.active_beams.retain_mut(|beam| {
            beam.intensity = beam.intensity.saturating_sub(1);
            beam.intensity > 0
        });

        // Random light fluctuations
        for source in &mut self.light_sources {
            if source.active && rng.gen_bool(0.1) {
                let change = rng.gen_range(-1..=1);
                source.intensity = (source.intensity as i32 + change).clamp(1, 10) as u8;
            }
        }
    }
}
