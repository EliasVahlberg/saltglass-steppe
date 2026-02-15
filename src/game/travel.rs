//! Data-driven overworld travel system.
//!
//! Loads terrain travel costs from `data/travel_config.json` and provides
//! adjacency checks and cost calculation for world map movement.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::Deserialize;

use super::world_map::{Biome, Terrain};

#[derive(Deserialize)]
struct TravelConfig {
    default_cost: u32,
    terrain_costs: HashMap<String, i32>,
    biome_modifiers: HashMap<String, i32>,
}

static CONFIG: Lazy<TravelConfig> = Lazy::new(|| {
    let data = include_str!("../../data/travel_config.json");
    serde_json::from_str(data).expect("Failed to parse travel_config.json")
});

/// Returns true if two world-map positions are adjacent (Manhattan distance ≤ 1, no diagonals).
pub fn is_adjacent(from: (usize, usize), to: (usize, usize)) -> bool {
    let dx = (from.0 as isize - to.0 as isize).unsigned_abs();
    let dy = (from.1 as isize - to.1 as isize).unsigned_abs();
    dx + dy == 1
}

/// Calculate the turn cost to travel into a tile with the given terrain and biome.
/// Cost = max(1, terrain_cost + biome_modifier).
pub fn travel_cost(terrain: Terrain, biome: Biome) -> u32 {
    let terrain_key = format!("{:?}", terrain).to_lowercase();
    let biome_key = biome.as_str();

    let base = CONFIG
        .terrain_costs
        .get(&terrain_key)
        .copied()
        .unwrap_or(CONFIG.default_cost as i32);

    let modifier = CONFIG.biome_modifiers.get(biome_key).copied().unwrap_or(0);

    (base + modifier).max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_cardinal() {
        assert!(is_adjacent((5, 5), (5, 6)));
        assert!(is_adjacent((5, 5), (5, 4)));
        assert!(is_adjacent((5, 5), (6, 5)));
        assert!(is_adjacent((5, 5), (4, 5)));
    }

    #[test]
    fn not_adjacent_diagonal() {
        assert!(!is_adjacent((5, 5), (6, 6)));
    }

    #[test]
    fn not_adjacent_far() {
        assert!(!is_adjacent((0, 0), (3, 0)));
    }

    #[test]
    fn same_tile_not_adjacent() {
        assert!(!is_adjacent((5, 5), (5, 5)));
    }

    #[test]
    fn cost_known_terrain() {
        assert_eq!(travel_cost(Terrain::Canyon, Biome::Desert), 4);
        assert_eq!(travel_cost(Terrain::Flat, Biome::Desert), 2);
    }

    #[test]
    fn cost_biome_modifier() {
        // Oasis has -1 modifier, flat base is 2 → 1
        assert_eq!(travel_cost(Terrain::Flat, Biome::Oasis), 1);
    }
}
