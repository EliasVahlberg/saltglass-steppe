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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StormEditType {
    Glass,   // Convert walls to glass (current behavior)
    Rotate,  // Rotate map sections 90 degrees
    Swap,    // Swap terrain types in areas
}

impl StormEditType {
    pub fn display_name(&self) -> &'static str {
        match self {
            StormEditType::Glass => "GLASS",
            StormEditType::Rotate => "ROTATE", 
            StormEditType::Swap => "SWAP",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Storm { 
    pub turns_until: u32, 
    pub intensity: u8,
    pub edit_types: Vec<StormEditType>,
}

impl Storm {
    pub fn forecast(rng: &mut ChaCha8Rng) -> Self {
        let cfg = &*STORM_CONFIG;
        let intensity = cfg.base_intensity + rng.gen_range(0..=cfg.intensity_variance);
        
        // Generate 1-3 edit types based on intensity
        let num_edits = (intensity / 2).max(1).min(3) as usize;
        let mut edit_types = Vec::new();
        
        let all_types = [StormEditType::Glass, StormEditType::Rotate, StormEditType::Swap];
        for _ in 0..num_edits {
            let edit_type = all_types[rng.gen_range(0..all_types.len())].clone();
            if !edit_types.iter().any(|t| matches!((t, &edit_type), 
                (StormEditType::Glass, StormEditType::Glass) |
                (StormEditType::Rotate, StormEditType::Rotate) |
                (StormEditType::Swap, StormEditType::Swap))) {
                edit_types.push(edit_type);
            }
        }
        
        Self {
            turns_until: rng.gen_range(cfg.min_interval..=cfg.max_interval),
            intensity,
            edit_types,
        }
    }
    
    pub fn tick(&mut self) -> bool {
        if self.turns_until > 0 { 
            self.turns_until -= 1; 
            self.turns_until == 0 
        } else { 
            false 
        }
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
