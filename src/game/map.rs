use bracket_pathfinding::prelude::*;
use noise::{NoiseFn, Perlin};
use once_cell::sync::Lazy;
use rand::{Rng, RngCore};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::constants::{FOV_RANGE, MAP_HEIGHT, MAP_WIDTH};
use super::light_defs::{get_spawn_rule, pick_light_type};
use super::world_map::{Biome, Terrain, POI};

#[derive(Debug, Clone, Deserialize)]
pub struct WallDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub hp: i32,
    pub description: String,
}

#[derive(Deserialize)]
struct WallsFile {
    walls: Vec<WallDef>,
}

#[derive(Debug, Clone, Deserialize)]
struct TerrainConfig {
    floor_threshold: f64,
    glass_density: f64,
    noise_scale: f64,
    wall_type: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BiomeModifier {
    glass_density_multiplier: Option<f64>,
    wall_type_override: Option<String>,
    floor_threshold_bonus: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct POILayout {
    central_clearing_size: usize,
}

#[derive(Deserialize)]
struct TerrainConfigFile {
    terrain_types: HashMap<String, TerrainConfig>,
    biome_modifiers: HashMap<String, BiomeModifier>,
    poi_layouts: HashMap<String, POILayout>,
}

static WALL_DEFS: Lazy<HashMap<String, WallDef>> = Lazy::new(|| {
    let data = include_str!("../../data/walls.json");
    let file: WallsFile = serde_json::from_str(data).expect("Failed to parse walls.json");
    file.walls.into_iter().map(|d| (d.id.clone(), d)).collect()
});

static TERRAIN_CONFIG: Lazy<TerrainConfigFile> = Lazy::new(|| {
    let data = include_str!("../../data/terrain_config.json");
    serde_json::from_str(data).expect("Failed to parse terrain_config.json")
});

pub fn get_wall_def(id: &str) -> Option<&'static WallDef> {
    WALL_DEFS.get(id)
}

fn random_wall_type(rng: &mut ChaCha8Rng) -> String {
    let types = ["sandstone", "shale", "salt_crystal"];
    types[rng.gen_range(0..types.len())].to_string()
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Tile {
    Floor,
    Wall { id: String, hp: i32 },
    Glass,
    Glare,      // Hot light tile that affects visibility and movement
    StairsDown,
    StairsUp,
    WorldExit,
}

impl Tile {
    pub fn glyph(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall { .. } => '#',
            Tile::Glass => '*',
            Tile::Glare => '░',
            Tile::StairsDown => '>',
            Tile::StairsUp => '<',
            Tile::WorldExit => 'O',
        }
    }
    pub fn walkable(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass | Tile::Glare | Tile::StairsDown | Tile::StairsUp | Tile::WorldExit) }
    pub fn transparent(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass | Tile::Glare | Tile::StairsDown | Tile::StairsUp | Tile::WorldExit) }

    pub fn name(&self) -> &str {
        match self {
            Tile::Floor => "Floor",
            Tile::Wall { id, .. } => get_wall_def(id).map(|d| d.name.as_str()).unwrap_or("Wall"),
            Tile::Glass => "Glass",
            Tile::Glare => "Glare",
            Tile::StairsDown => "Stairs Down",
            Tile::StairsUp => "Stairs Up",
            Tile::WorldExit => "World Map Exit",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Tile::Floor => "Dusty ground",
            Tile::Wall { id, .. } => get_wall_def(id).map(|d| d.description.as_str()).unwrap_or("Solid wall"),
            Tile::Glass => "Sharp refractive shards, dangerous to walk on",
            Tile::Glare => "Intense light that impairs vision and movement",
            Tile::StairsDown => "Stairs leading down into darkness",
            Tile::StairsUp => "Stairs leading back to the surface",
            Tile::WorldExit => "A passage to the world map",
        }
    }
}

/// Static light source placed in the map
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapLight {
    pub x: i32,
    pub y: i32,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub width: usize,
    pub height: usize,
    #[serde(default)]
    pub lights: Vec<MapLight>,
}

static VOID_WALL: Lazy<Tile> = Lazy::new(|| Tile::Wall { id: "void".to_string(), hp: 1000 });

impl Map {
    /// Generate a tile map from world context
    pub fn generate_from_world(
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        _elevation: u8,
    ) -> (Self, Vec<(i32, i32)>) {
        Self::generate_from_world_with_poi(rng, biome, terrain, _elevation, super::world_map::POI::None)
    }

    /// Generate a tile map from world context with POI
    pub fn generate_from_world_with_poi(
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        _elevation: u8,
        poi: POI,
    ) -> (Self, Vec<(i32, i32)>) {
        let seed = rng.next_u32();
        Self::generate_noise_terrain(seed, biome, terrain, poi)
    }

    fn generate_noise_terrain(
        seed: u32,
        biome: Biome,
        terrain: Terrain,
        poi: POI,
    ) -> (Self, Vec<(i32, i32)>) {
        // Get terrain config
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

        let config = &TERRAIN_CONFIG.terrain_types[terrain_key];
        let biome_mod = TERRAIN_CONFIG.biome_modifiers.get(biome_key);

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

        // Create noise generators
        let terrain_noise = Perlin::new(seed);
        let glass_noise = Perlin::new(seed + 1);
        let diagonal_noise = Perlin::new(seed + 2);
        
        let wall_hp = get_wall_def(&wall_type).map(|d| d.hp).unwrap_or(10);
        let mut tiles = vec![Tile::Wall { id: wall_type, hp: wall_hp }; MAP_WIDTH * MAP_HEIGHT];
        let mut clearings = Vec::new();

        // Generate base terrain with noise - more open areas
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let idx = y * MAP_WIDTH + x;
                let nx = x as f64 / config.noise_scale;
                let ny = y as f64 / config.noise_scale;
                
                let terrain_value = terrain_noise.get([nx, ny]);
                
                // Lower threshold for more open areas (50% more sparse)
                if terrain_value > (floor_threshold - 0.5) {
                    tiles[idx] = Tile::Floor;
                    
                    // Sharp diagonal glass formations
                    let diag_value = diagonal_noise.get([nx * 4.0, ny * 4.0]);
                    let diagonal_factor = ((x as f64 - y as f64) / 20.0).sin().abs();
                    
                    if diag_value > 0.4 && diagonal_factor > 0.7 {
                        tiles[idx] = Tile::Glass;
                    } else {
                        // Regular glass placement
                        let glass_value = glass_noise.get([nx * 2.0, ny * 2.0]);
                        if glass_value > (1.0 - glass_density * 0.7) {
                            tiles[idx] = Tile::Glass;
                        }
                    }
                }
            }
        }

        // Add POI-specific features
        if poi != POI::None {
            Self::add_poi_features(&mut tiles, poi, &clearings);
        }

        // Find natural clearings for spawn points
        clearings.extend(Self::find_clearings(&tiles));

        let map = Map {
            tiles,
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            lights: Vec::new(),
        };

        (map, clearings)
    }

    fn add_poi_features(tiles: &mut Vec<Tile>, poi: POI, _clearings: &[(i32, i32)]) {
        let poi_key = match poi {
            POI::Town => "town",
            POI::Landmark => "ruins", 
            POI::Shrine => "shrine",
            POI::Dungeon => "archive",
            _ => return,
        };

        if let Some(layout) = TERRAIN_CONFIG.poi_layouts.get(poi_key) {
            let center_x = MAP_WIDTH / 2;
            let center_y = MAP_HEIGHT / 2;
            let size = layout.central_clearing_size;

            // Create central clearing
            for y in center_y.saturating_sub(size/2)..=(center_y + size/2).min(MAP_HEIGHT-1) {
                for x in center_x.saturating_sub(size/2)..=(center_x + size/2).min(MAP_WIDTH-1) {
                    tiles[y * MAP_WIDTH + x] = Tile::Floor;
                }
            }
        }
    }

    fn find_clearings(tiles: &[Tile]) -> Vec<(i32, i32)> {
        let mut clearings = Vec::new();
        
        // Find floor areas that could serve as spawn points
        for y in 5..MAP_HEIGHT-5 {
            for x in 5..MAP_WIDTH-5 {
                if matches!(tiles[y * MAP_WIDTH + x], Tile::Floor) {
                    // Check if there's enough open space around this point
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
                    
                    if open_count >= 15 { // At least 15 of 25 tiles are floor
                        clearings.push((x as i32, y as i32));
                    }
                }
            }
        }
        
        clearings
    }

    /// Legacy generate (uses default biome/terrain)
    pub fn generate(rng: &mut ChaCha8Rng) -> (Self, Vec<(i32, i32)>) {
        let wall_type = random_wall_type(rng);
        let wall_hp = get_wall_def(&wall_type).map(|d| d.hp).unwrap_or(10);
        let mut tiles = vec![Tile::Wall { id: wall_type, hp: wall_hp }; MAP_WIDTH * MAP_HEIGHT];
        let mut room_centers = Vec::new();
        let num_rooms = rng.gen_range(5..9);

        for _ in 0..num_rooms {
            let w = rng.gen_range(4..10);
            let h = rng.gen_range(4..8);
            let x = rng.gen_range(1..MAP_WIDTH - w - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - h - 1);
            for ry in y..y + h {
                for rx in x..x + w {
                    tiles[ry * MAP_WIDTH + rx] = Tile::Floor;
                }
            }
            room_centers.push(((x + w / 2) as i32, (y + h / 2) as i32));
        }

        for i in 1..room_centers.len() {
            let (cx1, cy1) = room_centers[i - 1];
            let (cx2, cy2) = room_centers[i];
            for x in (cx1.min(cx2) as usize)..=(cx1.max(cx2) as usize) {
                tiles[cy1 as usize * MAP_WIDTH + x] = Tile::Floor;
            }
            for y in (cy1.min(cy2) as usize)..=(cy1.max(cy2) as usize) {
                tiles[y * MAP_WIDTH + cx2 as usize] = Tile::Floor;
            }
        }

        for _ in 0..rng.gen_range(10..20) {
            let x = rng.gen_range(1..MAP_WIDTH - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - 1);
            if tiles[y * MAP_WIDTH + x] == Tile::Floor {
                tiles[y * MAP_WIDTH + x] = Tile::Glass;
            }
        }

        // Spawn lights using default rules
        let rule = get_spawn_rule("default");
        let mut lights = Vec::new();
        for &(rx, ry) in &room_centers {
            let count = rng.gen_range(rule.lights_per_room[0]..=rule.lights_per_room[1]);
            for _ in 0..count {
                if let Some(light_id) = pick_light_type(rule, rng) {
                    let lx = rx + rng.gen_range(-2..=2);
                    let ly = ry + rng.gen_range(-2..=2);
                    lights.push(MapLight { x: lx, y: ly, id: light_id });
                }
            }
        }

        (Self { tiles, width: MAP_WIDTH, height: MAP_HEIGHT, lights }, room_centers)
    }

    /// Generate a subterranean map (cave/dungeon)
    pub fn generate_subterranean(rng: &mut ChaCha8Rng, layer: i32) -> (Self, Vec<(i32, i32)>) {
        let wall_type = "shale".to_string();
        let wall_hp = get_wall_def(&wall_type).map(|d| d.hp).unwrap_or(10);
        let mut tiles = vec![Tile::Wall { id: wall_type, hp: wall_hp }; MAP_WIDTH * MAP_HEIGHT];
        let mut room_centers = Vec::new();
        
        // More rooms deeper underground
        let num_rooms = rng.gen_range(4 + layer.unsigned_abs() as usize..8 + layer.unsigned_abs() as usize * 2);

        for _ in 0..num_rooms.min(12) {
            let w = rng.gen_range(3..8);
            let h = rng.gen_range(3..6);
            let x = rng.gen_range(1..MAP_WIDTH - w - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - h - 1);
            for ry in y..y + h {
                for rx in x..x + w {
                    tiles[ry * MAP_WIDTH + rx] = Tile::Floor;
                }
            }
            room_centers.push(((x + w / 2) as i32, (y + h / 2) as i32));
        }

        // Connect rooms with corridors
        for i in 1..room_centers.len() {
            let (cx1, cy1) = room_centers[i - 1];
            let (cx2, cy2) = room_centers[i];
            for x in (cx1.min(cx2) as usize)..=(cx1.max(cx2) as usize) {
                tiles[cy1 as usize * MAP_WIDTH + x] = Tile::Floor;
            }
            for y in (cy1.min(cy2) as usize)..=(cy1.max(cy2) as usize) {
                tiles[y * MAP_WIDTH + cx2 as usize] = Tile::Floor;
            }
        }

        // Place stairs up in first room
        if let Some(&(rx, ry)) = room_centers.first() {
            tiles[ry as usize * MAP_WIDTH + rx as usize] = Tile::StairsUp;
        }

        // Place stairs down in last room (if not at max depth)
        if layer > -3 {
            if let Some(&(rx, ry)) = room_centers.last() {
                tiles[ry as usize * MAP_WIDTH + rx as usize] = Tile::StairsDown;
            }
        }

        // Fewer lights underground
        let mut lights = Vec::new();
        for &(rx, ry) in room_centers.iter().take(2) {
            lights.push(MapLight { x: rx, y: ry, id: "crystal".to_string() });
        }

        (Self { tiles, width: MAP_WIDTH, height: MAP_HEIGHT, lights }, room_centers)
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Tile> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(&self.tiles[y as usize * self.width + x as usize])
        } else { None }
    }

    pub fn idx(&self, x: i32, y: i32) -> usize { y as usize * self.width + x as usize }

    /// Convert position to index, returning None if out of bounds
    pub fn pos_to_idx(&self, x: i32, y: i32) -> Option<usize> {
        if self.is_valid_position(x, y) {
            Some(self.idx(x, y))
        } else {
            None
        }
    }

    /// Check if position is within map bounds
    pub fn is_valid_position(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    /// Get tile at position, used by FOV system
    pub fn get_tile(&self, x: i32, y: i32) -> &Tile {
        self.get(x, y).unwrap_or(&VOID_WALL)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool { !self.tiles[idx].transparent() }
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = (idx % self.width) as i32;
        let y = (idx / self.width) as i32;
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if let Some(tile) = self.get(x + dx, y + dy) {
                if tile.walkable() {
                    exits.push((self.idx(x + dx, y + dy), 1.0));
                }
            }
        }
        exits
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point { Point::new(self.width as i32, self.height as i32) }
}

pub fn compute_fov(map: &Map, x: i32, y: i32) -> HashSet<usize> {
    field_of_view(Point::new(x, y), FOV_RANGE, map).into_iter().map(|p| map.idx(p.x, p.y)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn generate_from_world_deterministic() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        let (map1, rooms1) = Map::generate_from_world(&mut rng1, Biome::Saltflat, Terrain::Canyon, 100);
        let (map2, rooms2) = Map::generate_from_world(&mut rng2, Biome::Saltflat, Terrain::Canyon, 100);
        assert_eq!(map1.tiles, map2.tiles);
        assert_eq!(rooms1, rooms2);
    }

    #[test]
    fn biome_affects_wall_type() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let (map, _) = Map::generate_from_world(&mut rng, Biome::Saltflat, Terrain::Flat, 128);
        // Saltflat should use salt_crystal walls
        if let Tile::Wall { id, .. } = &map.tiles[0] {
            assert_eq!(id, "salt_crystal");
        }
    }
}
