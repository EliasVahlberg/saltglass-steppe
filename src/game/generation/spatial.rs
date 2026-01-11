use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Poisson disk sampling for spatial distribution
pub struct PoissonSampler {
    min_distance: f32,
    width: usize,
    height: usize,
    cell_size: f32,
    grid_width: usize,
    grid_height: usize,
    grid: Vec<Option<(i32, i32)>>,
}

impl PoissonSampler {
    pub fn new(width: usize, height: usize, min_distance: f32) -> Self {
        let cell_size = min_distance / 2.0_f32.sqrt();
        let grid_width = (width as f32 / cell_size).ceil() as usize;
        let grid_height = (height as f32 / cell_size).ceil() as usize;

        Self {
            min_distance,
            width,
            height,
            cell_size,
            grid_width,
            grid_height,
            grid: vec![None; grid_width * grid_height],
        }
    }

    pub fn sample_points(
        &mut self,
        candidates: &[(i32, i32)],
        max_points: usize,
        rng: &mut ChaCha8Rng,
    ) -> Vec<(i32, i32)> {
        self.grid.fill(None);
        let mut result = Vec::new();
        let mut active = Vec::new();

        if candidates.is_empty() {
            return result;
        }

        // Start with random candidate
        let first_idx = rng.gen_range(0..candidates.len());
        let first_point = candidates[first_idx];
        result.push(first_point);
        active.push(first_point);
        self.insert_point(first_point);

        while !active.is_empty() && result.len() < max_points {
            let active_idx = rng.gen_range(0..active.len());
            let point = active[active_idx];

            let mut found = false;
            for _ in 0..30 {
                // Try 30 times to find valid point
                if let Some(new_point) = self.find_valid_candidate(candidates, point, rng) {
                    if self.is_valid_point(new_point) {
                        result.push(new_point);
                        active.push(new_point);
                        self.insert_point(new_point);
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                active.swap_remove(active_idx);
            }
        }

        result
    }

    fn find_valid_candidate(
        &self,
        candidates: &[(i32, i32)],
        center: (i32, i32),
        rng: &mut ChaCha8Rng,
    ) -> Option<(i32, i32)> {
        // Find candidates within reasonable distance
        let search_radius = self.min_distance * 3.0;
        let valid_candidates: Vec<_> = candidates
            .iter()
            .filter(|&&(x, y)| {
                let dx = (x - center.0) as f32;
                let dy = (y - center.1) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                dist >= self.min_distance && dist <= search_radius
            })
            .collect();

        if valid_candidates.is_empty() {
            return None;
        }

        let idx = rng.gen_range(0..valid_candidates.len());
        Some(*valid_candidates[idx])
    }

    fn is_valid_point(&self, point: (i32, i32)) -> bool {
        let (x, y) = point;
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }

        let grid_x = (x as f32 / self.cell_size) as usize;
        let grid_y = (y as f32 / self.cell_size) as usize;

        // Check surrounding cells
        for dy in -2..=2 {
            for dx in -2..=2 {
                let check_x = grid_x as i32 + dx;
                let check_y = grid_y as i32 + dy;

                if check_x >= 0
                    && check_y >= 0
                    && check_x < self.grid_width as i32
                    && check_y < self.grid_height as i32
                {
                    let idx = check_y as usize * self.grid_width + check_x as usize;
                    if let Some(existing) = self.grid[idx] {
                        let dist_sq = ((x - existing.0).pow(2) + (y - existing.1).pow(2)) as f32;
                        if dist_sq < self.min_distance * self.min_distance {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    fn insert_point(&mut self, point: (i32, i32)) {
        let grid_x = (point.0 as f32 / self.cell_size) as usize;
        let grid_y = (point.1 as f32 / self.cell_size) as usize;
        let idx = grid_y * self.grid_width + grid_x;
        self.grid[idx] = Some(point);
    }
}

/// Simple grid-based distribution as fallback
pub fn distribute_points_grid(
    candidates: &[(i32, i32)],
    max_points: usize,
    min_distance: i32,
    rng: &mut ChaCha8Rng,
) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let mut shuffled = candidates.to_vec();

    // Shuffle candidates
    for i in (1..shuffled.len()).rev() {
        let j = rng.gen_range(0..=i);
        shuffled.swap(i, j);
    }

    for &candidate in &shuffled {
        if result.len() >= max_points {
            break;
        }

        // Check minimum distance to existing points
        let valid = result.iter().all(|&(x, y): &(i32, i32)| {
            let dx = (candidate.0 - x).abs();
            let dy = (candidate.1 - y).abs();
            dx >= min_distance || dy >= min_distance
        });

        if valid {
            result.push(candidate);
        }
    }

    result
}
