use bracket_noise::prelude::*;
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
use super::constraints::{ConstraintSystem, ConstraintContext, ObjectivePlacement, ConstraintSeverity};
use super::connectivity::{ensure_connectivity, GSBParams};
use super::structures::{Rectangle, Room, Corridor};
use super::structures::algorithms::SimpleRoom;
use super::structures::algorithms::*;

#[derive(Debug, Clone, Deserialize)]
pub struct TileGenConfig {
    pub terrain_types: HashMap<String, TerrainConfig>,
    pub biome_modifiers: HashMap<String, BiomeModifier>,
    pub poi_layouts: HashMap<String, POILayout>,
    pub feature_density: f64,
    pub variation_intensity: f64,
    pub structure_algorithm: Option<String>,
    pub algorithm_params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerrainConfig {
    pub floor_threshold: f64,
    pub glass_density: f64,
    pub noise_scale: f64,
    pub wall_type: String,
    pub floor_type: String,
    pub feature_weights: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BiomeModifier {
    pub glass_density_multiplier: Option<f64>,
    pub wall_type_override: Option<String>,
    pub floor_type_override: Option<String>,
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
    grammar: Grammar,
    template_library: TemplateLibrary,
}

impl TileGenerator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            grammar: Grammar::new(),
            template_library: TemplateLibrary::new(),
        })
    }
    
    /// Enhanced tile generation with integrated structure system
    pub fn generate_enhanced_tile_with_structures(
        &mut self, 
        poi_type: Option<POI>, 
        biome: &str,
        quest_ids: Vec<String>
    ) -> Map {
        self.generate_enhanced_tile_with_structures_seeded(poi_type, biome, quest_ids, 12345)
    }
    
    pub fn generate_enhanced_tile_with_structures_seeded(
        &mut self, 
        poi_type: Option<POI>, 
        biome: &str,
        quest_ids: Vec<String>,
        seed: u64
    ) -> Map {
        use rand::SeedableRng;
        
        // Convert string biome to enum
        let biome_enum = match biome {
            "saltflat" => Biome::Saltflat,
            "desert" => Biome::Desert,
            "ruins" => Biome::Ruins,
            "scrubland" => Biome::Scrubland,
            "oasis" => Biome::Oasis,
            _ => Biome::Saltflat,
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        
        // Generate base terrain using existing system
        let (map, _) = if let Some(poi) = poi_type {
            self.generate_enhanced_tile_with_quests(&mut rng, biome_enum, Terrain::Flat, 50, poi, &quest_ids)
        } else {
            self.generate_enhanced_tile(&mut rng, biome_enum, Terrain::Flat, 50, POI::None)
        };
        
        map
    }

    /// Generate enhanced tile map with all procedural systems and quest constraints
    pub fn generate_enhanced_tile(
        &self,
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        elevation: u8,
        poi: POI,
    ) -> (Map, Vec<(i32, i32)>) {
        self.generate_enhanced_tile_with_quests(rng, biome, terrain, elevation, poi, &[])
    }
    
    /// Generate enhanced tile map with quest constraint validation
    pub fn generate_enhanced_tile_with_quests(
        &self,
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        elevation: u8,
        poi: POI,
        quest_ids: &[String],
    ) -> (Map, Vec<(i32, i32)>) {
        const MAX_ATTEMPTS: usize = 5;
        
        for attempt in 0..MAX_ATTEMPTS {
            let seed = rng.next_u32();
            
            // Generate base terrain
            let (mut map, mut clearings) = self.generate_base_terrain(seed, biome, terrain, poi);
            
            // Validate constraints before proceeding (including quest constraints)
            let critical_satisfied = {
                let context = ConstraintContext {
                    map: &map,
                    biome,
                    entities: vec![], // No entities at terrain generation stage
                    resources: vec![], // No resources at terrain generation stage
                    objectives: vec![
                        // Add basic connectivity objectives
                        ObjectivePlacement {
                            objective_type: "spawn".to_string(),
                            x: MAP_WIDTH as i32 / 2,
                            y: MAP_HEIGHT as i32 / 2,
                            required: true,
                        },
                    ],
                };
                
                // Validate standard constraints
                let mut constraint_results = ConstraintSystem::validate_constraints(&context, rng);
                
                // Validate quest constraints if any quests are specified
                if !quest_ids.is_empty() {
                    use super::quest_constraints::QuestConstraintSystem;
                    let quest_results = QuestConstraintSystem::validate_quest_constraints(quest_ids, &context, rng);
                    constraint_results.extend(quest_results);
                }
                
                let critical_satisfied = ConstraintSystem::are_critical_constraints_satisfied(&constraint_results);
                
                if !critical_satisfied && attempt == MAX_ATTEMPTS - 1 {
                    // Apply emergency fixes for critical failures on last attempt
                    self.apply_emergency_fixes(&mut map, &constraint_results, rng);
                    
                    // Re-validate constraints after emergency fixes
                    let post_fix_context = ConstraintContext {
                        map: &map,
                        biome,
                        entities: vec![],
                        resources: vec![],
                        objectives: vec![
                            ObjectivePlacement {
                                objective_type: "spawn".to_string(),
                                x: MAP_WIDTH as i32 / 2,
                                y: MAP_HEIGHT as i32 / 2,
                                required: true,
                            },
                        ],
                    };
                    let post_fix_results = ConstraintSystem::validate_constraints(&post_fix_context, rng);
                    let post_fix_satisfied = ConstraintSystem::are_critical_constraints_satisfied(&post_fix_results);
                    
                    if !post_fix_satisfied {
                        println!("WARNING: Critical constraints still failing after emergency fixes");
                        for result in &post_fix_results {
                            if !result.passed && result.severity == ConstraintSeverity::Critical {
                                println!("  Still failing: {} - {}", result.rule_id, result.message);
                            }
                        }
                    }
                }
                
                critical_satisfied
            };
            
            // If constraints are satisfied or this is our last attempt, proceed
            if critical_satisfied || attempt == MAX_ATTEMPTS - 1 {
                
                // Add biome-specific features
                self.add_biome_features(&mut map, rng, biome, terrain);
                
                // Add quest-critical structures if this is a quest location
                if !quest_ids.is_empty() {
                    self.add_quest_structures(&mut map, rng, biome, poi, &quest_ids);
                }
                
                // Add procedural content using all systems
                self.add_procedural_content(&mut map, rng, biome, terrain, elevation, poi);
                
                // Ensure connectivity using Glass Seam Bridging algorithm
                self.ensure_basic_connectivity(&mut map, rng);
                
                // Enhance clearings with better distribution
                clearings.extend(self.find_enhanced_clearings(&map.tiles, biome, terrain));
                
                return (map, clearings);
            }
        }
        
        // This should never be reached due to the last attempt logic above
        unreachable!("Constraint validation loop should always return")
    }
    
    /// Apply emergency fixes for critical constraint failures
    fn apply_emergency_fixes(&self, map: &mut Map, constraint_results: &[super::constraints::ConstraintResult], rng: &mut ChaCha8Rng) {
        for result in constraint_results {
            if result.severity == ConstraintSeverity::Critical && !result.passed {
                match result.rule_id.as_str() {
                    "minimum_open_space" => {
                        self.ensure_minimum_open_space(map);
                    },
                    "basic_connectivity" | "exit_connectivity" | "connectivity_spawn_to_edges" => {
                        // Use GSB algorithm for all connectivity issues
                        let params = GSBParams::default();
                        let spawn = (MAP_WIDTH as i32 / 2, MAP_HEIGHT as i32 / 2);
                        ensure_connectivity(map, spawn, &params, rng);
                    },
                    _ => {}
                }
            }
        }
    }
    
    /// Ensure minimum open space by converting some walls to floors
    fn ensure_minimum_open_space(&self, map: &mut Map) {
        let current_open = map.tiles.iter()
            .filter(|tile| matches!(tile, Tile::Floor { id: _ } | Tile::Glass))
            .count();
        
        if current_open < 2000 {
            let needed = 2000 - current_open;
            let mut converted = 0;
            
            // Convert walls to floors in a pattern to create connected open space
            for y in (10..MAP_HEIGHT-10).step_by(3) {
                for x in (10..MAP_WIDTH-10).step_by(3) {
                    if converted >= needed { break; }
                    
                    let idx = y * MAP_WIDTH + x;
                    if matches!(map.tiles[idx], Tile::Wall { .. }) {
                        map.tiles[idx] = Tile::default_floor();
                        converted += 1;
                    }
                }
                if converted >= needed { break; }
            }
            // Emergency fix: converted walls to floors for minimum open space
        }
    }
    /// Ensure basic connectivity using Glass Seam Bridging algorithm
    fn ensure_basic_connectivity(&self, map: &mut Map, rng: &mut ChaCha8Rng) {
        // Use center as fallback spawn point
        let center_spawn = (MAP_WIDTH as i32 / 2, MAP_HEIGHT as i32 / 2);
        
        // Find actual clearings/spawn points
        let clearings = Map::find_clearings(&map.tiles);
        let spawn = if !clearings.is_empty() {
            clearings[0] // Use first clearing as spawn
        } else {
            center_spawn // Fallback to center
        };
        
        // Ensure the spawn point is always floor
        if let Some(idx) = map.pos_to_idx(spawn.0, spawn.1) {
            map.tiles[idx] = Tile::default_floor();
        }
        
        // Clear spawn area (5x5)
        for dy in -2..=2 {
            for dx in -2..=2 {
                let x = spawn.0 + dx;
                let y = spawn.1 + dy;
                if (dx.abs() <= 1 && dy.abs() <= 1) || (dx + dy).abs() <= 2 {
                    if let Some(idx) = map.pos_to_idx(x, y) {
                        map.tiles[idx] = Tile::default_floor();
                    }
                }
            }
        }
        
        // Apply Glass Seam Bridging algorithm
        let mut params = GSBParams::fast(); // Use fast profile for real-time generation
        params.min_area_ratio = 0.01; // Connect even very small regions (1% instead of 5%)
        params.connectivity_threshold = 0.95; // Require 95% connectivity
        let tunnels = ensure_connectivity(map, spawn, &params, rng);
        
        if !tunnels.is_empty() {
            #[cfg(debug_assertions)]
            println!("GSB: Created {} tunnels to ensure connectivity", tunnels.len());
        }
    }

    fn generate_base_terrain(&self, seed: u32, biome: Biome, terrain: Terrain, poi: POI) -> (Map, Vec<(i32, i32)>) {
        // Check if we should use structure algorithms instead of noise
        if let Some(algorithm) = &TILE_CONFIG.structure_algorithm {
            return self.generate_with_structure_algorithm(seed, algorithm, biome, terrain, poi);
        }
        
        // Original noise-based generation
        self.generate_noise_based_terrain(seed, biome, terrain, poi)
    }
    
    fn generate_with_structure_algorithm(&self, seed: u32, algorithm: &str, biome: Biome, terrain: Terrain, poi: POI) -> (Map, Vec<(i32, i32)>) {
        use rand::SeedableRng;
        let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
        let bounds = Rectangle::new(0, 0, MAP_WIDTH as u32, MAP_HEIGHT as u32);
        
        let mut tiles = vec![Tile::Wall { id: "stone".to_string(), hp: 100 }; MAP_WIDTH * MAP_HEIGHT];
        let mut clearings = Vec::new();
        
        match algorithm {
            "bsp" => {
                let params = BSPParams::default();
                let bsp = BSPAlgorithm::new(params);
                let (rooms, corridors) = bsp.generate(bounds, &mut rng);
                self.apply_bsp_result(&mut tiles, &rooms, &corridors, &mut clearings);
            },
            "cellular_automata" => {
                let params = CellularAutomataParams::default();
                let cellular = CellularAutomataAlgorithm::new(params);
                let walls = cellular.generate(bounds, &mut rng);
                self.apply_cellular_result(&mut tiles, &walls, &mut clearings);
            },
            "drunkard_walk" => {
                let params = DrunkardWalkParams::default();
                let drunkard = DrunkardWalkAlgorithm::new(params);
                let carved = drunkard.generate(bounds, &mut rng);
                self.apply_drunkard_result(&mut tiles, &carved, &mut clearings);
            },
            "simple_rooms" => {
                let params = SimpleRoomsParams::default();
                let simple = SimpleRoomsAlgorithm::new(params);
                let (simple_rooms, corridors) = simple.generate(bounds, &mut rng);
                self.apply_simple_rooms_result(&mut tiles, &simple_rooms, &corridors, &mut clearings);
            },
            _ => {
                // Fallback to noise-based generation for unimplemented algorithms
                return self.generate_noise_based_terrain(seed, biome, terrain, poi);
            }
        }
        
        let map = Map {
            tiles,
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            lights: Vec::new(),
            inscriptions: Vec::new(),
            area_description: None,
            metadata: std::collections::HashMap::new(),
        };
        
        (map, clearings)
    }
    
    fn apply_bsp_result(&self, tiles: &mut Vec<Tile>, rooms: &[Room], corridors: &[Corridor], clearings: &mut Vec<(i32, i32)>) {
        // Convert rooms to floor tiles
        for room in rooms {
            let bounds = &room.bounds;
            for y in bounds.y..(bounds.y + bounds.height) {
                for x in bounds.x..(bounds.x + bounds.width) {
                    if let Some(idx) = self.pos_to_idx(x as i32, y as i32) {
                        tiles[idx] = Tile::default_floor();
                    }
                }
            }
            // Add room center as clearing
            let center_x = bounds.x + bounds.width / 2;
            let center_y = bounds.y + bounds.height / 2;
            clearings.push((center_x as i32, center_y as i32));
        }
        
        // Add corridors - for now just connect start to end
        for corridor in corridors {
            let (start_x, start_y) = corridor.start;
            let (end_x, end_y) = corridor.end;
            
            // Simple line drawing from start to end
            let dx = end_x as i32 - start_x as i32;
            let dy = end_y as i32 - start_y as i32;
            let steps = dx.abs().max(dy.abs());
            
            if steps > 0 {
                for i in 0..=steps {
                    let x = start_x as i32 + (dx * i) / steps;
                    let y = start_y as i32 + (dy * i) / steps;
                    if let Some(idx) = self.pos_to_idx(x, y) {
                        tiles[idx] = Tile::default_floor();
                    }
                }
            }
        }
    }
    
    fn apply_cellular_result(&self, tiles: &mut Vec<Tile>, walls: &[(i32, i32)], clearings: &mut Vec<(i32, i32)>) {
        // Start with all floors
        for tile in tiles.iter_mut() {
            *tile = Tile::default_floor();
        }
        
        // Apply walls from cellular automata
        for &(x, y) in walls {
            if let Some(idx) = self.pos_to_idx(x, y) {
                tiles[idx] = Tile::Wall { id: "stone".to_string(), hp: 100 };
            }
        }
        
        // Find clearings (large open areas)
        clearings.extend(self.find_clearings(tiles));
    }
    
    fn apply_drunkard_result(&self, tiles: &mut Vec<Tile>, carved: &[(u32, u32)], clearings: &mut Vec<(i32, i32)>) {
        // Apply carved tiles as floors
        for &(x, y) in carved {
            if let Some(idx) = self.pos_to_idx(x as i32, y as i32) {
                tiles[idx] = Tile::default_floor();
            }
        }
        
        // Find clearings
        clearings.extend(self.find_clearings(tiles));
    }
    
    fn apply_simple_rooms_result(&self, tiles: &mut Vec<Tile>, simple_rooms: &[SimpleRoom], corridors: &[(u32, u32)], clearings: &mut Vec<(i32, i32)>) {
        // Convert SimpleRoom to floor tiles
        for room in simple_rooms {
            let bounds = &room.bounds;
            for y in bounds.y..(bounds.y + bounds.height) {
                for x in bounds.x..(bounds.x + bounds.width) {
                    if let Some(idx) = self.pos_to_idx(x as i32, y as i32) {
                        tiles[idx] = Tile::default_floor();
                    }
                }
            }
            // Add room center as clearing
            let center_x = bounds.x + bounds.width / 2;
            let center_y = bounds.y + bounds.height / 2;
            clearings.push((center_x as i32, center_y as i32));
        }
        
        // Add corridor tiles
        for &(x, y) in corridors {
            if let Some(idx) = self.pos_to_idx(x as i32, y as i32) {
                tiles[idx] = Tile::default_floor();
            }
        }
    }
    
    fn pos_to_idx(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < MAP_WIDTH as i32 && y >= 0 && y < MAP_HEIGHT as i32 {
            Some((y as usize) * MAP_WIDTH + (x as usize))
        } else {
            None
        }
    }

    fn generate_noise_based_terrain(&self, seed: u32, biome: Biome, terrain: Terrain, poi: POI) -> (Map, Vec<(i32, i32)>) {
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
        let mut floor_type = config.floor_type.clone();

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
            if let Some(override_type) = &modifier.floor_type_override {
                floor_type = override_type.clone();
            }
        }

        // Enhanced noise generation with multiple layers using bracket-noise
        // Use separate seeds to avoid affecting the main RNG state
        let mut noise = FastNoise::seeded(seed.wrapping_mul(1000) as u64);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(1.0 / config.noise_scale as f32);
        
        let mut glass_noise = FastNoise::seeded(seed.wrapping_mul(1000).wrapping_add(1) as u64);
        glass_noise.set_noise_type(NoiseType::Perlin);
        glass_noise.set_frequency(1.0 / config.noise_scale as f32);
        
        let mut variation_noise = FastNoise::seeded(seed.wrapping_mul(1000).wrapping_add(2) as u64);
        variation_noise.set_noise_type(NoiseType::Perlin);
        variation_noise.set_frequency(2.0 / config.noise_scale as f32);
        
        let mut feature_noise = FastNoise::seeded(seed.wrapping_mul(1000).wrapping_add(3) as u64);
        feature_noise.set_noise_type(NoiseType::Perlin);
        feature_noise.set_frequency(0.5 / config.noise_scale as f32);
        
        let wall_hp = self.get_wall_hp(&wall_type);
        let mut tiles = vec![Tile::Wall { id: wall_type.clone(), hp: wall_hp }; MAP_WIDTH * MAP_HEIGHT];
        let mut clearings = Vec::new();

        // Generate base terrain with enhanced variation
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let idx = y * MAP_WIDTH + x;
                let nx = x as f64 / config.noise_scale;
                let ny = y as f64 / config.noise_scale;
                
                // Multi-layer terrain generation
                // bracket-noise returns values in [-1, 1], keep in original range for compatibility
                let base_terrain = noise.get_noise(nx as f32, ny as f32) as f64; // Keep in [-1, 1]
                let variation = (variation_noise.get_noise((nx * 2.0) as f32, (ny * 2.0) as f32) as f64) * TILE_CONFIG.variation_intensity;
                let terrain_value = base_terrain + variation;
                
                // More varied floor generation
                let adjusted_threshold = floor_threshold + ((feature_noise.get_noise((nx * 0.5) as f32, (ny * 0.5) as f32) as f64) * 0.2);
                
                if terrain_value > adjusted_threshold {
                    tiles[idx] = Tile::floor(&floor_type);
                    
                    // Enhanced glass placement with patterns
                    let glass_value = glass_noise.get_noise((nx * 2.0) as f32, (ny * 2.0) as f32) as f64; // Keep in [-1, 1]
                    let pattern_factor = self.calculate_glass_pattern(x, y, biome, terrain);
                    
                    if glass_value > (0.0 - glass_density * pattern_factor) { // Adjust for [-1, 1] range
                        tiles[idx] = Tile::Glass;
                    }
                } else {
                    // Generate walls when terrain_value <= adjusted_threshold
                    tiles[idx] = Tile::Wall { id: wall_type.clone(), hp: 100 };
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
            metadata: std::collections::HashMap::new(),
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
        // Generate biome-specific environmental features using static method
        let features = BiomeSystem::generate_environmental_features(biome, 3, rng);
        
        // Convert features to map elements (lights, special tiles, etc.)
        for feature in features {
            if let Some(pos) = self.find_feature_placement(map, rng) {
                match feature.feature_type.as_str() {
                    "crystal_formation" | "salt_crystal_formation" => {
                        map.lights.push(MapLight {
                            x: pos.0,
                            y: pos.1,
                            id: "crystal".to_string(),
                        });
                    },
                    "glass_spire" | "salt_spire" => {
                        if let Some(idx) = map.pos_to_idx(pos.0, pos.1) {
                            if matches!(map.tiles[idx], Tile::Floor { id: _ }) {
                                map.tiles[idx] = Tile::Glass;
                            }
                        }
                    },
                    feature_type if feature_type.contains("crystal") => {
                        map.lights.push(MapLight {
                            x: pos.0,
                            y: pos.1,
                            id: "crystal".to_string(),
                        });
                    },
                    _ => {
                        // Other feature types can be handled here as needed
                    }
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
                ("terrain".to_string(), serde_json::Value::String(format!("{:?}", terrain).to_lowercase())),
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

        // Generate terrain-specific environmental templates
        let template_category = match terrain {
            Terrain::Canyon => "canyon_environmental",
            Terrain::Mesa => "mesa_environmental",
            Terrain::Hills => "hills_environmental", 
            Terrain::Dunes => "dunes_environmental",
            Terrain::Flat => "flat_environmental",
        };

        if let Ok(template) = self.template_library.instantiate(template_category, &context, rng) {
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
        
        // Fallback to generic environmental template
        if let Ok(template) = self.template_library.instantiate("environmental", &context, rng) {
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
                if matches!(tiles[y * MAP_WIDTH + x], Tile::Floor { id: _ }) {
                    let mut open_count = 0;
                    for dy in -(search_radius as i32)..=(search_radius as i32) {
                        for dx in -(search_radius as i32)..=(search_radius as i32) {
                            let ny = (y as i32 + dy) as usize;
                            let nx = (x as i32 + dx) as usize;
                            if ny < MAP_HEIGHT && nx < MAP_WIDTH {
                                if matches!(tiles[ny * MAP_WIDTH + nx], Tile::Floor { id: _ }) {
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
                        tiles[y * MAP_WIDTH + x] = Tile::default_floor();
                    }
                }
            }
        }
    }

    fn find_clearings(&self, tiles: &[Tile]) -> Vec<(i32, i32)> {
        let mut clearings = Vec::new();
        
        for y in 5..MAP_HEIGHT-5 {
            for x in 5..MAP_WIDTH-5 {
                if matches!(tiles[y * MAP_WIDTH + x], Tile::Floor { id: _ }) {
                    let mut open_count = 0;
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let ny = (y as i32 + dy) as usize;
                            let nx = (x as i32 + dx) as usize;
                            if ny < MAP_HEIGHT && nx < MAP_WIDTH {
                                if matches!(tiles[ny * MAP_WIDTH + nx], Tile::Floor { id: _ }) {
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
                if matches!(tile, Tile::Floor { id: _ }) {
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
                            if matches!(adj_tile, Tile::Floor { id: _ }) {
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
        
        // Test that basic map generation works
        assert!(map.tiles.len() == MAP_WIDTH * MAP_HEIGHT);
        
        // With the new bracket-noise implementation, we should still get some special features
        // The exact features may vary due to the noise implementation change, but shrine should have something
        let has_special_features = map.area_description.is_some() || 
                                  !map.inscriptions.is_empty() || 
                                  !map.lights.is_empty();
        assert!(has_special_features, "Shrine should have some special features");
    }
}

impl TileGenerator {
    /// Add quest-critical structures to the map
    fn add_quest_structures(&self, map: &mut Map, rng: &mut ChaCha8Rng, biome: Biome, poi: POI, quest_ids: &[String]) {
        // Check if this is the vitrified library location
        if quest_ids.iter().any(|id| id == "the_broken_key") && matches!(poi, POI::Landmark) && matches!(biome, Biome::Ruins) {
            self.place_vitrified_library_ruins(map, rng);
        }
    }
    
    /// Place the large vitrified library ruins structure using procedural generation
    fn place_vitrified_library_ruins(&self, map: &mut Map, rng: &mut ChaCha8Rng) {
        use crate::game::generation::structures::{RuinsGenerator, StructureGenerator, StructureParams, StructureType};
        
        let params = StructureParams {
            structure_type: StructureType::Ruins,
            size: (25, 20),
            theme: "vitrified_library".to_string(),
            quest_requirements: vec!["the_broken_key".to_string()],
            biome_context: "ruins".to_string(),
            organic_walls: false,
        };
        
        let generator = RuinsGenerator::new();
        if let Some(structure) = generator.generate(&params, rng) {
            self.integrate_structure_with_terrain(map, &structure);
        }
    }
    
    /// Integrate a generated structure with the existing terrain
    fn integrate_structure_with_terrain(&self, map: &mut Map, structure: &crate::game::generation::structures::Structure) {
        use crate::game::constants::{MAP_WIDTH, MAP_HEIGHT};
        use crate::game::map::Tile;
        
        // Calculate structure placement (centered in map)
        let start_x = (MAP_WIDTH as u32 - structure.bounds.width) / 2;
        let start_y = (MAP_HEIGHT as u32 - structure.bounds.height) / 2;
        
        // Place structure rooms
        for room in &structure.rooms {
            let room_start_x = start_x + room.bounds.x;
            let room_start_y = start_y + room.bounds.y;
            
            // Create room floor
            for dy in 0..room.bounds.height {
                for dx in 0..room.bounds.width {
                    let x = room_start_x + dx;
                    let y = room_start_y + dy;
                    
                    if x < MAP_WIDTH as u32 && y < MAP_HEIGHT as u32 {
                        let idx = (y * MAP_WIDTH as u32 + x) as usize;
                        if idx < map.tiles.len() {
                            map.tiles[idx] = Tile::Floor {
                                id: "stone_floor".to_string(),
                            };
                        }
                    }
                }
            }
            
            // Create room walls (perimeter)
            for dy in 0..=room.bounds.height + 1 {
                for dx in 0..=room.bounds.width + 1 {
                    let x = room_start_x + dx;
                    let y = room_start_y + dy;
                    
                    if x < MAP_WIDTH as u32 && y < MAP_HEIGHT as u32 {
                        let idx = (y * MAP_WIDTH as u32 + x) as usize;
                        if idx < map.tiles.len() {
                            // Only place walls on the perimeter
                            if dx == 0 || dx == room.bounds.width + 1 || dy == 0 || dy == room.bounds.height + 1 {
                                // Don't overwrite existing floors
                                if !matches!(map.tiles[idx], Tile::Floor { .. }) {
                                    map.tiles[idx] = Tile::Wall {
                                        id: "stone_wall".to_string(),
                                        hp: 100,
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Create connections between rooms (simple doorways)
        self.create_room_connections(map, structure, start_x, start_y);
        
        // Store structure metadata and spawn points
        for (key, value) in &structure.metadata {
            map.metadata.insert(key.clone(), value.clone());
        }
        
        // Store spawn points for later processing
        let spawn_data: Vec<_> = structure.spawn_points.iter().map(|sp| {
            (start_x + sp.position.0, start_y + sp.position.1, sp.spawn_type.as_str(), sp.entity_id.as_str())
        }).collect();
        map.metadata.insert("vitrified_library_spawns".to_string(), 
                           serde_json::to_string(&spawn_data).unwrap_or_default());
    }
    
    /// Create connections between rooms in a structure
    fn create_room_connections(&self, map: &mut Map, structure: &crate::game::generation::structures::Structure, start_x: u32, start_y: u32) {
        use crate::game::map::Tile;
        use crate::game::constants::{MAP_WIDTH, MAP_HEIGHT};
        
        // For the vitrified library, create doorways from chambers to main hall
        if structure.rooms.len() > 1 {
            let main_hall = &structure.rooms[0]; // First room is main hall
            
            for chamber in structure.rooms.iter().skip(1) {
                // Find closest points between main hall and chamber
                let hall_center_x = main_hall.bounds.x + main_hall.bounds.width / 2;
                let hall_center_y = main_hall.bounds.y + main_hall.bounds.height / 2;
                let chamber_center_x = chamber.bounds.x + chamber.bounds.width / 2;
                let chamber_center_y = chamber.bounds.y + chamber.bounds.height / 2;
                
                // Create a simple L-shaped corridor
                let corridor_start_x = start_x + chamber_center_x;
                let corridor_start_y = start_y + chamber_center_y;
                let corridor_end_x = start_x + hall_center_x;
                let corridor_end_y = start_y + hall_center_y;
                
                // Horizontal segment
                let min_x = corridor_start_x.min(corridor_end_x);
                let max_x = corridor_start_x.max(corridor_end_x);
                for x in min_x..=max_x {
                    if x < MAP_WIDTH as u32 && corridor_start_y < MAP_HEIGHT as u32 {
                        let idx = (corridor_start_y * MAP_WIDTH as u32 + x) as usize;
                        if idx < map.tiles.len() {
                            map.tiles[idx] = Tile::Floor {
                                id: "stone_floor".to_string(),
                            };
                        }
                    }
                }
                
                // Vertical segment
                let min_y = corridor_start_y.min(corridor_end_y);
                let max_y = corridor_start_y.max(corridor_end_y);
                for y in min_y..=max_y {
                    if corridor_end_x < MAP_WIDTH as u32 && y < MAP_HEIGHT as u32 {
                        let idx = (y * MAP_WIDTH as u32 + corridor_end_x) as usize;
                        if idx < map.tiles.len() {
                            map.tiles[idx] = Tile::Floor {
                                id: "stone_floor".to_string(),
                            };
                        }
                    }
                }
            }
        }
    }
}


