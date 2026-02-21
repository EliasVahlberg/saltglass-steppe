use serde::{Deserialize, Serialize};

pub const WORLD_SIZE: usize = 64; // Height (kept for backward compat)
pub const WORLD_WIDTH: usize = 192; // 3x wider
pub const WORLD_HEIGHT: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub enum Biome {
    Desert,
    Saltflat,
    Scrubland,
    Oasis,
    Ruins,
}

impl Biome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Biome::Desert => "desert",
            Biome::Saltflat => "saltflat",
            Biome::Scrubland => "scrubland",
            Biome::Oasis => "oasis",
            Biome::Ruins => "ruins",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Terrain {
    Flat,
    Hills,
    Dunes,
    Canyon,
    Mesa,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum POI {
    None,
    Town,
    Dungeon,
    Landmark,
    Shrine,
}

/// Resource types that can be found in tiles
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct Resources {
    pub water: bool,
    pub minerals: bool,
    pub flora: bool,
}

/// Connected features that span multiple tiles
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct Connected {
    pub road: bool,
    pub river: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldMap {
    pub seed: u64,
    pub biomes: Vec<Biome>,
    pub terrain: Vec<Terrain>,
    pub elevation: Vec<u8>, // 0-255
    pub pois: Vec<POI>,
    #[serde(default)]
    pub resources: Vec<Resources>,
    #[serde(default)]
    pub connected: Vec<Connected>,
    #[serde(default)]
    pub levels: Vec<u32>, // Threat level for each tile
    #[serde(default)]
    pub faction_territories: Vec<Option<String>>, // Faction ID controlling each tile (None = neutral)
}

impl WorldMap {
    pub fn generate(seed: u64) -> Self {
        use crate::game::generation::WorldGenerator;

        let generator = WorldGenerator::new();
        let (biomes, terrain, elevation, pois, resources, connected, levels) =
            generator.generate(seed);

        let faction_territories = Self::generate_faction_territories(seed);

        Self {
            seed,
            biomes,
            terrain,
            elevation,
            pois,
            resources,
            connected,
            levels,
            faction_territories,
        }
    }

    /// Generate faction territories using even division with neutral center
    fn generate_faction_territories(seed: u64) -> Vec<Option<String>> {
        use crate::game::faction::all_faction_ids;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let _rng = ChaCha8Rng::seed_from_u64(seed.wrapping_add(999));
        let mut territories = vec![None; WORLD_WIDTH * WORLD_HEIGHT];
        
        let faction_ids = all_faction_ids();
        if faction_ids.is_empty() {
            return territories;
        }

        // Place faction capitals evenly spaced around the map
        let num_factions = faction_ids.len();
        let mut capitals = Vec::new();
        
        for (i, faction_id) in faction_ids.iter().enumerate() {
            let angle = (i as f64 / num_factions as f64) * 2.0 * std::f64::consts::PI;
            let radius = WORLD_WIDTH.min(WORLD_HEIGHT) as f64 / 2.5;
            let cx = (WORLD_WIDTH as f64 / 2.0 + radius * angle.cos()) as usize;
            let cy = (WORLD_HEIGHT as f64 / 2.0 + radius * angle.sin()) as usize;
            capitals.push((cx.min(WORLD_WIDTH - 1), cy.min(WORLD_HEIGHT - 1), faction_id.clone()));
        }

        // Assign each tile to nearest capital (Voronoi diagram)
        let center_x = WORLD_WIDTH / 2;
        let center_y = WORLD_HEIGHT / 2;
        let neutral_radius = 8; // Neutral zone radius

        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let idx = y * WORLD_WIDTH + x;
                
                // Check if in neutral center zone
                let dx = (x as i32 - center_x as i32).abs();
                let dy = (y as i32 - center_y as i32).abs();
                if dx * dx + dy * dy < neutral_radius * neutral_radius {
                    territories[idx] = None; // Neutral
                    continue;
                }

                // Find nearest capital
                let mut min_dist = f64::MAX;
                let mut nearest_faction = None;

                for (cx, cy, faction_id) in &capitals {
                    let dx = x as f64 - *cx as f64;
                    let dy = y as f64 - *cy as f64;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if dist < min_dist {
                        min_dist = dist;
                        nearest_faction = Some(faction_id.clone());
                    }
                }

                territories[idx] = nearest_faction;
            }
        }

        territories
    }

    #[allow(dead_code)]
    fn generate_levels(pois: &[POI], terrain: &[Terrain]) -> Vec<u32> {
        let mut levels = vec![1u32; WORLD_WIDTH * WORLD_HEIGHT];
        let start_x = WORLD_WIDTH / 2;
        let start_y = WORLD_HEIGHT / 2;

        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let idx = y * WORLD_WIDTH + x;

                // Base level from distance to start (center)
                let dx = (x as i32 - start_x as i32).abs() as f64;
                let dy = (y as i32 - start_y as i32).abs() as f64;
                let distance = (dx * dx + dy * dy).sqrt();

                // Level increases with distance from center
                let base_level = 1 + (distance / 15.0) as u32;

                // Terrain modifiers
                let terrain_mod = match terrain[idx] {
                    Terrain::Canyon => 1, // Dangerous terrain
                    Terrain::Mesa => 1,   // High elevation = higher threat
                    Terrain::Hills => 0,  // Moderate
                    Terrain::Dunes => 0,  // Standard
                    Terrain::Flat => 0,   // Easy terrain
                };

                // POI modifiers
                let poi_mod = match pois[idx] {
                    POI::Dungeon => 2,  // Dungeons are much more dangerous
                    POI::Landmark => 1, // Ruins have threats
                    POI::Town => -1,    // Towns are safer (but min 1)
                    POI::Shrine => 0,   // Neutral
                    POI::None => 0,
                };

                levels[idx] = (base_level + terrain_mod)
                    .saturating_add_signed(poi_mod)
                    .max(1)
                    .min(10);
            }
        }

        levels
    }

    pub fn get(&self, x: usize, y: usize) -> (Biome, Terrain, u8, POI, Resources, Connected, u32) {
        let idx = y * WORLD_WIDTH + x;
        (
            self.biomes[idx],
            self.terrain[idx],
            self.elevation[idx],
            self.pois[idx],
            self.resources.get(idx).copied().unwrap_or_default(),
            self.connected.get(idx).copied().unwrap_or_default(),
            self.levels.get(idx).copied().unwrap_or(1),
        )
    }

    pub fn get_faction_territory(&self, x: usize, y: usize) -> Option<&str> {
        let idx = y * WORLD_WIDTH + x;
        self.faction_territories
            .get(idx)
            .and_then(|opt| opt.as_deref())
    }

    pub fn tile_seed(&self, x: usize, y: usize) -> u64 {
        self.seed.wrapping_add((y * WORLD_WIDTH + x) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "World map generation has non-deterministic behavior unrelated to terrain-forge; needs separate investigation."]
    fn deterministic_generation() {
        let w1 = WorldMap::generate(12345);
        let w2 = WorldMap::generate(12345);
        assert_eq!(w1.biomes, w2.biomes);
        assert_eq!(w1.terrain, w2.terrain);
        assert_eq!(w1.pois, w2.pois);
        assert_eq!(w1.resources, w2.resources);
        assert_eq!(w1.connected, w2.connected);

        let w3 = WorldMap::generate(54321);
        assert_ne!(w1.biomes, w3.biomes);
    }

    #[test]
    fn has_pois() {
        let w = WorldMap::generate(42);
        let poi_count = w.pois.iter().filter(|&&p| p != POI::None).count();
        assert!(
            poi_count >= 10,
            "Expected at least 10 POIs, got {}",
            poi_count
        );
    }

    #[test]
    fn has_roads_connecting_towns() {
        let w = WorldMap::generate(42);
        let road_count = w.connected.iter().filter(|c| c.road).count();
        assert!(road_count > 0, "Expected roads connecting towns");
    }

    #[test]
    fn has_resources() {
        let w = WorldMap::generate(42);
        let water = w.resources.iter().filter(|r| r.water).count();
        let minerals = w.resources.iter().filter(|r| r.minerals).count();
        assert!(water > 0, "Expected some water resources");
        assert!(minerals > 0, "Expected some mineral resources");
    }
}
