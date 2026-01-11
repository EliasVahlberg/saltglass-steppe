//! Particle effects system for visual enhancements

use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

/// A single particle with position, velocity, and visual properties
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub color: Color,
    pub character: char,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
    pub brightness: f32,
    pub effect_type: ParticleType,
}

/// Types of particle effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticleType {
    Sparkle,
    Glow,
    Float,
    Drift,
    Pulse,
    Shimmer,
}

/// Configuration for particle effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleConfig {
    pub sparkles: SparkleConfig,
    pub glow: GlowConfig,
    pub float: FloatConfig,
    pub drift: DriftConfig,
    pub pulse: PulseConfig,
    pub shimmer: ShimmerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkleConfig {
    pub enabled: bool,
    pub spawn_rate: f32,
    pub lifetime: f32,
    pub colors: Vec<String>,
    pub characters: Vec<char>,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlowConfig {
    pub enabled: bool,
    pub pulse_speed: f32,
    pub min_brightness: f32,
    pub max_brightness: f32,
    pub colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatConfig {
    pub enabled: bool,
    pub velocity_range: (f32, f32),
    pub lifetime: f32,
    pub gravity: f32,
    pub characters: Vec<char>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfig {
    pub enabled: bool,
    pub wind_strength: f32,
    pub turbulence: f32,
    pub fade_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseConfig {
    pub enabled: bool,
    pub frequency: f32,
    pub amplitude: f32,
    pub wave_type: String, // "sine", "square", "triangle"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimmerConfig {
    pub enabled: bool,
    pub speed: f32,
    pub color_cycle: Vec<String>,
    pub intensity_variation: f32,
}

/// Particle system manager
pub struct ParticleSystem {
    particles: Vec<Particle>,
    config: ParticleConfig,
    frame_count: u64,
    spawn_timer: f32,
    max_particles: usize,
}

impl ParticleSystem {
    /// Create a new particle system
    pub fn new(config: ParticleConfig) -> Self {
        Self {
            particles: Vec::new(),
            config,
            frame_count: 0,
            spawn_timer: 0.0,
            max_particles: 100,
        }
    }

    /// Update all particles
    pub fn update(&mut self, delta_time: f32) {
        self.frame_count += 1;
        self.spawn_timer += delta_time;

        // Update existing particles
        for particle in &mut self.particles {
            Self::update_particle_static(particle, delta_time, &self.config, self.frame_count);
        }

        // Remove dead particles
        self.particles.retain(|p| p.lifetime > 0.0);

        // Spawn new particles based on configuration
        self.spawn_particles(delta_time);
    }

    /// Update a single particle based on its type (static version to avoid borrowing issues)
    fn update_particle_static(
        particle: &mut Particle,
        delta_time: f32,
        config: &ParticleConfig,
        frame_count: u64,
    ) {
        // Update position
        particle.position.0 += particle.velocity.0 * delta_time;
        particle.position.1 += particle.velocity.1 * delta_time;

        // Update lifetime
        particle.lifetime -= delta_time;

        // Type-specific updates
        match particle.effect_type {
            ParticleType::Sparkle => {
                // Sparkles fade quickly and twinkle
                let life_ratio = particle.lifetime / particle.max_lifetime;
                particle.brightness = (life_ratio * 2.0).min(1.0);
            }
            ParticleType::Glow => {
                // Glowing particles pulse
                let pulse = (frame_count as f32 * config.glow.pulse_speed).sin();
                particle.brightness = config.glow.min_brightness
                    + (config.glow.max_brightness - config.glow.min_brightness)
                        * (pulse * 0.5 + 0.5);
            }
            ParticleType::Float => {
                // Floating particles affected by gravity
                particle.velocity.1 += config.float.gravity * delta_time;
            }
            ParticleType::Drift => {
                // Drifting particles affected by wind
                particle.velocity.0 += config.drift.wind_strength * delta_time;
                particle.brightness *= 1.0 - (config.drift.fade_rate * delta_time);
            }
            ParticleType::Pulse => {
                // Pulsing particles change size
                let pulse = (frame_count as f32 * config.pulse.frequency).sin();
                particle.size = 1.0 + config.pulse.amplitude * pulse;
            }
            ParticleType::Shimmer => {
                // Shimmering particles cycle through colors
                let cycle_pos = (frame_count as f32 * config.shimmer.speed)
                    % config.shimmer.color_cycle.len() as f32;
                let color_index = cycle_pos as usize % config.shimmer.color_cycle.len();
                particle.color = parse_color(&config.shimmer.color_cycle[color_index]);
            }
        }
    }

    /// Spawn new particles based on configuration
    fn spawn_particles(&mut self, _delta_time: f32) {
        if self.particles.len() >= self.max_particles {
            return;
        }

        // Spawn sparkles
        if self.config.sparkles.enabled && self.spawn_timer >= 1.0 / self.config.sparkles.spawn_rate
        {
            self.spawn_sparkle();
            self.spawn_timer = 0.0;
        }
    }

    /// Spawn a sparkle particle
    fn spawn_sparkle(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let color_name =
            &self.config.sparkles.colors[rng.gen_range(0..self.config.sparkles.colors.len())];
        let character = self.config.sparkles.characters
            [rng.gen_range(0..self.config.sparkles.characters.len())];

        let particle = Particle {
            position: (rng.gen_range(0.0..80.0), rng.gen_range(0.0..24.0)),
            velocity: (rng.gen_range(-2.0..2.0), rng.gen_range(-1.0..1.0)),
            color: parse_color(color_name),
            character,
            lifetime: self.config.sparkles.lifetime,
            max_lifetime: self.config.sparkles.lifetime,
            size: 1.0,
            brightness: self.config.sparkles.intensity,
            effect_type: ParticleType::Sparkle,
        };

        self.particles.push(particle);
    }

    /// Add a particle at a specific location
    pub fn add_particle(&mut self, x: f32, y: f32, effect_type: ParticleType) {
        if self.particles.len() >= self.max_particles {
            return;
        }

        let particle = match effect_type {
            ParticleType::Sparkle => self.create_sparkle_at(x, y),
            ParticleType::Glow => self.create_glow_at(x, y),
            ParticleType::Float => self.create_float_at(x, y),
            ParticleType::Drift => self.create_drift_at(x, y),
            ParticleType::Pulse => self.create_pulse_at(x, y),
            ParticleType::Shimmer => self.create_shimmer_at(x, y),
        };

        self.particles.push(particle);
    }

    /// Create specific particle types
    fn create_sparkle_at(&self, x: f32, y: f32) -> Particle {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let color_name =
            &self.config.sparkles.colors[rng.gen_range(0..self.config.sparkles.colors.len())];
        let character = self.config.sparkles.characters
            [rng.gen_range(0..self.config.sparkles.characters.len())];

        Particle {
            position: (x, y),
            velocity: (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
            color: parse_color(color_name),
            character,
            lifetime: self.config.sparkles.lifetime,
            max_lifetime: self.config.sparkles.lifetime,
            size: 1.0,
            brightness: self.config.sparkles.intensity,
            effect_type: ParticleType::Sparkle,
        }
    }

    fn create_glow_at(&self, x: f32, y: f32) -> Particle {
        let color_name = &self.config.glow.colors[0];

        Particle {
            position: (x, y),
            velocity: (0.0, 0.0),
            color: parse_color(color_name),
            character: '●',
            lifetime: 5.0, // Glows last longer
            max_lifetime: 5.0,
            size: 1.0,
            brightness: self.config.glow.max_brightness,
            effect_type: ParticleType::Glow,
        }
    }

    fn create_float_at(&self, x: f32, y: f32) -> Particle {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let character =
            self.config.float.characters[rng.gen_range(0..self.config.float.characters.len())];
        let velocity =
            rng.gen_range(self.config.float.velocity_range.0..self.config.float.velocity_range.1);

        Particle {
            position: (x, y),
            velocity: (0.0, -velocity), // Float upward
            color: Color::White,
            character,
            lifetime: self.config.float.lifetime,
            max_lifetime: self.config.float.lifetime,
            size: 1.0,
            brightness: 1.0,
            effect_type: ParticleType::Float,
        }
    }

    fn create_drift_at(&self, x: f32, y: f32) -> Particle {
        Particle {
            position: (x, y),
            velocity: (0.0, 0.0),
            color: Color::Gray,
            character: '~',
            lifetime: 3.0,
            max_lifetime: 3.0,
            size: 1.0,
            brightness: 1.0,
            effect_type: ParticleType::Drift,
        }
    }

    fn create_pulse_at(&self, x: f32, y: f32) -> Particle {
        Particle {
            position: (x, y),
            velocity: (0.0, 0.0),
            color: Color::Yellow,
            character: '◆',
            lifetime: 2.0,
            max_lifetime: 2.0,
            size: 1.0,
            brightness: 1.0,
            effect_type: ParticleType::Pulse,
        }
    }

    fn create_shimmer_at(&self, x: f32, y: f32) -> Particle {
        let color_name = &self.config.shimmer.color_cycle[0];

        Particle {
            position: (x, y),
            velocity: (0.0, 0.0),
            color: parse_color(color_name),
            character: '✦',
            lifetime: 4.0,
            max_lifetime: 4.0,
            size: 1.0,
            brightness: 1.0,
            effect_type: ParticleType::Shimmer,
        }
    }

    /// Get all particles for rendering
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// Set maximum number of particles
    pub fn set_max_particles(&mut self, max: usize) {
        self.max_particles = max;
    }
}

/// Parse color string to ratatui Color
fn parse_color(color_str: &str) -> Color {
    match color_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => Color::White,
    }
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            sparkles: SparkleConfig {
                enabled: true,
                spawn_rate: 2.0,
                lifetime: 1.0,
                colors: vec![
                    "White".to_string(),
                    "Yellow".to_string(),
                    "Cyan".to_string(),
                ],
                characters: vec!['*', '✦', '✧', '◆'],
                intensity: 1.0,
            },
            glow: GlowConfig {
                enabled: true,
                pulse_speed: 0.1,
                min_brightness: 0.3,
                max_brightness: 1.0,
                colors: vec!["Yellow".to_string(), "White".to_string()],
            },
            float: FloatConfig {
                enabled: true,
                velocity_range: (0.5, 2.0),
                lifetime: 3.0,
                gravity: 0.1,
                characters: vec!['°', '·', '˚'],
            },
            drift: DriftConfig {
                enabled: true,
                wind_strength: 0.5,
                turbulence: 0.1,
                fade_rate: 0.2,
            },
            pulse: PulseConfig {
                enabled: true,
                frequency: 0.2,
                amplitude: 0.3,
                wave_type: "sine".to_string(),
            },
            shimmer: ShimmerConfig {
                enabled: true,
                speed: 0.05,
                color_cycle: vec![
                    "Cyan".to_string(),
                    "LightCyan".to_string(),
                    "White".to_string(),
                ],
                intensity_variation: 0.2,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let config = ParticleConfig::default();
        let mut system = ParticleSystem::new(config);

        system.add_particle(10.0, 10.0, ParticleType::Sparkle);
        assert_eq!(system.particles().len(), 1);

        let particle = &system.particles()[0];
        assert_eq!(particle.position, (10.0, 10.0));
        assert_eq!(particle.effect_type, ParticleType::Sparkle);
    }

    #[test]
    fn test_particle_update() {
        let config = ParticleConfig::default();
        let mut system = ParticleSystem::new(config);

        system.add_particle(10.0, 10.0, ParticleType::Float);
        let initial_lifetime = system.particles()[0].lifetime;

        system.update(0.1);

        let particle = &system.particles()[0];
        assert!(particle.lifetime < initial_lifetime);
        assert_ne!(particle.position, (10.0, 10.0)); // Should have moved
    }

    #[test]
    fn test_particle_removal() {
        let mut config = ParticleConfig::default();
        config.sparkles.enabled = false; // Disable automatic spawning
        let mut system = ParticleSystem::new(config);

        system.add_particle(10.0, 10.0, ParticleType::Sparkle);
        assert_eq!(system.particles().len(), 1);

        // Update many times to exceed lifetime
        for _ in 0..20 {
            // 2 seconds should be enough for 1.0s lifetime
            system.update(0.1);
        }

        assert_eq!(system.particles().len(), 0); // Should be removed
    }
}
