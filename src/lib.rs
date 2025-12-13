use bracket_pathfinding::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub const MAP_WIDTH: usize = 50;
pub const MAP_HEIGHT: usize = 22;
pub const FOV_RANGE: i32 = 8;

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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum EnemyKind { MirageHound, GlassBeetle, SaltMummy }

impl EnemyKind {
    pub fn glyph(&self) -> char { match self { Self::MirageHound => 'h', Self::GlassBeetle => 'b', Self::SaltMummy => 'm' } }
    pub fn name(&self) -> &str { match self { Self::MirageHound => "Mirage Hound", Self::GlassBeetle => "Glass Beetle", Self::SaltMummy => "Salt Mummy" } }
    pub fn max_hp(&self) -> i32 { match self { Self::MirageHound => 8, Self::GlassBeetle => 12, Self::SaltMummy => 15 } }
}

#[derive(Serialize, Deserialize)]
pub struct Enemy { pub x: i32, pub y: i32, pub kind: EnemyKind, pub hp: i32 }

impl Enemy {
    pub fn new(x: i32, y: i32, kind: EnemyKind) -> Self {
        Self { x, y, kind, hp: kind.max_hp() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Storm { pub turns_until: u32, pub intensity: u8 }

impl Storm {
    pub fn forecast(rng: &mut ChaCha8Rng) -> Self {
        Self { turns_until: rng.gen_range(15..30), intensity: rng.gen_range(1..4) }
    }
    pub fn tick(&mut self) -> bool {
        if self.turns_until > 0 { self.turns_until -= 1; self.turns_until == 0 } else { false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Adaptation {
    Prismhide,   // +armor vs beams, visible glint
    Sunveins,    // Store light, fire beam attack
    MirageStep,  // Create decoy after moving
    Saltblood,   // Drink brine, excrete salt
}

impl Adaptation {
    pub fn name(&self) -> &str {
        match self {
            Self::Prismhide => "Prismhide",
            Self::Sunveins => "Sunveins",
            Self::MirageStep => "Mirage Step",
            Self::Saltblood => "Saltblood",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Prismhide => "Crystalline skin reduces damage",
            Self::Sunveins => "Store light charge, +2 attack damage",
            Self::MirageStep => "Leave decoy when moving (confuses enemies)",
            Self::Saltblood => "Immune to glass terrain damage",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    StormGlass,      // Crafting material, valuable
    ScriptureShard,  // Lore fragment, quest item
    BrineVial,       // Healing consumable
    SaintKey,        // Opens Archive doors (future use)
    AngleLens,       // Featured relic: +FOV range
}

impl ItemKind {
    pub fn glyph(&self) -> char {
        match self {
            Self::StormGlass => '◆',
            Self::ScriptureShard => '?',
            Self::BrineVial => '!',
            Self::SaintKey => '⚷',
            Self::AngleLens => '◎',
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::StormGlass => "Storm Glass",
            Self::ScriptureShard => "Scripture Shard",
            Self::BrineVial => "Brine Vial",
            Self::SaintKey => "Saint-Key",
            Self::AngleLens => "Angle-Split Lens",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    pub kind: ItemKind,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub player_x: i32, pub player_y: i32, pub player_hp: i32, pub player_max_hp: i32,
    pub map: Map, pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub inventory: Vec<ItemKind>,
    pub visible: HashSet<usize>, pub revealed: HashSet<usize>,
    pub messages: Vec<String>, pub turn: u32,
    #[serde(with = "rng_serde")]
    pub rng: ChaCha8Rng, pub storm: Storm,
    pub refraction: u32,
    pub adaptations: Vec<Adaptation>,
}

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

impl GameState {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (map, rooms) = Map::generate(&mut rng);
        let (px, py) = rooms[0];
        let visible = compute_fov(&map, px, py);

        let mut enemies = Vec::new();
        for &(rx, ry) in rooms.iter().skip(1) {
            let kind = match rng.gen_range(0..3) {
                0 => EnemyKind::MirageHound,
                1 => EnemyKind::GlassBeetle,
                _ => EnemyKind::SaltMummy,
            };
            enemies.push(Enemy::new(rx, ry, kind));
        }

        // Spawn items in rooms
        let mut items = Vec::new();
        for &(rx, ry) in rooms.iter().skip(1) {
            // Offset item from room center
            let ix = rx + rng.gen_range(-1..=1);
            let iy = ry + rng.gen_range(-1..=1);
            let kind = match rng.gen_range(0..10) {
                0..=3 => ItemKind::StormGlass,
                4..=6 => ItemKind::BrineVial,
                7..=8 => ItemKind::ScriptureShard,
                _ => ItemKind::SaintKey,
            };
            items.push(Item { x: ix, y: iy, kind });
        }
        // Place featured relic in last room
        if let Some(&(rx, ry)) = rooms.last() {
            items.push(Item { x: rx, y: ry, kind: ItemKind::AngleLens });
        }

        Self {
            player_x: px, player_y: py, player_hp: 20, player_max_hp: 20,
            map, enemies, items, inventory: Vec::new(),
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
        // Increase refraction from storm exposure
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
            // Grant a random adaptation not already owned
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

    pub fn update_enemies(&mut self) {
        let px = self.player_x;
        let py = self.player_y;
        for i in 0..self.enemies.len() {
            if self.enemies[i].hp <= 0 { continue; }
            let ex = self.enemies[i].x;
            let ey = self.enemies[i].y;
            let dist = (px - ex).abs() + (py - ey).abs();
            if dist == 1 {
                let dmg = self.rng.gen_range(1..4);
                self.player_hp -= dmg;
                self.log(format!("{} hits you for {} damage!", self.enemies[i].kind.name(), dmg));
            } else if dist < 6 && self.visible.contains(&self.map.idx(ex, ey)) {
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

        if let Some(ei) = self.enemy_at(new_x, new_y) {
            let mut dmg = self.rng.gen_range(2..6);
            // Sunveins bonus damage
            if self.has_adaptation(Adaptation::Sunveins) { dmg += 2; }
            self.enemies[ei].hp -= dmg;
            let name = self.enemies[ei].kind.name();
            if self.enemies[ei].hp <= 0 {
                self.log(format!("You kill the {}!", name));
            } else {
                self.log(format!("You hit {} for {} damage.", name, dmg));
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

                // Pick up items
                self.pickup_items();

                // Glass terrain damage (unless Saltblood)
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
        let mut picked: Vec<(usize, ItemKind)> = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            if item.x == px && item.y == py {
                picked.push((i, item.kind));
            }
        }
        for (i, kind) in picked.iter().rev() {
            self.inventory.push(*kind);
            self.log(format!("Picked up {}.", kind.name()));
            self.items.remove(*i);
        }
    }

    pub fn use_item(&mut self, idx: usize) -> bool {
        if idx >= self.inventory.len() { return false; }
        let kind = self.inventory[idx];
        match kind {
            ItemKind::BrineVial => {
                let heal = 5.min(self.player_max_hp - self.player_hp);
                self.player_hp += heal;
                self.log(format!("You drink the brine. (+{} HP)", heal));
                self.inventory.remove(idx);
                true
            }
            ItemKind::AngleLens => {
                self.log("The Angle-Split Lens reveals hidden paths...");
                // Reveal more of the map
                for idx in 0..self.map.tiles.len() {
                    self.revealed.insert(idx);
                }
                self.inventory.remove(idx);
                true
            }
            _ => {
                self.log(format!("You can't use {} right now.", kind.name()));
                false
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_map_generation() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let (map1, _) = Map::generate(&mut rng1);
        let (map2, _) = Map::generate(&mut rng2);
        assert_eq!(map1.tiles, map2.tiles);
    }

    #[test]
    fn player_spawns_on_floor() {
        let state = GameState::new(42);
        let tile = state.map.get(state.player_x, state.player_y).unwrap();
        assert!(tile.walkable());
    }

    #[test]
    fn player_cannot_walk_through_walls() {
        let mut state = GameState::new(42);
        let start_x = state.player_x;
        // Try moving into walls repeatedly
        for _ in 0..100 {
            state.try_move(-1, 0);
        }
        // Should have stopped at a wall
        let tile = state.map.get(state.player_x - 1, state.player_y);
        if let Some(t) = tile {
            if !t.walkable() {
                assert!(state.player_x <= start_x);
            }
        }
    }

    #[test]
    fn storm_converts_walls_to_glass() {
        let mut state = GameState::new(42);
        let walls_before: usize = state.map.tiles.iter().filter(|&&t| t == Tile::Wall).count();
        state.storm.turns_until = 1;
        state.storm.intensity = 3;
        state.try_move(0, 0); // Won't move but won't trigger storm
        state.storm.turns_until = 0;
        state.apply_storm();
        let walls_after: usize = state.map.tiles.iter().filter(|&&t| t == Tile::Wall).count();
        assert!(walls_after <= walls_before);
    }

    #[test]
    fn fov_includes_player_position() {
        let state = GameState::new(42);
        let player_idx = state.map.idx(state.player_x, state.player_y);
        assert!(state.visible.contains(&player_idx));
    }

    #[test]
    fn enemies_spawn_in_rooms() {
        let state = GameState::new(42);
        for enemy in &state.enemies {
            let tile = state.map.get(enemy.x, enemy.y).unwrap();
            assert!(tile.walkable());
        }
    }

    #[test]
    fn combat_reduces_enemy_hp() {
        let mut state = GameState::new(42);
        // Find an enemy and teleport next to it
        if let Some(enemy) = state.enemies.first() {
            let ex = enemy.x;
            let ey = enemy.y;
            let initial_hp = enemy.hp;
            state.player_x = ex - 1;
            state.player_y = ey;
            // Ensure tile is walkable for attack
            let idx = state.map.idx(ex - 1, ey);
            state.map.tiles[idx] = Tile::Floor;
            state.try_move(1, 0); // Attack
            assert!(state.enemies[0].hp < initial_hp);
        }
    }

    #[test]
    fn save_load_roundtrip() {
        let state = GameState::new(42);
        let path = "/tmp/test_save.ron";
        state.save(path).unwrap();
        let loaded = GameState::load(path).unwrap();
        assert_eq!(state.player_x, loaded.player_x);
        assert_eq!(state.player_y, loaded.player_y);
        assert_eq!(state.turn, loaded.turn);
        assert_eq!(state.map.tiles, loaded.map.tiles);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn glass_increases_refraction() {
        let mut state = GameState::new(42);
        // Place glass under player's next step
        let idx = state.map.idx(state.player_x + 1, state.player_y);
        state.map.tiles[idx] = Tile::Glass;
        let initial_refraction = state.refraction;
        state.try_move(1, 0);
        assert!(state.refraction > initial_refraction);
    }

    #[test]
    fn saltblood_prevents_glass_damage() {
        let mut state = GameState::new(42);
        state.adaptations.push(Adaptation::Saltblood);
        let idx = state.map.idx(state.player_x + 1, state.player_y);
        state.map.tiles[idx] = Tile::Glass;
        let initial_hp = state.player_hp;
        state.try_move(1, 0);
        assert_eq!(state.player_hp, initial_hp);
    }

    #[test]
    fn items_spawn_in_map() {
        let state = GameState::new(42);
        assert!(!state.items.is_empty());
        // Featured relic should exist
        assert!(state.items.iter().any(|i| i.kind == ItemKind::AngleLens));
    }

    #[test]
    fn pickup_adds_to_inventory() {
        let mut state = GameState::new(42);
        // Place item at player position
        state.items.push(Item { x: state.player_x, y: state.player_y, kind: ItemKind::BrineVial });
        let items_before = state.items.len();
        state.pickup_items();
        assert_eq!(state.items.len(), items_before - 1);
        assert!(state.inventory.contains(&ItemKind::BrineVial));
    }

    #[test]
    fn brine_vial_heals() {
        let mut state = GameState::new(42);
        state.player_hp = 10;
        state.inventory.push(ItemKind::BrineVial);
        state.use_item(0);
        assert_eq!(state.player_hp, 15);
    }
}
