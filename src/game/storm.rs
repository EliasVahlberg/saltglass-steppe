use once_cell::sync::Lazy;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct StormConfigFile {
    min_interval: u32,
    max_interval: u32,
    base_intensity: u8,
    intensity_variance: u8,
    pub glass_conversion_chance: f32,
    pub storm_glass_drop_chance: f32,
    pub wraith_spawn_max: usize,
    pub refraction_multiplier: u32,
}

static STORM_CONFIG: Lazy<StormConfigFile> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../data/storm_config.json"))
        .expect("Failed to parse storm_config.json")
});

#[derive(Serialize, Deserialize)]
pub struct Storm { pub turns_until: u32, pub intensity: u8 }

impl Storm {
    pub fn forecast(rng: &mut ChaCha8Rng) -> Self {
        let cfg = &*STORM_CONFIG;
        Self {
            turns_until: rng.gen_range(cfg.min_interval..=cfg.max_interval),
            intensity: cfg.base_intensity + rng.gen_range(0..=cfg.intensity_variance),
        }
    }
    pub fn tick(&mut self) -> bool {
        if self.turns_until > 0 { self.turns_until -= 1; self.turns_until == 0 } else { false }
    }
}

pub fn glass_conversion_chance() -> f32 {
    STORM_CONFIG.glass_conversion_chance
}

pub fn storm_glass_drop_chance() -> f32 {
    STORM_CONFIG.storm_glass_drop_chance
}

pub fn wraith_spawn_max() -> usize {
    STORM_CONFIG.wraith_spawn_max
}

pub fn refraction_multiplier() -> u32 {
    STORM_CONFIG.refraction_multiplier
}
