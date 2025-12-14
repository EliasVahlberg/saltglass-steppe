use bracket_pathfinding::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::{
    adaptation::Adaptation,
    enemy::{all_enemy_ids, Enemy},
    item::{get_item_def, Item},
    map::{compute_fov, Map, Tile},
    npc::Npc,
    storm::Storm,
};

mod rng_serde {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct RngState([u8; 32]);

    pub fn serialize<S: Serializer>(rng: &ChaCha8Rng, s: S) -> Result<S::Ok, S::Error> {
        let bytes: [u8; 32] = rng.get_seed();
        RngState(bytes).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<ChaCha8Rng, D::Error> {
        let state = RngState::deserialize(d)?;
        Ok(ChaCha8Rng::from_seed(state.0))
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub player_x: i32, pub player_y: i32, pub player_hp: i32, pub player_max_hp: i32,
    pub map: Map, pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub inventory: Vec<String>,
    pub visible: HashSet<usize>, pub revealed: HashSet<usize>,
    pub messages: Vec<String>, pub turn: u32,
    #[serde(with = "rng_serde")]
    pub rng: ChaCha8Rng, pub storm: Storm,
    pub refraction: u32,
    pub adaptations: Vec<Adaptation>,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (map, rooms) = Map::generate(&mut rng);
        let (px, py) = rooms[0];
        let visible = compute_fov(&map, px, py);

        let enemy_ids = all_enemy_ids();
        let mut enemies = Vec::new();
        for &(rx, ry) in rooms.iter().skip(1).take(rooms.len().saturating_sub(3)) {
            let id = enemy_ids[rng.gen_range(0..enemy_ids.len())];
            enemies.push(Enemy::new(rx, ry, id));
        }

        // Spawn one NPC (Mirror Monk) in a later room
        let mut npcs = Vec::new();
        if rooms.len() > 3 {
            let npc_room = rooms[rooms.len() - 2];
            npcs.push(Npc::new(npc_room.0, npc_room.1, "mirror_monk"));
        }

        let spawn_items = ["storm_glass", "storm_glass", "storm_glass", "storm_glass",
                          "brine_vial", "brine_vial", "brine_vial",
                          "scripture_shard", "scripture_shard", "saint_key"];
        let mut items = Vec::new();
        let mut used_positions = std::collections::HashSet::new();
        for &(rx, ry) in rooms.iter().skip(1) {
            let ix = rx + rng.gen_range(-1..=1);
            let iy = ry + rng.gen_range(-1..=1);
            if !used_positions.contains(&(ix, iy)) {
                used_positions.insert((ix, iy));
                let id = spawn_items[rng.gen_range(0..spawn_items.len())];
                items.push(Item::new(ix, iy, id));
            }
        }
        if let Some(&(rx, ry)) = rooms.last() {
            if !used_positions.contains(&(rx, ry)) {
                items.push(Item::new(rx, ry, "angle_lens"));
            }
        }

        Self {
            player_x: px, player_y: py, player_hp: 20, player_max_hp: 20,
            map, enemies, npcs, items, inventory: Vec::new(),
            visible: visible.clone(), revealed: visible,
            messages: vec!["Welcome to the Saltglass Steppe.".into()],
            turn: 0, rng, storm: Storm::forecast(&mut ChaCha8Rng::seed_from_u64(seed + 1)),
            refraction: 0, adaptations: Vec::new(),
        }
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.messages.push(msg.into());
        if self.messages.len() > 5 { self.messages.remove(0); }
    }

    pub fn apply_storm(&mut self) {
        self.log(format!("⚡ GLASS STORM! Intensity {}", self.storm.intensity));
        self.refraction += self.storm.intensity as u32 * 10;
        self.check_adaptation_threshold();

        for _ in 0..(self.storm.intensity as usize * 5) {
            let x = self.rng.gen_range(1..self.map.width - 1);
            let y = self.rng.gen_range(1..self.map.height - 1);
            if self.map.tiles[y * self.map.width + x] == Tile::Wall {
                self.map.tiles[y * self.map.width + x] = Tile::Glass;
            }
        }
        self.storm = Storm::forecast(&mut self.rng);
        self.visible = compute_fov(&self.map, self.player_x, self.player_y);
    }

    pub fn check_adaptation_threshold(&mut self) {
        let thresholds = [25, 50, 75, 100];
        let adaptation_count = self.adaptations.len();
        if adaptation_count < thresholds.len() && self.refraction >= thresholds[adaptation_count] {
            let available: Vec<Adaptation> = [
                Adaptation::Prismhide, Adaptation::Sunveins,
                Adaptation::MirageStep, Adaptation::Saltblood,
            ].into_iter().filter(|a| !self.adaptations.contains(a)).collect();

            if !available.is_empty() {
                let idx = self.rng.gen_range(0..available.len());
                let adaptation = available[idx];
                self.adaptations.push(adaptation);
                self.log(format!("🧬 You gain {}!", adaptation.name()));
            }
        }
    }

    pub fn has_adaptation(&self, a: Adaptation) -> bool {
        self.adaptations.contains(&a)
    }

    pub fn enemy_at(&self, x: i32, y: i32) -> Option<usize> {
        self.enemies.iter().position(|e| e.x == x && e.y == y && e.hp > 0)
    }

    pub fn npc_at(&self, x: i32, y: i32) -> Option<usize> {
        self.npcs.iter().position(|n| n.x == x && n.y == y)
    }

    fn direction_from(&self, x: i32, y: i32) -> &'static str {
        let dx = x - self.player_x;
        let dy = y - self.player_y;
        match (dx.signum(), dy.signum()) {
            (0, -1) => "to the north",
            (0, 1) => "to the south",
            (-1, 0) => "to the west",
            (1, 0) => "to the east",
            (-1, -1) => "to the northwest",
            (1, -1) => "to the northeast",
            (-1, 1) => "to the southwest",
            (1, 1) => "to the southeast",
            _ => "nearby",
        }
    }

    pub fn update_enemies(&mut self) {
        if self.player_hp <= 0 { return; }
        let px = self.player_x;
        let py = self.player_y;
        for i in 0..self.enemies.len() {
            if self.enemies[i].hp <= 0 { continue; }
            let ex = self.enemies[i].x;
            let ey = self.enemies[i].y;
            let def = self.enemies[i].def();
            let sight = def.map(|d| d.sight_range).unwrap_or(6);
            let dist = (px - ex).abs() + (py - ey).abs();
            if dist == 1 {
                let (dmin, dmax) = def.map(|d| (d.damage_min, d.damage_max)).unwrap_or((1, 3));
                let dmg = self.rng.gen_range(dmin..=dmax);
                self.player_hp -= dmg;
                let dir = self.direction_from(ex, ey);
                self.log(format!("{} {} attacks you for {} damage!", self.enemies[i].name(), dir, dmg));
                if self.player_hp <= 0 { return; }
            } else if dist < sight && self.visible.contains(&self.map.idx(ex, ey)) {
                let path = a_star_search(self.map.idx(ex, ey), self.map.idx(px, py), &self.map);
                if path.success && path.steps.len() > 1 {
                    let next = path.steps[1];
                    let nx = (next % self.map.width) as i32;
                    let ny = (next / self.map.width) as i32;
                    if self.enemy_at(nx, ny).is_none() && !(nx == px && ny == py) {
                        self.enemies[i].x = nx;
                        self.enemies[i].y = ny;
                    }
                }
            }
        }
    }

    pub fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let new_x = self.player_x + dx;
        let new_y = self.player_y + dy;

        // NPC interaction (bump to talk)
        if let Some(ni) = self.npc_at(new_x, new_y) {
            let dialogue = self.npcs[ni].dialogue(&self.adaptations);
            let name = self.npcs[ni].name();
            self.log(format!("{}: \"{}\"", name, dialogue));
            self.npcs[ni].talked = true;
            return true;
        }

        if let Some(ei) = self.enemy_at(new_x, new_y) {
            let mut dmg = self.rng.gen_range(2..6);
            if self.has_adaptation(Adaptation::Sunveins) { dmg += 2; }
            self.enemies[ei].hp -= dmg;
            let name = self.enemies[ei].name().to_string();
            let dir = self.direction_from(new_x, new_y);
            if self.enemies[ei].hp <= 0 {
                self.log(format!("You kill the {} {}!", name, dir));
            } else {
                self.log(format!("You hit the {} {} for {} damage.", name, dir, dmg));
            }
            self.turn += 1;
            self.update_enemies();
            if self.storm.tick() { self.apply_storm(); }
            return true;
        }

        if let Some(tile) = self.map.get(new_x, new_y) {
            if tile.walkable() {
                self.player_x = new_x;
                self.player_y = new_y;
                self.turn += 1;
                self.visible = compute_fov(&self.map, new_x, new_y);
                self.revealed.extend(&self.visible);
                self.pickup_items();

                if tile == Tile::Glass {
                    if self.has_adaptation(Adaptation::Saltblood) {
                        self.log("Your saltblood protects you from the glass.");
                    } else {
                        self.player_hp -= 1;
                        self.refraction += 1;
                        self.log("Sharp glass cuts you! (-1 HP, +1 Refraction)");
                        self.check_adaptation_threshold();
                    }
                }

                self.update_enemies();
                if self.storm.tick() { self.apply_storm(); }
                return true;
            }
        }
        false
    }

    pub fn pickup_items(&mut self) {
        let px = self.player_x;
        let py = self.player_y;
        let mut picked: Vec<(usize, String)> = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            if item.x == px && item.y == py {
                picked.push((i, item.id.clone()));
            }
        }
        for (i, id) in picked.iter().rev() {
            let name = get_item_def(id).map(|d| d.name.as_str()).unwrap_or("item");
            self.inventory.push(id.clone());
            self.log(format!("Picked up {}.", name));
            self.items.remove(*i);
        }
    }

    pub fn use_item(&mut self, idx: usize) -> bool {
        if idx >= self.inventory.len() { return false; }
        let id = &self.inventory[idx];
        let def = match get_item_def(id) {
            Some(d) => d,
            None => return false,
        };
        if !def.usable {
            self.log(format!("You can't use {} right now.", def.name));
            return false;
        }
        if def.heal > 0 {
            let heal = def.heal.min(self.player_max_hp - self.player_hp);
            self.player_hp += heal;
            self.log(format!("You use {}. (+{} HP)", def.name, heal));
        }
        if def.reveals_map {
            self.log(format!("The {} reveals hidden paths...", def.name));
            for idx in 0..self.map.tiles.len() {
                self.revealed.insert(idx);
            }
        }
        self.inventory.remove(idx);
        true
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let data = ron::to_string(self).map_err(|e| e.to_string())?;
        fs::write(path, data).map_err(|e| e.to_string())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&data).map_err(|e| e.to_string())
    }
}
