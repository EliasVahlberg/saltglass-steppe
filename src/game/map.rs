use bracket_pathfinding::prelude::*;
use once_cell::sync::Lazy;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::constants::{FOV_RANGE, MAP_HEIGHT, MAP_WIDTH};
use super::light_defs::{get_spawn_rule, pick_light_type};
use super::world_map::{Biome, Terrain};

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

static WALL_DEFS: Lazy<HashMap<String, WallDef>> = Lazy::new(|| {
    let data = include_str!("../../data/walls.json");
    let file: WallsFile = serde_json::from_str(data).expect("Failed to parse walls.json");
    file.walls.into_iter().map(|d| (d.id.clone(), d)).collect()
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
            Tile::StairsDown => '>',
            Tile::StairsUp => '<',
            Tile::WorldExit => 'O',
        }
    }
    pub fn walkable(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass | Tile::StairsDown | Tile::StairsUp | Tile::WorldExit) }
    pub fn transparent(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass | Tile::StairsDown | Tile::StairsUp | Tile::WorldExit) }

    pub fn name(&self) -> &str {
        match self {
            Tile::Floor => "Floor",
            Tile::Wall { id, .. } => get_wall_def(id).map(|d| d.name.as_str()).unwrap_or("Wall"),
            Tile::Glass => "Glass",
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
        poi: super::world_map::POI,
    ) -> (Self, Vec<(i32, i32)>) {
        // Wall type based on biome
        let wall_type = match biome {
            Biome::Saltflat => "salt_crystal",
            Biome::Ruins => "shale",
            _ => "sandstone",
        }.to_string();
        let wall_hp = get_wall_def(&wall_type).map(|d| d.hp).unwrap_or(10);
        let mut tiles = vec![Tile::Wall { id: wall_type, hp: wall_hp }; MAP_WIDTH * MAP_HEIGHT];
        let mut room_centers = Vec::new();

        // Room count based on terrain
        let (min_rooms, max_rooms) = match terrain {
            Terrain::Canyon => (3, 5),
            Terrain::Mesa => (4, 6),
            Terrain::Hills => (5, 8),
            Terrain::Dunes => (4, 7),
            Terrain::Flat => (6, 10),
        };
        let num_rooms = rng.gen_range(min_rooms..=max_rooms);

        // Glass density based on biome
        let glass_density = match biome {
            Biome::Saltflat => (20, 35),
            Biome::Oasis => (5, 10),
            _ => (10, 20),
        };

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

        for _ in 0..rng.gen_range(glass_density.0..glass_density.1) {
            let x = rng.gen_range(1..MAP_WIDTH - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - 1);
            if tiles[y * MAP_WIDTH + x] == Tile::Floor {
                tiles[y * MAP_WIDTH + x] = Tile::Glass;
            }
        }

        // Add stairs down for dungeon POI
        if poi == super::world_map::POI::Dungeon {
            if let Some(&(rx, ry)) = room_centers.last() {
                tiles[ry as usize * MAP_WIDTH + rx as usize] = Tile::StairsDown;
            }
        }

        // Add WorldExit tiles at map edges (one per edge, connected to nearest room)
        // North edge
        if let Some(&(rx, _)) = room_centers.iter().min_by_key(|(_, y)| *y) {
            let exit_x = rx as usize;
            for y in 0..MAP_HEIGHT {
                if tiles[y * MAP_WIDTH + exit_x] == Tile::Floor {
                    // Carve path to edge
                    for ey in 0..y {
                        tiles[ey * MAP_WIDTH + exit_x] = Tile::Floor;
                    }
                    tiles[exit_x] = Tile::WorldExit; // y=0
                    break;
                }
            }
        }
        // South edge
        if let Some(&(rx, _)) = room_centers.iter().max_by_key(|(_, y)| *y) {
            let exit_x = rx as usize;
            for y in (0..MAP_HEIGHT).rev() {
                if tiles[y * MAP_WIDTH + exit_x] == Tile::Floor {
                    for ey in (y + 1)..MAP_HEIGHT {
                        tiles[ey * MAP_WIDTH + exit_x] = Tile::Floor;
                    }
                    tiles[(MAP_HEIGHT - 1) * MAP_WIDTH + exit_x] = Tile::WorldExit;
                    break;
                }
            }
        }
        // West edge
        if let Some(&(_, ry)) = room_centers.iter().min_by_key(|(x, _)| *x) {
            let exit_y = ry as usize;
            for x in 0..MAP_WIDTH {
                if tiles[exit_y * MAP_WIDTH + x] == Tile::Floor {
                    for ex in 0..x {
                        tiles[exit_y * MAP_WIDTH + ex] = Tile::Floor;
                    }
                    tiles[exit_y * MAP_WIDTH] = Tile::WorldExit; // x=0
                    break;
                }
            }
        }
        // East edge
        if let Some(&(_, ry)) = room_centers.iter().max_by_key(|(x, _)| *x) {
            let exit_y = ry as usize;
            for x in (0..MAP_WIDTH).rev() {
                if tiles[exit_y * MAP_WIDTH + x] == Tile::Floor {
                    for ex in (x + 1)..MAP_WIDTH {
                        tiles[exit_y * MAP_WIDTH + ex] = Tile::Floor;
                    }
                    tiles[exit_y * MAP_WIDTH + MAP_WIDTH - 1] = Tile::WorldExit;
                    break;
                }
            }
        }

        // Spawn lights based on biome
        let biome_key = match biome {
            Biome::Saltflat => "saltflat",
            Biome::Ruins => "ruins",
            _ => "default",
        };
        let rule = get_spawn_rule(biome_key);
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
