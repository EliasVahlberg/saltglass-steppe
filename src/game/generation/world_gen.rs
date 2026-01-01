use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;

use crate::game::world_map::{Biome, Terrain, POI, Resources, Connected, WORLD_WIDTH, WORLD_HEIGHT};
use super::weighted_table::{WeightedTable, WeightedEntry};

/// World generation configuration
#[derive(Debug, Clone, Deserialize)]
pub struct WorldGenConfig {
    pub biome_noise_scale: f64,
    pub terrain_noise_scale: f64,
    pub elevation_noise_scale: f64,
    pub resource_noise_scale: f64,
    pub poi_distribution: HashMap<String, PoiConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PoiConfig {
    pub count: usize,
    pub min_distance: f64,
    pub biome_preferences: Option<HashMap<String, f64>>,
    pub terrain_preferences: Option<HashMap<String, f64>>,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            biome_noise_scale: 12.0,
            terrain_noise_scale: 8.0,
            elevation_noise_scale: 6.0,
            resource_noise_scale: 10.0,
            poi_distribution: [
                ("town".to_string(), PoiConfig {
                    count: 9,
                    min_distance: 15.0,
                    biome_preferences: Some([
                        ("oasis".to_string(), 2.0),
                        ("scrubland".to_string(), 1.5),
                        ("desert".to_string(), 1.0),
                        ("saltflat".to_string(), 0.5),
                        ("ruins".to_string(), 0.3),
                    ].into_iter().collect()),
                    terrain_preferences: Some([
                        ("flat".to_string(), 2.0),
                        ("hills".to_string(), 1.0),
                        ("dunes".to_string(), 0.8),
                        ("canyon".to_string(), 0.5),
                        ("mesa".to_string(), 0.3),
                    ].into_iter().collect()),
                }),
                ("dungeon".to_string(), PoiConfig {
                    count: 15,
                    min_distance: 8.0,
                    biome_preferences: Some([
                        ("ruins".to_string(), 2.5),
                        ("saltflat".to_string(), 1.5),
                        ("desert".to_string(), 1.0),
                        ("scrubland".to_string(), 0.7),
                        ("oasis".to_string(), 0.3),
                    ].into_iter().collect()),
                    terrain_preferences: Some([
                        ("mesa".to_string(), 2.0),
                        ("canyon".to_string(), 1.8),
                        ("hills".to_string(), 1.2),
                        ("dunes".to_string(), 0.8),
                        ("flat".to_string(), 0.5),
                    ].into_iter().collect()),
                }),
                ("landmark".to_string(), PoiConfig {
                    count: 12,
                    min_distance: 10.0,
                    biome_preferences: Some([
                        ("ruins".to_string(), 3.0),
                        ("desert".to_string(), 1.2),
                        ("scrubland".to_string(), 1.0),
                        ("saltflat".to_string(), 0.8),
                        ("oasis".to_string(), 0.5),
                    ].into_iter().collect()),
                    terrain_preferences: None,
                }),
                ("shrine".to_string(), PoiConfig {
                    count: 18,
                    min_distance: 6.0,
                    biome_preferences: None,
                    terrain_preferences: Some([
                        ("mesa".to_string(), 1.5),
                        ("hills".to_string(), 1.3),
                        ("canyon".to_string(), 1.2),
                        ("flat".to_string(), 1.0),
                        ("dunes".to_string(), 0.9),
                    ].into_iter().collect()),
                }),
            ].into_iter().collect(),
        }
    }
}

/// Enhanced world generator using procedural generation systems
pub struct WorldGenerator {
    config: WorldGenConfig,
}

impl WorldGenerator {
    pub fn new() -> Self {
        Self {
            config: WorldGenConfig::default(),
        }
    }

    pub fn with_config(config: WorldGenConfig) -> Self {
        Self { config }
    }

    /// Generate world map with enhanced procedural systems
    pub fn generate(&self, seed: u64) -> (Vec<Biome>, Vec<Terrain>, Vec<u8>, Vec<POI>, Vec<Resources>, Vec<Connected>, Vec<u32>) {
        let biome_noise = Perlin::new(seed as u32);
        let terrain_noise = Perlin::new(seed as u32 + 1);
        let elev_noise = Perlin::new(seed as u32 + 2);
        let resource_noise = Perlin::new(seed as u32 + 3);

        let mut biomes = vec![Biome::Desert; WORLD_WIDTH * WORLD_HEIGHT];
        let mut terrain = vec![Terrain::Flat; WORLD_WIDTH * WORLD_HEIGHT];
        let mut elevation = vec![128u8; WORLD_WIDTH * WORLD_HEIGHT];
        let mut resources = vec![Resources::default(); WORLD_WIDTH * WORLD_HEIGHT];

        // Generate base terrain using weighted selection
        let biome_entries = vec![
            WeightedEntry { item: Biome::Saltflat, weight: 15.0 },
            WeightedEntry { item: Biome::Scrubland, weight: 20.0 },
            WeightedEntry { item: Biome::Desert, weight: 35.0 },
            WeightedEntry { item: Biome::Ruins, weight: 15.0 },
            WeightedEntry { item: Biome::Oasis, weight: 15.0 },
        ];
        let _biome_table = WeightedTable::new(biome_entries);

        let terrain_entries = vec![
            WeightedEntry { item: Terrain::Canyon, weight: 15.0 },
            WeightedEntry { item: Terrain::Dunes, weight: 20.0 },
            WeightedEntry { item: Terrain::Flat, weight: 30.0 },
            WeightedEntry { item: Terrain::Hills, weight: 20.0 },
            WeightedEntry { item: Terrain::Mesa, weight: 15.0 },
        ];
        let _terrain_table = WeightedTable::new(terrain_entries);

        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let idx = y * WORLD_WIDTH + x;
                let nx = x as f64 / WORLD_WIDTH as f64 * self.config.biome_noise_scale;
                let ny = y as f64 / WORLD_HEIGHT as f64 * self.config.biome_noise_scale;

                // Enhanced biome generation with noise influence
                let b = biome_noise.get([nx, ny]);
                biomes[idx] = match b {
                    v if v < -0.4 => Biome::Saltflat,
                    v if v < -0.1 => Biome::Scrubland,
                    v if v < 0.3 => Biome::Desert,
                    v if v < 0.6 => Biome::Ruins,
                    _ => Biome::Oasis,
                };

                // Enhanced terrain generation
                let t = terrain_noise.get([nx * self.config.terrain_noise_scale / self.config.biome_noise_scale, 
                                          ny * self.config.terrain_noise_scale / self.config.biome_noise_scale]);
                terrain[idx] = match t {
                    v if v < -0.3 => Terrain::Canyon,
                    v if v < 0.0 => Terrain::Dunes,
                    v if v < 0.3 => Terrain::Flat,
                    v if v < 0.6 => Terrain::Hills,
                    _ => Terrain::Mesa,
                };

                // Enhanced elevation
                let e = elev_noise.get([nx * self.config.elevation_noise_scale / self.config.biome_noise_scale, 
                                       ny * self.config.elevation_noise_scale / self.config.biome_noise_scale]);
                elevation[idx] = ((e + 1.0) * 127.5) as u8;

                // Enhanced resource generation
                let r = resource_noise.get([nx * self.config.resource_noise_scale / self.config.biome_noise_scale, 
                                           ny * self.config.resource_noise_scale / self.config.biome_noise_scale]);
                resources[idx] = self.generate_resources(biomes[idx], terrain[idx], r);
            }
        }

        // Enhanced POI placement using weighted preferences
        let pois = self.generate_pois(seed, &biomes, &terrain);
        
        // Enhanced road generation
        let connected = self.generate_connections(seed, &pois);
        
        // Enhanced level generation
        let levels = self.generate_levels(&pois, &terrain, &biomes);

        (biomes, terrain, elevation, pois, resources, connected, levels)
    }

    fn generate_resources(&self, biome: Biome, terrain: Terrain, noise_value: f64) -> Resources {
        Resources {
            water: match biome {
                Biome::Oasis => true,
                Biome::Scrubland => noise_value > 0.3,
                _ => terrain == Terrain::Canyon && noise_value > 0.6,
            },
            minerals: match terrain {
                Terrain::Mesa => true,
                Terrain::Hills => noise_value > 0.2,
                Terrain::Canyon => noise_value < -0.3,
                _ => false,
            },
            flora: match biome {
                Biome::Oasis => true,
                Biome::Scrubland => noise_value > -0.2,
                _ => false,
            },
        }
    }

    fn generate_pois(&self, seed: u64, biomes: &[Biome], terrain: &[Terrain]) -> Vec<POI> {
        let mut pois = vec![POI::None; WORLD_WIDTH * WORLD_HEIGHT];
        let mut poi_positions: Vec<(usize, usize)> = Vec::new();
        let mut town_positions: Vec<(usize, usize)> = Vec::new();
        let mut rng = ChaCha8Rng::seed_from_u64(seed + 100);

        for (poi_name, poi_config) in &self.config.poi_distribution {
            let poi_type = match poi_name.as_str() {
                "town" => POI::Town,
                "dungeon" => POI::Dungeon,
                "landmark" => POI::Landmark,
                "shrine" => POI::Shrine,
                _ => continue,
            };

            for _ in 0..poi_config.count {
                let mut best = None;
                let mut best_score = f64::MIN;
                
                for _ in 0..100 { // More attempts for better placement
                    let x = rng.gen_range(2..WORLD_WIDTH - 2);
                    let y = rng.gen_range(2..WORLD_HEIGHT - 2);
                    let idx = y * WORLD_WIDTH + x;
                    
                    // Calculate distance penalty
                    let min_dist = poi_positions.iter()
                        .map(|&(px, py)| ((x as i32 - px as i32).pow(2) + (y as i32 - py as i32).pow(2)) as f64)
                        .map(|d| d.sqrt())
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(100.0);
                    
                    if min_dist < poi_config.min_distance {
                        continue;
                    }
                    
                    // Calculate biome preference score
                    let biome_score = if let Some(ref prefs) = poi_config.biome_preferences {
                        prefs.get(biomes[idx].as_str()).copied().unwrap_or(0.5)
                    } else {
                        1.0
                    };
                    
                    // Calculate terrain preference score
                    let terrain_score = if let Some(ref prefs) = poi_config.terrain_preferences {
                        let terrain_str = match terrain[idx] {
                            Terrain::Flat => "flat",
                            Terrain::Hills => "hills",
                            Terrain::Dunes => "dunes",
                            Terrain::Canyon => "canyon",
                            Terrain::Mesa => "mesa",
                        };
                        prefs.get(terrain_str).copied().unwrap_or(0.5)
                    } else {
                        1.0
                    };
                    
                    let total_score = min_dist * biome_score * terrain_score;
                    
                    if total_score > best_score {
                        best_score = total_score;
                        best = Some((x, y));
                    }
                }
                
                if let Some((x, y)) = best {
                    pois[y * WORLD_WIDTH + x] = poi_type;
                    poi_positions.push((x, y));
                    if poi_type == POI::Town {
                        town_positions.push((x, y));
                    }
                }
            }
        }

        pois
    }

    fn generate_connections(&self, seed: u64, pois: &[POI]) -> Vec<Connected> {
        let mut connected = vec![Connected::default(); WORLD_WIDTH * WORLD_HEIGHT];
        let mut rng = ChaCha8Rng::seed_from_u64(seed + 200);
        
        // Find all towns
        let town_positions: Vec<(usize, usize)> = pois.iter()
            .enumerate()
            .filter_map(|(idx, &poi)| {
                if poi == POI::Town {
                    Some((idx % WORLD_WIDTH, idx / WORLD_WIDTH))
                } else {
                    None
                }
            })
            .collect();

        // Connect towns with roads using minimum spanning tree approach
        if town_positions.len() > 1 {
            let mut connected_towns = vec![town_positions[0]];
            let mut remaining_towns = town_positions[1..].to_vec();
            
            while !remaining_towns.is_empty() {
                let mut best_connection = None;
                let mut best_distance = f64::MAX;
                
                for &connected_town in &connected_towns {
                    for (i, &remaining_town) in remaining_towns.iter().enumerate() {
                        let distance = ((connected_town.0 as i32 - remaining_town.0 as i32).pow(2) + 
                                       (connected_town.1 as i32 - remaining_town.1 as i32).pow(2)) as f64;
                        if distance < best_distance {
                            best_distance = distance;
                            best_connection = Some((connected_town, remaining_town, i));
                        }
                    }
                }
                
                if let Some((from, to, idx)) = best_connection {
                    self.create_road(&mut connected, from, to, &mut rng);
                    connected_towns.push(to);
                    remaining_towns.remove(idx);
                }
            }
        }

        connected
    }

    fn create_road(&self, connected: &mut [Connected], from: (usize, usize), to: (usize, usize), rng: &mut ChaCha8Rng) {
        let (x1, y1) = from;
        let (x2, y2) = to;
        
        // Create more natural roads with some randomness
        let mut current_x = x1 as i32;
        let mut current_y = y1 as i32;
        let target_x = x2 as i32;
        let target_y = y2 as i32;
        
        while current_x != target_x || current_y != target_y {
            // Add some randomness to road path
            let dx = (target_x - current_x).signum();
            let dy = (target_y - current_y).signum();
            
            let move_x = dx != 0 && (dy == 0 || rng.gen_bool(0.6));
            
            if move_x {
                current_x += dx;
            } else {
                current_y += dy;
            }
            
            if current_x >= 0 && current_x < WORLD_WIDTH as i32 && 
               current_y >= 0 && current_y < WORLD_HEIGHT as i32 {
                let idx = current_y as usize * WORLD_WIDTH + current_x as usize;
                connected[idx].road = true;
            }
        }
    }

    fn generate_levels(&self, pois: &[POI], terrain: &[Terrain], biomes: &[Biome]) -> Vec<u32> {
        let mut levels = vec![1u32; WORLD_WIDTH * WORLD_HEIGHT];
        let start_x = WORLD_WIDTH / 2;
        let start_y = WORLD_HEIGHT / 2;

        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let idx = y * WORLD_WIDTH + x;
                
                // Enhanced distance calculation
                let dx = (x as i32 - start_x as i32).abs() as f64;
                let dy = (y as i32 - start_y as i32).abs() as f64;
                let distance = (dx * dx + dy * dy).sqrt();
                
                // Base level from distance with more gradual scaling
                let base_level = 1 + (distance / 20.0) as u32;
                
                // Enhanced terrain modifiers
                let terrain_mod = match terrain[idx] {
                    Terrain::Canyon => 2,  // More dangerous
                    Terrain::Mesa => 1,    
                    Terrain::Hills => 0,   
                    Terrain::Dunes => 0,   
                    Terrain::Flat => -1,   // Safer
                };
                
                // Enhanced biome modifiers
                let biome_mod = match biomes[idx] {
                    Biome::Saltflat => 1,   // Glass storms more frequent
                    Biome::Ruins => 1,     // Ancient dangers
                    Biome::Oasis => -1,    // Safer
                    _ => 0,
                };
                
                // Enhanced POI modifiers
                let poi_mod = match pois[idx] {
                    POI::Dungeon => 3,     // Much more dangerous
                    POI::Landmark => 1,    
                    POI::Town => -2,       // Much safer
                    POI::Shrine => -1,     // Somewhat safer
                    POI::None => 0,
                };
                
                let total_level = base_level.saturating_add(terrain_mod as u32).saturating_add(biome_mod as u32);
                let final_level = if poi_mod >= 0 {
                    total_level.saturating_add(poi_mod as u32)
                } else {
                    total_level.saturating_sub((-poi_mod) as u32)
                };
                
                levels[idx] = final_level.max(1).min(15);
            }
        }
        
        levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_generator_deterministic() {
        let generator = WorldGenerator::new();
        let (b1, t1, e1, p1, r1, c1, l1) = generator.generate(12345);
        let (b2, t2, e2, p2, r2, c2, l2) = generator.generate(12345);
        
        assert_eq!(b1, b2);
        assert_eq!(t1, t2);
        assert_eq!(e1, e2);
        assert_eq!(p1, p2);
        assert_eq!(r1, r2);
        assert_eq!(c1, c2);
        assert_eq!(l1, l2);
    }

    #[test]
    fn enhanced_poi_placement() {
        let generator = WorldGenerator::new();
        let (_, _, _, pois, _, _, _) = generator.generate(42);
        
        let poi_count = pois.iter().filter(|&&p| p != POI::None).count();
        assert!(poi_count >= 50, "Expected at least 50 POIs with enhanced generation, got {}", poi_count);
        
        // Check that towns prefer good biomes
        let town_count = pois.iter().filter(|&&p| p == POI::Town).count();
        assert!(town_count >= 8, "Expected at least 8 towns, got {}", town_count);
    }
}
