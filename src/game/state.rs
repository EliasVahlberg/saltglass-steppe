use bracket_pathfinding::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::{
    action::{action_cost, default_player_ap},
    adaptation::Adaptation,
    enemy::Enemy,
    equipment::{EquipSlot, Equipment},
    event::GameEvent,
    item::{get_item_def, Item},
    lighting::{compute_lighting, LightMap, LightSource},
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
    pub turns_remaining: u32,
}

fn default_ambient_light() -> u8 { 100 }

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub player_x: i32, pub player_y: i32, pub player_hp: i32, pub player_max_hp: i32,
    #[serde(default = "default_player_ap")]
    pub player_ap: i32,
    #[serde(default = "default_player_ap")]
    pub player_max_ap: i32,
    #[serde(default)]
    pub player_reflex: i32,
    #[serde(default)]
    pub player_armor: i32,
    #[serde(default)]
    pub equipped_weapon: Option<String>,
    #[serde(default)]
    pub equipment: Equipment,
    #[serde(default)]
    pub status_effects: Vec<super::status::StatusEffect>,
    pub map: Map, pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub inventory: Vec<String>,
    pub visible: HashSet<usize>, pub revealed: HashSet<usize>,
    #[serde(skip)]
    pub light_map: LightMap,
    #[serde(default = "default_ambient_light")]
    pub ambient_light: u8,
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
    #[serde(skip)]
    pub event_queue: Vec<GameEvent>,
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

        let ambient = 100u8;
        let light_sources = vec![LightSource { x: px, y: py, radius: 8, intensity: 150 }];
        let light_map = compute_lighting(&light_sources, ambient);

        let mut state = Self {
            player_x: px, player_y: py, player_hp: 20, player_max_hp: 20,
            player_ap: default_player_ap(), player_max_ap: default_player_ap(),
            player_reflex: 5, player_armor: 0, equipped_weapon: None,
            equipment: Equipment::default(),
            status_effects: Vec::new(),
            map, enemies, npcs, items, inventory: Vec::new(),
            visible: visible.clone(), revealed: visible,
            light_map, ambient_light: ambient,
            messages: vec!["Welcome to the Saltglass Steppe.".into()],
            turn: 0, rng, storm: Storm::forecast(&mut ChaCha8Rng::seed_from_u64(seed + 1)),
            refraction: 0, adaptations: Vec::new(), adaptations_hidden_turns: 0,
            triggered_effects: Vec::new(),
            enemy_positions: HashMap::new(),
            npc_positions: HashMap::new(),
            item_positions: HashMap::new(),
            event_queue: Vec::new(),
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

    pub fn update_lighting(&mut self) {
        // Player always has a torch
        let sources = vec![LightSource { x: self.player_x, y: self.player_y, radius: 8, intensity: 150 }];
        self.light_map = compute_lighting(&sources, self.ambient_light);
    }

    pub fn get_light_level(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 { return 0; }
        let idx = y as usize * self.map.width + x as usize;
        self.light_map.get(idx).copied().unwrap_or(0)
    }

    pub fn trigger_effect(&mut self, effect: &str, duration: u32) {
        self.triggered_effects.push(TriggeredEffect {
            effect: effect.to_string(),
            turns_remaining: duration,
        });
    }

    pub fn emit(&mut self, event: GameEvent) {
        self.event_queue.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.event_queue)
    }

    pub fn visible_adaptation_count(&self) -> usize {
        if self.adaptations_hidden_turns > 0 { 0 } else { self.adaptations.len() }
    }

    /// End turn: reset AP, tick status effects, run enemy turns, tick storm
    pub fn end_turn(&mut self) {
        self.player_ap = self.player_max_ap;
        self.tick_status_effects();
        self.tick_turn();
        self.update_enemies();
        if self.storm.tick() { self.apply_storm(); }
    }

    /// Tick all status effects, apply damage, remove expired
    fn tick_status_effects(&mut self) {
        let mut total_damage = 0;
        let mut messages = Vec::new();
        for effect in &mut self.status_effects {
            let dmg = effect.tick();
            if dmg > 0 {
                total_damage += dmg;
                messages.push(format!("{} deals {} damage.", effect.name(), dmg));
            }
        }
        for msg in messages {
            self.log(msg);
        }
        self.player_hp -= total_damage;
        self.status_effects.retain(|e| !e.is_expired());
    }

    /// Apply a status effect to the player
    pub fn apply_status(&mut self, effect: super::status::StatusEffect) {
        self.log(format!("You are {}! ({} turns)", effect.name(), effect.duration));
        self.status_effects.push(effect);
    }

    /// Wait in place (costs 0 AP, ends turn)
    pub fn wait_turn(&mut self) {
        self.end_turn();
    }

    /// Auto-end turn if player has no AP left
    pub(crate) fn check_auto_end_turn(&mut self) {
        if self.player_ap <= 0 {
            self.end_turn();
        }
    }

    fn tick_turn(&mut self) {
        self.turn += 1;
        if self.adaptations_hidden_turns > 0 {
            self.adaptations_hidden_turns -= 1;
            if self.adaptations_hidden_turns == 0 {
                self.log("The tincture wears off. Your glow returns.");
            }
        }
        // Tick down triggered effects
        self.triggered_effects.retain_mut(|e| {
            e.turns_remaining = e.turns_remaining.saturating_sub(1);
            e.turns_remaining > 0
        });
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
        self.update_lighting();
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
                self.emit(GameEvent::AdaptationGained { name: adaptation.name().to_string() });
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

    /// Auto-explore: find nearest unexplored walkable tile and move toward it
    pub fn auto_explore(&mut self) -> bool {
        let start = self.map.idx(self.player_x, self.player_y);
        
        // BFS to find nearest unexplored walkable tile
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, vec![start]));
        visited.insert(start);
        
        let target = loop {
            let (idx, path) = match queue.pop_front() {
                Some(p) => p,
                None => return false, // No unexplored tiles reachable
            };
            
            // Check if this tile is unexplored
            if !self.revealed.contains(&idx) {
                // Return the first step toward this tile
                if path.len() > 1 {
                    break Some(path[1]);
                }
                return false;
            }
            
            // Add neighbors
            for (next_idx, _) in self.map.get_available_exits(idx) {
                if !visited.contains(&next_idx) {
                    visited.insert(next_idx);
                    let mut new_path = path.clone();
                    new_path.push(next_idx);
                    queue.push_back((next_idx, new_path));
                }
            }
        };
        
        if let Some(next_idx) = target {
            let nx = (next_idx % self.map.width) as i32;
            let ny = (next_idx / self.map.width) as i32;
            let dx = nx - self.player_x;
            let dy = ny - self.player_y;
            self.try_move(dx, dy)
        } else {
            false
        }
    }

    pub fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let new_x = self.player_x + dx;
        let new_y = self.player_y + dy;

        // NPC interaction (bump to talk)
        if let Some(ni) = self.npc_at(new_x, new_y) {
            let cost = action_cost("interact");
            if self.player_ap < cost { return false; }
            self.player_ap -= cost;
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

        if self.enemy_at(new_x, new_y).is_some() {
            let hit = self.attack_melee(new_x, new_y);
            if hit { self.check_auto_end_turn(); }
            return hit;
        }

        if let Some(tile) = self.map.get(new_x, new_y) {
            let walkable = tile.walkable();
            let is_glass = *tile == Tile::Glass;
            if walkable {
                let cost = action_cost("move");
                if self.player_ap < cost { return false; }
                self.player_ap -= cost;
                self.player_x = new_x;
                self.player_y = new_y;
                self.visible = compute_fov(&self.map, new_x, new_y);
                self.update_lighting();
                self.revealed.extend(&self.visible);
                self.pickup_items();

                if is_glass {
                    if self.adaptations.iter().any(|a| a.has_immunity("glass")) {
                        self.log("Your saltblood protects you from the glass.");
                    } else {
                        self.player_hp -= 1;
                        self.refraction += 1;
                        self.log("Sharp glass cuts you! (-1 HP, +1 Refraction)");
                        self.check_adaptation_threshold();
                    }
                }

                self.check_auto_end_turn();
                return true;
            }
        }
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
                        self.trigger_effect(&e.effect, 3);
                    }
                }
            }
            self.inventory.push(id.clone());
            self.emit(GameEvent::ItemPickedUp { item_id: id });
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
        let cost = action_cost("use_item");
        if self.player_ap < cost { return false; }
        let id = &self.inventory[idx];
        let def = match get_item_def(id) {
            Some(d) => d,
            None => return false,
        };
        if !def.usable {
            self.log(format!("You can't use {} right now.", def.name));
            return false;
        }
        self.player_ap -= cost;
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

    /// Equip an item from inventory to a slot
    pub fn equip_item(&mut self, inv_idx: usize, slot: EquipSlot) -> bool {
        if inv_idx >= self.inventory.len() { return false; }
        let item_id = self.inventory[inv_idx].clone();
        
        // Unequip current item in slot (returns to inventory)
        if let Some(old) = self.equipment.set(slot, Some(item_id)) {
            self.inventory.push(old);
        }
        self.inventory.remove(inv_idx);
        self.recalc_equipment_stats();
        true
    }

    /// Unequip item from slot back to inventory
    pub fn unequip_slot(&mut self, slot: EquipSlot) -> bool {
        if let Some(item) = self.equipment.set(slot, None) {
            self.inventory.push(item);
            self.recalc_equipment_stats();
            true
        } else {
            false
        }
    }

    /// Recalculate stats from equipment
    fn recalc_equipment_stats(&mut self) {
        // Sync equipped_weapon with equipment.weapon for backward compat
        self.equipped_weapon = self.equipment.weapon.clone();
        
        // Calculate armor from equipped armor item
        self.player_armor = self.equipment.armor.as_ref()
            .and_then(|id| get_item_def(id))
            .map(|def| def.armor_value)
            .unwrap_or(0);
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
