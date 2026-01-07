use crate::game::map::{Map, Tile};
use crate::game::constants::{MAP_WIDTH, MAP_HEIGHT};
use rand_chacha::ChaCha8Rng;
use rand::Rng;

/// Simple trait for layered generation algorithms
pub trait LayerAlgorithm {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Map;
}

/// Percolation algorithm for natural cave networks
pub struct PercolationAlgorithm {
    pub probability: f64,
    pub iterations: usize,
    pub connectivity_threshold: usize,
}

impl PercolationAlgorithm {
    pub fn new() -> Self {
        Self {
            probability: 0.45,
            iterations: 3,
            connectivity_threshold: 4,
        }
    }

    pub fn with_params(probability: f64, iterations: usize, connectivity_threshold: usize) -> Self {
        Self {
            probability,
            iterations,
            connectivity_threshold,
        }
    }
}

impl LayerAlgorithm for PercolationAlgorithm {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Map {
        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        
        // Initialize with random floors based on probability
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let idx = y * MAP_WIDTH + x;
                if idx < map.tiles.len() {
                    if rng.gen_range(0.0..1.0) < self.probability {
                        map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                    } else {
                        map.tiles[idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                    }
                }
            }
        }

        // Apply connectivity-based smoothing iterations
        for _ in 0..self.iterations {
            let mut new_tiles = map.tiles.clone();
            
            for y in 1..MAP_HEIGHT - 1 {
                for x in 1..MAP_WIDTH - 1 {
                    let idx = y * MAP_WIDTH + x;
                    if idx >= new_tiles.len() { continue; }
                    
                    let floor_neighbors = self.count_floor_neighbors(&map, x as i32, y as i32);
                    
                    // Keep floors if they have enough floor neighbors
                    if matches!(map.tiles[idx], Tile::Floor { .. }) {
                        if floor_neighbors < self.connectivity_threshold {
                            new_tiles[idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                        }
                    } else {
                        // Convert walls to floors if surrounded by floors
                        if floor_neighbors >= 6 {
                            new_tiles[idx] = Tile::Floor { id: "stone".to_string() };
                        }
                    }
                }
            }
            
            map.tiles = new_tiles;
        }

        // Ensure borders are walls
        for x in 0..MAP_WIDTH {
            if x < map.tiles.len() {
                map.tiles[x] = Tile::Wall { id: "stone".to_string(), hp: 100 };
            }
            let bottom_idx = (MAP_HEIGHT - 1) * MAP_WIDTH + x;
            if bottom_idx < map.tiles.len() {
                map.tiles[bottom_idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
            }
        }
        
        for y in 0..MAP_HEIGHT {
            let left_idx = y * MAP_WIDTH;
            if left_idx < map.tiles.len() {
                map.tiles[left_idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
            }
            let right_idx = y * MAP_WIDTH + (MAP_WIDTH - 1);
            if right_idx < map.tiles.len() {
                map.tiles[right_idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
            }
        }

        map
    }
}

impl PercolationAlgorithm {
    fn count_floor_neighbors(&self, map: &Map, x: i32, y: i32) -> usize {
        let mut count = 0;
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                
                let nx = x + dx;
                let ny = y + dy;
                
                if nx >= 0 && nx < MAP_WIDTH as i32 && ny >= 0 && ny < MAP_HEIGHT as i32 {
                    let idx = (ny as usize) * MAP_WIDTH + (nx as usize);
                    if idx < map.tiles.len() && matches!(map.tiles[idx], Tile::Floor { .. }) {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
}
