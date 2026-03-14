//! Terrain-aware A* pathfinding for settlement road generation.
//!
//! Costs are data-driven from `terrain_config.json` → `tile_movement_costs`.
//! The pathfinder operates on a flat cost grid built from the map, producing
//! organic paths that prefer cheap terrain (dry_soil) over expensive terrain
//! (glass, sand) and never cross walls.

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::game::map::{Map, Tile};

// ── Data-driven cost config ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TileMovementCosts {
    floors: HashMap<String, f32>,
    #[allow(dead_code)]
    walls: String, // always "impassable"
    glass: f32,
    glare: f32,
    default_floor: f32,
}

#[derive(Debug, Deserialize)]
struct TerrainConfigPartial {
    tile_movement_costs: TileMovementCosts,
}

static MOVEMENT_COSTS: Lazy<TileMovementCosts> = Lazy::new(|| {
    let data = include_str!("../../../../data/terrain_config.json");
    let cfg: TerrainConfigPartial =
        serde_json::from_str(data).expect("Failed to parse tile_movement_costs from terrain_config.json");
    cfg.tile_movement_costs
});

/// Look up the pathfinding cost for a single tile.
fn tile_cost(tile: &Tile) -> f32 {
    let costs = &*MOVEMENT_COSTS;
    match tile {
        Tile::Wall { .. } => f32::INFINITY,
        Tile::Floor { id } => *costs.floors.get(id.as_str()).unwrap_or(&costs.default_floor),
        Tile::Glass => costs.glass,
        Tile::Glare => costs.glare,
        Tile::StairsDown | Tile::StairsUp | Tile::WorldExit => costs.default_floor,
    }
}

// ── Cost grid ────────────────────────────────────────────────────────

/// Build a flat cost grid from a `Map`. Index = `y * width + x`.
pub fn build_cost_grid(map: &Map) -> Vec<f32> {
    map.tiles.iter().map(|t| tile_cost(t)).collect()
}

// ── A* pathfinder ────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
struct Node {
    f: f32,
    pos: usize,
}

impl Eq for Node {}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 4-directional A* on a cost grid. Returns path including start and goal,
/// or `None` if no finite-cost path exists.
pub fn astar_path(
    costs: &[f32],
    w: usize,
    h: usize,
    from: (i32, i32),
    to: (i32, i32),
) -> Option<Vec<(i32, i32)>> {
    let start = from.1 as usize * w + from.0 as usize;
    let goal = to.1 as usize * w + to.0 as usize;
    if start >= w * h || goal >= w * h {
        return None;
    }

    let heuristic = |idx: usize| -> f32 {
        let (x, y) = (idx % w, idx / w);
        (x as f32 - to.0 as f32).abs() + (y as f32 - to.1 as f32).abs()
    };

    let mut g_score = vec![f32::INFINITY; w * h];
    g_score[start] = 0.0;
    let mut came_from = vec![usize::MAX; w * h];
    let mut open = BinaryHeap::new();
    open.push(Node { f: heuristic(start), pos: start });

    const DIRS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    while let Some(Node { pos, .. }) = open.pop() {
        if pos == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut cur = goal;
            while cur != usize::MAX {
                path.push(((cur % w) as i32, (cur / w) as i32));
                cur = came_from[cur];
            }
            path.reverse();
            return Some(path);
        }

        let (cx, cy) = ((pos % w) as i32, (pos / w) as i32);
        for (dx, dy) in DIRS {
            let (nx, ny) = (cx + dx, cy + dy);
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let nidx = ny as usize * w + nx as usize;
            let step_cost = costs[nidx];
            if step_cost.is_infinite() {
                continue;
            }
            let tentative = g_score[pos] + step_cost;
            if tentative < g_score[nidx] {
                g_score[nidx] = tentative;
                came_from[nidx] = pos;
                open.push(Node { f: tentative + heuristic(nidx), pos: nidx });
            }
        }
    }

    None // no path
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a cost grid from a simple char map.
    /// '#' = wall (inf), '.' = floor (1.0), 'g' = glass (8.0), ' ' = floor (1.0)
    fn grid_from_str(s: &str) -> (Vec<f32>, usize, usize) {
        let rows: Vec<&str> = s.lines().filter(|l| !l.is_empty()).collect();
        let h = rows.len();
        let w = rows[0].len();
        let costs: Vec<f32> = rows.iter().flat_map(|row| {
            row.chars().map(|c| match c {
                '#' => f32::INFINITY,
                'g' => 8.0,
                _ => 1.0,
            })
        }).collect();
        (costs, w, h)
    }

    #[test]
    fn test_direct_path() {
        let (costs, w, h) = grid_from_str(
            ".....\n\
             .....\n\
             .....",
        );
        let path = astar_path(&costs, w, h, (0, 1), (4, 1)).unwrap();
        assert_eq!(*path.first().unwrap(), (0, 1));
        assert_eq!(*path.last().unwrap(), (4, 1));
        assert_eq!(path.len(), 5); // straight horizontal
    }

    #[test]
    fn test_wall_avoidance() {
        let (costs, w, h) = grid_from_str(
            "..#..\n\
             ..#..\n\
             .....",
        );
        let path = astar_path(&costs, w, h, (0, 0), (4, 0)).unwrap();
        // Must go around the wall via row 2
        assert!(path.iter().all(|&(x, y)| {
            let idx = y as usize * w + x as usize;
            costs[idx].is_finite()
        }));
        assert_eq!(*path.last().unwrap(), (4, 0));
    }

    #[test]
    fn test_cost_preference() {
        // Two routes: top row has glass (expensive), bottom row is clear
        let (costs, w, h) = grid_from_str(
            ".ggg.\n\
             .....\n\
             .....",
        );
        let path = astar_path(&costs, w, h, (0, 0), (4, 0)).unwrap();
        // Should prefer going through row 1 (cost 1.0 each) over glass (8.0 each)
        let glass_cells: usize = path.iter().filter(|&&(x, y)| {
            let idx = y as usize * w + x as usize;
            costs[idx] > 1.0
        }).count();
        assert_eq!(glass_cells, 0, "Path should avoid expensive glass tiles");
    }

    #[test]
    fn test_no_path() {
        let (costs, w, h) = grid_from_str(
            "..#..\n\
             ..#..\n\
             ..#..",
        );
        assert!(astar_path(&costs, w, h, (0, 0), (4, 0)).is_none());
    }

    #[test]
    fn test_same_start_goal() {
        let (costs, w, h) = grid_from_str("...");
        let path = astar_path(&costs, w, h, (1, 0), (1, 0)).unwrap();
        assert_eq!(path, vec![(1, 0)]);
    }
}
