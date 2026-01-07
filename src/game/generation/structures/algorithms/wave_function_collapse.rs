use crate::game::generation::structures::Rectangle;
use rand_chacha::ChaCha8Rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveFunctionCollapseParams {
    pub tile_size: u32,
    pub overlap: u32,
    pub max_iterations: u32,
    pub entropy_heuristic: EntropyHeuristic,
    pub pattern_weights: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropyHeuristic {
    MinimumEntropy,
    WeightedRandom,
    CornerFirst,
}

impl Default for WaveFunctionCollapseParams {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("floor".to_string(), 0.6);
        weights.insert("wall".to_string(), 0.3);
        weights.insert("door".to_string(), 0.1);
        
        Self {
            tile_size: 3,
            overlap: 1,
            max_iterations: 1000,
            entropy_heuristic: EntropyHeuristic::MinimumEntropy,
            pattern_weights: weights,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Pattern {
    tiles: Vec<Vec<String>>,
    weight: f32,
}

#[derive(Debug, Clone)]
struct Cell {
    possible_patterns: Vec<usize>,
    collapsed: bool,
}

pub struct WaveFunctionCollapseGenerator {
    params: WaveFunctionCollapseParams,
    patterns: Vec<Pattern>,
}

impl WaveFunctionCollapseGenerator {
    pub fn new(params: WaveFunctionCollapseParams) -> Self {
        let patterns = Self::generate_basic_patterns(&params);
        Self { params, patterns }
    }

    pub fn generate(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> HashMap<String, Vec<(u32, u32)>> {
        let grid_width = (bounds.width / self.params.tile_size) as usize;
        let grid_height = (bounds.height / self.params.tile_size) as usize;
        
        let mut grid = vec![vec![Cell {
            possible_patterns: (0..self.patterns.len()).collect(),
            collapsed: false,
        }; grid_width]; grid_height];

        let mut iterations = 0;
        while iterations < self.params.max_iterations && !self.is_fully_collapsed(&grid) {
            // Find cell with minimum entropy
            if let Some((x, y)) = self.find_minimum_entropy_cell(&grid) {
                // Collapse the cell
                self.collapse_cell(&mut grid, x, y, rng);
                
                // Propagate constraints (simplified)
                self.propagate_constraints(&mut grid, x, y);
            } else {
                break; // No more cells to collapse or contradiction
            }
            
            iterations += 1;
        }

        // Convert grid to world coordinates
        self.grid_to_world_coords(&grid, &bounds)
    }

    fn generate_basic_patterns(params: &WaveFunctionCollapseParams) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        let size = params.tile_size as usize;
        
        // Generate basic patterns
        for (tile_type, &weight) in &params.pattern_weights {
            // Solid pattern
            let solid_pattern = Pattern {
                tiles: vec![vec![tile_type.clone(); size]; size],
                weight,
            };
            patterns.push(solid_pattern);
        }
        
        patterns
    }

    fn is_fully_collapsed(&self, grid: &[Vec<Cell>]) -> bool {
        grid.iter().all(|row| row.iter().all(|cell| cell.collapsed))
    }

    fn find_minimum_entropy_cell(&self, grid: &[Vec<Cell>]) -> Option<(usize, usize)> {
        let mut min_entropy = usize::MAX;
        let mut candidates = Vec::new();
        
        for (y, row) in grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if !cell.collapsed {
                    let entropy = cell.possible_patterns.len();
                    if entropy < min_entropy && entropy > 0 {
                        min_entropy = entropy;
                        candidates.clear();
                        candidates.push((x, y));
                    } else if entropy == min_entropy {
                        candidates.push((x, y));
                    }
                }
            }
        }
        
        if candidates.is_empty() {
            None
        } else {
            Some(candidates[0]) // For determinism, take first candidate
        }
    }

    fn collapse_cell(&self, grid: &mut [Vec<Cell>], x: usize, y: usize, rng: &mut ChaCha8Rng) {
        let cell = &mut grid[y][x];
        if cell.possible_patterns.is_empty() {
            return; // Contradiction - cannot collapse
        }
        
        // Weighted random selection
        let total_weight: f32 = cell.possible_patterns.iter()
            .map(|&i| self.patterns[i].weight)
            .sum();
        
        let random_val = rng.r#gen::<f32>();
        let mut roll = random_val * total_weight;
        let mut selected_pattern = 0;
        
        for &pattern_idx in &cell.possible_patterns {
            roll -= self.patterns[pattern_idx].weight;
            if roll <= 0.0 {
                selected_pattern = pattern_idx;
                break;
            }
        }
        
        cell.possible_patterns = vec![selected_pattern];
        cell.collapsed = true;
    }

    fn propagate_constraints(&self, grid: &mut [Vec<Cell>], start_x: usize, start_y: usize) {
        // Simplified constraint propagation - just ensure neighboring cells are compatible
        let directions = [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)];
        
        for (dx, dy) in directions {
            let nx = start_x as i32 + dx;
            let ny = start_y as i32 + dy;
            
            if nx >= 0 && ny >= 0 && 
               (nx as usize) < grid[0].len() && (ny as usize) < grid.len() {
                let nx = nx as usize;
                let ny = ny as usize;
                
                if !grid[ny][nx].collapsed {
                    // Simplified: just remove incompatible patterns
                    let reference_pattern = grid[start_y][start_x].possible_patterns[0];
                    grid[ny][nx].possible_patterns.retain(|&pattern_idx| {
                        self.patterns_compatible(pattern_idx, reference_pattern)
                    });
                }
            }
        }
    }

    fn patterns_compatible(&self, _pattern1: usize, _pattern2: usize) -> bool {
        // Simplified compatibility - always compatible for now
        true
    }

    fn grid_to_world_coords(&self, grid: &[Vec<Cell>], bounds: &Rectangle) -> HashMap<String, Vec<(u32, u32)>> {
        let mut result: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
        let tile_size = self.params.tile_size;
        
        for (grid_y, row) in grid.iter().enumerate() {
            for (grid_x, cell) in row.iter().enumerate() {
                if cell.collapsed && !cell.possible_patterns.is_empty() {
                    let pattern_idx = cell.possible_patterns[0];
                    let pattern = &self.patterns[pattern_idx];
                    
                    for (py, pattern_row) in pattern.tiles.iter().enumerate() {
                        for (px, tile_type) in pattern_row.iter().enumerate() {
                            let world_x = bounds.x + (grid_x as u32 * tile_size) + px as u32;
                            let world_y = bounds.y + (grid_y as u32 * tile_size) + py as u32;
                            
                            if world_x < bounds.x + bounds.width && world_y < bounds.y + bounds.height {
                                result.entry(tile_type.clone())
                                    .or_insert_with(Vec::new)
                                    .push((world_x, world_y));
                            }
                        }
                    }
                }
            }
        }
        
        result
    }
}
