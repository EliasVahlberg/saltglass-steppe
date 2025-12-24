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
    Glass,      // Convert walls to glass (current behavior)
    Rotate,     // Rotate map sections 90 degrees
    Swap,       // Swap terrain types in areas
    Mirror,     // Reflect map sections horizontally/vertically
    Fracture,   // Create glass seams/cracks through terrain
    Crystallize,// Convert floor tiles to crystal formations
    Vortex,     // Spiral rearrangement of map sections
}

impl StormEditType {
    pub fn display_name(&self) -> &'static str {
        match self {
            StormEditType::Glass => "GLASS",
            StormEditType::Rotate => "ROTATE", 
            StormEditType::Swap => "SWAP",
            StormEditType::Mirror => "MIRROR",
            StormEditType::Fracture => "FRACTURE",
            StormEditType::Crystallize => "CRYSTAL",
            StormEditType::Vortex => "VORTEX",
        }
    }

    pub fn all_types() -> Vec<StormEditType> {
        vec![
            StormEditType::Glass,
            StormEditType::Rotate,
            StormEditType::Swap,
            StormEditType::Mirror,
            StormEditType::Fracture,
            StormEditType::Crystallize,
            StormEditType::Vortex,
        ]
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
        
        // Generate edit types based on intensity
        let edit_types = Self::generate_edit_types(intensity, rng);
        
        Self {
            turns_until: rng.gen_range(cfg.min_interval..=cfg.max_interval),
            intensity,
            edit_types,
        }
    }

    fn generate_edit_types(intensity: u8, rng: &mut ChaCha8Rng) -> Vec<StormEditType> {
        let all_types = StormEditType::all_types();
        let mut edit_types = Vec::new();
        
        // Number of edit types based on intensity
        let num_types = match intensity {
            1..=2 => 1,                    // Micro-storms: single effect
            3..=5 => rng.gen_range(1..=2), // Normal storms: 1-2 effects
            6..=7 => rng.gen_range(2..=3), // Strong storms: 2-3 effects
            _ => rng.gen_range(3..=4),     // Mega-storms: 3-4 effects
        };
        
        // Select random edit types without duplicates
        let mut available = all_types;
        for _ in 0..num_types.min(available.len()) {
            let idx = rng.gen_range(0..available.len());
            edit_types.push(available.remove(idx));
        }
        
        edit_types
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
