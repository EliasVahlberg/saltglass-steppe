use once_cell::sync::Lazy;
use rand::distributions::{Distribution, WeightedIndex};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;
use terrain_forge::{
    Grid, Rng as ForgeRng, SemanticConfig, SemanticExtractor, Tile as ForgeTile, algorithms,
};

use crate::game::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::game::map::{Map, MapFeature, Tile};
use crate::game::world_map::{Biome, POI, Terrain};

#[derive(Debug, Clone, Deserialize)]
struct TerrainConfig {
    wall_type: String,
    floor_type: String,
    feature_weights: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize)]
struct BiomeModifier {
    wall_type_override: Option<String>,
    floor_type_override: Option<String>,
    unique_features: Option<Vec<String>>,
    feature_weights: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize)]
struct POILayout {
    central_clearing_size: usize,
    structure_density: Option<f64>,
    special_features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct TileGenConfig {
    terrain_types: HashMap<String, TerrainConfig>,
    biome_modifiers: HashMap<String, BiomeModifier>,
    poi_layouts: HashMap<String, POILayout>,
    #[serde(default = "default_variation_intensity")]
    variation_intensity: f64,
    #[serde(default)]
    structure_algorithm: Option<String>,
    #[serde(default)]
    algorithm_params: Option<serde_json::Value>,
}

fn default_variation_intensity() -> f64 {
    0.0
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
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut grid: Grid<ForgeTile> = Grid::new(MAP_WIDTH, MAP_HEIGHT);

        let algo_name = select_algorithm(
            poi,
            TILE_CONFIG.structure_algorithm.as_deref(),
            TILE_CONFIG.variation_intensity,
            &mut rng,
        );

        if let Some(algo) = algorithms::get(&algo_name) {
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

        let poi_layout = lookup_poi_layout(poi);

        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        map.metadata
            .insert("tilegen_algorithm".to_string(), algo_name.clone());
        if let Some(params) = TILE_CONFIG.algorithm_params.as_ref() {
            map.metadata
                .insert("tilegen_algorithm_params".to_string(), params.to_string());
        }

        for (x, y, cell) in grid.iter() {
            let idx = y * MAP_WIDTH + x;
            if idx < map.tiles.len() {
                map.tiles[idx] = match cell {
                    ForgeTile::Floor => Tile::Floor {
                        id: floor_id.clone(),
                    },
                    ForgeTile::Wall => Tile::Wall {
                        id: wall_id.clone(),
                        hp: 100,
                    },
                };
            }
        }

        if let Some(layout) = poi_layout {
            apply_poi_layout(&mut map, layout, &floor_id, &wall_id, &mut rng);
        }

        let mut floor_positions = collect_floor_positions(&map);
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

        // POI-specific features (kept simple and data-driven)
        if let Some(layout) = poi_layout {
            let mut available_positions = floor_positions.clone();
            place_special_features(&mut map, layout, &mut available_positions, &mut rng);
        }

        // Semantic extraction for spawn markers/regions
        let mut forge_rng = ForgeRng::new(seed);
        let mut semantic_config = semantic_config_for(&algo_name);
        semantic_config.marker_types = marker_types_for(poi);
        let extractor = SemanticExtractor::new(semantic_config);
        let semantic = extractor.extract(&grid, &mut forge_rng);

        let region_kinds: HashMap<u32, String> = semantic
            .regions
            .iter()
            .map(|r| (r.id, r.kind.clone()))
            .collect();
        map.metadata.insert(
            "forge_regions".to_string(),
            semantic.regions.len().to_string(),
        );
        map.metadata.insert(
            "forge_markers".to_string(),
            semantic.markers.len().to_string(),
        );
        map.metadata.insert(
            "forge_connectivity_edges".to_string(),
            semantic.connectivity.edges.len().to_string(),
        );

        for marker in semantic.markers {
            let mut metadata = marker.metadata.clone();
            if let Some(region_id) = marker.region_id {
                if let Some(kind) = region_kinds.get(&region_id) {
                    metadata.insert("region_kind".to_string(), kind.clone());
                }
                metadata.insert("region_id".to_string(), region_id.to_string());
            }
            metadata.insert("marker_weight".to_string(), marker.weight.to_string());

            map.features.push(MapFeature {
                x: marker.x as i32,
                y: marker.y as i32,
                feature_id: marker.tag.clone(),
                source: Some("forge_marker".to_string()),
                metadata,
            });
        }

        (map, floor_positions)
    }
}

fn select_algorithm(
    poi: POI,
    override_name: Option<&str>,
    variation_intensity: f64,
    rng: &mut ChaCha8Rng,
) -> String {
    if let Some(name) = override_name {
        return name.to_string();
    }

    let base = match poi {
        POI::Town => "rooms",
        POI::Dungeon | POI::Landmark => "bsp",
        _ => "cellular",
    };

    if variation_intensity <= 0.0 {
        return base.to_string();
    }

    let mut candidates = vec![(base.to_string(), 1.0 + variation_intensity)];
    for alt in ["cellular", "bsp", "rooms"] {
        if alt != base {
            candidates.push((alt.to_string(), variation_intensity.max(0.1)));
        }
    }

    let dist = WeightedIndex::new(candidates.iter().map(|(_, w)| *w))
        .unwrap_or_else(|_| WeightedIndex::new([1.0]).unwrap());
    candidates[dist.sample(rng)].0.clone()
}

fn lookup_poi_layout(poi: POI) -> Option<&'static POILayout> {
    let key = match poi {
        POI::Town => Some("town"),
        POI::Landmark => Some("ruins"),
        POI::Shrine => Some("shrine"),
        POI::Dungeon => Some("archive"),
        _ => None,
    }?;

    TILE_CONFIG.poi_layouts.get(key)
}

fn apply_poi_layout(
    map: &mut Map,
    layout: &POILayout,
    floor_id: &str,
    wall_id: &str,
    rng: &mut ChaCha8Rng,
) {
    let center_x = MAP_WIDTH / 2;
    let center_y = MAP_HEIGHT / 2;
    let half = layout.central_clearing_size / 2;

    for y in center_y.saturating_sub(half)..=(center_y + half).min(MAP_HEIGHT - 1) {
        for x in center_x.saturating_sub(half)..=(center_x + half).min(MAP_WIDTH - 1) {
            map.tiles[y * MAP_WIDTH + x] = Tile::Floor {
                id: floor_id.to_string(),
            };
        }
    }

    if let Some(density) = layout.structure_density {
        let clusters = (density * 10.0).ceil() as usize;
        for _ in 0..clusters {
            let start_x = rng.gen_range(
                center_x.saturating_sub(10)
                    ..=center_x.saturating_add(10).min(MAP_WIDTH.saturating_sub(1)),
            );
            let start_y = rng.gen_range(
                center_y.saturating_sub(6)
                    ..=center_y.saturating_add(6).min(MAP_HEIGHT.saturating_sub(1)),
            );
            for y in start_y..start_y + 2 {
                for x in start_x..start_x + 2 {
                    if x < MAP_WIDTH && y < MAP_HEIGHT {
                        map.tiles[y * MAP_WIDTH + x] = Tile::Wall {
                            id: wall_id.to_string(),
                            hp: 100,
                        };
                    }
                }
            }
        }
    }
}

fn collect_floor_positions(map: &Map) -> Vec<(i32, i32)> {
    map.tiles
        .iter()
        .enumerate()
        .filter_map(|(idx, tile)| match tile {
            Tile::Floor { .. } => {
                let x = (idx % MAP_WIDTH) as i32;
                let y = (idx / MAP_WIDTH) as i32;
                Some((x, y))
            }
            _ => None,
        })
        .collect()
}

fn semantic_config_for(algo_name: &str) -> SemanticConfig {
    let mut config = if matches!(algo_name, "rooms" | "bsp") {
        SemanticConfig::room_system()
    } else if algo_name == "maze" {
        SemanticConfig::maze_system()
    } else {
        SemanticConfig::cave_system()
    };
    config.max_markers_per_region = 4;
    config.marker_scaling_factor = 80.0;
    config.marker_placement.min_marker_distance = 3;
    config
}

fn marker_types_for(poi: POI) -> Vec<(String, f32)> {
    let mut markers = vec![
        ("light_anchor".to_string(), 0.6),
        ("loot_slot".to_string(), 0.45),
        ("enemy_spawn".to_string(), 0.5),
        ("story_hook".to_string(), 0.15),
    ];

    match poi {
        POI::Town => {
            markers.push(("npc_slot".to_string(), 0.4));
            markers.push(("shop_slot".to_string(), 0.25));
        }
        POI::Shrine => {
            markers.push(("altar".to_string(), 0.35));
            markers.push(("npc_slot".to_string(), 0.1));
        }
        POI::Dungeon | POI::Landmark => {
            markers.push(("boss_core".to_string(), 0.2));
        }
        _ => {}
    }

    markers
}

fn take_random_position(
    positions: &mut Vec<(i32, i32)>,
    rng: &mut ChaCha8Rng,
) -> Option<(i32, i32)> {
    if positions.is_empty() {
        return None;
    }
    let idx = rng.gen_range(0..positions.len());
    Some(positions.swap_remove(idx))
}

fn place_special_features(
    map: &mut Map,
    layout: &POILayout,
    available_positions: &mut Vec<(i32, i32)>,
    rng: &mut ChaCha8Rng,
) {
    if let Some(features) = &layout.special_features {
        for feature_id in features {
            if let Some((x, y)) = take_random_position(available_positions, rng) {
                map.features.push(MapFeature {
                    x,
                    y,
                    feature_id: feature_id.clone(),
                    source: Some("poi".to_string()),
                    metadata: HashMap::new(),
                });
            }
        }
    }
}
