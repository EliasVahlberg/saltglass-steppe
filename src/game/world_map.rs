use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const WORLD_SIZE: usize = 64;  // Height (kept for backward compat)
pub const WORLD_WIDTH: usize = 192; // 3x wider
pub const WORLD_HEIGHT: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Biome { Desert, Saltflat, Scrubland, Oasis, Ruins }

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Terrain { Flat, Hills, Dunes, Canyon, Mesa }

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum POI { None, Town, Dungeon, Landmark, Shrine }

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
    pub elevation: Vec<u8>,  // 0-255
    pub pois: Vec<POI>,
    #[serde(default)]
    pub resources: Vec<Resources>,
    #[serde(default)]
    pub connected: Vec<Connected>,
}

impl WorldMap {
    pub fn generate(seed: u64) -> Self {
        let biome_noise = Perlin::new(seed as u32);
        let terrain_noise = Perlin::new(seed as u32 + 1);
        let elev_noise = Perlin::new(seed as u32 + 2);
        let resource_noise = Perlin::new(seed as u32 + 3);

        let mut biomes = vec![Biome::Desert; WORLD_WIDTH * WORLD_HEIGHT];
        let mut terrain = vec![Terrain::Flat; WORLD_WIDTH * WORLD_HEIGHT];
        let mut elevation = vec![128u8; WORLD_WIDTH * WORLD_HEIGHT];
        let mut resources = vec![Resources::default(); WORLD_WIDTH * WORLD_HEIGHT];

        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let idx = y * WORLD_WIDTH + x;
                let nx = x as f64 / WORLD_WIDTH as f64 * 12.0; // Scale noise for wider map
                let ny = y as f64 / WORLD_HEIGHT as f64 * 4.0;

                // Biome from noise
                let b = biome_noise.get([nx, ny]);
                biomes[idx] = match b {
                    v if v < -0.4 => Biome::Saltflat,
                    v if v < -0.1 => Biome::Scrubland,
                    v if v < 0.3 => Biome::Desert,
                    v if v < 0.6 => Biome::Ruins,
                    _ => Biome::Oasis,
                };

                // Terrain from noise
                let t = terrain_noise.get([nx * 2.0, ny * 2.0]);
                terrain[idx] = match t {
                    v if v < -0.3 => Terrain::Canyon,
                    v if v < 0.0 => Terrain::Dunes,
                    v if v < 0.3 => Terrain::Flat,
                    v if v < 0.6 => Terrain::Hills,
                    _ => Terrain::Mesa,
                };

                // Elevation
                let e = elev_noise.get([nx * 1.5, ny * 1.5]);
                elevation[idx] = ((e + 1.0) * 127.5) as u8;

                // Resources based on biome and noise
                let r = resource_noise.get([nx * 3.0, ny * 3.0]);
                resources[idx] = Resources {
                    water: biomes[idx] == Biome::Oasis || (r > 0.6 && terrain[idx] == Terrain::Canyon),
                    minerals: terrain[idx] == Terrain::Mesa || (r < -0.5 && terrain[idx] == Terrain::Hills),
                    flora: biomes[idx] == Biome::Scrubland || biomes[idx] == Biome::Oasis,
                };
            }
        }

        // POI placement with distance penalty
        let mut pois = vec![POI::None; WORLD_WIDTH * WORLD_HEIGHT];
        let poi_types = [POI::Town, POI::Dungeon, POI::Landmark, POI::Shrine];
        let mut poi_positions: Vec<(usize, usize)> = Vec::new();
        let mut town_positions: Vec<(usize, usize)> = Vec::new();
        let mut rng = ChaCha8Rng::seed_from_u64(seed + 100);

        for &poi_type in &poi_types {
            let count = match poi_type {
                POI::Town => 9,      // 3x more towns
                POI::Dungeon => 15,  // 3x more dungeons
                POI::Landmark => 12, // 3x more landmarks
                POI::Shrine => 18,   // 3x more shrines
                POI::None => 0,
            };
            for _ in 0..count {
                let mut best = None;
                let mut best_score = f64::MIN;
                for _ in 0..50 {
                    let x = rng.gen_range(2..WORLD_WIDTH - 2);
                    let y = rng.gen_range(2..WORLD_HEIGHT - 2);
                    let min_dist = poi_positions.iter()
                        .map(|&(px, py)| ((x as i32 - px as i32).pow(2) + (y as i32 - py as i32).pow(2)) as f64)
                        .map(|d| d.sqrt())
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(100.0);
                    if min_dist > best_score {
                        best_score = min_dist;
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

        // Connect towns with roads
        let mut connected = vec![Connected::default(); WORLD_WIDTH * WORLD_HEIGHT];
        for i in 1..town_positions.len() {
            let (x1, y1) = town_positions[i - 1];
            let (x2, y2) = town_positions[i];
            // Simple L-shaped road
            for x in x1.min(x2)..=x1.max(x2) {
                connected[y1 * WORLD_WIDTH + x].road = true;
            }
            for y in y1.min(y2)..=y1.max(y2) {
                connected[y * WORLD_WIDTH + x2].road = true;
            }
        }

        Self { seed, biomes, terrain, elevation, pois, resources, connected }
    }

    pub fn get(&self, x: usize, y: usize) -> (Biome, Terrain, u8, POI, Resources, Connected) {
        let idx = y * WORLD_WIDTH + x;
        (
            self.biomes[idx],
            self.terrain[idx],
            self.elevation[idx],
            self.pois[idx],
            self.resources.get(idx).copied().unwrap_or_default(),
            self.connected.get(idx).copied().unwrap_or_default(),
        )
    }

    pub fn tile_seed(&self, x: usize, y: usize) -> u64 {
        self.seed.wrapping_add((y * WORLD_WIDTH + x) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_generation() {
        let w1 = WorldMap::generate(12345);
        let w2 = WorldMap::generate(12345);
        assert_eq!(w1.biomes, w2.biomes);
        assert_eq!(w1.terrain, w2.terrain);
        assert_eq!(w1.pois, w2.pois);
        assert_eq!(w1.resources, w2.resources);
        assert_eq!(w1.connected, w2.connected);
    }

    #[test]
    fn has_pois() {
        let w = WorldMap::generate(42);
        let poi_count = w.pois.iter().filter(|&&p| p != POI::None).count();
        assert!(poi_count >= 10, "Expected at least 10 POIs, got {}", poi_count);
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
