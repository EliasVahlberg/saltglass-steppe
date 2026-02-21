//! Overworld travel encounter system.

use std::collections::HashMap;
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EncounterConfig {
    base_encounter_rate: f32,
    danger_scaling: f32,
    min_rate: f32,
    max_rate: f32,
    cooldown_turns: u64,
    flee_base_cooldown: u64,
    flee_distance_required: i32,
    flee_base_chance: f32,
    encounter_types: EncounterTypesConfig,
    biome_modifiers: HashMap<String, BiomeModifier>,
}

#[derive(Deserialize)]
struct EncounterTypesConfig {
    hostile: HostileConfig,
    neutral: NeutralConfig,
    beneficial: BeneficialConfig,
}

#[derive(Deserialize)]
struct HostileConfig {
    weight: u32,
    threat_range: [u32; 2],
}

#[derive(Deserialize)]
struct NeutralConfig {
    weight: u32,
    events: Vec<NeutralEvent>,
}

#[derive(Deserialize)]
struct NeutralEvent {
    id: String,
    weight: u32,
    description: String,
}

#[derive(Deserialize)]
struct BeneficialConfig {
    weight: u32,
    boon_range: [u32; 2],
}

#[derive(Clone, Deserialize)]
struct BiomeModifier {
    danger_mult: f32,
    richness_mult: f32,
}

static CONFIG: Lazy<EncounterConfig> = Lazy::new(|| {
    let data = include_str!("../../data/encounter_config.json");
    serde_json::from_str(data).expect("Failed to parse encounter_config.json")
});

#[derive(Clone, Serialize, Deserialize)]
pub struct EncounterState {
    pub encounter_type: EncounterType,
    pub world_x: usize,
    pub world_y: usize,
    pub spawned_enemies: Vec<usize>,
    pub spawned_items: Vec<usize>,
    pub turns_in_encounter: u32,
    pub last_flee_attempt: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EncounterType {
    Hostile { threat_points: u32 },
    Neutral { event_id: String, description: String },
    Beneficial { boon_points: u32 },
}

impl EncounterState {
    pub fn can_flee(&self, current_turn: u32, difficulty_modifier: f32) -> bool {
        let cooldown = (CONFIG.flee_base_cooldown as f32 * difficulty_modifier) as u32;
        current_turn >= self.last_flee_attempt + cooldown
    }

    pub fn is_complete(&self, enemies: &[super::Enemy]) -> bool {
        match self.encounter_type {
            EncounterType::Hostile { .. } => {
                // Complete when all spawned enemies are dead
                self.spawned_enemies.iter().all(|&idx| {
                    enemies.get(idx).map(|e| e.hp <= 0).unwrap_or(true)
                })
            }
            EncounterType::Neutral { .. } | EncounterType::Beneficial { .. } => {
                // Complete after interacting or leaving map edge
                false // Handled by movement system
            }
        }
    }
}

/// Check if an encounter should trigger for this travel
pub fn should_trigger_encounter(
    world_seed: u64,
    world_x: usize,
    world_y: usize,
    total_tiles_traveled: u64,
    tile_danger: u32,
    last_encounter_turn: u32,
    current_turn: u32,
) -> bool {
    // Check cooldown
    if current_turn - last_encounter_turn < CONFIG.cooldown_turns as u32 {
        return false;
    }

    // Deterministic RNG based on world position and travel count
    let encounter_seed = world_seed
        .wrapping_add((world_x as u64) * 1000)
        .wrapping_add((world_y as u64) * 1000000)
        .wrapping_add(total_tiles_traveled);
    let mut rng = ChaCha8Rng::seed_from_u64(encounter_seed);
    let e = rng.gen_range(0.0..1.0);

    // Calculate encounter probability
    let danger_mod = 1.0 + (tile_danger as f32 * CONFIG.danger_scaling);
    let p = (CONFIG.base_encounter_rate * danger_mod).clamp(CONFIG.min_rate, CONFIG.max_rate);

    e < p
}

/// Generate an encounter for the given tile
pub fn generate_encounter(
    world_seed: u64,
    world_x: usize,
    world_y: usize,
    total_tiles_traveled: u64,
    tile_danger: u32,
    biome: &str,
) -> EncounterState {
    let encounter_seed = world_seed
        .wrapping_add((world_x as u64) * 1000)
        .wrapping_add((world_y as u64) * 1000000)
        .wrapping_add(total_tiles_traveled);
    let mut rng = ChaCha8Rng::seed_from_u64(encounter_seed);

    // Get biome modifiers
    let biome_mod = CONFIG.biome_modifiers.get(biome).cloned().unwrap_or(BiomeModifier {
        danger_mult: 1.0,
        richness_mult: 1.0,
    });

    // Roll encounter type
    let total_weight = CONFIG.encounter_types.hostile.weight
        + CONFIG.encounter_types.neutral.weight
        + CONFIG.encounter_types.beneficial.weight;
    let roll = rng.gen_range(0..total_weight);

    let encounter_type = if roll < CONFIG.encounter_types.hostile.weight {
        // Hostile encounter
        let base_threat = rng.gen_range(
            CONFIG.encounter_types.hostile.threat_range[0]
                ..=CONFIG.encounter_types.hostile.threat_range[1],
        );
        let threat_points = (base_threat as f32 * biome_mod.danger_mult * (tile_danger as f32 / 5.0)) as u32;
        EncounterType::Hostile { threat_points }
    } else if roll < CONFIG.encounter_types.hostile.weight + CONFIG.encounter_types.neutral.weight {
        // Neutral encounter
        let total_event_weight: u32 = CONFIG.encounter_types.neutral.events.iter().map(|e| e.weight).sum();
        let event_roll = rng.gen_range(0..total_event_weight);
        let mut cumulative = 0;
        let event = CONFIG.encounter_types.neutral.events.iter().find(|e| {
            cumulative += e.weight;
            event_roll < cumulative
        }).unwrap();
        
        EncounterType::Neutral {
            event_id: event.id.clone(),
            description: event.description.clone(),
        }
    } else {
        // Beneficial encounter
        let base_boon = rng.gen_range(
            CONFIG.encounter_types.beneficial.boon_range[0]
                ..=CONFIG.encounter_types.beneficial.boon_range[1],
        );
        let boon_points = (base_boon as f32 * biome_mod.richness_mult) as u32;
        EncounterType::Beneficial { boon_points }
    };

    EncounterState {
        encounter_type,
        world_x,
        world_y,
        spawned_enemies: Vec::new(),
        spawned_items: Vec::new(),
        turns_in_encounter: 0,
        last_flee_attempt: 0,
    }
}

/// Attempt to flee from an encounter
pub fn attempt_flee(
    player_x: i32,
    player_y: i32,
    enemies: &[super::Enemy],
    spawned_enemy_indices: &[usize],
    rng: &mut ChaCha8Rng,
) -> Result<(), String> {
    // Check distance from all spawned enemies
    for &idx in spawned_enemy_indices {
        if let Some(enemy) = enemies.get(idx) {
            if enemy.hp > 0 {
                let dist = (enemy.x - player_x).abs() + (enemy.y - player_y).abs();
                if dist <= CONFIG.flee_distance_required {
                    return Err(format!(
                        "Too close to enemies! Must be >{} cells away.",
                        CONFIG.flee_distance_required
                    ));
                }
            }
        }
    }

    // Roll for success
    let success_chance = CONFIG.flee_base_chance; // TODO: Add skill/gear modifiers in Task 1
    let roll = rng.gen_range(0.0..1.0);
    if roll < success_chance {
        Ok(())
    } else {
        Err("Failed to flee!".to_string())
    }
}

pub fn flee_distance_required() -> i32 {
    CONFIG.flee_distance_required
}
