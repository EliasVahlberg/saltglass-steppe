use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const WORLD_SIZE: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Biome { Desert, Saltflat, Scrubland, Oasis, Ruins }

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Terrain { Flat, Hills, Dunes, Canyon, Mesa }

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum POI { None, Town, Dungeon, Landmark, Shrine }

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldMap {
    pub seed: u64,
    pub biomes: Vec<Biome>,
    pub terrain: Vec<Terrain>,
    pub elevation: Vec<u8>,  // 0-255
    pub pois: Vec<POI>,
}

impl WorldMap {
    pub fn generate(seed: u64) -> Self {
        let biome_noise = Perlin::new(seed as u32);
        let terrain_noise = Perlin::new(seed as u32 + 1);
        let elev_noise = Perlin::new(seed as u32 + 2);

        let mut biomes = vec![Biome::Desert; WORLD_SIZE * WORLD_SIZE];
        let mut terrain = vec![Terrain::Flat; WORLD_SIZE * WORLD_SIZE];
        let mut elevation = vec![128u8; WORLD_SIZE * WORLD_SIZE];

        for y in 0..WORLD_SIZE {
            for x in 0..WORLD_SIZE {
                let idx = y * WORLD_SIZE + x;
                let nx = x as f64 / WORLD_SIZE as f64 * 4.0;
                let ny = y as f64 / WORLD_SIZE as f64 * 4.0;

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
            }
        }

        // POI placement with distance penalty
        let mut pois = vec![POI::None; WORLD_SIZE * WORLD_SIZE];
        let poi_types = [POI::Town, POI::Dungeon, POI::Landmark, POI::Shrine];
        let mut poi_positions: Vec<(usize, usize)> = Vec::new();
        let mut rng = ChaCha8Rng::seed_from_u64(seed + 100);

        for &poi_type in &poi_types {
            let count = match poi_type {
                POI::Town => 3,
                POI::Dungeon => 5,
                POI::Landmark => 4,
                POI::Shrine => 6,
                POI::None => 0,
            };
            for _ in 0..count {
                // Try to place with distance penalty
                let mut best = None;
                let mut best_score = f64::MIN;
                for _ in 0..50 {
                    let x = rng.gen_range(2..WORLD_SIZE - 2);
                    let y = rng.gen_range(2..WORLD_SIZE - 2);
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
                    pois[y * WORLD_SIZE + x] = poi_type;
                    poi_positions.push((x, y));
                }
            }
        }

        Self { seed, biomes, terrain, elevation, pois }
    }

    pub fn get(&self, x: usize, y: usize) -> (Biome, Terrain, u8, POI) {
        let idx = y * WORLD_SIZE + x;
        (self.biomes[idx], self.terrain[idx], self.elevation[idx], self.pois[idx])
    }

    pub fn tile_seed(&self, x: usize, y: usize) -> u64 {
        self.seed.wrapping_add((y * WORLD_SIZE + x) as u64)
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
    }

    #[test]
    fn has_pois() {
        let w = WorldMap::generate(42);
        let poi_count = w.pois.iter().filter(|&&p| p != POI::None).count();
        assert!(poi_count >= 10, "Expected at least 10 POIs, got {}", poi_count);
    }
}
