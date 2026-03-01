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

/// Stamp settlement buildings onto the map
pub fn stamp_settlement(map: &mut Map, settlement: &Settlement) {
    let library = match StructureLibrary::load() {
        Ok(lib) => lib,
        Err(_) => return, // Skip stamping if library fails to load
    };

    for building in &settlement.buildings {
        let structure = match library.get(&building.prefab_name) {
            Some(s) => s,
            None => continue, // Skip if structure not found
        };

        for (py, row) in structure.pattern.iter().enumerate() {
            for (px, &ch) in row.iter().enumerate() {
                if ch == ' ' {
                    continue; // Skip empty cells
                }

                let tile_x = building.x + px as i32;
                let tile_y = building.y + py as i32;

                if tile_x < 0 || tile_y < 0 || tile_x >= map.width as i32 || tile_y >= map.height as i32 {
                    continue; // Skip out of bounds
                }

                if let Some(legend_entry) = structure.legend.get(&ch) {
                    let tile = match legend_entry {
                        LegendEntry::Wall { id } => Tile::Wall { id: id.clone(), hp: 100 },
                        LegendEntry::Floor { id } => Tile::Floor { id: id.clone() },
                        LegendEntry::Door => Tile::Floor { id: "wood_floor".to_string() },
                        LegendEntry::Interactable { .. } => Tile::Floor { id: "wood_floor".to_string() },
                        LegendEntry::Npc { .. } => Tile::Floor { id: "wood_floor".to_string() },
                        LegendEntry::Structure { .. } => continue, // Skip nested structures
                    };

                    map.set_tile(tile_x as usize, tile_y as usize, tile);
                }
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
