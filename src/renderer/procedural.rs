use bracket_noise::prelude::*;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralConfig {
    pub weather: WeatherConfig,
    pub ambient_lighting: AmbientLightingConfig,
    pub atmospheric: AtmosphericConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub rain: WeatherEffectConfig,
    pub snow: WeatherEffectConfig,
    pub dust: WeatherEffectConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEffectConfig {
    pub enabled: bool,
    pub intensity: f32,
    pub speed: f32,
    pub characters: Vec<String>,
    pub colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientLightingConfig {
    pub enabled: bool,
    pub variation_speed: f32,
    pub variation_intensity: f32,
    pub base_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphericConfig {
    pub heat_shimmer: bool,
    pub dust_motes: bool,
    pub light_rays: bool,
}

impl Default for ProceduralConfig {
    fn default() -> Self {
        Self {
            weather: WeatherConfig {
                rain: WeatherEffectConfig {
                    enabled: false,
                    intensity: 0.3,
                    speed: 2.0,
                    characters: vec!["|".to_string(), "\\".to_string(), "/".to_string()],
                    colors: vec!["Blue".to_string(), "LightBlue".to_string()],
                },
                snow: WeatherEffectConfig {
                    enabled: false,
                    intensity: 0.2,
                    speed: 1.0,
                    characters: vec!["*".to_string(), "·".to_string(), "°".to_string()],
                    colors: vec!["White".to_string(), "LightGray".to_string()],
                },
                dust: WeatherEffectConfig {
                    enabled: true,
                    intensity: 0.1,
                    speed: 0.5,
                    characters: vec!["·".to_string(), "˚".to_string(), "°".to_string()],
                    colors: vec!["DarkGray".to_string(), "Gray".to_string()],
                },
            },
            ambient_lighting: AmbientLightingConfig {
                enabled: true,
                variation_speed: 0.02,
                variation_intensity: 0.15,
                base_level: 20,
            },
            atmospheric: AtmosphericConfig {
                heat_shimmer: true,
                dust_motes: true,
                light_rays: false,
            },
        }
    }
}

#[derive(Debug)]
pub struct WeatherParticle {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub character: char,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct ProceduralEffects {
    config: ProceduralConfig,
    noise: FastNoise,
    weather_particles: Vec<WeatherParticle>,
    start_time: Instant,
    last_spawn: Instant,
}

impl ProceduralEffects {
    pub fn new(config: ProceduralConfig) -> Self {
        let mut noise = FastNoise::seeded(42);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(0.1);
        
        Self {
            config,
            noise,
            weather_particles: Vec::new(),
            start_time: Instant::now(),
            last_spawn: Instant::now(),
        }
    }

    pub fn update(&mut self, delta_time: f32, viewport_width: i32, viewport_height: i32) {
        self.update_weather_particles(delta_time, viewport_width, viewport_height);
        self.spawn_weather_particles(viewport_width, viewport_height);
    }

    fn update_weather_particles(&mut self, delta_time: f32, viewport_width: i32, viewport_height: i32) {
        // Update existing particles
        for particle in &mut self.weather_particles {
            particle.x += particle.velocity_x * delta_time;
            particle.y += particle.velocity_y * delta_time;
            particle.lifetime += delta_time;
        }

        // Remove particles that are out of bounds or expired
        self.weather_particles.retain(|p| {
            p.x >= -5.0 && p.x < viewport_width as f32 + 5.0 &&
            p.y >= -5.0 && p.y < viewport_height as f32 + 5.0 &&
            p.lifetime < p.max_lifetime
        });
    }

    fn spawn_weather_particles(&mut self, viewport_width: i32, viewport_height: i32) {
        let now = Instant::now();
        let spawn_interval = Duration::from_millis(100);
        
        if now.duration_since(self.last_spawn) < spawn_interval {
            return;
        }
        
        self.last_spawn = now;

        // Spawn rain particles
        if self.config.weather.rain.enabled {
            self.spawn_rain_particles(viewport_width, viewport_height);
        }

        // Spawn snow particles
        if self.config.weather.snow.enabled {
            self.spawn_snow_particles(viewport_width, viewport_height);
        }

        // Spawn dust particles
        if self.config.weather.dust.enabled {
            self.spawn_dust_particles(viewport_width, viewport_height);
        }
    }

    fn spawn_rain_particles(&mut self, viewport_width: i32, _viewport_height: i32) {
        let config = &self.config.weather.rain;
        let spawn_count = (config.intensity * 10.0) as usize;
        
        for _ in 0..spawn_count {
            let x = rand::random::<f32>() * (viewport_width as f32 + 10.0) - 5.0;
            let y = -2.0;
            let character = config.characters[rand::random::<usize>() % config.characters.len()].chars().next().unwrap_or('|');
            let color = self.parse_color(&config.colors[rand::random::<usize>() % config.colors.len()]);
            
            self.weather_particles.push(WeatherParticle {
                x,
                y,
                velocity_x: rand::random::<f32>() * 2.0 - 1.0,
                velocity_y: config.speed * (8.0 + rand::random::<f32>() * 4.0),
                character,
                color,
                lifetime: 0.0,
                max_lifetime: 5.0,
            });
        }
    }

    fn spawn_snow_particles(&mut self, viewport_width: i32, _viewport_height: i32) {
        let config = &self.config.weather.snow;
        let spawn_count = (config.intensity * 8.0) as usize;
        
        for _ in 0..spawn_count {
            let x = rand::random::<f32>() * (viewport_width as f32 + 10.0) - 5.0;
            let y = -2.0;
            let character = config.characters[rand::random::<usize>() % config.characters.len()].chars().next().unwrap_or('*');
            let color = self.parse_color(&config.colors[rand::random::<usize>() % config.colors.len()]);
            
            self.weather_particles.push(WeatherParticle {
                x,
                y,
                velocity_x: rand::random::<f32>() * 1.0 - 0.5,
                velocity_y: config.speed * (2.0 + rand::random::<f32>() * 2.0),
                character,
                color,
                lifetime: 0.0,
                max_lifetime: 8.0,
            });
        }
    }

    fn spawn_dust_particles(&mut self, viewport_width: i32, viewport_height: i32) {
        let config = &self.config.weather.dust;
        let spawn_count = (config.intensity * 5.0) as usize;
        
        for _ in 0..spawn_count {
            let x = rand::random::<f32>() * viewport_width as f32;
            let y = rand::random::<f32>() * viewport_height as f32;
            let character = config.characters[rand::random::<usize>() % config.characters.len()].chars().next().unwrap_or('·');
            let color = self.parse_color(&config.colors[rand::random::<usize>() % config.colors.len()]);
            
            // Use noise for more natural movement
            let time = self.start_time.elapsed().as_secs_f32();
            let noise_x = self.noise.get_noise((x as f64 * 0.01) as f32, (time as f64 * 0.1) as f32) as f32;
            let noise_y = self.noise.get_noise((y as f64 * 0.01) as f32, (time as f64 * 0.1 + 100.0) as f32) as f32;
            
            self.weather_particles.push(WeatherParticle {
                x,
                y,
                velocity_x: noise_x * config.speed * 0.5,
                velocity_y: noise_y * config.speed * 0.3,
                character,
                color,
                lifetime: 0.0,
                max_lifetime: 10.0,
            });
        }
    }

    pub fn get_ambient_light_variation(&self) -> f32 {
        if !self.config.ambient_lighting.enabled {
            return 0.0;
        }

        let time = self.start_time.elapsed().as_secs_f32();
        let noise_value = self.noise.get_noise((time as f64 * self.config.ambient_lighting.variation_speed as f64) as f32, 0.0) as f32;
        noise_value * self.config.ambient_lighting.variation_intensity
    }

    pub fn get_weather_particles(&self) -> &[WeatherParticle] {
        &self.weather_particles
    }

    pub fn get_heat_shimmer_offset(&self, x: i32, y: i32) -> (i16, i16) {
        if !self.config.atmospheric.heat_shimmer {
            return (0, 0);
        }

        let time = self.start_time.elapsed().as_secs_f32();
        let noise_x = self.noise.get_noise((x as f64 * 0.1) as f32, (time as f64 * 2.0) as f32) as f32;
        let noise_y = self.noise.get_noise((x as f64 * 0.1 + 100.0) as f32, (y as f64 * 0.1) as f32) as f32;
        
        let offset_x = (noise_x * 0.5).round() as i16;
        let offset_y = (noise_y * 0.3).round() as i16;
        
        (offset_x, offset_y)
    }

    fn parse_color(&self, color_str: &str) -> Color {
        match color_str {
            "Black" => Color::Black,
            "Red" => Color::Red,
            "Green" => Color::Green,
            "Yellow" => Color::Yellow,
            "Blue" => Color::Blue,
            "Magenta" => Color::Magenta,
            "Cyan" => Color::Cyan,
            "Gray" => Color::Gray,
            "DarkGray" => Color::DarkGray,
            "LightRed" => Color::LightRed,
            "LightGreen" => Color::LightGreen,
            "LightYellow" => Color::LightYellow,
            "LightBlue" => Color::LightBlue,
            "LightMagenta" => Color::LightMagenta,
            "LightCyan" => Color::LightCyan,
            "White" => Color::White,
            _ => Color::White,
        }
    }
}
