use crate::game::map::{Map, Tile};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Analyze connectivity between floor tiles
pub fn analyze_connectivity(map: &Map) -> ConnectivityAnalysis {
    let floor_positions = get_floor_positions(map);
    let regions = find_connected_regions(map, &floor_positions);
    let largest_region_size = regions.iter().map(|r| r.len()).max().unwrap_or(0);
    let total_floor_tiles = floor_positions.len();
    
    ConnectivityAnalysis {
        total_floor_tiles,
        connected_regions: regions.len(),
        largest_region_size,
        connectivity_ratio: if total_floor_tiles > 0 {
            largest_region_size as f32 / total_floor_tiles as f32
        } else { 0.0 },
        isolated_regions: regions.iter().filter(|r| r.len() == 1).count(),
    }
}

/// Find all connected floor regions using flood fill
pub fn find_connected_regions(map: &Map, floor_positions: &HashSet<(usize, usize)>) -> Vec<HashSet<(usize, usize)>> {
    let mut visited = HashSet::new();
    let mut regions = Vec::new();
    
    for &pos in floor_positions {
        if !visited.contains(&pos) {
            let region = flood_fill_region(map, pos, &mut visited);
            if !region.is_empty() {
                regions.push(region);
            }
        }
    }
    
    regions
}

/// Get all floor tile positions
pub fn get_floor_positions(map: &Map) -> HashSet<(usize, usize)> {
    let mut positions = HashSet::new();
    for y in 0..map.height as usize {
        for x in 0..map.width as usize {
            if is_walkable_tile(&map.tiles[y * map.width as usize + x]) {
                positions.insert((x, y));
            }
        }
    }
    positions
}

/// Flood fill to find connected region
fn flood_fill_region(map: &Map, start: (usize, usize), visited: &mut HashSet<(usize, usize)>) -> HashSet<(usize, usize)> {
    let mut region = HashSet::new();
    let mut stack = vec![start];
    
    while let Some(pos) = stack.pop() {
        if visited.contains(&pos) || !is_in_bounds(map, pos) || !is_walkable_tile(&map.tiles[pos.1 * map.width as usize + pos.0]) {
            continue;
        }
        
        visited.insert(pos);
        region.insert(pos);
        
        // Add neighbors
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            if let (Some(nx), Some(ny)) = (pos.0.checked_add_signed(dx), pos.1.checked_add_signed(dy)) {
                stack.push((nx, ny));
            }
        }
    }
    
    region
}

fn is_in_bounds(map: &Map, pos: (usize, usize)) -> bool {
    pos.0 < map.width as usize && pos.1 < map.height as usize
}

/// Check if tile is walkable
fn is_walkable_tile(tile: &Tile) -> bool {
    matches!(tile, Tile::Floor { .. } | Tile::Glass)
}

/// Analyze tile distribution
pub fn analyze_tile_distribution(map: &Map) -> TileDistribution {
    let mut counts = HashMap::new();
    let total = map.tiles.len();
    
    for tile in &map.tiles {
        let tile_type = tile_to_string(tile);
        *counts.entry(tile_type).or_insert(0) += 1;
    }
    
    TileDistribution { counts, total }
}

fn tile_to_string(tile: &Tile) -> String {
    match tile {
        Tile::Wall { .. } => "wall".to_string(),
        Tile::Floor { .. } => "floor".to_string(),
        Tile::Glass => "glass".to_string(),
        _ => "other".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityAnalysis {
    pub total_floor_tiles: usize,
    pub connected_regions: usize,
    pub largest_region_size: usize,
    pub connectivity_ratio: f32,
    pub isolated_regions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileDistribution {
    pub counts: HashMap<String, usize>,
    pub total: usize,
}
