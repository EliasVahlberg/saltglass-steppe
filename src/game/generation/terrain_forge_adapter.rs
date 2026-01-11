use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use terrain_forge::{algorithms, Grid, Tile as ForgeTile};

use crate::game::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::game::map::{Map, Tile};
use crate::game::world_map::{Biome, POI, Terrain};

#[derive(Debug, Clone, Deserialize)]
struct TerrainConfig {
    wall_type: String,
    floor_type: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BiomeModifier {
    wall_type_override: Option<String>,
    floor_type_override: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TileGenConfig {
    terrain_types: HashMap<String, TerrainConfig>,
    biome_modifiers: HashMap<String, BiomeModifier>,
}

static TILE_CONFIG: Lazy<TileGenConfig> = Lazy::new(|| {
    let data = include_str!("../../../data/terrain_config.json");
    serde_json::from_str(data).expect("Failed to parse terrain_config.json")
});

/// New tile generator backed by terrain-forge.
pub struct TerrainForgeGenerator;

impl TerrainForgeGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_tile_with_seed(
        &self,
        biome: Biome,
        terrain: Terrain,
        _elevation: u8,
        poi: POI,
        seed: u64,
        _quest_ids: &[String],
    ) -> (Map, Vec<(i32, i32)>) {
        let mut grid: Grid<ForgeTile> = Grid::new(MAP_WIDTH, MAP_HEIGHT);

        let algo_name = match poi {
            POI::Town => "rooms",
            POI::Dungeon | POI::Landmark => "bsp",
            _ => "cellular",
        };

        if let Some(algo) = algorithms::get(algo_name) {
            algo.generate(&mut grid, seed);
        } else if let Some(fallback) = algorithms::get("cellular") {
            fallback.generate(&mut grid, seed);
        }

        let biome_key = match biome {
            Biome::Saltflat => "saltflat",
            Biome::Oasis => "oasis",
            Biome::Ruins => "ruins",
            Biome::Scrubland => "scrubland",
            Biome::Desert => "desert",
        };
        let terrain_key = match terrain {
            Terrain::Canyon => "canyon",
            Terrain::Mesa => "mesa",
            Terrain::Hills => "hills",
            Terrain::Dunes => "dunes",
            Terrain::Flat => "flat",
        };

        let base_cfg = TILE_CONFIG
            .terrain_types
            .get(terrain_key)
            .or_else(|| TILE_CONFIG.terrain_types.get("desert"))
            .expect("terrain_config must contain defaults");

        let modifier = TILE_CONFIG.biome_modifiers.get(biome_key);
        let wall_id = modifier
            .and_then(|m| m.wall_type_override.clone())
            .unwrap_or_else(|| base_cfg.wall_type.clone());
        let floor_id = modifier
            .and_then(|m| m.floor_type_override.clone())
            .unwrap_or_else(|| base_cfg.floor_type.clone());

        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        let mut floor_positions = Vec::new();

        for (x, y, cell) in grid.iter() {
            let idx = y * MAP_WIDTH + x;
            if idx < map.tiles.len() {
                map.tiles[idx] = match cell {
                    ForgeTile::Floor => {
                        floor_positions.push((x as i32, y as i32));
                        Tile::Floor {
                            id: floor_id.clone(),
                        }
                    }
                    ForgeTile::Wall => Tile::Wall {
                        id: wall_id.clone(),
                        hp: 100,
                    },
                };
            }
        }

        // Ensure at least one floor exists for spawning
        if floor_positions.is_empty() {
            let cx = (MAP_WIDTH / 2) as i32;
            let cy = (MAP_HEIGHT / 2) as i32;
            let idx = cy as usize * MAP_WIDTH + cx as usize;
            if idx < map.tiles.len() {
                map.tiles[idx] = Tile::Floor {
                    id: floor_id.clone(),
                };
                floor_positions.push((cx, cy));
            }
        }

        (map, floor_positions)
    }
}
