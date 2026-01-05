use crate::game::map::Map;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Calculate basic map metrics
pub fn calculate_map_metrics(map: &Map) -> MapMetrics {
    let total_tiles = map.tiles.len();
    let tile_counts = count_all_tiles(map);
    let perimeter = calculate_perimeter(map);
    let openness = calculate_openness_ratio(map);
    let complexity = calculate_complexity_score(map);
    
    MapMetrics {
        width: map.width as i32,
        height: map.height as i32,
        total_tiles,
        tile_counts,
        perimeter,
        openness,
        complexity,
    }
}

/// Count all tile types
fn count_all_tiles(map: &Map) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for tile in &map.tiles {
        let tile_type = match tile {
            crate::game::map::Tile::Wall { .. } => "wall",
            crate::game::map::Tile::Floor { .. } => "floor",
            crate::game::map::Tile::Glass => "glass",
            _ => "other",
        };
        *counts.entry(tile_type.to_string()).or_insert(0) += 1;
    }
    counts
}

/// Calculate map perimeter (wall tiles adjacent to floor)
fn calculate_perimeter(map: &Map) -> usize {
    let mut perimeter = 0;
    for y in 0..map.height as usize {
        for x in 0..map.width as usize {
            let idx = y * map.width as usize + x;
            if matches!(map.tiles[idx], crate::game::map::Tile::Wall { .. }) {
                // Check if adjacent to floor
                for (dx, dy) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                    if let (Some(nx), Some(ny)) = (x.checked_add_signed(dx as isize), y.checked_add_signed(dy as isize)) {
                        if nx < map.width as usize && ny < map.height as usize {
                            let neighbor_idx = ny * map.width as usize + nx;
                            if matches!(map.tiles[neighbor_idx], crate::game::map::Tile::Floor { .. } | crate::game::map::Tile::Glass) {
                                perimeter += 1;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    perimeter
}

/// Calculate openness ratio (floor tiles / total tiles)
fn calculate_openness_ratio(map: &Map) -> f32 {
    let total = map.tiles.len() as f32;
    let open_tiles = count_open_tiles(map) as f32;
    open_tiles / total
}

fn count_open_tiles(map: &Map) -> usize {
    map.tiles.iter().filter(|tile| matches!(tile, crate::game::map::Tile::Floor { .. } | crate::game::map::Tile::Glass)).count()
}

/// Calculate complexity score based on tile transitions
fn calculate_complexity_score(map: &Map) -> f32 {
    let mut transitions = 0;
    let mut total_checks = 0;
    
    for y in 0..map.height as usize {
        for x in 0..map.width as usize {
            let current_idx = y * map.width as usize + x;
            let current = &map.tiles[current_idx];
            
            for (dx, dy) in [(0i32, 1i32), (1, 0)] { // Only check right and down to avoid double counting
                if let (Some(nx), Some(ny)) = (x.checked_add_signed(dx as isize), y.checked_add_signed(dy as isize)) {
                    if nx < map.width as usize && ny < map.height as usize {
                        let neighbor_idx = ny * map.width as usize + nx;
                        let neighbor = &map.tiles[neighbor_idx];
                        if !tiles_match(current, neighbor) {
                            transitions += 1;
                        }
                        total_checks += 1;
                    }
                }
            }
        }
    }
    
    if total_checks > 0 {
        transitions as f32 / total_checks as f32
    } else {
        0.0
    }
}

fn tiles_match(a: &crate::game::map::Tile, b: &crate::game::map::Tile) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// Calculate path density (average path length between random points)
pub fn calculate_path_density(map: &Map, sample_points: usize) -> f32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let floor_positions: Vec<_> = get_floor_positions_vec(map);
    
    if floor_positions.len() < 2 {
        return 0.0;
    }
    
    let mut total_distance = 0.0;
    let mut valid_paths = 0;
    
    for _ in 0..sample_points {
        let start = floor_positions[rng.gen_range(0..floor_positions.len())];
        let end = floor_positions[rng.gen_range(0..floor_positions.len())];
        
        if let Some(distance) = manhattan_distance(start, end) {
            total_distance += distance;
            valid_paths += 1;
        }
    }
    
    if valid_paths > 0 {
        total_distance / valid_paths as f32
    } else {
        0.0
    }
}

fn get_floor_positions_vec(map: &Map) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    for y in 0..map.height as usize {
        for x in 0..map.width as usize {
            let idx = y * map.width as usize + x;
            if matches!(map.tiles[idx], crate::game::map::Tile::Floor { .. } | crate::game::map::Tile::Glass) {
                positions.push((x, y));
            }
        }
    }
    positions
}

fn manhattan_distance(a: (usize, usize), b: (usize, usize)) -> Option<f32> {
    Some(((a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()) as f32)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetrics {
    pub width: i32,
    pub height: i32,
    pub total_tiles: usize,
    pub tile_counts: HashMap<String, usize>,
    pub perimeter: usize,
    pub openness: f32,
    pub complexity: f32,
}
