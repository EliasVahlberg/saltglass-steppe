use serde::{Deserialize, Serialize};
use crate::renderer::{
    particles::ParticleConfig,
    animations::VisualAnimationConfig,
    themes::ThemeConfig,
    procedural::ProceduralConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsConfig {
    pub particles: ParticleConfig,
    pub animations: VisualAnimationConfig,
    pub themes: ThemeConfig,
    pub procedural: ProceduralConfig,
    pub global: GlobalEffectsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEffectsConfig {
    pub enabled: bool,
    pub performance_mode: PerformanceMode,
    pub quality_level: QualityLevel,
    pub max_effects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMode {
    Low,
    Balanced,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityLevel {
    Minimal,
    Standard,
    Enhanced,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            particles: ParticleConfig::default(),
            animations: VisualAnimationConfig::default(),
            themes: ThemeConfig::default(),
            procedural: ProceduralConfig::default(),
            global: GlobalEffectsConfig::default(),
        }
    }
}

impl Default for GlobalEffectsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            performance_mode: PerformanceMode::Balanced,
            quality_level: QualityLevel::Standard,
            max_effects: 100,
        }
    }
}

pub struct EffectsManager {
    config: EffectsConfig,
}

impl EffectsManager {
    pub fn new() -> Self {
        Self {
            config: EffectsConfig::default(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: EffectsConfig = serde_json::from_str(&content)?;
        Ok(Self { config })
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_config(&self) -> &EffectsConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut EffectsConfig {
        &mut self.config
    }

    pub fn set_performance_mode(&mut self, mode: PerformanceMode) {
        self.config.global.performance_mode = mode;
        self.apply_performance_settings();
    }

    pub fn set_quality_level(&mut self, level: QualityLevel) {
        self.config.global.quality_level = level;
        self.apply_quality_settings();
    }

    fn apply_performance_settings(&mut self) {
        match self.config.global.performance_mode {
            PerformanceMode::Low => {
                self.config.particles.sparkles.intensity = 0.5;
                self.config.procedural.weather.dust.intensity = 0.05;
                self.config.global.max_effects = 50;
            }
            PerformanceMode::Balanced => {
                self.config.particles.sparkles.intensity = 1.0;
                self.config.procedural.weather.dust.intensity = 0.1;
                self.config.global.max_effects = 100;
            }
            PerformanceMode::High => {
                self.config.particles.sparkles.intensity = 1.5;
                self.config.procedural.weather.dust.intensity = 0.2;
                self.config.global.max_effects = 200;
            }
        }
    }

    fn apply_quality_settings(&mut self) {
        match self.config.global.quality_level {
            QualityLevel::Minimal => {
                self.config.particles.sparkles.enabled = false;
                self.config.procedural.atmospheric.heat_shimmer = false;
            }
            QualityLevel::Standard => {
                self.config.particles.sparkles.enabled = true;
                self.config.procedural.atmospheric.heat_shimmer = true;
            }
            QualityLevel::Enhanced => {
                self.config.particles.sparkles.enabled = true;
                self.config.procedural.atmospheric.heat_shimmer = true;
                self.config.procedural.atmospheric.dust_motes = true;
            }
        }
    }
}
