use crate::game::map::{Map, Tile};
use std::collections::HashSet;

/// Field of View calculation using shadow casting algorithm
/// Based on Ruggrogue's implementation with diamond-shaped walls
#[derive(Debug, Clone)]
pub struct FieldOfView {
    pub visible_tiles: HashSet<(i32, i32)>,
    pub range: i32,
    pub dirty: bool,
}

impl Default for FieldOfView {
    fn default() -> Self {
        Self::new(super::constants::FOV_RANGE)
    }
}

impl FieldOfView {
    pub fn new(range: i32) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            range,
            dirty: true,
        }
    }

    pub fn is_visible(&self, pos: (i32, i32)) -> bool {
        self.visible_tiles.contains(&pos)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn calculate(&mut self, map: &Map, start_pos: (i32, i32)) {
        if !self.dirty {
            return;
        }

        self.visible_tiles.clear();
        
        // Always see the starting position
        self.visible_tiles.insert(start_pos);

        // Calculate FOV for all 8 octants
        for octant in 0..8 {
            self.calculate_octant(map, start_pos, octant);
        }

        self.dirty = false;
    }

    fn calculate_octant(&mut self, map: &Map, start: (i32, i32), octant: usize) {
        let mut sights_even = Vec::new();
        let mut sights_odd = vec![Sight::new(Slope::new(0, 1), Slope::new(1, 1))];

        for column in 1..=self.range {
            let (current_sights, next_sights) = if column % 2 == 0 {
                (&mut sights_odd, &mut sights_even)
            } else {
                (&mut sights_even, &mut sights_odd)
            };

            next_sights.clear();

            for sight in current_sights.iter() {
                self.process_sight(map, start, octant, column, sight, next_sights);
            }
        }
    }

    fn process_sight(
        &mut self,
        map: &Map,
        start: (i32, i32),
        octant: usize,
        column: i32,
        sight: &Sight,
        next_sights: &mut Vec<Sight>,
    ) {
        let (low_y, high_y) = self.calculate_y_range(column, sight);
        let mut working_slope: Option<Slope> = None;

        for y in low_y..=high_y {
            let (real_x, real_y) = self.octant_to_real(start, octant, column, y);
            
            // Check if tile is in bounds and within range
            if !map.is_valid_position(real_x, real_y) {
                continue;
            }

            let distance_sq = (real_x - start.0).pow(2) + (real_y - start.1).pow(2);
            if distance_sq > (self.range * self.range) {
                continue;
            }

            let low_mid_slope = Slope::new(2 * y - 1, 2 * column);
            let is_wall = matches!(map.get_tile(real_x, real_y), Tile::Wall { .. });

            // Handle sight transitions
            if is_wall && working_slope.is_some() {
                // End of floor run - add sight
                next_sights.push(Sight::new(working_slope.unwrap(), low_mid_slope));
                working_slope = None;
            } else if !is_wall && working_slope.is_none() {
                // Start of floor run
                working_slope = Some(low_mid_slope.max(sight.low));
            }

            // Mark tile as visible
            let tile_center_slope = Slope::new(2 * y, 2 * column);
            let is_symmetric = sight.low <= tile_center_slope && tile_center_slope <= sight.high;
            
            if is_symmetric || is_wall {
                self.visible_tiles.insert((real_x, real_y));
            }
        }

        // Handle final sight if we ended on floors
        if let Some(slope) = working_slope {
            next_sights.push(Sight::new(slope, sight.high));
        }
    }

    fn calculate_y_range(&self, column: i32, sight: &Sight) -> (i32, i32) {
        let low_y = (2 * column * sight.low.rise / sight.low.run + 1) / 2;
        let high_y = (2 * column * sight.high.rise / sight.high.run + 1) / 2;
        (low_y, high_y)
    }

    fn octant_to_real(&self, start: (i32, i32), octant: usize, x: i32, y: i32) -> (i32, i32) {
        let (rx_from_x, rx_from_y, ry_from_x, ry_from_y) = match octant {
            0 => (1, 0, 0, 1),
            1 => (0, 1, 1, 0),
            2 => (0, -1, 1, 0),
            3 => (-1, 0, 0, 1),
            4 => (-1, 0, 0, -1),
            5 => (0, -1, -1, 0),
            6 => (0, 1, -1, 0),
            7 => (1, 0, 0, -1),
            _ => panic!("Invalid octant: {}", octant),
        };

        (
            start.0 + x * rx_from_x + y * rx_from_y,
            start.1 + x * ry_from_x + y * ry_from_y,
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct Sight {
    low: Slope,
    high: Slope,
}

impl Sight {
    fn new(low: Slope, high: Slope) -> Self {
        Self { low, high }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slope {
    rise: i32,
    run: i32,
}

impl Slope {
    fn new(rise: i32, run: i32) -> Self {
        Self { rise, run }
    }

    fn max(self, other: Self) -> Self {
        if self <= other { other } else { self }
    }
}

impl PartialOrd for Slope {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Slope {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.rise * other.run).cmp(&(other.rise * self.run))
    }
}
