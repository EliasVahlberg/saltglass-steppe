use super::{WeightedEntry, WeightedTable};
use crate::game::world_map::Biome;
use once_cell::sync::Lazy;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WeightedSpawn {
    pub id: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub room: Option<String>, // "first", "last", "late" (last 2)
    #[serde(default = "default_min_level")]
    pub min_level: u32,
    #[serde(default = "default_max_level")]
    pub max_level: u32,
}

fn default_weight() -> u32 {
    1
}
fn default_min_level() -> u32 {
    1
}
fn default_max_level() -> u32 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpawnTable {
    #[serde(default)]
    pub items: Vec<WeightedSpawn>,
    #[serde(default)]
    pub enemies: Vec<WeightedSpawn>,
    #[serde(default)]
    pub npcs: Vec<WeightedSpawn>,
}

#[derive(Debug, Deserialize)]
pub struct SpawnTables {
    pub default: SpawnTable,
    #[serde(default)]
    pub saltflat: Option<SpawnTable>,
    #[serde(default)]
    pub oasis: Option<SpawnTable>,
    #[serde(default)]
    pub ruins: Option<SpawnTable>,
    #[serde(default)]
    pub scrubland: Option<SpawnTable>,
}

static SPAWN_TABLES: Lazy<SpawnTables> = Lazy::new(|| {
    let data = include_str!("../../../data/biome_spawn_tables.json");
    serde_json::from_str(data).expect("Failed to parse biome_spawn_tables.json")
});

pub fn load_spawn_tables() -> &'static SpawnTables {
    &SPAWN_TABLES
}

pub fn get_biome_spawn_table(biome: &Biome) -> &'static SpawnTable {
    let tables = &SPAWN_TABLES;
    match biome {
        Biome::Saltflat => tables.saltflat.as_ref().unwrap_or(&tables.default),
        Biome::Oasis => tables.oasis.as_ref().unwrap_or(&tables.default),
        Biome::Ruins => tables.ruins.as_ref().unwrap_or(&tables.default),
        Biome::Scrubland => tables.scrubland.as_ref().unwrap_or(&tables.default),
        _ => &tables.default,
    }
}

pub fn weighted_pick<'a>(spawns: &'a [WeightedSpawn], rng: &mut ChaCha8Rng) -> Option<&'a str> {
    let total: u32 = spawns.iter().map(|s| s.weight).sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng.gen_range(0..total);
    for spawn in spawns {
        if roll < spawn.weight {
            return Some(&spawn.id);
        }
        roll -= spawn.weight;
    }
    None
}

pub fn weighted_pick_by_level_and_tier<'a>(
    spawns: &'a [WeightedSpawn],
    level: u32,
    rng: &mut ChaCha8Rng,
    is_item: bool,
) -> Option<&'a str> {
    let valid_spawns: Vec<_> = spawns
        .iter()
        .filter(|s| {
            let level_ok = level >= s.min_level && level <= s.max_level;
            if !is_item {
                return level_ok;
            }

            // For items, use tier-based filtering based on level
            let tier_threshold = match level {
                1 => 1,      // Only tier 1 items
                2..=3 => 2,  // Tier 1-2 items
                4..=6 => 3,  // Tier 1-3 items
                7..=8 => 4,  // Tier 1-4 items
                9..=10 => 5, // All tiers
                _ => 1,
            };

            level_ok && get_item_tier(&s.id).unwrap_or(1) <= tier_threshold
        })
        .collect();

    let total: u32 = valid_spawns.iter().map(|s| s.weight).sum();
    if total == 0 {
        return None;
    }

    let mut roll = rng.gen_range(0..total);
    for spawn in valid_spawns {
        if roll < spawn.weight {
            return Some(&spawn.id);
        }
        roll -= spawn.weight;
    }
    None
}

fn get_item_tier(item_id: &str) -> Option<u32> {
    crate::game::item::get_item_def(item_id).map(|def| def.tier)
}

/// Enhanced weighted selection using WeightedTable system
pub fn weighted_pick_enhanced<'a>(
    spawns: &'a [WeightedSpawn],
    rng: &mut ChaCha8Rng,
) -> Option<&'a str> {
    let entries: Vec<WeightedEntry<&str>> = spawns
        .iter()
        .map(|s| WeightedEntry {
            item: s.id.as_str(),
            weight: s.weight as f32,
        })
        .collect();

    let table = WeightedTable::new(entries);
    table.select(rng)
}

/// Enhanced level and tier-based selection using WeightedTable system
pub fn weighted_pick_by_level_and_tier_enhanced<'a>(
    spawns: &'a [WeightedSpawn],
    level: u32,
    rng: &mut ChaCha8Rng,
    is_item: bool,
) -> Option<&'a str> {
    let valid_spawns: Vec<_> = spawns
        .iter()
        .filter(|s| {
            let level_ok = level >= s.min_level && level <= s.max_level;
            if !is_item {
                return level_ok;
            }

            // For items, use tier-based filtering based on level
            let tier_threshold = match level {
                1 => 1,      // Only tier 1 items
                2..=3 => 2,  // Tier 1-2 items
                4..=6 => 3,  // Tier 1-3 items
                7..=8 => 4,  // Tier 1-4 items
                9..=10 => 5, // All tiers
                _ => 1,
            };

            level_ok && get_item_tier(&s.id).unwrap_or(1) <= tier_threshold
        })
        .collect();

    let entries: Vec<WeightedEntry<&str>> = valid_spawns
        .iter()
        .map(|s| WeightedEntry {
            item: s.id.as_str(),
            weight: s.weight as f32,
        })
        .collect();

    let table = WeightedTable::new(entries);
    table.select(rng)
}
