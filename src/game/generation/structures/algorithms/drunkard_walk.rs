use crate::game::generation::structures::Rectangle;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrunkardWalkParams {
    pub num_walkers: u32,
    pub steps_per_walker: u32,
    pub change_direction_chance: f32,
    pub spawn_new_walker_chance: f32,
    pub max_walkers: u32,
    pub carve_radius: u32,
}

impl Default for DrunkardWalkParams {
    fn default() -> Self {
        Self {
            num_walkers: 2,
            steps_per_walker: 100,
            change_direction_chance: 0.15,
            spawn_new_walker_chance: 0.02,
            max_walkers: 4,
            carve_radius: 0, // Only carve single tiles
        }
    }
}

#[derive(Debug, Clone)]
struct Walker {
    x: u32,
    y: u32,
    direction: (i32, i32),
    steps_remaining: u32,
}

pub struct DrunkardWalkAlgorithm {
    params: DrunkardWalkParams,
}

impl DrunkardWalkAlgorithm {
    pub fn new(params: DrunkardWalkParams) -> Self {
        Self { params }
    }

    pub fn generate(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> Vec<(u32, u32)> {
        let mut carved_tiles = Vec::new();
        let mut walkers = Vec::new();
        let mut total_iterations = 0;
        const MAX_ITERATIONS: u32 = 10000; // Prevent infinite loops

        // Initialize walkers
        for _ in 0..self.params.num_walkers {
            walkers.push(Walker {
                x: bounds.x + rng.gen_range(1..bounds.width - 1),
                y: bounds.y + rng.gen_range(1..bounds.height - 1),
                direction: self.random_direction(rng),
                steps_remaining: self.params.steps_per_walker,
            });
        }

        while !walkers.is_empty() && total_iterations < MAX_ITERATIONS {
            total_iterations += 1;
            let mut new_walkers = Vec::new();
            let walker_count = walkers.len();

            for i in 0..walker_count {
                let walker = &mut walkers[i];

                // Carve current position
                self.carve_area(walker.x, walker.y, &mut carved_tiles, &bounds);

                // Move walker
                let new_x = (walker.x as i32 + walker.direction.0).max(0) as u32;
                let new_y = (walker.y as i32 + walker.direction.1).max(0) as u32;

                walker.x = new_x.min(bounds.x + bounds.width - 2).max(bounds.x + 1);
                walker.y = new_y.min(bounds.y + bounds.height - 2).max(bounds.y + 1);

                // Change direction randomly
                let random_val = rng.r#gen::<f32>();
                if random_val < self.params.change_direction_chance {
                    walker.direction = self.random_direction(rng);
                }

                // Spawn new walker (with stricter limits)
                let spawn_val = rng.r#gen::<f32>();
                if spawn_val < self.params.spawn_new_walker_chance
                    && walker_count + new_walkers.len() < self.params.max_walkers as usize
                    && total_iterations < MAX_ITERATIONS / 2
                {
                    // Stop spawning halfway through
                    new_walkers.push(Walker {
                        x: walker.x,
                        y: walker.y,
                        direction: self.random_direction(rng),
                        steps_remaining: self.params.steps_per_walker,
                    });
                }

                walker.steps_remaining -= 1;
            }

            // Remove exhausted walkers
            walkers.retain(|w| w.steps_remaining > 0);
            walkers.extend(new_walkers);
        }

        carved_tiles
    }

    fn random_direction(&self, rng: &mut ChaCha8Rng) -> (i32, i32) {
        let direction_choice = rng.gen_range(0..4);
        match direction_choice {
            0 => (0, -1), // North
            1 => (1, 0),  // East
            2 => (0, 1),  // South
            _ => (-1, 0), // West
        }
    }

    fn carve_area(&self, x: u32, y: u32, carved_tiles: &mut Vec<(u32, u32)>, bounds: &Rectangle) {
        let radius = self.params.carve_radius;
        if radius == 0 {
            // Only carve single tile
            if x >= bounds.x
                && x < bounds.x + bounds.width
                && y >= bounds.y
                && y < bounds.y + bounds.height
            {
                carved_tiles.push((x, y));
            }
        } else {
            // Carve area around position
            for dy in 0..=radius * 2 {
                for dx in 0..=radius * 2 {
                    let nx = x.saturating_sub(radius).saturating_add(dx);
                    let ny = y.saturating_sub(radius).saturating_add(dy);

                    if nx >= bounds.x
                        && nx < bounds.x + bounds.width
                        && ny >= bounds.y
                        && ny < bounds.y + bounds.height
                    {
                        carved_tiles.push((nx, ny));
                    }
                }
            }
        }
    }
}
