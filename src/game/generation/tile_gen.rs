use noise::{NoiseFn, Perlin};
use once_cell::sync::Lazy;
use rand::{Rng, RngCore};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;

use crate::game::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::game::map::{Tile, Map, MapLight, MapInscription};
use crate::game::world_map::{Biome, Terrain, POI};
use super::biomes::BiomeSystem;
use super::grammar::{Grammar, GrammarContext};
use super::templates::{TemplateLibrary, TemplateContext};

#[derive(Debug, Clone, Deserialize)]
pub struct TileGenConfig {
    pub terrain_types: HashMap<String, TerrainConfig>,
    pub biome_modifiers: HashMap<String, BiomeModifier>,
    pub poi_layouts: HashMap<String, POILayout>,
    pub feature_density: f64,
    pub variation_intensity: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerrainConfig {
    pub floor_threshold: f64,
    pub glass_density: f64,
    pub noise_scale: f64,
    pub wall_type: String,
    pub feature_weights: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BiomeModifier {
    pub glass_density_multiplier: Option<f64>,
    pub wall_type_override: Option<String>,
    pub floor_threshold_bonus: Option<f64>,
    pub unique_features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct POILayout {
    pub central_clearing_size: usize,
    pub structure_density: Option<f64>,
    pub special_features: Option<Vec<String>>,
}

static TILE_CONFIG: Lazy<TileGenConfig> = Lazy::new(|| {
    let data = include_str!("../../../data/terrain_config.json");
    serde_json::from_str(data).expect("Failed to parse terrain_config.json")
});

/// Enhanced tile generator using all procedural generation systems
pub struct TileGenerator {
    biome_system: BiomeSystem,
    grammar: Grammar,
    template_library: TemplateLibrary,
}

impl TileGenerator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            biome_system: BiomeSystem,
            grammar: Grammar::new(),
            template_library: TemplateLibrary::new(),
        })
    }

    /// Generate enhanced tile map with all procedural systems
    pub fn generate_enhanced_tile(
        &self,
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        elevation: u8,
        poi: POI,
    ) -> (Map, Vec<(i32, i32)>) {
        let seed = rng.next_u32();
        
        // Generate base terrain
        let (mut map, mut clearings) = self.generate_base_terrain(seed, biome, terrain, poi);
        
        // Add biome-specific features
        self.add_biome_features(&mut map, rng, biome, terrain);
        
        // Add procedural content using all systems
        self.add_procedural_content(&mut map, rng, biome, terrain, elevation, poi);
        
        // Enhance clearings with better distribution
        clearings.extend(self.find_enhanced_clearings(&map.tiles, biome, terrain));
        
        (map, clearings)
    }

    fn generate_base_terrain(&self, seed: u32, biome: Biome, terrain: Terrain, poi: POI) -> (Map, Vec<(i32, i32)>) {
        let terrain_key = match terrain {
            Terrain::Canyon => "canyon",
            Terrain::Mesa => "mesa", 
            Terrain::Hills => "hills",
            Terrain::Dunes => "dunes",
            Terrain::Flat => "flat",
        };
        
        let biome_key = match biome {
            Biome::Saltflat => "saltflat",
            Biome::Oasis => "oasis",
            Biome::Ruins => "ruins",
            _ => "desert",
        };

        let config = &TILE_CONFIG.terrain_types[terrain_key];
        let biome_mod = TILE_CONFIG.biome_modifiers.get(biome_key);

        // Apply biome modifiers
        let mut floor_threshold = config.floor_threshold;
        let mut glass_density = config.glass_density;
        let mut wall_type = config.wall_type.clone();

        if let Some(modifier) = biome_mod {
            if let Some(bonus) = modifier.floor_threshold_bonus {
                floor_threshold += bonus;
            }
            if let Some(multiplier) = modifier.glass_density_multiplier {
                glass_density *= multiplier;
            }
            if let Some(override_type) = &modifier.wall_type_override {
                wall_type = override_type.clone();
            }
        }

        // Enhanced noise generation with multiple layers
        let terrain_noise = Perlin::new(seed);
        let glass_noise = Perlin::new(seed + 1);
        let variation_noise = Perlin::new(seed + 2);
        let feature_noise = Perlin::new(seed + 3);
        
        let wall_hp = self.get_wall_hp(&wall_type);
        let mut tiles = vec![Tile::Wall { id: wall_type, hp: wall_hp }; MAP_WIDTH * MAP_HEIGHT];
        let mut clearings = Vec::new();

        // Generate base terrain with enhanced variation
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let idx = y * MAP_WIDTH + x;
                let nx = x as f64 / config.noise_scale;
                let ny = y as f64 / config.noise_scale;
                
                // Multi-layer terrain generation
                let base_terrain = terrain_noise.get([nx, ny]);
                let variation = variation_noise.get([nx * 2.0, ny * 2.0]) * TILE_CONFIG.variation_intensity;
                let terrain_value = base_terrain + variation;
                
                // More varied floor generation
                let adjusted_threshold = floor_threshold + (feature_noise.get([nx * 0.5, ny * 0.5]) * 0.2);
                
                if terrain_value > adjusted_threshold {
                    tiles[idx] = Tile::Floor;
                    
                    // Enhanced glass placement with patterns
                    let glass_value = glass_noise.get([nx * 2.0, ny * 2.0]);
                    let pattern_factor = self.calculate_glass_pattern(x, y, biome, terrain);
                    
                    if glass_value > (1.0 - glass_density * pattern_factor) {
                        tiles[idx] = Tile::Glass;
                    }
                }
            }
        }

        // Add POI-specific features
        if poi != POI::None {
            self.add_poi_features(&mut tiles, poi, &clearings);
        }

        // Find natural clearings
        clearings.extend(self.find_clearings(&tiles));

        let map = Map {
            tiles,
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            lights: Vec::new(),
            inscriptions: Vec::new(),
            area_description: None,
        };

        (map, clearings)
    }

    fn calculate_glass_pattern(&self, x: usize, y: usize, biome: Biome, terrain: Terrain) -> f64 {
        let mut pattern_factor = 1.0;
        
        // Biome-specific glass patterns
        match biome {
            Biome::Saltflat => {
                // Crystalline formations in saltflats
                let crystal_pattern = ((x as f64 / 8.0).sin() * (y as f64 / 8.0).cos()).abs();
                pattern_factor *= 1.0 + crystal_pattern;
            },
            Biome::Ruins => {
                // Shattered glass in ruins
                let shatter_pattern = ((x + y) % 7) as f64 / 7.0;
                pattern_factor *= 0.8 + shatter_pattern * 0.4;
            },
            _ => {}
        }
        
        // Terrain-specific patterns
        match terrain {
            Terrain::Canyon => {
                // Linear glass seams in canyons
                let seam_pattern = ((x as f64 - y as f64) / 10.0).sin().abs();
                pattern_factor *= 1.0 + seam_pattern * 0.5;
            },
            _ => {}
        }
        
        pattern_factor
    }

    fn add_biome_features(&self, map: &mut Map, rng: &mut ChaCha8Rng, biome: Biome, _terrain: Terrain) {
        // Generate biome-specific environmental features
        let features = BiomeSystem::generate_environmental_features(biome, 3, rng);
        
        // Convert features to map elements (lights, special tiles, etc.)
        for feature in features {
            if let Some(pos) = self.find_feature_placement(map, rng) {
                match feature.feature_type.as_str() {
                    "crystal_formation" => {
                        map.lights.push(MapLight {
                            x: pos.0,
                            y: pos.1,
                            id: "crystal".to_string(),
                        });
                    },
                    "glass_spire" => {
                        if let Some(idx) = map.pos_to_idx(pos.0, pos.1) {
                            if matches!(map.tiles[idx], Tile::Floor) {
                                map.tiles[idx] = Tile::Glass;
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn add_procedural_content(&self, map: &mut Map, rng: &mut ChaCha8Rng, biome: Biome, terrain: Terrain, elevation: u8, poi: POI) {
        // Generate area description using grammar system
        if let Some(description) = self.generate_area_description(rng, biome, terrain, poi) {
            map.area_description = Some(description);
        }
        
        // Add template-based content
        self.add_template_content(map, rng, biome, terrain, elevation, poi);
        
        // Add inscriptions using enhanced placement
        self.add_enhanced_inscriptions(map, rng, biome, poi);
    }

    fn generate_area_description(&self, rng: &mut ChaCha8Rng, biome: Biome, terrain: Terrain, poi: POI) -> Option<String> {
        let context = GrammarContext {
            variables: [
                ("biome".to_string(), biome.as_str().to_string()),
                ("terrain".to_string(), match terrain {
                    Terrain::Flat => "flat",
                    Terrain::Hills => "hills",
                    Terrain::Dunes => "dunes",
                    Terrain::Canyon => "canyon",
                    Terrain::Mesa => "mesa",
                }.to_string()),
                ("poi".to_string(), match poi {
                    POI::Town => "settlement",
                    POI::Shrine => "shrine",
                    POI::Landmark => "ruins",
                    POI::Dungeon => "archive",
                    POI::None => "wilderness",
                }.to_string()),
            ].into_iter().collect(),
        };

        self.grammar.generate("area_description", &context, rng).ok()
    }

    fn add_template_content(&self, map: &mut Map, rng: &mut ChaCha8Rng, biome: Biome, terrain: Terrain, elevation: u8, poi: POI) {
        let context = TemplateContext {
            variables: [
                ("biome".to_string(), serde_json::Value::String(biome.as_str().to_string())),
                ("elevation".to_string(), serde_json::Value::String(elevation.to_string())),
                ("poi_type".to_string(), serde_json::Value::String(match poi {
                    POI::Town => "settlement",
                    POI::Shrine => "shrine",
                    POI::Landmark => "ruins", 
                    POI::Dungeon => "archive",
                    POI::None => "wilderness",
                }.to_string())),
            ].into_iter().collect(),
        };

        // Generate environmental templates
        if let Ok(template) = self.template_library.instantiate("environmental", &context, rng) {
            // Apply template effects to map (could add special tiles, lights, etc.)
            if let Some(light_type) = template.get("light_source").and_then(|v| v.as_str()) {
                if let Some(pos) = self.find_feature_placement(map, rng) {
                    map.lights.push(MapLight {
                        x: pos.0,
                        y: pos.1,
                        id: light_type.to_string(),
                    });
                }
            }
        }
    }

    fn add_enhanced_inscriptions(&self, map: &mut Map, rng: &mut ChaCha8Rng, biome: Biome, poi: POI) {
        let inscription_count = match poi {
            POI::Town => rng.gen_range(3..7),
            POI::Shrine => rng.gen_range(4..8),
            POI::Landmark => rng.gen_range(2..5),
            POI::Dungeon => rng.gen_range(1..4),
            _ => rng.gen_range(0..3),
        };
        
        for _ in 0..inscription_count {
            if let Some((x, y)) = self.find_inscription_location(map, rng) {
                let inscription_type = if matches!(poi, POI::Shrine) && rng.gen_bool(0.7) {
                    "shrine_text"
                } else if rng.gen_bool(0.6) {
                    "inscription"
                } else {
                    "graffiti"
                };
                
                // Generate text using grammar system
                let context = GrammarContext {
                    variables: [
                        ("biome".to_string(), biome.as_str().to_string()),
                        ("type".to_string(), inscription_type.to_string()),
                    ].into_iter().collect(),
                };
                
                if let Ok(text) = self.grammar.generate("inscription", &context, rng) {
                    map.inscriptions.push(MapInscription {
                        x,
                        y,
                        text,
                        inscription_type: inscription_type.to_string(),
                    });
                }
            }
        }
    }

    fn find_enhanced_clearings(&self, tiles: &[Tile], biome: Biome, terrain: Terrain) -> Vec<(i32, i32)> {
        let mut clearings = Vec::new();
        
        // Biome-specific clearing requirements
        let min_open_tiles = match biome {
            Biome::Oasis => 20,      // Larger clearings in oases
            Biome::Saltflat => 12,   // Medium clearings in saltflats
            Biome::Ruins => 8,       // Smaller clearings in ruins
            _ => 15,                 // Default
        };
        
        let search_radius = match terrain {
            Terrain::Canyon => 1,    // Tight spaces in canyons
            Terrain::Mesa => 3,      // Wide spaces on mesas
            _ => 2,                  // Default
        };
        
        for y in (search_radius + 2)..(MAP_HEIGHT - search_radius - 2) {
            for x in (search_radius + 2)..(MAP_WIDTH - search_radius - 2) {
                if matches!(tiles[y * MAP_WIDTH + x], Tile::Floor) {
                    let mut open_count = 0;
                    for dy in -(search_radius as i32)..=(search_radius as i32) {
                        for dx in -(search_radius as i32)..=(search_radius as i32) {
                            let ny = (y as i32 + dy) as usize;
                            let nx = (x as i32 + dx) as usize;
                            if ny < MAP_HEIGHT && nx < MAP_WIDTH {
                                if matches!(tiles[ny * MAP_WIDTH + nx], Tile::Floor) {
                                    open_count += 1;
                                }
                            }
                        }
                    }
                    
                    let total_tiles = ((search_radius * 2 + 1) * (search_radius * 2 + 1)) as i32;
                    if open_count >= min_open_tiles.min(total_tiles - 5) {
                        clearings.push((x as i32, y as i32));
                    }
                }
            }
        }
        
        clearings
    }

    // Helper methods
    fn get_wall_hp(&self, wall_type: &str) -> i32 {
        match wall_type {
            "salt_crystal" => 8,
            "sandstone" => 12,
            "shale" => 15,
            _ => 10,
        }
    }

    fn add_poi_features(&self, tiles: &mut Vec<Tile>, poi: POI, _clearings: &[(i32, i32)]) {
        let poi_key = match poi {
            POI::Town => "town",
            POI::Landmark => "ruins", 
            POI::Shrine => "shrine",
            POI::Dungeon => "archive",
            _ => return,
        };

        if let Some(layout) = TILE_CONFIG.poi_layouts.get(poi_key) {
            let center_x = MAP_WIDTH / 2;
            let center_y = MAP_HEIGHT / 2;
            let size = layout.central_clearing_size;

            // Create central clearing with enhanced shape
            for y in center_y.saturating_sub(size/2)..=(center_y + size/2).min(MAP_HEIGHT-1) {
                for x in center_x.saturating_sub(size/2)..=(center_x + size/2).min(MAP_WIDTH-1) {
                    // Create more organic clearing shapes
                    let dx = x as i32 - center_x as i32;
                    let dy = y as i32 - center_y as i32;
                    let distance = ((dx * dx + dy * dy) as f64).sqrt();
                    
                    if distance <= (size as f64 / 2.0) {
                        tiles[y * MAP_WIDTH + x] = Tile::Floor;
                    }
                }
            }
        }
    }

    fn find_clearings(&self, tiles: &[Tile]) -> Vec<(i32, i32)> {
        let mut clearings = Vec::new();
        
        for y in 5..MAP_HEIGHT-5 {
            for x in 5..MAP_WIDTH-5 {
                if matches!(tiles[y * MAP_WIDTH + x], Tile::Floor) {
                    let mut open_count = 0;
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let ny = (y as i32 + dy) as usize;
                            let nx = (x as i32 + dx) as usize;
                            if ny < MAP_HEIGHT && nx < MAP_WIDTH {
                                if matches!(tiles[ny * MAP_WIDTH + nx], Tile::Floor) {
                                    open_count += 1;
                                }
                            }
                        }
                    }
                    
                    if open_count >= 15 {
                        clearings.push((x as i32, y as i32));
                    }
                }
            }
        }
        
        clearings
    }

    fn find_feature_placement(&self, map: &Map, rng: &mut ChaCha8Rng) -> Option<(i32, i32)> {
        for _ in 0..20 {
            let x = rng.gen_range(5..MAP_WIDTH-5) as i32;
            let y = rng.gen_range(5..MAP_HEIGHT-5) as i32;
            
            if let Some(tile) = map.get(x, y) {
                if matches!(tile, Tile::Floor) {
                    return Some((x, y));
                }
            }
        }
        None
    }

    fn find_inscription_location(&self, map: &Map, rng: &mut ChaCha8Rng) -> Option<(i32, i32)> {
        let mut candidates = Vec::new();
        
        for y in 0..map.height {
            for x in 0..map.width {
                let tile = &map.tiles[y * map.width + x];
                if matches!(tile, Tile::Wall { .. } | Tile::Glass) {
                    // Check if there's a floor tile adjacent
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        if let Some(adj_tile) = map.get(x as i32 + dx, y as i32 + dy) {
                            if matches!(adj_tile, Tile::Floor) {
                                candidates.push((x as i32, y as i32));
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        if candidates.is_empty() {
            None
        } else {
            Some(candidates[rng.gen_range(0..candidates.len())])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn tile_generator_deterministic() {
        let generator = TileGenerator::new().unwrap();
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        
        let (map1, clearings1) = generator.generate_enhanced_tile(&mut rng1, Biome::Desert, Terrain::Flat, 128, POI::None);
        let (map2, clearings2) = generator.generate_enhanced_tile(&mut rng2, Biome::Desert, Terrain::Flat, 128, POI::None);
        
        assert_eq!(map1.tiles, map2.tiles);
        assert_eq!(clearings1, clearings2);
    }

    #[test]
    fn enhanced_features_present() {
        let generator = TileGenerator::new().unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        
        let (map, _) = generator.generate_enhanced_tile(&mut rng, Biome::Saltflat, Terrain::Canyon, 200, POI::Shrine);
        
        // Should have area description
        assert!(map.area_description.is_some());
        
        // Should have inscriptions for shrine
        assert!(!map.inscriptions.is_empty());
        
        // Should have some lights/features
        assert!(!map.lights.is_empty());
    }
}
