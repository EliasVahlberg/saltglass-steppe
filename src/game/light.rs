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

impl Direction {
    pub fn to_delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
        }
    }
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
    White,  // Standard light
    Red,    // Heat/damage
    Blue,   // Cold/slowing
    Green,  // Healing
    Yellow, // Revealing/detection
    Violet, // Psychic enhancement
}

impl LightColor {
    pub fn damage_modifier(self) -> f32 {
        match self {
            LightColor::Red => 1.5,
            LightColor::Blue => 0.8,
            LightColor::White => 1.0,
            _ => 1.0,
        }
    }
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
    pub angle: u8,       // 0-7 representing 8 directions
    pub efficiency: f32, // 0.0-1.0
}

/// Light manipulation system state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LightSystem {
    pub light_sources: Vec<LightSource>,
    pub refraction_surfaces: Vec<RefractionSurface>,
    pub active_beams: Vec<LightBeam>,
    pub light_energy: u32, // Player's stored light energy
}

impl LightSystem {
    /// Create a new light beam from a source
    pub fn create_beam(
        &mut self,
        source_x: i32,
        source_y: i32,
        direction: Direction,
        intensity: u8,
        color: LightColor,
        range: u8,
    ) {
        let beam = LightBeam {
            x: source_x,
            y: source_y,
            direction,
            intensity,
            color,
            range,
        };
        self.active_beams.push(beam);
    }

    /// Calculate light beam path with refraction
    pub fn trace_beam(&self, beam: &LightBeam, map: &crate::game::Map) -> Vec<(i32, i32)> {
        let mut path = Vec::new();
        let mut current_x = beam.x;
        let mut current_y = beam.y;
        let mut current_dir = beam.direction;
        let mut remaining_range = beam.range;

        while remaining_range > 0 {
            let (dx, dy) = current_dir.to_delta();
            current_x += dx;
            current_y += dy;

            // Check bounds
            if current_x < 0
                || current_y < 0
                || current_x >= map.width as i32
                || current_y >= map.height as i32
            {
                break;
            }

            path.push((current_x, current_y));

            // Check for refraction surfaces
            if let Some(surface) = self
                .refraction_surfaces
                .iter()
                .find(|s| s.x == current_x && s.y == current_y)
            {
                // Refract the beam
                current_dir = self.refract_direction(current_dir, surface.angle);
            }

            // Check for walls (stop beam)
            let idx = map.idx(current_x, current_y);
            if matches!(map.tiles[idx], crate::game::map::Tile::Wall { .. }) {
                break;
            }

            remaining_range -= 1;
        }

        path
    }

    /// Refract light direction based on surface angle
    fn refract_direction(&self, incoming: Direction, surface_angle: u8) -> Direction {
        let directions = [
            Direction::North,
            Direction::NorthEast,
            Direction::East,
            Direction::SouthEast,
            Direction::South,
            Direction::SouthWest,
            Direction::West,
            Direction::NorthWest,
        ];

        let incoming_idx = directions.iter().position(|&d| d == incoming).unwrap_or(0);
        let reflection_offset = (surface_angle as usize * 2) % 8;
        let new_idx = (incoming_idx + reflection_offset) % 8;

        directions[new_idx]
    }

    /// Add light source at position
    pub fn add_light_source(&mut self, x: i32, y: i32, intensity: u8, color: LightColor) {
        self.light_sources.push(LightSource {
            x,
            y,
            intensity,
            color,
            active: true,
        });
    }

    /// Add refraction surface at position
    pub fn add_refraction_surface(&mut self, x: i32, y: i32, angle: u8, efficiency: f32) {
        self.refraction_surfaces.push(RefractionSurface {
            x,
            y,
            angle,
            efficiency,
        });
    }

    /// Calculate light damage at position
    pub fn calculate_light_damage(&self, x: i32, y: i32, map: &crate::game::Map) -> u32 {
        let mut total_damage = 0;

        for beam in &self.active_beams {
            let path = self.trace_beam(beam, map);
            if path.contains(&(x, y)) {
                let base_damage = beam.intensity as u32;
                let color_modifier = beam.color.damage_modifier();
                total_damage += (base_damage as f32 * color_modifier) as u32;
            }
        }

        total_damage
    }

    /// Check if position is illuminated
    pub fn is_illuminated(&self, x: i32, y: i32, map: &crate::game::Map) -> bool {
        // Check direct light sources
        for source in &self.light_sources {
            if !source.active {
                continue;
            }
            let distance = ((x - source.x).abs() + (y - source.y).abs()) as u8;
            if distance <= source.intensity {
                return true;
            }
        }

        // Check light beams
        for beam in &self.active_beams {
            let path = self.trace_beam(beam, map);
            if path.contains(&(x, y)) {
                return true;
            }
        }

        false
    }

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

    /// Player ability: Focus light beam
    pub fn focus_beam(
        &mut self,
        player_x: i32,
        player_y: i32,
        direction: Direction,
        cost: u32,
    ) -> bool {
        if self.light_energy < cost {
            return false;
        }

        self.light_energy -= cost;
        self.create_beam(player_x, player_y, direction, 5, LightColor::White, 8);
        true
    }

    /// Player ability: Create light prism
    pub fn create_prism(&mut self, x: i32, y: i32, cost: u32) -> bool {
        if self.light_energy < cost {
            return false;
        }

        self.light_energy -= cost;
        self.add_refraction_surface(x, y, 2, 0.8);
        true
    }

    /// Gain light energy from sources
    pub fn absorb_light(&mut self, x: i32, y: i32, map: &crate::game::Map) -> u32 {
        let mut absorbed = 0;

        for source in &self.light_sources {
            if !source.active {
                continue;
            }
            let distance = ((x - source.x).abs() + (y - source.y).abs()) as u8;
            if distance <= 2 {
                absorbed += source.intensity as u32;
            }
        }

        // Check beams
        for beam in &self.active_beams {
            let path = self.trace_beam(beam, map);
            if path.contains(&(x, y)) {
                absorbed += beam.intensity as u32;
            }
        }

        self.light_energy += absorbed;
        absorbed
    }
}
