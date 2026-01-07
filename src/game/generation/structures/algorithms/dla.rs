use crate::game::map::{Map, Tile};
use crate::game::constants::{MAP_WIDTH, MAP_HEIGHT};
use rand_chacha::ChaCha8Rng;
use rand::Rng;
use std::collections::HashSet;

/// Simple trait for layered generation algorithms
pub trait LayerAlgorithm {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Map;
}

/// Diffusion-Limited Aggregation algorithm for organic cave-like structures
pub struct DLAAlgorithm {
    pub seed_count: usize,
    pub walker_count: usize,
    pub max_iterations: usize,
}

impl DLAAlgorithm {
    pub fn new() -> Self {
        Self {
            seed_count: 5,
            walker_count: 1000,
            max_iterations: 10000,
        }
    }

    pub fn with_params(seed_count: usize, walker_count: usize, max_iterations: usize) -> Self {
        Self {
            seed_count,
            walker_count,
            max_iterations,
        }
    }
}

impl LayerAlgorithm for DLAAlgorithm {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Map {
        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        
        // Initialize with walls
        for tile in map.tiles.iter_mut() {
            *tile = Tile::Wall { id: "stone".to_string(), hp: 100 };
        }

        let mut aggregated = HashSet::new();
        
        // Place initial seeds
        for _ in 0..self.seed_count {
            let x = rng.gen_range(MAP_WIDTH / 4..3 * MAP_WIDTH / 4);
            let y = rng.gen_range(MAP_HEIGHT / 4..3 * MAP_HEIGHT / 4);
            let idx = y * MAP_WIDTH + x;
            
            if idx < map.tiles.len() {
                map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                aggregated.insert((x, y));
            }
        }

        // Run DLA simulation
        for _ in 0..self.walker_count {
            let mut walker_x = rng.gen_range(1..MAP_WIDTH - 1);
            let mut walker_y = rng.gen_range(1..MAP_HEIGHT - 1);
            
            // Random walk until touching aggregated structure
            for _ in 0..self.max_iterations {
                // Check if adjacent to aggregated structure
                let neighbors = [
                    (walker_x - 1, walker_y),
                    (walker_x + 1, walker_y),
                    (walker_x, walker_y - 1),
                    (walker_x, walker_y + 1),
                ];
                
                let mut adjacent_to_structure = false;
                for (nx, ny) in neighbors {
                    if aggregated.contains(&(nx, ny)) {
                        adjacent_to_structure = true;
                        break;
                    }
                }
                
                if adjacent_to_structure {
                    // Stick to structure
                    let idx = walker_y * MAP_WIDTH + walker_x;
                    if idx < map.tiles.len() {
                        map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                        aggregated.insert((walker_x, walker_y));
                    }
                    break;
                }
                
                // Random walk step
                let direction = rng.gen_range(0..4);
                match direction {
                    0 if walker_x > 1 => walker_x -= 1,
                    1 if walker_x < MAP_WIDTH - 2 => walker_x += 1,
                    2 if walker_y > 1 => walker_y -= 1,
                    3 if walker_y < MAP_HEIGHT - 2 => walker_y += 1,
                    _ => {}
                }
                
                // Boundary check - restart if walker goes too far
                if walker_x <= 0 || walker_x >= MAP_WIDTH - 1 || 
                   walker_y <= 0 || walker_y >= MAP_HEIGHT - 1 {
                    walker_x = rng.gen_range(1..MAP_WIDTH - 1);
                    walker_y = rng.gen_range(1..MAP_HEIGHT - 1);
                }
            }
        }

        map
    }
}
