pub mod layout;
pub mod buildings;
pub mod faction_theme;
pub mod population;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::game::map::{Map, Tile};
use crate::game::generation::structure_library::{StructureLibrary, LegendEntry};

/// Configuration for settlement generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    pub seed: u64,
    pub tier: SettlementTier,
    pub faction_control: Vec<(String, f32)>, // (faction_id, control_percentage)
}

/// Settlement tier determines size and complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementTier {
    Village,
    Town,
    City,
}

/// A placed building in the settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub prefab_name: String,
    pub x: i32,
    pub y: i32,
    pub faction: Option<String>,
    /// Clockwise rotation in degrees (0 / 90 / 180 / 270)
    #[serde(default)]
    pub rotation: u16,
}

/// Generated settlement with all placed buildings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub config: SettlementConfig,
    pub buildings: Vec<Building>,
    pub width: usize,
    pub height: usize,
}

/// Main entry point for settlement generation
pub fn generate_settlement<R: Rng>(config: SettlementConfig, rng: &mut R) -> Settlement {
    let (width, height) = layout::calculate_dimensions(&config);
    let buildings = buildings::place_buildings(&config, width, height, rng);
    
    Settlement {
        config,
        buildings,
        width,
        height,
    }
}

/// Effective (width, height) of a building after rotation (90/270 swap dims).
fn effective_dims(w: i32, h: i32, rotation: u16) -> (i32, i32) {
    if rotation == 90 || rotation == 270 { (h, w) } else { (w, h) }
}

/// Entrance side after applying clockwise rotation.
fn rotated_side(side: &str, rotation: u16) -> &'static str {
    const DIRS: [&str; 4] = ["north", "east", "south", "west"];
    let idx = DIRS.iter().position(|&d| d == side).unwrap_or(2);
    DIRS[(idx + (rotation / 90) as usize) % 4]
}

/// Map pattern coords (px, py) to world offset given rotation and original (w, h).
fn rotate_coords(px: i32, py: i32, w: i32, h: i32, rotation: u16) -> (i32, i32) {
    match rotation {
        90  => (h - 1 - py, px),
        180 => (w - 1 - px, h - 1 - py),
        270 => (py, w - 1 - px),
        _   => (px, py),
    }
}
/// Clear natural walls within DILATION tiles of any building footprint.
/// Uses a per-building distance field so the cleared area follows the actual
/// settlement shape (union of rounded rectangles) rather than a bounding box.
pub fn clear_settlement_footprint(map: &mut Map, settlement: &Settlement) {
    let library = match StructureLibrary::load() {
        Ok(lib) => lib,
        Err(_) => return,
    };

    const DILATION: i32 = 8;
    const DILATION_SQ: i32 = DILATION * DILATION;

    // Collect (x, y, w, h) for each building — using effective (rotated) dimensions
    let footprints: Vec<(i32, i32, i32, i32)> = settlement.buildings.iter().map(|b| {
        let (w, h) = library.get(&b.prefab_name)
            .map(|s| (s.width as i32, s.height as i32))
            .unwrap_or((6, 6));
        let (ew, eh) = effective_dims(w, h, b.rotation);
        (b.x, b.y, ew, eh)
    }).collect();

    if footprints.is_empty() { return; }

    let margin = DILATION + 1;
    let min_x = footprints.iter().map(|&(x, _, _, _)| x).min().unwrap() - margin;
    let min_y = footprints.iter().map(|&(_, y, _, _)| y).min().unwrap() - margin;
    let max_x = footprints.iter().map(|&(x, _, w, _)| x + w).max().unwrap() + margin;
    let max_y = footprints.iter().map(|&(_, y, _, h)| y + h).max().unwrap() + margin;

    for cy in min_y..max_y {
        for cx in min_x..max_x {
            if cx < 0 || cy < 0 || cx >= map.width as i32 || cy >= map.height as i32 { continue; }
            if !matches!(map.get_tile(cx, cy), Tile::Wall { .. }) { continue; }

            // Distance from (cx,cy) to nearest point inside each building rectangle
            let near_enough = footprints.iter().any(|&(bx, by, bw, bh)| {
                let dx = if cx < bx { bx - cx } else if cx >= bx + bw { cx - (bx + bw - 1) } else { 0 };
                let dy = if cy < by { by - cy } else if cy >= by + bh { cy - (by + bh - 1) } else { 0 };
                dx * dx + dy * dy <= DILATION_SQ
            });

            if near_enough {
                map.set_tile(cx as usize, cy as usize, Tile::Floor { id: "dry_soil".to_string() });
            }
        }
    }
}

/// Stamp settlement buildings onto the map
pub fn stamp_settlement(map: &mut Map, settlement: &Settlement) {
    let library = match StructureLibrary::load() {
        Ok(lib) => lib,
        Err(_) => return,
    };

    for building in &settlement.buildings {
        let structure = match library.get(&building.prefab_name) {
            Some(s) => s,
            None => continue,
        };

        // Clear a floor footprint (effective rotated bounds + 1 tile padding) before stamping
        let (ew, eh) = effective_dims(structure.width as i32, structure.height as i32, building.rotation);
        let pad = 1i32;
        for cy in (building.y - pad)..(building.y + eh + pad) {
            for cx in (building.x - pad)..(building.x + ew + pad) {
                if cx >= 0 && cy >= 0 && cx < map.width as i32 && cy < map.height as i32 {
                    if matches!(map.get_tile(cx, cy), Tile::Wall { .. }) {
                        map.set_tile(cx as usize, cy as usize, Tile::Floor { id: "dry_soil".to_string() });
                    }
                }
            }
        }

        for (py, row) in structure.pattern.iter().enumerate() {
            for (px, &ch) in row.iter().enumerate() {
                if ch == ' ' { continue; }
                let (ox, oy) = rotate_coords(px as i32, py as i32, structure.width as i32, structure.height as i32, building.rotation);
                let tile_x = building.x + ox;
                let tile_y = building.y + oy;
                if tile_x < 0 || tile_y < 0 || tile_x >= map.width as i32 || tile_y >= map.height as i32 {
                    continue;
                }
                if let Some(legend_entry) = structure.legend.get(&ch) {
                    let tile = match legend_entry {
                        LegendEntry::Wall { id } => Tile::Wall { id: id.clone(), hp: 100 },
                        LegendEntry::Floor { id } => Tile::Floor { id: id.clone() },
                        LegendEntry::Door => Tile::Floor { id: "wood_floor".to_string() },
                        LegendEntry::Interactable { .. } => Tile::Floor { id: "wood_floor".to_string() },
                        LegendEntry::Npc { .. } => Tile::Floor { id: "wood_floor".to_string() },
                        LegendEntry::Structure { .. } => continue,
                        LegendEntry::Ground => continue,
                        LegendEntry::Path => Tile::Floor { id: "dirt_path".to_string() },
                    };
                    map.set_tile(tile_x as usize, tile_y as usize, tile);
                }
            }
        }
    }
}

/// Paint dirt paths along MST edges between building entrances.
/// The settlement footprint is already cleared, so paths are painted
/// onto dry_soil tiles only — indoor floors are never overwritten.
pub fn paint_roads(map: &mut Map, settlement: &Settlement) {
    let library = match StructureLibrary::load() {
        Ok(lib) => lib,
        Err(_) => return,
    };

    let entrances: Vec<(i32, i32)> = settlement.buildings.iter().map(|b| {
        let structure = library.get(&b.prefab_name);
        let (w, h) = structure
            .map(|s| (s.width as i32, s.height as i32))
            .unwrap_or((6, 6));
        let (ew, eh) = effective_dims(w, h, b.rotation);
        let natural_side = structure
            .and_then(|s| s.metadata.entrance_side.as_deref())
            .unwrap_or("south");
        let side = if natural_side == "any" { "south" } else { rotated_side(natural_side, b.rotation) };
        match side {
            "north" => (b.x + ew / 2, b.y),
            "east"  => (b.x + ew - 1, b.y + eh / 2),
            "west"  => (b.x,          b.y + eh / 2),
            _       => (b.x + ew / 2, b.y + eh - 1), // south
        }
    }).collect();

    let n = entrances.len();
    if n < 2 { return; }

    let mut edges: Vec<(f32, usize, usize)> = (0..n)
        .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
        .map(|(i, j)| {
            let dx = (entrances[i].0 - entrances[j].0) as f32;
            let dy = (entrances[i].1 - entrances[j].1) as f32;
            ((dx * dx + dy * dy).sqrt(), i, j)
        })
        .collect();
    edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut parent: Vec<usize> = (0..n).collect();
    let find = |parent: &mut Vec<usize>, mut x: usize| -> usize {
        while parent[x] != x { x = parent[x]; }
        x
    };

    let mut mst: Vec<(usize, usize)> = Vec::new();
    let mut extras: Vec<(usize, usize)> = Vec::new();
    for (_, i, j) in edges {
        let (pi, pj) = (find(&mut parent, i), find(&mut parent, j));
        if pi != pj { parent[pi] = pj; mst.push((i, j)); }
        else { extras.push((i, j)); }
    }

    let loop_count = (n / 5).min(extras.len());
    for (i, j) in mst.into_iter().chain(extras.into_iter().take(loop_count)) {
        paint_path(map, entrances[i], entrances[j]);
    }
}

fn paint_path(map: &mut Map, from: (i32, i32), to: (i32, i32)) {
    let (mut x, mut y) = from;
    while x != to.0 {
        if x >= 0 && y >= 0 && x < map.width as i32 && y < map.height as i32 {
            if matches!(map.get_tile(x, y), Tile::Floor { id } if id == "dry_soil") {
                map.set_tile(x as usize, y as usize, Tile::Floor { id: "dirt_path".to_string() });
            }
        }
        x += if to.0 > x { 1 } else { -1 };
    }
    while y != to.1 {
        if x >= 0 && y >= 0 && x < map.width as i32 && y < map.height as i32 {
            if matches!(map.get_tile(x, y), Tile::Floor { id } if id == "dry_soil") {
                map.set_tile(x as usize, y as usize, Tile::Floor { id: "dirt_path".to_string() });
            }
        }
        y += if to.1 > y { 1 } else { -1 };
    }
}

/// Place decorative elements in open floor spaces within settlement bounds
pub fn place_decorations<R: Rng>(map: &mut Map, settlement: &Settlement, rng: &mut R) {
    let dominant_faction = faction_theme::get_dominant_faction(&settlement.config);
    
    for y in 0..settlement.height {
        for x in 0..settlement.width {
            let tile = map.get_tile(x as i32, y as i32);
            if matches!(tile, Tile::Floor { id } if id == "dry_soil") && rng.gen_bool(0.08) {
                let decoration_id = match dominant_faction.as_deref() {
                    Some("MirrorMonks") => ["prismatic_tiles", "light_pool", "crystal_moss"][rng.gen_range(0..3)],
                    Some("StormCults") => ["storm_glass_shards", "void_stone", "glass_sand"][rng.gen_range(0..3)],
                    Some("SaltTradingCompany") => ["salt_crust", "salt_gravel", "brine_mud"][rng.gen_range(0..3)],
                    _ => ["ancient_tile", "crushed_saltglass", "soft_sand"][rng.gen_range(0..3)],
                };
                map.set_tile(x, y, Tile::Floor { id: decoration_id.to_string() });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_generate_settlement_village() {
        let config = SettlementConfig {
            seed: 12345,
            tier: SettlementTier::Village,
            faction_control: vec![("MirrorMonks".to_string(), 0.6)],
        };
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let settlement = generate_settlement(config.clone(), &mut rng);
        
        assert_eq!(settlement.width, 80);
        assert_eq!(settlement.height, 60);
        assert_eq!(settlement.config.tier, SettlementTier::Village);
    }

    #[test]
    fn test_generate_settlement_town() {
        let config = SettlementConfig {
            seed: 54321,
            tier: SettlementTier::Town,
            faction_control: vec![
                ("SaltTraders".to_string(), 0.4),
                ("Glassborn".to_string(), 0.3),
            ],
        };
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let settlement = generate_settlement(config, &mut rng);
        
        assert_eq!(settlement.width, 120);
        assert_eq!(settlement.height, 90);
    }

    #[test]
    fn test_faction_theme_dominant() {
        let config = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Village,
            faction_control: vec![
                ("MirrorMonks".to_string(), 0.5),
                ("Glassborn".to_string(), 0.3),
            ],
        };
        
        let dominant = faction_theme::get_dominant_faction(&config);
        assert_eq!(dominant, Some("MirrorMonks".to_string()));
    }

    #[test]
    fn test_faction_theme_significant() {
        let config = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Town,
            faction_control: vec![
                ("MirrorMonks".to_string(), 0.4),
                ("Glassborn".to_string(), 0.3),
                ("SaltTraders".to_string(), 0.2),
            ],
        };
        
        let significant = faction_theme::get_significant_factions(&config);
        assert_eq!(significant.len(), 2); // Only >25%
        assert!(significant.contains(&"MirrorMonks".to_string()));
        assert!(significant.contains(&"Glassborn".to_string()));
    }

    #[test]
    fn test_population_calculation() {
        let village = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Village,
            faction_control: vec![],
        };
        assert_eq!(population::calculate_population(&village), 20);

        let town = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Town,
            faction_control: vec![],
        };
        assert_eq!(population::calculate_population(&town), 50);

        let city = SettlementConfig {
            seed: 1,
            tier: SettlementTier::City,
            faction_control: vec![],
        };
        assert_eq!(population::calculate_population(&city), 100);
    }

    #[test]
    fn test_stamp_settlement() {
        use crate::game::map::{Map, Tile};
        
        let config = SettlementConfig {
            seed: 12345,
            tier: SettlementTier::Town,
            faction_control: vec![],
        };
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let settlement = generate_settlement(config, &mut rng);
        
        // Create a test map
        let mut map = Map {
            tiles: vec![Tile::default_floor(); 120 * 90],
            width: 120,
            height: 90,
            lights: vec![],
            features: vec![],
            inscriptions: vec![],
            area_description: None,
            metadata: std::collections::HashMap::new(),
        };
        
        // Stamp the settlement
        stamp_settlement(&mut map, &settlement);
        
        // Verify that some tiles were modified (should have walls/floors from buildings)
        let has_walls = map.tiles.iter().any(|tile| matches!(tile, Tile::Wall { .. }));
        assert!(has_walls, "Settlement stamping should create wall tiles");
    }
}
