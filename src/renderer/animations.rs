use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAnimationConfig {
    pub blink: BlinkConfig,
    pub glow: GlowConfig,
    pub screen_shake: ScreenShakeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlinkConfig {
    pub duration_ms: u64,
    pub on_color: String,
    pub off_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlowConfig {
    pub cycle_duration_ms: u64,
    pub base_color: String,
    pub glow_color: String,
    pub intensity_steps: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenShakeConfig {
    pub duration_ms: u64,
    pub intensity: i16,
    pub frequency_hz: f32,
}

impl Default for VisualAnimationConfig {
    fn default() -> Self {
        Self {
            blink: BlinkConfig {
                duration_ms: 500,
                on_color: "White".to_string(),
                off_color: "DarkGray".to_string(),
            },
            glow: GlowConfig {
                cycle_duration_ms: 1000,
                base_color: "Yellow".to_string(),
                glow_color: "LightYellow".to_string(),
                intensity_steps: 5,
            },
            screen_shake: ScreenShakeConfig {
                duration_ms: 200,
                intensity: 2,
                frequency_hz: 20.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    Blink { config: BlinkConfig },
    Glow { config: GlowConfig },
    ScreenShake { config: ScreenShakeConfig },
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub animation_type: AnimationType,
    pub start_time: Instant,
    pub duration: Duration,
    pub active: bool,
}

impl Animation {
    pub fn new_blink(config: BlinkConfig) -> Self {
        let duration = Duration::from_millis(config.duration_ms);
        Self {
            animation_type: AnimationType::Blink { config },
            start_time: Instant::now(),
            duration,
            active: true,
        }
    }

    pub fn new_glow(config: GlowConfig) -> Self {
        let duration = Duration::from_millis(config.cycle_duration_ms);
        Self {
            animation_type: AnimationType::Glow { config },
            start_time: Instant::now(),
            duration,
            active: true,
        }
    }

    pub fn new_screen_shake(config: ScreenShakeConfig) -> Self {
        let duration = Duration::from_millis(config.duration_ms);
        Self {
            animation_type: AnimationType::ScreenShake { config },
            start_time: Instant::now(),
            duration,
            active: true,
        }
    }

    pub fn update(&mut self) -> bool {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            match &self.animation_type {
                AnimationType::Glow { .. } => {
                    // Glow animations loop
                    self.start_time = Instant::now();
                }
                _ => {
                    self.active = false;
                }
            }
        }
        self.active
    }

    pub fn get_style(&self, base_style: Style) -> Style {
        if !self.active {
            return base_style;
        }

        let elapsed = self.start_time.elapsed();
        let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();

        match &self.animation_type {
            AnimationType::Blink { config } => {
                let blink_on = (progress * 2.0) as u32 % 2 == 0;
                let color = if blink_on {
                    parse_color(&config.on_color)
                } else {
                    parse_color(&config.off_color)
                };
                base_style.fg(color)
            }
            AnimationType::Glow { config } => {
                let cycle_progress = (progress * config.intensity_steps as f32) % config.intensity_steps as f32;
                let intensity = (cycle_progress.sin().abs() * 255.0) as u8;
                let base_color = parse_color(&config.base_color);
                let glow_color = parse_color(&config.glow_color);
                
                // Blend colors based on intensity
                let blended_color = blend_colors(base_color, glow_color, intensity);
                base_style.fg(blended_color)
            }
            AnimationType::ScreenShake { .. } => {
                // Screen shake doesn't affect style, only viewport offset
                base_style
            }
        }
    }

    pub fn get_screen_offset(&self) -> (i16, i16) {
        if !self.active {
            return (0, 0);
        }

        match &self.animation_type {
            AnimationType::ScreenShake { config } => {
                let elapsed = self.start_time.elapsed();
                let progress = elapsed.as_secs_f32();
                let frequency = config.frequency_hz * 2.0 * std::f32::consts::PI;
                
                let x_offset = (progress * frequency).sin() * config.intensity as f32;
                let y_offset = (progress * frequency * 1.3).cos() * config.intensity as f32;
                
                (x_offset as i16, y_offset as i16)
            }
            _ => (0, 0),
        }
    }
}

pub struct AnimationSystem {
    pub animations: Vec<Animation>,
    pub config: VisualAnimationConfig,
}

impl AnimationSystem {
    pub fn new(config: VisualAnimationConfig) -> Self {
        Self {
            animations: Vec::new(),
            config,
        }
    }

    pub fn add_blink(&mut self) {
        let animation = Animation::new_blink(self.config.blink.clone());
        self.animations.push(animation);
    }

    pub fn add_glow(&mut self) {
        let animation = Animation::new_glow(self.config.glow.clone());
        self.animations.push(animation);
    }

    pub fn add_screen_shake(&mut self) {
        let animation = Animation::new_screen_shake(self.config.screen_shake.clone());
        self.animations.push(animation);
    }

    pub fn update(&mut self) {
        self.animations.retain_mut(|animation| animation.update());
    }

    pub fn get_combined_style(&self, base_style: Style) -> Style {
        self.animations.iter().fold(base_style, |style, animation| {
            animation.get_style(style)
        })
    }

    pub fn get_screen_offset(&self) -> (i16, i16) {
        self.animations.iter().fold((0, 0), |(x, y), animation| {
            let (dx, dy) = animation.get_screen_offset();
            (x + dx, y + dy)
        })
    }
}

fn parse_color(color_str: &str) -> Color {
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

fn blend_colors(base: Color, glow: Color, intensity: u8) -> Color {
    // Simple color blending - in a real implementation you might want more sophisticated blending
    if intensity > 128 {
        glow
    } else {
        base
    }
}
