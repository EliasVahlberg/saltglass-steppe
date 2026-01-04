use crate::game::generation::structures::{Rectangle, StructureGenerator, StructureParams, Structure, POIType, StructureFeature};
use rand_chacha::ChaCha8Rng;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Parameters for cellular automata generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellularAutomataParams {
    /// Initial probability of a cell being alive (wall)
    pub initial_wall_probability: f64,
    /// Number of iterations to run the cellular automata
    pub iterations: u32,
    /// Minimum neighbors required for a cell to survive
    pub survival_threshold: u8,
    /// Minimum neighbors required for a cell to be born
    pub birth_threshold: u8,
    /// Whether to use Moore neighborhood (8 neighbors) or Von Neumann (4 neighbors)
    pub use_moore_neighborhood: bool,
}

impl Default for CellularAutomataParams {
    fn default() -> Self {
        Self {
            initial_wall_probability: 0.45,
            iterations: 5,
            survival_threshold: 4,
            birth_threshold: 5,
            use_moore_neighborhood: true,
        }
    }
}

/// Cellular Automata algorithm for generating organic cave-like structures
#[derive(Debug)]
pub struct CellularAutomataAlgorithm {
    params: CellularAutomataParams,
}

impl CellularAutomataAlgorithm {
    pub fn new(params: CellularAutomataParams) -> Self {
        Self { params }
    }

    /// Generate organic walls using cellular automata
    pub fn generate(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> Vec<(i32, i32)> {
        let width = bounds.width as usize;
        let height = bounds.height as usize;
        
        // Initialize grid with random walls
        let mut grid = vec![vec![false; width]; height];
        for y in 0..height {
            for x in 0..width {
                grid[y][x] = rng.r#gen::<f64>() < self.params.initial_wall_probability;
            }
        }
        
        // Run cellular automata iterations
        for _ in 0..self.params.iterations {
            grid = self.iterate_generation(&grid);
        }
        
        // Convert grid to wall positions
        let mut walls = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if grid[y][x] {
                    walls.push((bounds.x as i32 + x as i32, bounds.y as i32 + y as i32));
                }
            }
        }
        
        walls
    }
    
    /// Run one iteration of cellular automata
    fn iterate_generation(&self, grid: &[Vec<bool>]) -> Vec<Vec<bool>> {
        let height = grid.len();
        let width = grid[0].len();
        let mut new_grid = vec![vec![false; width]; height];
        
        for y in 0..height {
            for x in 0..width {
                let neighbor_count = self.count_neighbors(grid, x, y);
                let is_alive = grid[y][x];
                
                new_grid[y][x] = if is_alive {
                    // Survival rule
                    neighbor_count >= self.params.survival_threshold
                } else {
                    // Birth rule
                    neighbor_count >= self.params.birth_threshold
                };
            }
        }
        
        new_grid
    }
    
    /// Count living neighbors for a cell
    fn count_neighbors(&self, grid: &[Vec<bool>], x: usize, y: usize) -> u8 {
        let height = grid.len() as i32;
        let width = grid[0].len() as i32;
        let mut count = 0;
        
        let offsets = if self.params.use_moore_neighborhood {
            // Moore neighborhood (8 neighbors)
            vec![
                (-1, -1), (0, -1), (1, -1),
                (-1,  0),          (1,  0),
                (-1,  1), (0,  1), (1,  1),
            ]
        } else {
            // Von Neumann neighborhood (4 neighbors)
            vec![(0, -1), (-1, 0), (1, 0), (0, 1)]
        };
        
        for (dx, dy) in offsets {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            
            if nx >= 0 && nx < width && ny >= 0 && ny < height {
                if grid[ny as usize][nx as usize] {
                    count += 1;
                }
            } else {
                // Treat out-of-bounds as walls
                count += 1;
            }
        }
        
        count
    }
}

impl StructureGenerator for CellularAutomataAlgorithm {
    fn generate(&self, params: &StructureParams, rng: &mut ChaCha8Rng) -> Option<Structure> {
        let bounds = Rectangle::new(0, 0, params.size.0, params.size.1);
        let walls = self.generate(bounds, rng);
        
        // Convert walls to features
        let features = walls.into_iter().map(|(x, y)| {
            StructureFeature {
                feature_type: "wall".to_string(),
                position: (x as u32, y as u32),
                properties: std::collections::HashMap::new(),
            }
        }).collect();
        
        let bounds_result = Rectangle::new(0, 0, params.size.0, params.size.1);
        
        Some(Structure {
            structure_type: params.structure_type.clone(),
            bounds: bounds_result,
            rooms: Vec::new(),
            corridors: Vec::new(),
            features,
            spawn_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        })
    }
    
    fn get_supported_poi_types(&self) -> Vec<POIType> {
        vec![POIType::Dungeon, POIType::Landmark]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_cellular_automata_generation() {
        let params = CellularAutomataParams::default();
        let algorithm = CellularAutomataAlgorithm::new(params);
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        let bounds = Rectangle::new(0, 0, 20, 15);
        let walls = algorithm.generate(bounds, &mut rng);
        
        // Should generate some walls
        assert!(!walls.is_empty());
        
        // All walls should be within bounds (create new bounds for comparison)
        let bounds_check = Rectangle::new(0, 0, 20, 15);
        for (x, y) in &walls {
            assert!(*x >= bounds_check.x as i32);
            assert!(*x < (bounds_check.x + bounds_check.width) as i32);
            assert!(*y >= bounds_check.y as i32);
            assert!(*y < (bounds_check.y + bounds_check.height) as i32);
        }
        
        // Should be deterministic
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        let bounds2 = Rectangle::new(0, 0, 20, 15);
        let walls2 = algorithm.generate(bounds2, &mut rng2);
        assert_eq!(walls, walls2);
    }
    
    #[test]
    fn test_neighbor_counting() {
        let params = CellularAutomataParams::default();
        let algorithm = CellularAutomataAlgorithm::new(params);
        
        // Create a test grid with known pattern
        let grid = vec![
            vec![true,  false, true ],
            vec![false, true,  false],
            vec![true,  false, true ],
        ];
        
        // Center cell should have 4 neighbors (Moore neighborhood)
        let count = algorithm.count_neighbors(&grid, 1, 1);
        assert_eq!(count, 4);
        
        // Corner cell should have 1 neighbor + 2 out-of-bounds = 3
        let count = algorithm.count_neighbors(&grid, 0, 0);
        assert_eq!(count, 3); // 1 actual neighbor + 2 out-of-bounds treated as walls
    }
    
    #[test]
    fn test_von_neumann_neighborhood() {
        let mut params = CellularAutomataParams::default();
        params.use_moore_neighborhood = false;
        let algorithm = CellularAutomataAlgorithm::new(params);
        
        let grid = vec![
            vec![true,  false, true ],
            vec![false, true,  false],
            vec![true,  false, true ],
        ];
        
        // Center cell should have 0 neighbors (Von Neumann neighborhood)
        let count = algorithm.count_neighbors(&grid, 1, 1);
        assert_eq!(count, 0);
    }
}
