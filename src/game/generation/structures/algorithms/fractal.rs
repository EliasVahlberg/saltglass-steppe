use crate::game::map::{Map, Tile};
use crate::game::constants::{MAP_WIDTH, MAP_HEIGHT};
use rand_chacha::ChaCha8Rng;
use rand::Rng;

/// Simple trait for layered generation algorithms
pub trait LayerAlgorithm {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Map;
}

/// Fractal subdivision algorithm for complex terrain generation
pub struct FractalAlgorithm {
    pub roughness: f64,
    pub iterations: usize,
    pub threshold: f64,
}

impl FractalAlgorithm {
    pub fn new() -> Self {
        Self {
            roughness: 0.5,
            iterations: 6,
            threshold: 0.4,
        }
    }

    pub fn with_params(roughness: f64, iterations: usize, threshold: f64) -> Self {
        Self {
            roughness,
            iterations,
            threshold,
        }
    }
}

impl LayerAlgorithm for FractalAlgorithm {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Map {
        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        
        // Create height map using diamond-square algorithm
        let size = (MAP_WIDTH.max(MAP_HEIGHT) as f64).log2().ceil() as usize;
        let grid_size = 2_usize.pow(size as u32) + 1;
        let mut heights = vec![vec![0.0; grid_size]; grid_size];
        
        // Initialize corners
        heights[0][0] = rng.gen_range(0.0..1.0);
        heights[0][grid_size - 1] = rng.gen_range(0.0..1.0);
        heights[grid_size - 1][0] = rng.gen_range(0.0..1.0);
        heights[grid_size - 1][grid_size - 1] = rng.gen_range(0.0..1.0);
        
        let mut step_size = grid_size - 1;
        let mut scale = 1.0;
        
        // Diamond-square iterations
        for _ in 0..self.iterations {
            let half_step = step_size / 2;
            
            if half_step == 0 { break; }
            
            // Diamond step
            for y in (half_step..grid_size).step_by(step_size) {
                for x in (half_step..grid_size).step_by(step_size) {
                    let avg = (heights[y - half_step][x - half_step] +
                              heights[y - half_step][x + half_step] +
                              heights[y + half_step][x - half_step] +
                              heights[y + half_step][x + half_step]) / 4.0;
                    
                    let noise = (rng.gen_range(0.0..1.0) - 0.5) * scale;
                    heights[y][x] = avg + noise;
                }
            }
            
            // Square step
            for y in 0..grid_size {
                for x in 0..grid_size {
                    if heights[y][x] != 0.0 { continue; }
                    
                    let mut sum = 0.0;
                    let mut count = 0;
                    
                    // Check four neighbors
                    let neighbors = [
                        (x as i32, y as i32 - half_step as i32),
                        (x as i32, y as i32 + half_step as i32),
                        (x as i32 - half_step as i32, y as i32),
                        (x as i32 + half_step as i32, y as i32),
                    ];
                    
                    for (nx, ny) in neighbors {
                        if nx >= 0 && nx < grid_size as i32 && ny >= 0 && ny < grid_size as i32 {
                            sum += heights[ny as usize][nx as usize];
                            count += 1;
                        }
                    }
                    
                    if count > 0 {
                        let avg = sum / count as f64;
                        let noise = (rng.gen_range(0.0..1.0) - 0.5) * scale;
                        heights[y][x] = avg + noise;
                    }
                }
            }
            
            step_size = half_step;
            scale *= self.roughness;
        }
        
        // Convert height map to tiles
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let idx = y * MAP_WIDTH + x;
                if idx >= map.tiles.len() { continue; }
                
                let grid_x = (x as f64 / MAP_WIDTH as f64 * (grid_size - 1) as f64) as usize;
                let grid_y = (y as f64 / MAP_HEIGHT as f64 * (grid_size - 1) as f64) as usize;
                
                let height = heights[grid_y.min(grid_size - 1)][grid_x.min(grid_size - 1)];
                
                if height > self.threshold {
                    map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                } else {
                    map.tiles[idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
                }
            }
        }
        
        map
    }
}
