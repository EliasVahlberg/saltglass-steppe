use bracket_algorithm_traits::prelude::{Algorithm2D, BaseMap};
use bracket_geometry::prelude::Point;
use bracket_pathfinding::prelude::*;
use once_cell::sync::Lazy;
use rand::{Rng, RngCore};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};

use super::constants::{FOV_RANGE, MAP_HEIGHT, MAP_WIDTH};
use super::generation::TerrainForgeGenerator;
use super::light_defs::{get_spawn_rule, pick_light_type};
use super::world_map::{Biome, POI, Terrain};

#[derive(Debug, Clone, Deserialize)]
pub struct WallDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub color: String,
    pub hp: i32,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FloorDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub color: String,
    pub description: String,
}

#[derive(Deserialize)]
struct WallsFile {
    walls: Vec<WallDef>,
}

#[derive(Deserialize)]
struct FloorsFile {
    floors: Vec<FloorDef>,
}

static WALL_DEFS: Lazy<HashMap<String, WallDef>> = Lazy::new(|| {
    let data = include_str!("../../data/walls.json");
    let file: WallsFile = serde_json::from_str(data).expect("Failed to parse walls.json");
    file.walls.into_iter().map(|d| (d.id.clone(), d)).collect()
});

static FLOOR_DEFS: Lazy<HashMap<String, FloorDef>> = Lazy::new(|| {
    let data = include_str!("../../data/floors.json");
    let file: FloorsFile = serde_json::from_str(data).expect("Failed to parse floors.json");
    file.floors.into_iter().map(|d| (d.id.clone(), d)).collect()
});

pub fn get_wall_def(id: &str) -> Option<&'static WallDef> {
    WALL_DEFS.get(id)
}

pub fn get_floor_def(id: &str) -> Option<&'static FloorDef> {
    FLOOR_DEFS.get(id)
}

fn random_wall_type(rng: &mut ChaCha8Rng) -> String {
    let types = ["sandstone", "shale", "salt_crystal"];
    types[rng.gen_range(0..types.len())].to_string()
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Tile {
    Floor { id: String },
    Wall { id: String, hp: i32 },
    Glass,
    Glare, // Hot light tile that affects visibility and movement
    StairsDown,
    StairsUp,
    WorldExit,
}

impl Tile {
    /// Create a default floor tile
    pub fn default_floor() -> Self {
        Tile::Floor {
            id: "dry_soil".to_string(),
        }
    }

    /// Create a floor tile with specific type
    pub fn floor(id: &str) -> Self {
        Tile::Floor { id: id.to_string() }
    }

    pub fn glyph(&self) -> char {
        match self {
            Tile::Floor { id } => {
                if let Some(def) = get_floor_def(id) {
                    def.glyph.chars().next().unwrap_or('.')
                } else {
                    '.'
                }
            }
            Tile::Wall { id, .. } => {
                if let Some(def) = get_wall_def(id) {
                    def.glyph.chars().next().unwrap_or('#')
                } else {
                    '#'
                }
            }
            Tile::Glass => '*',
            Tile::Glare => '░',
            Tile::StairsDown => '>',
            Tile::StairsUp => '<',
            Tile::WorldExit => 'O',
        }
    }
    pub fn walkable(&self) -> bool {
        matches!(
            self,
            Tile::Floor { .. }
                | Tile::Glass
                | Tile::Glare
                | Tile::StairsDown
                | Tile::StairsUp
                | Tile::WorldExit
        )
    }
    pub fn transparent(&self) -> bool {
        matches!(
            self,
            Tile::Floor { .. }
                | Tile::Glass
                | Tile::Glare
                | Tile::StairsDown
                | Tile::StairsUp
                | Tile::WorldExit
        )
    }

    pub fn name(&self) -> &str {
        match self {
            Tile::Floor { id } => {
                if let Some(def) = get_floor_def(id) {
                    &def.name
                } else {
                    "Floor"
                }
            }
            Tile::Wall { id, .. } => {
                if let Some(def) = get_wall_def(id) {
                    &def.name
                } else {
                    "Wall"
                }
            }
            Tile::Glass => "Glass",
            Tile::Glare => "Glare",
            Tile::StairsDown => "Stairs Down",
            Tile::StairsUp => "Stairs Up",
            Tile::WorldExit => "World Exit",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Tile::Floor { id } => {
                if let Some(def) = get_floor_def(id) {
                    &def.description
                } else {
                    "Dusty ground"
                }
            }
            Tile::Wall { id, .. } => {
                if let Some(def) = get_wall_def(id) {
                    &def.description
                } else {
                    "Solid wall"
                }
            }
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

/// Data-driven feature placed on the map
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapFeature {
    pub x: i32,
    pub y: i32,
    pub feature_id: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Inscription or graffiti placed on the map
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapInscription {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub inscription_type: String, // "inscription", "graffiti", "shrine_text"
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub width: usize,
    pub height: usize,
    #[serde(default)]
    pub lights: Vec<MapLight>,
    #[serde(default)]
    pub features: Vec<MapFeature>,
    #[serde(default)]
    pub inscriptions: Vec<MapInscription>,
    #[serde(default)]
    pub area_description: Option<String>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

static VOID_WALL: Lazy<Tile> = Lazy::new(|| Tile::Wall {
    id: "void".to_string(),
    hp: 1000,
});

impl Map {
    /// Create a new empty map filled with walls
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![
                Tile::Wall {
                    id: "stone".to_string(),
                    hp: 100
                };
                width * height
            ],
            lights: Vec::new(),
            features: Vec::new(),
            inscriptions: Vec::new(),
            area_description: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set a tile at the given coordinates
    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.tiles[idx] = tile;
        }
    }

    /// Generate a tile map from world context
    pub fn generate_from_world(
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        _elevation: u8,
    ) -> (Self, Vec<(i32, i32)>) {
        Self::generate_from_world_with_poi(
            rng,
            biome,
            terrain,
            _elevation,
            super::world_map::POI::None,
        )
    }

    /// Generate a tile map from world context with POI and quest constraints
    pub fn generate_from_world_with_poi(
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        elevation: u8,
        poi: POI,
    ) -> (Self, Vec<(i32, i32)>) {
        Self::generate_from_world_with_poi_and_quests(rng, biome, terrain, elevation, poi, &[])
    }

    /// Generate a tile map with quest constraint validation
    pub fn generate_from_world_with_poi_and_quests(
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        elevation: u8,
        poi: POI,
        quest_ids: &[String],
    ) -> (Self, Vec<(i32, i32)>) {
        let generator = TerrainForgeGenerator::new();
        let (mut map, spawn_points) = generator.generate_tile_with_seed(
            biome,
            terrain,
            elevation,
            poi,
            rng.next_u64(),
            quest_ids,
        );
        map.generate_narrative_content(rng, biome, terrain, poi);
        (map, spawn_points)
    }

    /// Legacy generate (uses default biome/terrain)
    pub fn generate(rng: &mut ChaCha8Rng) -> (Self, Vec<(i32, i32)>) {
        let wall_type = random_wall_type(rng);
        let wall_hp = get_wall_def(&wall_type).map(|d| d.hp).unwrap_or(10);
        let floor_type = "dry_soil".to_string(); // Default floor type
        let mut tiles = vec![
            Tile::Wall {
                id: wall_type,
                hp: wall_hp
            };
            MAP_WIDTH * MAP_HEIGHT
        ];
        let mut room_centers = Vec::new();
        let num_rooms = rng.gen_range(5..9);

        for _ in 0..num_rooms {
            let w = rng.gen_range(4..10);
            let h = rng.gen_range(4..8);
            let x = rng.gen_range(1..MAP_WIDTH - w - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - h - 1);
            for ry in y..y + h {
                for rx in x..x + w {
                    tiles[ry * MAP_WIDTH + rx] = Tile::Floor {
                        id: floor_type.clone(),
                    };
                }
            }
            room_centers.push(((x + w / 2) as i32, (y + h / 2) as i32));
        }

        for i in 1..room_centers.len() {
            let (cx1, cy1) = room_centers[i - 1];
            let (cx2, cy2) = room_centers[i];
            for x in (cx1.min(cx2) as usize)..=(cx1.max(cx2) as usize) {
                tiles[cy1 as usize * MAP_WIDTH + x] = Tile::Floor {
                    id: floor_type.clone(),
                };
            }
            for y in (cy1.min(cy2) as usize)..=(cy1.max(cy2) as usize) {
                tiles[y * MAP_WIDTH + cx2 as usize] = Tile::Floor {
                    id: floor_type.clone(),
                };
            }
        }

        for _ in 0..rng.gen_range(10..20) {
            let x = rng.gen_range(1..MAP_WIDTH - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - 1);
            if matches!(tiles[y * MAP_WIDTH + x], Tile::Floor { .. }) {
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
                    lights.push(MapLight {
                        x: lx,
                        y: ly,
                        id: light_id,
                    });
                }
            }
        }

        (
            Self {
                tiles,
                width: MAP_WIDTH,
                height: MAP_HEIGHT,
                lights,
                features: Vec::new(),
                inscriptions: Vec::new(),
                area_description: None,
                metadata: std::collections::HashMap::new(),
            },
            room_centers,
        )
    }

    /// Generate a subterranean map (cave/dungeon)
    pub fn generate_subterranean(rng: &mut ChaCha8Rng, layer: i32) -> (Self, Vec<(i32, i32)>) {
        let wall_type = "shale".to_string();
        let wall_hp = get_wall_def(&wall_type).map(|d| d.hp).unwrap_or(10);
        let floor_type = "ancient_tile".to_string(); // Archive floors
        let mut tiles = vec![
            Tile::Wall {
                id: wall_type,
                hp: wall_hp
            };
            MAP_WIDTH * MAP_HEIGHT
        ];
        let mut room_centers = Vec::new();

        // More rooms deeper underground
        let num_rooms =
            rng.gen_range(4 + layer.unsigned_abs() as usize..8 + layer.unsigned_abs() as usize * 2);

        for _ in 0..num_rooms.min(12) {
            let w = rng.gen_range(3..8);
            let h = rng.gen_range(3..6);
            let x = rng.gen_range(1..MAP_WIDTH - w - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - h - 1);
            for ry in y..y + h {
                for rx in x..x + w {
                    tiles[ry * MAP_WIDTH + rx] = Tile::Floor {
                        id: floor_type.clone(),
                    };
                }
            }
            room_centers.push(((x + w / 2) as i32, (y + h / 2) as i32));
        }

        // Connect rooms with corridors
        for i in 1..room_centers.len() {
            let (cx1, cy1) = room_centers[i - 1];
            let (cx2, cy2) = room_centers[i];
            for x in (cx1.min(cx2) as usize)..=(cx1.max(cx2) as usize) {
                tiles[cy1 as usize * MAP_WIDTH + x] = Tile::Floor {
                    id: floor_type.clone(),
                };
            }
            for y in (cy1.min(cy2) as usize)..=(cy1.max(cy2) as usize) {
                tiles[y * MAP_WIDTH + cx2 as usize] = Tile::Floor {
                    id: floor_type.clone(),
                };
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
            lights.push(MapLight {
                x: rx,
                y: ry,
                id: "crystal".to_string(),
            });
        }

        (
            Self {
                tiles,
                width: MAP_WIDTH,
                height: MAP_HEIGHT,
                lights,
                features: Vec::new(),
                inscriptions: Vec::new(),
                area_description: None,
                metadata: std::collections::HashMap::new(),
            },
            room_centers,
        )
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Tile> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(&self.tiles[y as usize * self.width + x as usize])
        } else {
            None
        }
    }

    pub fn idx(&self, x: i32, y: i32) -> usize {
        y as usize * self.width + x as usize
    }

    /// Convert position to index, returning None if out of bounds
    pub fn pos_to_idx(&self, x: i32, y: i32) -> Option<usize> {
        if self.is_valid_position(x, y) {
            Some(self.idx(x, y))
        } else {
            None
        }
    }

    /// Convert index to position
    pub fn idx_to_pos(&self, idx: usize) -> Option<(i32, i32)> {
        if idx < self.tiles.len() {
            let x = (idx % self.width) as i32;
            let y = (idx / self.width) as i32;
            Some((x, y))
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

    /// Generate narrative content for the map
    fn generate_narrative_content(
        &mut self,
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        poi: POI,
    ) {
        // Generate area description
        self.area_description = self.generate_area_description(rng, biome, terrain, poi);

        // Place inscriptions and graffiti
        self.place_inscriptions(rng, biome, poi);
    }

    /// Generate contextual area description
    fn generate_area_description(
        &self,
        rng: &mut ChaCha8Rng,
        biome: Biome,
        terrain: Terrain,
        poi: POI,
    ) -> Option<String> {
        use super::generation::{NarrativeContext, NarrativeGenerator};

        if let Ok(generator) = NarrativeGenerator::new() {
            let biome_str = match biome {
                Biome::Desert => "desert",
                Biome::Saltflat => "saltflat",
                Biome::Scrubland => "scrubland",
                Biome::Oasis => "oasis",
                Biome::Ruins => "ruins",
            };

            let terrain_str = match terrain {
                Terrain::Flat => "flat",
                Terrain::Hills => "hills",
                Terrain::Dunes => "dunes",
                Terrain::Canyon => "canyon",
                Terrain::Mesa => "mesa",
            };

            let location_type = match poi {
                POI::Town => "town",
                POI::Shrine => "shrine",
                POI::Landmark => "ruins",
                POI::Dungeon => "archive",
                _ => "wilderness",
            };

            let context = NarrativeContext {
                biome: Some(biome_str.to_string()),
                terrain: Some(terrain_str.to_string()),
                adaptations: Vec::new(),
                faction_reputation: std::collections::HashMap::new(),
                refraction_level: 0,
                location_type: Some(location_type.to_string()),
            };

            generator.generate_contextual_description(&context, &mut rng.clone())
        } else {
            None
        }
    }

    /// Place inscriptions and graffiti on walls and glass
    fn place_inscriptions(&mut self, rng: &mut ChaCha8Rng, _biome: Biome, poi: POI) {
        use super::generation::NarrativeGenerator;

        if let Ok(generator) = NarrativeGenerator::new() {
            let inscription_count = match poi {
                POI::Town => rng.gen_range(2..5),
                POI::Shrine => rng.gen_range(3..6),
                POI::Landmark => rng.gen_range(1..4),
                POI::Dungeon => rng.gen_range(1..3),
                _ => rng.gen_range(0..2),
            };

            for _ in 0..inscription_count {
                if let Some((x, y)) = self.find_inscription_location(rng) {
                    let inscription_type = if matches!(poi, POI::Shrine) && rng.gen_bool(0.7) {
                        "shrine_text"
                    } else if rng.gen_bool(0.6) {
                        "inscription"
                    } else {
                        "graffiti"
                    };

                    if let Some(text) =
                        generator.generate_environmental_text(inscription_type, &mut rng.clone())
                    {
                        self.inscriptions.push(MapInscription {
                            x,
                            y,
                            text,
                            inscription_type: inscription_type.to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Find suitable location for inscription (wall or glass tile)
    fn find_inscription_location(&self, rng: &mut ChaCha8Rng) -> Option<(i32, i32)> {
        let mut candidates = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let tile = &self.tiles[y * self.width + x];
                if matches!(tile, Tile::Wall { .. } | Tile::Glass) {
                    // Check if there's a floor tile adjacent (visible to players)
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        if let Some(adj_tile) = self.get(x as i32 + dx, y as i32 + dy) {
                            if matches!(adj_tile, Tile::Floor { .. }) {
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

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        !self.tiles[idx].transparent()
    }
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
    fn dimensions(&self) -> Point {
        Point::new(self.width as i32, self.height as i32)
    }
}

pub fn compute_fov(map: &Map, x: i32, y: i32) -> HashSet<usize> {
    field_of_view(Point::new(x, y), FOV_RANGE, map)
        .into_iter()
        .map(|p| map.idx(p.x, p.y))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn generate_from_world_deterministic() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        let (map1, rooms1) =
            Map::generate_from_world(&mut rng1, Biome::Saltflat, Terrain::Canyon, 100);
        let (map2, rooms2) =
            Map::generate_from_world(&mut rng2, Biome::Saltflat, Terrain::Canyon, 100);
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
