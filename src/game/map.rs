use bracket_pathfinding::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::constants::{FOV_RANGE, MAP_HEIGHT, MAP_WIDTH};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Tile { Floor, Wall, Glass }

impl Tile {
    pub fn glyph(&self) -> char {
        match self { Tile::Floor => '.', Tile::Wall => '#', Tile::Glass => '*' }
    }
    pub fn walkable(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass) }
    pub fn transparent(&self) -> bool { matches!(self, Tile::Floor | Tile::Glass) }
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub width: usize,
    pub height: usize,
}

impl Map {
    pub fn generate(rng: &mut ChaCha8Rng) -> (Self, Vec<(i32, i32)>) {
        let mut tiles = vec![Tile::Wall; MAP_WIDTH * MAP_HEIGHT];
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

    pub fn get(&self, x: i32, y: i32) -> Option<Tile> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(self.tiles[y as usize * self.width + x as usize])
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
