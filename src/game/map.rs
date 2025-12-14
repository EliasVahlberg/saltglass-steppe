use bracket_pathfinding::prelude::*;
use once_cell::sync::Lazy;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::constants::{FOV_RANGE, MAP_HEIGHT, MAP_WIDTH};

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
}

impl Tile {
    pub fn glyph(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall { .. } => '#',
            Tile::Glass => '*',
        }
    }
    pub fn walkable(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass) }
    pub fn transparent(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass) }

    pub fn name(&self) -> &str {
        match self {
            Tile::Floor => "Floor",
            Tile::Wall { id, .. } => get_wall_def(id).map(|d| d.name.as_str()).unwrap_or("Wall"),
            Tile::Glass => "Glass",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Tile::Floor => "Dusty ground",
            Tile::Wall { id, .. } => get_wall_def(id).map(|d| d.description.as_str()).unwrap_or("Solid wall"),
            Tile::Glass => "Sharp refractive shards, dangerous to walk on",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub width: usize,
    pub height: usize,
}

impl Map {
    pub fn generate(rng: &mut ChaCha8Rng) -> (Self, Vec<(i32, i32)>) {
        let wall_type = random_wall_type(rng);
        let wall_hp = get_wall_def(&wall_type).map(|d| d.hp).unwrap_or(10);
        let mut tiles = vec![Tile::Wall { id: wall_type.clone(), hp: wall_hp }; MAP_WIDTH * MAP_HEIGHT];
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

        (Self { tiles, width: MAP_WIDTH, height: MAP_HEIGHT }, room_centers)
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
