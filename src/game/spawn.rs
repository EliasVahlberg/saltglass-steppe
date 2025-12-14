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
}

fn default_weight() -> u32 { 1 }

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
}

static SPAWN_TABLES: Lazy<SpawnTables> = Lazy::new(|| {
    let data = include_str!("../../data/spawn_tables.json");
    serde_json::from_str(data).expect("Failed to parse spawn_tables.json")
});

pub fn load_spawn_tables() -> &'static SpawnTables {
    &SPAWN_TABLES
}

pub fn weighted_pick<'a>(spawns: &'a [WeightedSpawn], rng: &mut ChaCha8Rng) -> Option<&'a str> {
    let total: u32 = spawns.iter().map(|s| s.weight).sum();
    if total == 0 { return None; }
    let mut roll = rng.gen_range(0..total);
    for spawn in spawns {
        if roll < spawn.weight {
            return Some(&spawn.id);
        }
        roll -= spawn.weight;
    }
    None
}
