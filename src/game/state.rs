use bracket_pathfinding::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::{
    adaptation::Adaptation,
    enemy::{BehaviorContext, Enemy},
    item::{get_item_def, Item},
    map::{compute_fov, Map, Tile},
    npc::Npc,
    spawn::{load_spawn_tables, weighted_pick},
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

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggeredEffect {
    pub effect: String,
    pub frames_remaining: u32,
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
    #[serde(default)]
    pub adaptations_hidden_turns: u32,
    #[serde(default)]
    pub triggered_effects: Vec<TriggeredEffect>,
    #[serde(skip)]
    pub enemy_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub npc_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub item_positions: HashMap<(i32, i32), Vec<usize>>,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (map, rooms) = Map::generate(&mut rng);
        let (px, py) = rooms[0];
        let visible = compute_fov(&map, px, py);
        let tables = load_spawn_tables();
        let table = &tables.default;

        // Spawn enemies
        let mut enemies = Vec::new();
        for &(rx, ry) in rooms.iter().skip(1).take(rooms.len().saturating_sub(3)) {
            if let Some(id) = weighted_pick(&table.enemies, &mut rng) {
                enemies.push(Enemy::new(rx, ry, id));
            }
        }

        // Spawn NPCs
        let mut npcs = Vec::new();
        let late_room = rooms.len().saturating_sub(2);
        for spawn in &table.npcs {
            let room_idx = match spawn.room.as_deref() {
                Some("late") => Some(late_room),
                Some("last") => Some(rooms.len() - 1),
                Some("first") => Some(0),
                _ => {
                    if rng.gen_ratio(spawn.weight.min(10), 10) {
                        Some(rng.gen_range(1..rooms.len()))
                    } else { None }
                }
            };
            if let Some(idx) = room_idx {
                if idx < rooms.len() {
                    let (rx, ry) = rooms[idx];
                    npcs.push(Npc::new(rx, ry, &spawn.id));
                }
            }
        }

        // Spawn items
        let mut items = Vec::new();
        let mut used_positions = HashSet::new();
        for spawn in &table.items {
            if let Some("last") = spawn.room.as_deref() {
                if let Some(&(rx, ry)) = rooms.last() {
                    if !used_positions.contains(&(rx, ry)) {
                        used_positions.insert((rx, ry));
                        items.push(Item::new(rx, ry, &spawn.id));
                    }
                }
                continue;
            }
            for _ in 0..spawn.weight {
                if let Some(&(rx, ry)) = rooms.get(rng.gen_range(1..rooms.len())) {
                    let ix = rx + rng.gen_range(-1..=1);
                    let iy = ry + rng.gen_range(-1..=1);
                    if !used_positions.contains(&(ix, iy)) {
                        used_positions.insert((ix, iy));
                        items.push(Item::new(ix, iy, &spawn.id));
                    }
                }
            }
        }

        let mut state = Self {
            player_x: px, player_y: py, player_hp: 20, player_max_hp: 20,
            map, enemies, npcs, items, inventory: Vec::new(),
            visible: visible.clone(), revealed: visible,
            messages: vec!["Welcome to the Saltglass Steppe.".into()],
            turn: 0, rng, storm: Storm::forecast(&mut ChaCha8Rng::seed_from_u64(seed + 1)),
            refraction: 0, adaptations: Vec::new(), adaptations_hidden_turns: 0,
            triggered_effects: Vec::new(),
            enemy_positions: HashMap::new(),
            npc_positions: HashMap::new(),
            item_positions: HashMap::new(),
        };
        state.rebuild_spatial_index();
        state
    }

    pub fn rebuild_spatial_index(&mut self) {
        self.enemy_positions.clear();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.hp > 0 {
                self.enemy_positions.insert((e.x, e.y), i);
            }
        }
        self.npc_positions.clear();
        for (i, n) in self.npcs.iter().enumerate() {
            self.npc_positions.insert((n.x, n.y), i);
        }
        self.item_positions.clear();
        for (i, item) in self.items.iter().enumerate() {
            self.item_positions.entry((item.x, item.y)).or_default().push(i);
        }
    }

    pub fn trigger_effect(&mut self, effect: &str, duration: u32) {
        self.triggered_effects.push(TriggeredEffect {
            effect: effect.to_string(),
            frames_remaining: duration,
        });
    }

    pub fn visible_adaptation_count(&self) -> usize {
        if self.adaptations_hidden_turns > 0 { 0 } else { self.adaptations.len() }
    }

    fn tick_turn(&mut self) {
        self.turn += 1;
        if self.adaptations_hidden_turns > 0 {
            self.adaptations_hidden_turns -= 1;
            if self.adaptations_hidden_turns == 0 {
                self.log("The tincture wears off. Your glow returns.");
            }
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
            if matches!(self.map.tiles[y * self.map.width + x], Tile::Wall { .. }) {
                self.map.tiles[y * self.map.width + x] = Tile::Glass;
            }
        }
        
        // Spawn storm enemies on glass tiles
        let glass_tiles: Vec<(i32, i32)> = (0..self.map.tiles.len())
            .filter(|&i| self.map.tiles[i] == Tile::Glass)
            .map(|i| ((i % self.map.width) as i32, (i / self.map.width) as i32))
            .filter(|&(x, y)| self.enemy_at(x, y).is_none() && !(x == self.player_x && y == self.player_y))
            .collect();
        if !glass_tiles.is_empty() {
            let spawn_count = (self.storm.intensity as usize).min(2);
            for _ in 0..spawn_count {
                let idx = self.rng.gen_range(0..glass_tiles.len());
                let (x, y) = glass_tiles[idx];
                let enemy_idx = self.enemies.len();
                self.enemies.push(Enemy::new(x, y, "refraction_wraith"));
                self.enemy_positions.insert((x, y), enemy_idx);
                self.log("A wraith coalesces from the storm's edge.");
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
        self.enemy_positions.get(&(x, y)).copied()
    }

    pub fn npc_at(&self, x: i32, y: i32) -> Option<usize> {
        self.npc_positions.get(&(x, y)).copied()
    }

    pub fn describe_at(&self, x: i32, y: i32) -> String {
        let idx = self.map.idx(x, y);
        if !self.visible.contains(&idx) && !self.revealed.contains(&idx) {
            return "Unknown".into();
        }
        if x == self.player_x && y == self.player_y {
            return "You".into();
        }
        if let Some(ei) = self.enemy_at(x, y) {
            let e = &self.enemies[ei];
            let desc = e.def().map(|d| d.description.as_str()).unwrap_or("A creature");
            return format!("{} (HP: {}) - {}", e.name(), e.hp, desc);
        }
        if let Some(ni) = self.npc_at(x, y) {
            let n = &self.npcs[ni];
            let desc = n.def().map(|d| d.description.as_str()).unwrap_or("A person");
            return format!("{} - {}", n.name(), desc);
        }
        if let Some(item) = self.items.iter().find(|i| i.x == x && i.y == y) {
            if let Some(def) = get_item_def(&item.id) {
                return format!("{} - {}", def.name, def.description);
            }
        }
        if let Some(tile) = self.map.get(x, y) {
            return format!("{} - {}", tile.name(), tile.description());
        }
        "Void".into()
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
        let inventory = self.inventory.clone();
        let adaptation_count = self.adaptations.len();
        
        for i in 0..self.enemies.len() {
            if self.enemies[i].hp <= 0 { continue; }
            let ex = self.enemies[i].x;
            let ey = self.enemies[i].y;
            let def = match self.enemies[i].def() {
                Some(d) => d,
                None => continue,
            };
            let sight = def.sight_range;
            let dist = (px - ex).abs() + (py - ey).abs();
            
            // Check behaviors
            let ctx = BehaviorContext {
                player_adaptations: adaptation_count,
                player_items: &inventory,
            };
            let mut is_passive = false;
            let mut should_flee = false;
            for behavior in &def.behaviors {
                match behavior.behavior_type.as_str() {
                    "passive_if" => {
                        if behavior.condition_met(&ctx) { is_passive = true; }
                    }
                    "flee_if" => {
                        if behavior.condition_met(&ctx) { should_flee = true; }
                    }
                    _ => {}
                }
            }
            
            if is_passive { continue; }
            
            if should_flee && dist < sight && dist > 1 {
                let dx = (ex - px).signum();
                let dy = (ey - py).signum();
                let nx = ex + dx;
                let ny = ey + dy;
                if self.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false) 
                    && self.enemy_at(nx, ny).is_none() {
                    self.enemy_positions.remove(&(ex, ey));
                    self.enemies[i].x = nx;
                    self.enemies[i].y = ny;
                    self.enemy_positions.insert((nx, ny), i);
                }
                continue;
            }
            
            if dist == 1 {
                let dmg = self.rng.gen_range(def.damage_min..=def.damage_max);
                self.player_hp -= dmg;
                let dir = self.direction_from(ex, ey);
                self.log(format!("{} {} attacks you for {} damage!", self.enemies[i].name(), dir, dmg));
                
                // Trigger on_hit effects
                for e in &def.effects {
                    if e.condition == "on_hit" {
                        self.trigger_effect(&e.effect, 20);
                    }
                }
                
                // Check on_hit behaviors
                for behavior in &def.behaviors {
                    if behavior.behavior_type == "on_hit_refraction" {
                        if let Some(val) = behavior.value {
                            self.refraction += val;
                            self.log(format!("Glass shards pierce you. (+{} Refraction)", val));
                            self.check_adaptation_threshold();
                        }
                    }
                }
                
                if self.player_hp <= 0 { return; }
            } else if dist < sight {
                let enemy_idx = self.map.idx(ex, ey);
                let (nx, ny) = if self.visible.contains(&enemy_idx) {
                    // Visible: use A* pathfinding
                    let path = a_star_search(enemy_idx, self.map.idx(px, py), &self.map);
                    if path.success && path.steps.len() > 1 {
                        let next = path.steps[1];
                        ((next % self.map.width) as i32, (next / self.map.width) as i32)
                    } else {
                        continue;
                    }
                } else {
                    // Not visible: simple directional movement toward player
                    let dx = (px - ex).signum();
                    let dy = (py - ey).signum();
                    (ex + dx, ey + dy)
                };
                
                if self.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false)
                    && self.enemy_at(nx, ny).is_none() 
                    && !(nx == px && ny == py) {
                    self.enemy_positions.remove(&(ex, ey));
                    self.enemies[i].x = nx;
                    self.enemies[i].y = ny;
                    self.enemy_positions.insert((nx, ny), i);
                }
            }
        }
    }

    pub fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let new_x = self.player_x + dx;
        let new_y = self.player_y + dy;

        // NPC interaction (bump to talk)
        if let Some(ni) = self.npc_at(new_x, new_y) {
            let visible_adaptations: Vec<Adaptation> = if self.adaptations_hidden_turns > 0 {
                Vec::new()
            } else {
                self.adaptations.clone()
            };
            let dialogue = self.npcs[ni].dialogue(&visible_adaptations);
            let name = self.npcs[ni].name().to_string();
            self.log(format!("{}: \"{}\"", name, dialogue));
            
            // Execute first available action
            let actions = self.npcs[ni].available_actions(&visible_adaptations);
            for action in actions {
                // Item exchange
                if let (Some(gives), Some(consumes)) = (&action.effect.gives_item, &action.effect.consumes) {
                    if let Some(idx) = self.inventory.iter().position(|id| id == consumes) {
                        self.inventory.remove(idx);
                        self.inventory.push(gives.clone());
                        let gives_name = get_item_def(gives).map(|d| d.name.as_str()).unwrap_or("item");
                        self.log(format!("The pilgrim presses {} into your hand.", gives_name));
                        break;
                    }
                }
                // Heal action
                if let Some(heal) = action.effect.heal {
                    let actual = heal.min(self.player_max_hp - self.player_hp);
                    self.player_hp += actual;
                    self.log(format!("You rest. (+{} HP)", actual));
                    break;
                }
            }
            
            self.npcs[ni].talked = true;
            return true;
        }

        if let Some(ei) = self.enemy_at(new_x, new_y) {
            let mut dmg = self.rng.gen_range(2..6);
            if self.has_adaptation(Adaptation::Sunveins) { dmg += 2; }
            self.enemies[ei].hp -= dmg;
            let name = self.enemies[ei].name().to_string();
            let dir = self.direction_from(new_x, new_y);
            
            // Trigger on_hit effects
            if let Some(def) = self.enemies[ei].def() {
                for e in &def.effects {
                    if e.condition == "on_hit" {
                        self.trigger_effect(&e.effect, 20);
                    }
                }
                // Damage reflection behavior
                for behavior in &def.behaviors {
                    if behavior.behavior_type == "reflect_damage" {
                        let percent = behavior.percent.unwrap_or(25);
                        let reflected = (dmg as u32 * percent / 100) as i32;
                        if reflected > 0 {
                            self.player_hp -= reflected;
                            self.log(format!("Light bends—your attack refracts back! (-{} HP)", reflected));
                        }
                    }
                }
            }
            
            if self.enemies[ei].hp <= 0 {
                // Remove from spatial index
                self.enemy_positions.remove(&(new_x, new_y));
                // Trigger on_death effects
                if let Some(def) = self.enemies[ei].def() {
                    for e in &def.effects {
                        if e.condition == "on_death" {
                            self.trigger_effect(&e.effect, 30);
                        }
                    }
                }
                self.log(format!("You kill the {} {}!", name, dir));
            } else {
                self.log(format!("You hit the {} {} for {} damage.", name, dir, dmg));
            }
            self.tick_turn();
            self.update_enemies();
            if self.storm.tick() { self.apply_storm(); }
            return true;
        }

        if let Some(tile) = self.map.get(new_x, new_y) {
            let walkable = tile.walkable();
            let is_glass = *tile == Tile::Glass;
            if walkable {
                self.player_x = new_x;
                self.player_y = new_y;
                self.tick_turn();
                self.visible = compute_fov(&self.map, new_x, new_y);
                self.revealed.extend(&self.visible);
                self.pickup_items();

                if is_glass {
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

    pub fn try_break_wall(&mut self, x: i32, y: i32) -> bool {
        // Check if player has glass_pick
        let pick_idx = self.inventory.iter().position(|id| {
            get_item_def(id).map(|d| d.breaks_walls).unwrap_or(false)
        });
        if pick_idx.is_none() {
            self.log("You need a tool to break walls.");
            return false;
        }
        // Check adjacency
        let dist = (x - self.player_x).abs() + (y - self.player_y).abs();
        if dist != 1 {
            self.log("Too far to break.");
            return false;
        }
        // Check if it's a wall
        let idx = self.map.idx(x, y);
        if let Tile::Wall { ref id, hp } = self.map.tiles[idx].clone() {
            let new_hp = hp - 5;
            if new_hp <= 0 {
                self.map.tiles[idx] = Tile::Floor;
                self.log("The wall crumbles!");
            } else {
                self.map.tiles[idx] = Tile::Wall { id: id.clone(), hp: new_hp };
                self.log(format!("Cracks spread through the wall. (HP: {})", new_hp));
            }
            self.tick_turn();
            self.update_enemies();
            if self.storm.tick() { self.apply_storm(); }
            return true;
        }
        self.log("Nothing to break there.");
        false
    }

    pub fn pickup_items(&mut self) {
        let px = self.player_x;
        let py = self.player_y;
        let indices = match self.item_positions.remove(&(px, py)) {
            Some(v) => v,
            None => return,
        };
        // Process in reverse order to maintain valid indices during removal
        for &i in indices.iter().rev() {
            let id = self.items[i].id.clone();
            let def = get_item_def(&id);
            let name = def.map(|d| d.name.as_str()).unwrap_or("item");
            if let Some(d) = def {
                for e in &d.effects {
                    if e.condition == "on_pickup" {
                        self.trigger_effect(&e.effect, 30);
                    }
                }
            }
            self.inventory.push(id);
            self.log(format!("Picked up {}.", name));
            self.items.remove(i);
        }
        // Rebuild item positions since indices shifted
        self.item_positions.clear();
        for (i, item) in self.items.iter().enumerate() {
            self.item_positions.entry((item.x, item.y)).or_default().push(i);
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
        if def.reduces_refraction > 0 {
            let reduce = def.reduces_refraction.min(self.refraction);
            self.refraction -= reduce;
            self.log(format!("Your glow fades slightly. (-{} Refraction)", reduce));
        }
        if def.suppresses_adaptations {
            self.adaptations_hidden_turns = 10;
            self.log("Your glow dims. The tincture masks your changes.");
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
        let mut state: Self = ron::from_str(&data).map_err(|e| e.to_string())?;
        state.rebuild_spatial_index();
        Ok(state)
    }
}
