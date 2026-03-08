use once_cell::sync::Lazy;
use rand::distributions::{Distribution, WeightedIndex};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;
use terrain_forge::{Grid, Rng as ForgeRng, SemanticExtractor, Tile as ForgeTile, ops};

use crate::game::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::game::map::{Map, MapFeature, Tile};
use crate::game::world_map::{Biome, POI, Terrain};

// These config structs are deserialized from terrain_config.json.
// Some fields are parsed for forward-compatibility but not yet used in generation logic.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct TerrainConfig {
    wall_type: String,
    floor_type: String,
    feature_weights: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct BiomeModifier {
    wall_type_override: Option<String>,
    floor_type_override: Option<String>,
    unique_features: Option<Vec<String>>,
    feature_weights: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct POILayout {
    central_clearing_size: usize,
    structure_density: Option<f64>,
    special_features: Option<Vec<String>>,
    // Town-specific
    building_clusters: Option<usize>,
    building_size_min: Option<usize>,
    building_size_max: Option<usize>,
    road_width: Option<usize>,
    market_area_size: Option<usize>,
    // Ruins-specific
    rubble_density: Option<f64>,
    partial_walls: Option<bool>,
    // Shrine-specific
    meditation_paths: Option<usize>,
    path_width: Option<usize>,
    altar_platform_size: Option<usize>,
    // Archive/Dungeon-specific
    chamber_count: Option<usize>,
    chamber_size_min: Option<usize>,
    chamber_size_max: Option<usize>,
    corridor_width: Option<usize>,
    dead_ends: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct AlgorithmLayer {
    algorithm: String,
    blend: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct BiomeAlgorithmProfile {
    default: HashMap<String, f64>,
    #[serde(default)]
    terrain_overrides: HashMap<String, HashMap<String, f64>>,
    #[serde(default)]
    algorithm_layers: Option<Vec<AlgorithmLayer>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct TileGenConfig {
    terrain_types: HashMap<String, TerrainConfig>,
    biome_modifiers: HashMap<String, BiomeModifier>,
    poi_layouts: HashMap<String, POILayout>,
    #[serde(default)]
    biome_algorithm_profiles: HashMap<String, BiomeAlgorithmProfile>,
    #[serde(default)]
    poi_algorithm_overrides: HashMap<String, HashMap<String, f64>>,
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

        // Check if POI should use DungeonGenerator
        if matches!(poi, POI::Dungeon | POI::Landmark | POI::Shrine) {
            if let Some(map) = generate_with_dungeon_generator(poi, seed, biome, terrain, &mut rng)
            {
                let floor_positions = collect_floor_positions(&map);
                return (map, floor_positions);
            }
            // Fall back to terrain-forge if DungeonGenerator fails
        }

        let mut grid: Grid<ForgeTile> = Grid::new(MAP_WIDTH, MAP_HEIGHT);

        let biome_key = match biome {
            Biome::Saltflat => "saltflat",
            Biome::Oasis => "oasis",
            Biome::Ruins => "ruins",
            Biome::Scrubland => "scrubland",
            Biome::Desert => "desert",
        };

        // Check for algorithm layers first
        if let Some(profile) = TILE_CONFIG.biome_algorithm_profiles.get(biome_key) {
            if let Some(layers) = &profile.algorithm_layers {
                // Use layered generation
                apply_layers(&mut grid, layers, seed);
            } else {
                // Existing single-algorithm path
                let algo_name = select_algorithm(
                    poi,
                    biome,
                    terrain,
                    TILE_CONFIG.structure_algorithm.as_deref(),
                    &mut rng,
                );
                if ops::generate(&algo_name, &mut grid, Some(seed), None).is_err() {
                    ops::generate("cellular", &mut grid, Some(seed), None).ok();
                }
                ops::generate("glass_seam", &mut grid, Some(seed), None).ok();
            }
        } else {
            // Fallback to single algorithm
            let algo_name = select_algorithm(
                poi,
                biome,
                terrain,
                TILE_CONFIG.structure_algorithm.as_deref(),
                &mut rng,
            );
            if ops::generate(&algo_name, &mut grid, Some(seed), None).is_err() {
                ops::generate("cellular", &mut grid, Some(seed), None).ok();
            }
            ops::generate("glass_seam", &mut grid, Some(seed), None).ok();
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

        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        
        // Set metadata based on generation method
        if let Some(profile) = TILE_CONFIG.biome_algorithm_profiles.get(biome_key) {
            if let Some(layers) = &profile.algorithm_layers {
                let layer_names: Vec<String> = layers.iter().map(|l| l.algorithm.clone()).collect();
                map.metadata.insert("tilegen_algorithm".to_string(), format!("layered: {}", layer_names.join(" -> ")));
            } else {
                let algo_name = select_algorithm(poi, biome, terrain, TILE_CONFIG.structure_algorithm.as_deref(), &mut rng);
                map.metadata.insert("tilegen_algorithm".to_string(), algo_name);
            }
        } else {
            let algo_name = select_algorithm(poi, biome, terrain, TILE_CONFIG.structure_algorithm.as_deref(), &mut rng);
            map.metadata.insert("tilegen_algorithm".to_string(), algo_name);
        }

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

        // Ensure map connectivity via Glass Seam Bridging (handled by terrain-forge during generation)
        // No manual post-processing needed - terrain-forge's GSB algorithm ensures connectivity

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
        
        // Determine primary algorithm for semantic extraction
        let primary_algo = if let Some(profile) = TILE_CONFIG.biome_algorithm_profiles.get(biome_key) {
            if let Some(layers) = &profile.algorithm_layers {
                // Use first non-glass_seam algorithm as primary
                layers.iter()
                    .find(|l| l.algorithm != "glass_seam")
                    .map(|l| l.algorithm.as_str())
                    .unwrap_or("cellular")
            } else {
                &select_algorithm(poi, biome, terrain, TILE_CONFIG.structure_algorithm.as_deref(), &mut rng)
            }
        } else {
            &select_algorithm(poi, biome, terrain, TILE_CONFIG.structure_algorithm.as_deref(), &mut rng)
        };
        
        let semantic = match primary_algo {
            "bsp" | "rooms" => SemanticExtractor::for_rooms(),
            "maze" => SemanticExtractor::for_mazes(),
            _ => SemanticExtractor::for_caves(), // cellular, drunkard, etc.
        }
        .extract(&grid, &mut forge_rng);

        // TODO: v0.7.0 SemanticExtractor doesn't support POI-specific marker configuration
        // (towns should get npc_slot/shop_slot, shrines should get altar, dungeons should get boss_core)

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
                feature_id: match &marker.marker_type {
                    terrain_forge::semantic::MarkerType::Custom(s) => s.clone(),
                    terrain_forge::semantic::MarkerType::Spawn => "Spawn".to_string(),
                    terrain_forge::semantic::MarkerType::Exit => "Exit".to_string(),
                    terrain_forge::semantic::MarkerType::QuestObjective { priority } => {
                        format!("QuestObjective_{}", priority)
                    }
                    terrain_forge::semantic::MarkerType::QuestStart => "QuestStart".to_string(),
                    terrain_forge::semantic::MarkerType::QuestEnd => "QuestEnd".to_string(),
                    terrain_forge::semantic::MarkerType::LootTier { tier } => {
                        format!("LootTier_{}", tier)
                    }
                    terrain_forge::semantic::MarkerType::Treasure => "Treasure".to_string(),
                    terrain_forge::semantic::MarkerType::EncounterZone { difficulty } => {
                        format!("EncounterZone_{}", difficulty)
                    }
                    terrain_forge::semantic::MarkerType::BossRoom => "BossRoom".to_string(),
                    terrain_forge::semantic::MarkerType::SafeZone => "SafeZone".to_string(),
                },
                source: Some("forge_marker".to_string()),
                metadata,
            });
        }

        // Inject POI-specific markers
        inject_poi_markers(&mut map, poi, &floor_positions, &mut rng);

        (map, floor_positions)
    }
}

fn generate_with_dungeon_generator(
    poi: POI,
    seed: u64,
    biome: Biome,
    terrain: Terrain,
    rng: &mut ChaCha8Rng,
) -> Option<Map> {
    use terrain_forge::{Grid, Params, Tile as ForgeTile};

    // Use biome-aware algorithm selection for POI tiles
    let algo = select_algorithm(poi, biome, terrain, None, rng);

    // Use BSP algorithm with different params based on POI type
    let mut params = Params::new();
    match poi {
        POI::Dungeon => {
            params.insert("min_room_size".to_string(), serde_json::json!(7));
            params.insert("max_depth".to_string(), serde_json::json!(4));
            params.insert("room_padding".to_string(), serde_json::json!(2));
        }
        POI::Landmark => {
            params.insert("min_room_size".to_string(), serde_json::json!(8));
            params.insert("max_depth".to_string(), serde_json::json!(5));
            params.insert("room_padding".to_string(), serde_json::json!(3));
        }
        POI::Shrine => {
            params.insert("min_room_size".to_string(), serde_json::json!(5));
            params.insert("max_depth".to_string(), serde_json::json!(3));
            params.insert("room_padding".to_string(), serde_json::json!(2));
        }
        _ => return None,
    };

    // Generate using selected algorithm (params only apply to bsp/rooms)
    let mut grid: Grid<ForgeTile> = Grid::new(MAP_WIDTH, MAP_HEIGHT);
    let params_ref = if algo == "bsp" || algo == "rooms" {
        Some(&params)
    } else {
        None
    };
    terrain_forge::ops::generate(&algo, &mut grid, Some(seed), params_ref).ok()?;

    // Get biome materials
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
        .or_else(|| TILE_CONFIG.terrain_types.get("desert"))?;
    let modifier = TILE_CONFIG.biome_modifiers.get(biome_key);

    let wall_id = modifier
        .and_then(|m| m.wall_type_override.clone())
        .unwrap_or_else(|| base_cfg.wall_type.clone());
    let floor_id = modifier
        .and_then(|m| m.floor_type_override.clone())
        .unwrap_or_else(|| base_cfg.floor_type.clone());

    // Convert to game map
    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let idx = y * MAP_WIDTH + x;
            map.tiles[idx] = match grid.get(x as i32, y as i32) {
                Some(ForgeTile::Wall) => Tile::Wall {
                    id: wall_id.clone(),
                    hp: 100,
                },
                Some(ForgeTile::Floor) => Tile::Floor {
                    id: floor_id.clone(),
                },
                _ => Tile::Wall {
                    id: wall_id.clone(),
                    hp: 100,
                },
            };
        }
    }

    map.metadata
        .insert("tilegen_algorithm".to_string(), algo.clone());
    Some(map)
}

fn select_algorithm(
    poi: POI,
    biome: Biome,
    terrain: Terrain,
    override_name: Option<&str>,
    rng: &mut ChaCha8Rng,
) -> String {
    if let Some(name) = override_name {
        return name.to_string();
    }

    // POI overrides take highest priority
    let poi_key = match poi {
        POI::Town => Some("town"),
        POI::Dungeon => Some("dungeon"),
        POI::Landmark => Some("landmark"),
        POI::Shrine => Some("shrine"),
        POI::None => None,
    };
    if let Some(weights) = poi_key.and_then(|k| TILE_CONFIG.poi_algorithm_overrides.get(k)) {
        return weighted_pick(weights, rng);
    }

    // Biome profile with optional terrain override
    let biome_key = biome.as_str();
    let terrain_key = match terrain {
        Terrain::Canyon => "canyon",
        Terrain::Mesa => "mesa",
        Terrain::Hills => "hills",
        Terrain::Dunes => "dunes",
        Terrain::Flat => "flat",
    };

    if let Some(profile) = TILE_CONFIG.biome_algorithm_profiles.get(biome_key) {
        let weights = profile
            .terrain_overrides
            .get(terrain_key)
            .unwrap_or(&profile.default);
        return weighted_pick(weights, rng);
    }

    // Fallback: cellular
    "cellular".to_string()
}

fn blend(base: &mut Grid<ForgeTile>, overlay: &Grid<ForgeTile>, mode: &str) {
    for (x, y, cell) in overlay.iter() {
        match mode {
            "replace" => { base.set(x as i32, y as i32, *cell); }
            "overlay" => {
                if *cell == ForgeTile::Wall {
                    base.set(x as i32, y as i32, *cell);
                }
            }
            "mask" => {
                if *cell == ForgeTile::Floor {
                    base.set(x as i32, y as i32, *cell);
                }
            }
            _ => { base.set(x as i32, y as i32, *cell); }
        }
    }
}

fn apply_layers(grid: &mut Grid<ForgeTile>, layers: &[AlgorithmLayer], seed: u64) {
    for (i, layer) in layers.iter().enumerate() {
        let layer_seed = seed.wrapping_add(i as u64 * 0x9e3779b9);
        let mut scratch: Grid<ForgeTile> = Grid::new(grid.width(), grid.height());
        ops::generate(&layer.algorithm, &mut scratch, Some(layer_seed), None).ok();
        blend(grid, &scratch, &layer.blend);
    }
}

fn weighted_pick(weights: &HashMap<String, f64>, rng: &mut ChaCha8Rng) -> String {
    let entries: Vec<_> = weights.iter().collect();
    if entries.is_empty() {
        return "cellular".to_string();
    }
    let dist = WeightedIndex::new(entries.iter().map(|(_, w)| *w))
        .unwrap_or_else(|_| WeightedIndex::new([1.0]).unwrap());
    entries[dist.sample(rng)].0.clone()
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

    // Create central clearing
    for y in center_y.saturating_sub(half)..=(center_y + half).min(MAP_HEIGHT - 1) {
        for x in center_x.saturating_sub(half)..=(center_x + half).min(MAP_WIDTH - 1) {
            map.tiles[y * MAP_WIDTH + x] = Tile::Floor {
                id: floor_id.to_string(),
            };
        }
    }

    // Apply POI-specific patterns
    if let Some(clusters) = layout.building_clusters {
        // Town: Create building clusters with roads
        let size_min = layout.building_size_min.unwrap_or(3);
        let size_max = layout.building_size_max.unwrap_or(6);

        for _ in 0..clusters {
            let bldg_x = rng.gen_range(
                center_x.saturating_sub(half + 5)
                    ..=center_x
                        .saturating_add(half + 5)
                        .min(MAP_WIDTH.saturating_sub(1)),
            );
            let bldg_y = rng.gen_range(
                center_y.saturating_sub(half + 5)
                    ..=center_y
                        .saturating_add(half + 5)
                        .min(MAP_HEIGHT.saturating_sub(1)),
            );
            let bldg_w = rng.gen_range(size_min..=size_max);
            let bldg_h = rng.gen_range(size_min..=size_max);

            // Create building walls
            for y in bldg_y..bldg_y + bldg_h {
                for x in bldg_x..bldg_x + bldg_w {
                    if x < MAP_WIDTH && y < MAP_HEIGHT {
                        if x == bldg_x
                            || x == bldg_x + bldg_w - 1
                            || y == bldg_y
                            || y == bldg_y + bldg_h - 1
                        {
                            map.tiles[y * MAP_WIDTH + x] = Tile::Wall {
                                id: wall_id.to_string(),
                                hp: 100,
                            };
                        }
                    }
                }
            }
        }
    } else if let Some(chamber_count) = layout.chamber_count {
        // Archive/Dungeon: Create chambers with corridors
        let size_min = layout.chamber_size_min.unwrap_or(4);
        let size_max = layout.chamber_size_max.unwrap_or(8);

        for _ in 0..chamber_count {
            let chamber_x = rng.gen_range(10..MAP_WIDTH.saturating_sub(10));
            let chamber_y = rng.gen_range(10..MAP_HEIGHT.saturating_sub(10));
            let chamber_w = rng.gen_range(size_min..=size_max);
            let chamber_h = rng.gen_range(size_min..=size_max);

            // Carve chamber
            for y in chamber_y..chamber_y + chamber_h {
                for x in chamber_x..chamber_x + chamber_w {
                    if x < MAP_WIDTH && y < MAP_HEIGHT {
                        map.tiles[y * MAP_WIDTH + x] = Tile::Floor {
                            id: floor_id.to_string(),
                        };
                    }
                }
            }
        }
    } else if let Some(paths) = layout.meditation_paths {
        // Shrine: Create meditation paths radiating from center
        let path_width = layout.path_width.unwrap_or(1);

        for i in 0..paths {
            let angle = (i as f64 / paths as f64) * 2.0 * std::f64::consts::PI;
            let dx = angle.cos();
            let dy = angle.sin();

            for dist in 0..20 {
                let px = center_x as i32 + (dx * dist as f64) as i32;
                let py = center_y as i32 + (dy * dist as f64) as i32;

                for offset in -(path_width as i32 / 2)..=(path_width as i32 / 2) {
                    let x = (px + offset).max(0).min(MAP_WIDTH as i32 - 1) as usize;
                    let y = py.max(0).min(MAP_HEIGHT as i32 - 1) as usize;

                    if x < MAP_WIDTH && y < MAP_HEIGHT {
                        map.tiles[y * MAP_WIDTH + x] = Tile::Floor {
                            id: floor_id.to_string(),
                        };
                    }
                }
            }
        }
    } else if let Some(density) = layout.structure_density {
        // Generic: Random wall clusters (fallback)
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
            let cluster_size = rng.gen_range(2..=4);
            for y in start_y..start_y + cluster_size {
                for x in start_x..start_x + cluster_size {
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

fn inject_poi_markers(
    map: &mut Map,
    poi: POI,
    floor_positions: &[(i32, i32)],
    rng: &mut ChaCha8Rng,
) {
    let center_x = MAP_WIDTH as i32 / 2;
    let center_y = MAP_HEIGHT as i32 / 2;

    match poi {
        POI::Town => {
            // Add shop_slot markers near center (2-3 merchants)
            let center_positions: Vec<_> = floor_positions
                .iter()
                .filter(|(fx, fy)| {
                    let dx = fx - center_x;
                    let dy = fy - center_y;
                    dx * dx + dy * dy < 100 // Within ~10 tiles of center
                })
                .collect();

            for _ in 0..rng.gen_range(2..=3) {
                if let Some(&&(x, y)) = center_positions.choose(rng) {
                    map.features.push(MapFeature {
                        x,
                        y,
                        feature_id: "shop_slot".to_string(),
                        source: Some("poi_injection".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }

            // Add npc_slot markers (3-5 town NPCs)
            for _ in 0..rng.gen_range(3..=5) {
                if let Some(&(x, y)) = floor_positions.choose(rng) {
                    map.features.push(MapFeature {
                        x,
                        y,
                        feature_id: "npc_slot".to_string(),
                        source: Some("poi_injection".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        POI::Shrine => {
            // Add altar at center
            if let Some(&(x, y)) = floor_positions.iter().min_by_key(|(fx, fy)| {
                let dx = fx - center_x;
                let dy = fy - center_y;
                dx * dx + dy * dy
            }) {
                map.features.push(MapFeature {
                    x,
                    y,
                    feature_id: "altar".to_string(),
                    source: Some("poi_injection".to_string()),
                    metadata: HashMap::new(),
                });
            }

            // Add 1-2 npc_slot for shrine keepers
            for _ in 0..rng.gen_range(1..=2) {
                if let Some(&(x, y)) = floor_positions.choose(rng) {
                    map.features.push(MapFeature {
                        x,
                        y,
                        feature_id: "npc_slot".to_string(),
                        source: Some("poi_injection".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        POI::Dungeon => {
            // Add boss_core marker in a far corner
            if let Some(&(x, y)) = floor_positions.iter().max_by_key(|(fx, fy)| {
                let dx = fx - center_x;
                let dy = fy - center_y;
                dx * dx + dy * dy
            }) {
                map.features.push(MapFeature {
                    x,
                    y,
                    feature_id: "boss_core".to_string(),
                    source: Some("poi_injection".to_string()),
                    metadata: HashMap::new(),
                });
            }

            // Add loot_slot markers (3-5 treasure spots)
            for _ in 0..rng.gen_range(3..=5) {
                if let Some(&(x, y)) = floor_positions.choose(rng) {
                    map.features.push(MapFeature {
                        x,
                        y,
                        feature_id: "loot_slot".to_string(),
                        source: Some("poi_injection".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        POI::Landmark => {
            // Add story_hook markers (2-3 lore fragments)
            for _ in 0..rng.gen_range(2..=3) {
                if let Some(&(x, y)) = floor_positions.choose(rng) {
                    map.features.push(MapFeature {
                        x,
                        y,
                        feature_id: "story_hook".to_string(),
                        source: Some("poi_injection".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }

            // Add loot_slot (1-2 relics)
            for _ in 0..rng.gen_range(1..=2) {
                if let Some(&(x, y)) = floor_positions.choose(rng) {
                    map.features.push(MapFeature {
                        x,
                        y,
                        feature_id: "loot_slot".to_string(),
                        source: Some("poi_injection".to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        _ => {} // No special markers for generic tiles
    }
}
