use once_cell::sync::Lazy;
use rand::distributions::{Distribution, WeightedIndex};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;
use terrain_forge::{Grid, Tile as ForgeTile, algorithms};

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
    #[serde(default = "default_feature_density")]
    feature_density: f64,
    #[serde(default = "default_variation_intensity")]
    variation_intensity: f64,
    #[serde(default)]
    structure_algorithm: Option<String>,
    #[serde(default)]
    algorithm_params: Option<serde_json::Value>,
}

fn default_feature_density() -> f64 {
    0.1
}
fn default_variation_intensity() -> f64 {
    0.0
}

#[derive(Clone)]
struct FeatureCandidate {
    id: String,
    weight: f64,
    source: String,
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

        let mut available_positions = floor_positions.clone();

        if let Some(layout) = poi_layout {
            place_special_features(&mut map, layout, &mut available_positions, &mut rng);
        }

        let candidates = build_feature_candidates(base_cfg, modifier);
        place_features(
            &mut map,
            &candidates,
            TILE_CONFIG.feature_density,
            TILE_CONFIG.variation_intensity,
            available_positions,
            &mut rng,
        );

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

fn build_feature_candidates(
    terrain_cfg: &TerrainConfig,
    modifier: Option<&BiomeModifier>,
) -> Vec<FeatureCandidate> {
    let mut aggregated: HashMap<String, (f64, String)> = HashMap::new();

    if let Some(weights) = &terrain_cfg.feature_weights {
        for (id, weight) in weights {
            if *weight > 0.0 {
                aggregated
                    .entry(id.clone())
                    .and_modify(|(w, _)| *w += weight)
                    .or_insert((*weight, "terrain".to_string()));
            }
        }
    }

    if let Some(modifier) = modifier {
        if let Some(weights) = &modifier.feature_weights {
            for (id, weight) in weights {
                if *weight > 0.0 {
                    aggregated
                        .entry(id.clone())
                        .and_modify(|(w, s)| {
                            *w += weight;
                            if s == "terrain" {
                                *s = "biome".to_string();
                            }
                        })
                        .or_insert((*weight, "biome".to_string()));
                }
            }
        }

        if let Some(unique) = &modifier.unique_features {
            for id in unique {
                aggregated
                    .entry(id.clone())
                    .and_modify(|(w, s)| {
                        *w += 1.0;
                        if s == "terrain" {
                            *s = "biome".to_string();
                        }
                    })
                    .or_insert((1.0, "biome".to_string()));
            }
        }
    }

    aggregated
        .into_iter()
        .map(|(id, (weight, source))| FeatureCandidate { id, weight, source })
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
                });
            }
        }
    }
}

fn place_features(
    map: &mut Map,
    candidates: &[FeatureCandidate],
    feature_density: f64,
    variation_intensity: f64,
    mut available_positions: Vec<(i32, i32)>,
    rng: &mut ChaCha8Rng,
) {
    if candidates.is_empty() || available_positions.is_empty() || feature_density <= 0.0 {
        return;
    }

    available_positions.shuffle(rng);
    let density_scale = if variation_intensity > 0.0 {
        1.0 + rng.gen_range(-variation_intensity..=variation_intensity)
    } else {
        1.0
    };
    let scaled_density = (feature_density * density_scale).max(0.0);
    let feature_count = ((available_positions.len() as f64) * scaled_density).round() as usize;
    if feature_count == 0 {
        return;
    }

    let weights: Vec<f64> = candidates
        .iter()
        .map(|c| if c.weight > 0.0 { c.weight } else { 0.1 })
        .collect();
    let dist = match WeightedIndex::new(weights) {
        Ok(d) => d,
        Err(_) => return,
    };

    for pos in available_positions.into_iter().take(feature_count) {
        let idx = dist.sample(rng);
        let candidate = &candidates[idx];
        map.features.push(MapFeature {
            x: pos.0,
            y: pos.1,
            feature_id: candidate.id.clone(),
            source: Some(candidate.source.clone()),
        });
    }
}
