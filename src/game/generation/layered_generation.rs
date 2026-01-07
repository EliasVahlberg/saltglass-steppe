use crate::game::map::{Map, Tile};
use crate::game::constants::{MAP_WIDTH, MAP_HEIGHT};
use rand_chacha::ChaCha8Rng;
use rand::{SeedableRng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Replace,
    Additive,
    Subtractive,
    Intersection,
    Erosion,
    Multiply,
    Screen,
    Overlay,
    Difference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub algorithm: String,
    pub algorithm_params: serde_json::Value,
    pub weight: f64,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredGenerationConfig {
    pub layers: Vec<LayerConfig>,
    pub blend_mode: BlendMode,
}

pub struct LayeredGenerator {
    config: LayeredGenerationConfig,
}

impl LayeredGenerator {
    pub fn new(config: LayeredGenerationConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, seed: u64) -> Map {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut result_map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        
        // Initialize with walls
        for tile in result_map.tiles.iter_mut() {
            *tile = Tile::Wall { id: "stone".to_string(), hp: 100 };
        }

        for layer in &self.config.layers {
            let layer_map = self.generate_layer(&layer, &mut rng);
            self.blend_layer(&mut result_map, &layer_map, &layer.blend_mode, &mut rng);
        }

        result_map
    }

    fn generate_layer(&self, layer: &LayerConfig, rng: &mut ChaCha8Rng) -> Map {
        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        
        // Initialize with walls
        for tile in map.tiles.iter_mut() {
            *tile = Tile::Wall { id: "stone".to_string(), hp: 100 };
        }
        
        match layer.algorithm.as_str() {
            "cellular_automata" => {
                let wall_prob = layer.algorithm_params.get("initial_wall_probability")
                    .and_then(|v| v.as_f64()).unwrap_or(0.45);
                
                // Initial random placement
                for y in 1..(MAP_HEIGHT - 1) {
                    for x in 1..(MAP_WIDTH - 1) {
                        let idx = y * MAP_WIDTH + x;
                        if idx < map.tiles.len() && rng.gen_range(0.0..1.0) > wall_prob {
                            map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                        }
                    }
                }
            },
            "simple_rooms" => {
                let room_count = layer.algorithm_params.get("room_count")
                    .and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                
                for _ in 0..room_count {
                    let width = rng.gen_range(6..=12);
                    let height = rng.gen_range(6..=12);
                    let x = rng.gen_range(1..(MAP_WIDTH - width - 1));
                    let y = rng.gen_range(1..(MAP_HEIGHT - height - 1));
                    
                    // Create room
                    for ry in y..(y + height) {
                        for rx in x..(x + width) {
                            let idx = ry * MAP_WIDTH + rx;
                            if idx < map.tiles.len() {
                                map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                            }
                        }
                    }
                }
            },
            "drunkard_walk" => {
                let walker_count = layer.algorithm_params.get("walker_count")
                    .and_then(|v| v.as_u64()).unwrap_or(3) as usize;
                let max_steps = layer.algorithm_params.get("max_steps")
                    .and_then(|v| v.as_u64()).unwrap_or(500) as usize;
                
                for _ in 0..walker_count {
                    let mut x = rng.gen_range(1..(MAP_WIDTH - 1));
                    let mut y = rng.gen_range(1..(MAP_HEIGHT - 1));
                    
                    for _ in 0..max_steps {
                        let idx = y * MAP_WIDTH + x;
                        if idx < map.tiles.len() {
                            map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                        }
                        
                        // Random walk step
                        match rng.gen_range(0..4) {
                            0 if x > 1 => x -= 1,
                            1 if x < MAP_WIDTH - 2 => x += 1,
                            2 if y > 1 => y -= 1,
                            3 if y < MAP_HEIGHT - 2 => y += 1,
                            _ => {}
                        }
                    }
                }
            },
            "smooth" => {
                // Smoothing algorithm - reduces noise and jagged edges
                let iterations = layer.algorithm_params.get("iterations")
                    .and_then(|v| v.as_u64()).unwrap_or(2) as usize;
                let threshold = layer.algorithm_params.get("threshold")
                    .and_then(|v| v.as_f64()).unwrap_or(4.0);
                
                // Start with current map state or random
                for y in 1..(MAP_HEIGHT - 1) {
                    for x in 1..(MAP_WIDTH - 1) {
                        let idx = y * MAP_WIDTH + x;
                        if idx < map.tiles.len() && rng.gen_range(0.0..1.0) > 0.5 {
                            map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                        }
                    }
                }
                
                // Apply smoothing iterations
                for _ in 0..iterations {
                    let mut new_tiles = map.tiles.clone();
                    for y in 1..(MAP_HEIGHT - 1) {
                        for x in 1..(MAP_WIDTH - 1) {
                            let idx = y * MAP_WIDTH + x;
                            if idx >= map.tiles.len() { continue; }
                            
                            // Count neighboring floors
                            let mut floor_count = 0;
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    let nx = (x as i32 + dx) as usize;
                                    let ny = (y as i32 + dy) as usize;
                                    let nidx = ny * MAP_WIDTH + nx;
                                    if nidx < map.tiles.len() && matches!(map.tiles[nidx], Tile::Floor { .. }) {
                                        floor_count += 1;
                                    }
                                }
                            }
                            
                            // Apply smoothing rule
                            if floor_count as f64 >= threshold {
                                new_tiles[idx] = Tile::Floor { id: "stone".to_string() };
                            } else {
                                new_tiles[idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                            }
                        }
                    }
                    map.tiles = new_tiles;
                }
            },
            "connect" => {
                // Connectivity enhancer - adds corridors between isolated areas
                let corridor_width = layer.algorithm_params.get("corridor_width")
                    .and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                let max_corridors = layer.algorithm_params.get("max_corridors")
                    .and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                
                // Find isolated floor regions and connect them
                for _ in 0..max_corridors {
                    let start_x = rng.gen_range(1..(MAP_WIDTH - 1));
                    let start_y = rng.gen_range(1..(MAP_HEIGHT - 1));
                    let end_x = rng.gen_range(1..(MAP_WIDTH - 1));
                    let end_y = rng.gen_range(1..(MAP_HEIGHT - 1));
                    
                    // Create L-shaped corridor
                    let mut x = start_x;
                    let mut y = start_y;
                    
                    // Horizontal segment
                    while x != end_x {
                        for w in 0..corridor_width {
                            let cy = y + w;
                            if cy < MAP_HEIGHT {
                                let idx = cy * MAP_WIDTH + x;
                                if idx < map.tiles.len() {
                                    map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                                }
                            }
                        }
                        x = if x < end_x { x + 1 } else { x - 1 };
                    }
                    
                    // Vertical segment
                    while y != end_y {
                        for w in 0..corridor_width {
                            let cx = x + w;
                            if cx < MAP_WIDTH {
                                let idx = y * MAP_WIDTH + cx;
                                if idx < map.tiles.len() {
                                    map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                                }
                            }
                        }
                        y = if y < end_y { y + 1 } else { y - 1 };
                    }
                }
            },
            _ => {
                // Default: random floors
                for _ in 0..500 {
                    let x = rng.gen_range(1..(MAP_WIDTH - 1));
                    let y = rng.gen_range(1..(MAP_HEIGHT - 1));
                    let idx = y * MAP_WIDTH + x;
                    if idx < map.tiles.len() {
                        map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                    }
                }
            }
        }
        
        map
    }

    fn blend_layer(&self, base: &mut Map, layer: &Map, blend_mode: &BlendMode, rng: &mut ChaCha8Rng) {
        for (i, layer_tile) in layer.tiles.iter().enumerate() {
            if i >= base.tiles.len() { break; }
            
            let is_layer_floor = matches!(layer_tile, Tile::Floor { .. });
            let is_base_floor = matches!(base.tiles[i], Tile::Floor { .. });
            
            match blend_mode {
                BlendMode::Replace => {
                    base.tiles[i] = layer_tile.clone();
                },
                BlendMode::Additive => {
                    if is_layer_floor {
                        base.tiles[i] = layer_tile.clone();
                    }
                },
                BlendMode::Subtractive => {
                    if is_layer_floor && is_base_floor {
                        base.tiles[i] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                    }
                },
                BlendMode::Intersection => {
                    if is_layer_floor && is_base_floor {
                        base.tiles[i] = layer_tile.clone();
                    } else {
                        base.tiles[i] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                    }
                },
                BlendMode::Erosion => {
                    if is_layer_floor && matches!(base.tiles[i], Tile::Wall { .. }) {
                        let random_val = rng.gen_range(0.0..1.0);
                        if random_val < 0.3 {
                            base.tiles[i] = Tile::Floor { id: "stone".to_string() };
                        }
                    }
                },
                BlendMode::Multiply => {
                    // Multiply: Both must be floor to result in floor
                    if is_layer_floor && is_base_floor {
                        base.tiles[i] = layer_tile.clone();
                    } else {
                        base.tiles[i] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                    }
                },
                BlendMode::Screen => {
                    // Screen: Either can be floor to result in floor
                    if is_layer_floor || is_base_floor {
                        base.tiles[i] = Tile::Floor { id: "stone".to_string() };
                    }
                },
                BlendMode::Overlay => {
                    // Overlay: Layer takes precedence with probability based on base
                    if is_layer_floor {
                        let blend_strength = if is_base_floor { 0.8 } else { 0.4 };
                        if rng.gen_range(0.0..1.0) < blend_strength {
                            base.tiles[i] = layer_tile.clone();
                        }
                    }
                },
                BlendMode::Difference => {
                    // Difference: Floor only if exactly one is floor
                    if is_layer_floor != is_base_floor {
                        base.tiles[i] = Tile::Floor { id: "stone".to_string() };
                    } else {
                        base.tiles[i] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                    }
                },
            }
        }
    }
}
