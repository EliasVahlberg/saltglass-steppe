use crate::game::generation::structures::Rectangle;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeParams {
    pub cell_size: u32,
    pub wall_thickness: u32,
    pub algorithm: MazeAlgorithm,
    pub add_loops: bool,
    pub loop_probability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MazeAlgorithm {
    RecursiveBacktracking,
    Kruskal,
    Prim,
}

impl Default for MazeParams {
    fn default() -> Self {
        Self {
            cell_size: 3,
            wall_thickness: 1,
            algorithm: MazeAlgorithm::RecursiveBacktracking,
            add_loops: true,
            loop_probability: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CellState {
    Wall,
    Floor,
}

pub struct MazeGenerator {
    params: MazeParams,
}

impl MazeGenerator {
    pub fn new(params: MazeParams) -> Self {
        Self { params }
    }

    pub fn generate(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> Vec<(u32, u32)> {
        match self.params.algorithm {
            MazeAlgorithm::RecursiveBacktracking => self.recursive_backtracking(bounds, rng),
            MazeAlgorithm::Kruskal => self.kruskal(bounds, rng),
            MazeAlgorithm::Prim => self.prim(bounds, rng),
        }
    }

    fn recursive_backtracking(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> Vec<(u32, u32)> {
        let cell_size = self.params.cell_size;
        let maze_width = bounds.width / cell_size;
        let maze_height = bounds.height / cell_size;

        let mut grid = vec![vec![CellState::Wall; maze_width as usize]; maze_height as usize];
        let mut stack = Vec::new();

        // Start from random cell
        let start_x = rng.gen_range(0..maze_width);
        let start_y = rng.gen_range(0..maze_height);

        grid[start_y as usize][start_x as usize] = CellState::Floor;
        stack.push((start_x, start_y));

        while let Some((x, y)) = stack.pop() {
            let neighbors = self.get_unvisited_neighbors(x, y, &grid, maze_width, maze_height);

            if !neighbors.is_empty() {
                stack.push((x, y));

                let (nx, ny) = neighbors[rng.gen_range(0..neighbors.len())];

                // Carve path to neighbor
                grid[ny as usize][nx as usize] = CellState::Floor;

                // Carve wall between current cell and neighbor
                let wall_x = (x + nx) / 2;
                let wall_y = (y + ny) / 2;
                if wall_x < maze_width && wall_y < maze_height {
                    grid[wall_y as usize][wall_x as usize] = CellState::Floor;
                }

                stack.push((nx, ny));
            }
        }

        // Add loops if requested
        if self.params.add_loops {
            self.add_random_loops(&mut grid, maze_width, maze_height, rng);
        }

        // Convert grid to world coordinates
        self.grid_to_world_coords(&grid, bounds, cell_size)
    }

    fn kruskal(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> Vec<(u32, u32)> {
        // Simplified implementation - just return recursive backtracking for now
        self.recursive_backtracking(bounds, rng)
    }

    fn prim(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> Vec<(u32, u32)> {
        // Simplified implementation - just return recursive backtracking for now
        self.recursive_backtracking(bounds, rng)
    }

    fn get_unvisited_neighbors(
        &self,
        x: u32,
        y: u32,
        grid: &[Vec<CellState>],
        width: u32,
        height: u32,
    ) -> Vec<(u32, u32)> {
        let mut neighbors = Vec::new();
        let directions = [(0i32, -2i32), (2, 0), (0, 2), (-2, 0)];

        for (dx, dy) in directions {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let nx = nx as u32;
                let ny = ny as u32;
                if grid[ny as usize][nx as usize] == CellState::Wall {
                    neighbors.push((nx, ny));
                }
            }
        }

        neighbors
    }

    fn add_random_loops(
        &self,
        grid: &mut [Vec<CellState>],
        width: u32,
        height: u32,
        rng: &mut ChaCha8Rng,
    ) {
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let random_val = rng.r#gen::<f32>();
                if grid[y as usize][x as usize] == CellState::Wall
                    && random_val < self.params.loop_probability
                {
                    grid[y as usize][x as usize] = CellState::Floor;
                }
            }
        }
    }

    fn grid_to_world_coords(
        &self,
        grid: &[Vec<CellState>],
        bounds: Rectangle,
        cell_size: u32,
    ) -> Vec<(u32, u32)> {
        let mut floor_tiles = Vec::new();

        for (y, row) in grid.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell == CellState::Floor {
                    let world_x = bounds.x + (x as u32 * cell_size);
                    let world_y = bounds.y + (y as u32 * cell_size);

                    // Fill cell area
                    for dy in 0..cell_size {
                        for dx in 0..cell_size {
                            if world_x + dx < bounds.x + bounds.width
                                && world_y + dy < bounds.y + bounds.height
                            {
                                floor_tiles.push((world_x + dx, world_y + dy));
                            }
                        }
                    }
                }
            }
        }

        floor_tiles
    }
}
