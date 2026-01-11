use crate::game::map::{Map, Tile};
use crate::tilegen_tools::ConnectivityAnalysis;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

/// Check if map meets connectivity constraints
pub fn check_connectivity_constraint(
    analysis: &ConnectivityAnalysis,
    min_ratio: f32,
) -> ConstraintResult {
    let passed = analysis.connectivity_ratio >= min_ratio;
    ConstraintResult {
        constraint_type: "connectivity".to_string(),
        passed,
        actual_value: analysis.connectivity_ratio,
        expected_value: min_ratio,
        message: if passed {
            format!(
                "Connectivity ratio {:.2} meets minimum {:.2}",
                analysis.connectivity_ratio, min_ratio
            )
        } else {
            format!(
                "Connectivity ratio {:.2} below minimum {:.2}",
                analysis.connectivity_ratio, min_ratio
            )
        },
    }
}

/// Check if map has minimum number of floor tiles
pub fn check_floor_density_constraint(map: &Map, min_density: f32) -> ConstraintResult {
    let total_tiles = map.tiles.len() as f32;
    let floor_count = count_floor_tiles(map) as f32;
    let actual_density = floor_count / total_tiles;
    let passed = actual_density >= min_density;

    ConstraintResult {
        constraint_type: "floor_density".to_string(),
        passed,
        actual_value: actual_density,
        expected_value: min_density,
        message: if passed {
            format!(
                "Floor density {:.2} meets minimum {:.2}",
                actual_density, min_density
            )
        } else {
            format!(
                "Floor density {:.2} below minimum {:.2}",
                actual_density, min_density
            )
        },
    }
}

/// Check if map has reasonable wall-to-floor ratio
pub fn check_wall_ratio_constraint(map: &Map, max_wall_ratio: f32) -> ConstraintResult {
    let total_tiles = map.tiles.len() as f32;
    let wall_count = count_wall_tiles(map) as f32;
    let actual_ratio = wall_count / total_tiles;
    let passed = actual_ratio <= max_wall_ratio;

    ConstraintResult {
        constraint_type: "wall_ratio".to_string(),
        passed,
        actual_value: actual_ratio,
        expected_value: max_wall_ratio,
        message: if passed {
            format!(
                "Wall ratio {:.2} within maximum {:.2}",
                actual_ratio, max_wall_ratio
            )
        } else {
            format!(
                "Wall ratio {:.2} exceeds maximum {:.2}",
                actual_ratio, max_wall_ratio
            )
        },
    }
}

/// Check if a path exists between two points using BFS
pub fn is_path_exists(map: &Map, start: (i32, i32), end: (i32, i32)) -> bool {
    if start == end {
        return true;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == end {
            return true;
        }

        // Check 4-directional neighbors
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && nx < map.width as i32 && ny >= 0 && ny < map.height as i32 {
                let pos = (nx, ny);
                if !visited.contains(&pos) && is_tile_walkable(map, nx, ny) {
                    visited.insert(pos);
                    queue.push_back(pos);
                }
            }
        }
    }

    false
}

/// Calculate Euclidean distance between two points
pub fn calculate_distance(p1: (i32, i32), p2: (i32, i32)) -> f32 {
    let dx = (p1.0 - p2.0) as f32;
    let dy = (p1.1 - p2.1) as f32;
    (dx * dx + dy * dy).sqrt()
}

/// Check if a tile is walkable
pub fn is_tile_walkable(map: &Map, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
        return false;
    }

    if let Some(idx) = map.pos_to_idx(x, y) {
        matches!(map.tiles[idx], Tile::Floor { .. } | Tile::Glass)
    } else {
        false
    }
}

/// Validate all standard constraints
pub fn validate_standard_constraints(
    map: &Map,
    analysis: &ConnectivityAnalysis,
) -> Vec<ConstraintResult> {
    vec![
        check_connectivity_constraint(analysis, 0.8),
        check_floor_density_constraint(map, 0.3),
        check_wall_ratio_constraint(map, 0.6),
    ]
}

fn count_floor_tiles(map: &Map) -> usize {
    map.tiles
        .iter()
        .filter(|tile| matches!(tile, Tile::Floor { .. } | Tile::Glass))
        .count()
}

fn count_wall_tiles(map: &Map) -> usize {
    map.tiles
        .iter()
        .filter(|tile| matches!(tile, Tile::Wall { .. }))
        .count()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    pub constraint_type: String,
    pub passed: bool,
    pub actual_value: f32,
    pub expected_value: f32,
    pub message: String,
}
