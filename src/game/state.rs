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
    quest::QuestLog,
    spawn::{load_spawn_tables, weighted_pick},
    storm::Storm,
    tutorial::TutorialProgress,
    world_map::WorldMap,
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

/// Message types for color-coded log display
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MsgType {
    #[default]
    System,
    Combat,
    Loot,
    Status,
    Dialogue,
}

/// Game message with type for color-coding
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameMessage {
    pub text: String,
    pub msg_type: MsgType,
    pub turn: u32,
}

impl GameMessage {
    pub fn new(text: impl Into<String>, msg_type: MsgType, turn: u32) -> Self {
        Self { text: text.into(), msg_type, turn }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggeredEffect {
    pub effect: String,
    pub turns_remaining: u32,
}

fn default_ambient_light() -> u8 { 100 }
fn default_time_of_day() -> u8 { 8 } // Start at 8 AM

/// Weather conditions affecting visibility and lighting
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Weather {
    #[default]
    Clear,
    Dusty,      // Reduced visibility
    Sandstorm,  // Severely reduced visibility
}

impl Weather {
    pub fn visibility_modifier(&self) -> i32 {
        match self {
            Weather::Clear => 0,
            Weather::Dusty => -2,
            Weather::Sandstorm => -4,
        }
    }
    
    pub fn ambient_modifier(&self) -> i32 {
        match self {
            Weather::Clear => 0,
            Weather::Dusty => -20,
            Weather::Sandstorm => -50,
        }
    }
}

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
    pub player_xp: u32,
    #[serde(default)]
    pub player_level: u32,
    #[serde(default)]
    pub pending_stat_points: i32,
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
    pub messages: Vec<GameMessage>, pub turn: u32,
    #[serde(with = "rng_serde")]
    pub rng: ChaCha8Rng, pub storm: Storm,
    pub refraction: u32,
    pub adaptations: Vec<Adaptation>,
    #[serde(default)]
    pub adaptations_hidden_turns: u32,
    #[serde(default)]
    pub quest_log: QuestLog,
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
    #[serde(skip)]
    pub hit_flash_positions: Vec<(i32, i32, u32)>,
    #[serde(skip)]
    pub damage_numbers: Vec<DamageNumber>,
    #[serde(skip)]
    pub projectile_trails: Vec<ProjectileTrail>,
    #[serde(skip)]
    pub mock_combat_hit: Option<bool>,
    #[serde(skip)]
    pub mock_combat_damage: Option<i32>,
    #[serde(skip)]
    pub meta: super::meta::MetaProgress,
    // World map for lazy tile generation
    #[serde(default)]
    pub world_map: Option<WorldMap>,
    #[serde(default)]
    pub world_x: usize,
    #[serde(default)]
    pub world_y: usize,
    /// Current layer: 0 = surface, negative = underground
    #[serde(default)]
    pub layer: i32,
    /// Time of day (0-23 hours)
    #[serde(default = "default_time_of_day")]
    pub time_of_day: u8,
    /// Current weather condition
    #[serde(default)]
    pub weather: Weather,
    /// Tutorial system progress tracking
    #[serde(default)]
    pub tutorial_progress: TutorialProgress,
    // Debug flags
    #[serde(skip)]
    pub debug_god_view: bool,
    #[serde(skip)]
    pub debug_phase: bool,
}

/// Floating damage number for visual feedback
#[derive(Clone)]
pub struct DamageNumber {
    pub x: i32,
    pub y: i32,
    pub value: i32,
    pub frames: u32,
    pub is_heal: bool,
}

/// Projectile trail for ranged attack animation
#[derive(Clone)]
pub struct ProjectileTrail {
    pub path: Vec<(i32, i32)>,
    pub current_idx: usize,
    pub frames_per_tile: u32,
    pub frame_counter: u32,
    pub char: char,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        // Generate world map
        let world_map = WorldMap::generate(seed);
        let world_x = super::world_map::WORLD_SIZE / 2;
        let world_y = super::world_map::WORLD_SIZE / 2;
        
        // Get world context for starting tile
        let (biome, terrain, elevation, poi, _resources, _connected) = world_map.get(world_x, world_y);
        
        // Generate tile map using world context
        let tile_seed = world_map.tile_seed(world_x, world_y);
        let mut rng = ChaCha8Rng::seed_from_u64(tile_seed);
        let (mut map, rooms) = Map::generate_from_world_with_poi(&mut rng, biome, terrain, elevation, poi);
        let (px, py) = rooms[0];
        
        // Add world exit to starting tile (near spawn point)
        let exit_x = (px + 1).min(map.width as i32 - 1) as usize;
        let exit_y = py as usize;
        map.tiles[exit_y * map.width + exit_x] = Tile::WorldExit;
        
        let visible = compute_fov(&map, px, py);
        let tables = load_spawn_tables();
        let table = &tables.default;

        // Spawn enemies (fewer on starting tile for hospitable start)
        let mut enemies = Vec::new();
        let enemy_rooms = rooms.iter().skip(2).take(rooms.len().saturating_sub(4));
        for &(rx, ry) in enemy_rooms {
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

        // Spawn items (more on starting tile for hospitable start)
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
            for _ in 0..(spawn.weight + 1) { // +1 for hospitable start
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
            player_reflex: 5, player_armor: 0, player_xp: 0, player_level: 0,
            pending_stat_points: 0,
            equipped_weapon: None,
            equipment: Equipment::default(),
            status_effects: Vec::new(),
            map, enemies, npcs, items, inventory: Vec::new(),
            visible: visible.clone(), revealed: visible,
            light_map, ambient_light: ambient,
            messages: vec![GameMessage::new("Welcome to the Saltglass Steppe.", MsgType::System, 0)],
            turn: 0, rng, storm: Storm::forecast(&mut ChaCha8Rng::seed_from_u64(seed + 1)),
            refraction: 0, adaptations: Vec::new(), adaptations_hidden_turns: 0,
            quest_log: QuestLog::default(),
            triggered_effects: Vec::new(),
            enemy_positions: HashMap::new(),
            npc_positions: HashMap::new(),
            item_positions: HashMap::new(),
            event_queue: Vec::new(),
            hit_flash_positions: Vec::new(),
            damage_numbers: Vec::new(),
            projectile_trails: Vec::new(),
            mock_combat_hit: None,
            mock_combat_damage: None,
            meta: super::meta::MetaProgress::load(),
            world_map: Some(world_map),
            world_x,
            world_y,
            layer: 0,
            time_of_day: 8,
            weather: Weather::Clear,
            tutorial_progress: TutorialProgress::default(),
            debug_god_view: false,
            debug_phase: false,
        };
        state.rebuild_spatial_index();
        state
    }

    /// Create a new game with a specific character class
    pub fn new_with_class(seed: u64, class_id: &str) -> Self {
        let mut state = Self::new(seed);
        
        if let Some(class) = super::meta::get_class(class_id) {
            state.player_hp = class.starting_hp;
            state.player_max_hp = class.starting_hp;
            state.player_ap = class.starting_ap;
            state.player_max_ap = class.starting_ap;
            
            // Add starting items
            for item_id in &class.starting_items {
                state.inventory.push(item_id.clone());
            }
            
            // Add starting adaptations
            for adapt_id in &class.starting_adaptations {
                if let Some(adapt) = Adaptation::from_id(adapt_id) {
                    state.adaptations.push(adapt);
                }
            }
            
            state.log(format!("You begin as a {}.", class.name));
        }
        
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

    /// Travel to a new world tile (lazy generation)
    pub fn travel_to_tile(&mut self, new_wx: usize, new_wy: usize) {
        let world_map = match &self.world_map {
            Some(wm) => wm,
            None => return,
        };
        
        let (biome, terrain, elevation, poi, _resources, _connected) = world_map.get(new_wx, new_wy);
        let tile_seed = world_map.tile_seed(new_wx, new_wy);
        let mut rng = ChaCha8Rng::seed_from_u64(tile_seed);
        
        // Generate new tile map
        let (map, rooms) = Map::generate_from_world_with_poi(&mut rng, biome, terrain, elevation, poi);
        let (px, py) = rooms[0];
        
        // Spawn enemies based on POI
        let tables = load_spawn_tables();
        let table = &tables.default;
        let mut enemies = Vec::new();
        let enemy_count = match poi {
            super::world_map::POI::Town => 0,
            super::world_map::POI::Shrine => 1,
            _ => rooms.len().saturating_sub(2),
        };
        for &(rx, ry) in rooms.iter().skip(1).take(enemy_count) {
            if let Some(id) = weighted_pick(&table.enemies, &mut rng) {
                enemies.push(Enemy::new(rx, ry, id));
            }
        }
        
        // Spawn items
        let mut items = Vec::new();
        let mut used_positions = HashSet::new();
        for spawn in &table.items {
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
        
        // Update state
        self.world_x = new_wx;
        self.world_y = new_wy;
        self.map = map;
        self.enemies = enemies;
        self.items = items;
        self.npcs = Vec::new(); // NPCs are tile-specific
        self.player_x = px;
        self.player_y = py;
        self.visible = compute_fov(&self.map, px, py);
        self.revealed = self.visible.clone();
        self.rebuild_spatial_index();
        self.update_lighting();
        
        self.log(format!("You enter a new area ({:?} {:?}).", biome, terrain));
    }

    /// Enter subterranean layer (go down stairs)
    pub fn enter_subterranean(&mut self) -> bool {
        // Check if standing on stairs down
        if let Some(tile) = self.map.get(self.player_x, self.player_y) {
            if *tile != Tile::StairsDown { return false; }
        } else { return false; }

        self.layer -= 1;
        let seed = self.world_map.as_ref()
            .map(|wm| wm.tile_seed(self.world_x, self.world_y))
            .unwrap_or(42)
            .wrapping_add(self.layer.unsigned_abs() as u64 * 1000);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        
        let (map, rooms) = Map::generate_subterranean(&mut rng, self.layer);
        let (px, py) = rooms[0];
        
        self.map = map;
        self.enemies = Vec::new();
        self.items = Vec::new();
        self.npcs = Vec::new();
        self.player_x = px;
        self.player_y = py;
        self.visible = compute_fov(&self.map, px, py);
        self.revealed = self.visible.clone();
        self.rebuild_spatial_index();
        self.update_lighting();
        
        self.log(format!("You descend to level {}.", -self.layer));
        true
    }

    /// Exit subterranean layer (go up stairs)
    pub fn exit_subterranean(&mut self) -> bool {
        // Check if standing on stairs up
        if let Some(tile) = self.map.get(self.player_x, self.player_y) {
            if *tile != Tile::StairsUp { return false; }
        } else { return false; }

        if self.layer >= 0 { return false; } // Already on surface

        self.layer += 1;
        
        if self.layer == 0 {
            // Return to surface - regenerate surface tile
            self.travel_to_tile(self.world_x, self.world_y);
            self.log("You return to the surface.");
        } else {
            // Go up one underground level
            let seed = self.world_map.as_ref()
                .map(|wm| wm.tile_seed(self.world_x, self.world_y))
                .unwrap_or(42)
                .wrapping_add(self.layer.unsigned_abs() as u64 * 1000);
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            
            let (map, rooms) = Map::generate_subterranean(&mut rng, self.layer);
            let (px, py) = rooms.last().copied().unwrap_or((5, 5));
            
            self.map = map;
            self.enemies = Vec::new();
            self.items = Vec::new();
            self.npcs = Vec::new();
            self.player_x = px;
            self.player_y = py;
            self.visible = compute_fov(&self.map, px, py);
            self.revealed = self.visible.clone();
            self.rebuild_spatial_index();
            self.update_lighting();
            
            self.log(format!("You ascend to level {}.", -self.layer));
        }
        true
    }

    pub fn update_lighting(&mut self) {
        let mut sources = Vec::new();
        // Equipped light sources (check all slots)
        for (_, slot_item) in self.equipment.iter() {
            if let Some(id) = slot_item {
                if let Some(def) = get_item_def(id) {
                    if let Some(ref ls) = def.light_source {
                        sources.push(LightSource { x: self.player_x, y: self.player_y, radius: ls.radius, intensity: ls.intensity });
                    }
                }
            }
        }
        // Default player light if no equipped light source
        if sources.is_empty() {
            sources.push(LightSource { x: self.player_x, y: self.player_y, radius: 8, intensity: 150 });
        }
        // Map lights
        for ml in &self.map.lights {
            if let Some(def) = super::light_defs::get_light_def(&ml.id) {
                sources.push(LightSource { x: ml.x, y: ml.y, radius: def.radius, intensity: def.intensity });
            }
        }
        // Items on ground with light_source property
        for item in &self.items {
            if let Some(def) = get_item_def(&item.id) {
                if let Some(ref ls) = def.light_source {
                    sources.push(LightSource { x: item.x, y: item.y, radius: ls.radius, intensity: ls.intensity });
                }
            }
        }
        self.light_map = compute_lighting(&sources, self.effective_ambient_light());
    }

    /// Calculate effective ambient light based on time of day and weather
    pub fn effective_ambient_light(&self) -> u8 {
        // Underground has fixed low ambient
        if self.layer < 0 {
            return 30;
        }
        
        // Base ambient from time of day (0-23 hours)
        let time_ambient = match self.time_of_day {
            0..=4 => 30,    // Night
            5..=6 => 60,    // Dawn
            7..=17 => 120,  // Day
            18..=19 => 80,  // Dusk
            20..=23 => 40,  // Night
            _ => 100,
        };
        
        // Apply weather modifier
        let weather_mod = self.weather.ambient_modifier();
        (time_ambient as i32 + weather_mod).clamp(10, 200) as u8
    }

    /// Advance time by one turn (10 turns = 1 hour)
    pub fn tick_time(&mut self) {
        if self.turn % 10 == 0 {
            self.time_of_day = (self.time_of_day + 1) % 24;
            
            // Random weather changes at dawn/dusk
            if self.time_of_day == 6 || self.time_of_day == 18 {
                let roll = self.rng.gen_range(0..10);
                self.weather = match roll {
                    0..=6 => Weather::Clear,
                    7..=8 => Weather::Dusty,
                    9 => Weather::Sandstorm,
                    _ => Weather::Clear,
                };
            }
        }
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

    /// Gain XP and check for level up
    pub fn gain_xp(&mut self, amount: u32) {
        use super::progression::{xp_for_level, stat_points_per_level, max_level};
        
        self.player_xp += amount;
        self.log(format!("+{} XP", amount));
        
        // Check for level up
        while self.player_level < max_level() {
            let next_threshold = xp_for_level(self.player_level + 1);
            if self.player_xp >= next_threshold {
                self.player_level += 1;
                let points = stat_points_per_level();
                self.pending_stat_points += points;
                self.log(format!("⬆ LEVEL {}! (+{} stat points)", self.player_level, points));
                self.emit(GameEvent::LevelUp { level: self.player_level });
            } else {
                break;
            }
        }
    }

    /// Allocate a stat point to a specific stat
    pub fn allocate_stat(&mut self, stat: &str) -> bool {
        if self.pending_stat_points <= 0 { return false; }
        
        match stat {
            "max_hp" => {
                self.player_max_hp += 1;
                self.player_hp += 1; // Also heal
            }
            "max_ap" => self.player_max_ap += 1,
            "reflex" => self.player_reflex += 1,
            _ => return false,
        }
        
        self.pending_stat_points -= 1;
        self.log(format!("+1 {}", stat));
        true
    }

    /// End turn: reset AP, tick status effects, run enemy turns, tick storm, tick time
    pub fn end_turn(&mut self) {
        self.player_ap = self.player_max_ap;
        self.tick_status_effects();
        self.tick_turn();
        self.update_enemies();
        if self.storm.tick() { self.apply_storm(); }
        self.tick_time();
        self.update_lighting();
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
        self.log_typed(format!("You are {}! ({} turns)", effect.name(), effect.duration), MsgType::Status);
        self.status_effects.push(effect);
    }

    /// Wait in place (costs 0 AP, ends turn)
    pub fn wait_turn(&mut self) {
        self.end_turn();
    }

    /// Rest to recover HP (50% max HP). Requires no nearby enemies and costs 10 turns.
    pub fn rest(&mut self) -> Result<(), String> {
        // Check for nearby enemies (within FOV range)
        for enemy in &self.enemies {
            let dx = (enemy.x - self.player_x).abs();
            let dy = (enemy.y - self.player_y).abs();
            if dx <= super::constants::FOV_RANGE && dy <= super::constants::FOV_RANGE {
                return Err("You cannot rest with enemies nearby!".to_string());
            }
        }

        // Heal 50% max HP
        let heal_amount = (self.player_max_hp as f32 * 0.5) as i32;
        let old_hp = self.player_hp;
        self.player_hp = (self.player_hp + heal_amount).min(self.player_max_hp);
        let actual_heal = self.player_hp - old_hp;

        if actual_heal > 0 {
            self.log_typed(
                format!("You rest and recover {} HP.", actual_heal),
                MsgType::Status,
            );
        } else {
            self.log_typed("You rest but are already at full health.", MsgType::Status);
        }

        // Advance 10 turns
        for _ in 0..10 {
            self.tick_turn();
        }

        // Process enemy turns (they get to act while you rest)
        self.update_enemies();

        Ok(())
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
                self.log_typed("The tincture wears off. Your glow returns.", MsgType::Status);
            }
        }
        // Tick down triggered effects
        self.triggered_effects.retain_mut(|e| {
            e.turns_remaining = e.turns_remaining.saturating_sub(1);
            e.turns_remaining > 0
        });
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.log_typed(msg, MsgType::System);
    }

    pub fn log_typed(&mut self, msg: impl Into<String>, msg_type: MsgType) {
        self.messages.push(GameMessage::new(msg, msg_type, self.turn));
        if self.messages.len() > 5 { self.messages.remove(0); }
    }

    /// Execute a debug command
    pub fn debug_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        match parts.first().map(|s| *s) {
            Some("show") if parts.get(1) == Some(&"tile") => {
                self.debug_god_view = true;
                self.log("Debug: God view enabled");
            }
            Some("hide") if parts.get(1) == Some(&"tile") => {
                self.debug_god_view = false;
                self.log("Debug: God view disabled");
            }
            Some("sturdy") => {
                self.player_hp = 9999;
                self.player_max_hp = 9999;
                self.log("Debug: HP set to 9999/9999");
            }
            Some("phase") => {
                self.debug_phase = !self.debug_phase;
                self.log(format!("Debug: Phase {}", if self.debug_phase { "enabled" } else { "disabled" }));
            }
            Some("help") => {
                self.log("Commands: show tile, hide tile, sturdy, phase");
            }
            _ => self.log(format!("Unknown command: {}", cmd)),
        }
    }

    /// Trigger a hit flash effect at position
    pub fn trigger_hit_flash(&mut self, x: i32, y: i32) {
        self.hit_flash_positions.push((x, y, 6)); // 6 frames
    }

    /// Tick hit flash animations (call each frame)
    pub fn tick_hit_flash(&mut self) {
        self.hit_flash_positions.retain_mut(|(_, _, frames)| {
            *frames = frames.saturating_sub(1);
            *frames > 0
        });
    }

    /// Check if position has active hit flash
    pub fn has_hit_flash(&self, x: i32, y: i32) -> bool {
        self.hit_flash_positions.iter().any(|(fx, fy, _)| *fx == x && *fy == y)
    }

    /// Spawn a floating damage number
    pub fn spawn_damage_number(&mut self, x: i32, y: i32, value: i32, is_heal: bool) {
        self.damage_numbers.push(DamageNumber { x, y, value, frames: 12, is_heal });
    }

    /// Tick damage number animations
    pub fn tick_damage_numbers(&mut self) {
        self.damage_numbers.retain_mut(|dn| {
            dn.frames = dn.frames.saturating_sub(1);
            dn.frames > 0
        });
    }

    /// Spawn a projectile trail from source to target
    pub fn spawn_projectile(&mut self, from: (i32, i32), to: (i32, i32), ch: char) {
        let path = line_path(from, to);
        if path.len() > 1 {
            self.projectile_trails.push(ProjectileTrail {
                path,
                current_idx: 0,
                frames_per_tile: 2,
                frame_counter: 0,
                char: ch,
            });
        }
    }

    /// Tick projectile trail animations
    pub fn tick_projectile_trails(&mut self) {
        self.projectile_trails.retain_mut(|pt| {
            pt.frame_counter += 1;
            if pt.frame_counter >= pt.frames_per_tile {
                pt.frame_counter = 0;
                pt.current_idx += 1;
            }
            pt.current_idx < pt.path.len()
        });
    }

    /// Get current projectile position if any
    pub fn get_projectile_at(&self, x: i32, y: i32) -> Option<char> {
        for pt in &self.projectile_trails {
            if pt.current_idx < pt.path.len() {
                let (px, py) = pt.path[pt.current_idx];
                if px == x && py == y {
                    return Some(pt.char);
                }
            }
        }
        None
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
            let inventory_snapshot = self.inventory.clone();
            let ctx = super::npc::DialogueContext {
                adaptations: &visible_adaptations,
                inventory: &inventory_snapshot,
            };
            let dialogue = self.npcs[ni].dialogue(&ctx).to_string();
            let name = self.npcs[ni].name().to_string();
            let actions: Vec<_> = self.npcs[ni].available_actions(&ctx).into_iter().cloned().collect();
            
            self.log_typed(format!("{}: \"{}\"", name, dialogue), MsgType::Dialogue);
            
            // Execute first available action
            for action in &actions {
                // Item exchange
                if let (Some(gives), Some(consumes)) = (&action.effect.gives_item, &action.effect.consumes) {
                    if let Some(idx) = self.inventory.iter().position(|id| id == consumes) {
                        self.inventory.remove(idx);
                        self.inventory.push(gives.clone());
                        let gives_name = get_item_def(gives).map(|d| d.name.as_str()).unwrap_or("item");
                        self.log_typed(format!("The pilgrim presses {} into your hand.", gives_name), MsgType::Loot);
                        break;
                    }
                }
                // Heal action
                if let Some(heal) = action.effect.heal {
                    let actual = heal.min(self.player_max_hp - self.player_hp);
                    self.player_hp += actual;
                    self.log_typed(format!("You rest. (+{} HP)", actual), MsgType::Status);
                    break;
                }
            }
            
            self.npcs[ni].talked = true;
            self.quest_log.on_npc_talked(&self.npcs[ni].id);
            self.meta.discover_npc(&self.npcs[ni].id);
            self.check_auto_end_turn();
            return true;
        }

        if self.enemy_at(new_x, new_y).is_some() {
            // If not enough AP, end turn first then attack
            let cost = action_cost("attack_melee");
            if self.player_ap < cost {
                self.end_turn();
            }
            let hit = self.attack_melee(new_x, new_y);
            if hit { self.check_auto_end_turn(); }
            return hit;
        }

        if let Some(tile) = self.map.get(new_x, new_y) {
            let walkable = tile.walkable() || self.debug_phase;
            let is_glass = *tile == Tile::Glass;
            if walkable {
                let cost = action_cost("move");
                if self.player_ap < cost { return false; }
                self.player_ap -= cost;
                self.player_x = new_x;
                self.player_y = new_y;
                self.quest_log.on_position_changed(new_x, new_y);
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
        let mut picked_up = Vec::new();
        // Process in reverse order to maintain valid indices during removal
        for &i in indices.iter().rev() {
            let id = self.items[i].id.clone();
            let def = get_item_def(&id);
            // Skip non-pickup items (e.g., light sources)
            if !def.map(|d| d.pickup).unwrap_or(true) {
                continue;
            }
            let name = def.map(|d| d.name.as_str()).unwrap_or("item");
            if let Some(d) = def {
                for e in &d.effects {
                    if e.condition == "on_pickup" {
                        self.trigger_effect(&e.effect, 3);
                    }
                }
            }
            self.inventory.push(id.clone());
            self.quest_log.on_item_collected(&id);
            self.emit(GameEvent::ItemPickedUp { item_id: id.clone() });
            self.meta.discover_item(&id);
            self.log_typed(format!("Picked up {}.", name), MsgType::Loot);
            picked_up.push(i);
        }
        // Remove picked up items
        for i in picked_up {
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
            self.log_typed(format!("You use {}. (+{} HP)", def.name, heal), MsgType::Loot);
        }
        if def.reduces_refraction > 0 {
            let reduce = def.reduces_refraction.min(self.refraction);
            self.refraction -= reduce;
            self.log_typed(format!("Your glow fades slightly. (-{} Refraction)", reduce), MsgType::Status);
        }
        if def.suppresses_adaptations {
            self.adaptations_hidden_turns = 10;
            self.log_typed("Your glow dims. The tincture masks your changes.", MsgType::Status);
        }
        if def.reveals_map {
            self.log_typed(format!("The {} reveals hidden paths...", def.name), MsgType::Loot);
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
        
        // Calculate armor from equipped jacket item
        self.player_armor = self.equipment.jacket.as_ref()
            .and_then(|id| get_item_def(id))
            .map(|def| def.armor_value)
            .unwrap_or(0);
    }

    /// Accept a quest by ID
    pub fn accept_quest(&mut self, quest_id: &str) -> bool {
        if self.quest_log.accept(quest_id) {
            if let Some(def) = super::quest::get_quest_def(quest_id) {
                self.log(format!("Quest accepted: {}", def.name));
            }
            true
        } else {
            false
        }
    }

    /// Complete a quest and receive rewards
    pub fn complete_quest(&mut self, quest_id: &str) -> bool {
        if let Some(reward) = self.quest_log.complete(quest_id) {
            if let Some(def) = super::quest::get_quest_def(quest_id) {
                self.log(format!("Quest completed: {}", def.name));
            }
            if reward.xp > 0 {
                self.gain_xp(reward.xp);
            }
            for item_id in &reward.items {
                self.inventory.push(item_id.clone());
            }
            // Log unlocked quests
            if !reward.unlocks_quests.is_empty() {
                for unlocked_id in &reward.unlocks_quests {
                    if let Some(unlocked_def) = super::quest::get_quest_def(unlocked_id) {
                        self.log(format!("New quest available: {}", unlocked_def.name));
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Craft an item using a recipe
    pub fn craft(&mut self, recipe_id: &str) -> bool {
        let recipe = match super::crafting::get_recipe(recipe_id) {
            Some(r) => r,
            None => return false,
        };
        
        if !super::crafting::can_craft(recipe, &self.inventory) {
            return false;
        }
        
        // Remove materials
        for (item_id, &count) in &recipe.materials {
            for _ in 0..count {
                if let Some(idx) = self.inventory.iter().position(|id| id == item_id) {
                    self.inventory.remove(idx);
                }
            }
        }
        
        // Add output
        for _ in 0..recipe.output_count {
            self.inventory.push(recipe.output.clone());
        }
        
        self.log(format!("Crafted {}.", recipe.name));
        true
    }

    /// Get next tutorial message if conditions are met
    pub fn get_next_tutorial_message(&self) -> Option<&super::tutorial::TutorialMessage> {
        self.tutorial_progress.get_next_message(self)
    }

    /// Mark a tutorial message as shown
    pub fn dismiss_tutorial_message(&mut self, message_id: &str) {
        self.tutorial_progress.mark_shown(message_id);
    }

    /// Drop loot from an enemy at a specific position
    pub(crate) fn drop_enemy_loot(&mut self, loot_table: &[super::enemy::LootEntry], x: i32, y: i32) {
        if loot_table.is_empty() {
            return;
        }

        // Calculate total weight
        let total_weight: u32 = loot_table.iter().map(|entry| entry.weight).sum();
        if total_weight == 0 {
            return;
        }

        // Roll for loot drop
        let roll = self.rng.gen_range(0..total_weight);
        let mut cumulative = 0;
        for entry in loot_table {
            cumulative += entry.weight;
            if roll < cumulative {
                // Drop this item
                let item = Item::new(x, y, &entry.item);
                self.items.push(item);
                self.rebuild_spatial_index();
                if let Some(def) = get_item_def(&entry.item) {
                    self.log_typed(format!("The enemy drops {}.", def.name), MsgType::Loot);
                }
                return;
            }
        }
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

/// Simple Bresenham line for projectile paths
fn line_path(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let mut path = Vec::new();
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        path.push((x0, y0));
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
    path
}
