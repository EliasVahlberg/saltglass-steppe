use bracket_pathfinding::prelude::*;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::{
    adaptation::Adaptation,
    chest::Chest,
    enemy::Enemy,
    entity::Entity,
    generation::place_microstructures,
    generation::{
        distribute_points_grid, generate_loot, get_biome_spawn_table,
        weighted_pick_by_level_and_tier,
    },
    interactable::Interactable,
    item::{Item, get_item_def},
    lighting::{LightMap, LightSource, compute_lighting},
    map::{Map, Tile},
    map_features::MapFeatures,
    npc::Npc,
    storm::Storm,
    systems::combat::CombatSystem,
    systems::movement::MovementSystem,
    world_map::WorldMap,
};
use crate::game::narrative_engine::{NarrativeEngine, StoryModel};
use crate::game::player_state::PlayerState;
use crate::game::world_state::{Weather, WorldState};

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
    Social,
    Loot,
    Status,
    Dialogue,
    Warning,
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
        Self {
            text: text.into(),
            msg_type,
            turn,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggeredEffect {
    pub effect: String,
    pub turns_remaining: u32,
}

/// Decoy left by mirage_step adaptation
#[derive(Clone, Serialize, Deserialize)]
pub struct Decoy {
    pub x: i32,
    pub y: i32,
    pub turns_remaining: u32,
}

/// Spatial indices — rebuilt from entity positions, never serialized.
#[derive(Clone, Default)]
pub struct SpatialIndex {
    pub enemy_positions: HashMap<(i32, i32), usize>,
    pub npc_positions: HashMap<(i32, i32), usize>,
    pub item_positions: HashMap<(i32, i32), Vec<usize>>,
    pub chest_positions: HashMap<(i32, i32), usize>,
    pub interactable_positions: HashMap<(i32, i32), usize>,
    pub dirty: bool,
}

/// Debug/mock flags — never serialized, testing infrastructure only.
#[derive(Clone, Default)]
pub struct DebugState {
    pub god_view: bool,
    pub phase: bool,
    pub disable_glare: bool,
    pub mock_combat_hit: Option<bool>,
    pub mock_combat_damage: Option<i32>,
    pub test_mode: bool,
}

/// Pending UI handoff state — game logic sets, UI reads and clears.
#[derive(Clone, Default)]
pub struct PendingUi {
    pub book_open: Option<String>,
    pub trade: Option<String>,
    pub dialogue: Option<(String, String)>,
    pub aria_dialogue: Option<(String, Vec<String>)>,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub player: PlayerState,
    pub world: WorldState,
    pub visible: HashSet<usize>,
    pub revealed: HashSet<usize>,
    #[serde(skip)]
    pub light_map: LightMap,
    pub messages: Vec<GameMessage>,
    pub turn: u32,
    #[serde(with = "rng_serde")]
    pub rng: ChaCha8Rng,
    #[serde(default)]
    pub triggered_effects: Vec<TriggeredEffect>,
    /// Decoys left by mirage_step adaptation
    #[serde(default)]
    pub decoys: Vec<Decoy>,
    #[serde(skip)]
    pub spatial: SpatialIndex,
    #[serde(skip)]
    pub debug: DebugState,
    #[serde(skip)]
    pub pending_ui: PendingUi,
    #[serde(skip)]
    pub meta: super::meta::MetaProgress,
    /// Consecutive turns waited (for auto-rest)
    #[serde(default)]
    pub wait_counter: u32,
    /// Tutorial system progress tracking
    #[serde(default)]
    pub narrative: NarrativeEngine,
    /// Advanced map features (hidden locations, safe routes, etc.)
    #[serde(default)]
    pub map_features: MapFeatures,
    /// Original seed for reproducibility
    #[serde(default)]
    pub seed: u64,
    /// VERA trace — records effects for DES verification
    #[serde(skip)]
    pub trace: super::effects::Trace,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        // Generate world map
        let world_map = WorldMap::generate(seed);
        let world_x = super::world_map::WORLD_WIDTH / 2;
        let world_y = super::world_map::WORLD_HEIGHT / 2;

        // Get world context for starting tile
        let (biome, terrain, elevation, poi, _resources, _connected, level) =
            world_map.get(world_x, world_y);

        // Generate tile map using world context
        let tile_seed = world_map.tile_seed(world_x, world_y);
        let mut rng = ChaCha8Rng::seed_from_u64(tile_seed);
        let (mut map, rooms) =
            Map::generate_from_world_with_poi(&mut rng, biome, terrain, elevation, poi);
        let (px, py) = rooms[0];
        // Clamp spawn point away from map edges to ensure full 5x5 clearing fits
        let px = px.max(3).min(map.width as i32 - 4);
        let py = py.max(3).min(map.height as i32 - 4);

        // Clear 5x5 area around player spawn to ensure walkable space
        for dy in -2..=2 {
            for dx in -2..=2 {
                let cx = px + dx;
                let cy = py + dy;
                if cx >= 1 && cy >= 1 && cx < map.width as i32 - 1 && cy < map.height as i32 - 1 {
                    let idx = map.idx(cx, cy);
                    if !map.tiles[idx].walkable() {
                        map.tiles[idx] = Tile::default_floor();
                    }
                }
            }
        }

        // Add world exit to starting tile (near spawn point)
        let exit_x = (px + 1).min(map.width as i32 - 1) as usize;
        let exit_y = py as usize;
        map.tiles[exit_y * map.width + exit_x] = Tile::WorldExit;

        let visible = crate::game::map::compute_fov(&map, px, py);
        let table = get_biome_spawn_table(&biome);

        // Spawn enemies (fewer on starting tile for hospitable start)
        let mut enemies = Vec::new();
        let max_enemies = 8; // Limit total enemies regardless of clearing count
        let safe_distance = 15; // Minimum distance from player spawn
        let (px, py) = rooms[0]; // Player spawn position

        let safe_rooms: Vec<_> = rooms
            .iter()
            .filter(|&&(rx, ry)| {
                let dx = (rx - px).abs();
                let dy = (ry - py).abs();
                dx >= safe_distance || dy >= safe_distance
            })
            .cloned()
            .collect();

        // Use spatial distribution to spread out enemy spawns
        let distributed_positions = distribute_points_grid(
            &safe_rooms,
            max_enemies,
            20, // Minimum distance between enemies
            &mut rng,
        );

        for (rx, ry) in distributed_positions {
            if let Some(id) =
                weighted_pick_by_level_and_tier(&table.enemies, level, &mut rng, false)
            {
                enemies.push(Enemy::new(rx, ry, id));
            }
        }

        // Spawn NPCs
        let mut npcs = Vec::new();

        // ALWAYS spawn the dying pilgrim on the first tile for main questline
        // Find a safe position near the player spawn
        let pilgrim_pos = {
            let offsets = [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
            ];
            let mut spawn_pos = (px + 1, py); // Default fallback
            for &(dx, dy) in &offsets {
                let test_x = px + dx;
                let test_y = py + dy;
                if test_x >= 0
                    && test_y >= 0
                    && test_x < map.width as i32
                    && test_y < map.height as i32
                {
                    let test_idx = map.idx(test_x, test_y);
                    if map.tiles[test_idx].walkable() {
                        spawn_pos = (test_x, test_y);
                        break;
                    }
                }
            }
            spawn_pos
        };
        npcs.push(Npc::new(pilgrim_pos.0, pilgrim_pos.1, "dying_pilgrim"));

        // Spawn other NPCs from spawn table
        let late_room = rooms.len().saturating_sub(2);
        for spawn in &table.npcs {
            // Skip dying pilgrim since we already spawned it
            if spawn.id == "dying_pilgrim" {
                continue;
            }

            let room_idx = match spawn.room.as_deref() {
                Some("late") => Some(late_room),
                Some("last") => Some(rooms.len() - 1),
                Some("first") => Some(0),
                _ => {
                    if rng.gen_ratio(spawn.weight.min(10), 10) {
                        Some(rng.gen_range(1..rooms.len()))
                    } else {
                        None
                    }
                }
            };
            if let Some(idx) = room_idx
                && idx < rooms.len()
            {
                let (rx, ry) = rooms[idx];
                // If spawning in first room (where player is), find adjacent position
                let (npc_x, npc_y) = if idx == 0 {
                    // Try adjacent positions around the room center
                    let offsets = [
                        (1, 0),
                        (-1, 0),
                        (0, 1),
                        (0, -1),
                        (1, 1),
                        (-1, -1),
                        (1, -1),
                        (-1, 1),
                    ];
                    let mut spawn_pos = (rx, ry);
                    for &(dx, dy) in &offsets {
                        let test_x = rx + dx;
                        let test_y = ry + dy;
                        if test_x >= 0
                            && test_y >= 0
                            && test_x < map.width as i32
                            && test_y < map.height as i32
                        {
                            let test_idx = map.idx(test_x, test_y);
                            if map.tiles[test_idx].walkable() &&
                                   (test_x != px || test_y != py) && // Don't spawn on player
                                   (test_x != pilgrim_pos.0 || test_y != pilgrim_pos.1)
                            {
                                // Don't spawn on pilgrim
                                spawn_pos = (test_x, test_y);
                                break;
                            }
                        }
                    }
                    spawn_pos
                } else {
                    (rx, ry)
                };
                npcs.push(Npc::new(npc_x, npc_y, &spawn.id));
            }
        }

        // Spawn items (more on starting tile for hospitable start)
        let mut items = Vec::new();
        let mut used_positions = HashSet::new();

        // Always spawn hand torch near player start
        items.push(Item::new(px + 1, py, "hand_torch"));
        used_positions.insert((px + 1, py));

        // Always spawn glass pick (wall break tool) near player start
        items.push(Item::new(px - 1, py, "glass_pick"));
        used_positions.insert((px - 1, py));

        for spawn in &table.items {
            if let Some("last") = spawn.room.as_deref() {
                if let Some(&(rx, ry)) = rooms.last()
                    && !used_positions.contains(&(rx, ry))
                {
                    used_positions.insert((rx, ry));
                    // Check tier eligibility for last room items
                    if let Some(item_def) = super::item::get_item_def(&spawn.id) {
                        let tier_threshold = match level {
                            1 => 1,
                            2..=3 => 2,
                            4..=6 => 3,
                            7..=8 => 4,
                            9..=10 => 5,
                            _ => 1,
                        };
                        if item_def.tier <= tier_threshold {
                            items.push(Item::new(rx, ry, &spawn.id));
                        }
                    }
                }
                continue;
            }
            for _ in 0..(spawn.weight + 1) {
                // +1 for hospitable start
                if let Some(&(rx, ry)) = rooms.get(rng.gen_range(1..rooms.len())) {
                    let ix = rx + rng.gen_range(-1..=1);
                    let iy = ry + rng.gen_range(-1..=1);
                    if !used_positions.contains(&(ix, iy)) {
                        used_positions.insert((ix, iy));
                        // Check tier eligibility for regular items
                        if let Some(item_def) = super::item::get_item_def(&spawn.id) {
                            let tier_threshold = match level {
                                1 => 1,
                                2..=3 => 2,
                                4..=6 => 3,
                                7..=8 => 4,
                                9..=10 => 5,
                                _ => 1,
                            };
                            if item_def.tier <= tier_threshold {
                                items.push(Item::new(ix, iy, &spawn.id));
                            }
                        }
                    }
                }
            }
        }

        // Spawn chests in some rooms
        let mut chests = Vec::new();
        let chest_rooms: Vec<_> = rooms
            .iter()
            .skip(2) // Skip first two rooms (player start and adjacent)
            .take(3) // Limit to 3 chests per tile
            .collect();

        for &(rx, ry) in chest_rooms {
            if rng.gen_ratio(3, 10) {
                // 30% chance for chest in each room
                let chest_types = ["wooden_chest", "supply_crate", "glass_cache"];
                let chest_id = chest_types[rng.gen_range(0..chest_types.len())];

                // Generate loot for the chest
                let mut chest = Chest::new(rx, ry, chest_id);
                if let Some(def) = super::chest::get_chest_def(chest_id)
                    && let Some(loot_table) = &def.loot_table
                {
                    let loot = generate_loot(loot_table, rx, ry, &mut rng);
                    for item in loot {
                        chest.add_item(item);
                    }
                }
                chests.push(chest);
            }
        }

        // Place micro-structures
        let biome_str = match biome {
            super::world_map::Biome::Saltflat => "saltflat",
            super::world_map::Biome::Oasis => "oasis",
            super::world_map::Biome::Ruins => "ruins",
            super::world_map::Biome::Scrubland => "scrubland",
            _ => "saltflat",
        };

        let (microstructures, mut structure_npcs, mut structure_chests, mut structure_items) =
            place_microstructures(&mut map, biome_str, &rooms, (px, py), &mut rng);

        // Add structure entities to main collections
        npcs.append(&mut structure_npcs);
        chests.append(&mut structure_chests);
        items.append(&mut structure_items);

        let ambient = 100u8;
        let light_sources = vec![LightSource {
            x: px,
            y: py,
            radius: 8,
            intensity: 120,
        }]; // Reduced from 150 to avoid glare
        let light_map = compute_lighting(&light_sources, ambient);

        let mut player = PlayerState::new();
        player.x = px;
        player.y = py;
        player.hp = 20;
        player.max_hp = 20;
        player.reflex = 5;
        player.salt_scrip = 50;
        if let Some(first_quest) = super::quest::ActiveQuest::new("pilgrims_last_angle") {
            player.quest_log.active.push(first_quest);
        }

        let world = WorldState {
            world_map: Some(world_map),
            world_x,
            world_y,
            layer: 0,
            map,
            enemies,
            npcs,
            items,
            chests,
            interactables: Vec::new(),
            microstructures,
            storm: Storm::forecast(&mut ChaCha8Rng::seed_from_u64(seed + 1)),
            time_of_day: 8,
            weather: Weather::Clear,
            ambient_light: ambient,
            visual_effects: super::visual_effects::VisualEffects::default(),
            light_map: light_map.clone(),
            encounter_state: None,
            encounter_history: HashMap::new(),
            total_tiles_traveled: 0,
            world_map_target: None,
            world_map_path: Vec::new(),
            saved_on_world_map: false,
            enemy_positions: HashMap::new(),
            npc_positions: HashMap::new(),
            item_positions: HashMap::new(),
            chest_positions: HashMap::new(),
            interactable_positions: HashMap::new(),
        };

        let mut state = Self {
            player,
            world,
            visible: visible.clone(),
            revealed: visible,
            light_map,
            messages: vec![
                GameMessage::new("Welcome to the Saltglass Steppe.", MsgType::System, 0),
                GameMessage::new("Quest added: The Pilgrim's Last Angle", MsgType::System, 0),
            ],
            turn: 0,
            rng,
            triggered_effects: Vec::new(),
            decoys: Vec::new(),
            spatial: SpatialIndex { dirty: true, ..Default::default() },
            debug: DebugState::default(),
            pending_ui: PendingUi::default(),
            meta: super::meta::MetaProgress::load(),
            wait_counter: 0,
            narrative: NarrativeEngine::default(),
            map_features: MapFeatures::new(),
            seed,
            trace: Default::default(),
        };

        // Materialize terrain-forge markers into entities
        crate::game::generation::feature_materializer::materialize_features(
            &mut state, biome, terrain, poi, level,
        );

        state.rebuild_spatial_index();
        state
    }

    /// VERA dispatch — central command handler
    pub fn dispatch(&mut self, command: super::effects::Command) {
        use super::effects::Command;
        match command {
            Command::Move { dx, dy } => self.dispatch_move(dx, dy),
            Command::Attack { target_x, target_y } => {
                self.dispatch_melee_attack(target_x, target_y)
            }
            Command::RangedAttack { target_x, target_y } => {
                self.dispatch_ranged_attack(target_x, target_y)
            }
            Command::Wait => {
                let output = {
                    let ctx = super::effects::context::QueryContext::from_state(self);
                    super::rules::rule_wait(&ctx)
                };
                self.apply_and_trace(output, "rule_wait");
                self.end_turn();
            }
            Command::Rest => {
                let output = {
                    let ctx = super::effects::context::QueryContext::from_state(self);
                    super::rules::rule_rest(&ctx)
                };
                let had_effects = !output.effects.is_empty();
                self.apply_and_trace(output, "rule_rest");
                if had_effects {
                    // Rest advances turns internally; also run AI and housekeeping
                    for _ in 0..10 {
                        self.tick_turn_housekeeping();
                    }
                    self.update_enemies();
                }
            }
            Command::Equip { inv_idx, slot } => {
                let output = {
                    let ctx = super::effects::context::QueryContext::from_state(self);
                    super::rules::rule_equip(inv_idx, &slot, &ctx)
                };
                self.apply_and_trace(output, "rule_equip");
            }
            Command::Unequip { slot } => {
                let output = super::rules::rule_unequip(&slot);
                self.apply_and_trace(output, "rule_unequip");
            }
            Command::AllocateStat { stat } => {
                let output = {
                    let ctx = super::effects::context::QueryContext::from_state(self);
                    super::rules::rule_allocate_stat(&stat, &ctx)
                };
                self.apply_and_trace(output, "rule_allocate_stat");
            }
            Command::AcceptQuest { quest_id } => {
                self.dispatch_accept_quest(&quest_id);
            }
            Command::CompleteQuest { quest_id } => {
                self.dispatch_complete_quest(&quest_id);
            }
            Command::Interact { x, y } => {
                self.dispatch_interact(x, y);
            }
            Command::Examine { x, y } => {
                self.dispatch_examine(x, y);
            }
            Command::UsePsychic { ability_id } => {
                self.dispatch_use_psychic(&ability_id);
            }
            Command::FleeEncounter => {
                self.dispatch_flee_encounter();
            }
            Command::WorldMove { new_wx, new_wy } => {
                self.dispatch_world_move(new_wx, new_wy);
            }
            Command::WorldMoveSafe { new_wx, new_wy } => {
                self.dispatch_world_move_safe(new_wx, new_wy);
            }
            Command::EnterSubterranean => {
                self.enter_subterranean();
            }
            Command::ExitSubterranean => {
                self.exit_subterranean();
            }
            Command::FollowWorldPath => {
                self.dispatch_follow_world_path();
            }
            Command::CalculateWorldPath { target_wx, target_wy } => {
                self.dispatch_calculate_world_path((target_wx, target_wy));
            }
            _ => {
                let output = {
                    let ctx = super::effects::context::QueryContext::from_state(self);
                    match &command {
                        Command::UseItem { index } => {
                            super::rules::rule_use_item(*index, &ctx)
                        }
                        Command::UseItemOnTile { index, x, y } => {
                            super::rules::rule_use_item_on_tile(*index, *x, *y, &ctx)
                        }
                        _ => unreachable!(),
                    }
                };
                self.apply_and_trace(output, command.name());
            }
        }
    }

    /// Dispatch movement: rule determines outcome, imperative fallback for NPC/combat
    fn dispatch_move(&mut self, dx: i32, dy: i32) {
        use super::effects::MoveResult;

        self.ensure_spatial_index();
        let move_output = {
            let ctx = super::effects::context::QueryContext {
                player: &self.player,
                map: &self.world.map,
                revealed_count: self.revealed.len(),
                tile_count: self.world.map.tiles.len(),
                npc_positions: &self.spatial.npc_positions,
                enemy_positions: &self.spatial.enemy_positions,
                enemies: &self.world.enemies,
                visible: &self.visible,
                debug_phase: self.debug.phase,
                mock_combat_hit: self.debug.mock_combat_hit,
                mock_combat_damage: self.debug.mock_combat_damage,
                wait_counter: self.wait_counter,
                turn: self.turn,
                time_of_day: self.world.time_of_day,
                encounter_state: self.world.encounter_state.as_ref(),
                player_adaptations: &self.player.adaptations,
                player_refraction: self.player.refraction,
            };
            super::rules::rule_move(dx, dy, &ctx, &mut self.rng)
        };

        // Apply effects common to all branches (e.g. ResetWaitCounter)
        let turn = self.turn;
        for effect in &move_output.effects {
            self.apply_effect(effect);
            self.trace.record(
                effect,
                super::effects::TraceSource::Rule { name: "rule_move" },
                turn,
            );
        }
        for p in &move_output.presentation {
            self.apply_presentation(p);
        }

        match move_output.result {
            MoveResult::Npc => {
                // Legacy NPC interaction path (Phase 3+ migration)
                let new_x = self.player.x + dx;
                let new_y = self.player.y + dy;
                MovementSystem::handle_npc_interaction_legacy(self, new_x, new_y);
            }
            MoveResult::Combat => {
                let new_x = self.player.x + dx;
                let new_y = self.player.y + dy;
                self.dispatch_melee_attack(new_x, new_y);
            }
            MoveResult::Moved => {
                // Derives: FOV, lighting, pickup, world transition, adaptation, auto-end-turn
                self.update_fov();
                self.update_lighting();
                MovementSystem::pickup_items(self);
                let tile = self.world.map.get(self.player.x, self.player.y).cloned();
                if let Some(t) = tile {
                    MovementSystem::handle_world_transition(self, &t, self.player.x, self.player.y);
                }
                self.check_adaptation_threshold();
                self.check_auto_end_turn();
            }
            MoveResult::Blocked => {}
        }
    }

    /// Dispatch melee attack: rule produces effects, post-processing handles behaviors
    fn dispatch_melee_attack(&mut self, target_x: i32, target_y: i32) {
        self.ensure_spatial_index();
        let output = {
            let ctx = super::effects::context::QueryContext {
                player: &self.player,
                map: &self.world.map,
                revealed_count: self.revealed.len(),
                tile_count: self.world.map.tiles.len(),
                npc_positions: &self.spatial.npc_positions,
                enemy_positions: &self.spatial.enemy_positions,
                enemies: &self.world.enemies,
                visible: &self.visible,
                debug_phase: self.debug.phase,
                mock_combat_hit: self.debug.mock_combat_hit,
                mock_combat_damage: self.debug.mock_combat_damage,
                wait_counter: self.wait_counter,
                turn: self.turn,
                time_of_day: self.world.time_of_day,
                encounter_state: self.world.encounter_state.as_ref(),
                player_adaptations: &self.player.adaptations,
                player_refraction: self.player.refraction,
            };
            super::rules::rule_melee_attack(target_x, target_y, &ctx, &mut self.rng)
        };

        // Check what happened for post-processing
        let mut killed_idx = None;
        let mut damage_dealt = 0i32;
        let mut hit = false;
        for effect in &output.effects {
            match effect {
                super::effects::Effect::Combat(super::effects::CombatEffect::Kill {
                    enemy_idx,
                    ..
                }) => {
                    killed_idx = Some(*enemy_idx);
                }
                super::effects::Effect::Combat(super::effects::CombatEffect::DealDamage {
                    amount,
                    ..
                }) => {
                    damage_dealt = *amount;
                    hit = true;
                }
                _ => {}
            }
        }

        // Apply effects and trace
        let applied_effects = output.effects.clone();
        self.apply_and_trace(output, "rule_melee_attack");

        // Run reactions (currently no-op, infrastructure for future phases)
        self.run_reactions(&applied_effects, 0);

        // Post-processing: visual effects and behaviors (imperative, not rule-based)
        if hit {
            self.trigger_hit_flash(target_x, target_y);
            self.spawn_damage_number(target_x, target_y, damage_dealt, false);
        }

        // Swarm aggro
        if let Some(ei) = self.enemy_at(target_x, target_y).or(killed_idx)
            && let Some(enemy) = self.world.enemies.get(ei)
            && enemy.def().map(|d| d.swarm).unwrap_or(false)
        {
            let id = enemy.id.clone();
            let ex = enemy.x;
            let ey = enemy.y;
            CombatSystem::trigger_swarm_aggro(self, &id, ex, ey, 8);
        }

        // On-hit effects and reflect damage
        if hit
            && killed_idx.is_none()
            && let Some(ei) = self.enemy_at(target_x, target_y)
            && let Some(def) = self.world.enemies[ei].def()
        {
            for e in &def.effects {
                if e.condition == "on_hit" {
                    self.trigger_effect(&e.effect, 2);
                }
            }
            for behavior in &def.behaviors {
                if behavior.behavior_type == "reflect_damage" {
                    let percent = behavior.percent.unwrap_or(25);
                    let reflected = (damage_dealt as u32 * percent / 100) as i32;
                    if reflected > 0 {
                        self.player.hp -= reflected;
                        self.log_typed(
                            format!("The enemy reflects {} damage back at you!", reflected),
                            MsgType::Combat,
                        );
                    }
                }
            }
        }

        // On-death effects, XP, split on death
        if let Some(ki) = killed_idx {
            CombatSystem::process_enemy_death_post(self, ki);
        }

        self.check_auto_end_turn();
    }

    /// Dispatch ranged attack: rule produces effects, post-processing handles behaviors
    fn dispatch_ranged_attack(&mut self, target_x: i32, target_y: i32) {
        self.ensure_spatial_index();

        // Spawn projectile before the rule (visual only)
        let weapon_range = self
            .player
            .equipped_weapon
            .as_ref()
            .and_then(|id| super::combat::get_weapon_def(id))
            .map(|w| w.range)
            .unwrap_or(0);
        if weapon_range > 1 {
            let proj_char = if weapon_range > 3 { '*' } else { '-' };
            self.spawn_projectile(
                (self.player.x, self.player.y),
                (target_x, target_y),
                proj_char,
            );
        }

        let output = {
            let ctx = super::effects::context::QueryContext {
                player: &self.player,
                map: &self.world.map,
                revealed_count: self.revealed.len(),
                tile_count: self.world.map.tiles.len(),
                npc_positions: &self.spatial.npc_positions,
                enemy_positions: &self.spatial.enemy_positions,
                enemies: &self.world.enemies,
                visible: &self.visible,
                debug_phase: self.debug.phase,
                mock_combat_hit: self.debug.mock_combat_hit,
                mock_combat_damage: self.debug.mock_combat_damage,
                wait_counter: self.wait_counter,
                turn: self.turn,
                time_of_day: self.world.time_of_day,
                encounter_state: self.world.encounter_state.as_ref(),
                player_adaptations: &self.player.adaptations,
                player_refraction: self.player.refraction,
            };
            super::rules::rule_ranged_attack(target_x, target_y, &ctx, &mut self.rng)
        };

        let mut killed_idx = None;
        let mut damage_dealt = 0i32;
        let mut hit = false;
        for effect in &output.effects {
            match effect {
                super::effects::Effect::Combat(super::effects::CombatEffect::Kill {
                    enemy_idx,
                    ..
                }) => {
                    killed_idx = Some(*enemy_idx);
                }
                super::effects::Effect::Combat(super::effects::CombatEffect::DealDamage {
                    amount,
                    ..
                }) => {
                    damage_dealt = *amount;
                    hit = true;
                }
                _ => {}
            }
        }

        let applied_effects = output.effects.clone();
        self.apply_and_trace(output, "rule_ranged_attack");
        self.run_reactions(&applied_effects, 0);

        if hit {
            self.trigger_hit_flash(target_x, target_y);
            self.spawn_damage_number(target_x, target_y, damage_dealt, false);
        }

        // Swarm aggro
        if let Some(ei) = self.enemy_at(target_x, target_y).or(killed_idx)
            && let Some(enemy) = self.world.enemies.get(ei)
            && enemy.def().map(|d| d.swarm).unwrap_or(false)
        {
            let id = enemy.id.clone();
            let ex = enemy.x;
            let ey = enemy.y;
            CombatSystem::trigger_swarm_aggro(self, &id, ex, ey, 8);
        }

        if let Some(ki) = killed_idx {
            CombatSystem::process_enemy_death_post(self, ki);
        }

        self.check_auto_end_turn();
    }

    /// Apply a RuleOutput: effects → apply + trace, presentation → log
    fn dispatch_accept_quest(&mut self, quest_id: &str) {
        use super::effects::{Effect, QuestEffect, Presentation};

        let can_accept = self.player.quest_log.is_quest_available(quest_id, self);
        if !can_accept { return; }

        let mut effects: Vec<Effect> = vec![
            Effect::Quest(QuestEffect::Accept { quest_id: quest_id.to_string() }),
        ];
        let mut presentation = Vec::new();

        if let Some(def) = super::quest::get_quest_def(quest_id) {
            presentation.push(Presentation::LogMessage {
                text: format!("Quest accepted: {}", def.name),
                msg_type: "system".into(),
            });
            if def.category == "main" && quest_id.starts_with("faction_choice_") {
                let faction = if quest_id.contains("monks") { "Mirror Monks" }
                    else if quest_id.contains("engineers") { "Sand-Engineers" }
                    else if quest_id.contains("glassborn") { "Glassborn" }
                    else { "" };
                if !faction.is_empty() {
                    effects.push(Effect::Quest(QuestEffect::SetFactionAlignment {
                        faction: faction.to_string(),
                    }));
                    presentation.push(Presentation::LogMessage {
                        text: format!("You have aligned with the {}", faction),
                        msg_type: "system".into(),
                    });
                }
            }
        }

        let output = super::effects::RuleOutput { effects, presentation };
        self.apply_and_trace(output, "rule_accept_quest");
    }

    fn dispatch_complete_quest(&mut self, quest_id: &str) {
        use super::effects::{Effect, PlayerEffect, ItemEffect, Presentation};

        // complete() moves quest to completed and returns reward
        let reward = match self.player.quest_log.complete(quest_id) {
            Some(r) => r,
            None => return,
        };

        let mut effects = Vec::new();
        let mut presentation = Vec::new();

        if let Some(def) = super::quest::get_quest_def(quest_id) {
            presentation.push(Presentation::LogMessage {
                text: format!("Quest completed: {}", def.name),
                msg_type: "system".into(),
            });
        }

        if reward.xp > 0 {
            effects.push(Effect::Player(PlayerEffect::GainXp { amount: reward.xp }));
        }
        if reward.salt_scrip > 0 {
            effects.push(Effect::Player(PlayerEffect::GainSaltScrip { amount: reward.salt_scrip }));
            presentation.push(Presentation::LogMessage {
                text: format!("Received {} salt scrip", reward.salt_scrip),
                msg_type: "loot".into(),
            });
        }
        for item_id in &reward.items {
            effects.push(Effect::Item(ItemEffect::AddToInventory { item_id: item_id.clone() }));
        }
        for (faction_id, delta) in &reward.reputation_rewards {
            effects.push(Effect::Player(PlayerEffect::ModifyReputation {
                faction: faction_id.clone(),
                delta: *delta,
            }));
        }
        for unlocked_id in &reward.unlocks_quests {
            if let Some(unlocked_def) = super::quest::get_quest_def(unlocked_id) {
                presentation.push(Presentation::LogMessage {
                    text: format!("New quest available: {}", unlocked_def.name),
                    msg_type: "system".into(),
                });
            }
        }

        let output = super::effects::RuleOutput { effects, presentation };
        self.apply_and_trace(output, "rule_complete_quest");
    }

    fn dispatch_use_psychic(&mut self, ability_id: &str) {
        match self.player.psychic.use_ability(ability_id) {
            Ok(effect_id) => {
                let output = {
                    let ctx = super::effects::context::QueryContext::from_state(self);
                    super::rules::rule_use_psychic(&effect_id, &ctx)
                };
                self.apply_and_trace(output, "rule_use_psychic");
            }
            Err(e) => self.log(e),
        }
    }

    fn dispatch_flee_encounter(&mut self) {
        use super::effects::{Effect, PlayerEffect, Presentation};

        let encounter = match &self.world.encounter_state {
            Some(e) => e.clone(),
            None => {
                self.log("No active encounter.");
                return;
            }
        };

        let difficulty_mod = 1.0;
        if !encounter.can_flee(self.turn, difficulty_mod) {
            self.log("You cannot flee yet!");
            return;
        }

        match super::encounter::attempt_flee(
            self.player.x,
            self.player.y,
            &self.world.enemies,
            &encounter.spawned_enemies,
            &mut self.rng,
            self.player.skills.get_skill_level("wayfaring"),
        ) {
            Ok(()) => {
                let output = super::effects::RuleOutput {
                    effects: vec![Effect::Player(PlayerEffect::ClearEncounter)],
                    presentation: vec![Presentation::LogMessage {
                        text: "You successfully flee the encounter!".into(),
                        msg_type: "status".into(),
                    }],
                };
                self.apply_and_trace(output, "rule_flee_encounter");
            }
            Err(e) => {
                let output = super::effects::RuleOutput {
                    effects: vec![Effect::Player(PlayerEffect::SetLastFleeAttempt { turn: self.turn })],
                    presentation: vec![Presentation::LogMessage {
                        text: e,
                        msg_type: "warning".into(),
                    }],
                };
                self.apply_and_trace(output, "rule_flee_encounter");
            }
        }
    }

    fn dispatch_world_move(&mut self, new_wx: usize, new_wy: usize) {
        use super::effects::{Effect, PlayerEffect};

        let from = (self.world.world_x, self.world.world_y);
        let to = (new_wx, new_wy);

        if !super::travel::is_adjacent(from, to) {
            self.log("Too far to travel in one step. Move to an adjacent tile.");
            return;
        }

        let wm = match &self.world.world_map {
            Some(wm) => wm,
            None => return,
        };

        let (biome, terrain, _elev, _poi, _res, _conn, level) = wm.get(new_wx, new_wy);
        let cost = super::travel::travel_cost(terrain, biome);

        let mut effects: Vec<Effect> = vec![
            Effect::Player(PlayerEffect::IncrementTilesTraveled),
        ];
        for _ in 0..cost {
            effects.push(Effect::Player(PlayerEffect::AdvanceTurn));
        }

        // Check for encounter
        let last_encounter = self.world.encounter_history.get(&(new_wx, new_wy)).copied().unwrap_or(0);
        let encounter_triggered = super::encounter::should_trigger_encounter(
            self.seed, new_wx, new_wy, self.world.total_tiles_traveled + 1,
            level, last_encounter, self.turn + cost, self.player.skills.get_skill_level("wayfaring"),
        );

        effects.push(Effect::Player(PlayerEffect::SetWorldPosition { wx: new_wx, wy: new_wy }));

        let output = super::effects::RuleOutput {
            effects,
            presentation: Vec::new(),
        };
        self.apply_and_trace(output, "dispatch_world_move");

        if encounter_triggered {
            let encounter = super::encounter::generate_encounter(
                self.seed, new_wx, new_wy, self.world.total_tiles_traveled,
                level, biome.as_str(),
            );
            let encounter_msg = match &encounter.encounter_type {
                super::encounter::EncounterType::Hostile { threat_points } =>
                    format!("⚔ Hostile encounter! (Threat: {})", threat_points),
                super::encounter::EncounterType::Neutral { description, .. } =>
                    description.clone(),
                super::encounter::EncounterType::Beneficial { boon_points } =>
                    format!("✨ You discover something! (Value: {})", boon_points),
            };
            self.world.encounter_state = Some(encounter);
            self.world.encounter_history.insert((new_wx, new_wy), self.turn);
            self.travel_to_tile(new_wx, new_wy);
            self.spawn_encounter_entities();
            self.pending_ui.dialogue = Some(("Encounter!".to_string(), encounter_msg));
        }
    }

    fn dispatch_world_move_safe(&mut self, new_wx: usize, new_wy: usize) {
        use super::effects::{Effect, PlayerEffect};

        let from = (self.world.world_x, self.world.world_y);
        let to = (new_wx, new_wy);
        let is_same_tile = from == to;

        if !is_same_tile && !super::travel::is_adjacent(from, to) {
            self.log("Too far to travel in one step. Move to an adjacent tile.");
            return;
        }

        if !is_same_tile
            && let Some(wm) = &self.world.world_map
        {
                let (biome, terrain, _elev, _poi, _res, _conn, _level) = wm.get(new_wx, new_wy);
                let cost = super::travel::travel_cost(terrain, biome);

                let mut effects: Vec<Effect> = vec![
                    Effect::Player(PlayerEffect::IncrementTilesTraveled),
                ];
                for _ in 0..cost {
                    effects.push(Effect::Player(PlayerEffect::AdvanceTurn));
                }

                let output = super::effects::RuleOutput { effects, presentation: Vec::new() };
                self.apply_and_trace(output, "dispatch_world_move_safe");
        }

        // Always generate tile
        self.travel_to_tile(new_wx, new_wy);
    }

    fn dispatch_follow_world_path(&mut self) {
        if self.world.world_map_path.is_empty() {
            return;
        }

        let next_pos = self.world.world_map_path.remove(0);
        self.dispatch_world_move(next_pos.0, next_pos.1);

        // If encounter triggered, clear path
        if self.world.encounter_state.is_some() {
            self.world.world_map_path.clear();
            self.world.world_map_target = None;
            return;
        }

        // Check if we reached the target
        if let Some(target) = self.world.world_map_target
            && (self.world.world_x, self.world.world_y) == target
        {
            self.world.world_map_target = None;
            self.world.world_map_path.clear();
        }
    }

    fn dispatch_calculate_world_path(&mut self, target: (usize, usize)) {
        use super::effects::{Effect, MapEffect};

        if self.world.world_map.is_none() { return; }

        let start = (self.world.world_x, self.world.world_y);
        let mut path = Vec::new();
        let mut current = start;
        let max_steps = 500;
        let mut steps = 0;

        while current != target && steps < max_steps {
            let (cx, cy) = current;
            let (tx, ty) = target;
            let next = if cx < tx { (cx + 1, cy) }
                else if cx > tx { (cx - 1, cy) }
                else if cy < ty { (cx, cy + 1) }
                else if cy > ty { (cx, cy - 1) }
                else { break };
            path.push(next);
            current = next;
            steps += 1;
        }

        if !path.is_empty() {
            let output = super::effects::RuleOutput {
                effects: vec![Effect::Map(MapEffect::SetWorldPath {
                    path: path.clone(),
                    target: Some(target),
                })],
                presentation: Vec::new(),
            };
            self.apply_and_trace(output, "dispatch_calculate_world_path");
        }
    }

    fn dispatch_interact(&mut self, x: i32, y: i32) {
        use super::rules::actions::{InteractTarget, rule_interact};
        self.ensure_spatial_index();

        let target = if let Some(&idx) = self.spatial.interactable_positions.get(&(x, y))
            && let Some(interactable) = self.world.interactables.get_mut(idx)
        {
            let id = interactable.id.clone();
            let message = interactable.interact();
            InteractTarget::Interactable { id, message }
        } else if let Some(&idx) = self.spatial.npc_positions.get(&(x, y))
            && let Some(npc) = self.world.npcs.get(idx)
        {
            InteractTarget::Npc { id: npc.id.clone(), name: npc.name().to_string() }
        } else if let Some(&idx) = self.spatial.chest_positions.get(&(x, y))
            && let Some(chest) = self.world.chests.get(idx)
        {
            InteractTarget::Chest { name: chest.name().to_string() }
        } else {
            InteractTarget::Nothing
        };

        let output = rule_interact(target);
        self.apply_and_trace(output, "rule_interact");
        self.spatial.dirty = true;
    }

    fn dispatch_examine(&mut self, x: i32, y: i32) {
        use super::rules::actions::{ExamineTarget, rule_examine};
        use super::map::Tile;
        self.ensure_spatial_index();

        let target = if let Some(&idx) = self.spatial.interactable_positions.get(&(x, y))
            && let Some(interactable) = self.world.interactables.get(idx)
        {
            let id = interactable.id.clone();
            let message = interactable.examine();
            ExamineTarget::Interactable { id, message }
        } else if let Some(&idx) = self.spatial.enemy_positions.get(&(x, y))
            && let Some(enemy) = self.world.enemies.get(idx)
            && enemy.hp > 0
        {
            let max_hp = enemy.max_hp().unwrap_or(0);
            ExamineTarget::Enemy { name: enemy.name().to_string(), hp: enemy.hp, max_hp }
        } else if let Some(&idx) = self.spatial.npc_positions.get(&(x, y))
            && let Some(npc) = self.world.npcs.get(idx)
        {
            ExamineTarget::Npc { name: npc.name().to_string(), description: npc.description().to_string() }
        } else if let Some(indices) = self.spatial.item_positions.get(&(x, y))
            && !indices.is_empty()
            && let Some(item) = self.world.items.get(indices[0])
        {
            ExamineTarget::Item { name: item.name().to_string() }
        } else if let Some(&idx) = self.spatial.chest_positions.get(&(x, y))
            && let Some(chest) = self.world.chests.get(idx)
        {
            ExamineTarget::Chest { name: chest.name().to_string(), description: chest.description().to_string() }
        } else {
            let tile = self.world.map.get_tile(x, y);
            let desc: &'static str = match tile {
                Tile::Wall { .. } => "A solid wall.",
                Tile::Floor { .. } => "The ground here is clear.",
                Tile::Glass => "Dangerous glass terrain that refracts light.",
                _ => "You examine the area.",
            };
            ExamineTarget::Tile { description: desc }
        };

        let output = rule_examine(target);
        self.apply_and_trace(output, "rule_examine");
    }

    pub fn dispatch_craft(&mut self, recipe_id: &str) -> bool {
        // Check station requirement before calling rule (needs &mut self for spatial)
        if let Some(recipe) = super::crafting::get_recipe(recipe_id)
            && let Some(ref station) = recipe.station_required
        {
                let has_station = {
                    let mut found = false;
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            let pos = (self.player.x + dx, self.player.y + dy);
                            if let Some(&idx) = self.spatial.interactable_positions.get(&pos)
                                && let Some(inter) = self.world.interactables.get(idx)
                                && &inter.id == station
                            {
                                found = true;
                            }
                        }
                    }
                    found
                };
                if !has_station {
                    self.log(format!("Requires a nearby {}.", station.replace('_', " ")));
                    return false;
                }
        }

        let ctx = super::effects::context::QueryContext::from_state(self);
        let output = super::rules::economy::rule_craft(recipe_id, &ctx);
        let success = !output.effects.is_empty();
        self.apply_and_trace(output, "rule_craft");
        success
    }

    pub fn dispatch_buy_item(&mut self, item_id: &str, npc_id: &str) -> Result<(), String> {
        let ctx = super::effects::context::QueryContext::from_state(self);
        let output = super::rules::economy::rule_buy_item(item_id, npc_id, &ctx);
        // If no effects, the rule produced a warning — extract it as an error
        if output.effects.is_empty() {
            let msg = output.presentation.first()
                .map(|p| match p {
                    super::effects::Presentation::LogMessage { text, .. } => text.clone(),
                })
                .unwrap_or_else(|| "Cannot buy item.".into());
            return Err(msg);
        }
        self.apply_and_trace(output, "rule_buy_item");
        Ok(())
    }

    pub fn dispatch_sell_item(&mut self, item_id: &str) -> Result<(), String> {
        let inv_idx = self.player.inventory.iter().position(|id| id == item_id)
            .ok_or_else(|| "You don't have that item.".to_string())?;
        let ctx = super::effects::context::QueryContext::from_state(self);
        let output = super::rules::economy::rule_sell_item(inv_idx, &ctx);
        if output.effects.is_empty() {
            let msg = output.presentation.first()
                .map(|p| match p {
                    super::effects::Presentation::LogMessage { text, .. } => text.clone(),
                })
                .unwrap_or_else(|| "Cannot sell item.".into());
            return Err(msg);
        }
        self.apply_and_trace(output, "rule_sell_item");
        Ok(())
    }

    pub fn apply_and_trace(
        &mut self,
        output: super::effects::RuleOutput,
        rule_name: &'static str,
    ) {
        let turn = self.turn;
        for effect in &output.effects {
            self.apply_effect(effect);
            self.trace.record(
                effect,
                super::effects::TraceSource::Rule { name: rule_name },
                turn,
            );
        }
        for p in &output.presentation {
            self.apply_presentation(p);
        }
    }

    /// Run reactions triggered by applied effects. Max cascade depth 10.
    /// Run reactions triggered by applied effects. Max cascade depth 10.
    ///
    /// Kill → loot drop + quest progress via collect_reactions.
    pub fn run_reactions(&mut self, effects: &[super::effects::Effect], depth: u32) {
        if depth >= 10 {
            self.log("Warning: reaction cascade depth limit reached");
            return;
        }
        let reaction_effects = self.collect_reactions(effects);
        if reaction_effects.is_empty() {
            return;
        }
        let turn = self.turn;
        for (effect, source_name, trigger) in &reaction_effects {
            self.apply_effect(effect);
            self.trace.record(
                effect,
                super::effects::TraceSource::Reaction {
                    name: source_name,
                    trigger: Box::new(trigger.clone()),
                },
                turn,
            );
        }
        let next: Vec<_> = reaction_effects.into_iter().map(|(e, _, _)| e).collect();
        self.run_reactions(&next, depth + 1);
    }

    fn collect_reactions(
        &self,
        effects: &[super::effects::Effect],
    ) -> Vec<(super::effects::Effect, &'static str, super::effects::Effect)> {
        use super::effects::{CombatEffect, Effect, EventEffect, QuestNotifyKind};
        let mut results = Vec::new();
        for effect in effects {
            if let Effect::Combat(CombatEffect::Kill { enemy_id, x, y, .. }) = effect {
                results.push((
                    Effect::Event(EventEffect::LootDrop {
                        enemy_id: enemy_id.clone(),
                        x: *x,
                        y: *y,
                    }),
                    "reaction_loot_drop",
                    effect.clone(),
                ));
                results.push((
                    Effect::Event(EventEffect::QuestNotify {
                        kind: QuestNotifyKind::Kill {
                            enemy_id: enemy_id.clone(),
                        },
                    }),
                    "reaction_quest_kill",
                    effect.clone(),
                ));
            }
        }
        results
    }

    /// Create a new game with a specific character class
    pub fn new_with_class(seed: u64, class_id: &str) -> Self {
        let mut state = Self::new(seed);

        if let Some(class) = super::meta::get_class(class_id) {
            state.player.hp = class.starting_hp;
            state.player.max_hp = class.starting_hp;
            state.player.ap = class.starting_ap;
            state.player.max_ap = class.starting_ap;

            // Add starting items
            for item_id in &class.starting_items {
                state.player.inventory.push(item_id.clone());
            }

            // Add starting adaptations
            for adapt_id in &class.starting_adaptations {
                if let Some(adapt) = Adaptation::from_id(adapt_id) {
                    state.player.adaptations.push(adapt);
                }
            }

            // Add starting faction reputation
            let starting_rep = super::faction::get_starting_reputation(class_id);
            for (faction_id, rep) in starting_rep {
                state.player.faction_reputation.insert(faction_id, rep);
            }

            state.log(format!("You begin as a {}.", class.name));
        }

        state
    }

    /// Ensure spatial index is up to date before querying
    fn ensure_spatial_index(&mut self) {
        if self.spatial.dirty {
            self.rebuild_spatial_index_internal();
        }
    }

    /// Internal rebuild that clears the dirty flag
    fn rebuild_spatial_index_internal(&mut self) {
        self.spatial.enemy_positions.clear();
        for (i, e) in self.world.enemies.iter().enumerate() {
            if e.hp > 0 {
                self.spatial.enemy_positions.insert((e.x, e.y), i);
            }
        }
        self.spatial.npc_positions.clear();
        for (i, n) in self.world.npcs.iter().enumerate() {
            self.spatial.npc_positions.insert((n.x, n.y), i);
        }
        self.spatial.item_positions.clear();
        for (i, item) in self.world.items.iter().enumerate() {
            self.spatial.item_positions
                .entry((item.x, item.y))
                .or_default()
                .push(i);
        }
        self.spatial.chest_positions.clear();
        for (i, chest) in self.world.chests.iter().enumerate() {
            self.spatial.chest_positions.insert((chest.x, chest.y), i);
        }
        self.spatial.interactable_positions.clear();
        for (i, interactable) in self.world.interactables.iter().enumerate() {
            self.spatial.interactable_positions
                .insert((interactable.x, interactable.y), i);
        }
        self.spatial.dirty = false;
    }

    /// Rebuild spatial index (public, for backwards compatibility)
    pub fn rebuild_spatial_index(&mut self) {
        self.rebuild_spatial_index_internal();
    }

    /// Get quest IDs that have objectives at the given world coordinates
    pub fn get_quest_ids_for_location(&self, world_x: usize, world_y: usize) -> Vec<String> {
        let mut quest_ids = Vec::new();

        for quest in &self.player.quest_log.active {
            if let Some(def) = quest.def() {
                for (i, quest_obj) in def.objectives.iter().enumerate() {
                    if !quest.objectives[i].completed
                        && let crate::game::quest::ObjectiveType::Reach { x, y } = &quest_obj.objective_type
                        && *x as usize == world_x && *y as usize == world_y
                    {
                        quest_ids.push(quest.quest_id.clone());
                        break;
                    }
                }
            }
        }

        quest_ids
    }

    /// Travel to a new world tile (lazy generation)
    pub fn travel_to_tile(&mut self, new_wx: usize, new_wy: usize) {
        use crate::game::generation::tile_generator::{TileParams, generate_tile};

        let params = TileParams::from_world_state(self, new_wx, new_wy);
        let tile = generate_tile(&params);

        self.world.world_x = new_wx;
        self.world.world_y = new_wy;
        self.world.map = tile.map;
        self.world.enemies = tile.enemies;
        self.world.items = tile.items;
        self.world.npcs = tile.npcs;
        self.world.chests = tile.chests;
        self.player.x = tile.spawn_pos.0;
        self.player.y = tile.spawn_pos.1;

        // post-load hooks (keep these in state.rs, they need &mut self)
        let biome = params.biome;
        let terrain = params.terrain;
        let poi = params.poi;
        let level = params.level;
        let walkable = tile.walkable_positions;
        let mut rng = ChaCha8Rng::seed_from_u64(params.seed);

        crate::game::generation::feature_materializer::materialize_features(
            self, biome, terrain, poi, level,
        );
        if poi == super::world_map::POI::Town {
            self.spawn_crafting_stations(&walkable, &mut rng);
        }
        self.spawn_quest_required_npcs();
        self.update_fov();
        self.rebuild_spatial_index();
        self.update_lighting();
        self.generate_crystal_formations(&biome, &walkable, &mut rng);
        self.log(format!("You enter a new area ({:?} {:?}).", biome, terrain));
    }

    /// Spawn crafting stations at random walkable positions
    fn spawn_crafting_stations(
        &mut self,
        walkable: &[(i32, i32)],
        rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        use rand::seq::SliceRandom;
        let station_ids = ["crafting_table", "glass_forge"];
        let occupied: std::collections::HashSet<(i32, i32)> = self
            .world
            .interactables
            .iter()
            .map(|i| (i.x, i.y))
            .chain(self.world.npcs.iter().map(|n| (n.x, n.y)))
            .collect();
        let free: Vec<_> = walkable.iter().filter(|p| !occupied.contains(p)).collect();
        for id in &station_ids {
            if let Some(&&(x, y)) = free.choose(rng) {
                self.world
                    .interactables
                    .push(super::interactable::Interactable::new(id.to_string(), x, y));
            }
        }
    }

    /// Spawn NPCs required for active quests
    fn spawn_quest_required_npcs(&mut self) {
        // Check all active quests for NPCs that need to be spawned
        let mut required_npcs = Vec::new();

        for quest in &self.player.quest_log.active {
            if let Some(def) = super::quest::get_quest_def(&quest.quest_id) {
                for objective in &def.objectives {
                    if let super::quest::ObjectiveType::TalkTo { npc_id } =
                        &objective.objective_type
                    {
                        // Check if this NPC is already spawned
                        if !self.world.npcs.iter().any(|npc| npc.id == *npc_id) {
                            required_npcs.push(npc_id.clone());
                        }
                    }
                }
            }
        }

        // Spawn required NPCs at safe positions
        for npc_id in required_npcs {
            let spawn_pos = self.find_safe_spawn_position();
            if let Some((x, y)) = spawn_pos {
                self.world.npcs.push(super::npc::Npc::new(x, y, &npc_id));
                self.log(format!("A {} appears nearby.", npc_id.replace('_', " ")));
            }
        }
    }

    /// Find a safe position to spawn an NPC (not on player, walls, or other entities)
    fn find_safe_spawn_position(&self) -> Option<(i32, i32)> {
        let offsets = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (2, 0),
            (-2, 0),
            (0, 2),
            (0, -2),
        ];

        for &(dx, dy) in &offsets {
            let test_x = self.player.x + dx;
            let test_y = self.player.y + dy;

            if test_x >= 0
                && test_y >= 0
                && test_x < self.world.map.width as i32
                && test_y < self.world.map.height as i32
            {
                let test_idx = self.world.map.idx(test_x, test_y);
                if self.world.map.tiles[test_idx].walkable() {
                    // Check if position is free of other entities
                    let position_free = !self
                        .world
                        .enemies
                        .iter()
                        .any(|e| e.x == test_x && e.y == test_y)
                        && !self
                            .world
                            .npcs
                            .iter()
                            .any(|n| n.x == test_x && n.y == test_y)
                        && !self
                            .world
                            .items
                            .iter()
                            .any(|i| i.x == test_x && i.y == test_y);

                    if position_free {
                        return Some((test_x, test_y));
                    }
                }
            }
        }

        None
    }

    /// Travel to a world tile with safe spawn (not on wall/enemy/glass)
    /// Move on world map without generating tile (for fast worldmap travel)
    pub fn move_on_world_map(&mut self, new_wx: usize, new_wy: usize) -> Option<String> {
        use super::travel;

        let from = (self.world.world_x, self.world.world_y);
        let to = (new_wx, new_wy);

        // Reject non-adjacent travel
        if !travel::is_adjacent(from, to) {
            return Some("Too far to travel in one step. Move to an adjacent tile.".to_string());
        }

        // Calculate and apply travel cost
        if let Some(wm) = &self.world.world_map {
            let (biome, terrain, _elev, _poi, _res, _conn, level) = wm.get(new_wx, new_wy);
            let cost = travel::travel_cost(terrain, biome);
            self.turn += cost;
            self.world.total_tiles_traveled += 1;

            // Check for encounter
            let last_encounter = self
                .world
                .encounter_history
                .get(&(new_wx, new_wy))
                .copied()
                .unwrap_or(0);
            if super::encounter::should_trigger_encounter(
                self.seed,
                new_wx,
                new_wy,
                self.world.total_tiles_traveled,
                level,
                last_encounter,
                self.turn,
                self.player.skills.get_skill_level("wayfaring"),
            ) {
                // Generate encounter
                let encounter = super::encounter::generate_encounter(
                    self.seed,
                    new_wx,
                    new_wy,
                    self.world.total_tiles_traveled,
                    level,
                    biome.as_str(),
                );

                // Create encounter message for popup
                let encounter_msg = match &encounter.encounter_type {
                    super::encounter::EncounterType::Hostile { threat_points } => {
                        format!("⚔ Hostile encounter! (Threat: {})", threat_points)
                    }
                    super::encounter::EncounterType::Neutral { description, .. } => {
                        description.clone()
                    }
                    super::encounter::EncounterType::Beneficial { boon_points } => {
                        format!("✨ You discover something! (Value: {})", boon_points)
                    }
                };

                self.world.encounter_state = Some(encounter);
                self.world
                    .encounter_history
                    .insert((new_wx, new_wy), self.turn);

                // Update world position
                self.world.world_x = new_wx;
                self.world.world_y = new_wy;

                // Generate tile for encounter
                self.travel_to_tile(new_wx, new_wy);
                self.spawn_encounter_entities();

                return Some(encounter_msg);
            }

            // No encounter - just update position without generating tile
            self.world.world_x = new_wx;
            self.world.world_y = new_wy;
        }

        None
    }

    pub fn travel_to_tile_safe(&mut self, new_wx: usize, new_wy: usize) {
        use super::travel;

        let from = (self.world.world_x, self.world.world_y);
        let to = (new_wx, new_wy);

        // Reject non-adjacent travel
        if !travel::is_adjacent(from, to) {
            self.log("Too far to travel in one step. Move to an adjacent tile.");
            return;
        }

        // Calculate and apply travel cost before generating the tile
        if let Some(wm) = &self.world.world_map {
            let (biome, terrain, _elev, _poi, _res, _conn, level) = wm.get(new_wx, new_wy);
            let cost = travel::travel_cost(terrain, biome);
            self.turn += cost;
            self.world.total_tiles_traveled += 1;

            // Check for encounter
            let last_encounter = self
                .world
                .encounter_history
                .get(&(new_wx, new_wy))
                .copied()
                .unwrap_or(0);
            if super::encounter::should_trigger_encounter(
                self.seed,
                new_wx,
                new_wy,
                self.world.total_tiles_traveled,
                level,
                last_encounter,
                self.turn,
                self.player.skills.get_skill_level("wayfaring"),
            ) {
                // Generate encounter before entering tile
                let encounter = super::encounter::generate_encounter(
                    self.seed,
                    new_wx,
                    new_wy,
                    self.world.total_tiles_traveled,
                    level,
                    biome.as_str(),
                );

                // Log encounter message
                match &encounter.encounter_type {
                    super::encounter::EncounterType::Hostile { threat_points } => {
                        self.log_typed(
                            format!("⚔ Hostile encounter! (Threat: {})", threat_points),
                            MsgType::Warning,
                        );
                    }
                    super::encounter::EncounterType::Neutral { description, .. } => {
                        self.log_typed(description.clone(), MsgType::System);
                    }
                    super::encounter::EncounterType::Beneficial { boon_points } => {
                        self.log_typed(
                            format!("✨ You discover something! (Value: {})", boon_points),
                            MsgType::Loot,
                        );
                    }
                }

                self.world.encounter_state = Some(encounter);
                self.world
                    .encounter_history
                    .insert((new_wx, new_wy), self.turn);
            }

            self.log(format!(
                "Traveled to {:?} {:?} ({cost} turns).",
                terrain, biome
            ));
        }

        self.travel_to_tile(new_wx, new_wy);

        // Spawn encounter entities if needed
        if self.world.encounter_state.is_some() {
            self.spawn_encounter_entities();
        }

        // Find safe spawn position (not wall, glass, or enemy)
        let (mut px, mut py) = (self.player.x, self.player.y);

        // Check if current position is safe (walkable floor, no enemy)
        let is_safe = |map: &Map, enemies: &[Enemy], x: i32, y: i32| -> bool {
            match map.get(x, y) {
                Some(tile) if tile.walkable() => {
                    !enemies.iter().any(|e| e.x == x && e.y == y && e.hp > 0)
                }
                _ => false,
            }
        };

        if !is_safe(&self.world.map, &self.world.enemies, px, py) {
            // Search for safe position in expanding squares
            'search: for radius in 1i32..20 {
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        if dx.abs() == radius || dy.abs() == radius {
                            let nx = px + dx;
                            let ny = py + dy;
                            if is_safe(&self.world.map, &self.world.enemies, nx, ny) {
                                px = nx;
                                py = ny;
                                break 'search;
                            }
                        }
                    }
                }
            }
        }

        self.player.x = px;
        self.player.y = py;
        self.update_fov();
        self.update_lighting();
    }

    /// Spawn entities for the current encounter
    fn spawn_encounter_entities(&mut self) {
        use super::generation::spawn::{get_biome_spawn_table, weighted_pick_by_level_and_tier};

        let encounter = match &self.world.encounter_state {
            Some(e) => e.clone(),
            None => return,
        };

        let (biome, _, _, _, _, _, level) = match &self.world.world_map {
            Some(wm) => wm.get(encounter.world_x, encounter.world_y),
            None => return,
        };

        match &encounter.encounter_type {
            super::encounter::EncounterType::Hostile { threat_points } => {
                // Spawn enemies based on threat budget
                let table = get_biome_spawn_table(&biome);
                let mut remaining_threat = *threat_points;
                let mut spawned_indices = Vec::new();

                // Find spawn positions away from player
                let spawn_positions: Vec<(i32, i32)> = self
                    .world
                    .map
                    .tiles
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, tile)| {
                        if tile.walkable() {
                            let x = (idx % self.world.map.width) as i32;
                            let y = (idx / self.world.map.width) as i32;
                            let dist = (x - self.player.x).abs() + (y - self.player.y).abs();
                            if dist >= 15 { Some((x, y)) } else { None }
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut spawn_idx = 0;
                let mut spawned_count = 0;
                while remaining_threat > 0 && spawn_idx < spawn_positions.len() {
                    if let Some(enemy_id) =
                        weighted_pick_by_level_and_tier(&table.enemies, level, &mut self.rng, false)
                    {
                        // Estimate enemy threat (rough heuristic: level * 2)
                        let enemy_threat = (level * 2).min(remaining_threat);
                        remaining_threat = remaining_threat.saturating_sub(enemy_threat);

                        let (x, y) = spawn_positions[spawn_idx];
                        let enemy_index = self.world.enemies.len();
                        self.world.enemies.push(Enemy::new(x, y, enemy_id));
                        spawned_indices.push(enemy_index);
                        spawned_count += 1;
                        spawn_idx += 1;
                    } else {
                        break;
                    }
                }

                self.log(format!(
                    "Encounter spawned {} enemies (threat: {})",
                    spawned_count, threat_points
                ));

                // Update encounter state with spawned enemy indices
                if let Some(enc) = &mut self.world.encounter_state {
                    enc.spawned_enemies = spawned_indices;
                }
                self.rebuild_spatial_index();
            }
            super::encounter::EncounterType::Neutral { event_id, .. } => {
                match event_id.as_str() {
                    "trade_caravan" => {
                        // Spawn 1-2 trader NPCs
                        let trader_count = self.rng.gen_range(1..=2);
                        for _ in 0..trader_count {
                            if let Some((x, y)) = self.find_safe_spawn_position() {
                                self.world.npcs.push(Npc::new(x, y, "traveling_merchant"));
                            }
                        }
                        self.rebuild_spatial_index();
                    }
                    "animal_herd" => {
                        // Spawn 3-5 non-hostile animals
                        let animal_count = self.rng.gen_range(3..=5);
                        for _ in 0..animal_count {
                            if let Some((_x, _y)) = self.find_safe_spawn_position() {
                                // TODO: Add non-hostile animal enemy type in future
                                // For now, just log it
                            }
                        }
                        self.log("A herd of creatures grazes peacefully nearby.");
                    }
                    _ => {}
                }
            }
            super::encounter::EncounterType::Beneficial { boon_points } => {
                // Spawn items based on boon budget
                let table = get_biome_spawn_table(&biome);
                let mut remaining_boon = *boon_points;
                let mut spawned_indices = Vec::new();

                while remaining_boon > 0 && !table.items.is_empty() {
                    if let Some(item_spawn) = table.items.choose(&mut self.rng) {
                        if let Some(item_def) = super::item::get_item_def(&item_spawn.id) {
                            let item_value = item_def.value.min(remaining_boon);
                            remaining_boon = remaining_boon.saturating_sub(item_value);

                            if let Some((x, y)) = self.find_safe_spawn_position() {
                                let item_index = self.world.items.len();
                                self.world.items.push(Item::new(x, y, &item_spawn.id));
                                spawned_indices.push(item_index);
                            }
                        }
                    } else {
                        break;
                    }
                }

                // Update encounter state with spawned item indices
                if let Some(enc) = &mut self.world.encounter_state {
                    enc.spawned_items = spawned_indices;
                }
                self.rebuild_spatial_index();
            }
        }
    }

    /// Attempt to flee from current encounter
    pub fn attempt_flee_encounter(&mut self) -> Result<(), String> {
        let encounter = match &self.world.encounter_state {
            Some(e) => e.clone(),
            None => return Err("No active encounter.".to_string()),
        };

        // Check if flee is on cooldown
        let difficulty_mod = 1.0; // TODO: Calculate from tile danger
        if !encounter.can_flee(self.turn, difficulty_mod) {
            return Err("You cannot flee yet!".to_string());
        }

        // Attempt flee
        match super::encounter::attempt_flee(
            self.player.x,
            self.player.y,
            &self.world.enemies,
            &encounter.spawned_enemies,
            &mut self.rng,
            self.player.skills.get_skill_level("wayfaring"),
        ) {
            Ok(()) => {
                self.world.encounter_state = None;
                self.log_typed("You successfully flee the encounter!", MsgType::Status);
                Ok(())
            }
            Err(e) => {
                // Update last flee attempt
                if let Some(enc) = &mut self.world.encounter_state {
                    enc.last_flee_attempt = self.turn;
                }
                Err(e)
            }
        }
    }

    /// Enter subterranean layer (go down stairs)
    pub fn enter_subterranean(&mut self) -> bool {
        // Check if standing on stairs down
        if let Some(tile) = self.world.map.get(self.player.x, self.player.y) {
            if *tile != Tile::StairsDown {
                return false;
            }
        } else {
            return false;
        }

        self.world.layer -= 1;
        let seed = self
            .world
            .world_map
            .as_ref()
            .map(|wm| wm.tile_seed(self.world.world_x, self.world.world_y))
            .unwrap_or(42)
            .wrapping_add(self.world.layer.unsigned_abs() as u64 * 1000);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let (map, rooms) = Map::generate_subterranean(&mut rng, self.world.layer);
        let (px, py) = rooms[0];

        self.world.map = map;
        self.world.enemies = Vec::new();
        self.world.items = Vec::new();
        self.world.npcs = Vec::new();
        self.player.x = px;
        self.player.y = py;
        self.update_fov();
        self.rebuild_spatial_index();
        self.update_lighting();

        self.log(format!("You descend to level {}.", -self.world.layer));
        true
    }

    /// Exit subterranean layer (go up stairs)
    pub fn exit_subterranean(&mut self) -> bool {
        // Check if standing on stairs up
        if let Some(tile) = self.world.map.get(self.player.x, self.player.y) {
            if *tile != Tile::StairsUp {
                return false;
            }
        } else {
            return false;
        }

        if self.world.layer >= 0 {
            return false;
        } // Already on surface

        self.world.layer += 1;

        if self.world.layer == 0 {
            // Return to surface - regenerate surface tile
            self.travel_to_tile(self.world.world_x, self.world.world_y);
            self.log("You return to the surface.");
        } else {
            // Go up one underground level
            let seed = self
                .world
                .world_map
                .as_ref()
                .map(|wm| wm.tile_seed(self.world.world_x, self.world.world_y))
                .unwrap_or(42)
                .wrapping_add(self.world.layer.unsigned_abs() as u64 * 1000);
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            let (map, rooms) = Map::generate_subterranean(&mut rng, self.world.layer);
            let (px, py) = rooms.last().copied().unwrap_or((5, 5));

            self.world.map = map;
            self.world.enemies = Vec::new();
            self.world.items = Vec::new();
            self.world.npcs = Vec::new();
            self.player.x = px;
            self.player.y = py;
            self.update_fov();
            self.rebuild_spatial_index();
            self.update_lighting();

            self.log(format!("You ascend to level {}.", -self.world.layer));
        }
        true
    }

    /// Calculate world map path using A* with danger-based costs
    pub fn calculate_world_path(&mut self, target: (usize, usize)) -> bool {
        if self.world.world_map.is_none() {
            return false;
        }

        let start = (self.world.world_x, self.world.world_y);

        // Simple Manhattan distance pathfinding - just move towards target
        let mut path = Vec::new();
        let mut current = start;

        // Prevent infinite loops
        let max_steps = 500;
        let mut steps = 0;

        while current != target && steps < max_steps {
            let (cx, cy) = current;
            let (tx, ty) = target;

            // Move horizontally first, then vertically
            let next = if cx < tx {
                (cx + 1, cy)
            } else if cx > tx {
                (cx - 1, cy)
            } else if cy < ty {
                (cx, cy + 1)
            } else if cy > ty {
                (cx, cy - 1)
            } else {
                break;
            };

            path.push(next);
            current = next;
            steps += 1;
        }

        if !path.is_empty() {
            self.world.world_map_target = Some(target);
            self.world.world_map_path = path;
            true
        } else {
            false
        }
    }

    /// Move along the calculated world path
    pub fn move_along_path(&mut self) -> Result<bool, String> {
        if self.world.world_map_path.is_empty() {
            return Ok(false);
        }

        let next_pos = self.world.world_map_path.remove(0);

        // Use fast worldmap movement
        if let Some(_encounter_msg) = self.move_on_world_map(next_pos.0, next_pos.1) {
            // Encounter triggered - clear path
            self.world.world_map_path.clear();
            self.world.world_map_target = None;
            return Ok(true);
        }

        // Check if we reached the target
        if let Some(target) = self.world.world_map_target
            && (self.world.world_x, self.world.world_y) == target
        {
            self.world.world_map_target = None;
            self.world.world_map_path.clear();
        }

        Ok(true)
    }

    pub fn update_lighting(&mut self) {
        let mut sources = Vec::new();

        // Always add default player light
        sources.push(LightSource {
            x: self.player.x,
            y: self.player.y,
            radius: 5,
            intensity: 100,
        });

        // Equipped light sources (check all slots)
        for (_, slot_item) in self.player.equipment.iter() {
            if let Some(id) = slot_item
                && let Some(def) = get_item_def(id)
                && let Some(ref ls) = def.light_source
            {
                sources.push(LightSource {
                    x: self.player.x,
                    y: self.player.y,
                    radius: ls.radius,
                    intensity: ls.intensity,
                });
            }
        }
        // Map lights
        for ml in &self.world.map.lights {
            if let Some(def) = super::light_defs::get_light_def(&ml.id) {
                sources.push(LightSource {
                    x: ml.x,
                    y: ml.y,
                    radius: def.radius,
                    intensity: def.intensity,
                });
            }
        }
        // Items on ground with light_source property
        for item in &self.world.items {
            if let Some(def) = get_item_def(&item.id)
                && let Some(ref ls) = def.light_source
            {
                sources.push(LightSource {
                    x: item.x,
                    y: item.y,
                    radius: ls.radius,
                    intensity: ls.intensity,
                });
            }
        }
        self.light_map = compute_lighting(&sources, self.effective_ambient_light());
    }

    /// Update player field of view using shadow casting algorithm
    pub fn update_fov(&mut self) {
        self.visible = crate::game::map::compute_fov(&self.world.map, self.player.x, self.player.y);
        self.revealed.extend(&self.visible);
    }

    /// Generate procedural item lore using narrative templates
    pub fn generate_item_lore(&mut self, _item_category: &str) -> Option<String> {
        // Placeholder implementation - narrative generator not yet implemented
        None
    }

    /// Generate procedural location description
    pub fn generate_location_description(&mut self, _location_type: &str) -> Option<String> {
        // Placeholder implementation - narrative generator not yet implemented
        None
    }

    /// Generate contextual description based on current game state
    pub fn generate_contextual_description(&mut self) -> Option<String> {
        // Placeholder implementation - narrative generator not yet implemented
        None
    }

    /// Generate environmental storytelling text
    pub fn generate_environmental_text(&mut self, _environment_type: &str) -> Option<String> {
        // Placeholder implementation - narrative generator not yet implemented
        None
    }

    /// Generate NPC backstory using story model
    pub fn generate_npc_backstory(
        &mut self,
        npc_id: &str,
        _story_model: &StoryModel,
    ) -> Option<String> {
        // Placeholder implementation - narrative generator not yet implemented
        // Just return a simple backstory based on NPC definition
        super::npc::get_npc_def(npc_id)
            .map(|npc_def| format!("{} has a mysterious past.", npc_def.name))
    }

    /// Get faction lore from story model
    pub fn get_faction_lore(&self, _faction_name: &str) -> Option<String> {
        // Placeholder implementation - story model methods not yet implemented
        None
    }

    /// Generate crystal formations for appropriate biomes
    fn generate_crystal_formations(
        &mut self,
        biome: &super::world_map::Biome,
        rooms: &[(i32, i32)],
        rng: &mut ChaCha8Rng,
    ) {
        use super::crystal_resonance::CrystalFrequency;

        let formation_chance = match biome {
            super::world_map::Biome::Ruins => 0.6,
            super::world_map::Biome::Oasis => 0.4,
            super::world_map::Biome::Saltflat => 0.3,
            super::world_map::Biome::Scrubland => 0.2,
            super::world_map::Biome::Desert => 0.1,
        };

        if !rng.gen_bool(formation_chance) {
            return;
        }

        let formation_count = match biome {
            super::world_map::Biome::Ruins => rng.gen_range(2..=4),
            super::world_map::Biome::Oasis => rng.gen_range(1..=3),
            _ => rng.gen_range(1..=2),
        };

        let frequencies = CrystalFrequency::all();

        for _ in 0..formation_count {
            if let Some(&(rx, ry)) = rooms.get(rng.gen_range(0..rooms.len())) {
                let x = rx + rng.gen_range(-2..=2);
                let y = ry + rng.gen_range(-2..=2);

                // Don't place on player spawn or too close to enemies
                if (x - self.player.x).abs() < 5 && (y - self.player.y).abs() < 5 {
                    continue;
                }

                let frequency = frequencies[rng.gen_range(0..frequencies.len())];
                self.player.crystal_system.add_crystal(x, y, frequency);

                self.log_typed(
                    format!(
                        "A {} crystal formation glimmers nearby.",
                        frequency.name().to_lowercase()
                    ),
                    MsgType::Loot,
                );
            }
        }
    }

    /// Calculate effective ambient light based on time of day and weather
    pub fn effective_ambient_light(&self) -> u8 {
        // Underground has fixed low ambient
        if self.world.layer < 0 {
            return 30;
        }

        // Base ambient from time of day (0-23 hours)
        let time_ambient = match self.world.time_of_day {
            0..=4 => 30,   // Night
            5..=6 => 60,   // Dawn
            7..=17 => 120, // Day
            18..=19 => 80, // Dusk
            20..=23 => 40, // Night
            _ => 100,
        };

        // Apply weather modifier
        let weather_mod = self.world.weather.ambient_modifier();
        (time_ambient + weather_mod).clamp(10, 200) as u8
    }

    pub fn get_light_level(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 {
            return 0;
        }
        let idx = y as usize * self.world.map.width + x as usize;
        self.light_map.get(idx).copied().unwrap_or(0)
    }

    pub fn trigger_effect(&mut self, effect: &str, duration: u32) {
        self.triggered_effects.push(TriggeredEffect {
            effect: effect.to_string(),
            turns_remaining: duration,
        });
    }

    /// End turn: execute the turn phase sequence.
    pub fn end_turn(&mut self) {
        // Ensure spatial index is up to date before systems run
        self.ensure_spatial_index();
        for phase in super::effects::TurnPhase::sequence() {
            self.execute_phase(phase);
        }
    }

    fn execute_phase(&mut self, phase: &super::effects::TurnPhase) {
        use super::effects::{Effect, PlayerEffect, TurnPhase};
        use super::effects::trace::TraceSource;

        match phase {
            TurnPhase::ResetAp => {
                let effect = Effect::Player(PlayerEffect::ResetAp);
                self.apply_effect(&effect);
                self.trace.record(&effect, TraceSource::Rule { name: "end_turn" }, self.turn);
            }
            TurnPhase::TickStatusEffects => {
                let effect = Effect::Player(PlayerEffect::TickStatusEffects);
                self.apply_effect(&effect);
                self.trace.record(&effect, TraceSource::Rule { name: "end_turn" }, self.turn);
            }
            TurnPhase::TickSubsystems => {
                // Bridge effects — preserve RNG order: psychic, skills, light, void, crystal
                let effects = vec![
                    Effect::Player(PlayerEffect::TickPsychic),
                    Effect::Player(PlayerEffect::TickSkills),
                    Effect::Player(PlayerEffect::TickLightSystem),
                    Effect::Player(PlayerEffect::TickVoidSystem),
                    Effect::Player(PlayerEffect::TickCrystalSystem),
                ];
                for effect in &effects {
                    self.apply_effect(effect);
                    self.trace.record(effect, TraceSource::Rule { name: "end_turn" }, self.turn);
                }
            }
            TurnPhase::AdvanceTurn => {
                let effect = Effect::Player(PlayerEffect::AdvanceTurn);
                self.apply_effect(&effect);
                self.trace.record(&effect, TraceSource::Rule { name: "end_turn" }, self.turn);
                // Housekeeping (adaptation suppression, triggered effects, decoys, light effects)
                let hk = Effect::Player(PlayerEffect::TickHousekeeping);
                self.apply_effect(&hk);
                self.trace.record(&hk, TraceSource::Rule { name: "end_turn" }, self.turn);
                // Adaptation threshold check
                let ctx = super::effects::context::QueryContext::from_state(self);
                let output = super::rules::turn::rule_check_adaptation(&ctx);
                self.apply_and_trace(output, "rule_check_adaptation");
                // Quest turn progress (replaces TurnEnded event)
                let qt = Effect::Event(super::effects::EventEffect::QuestNotify {
                    kind: super::effects::QuestNotifyKind::Turn,
                });
                self.apply_effect(&qt);
                self.trace.record(&qt, TraceSource::Rule { name: "end_turn" }, self.turn);
            }
            TurnPhase::RunAI => {
                let effect = Effect::Player(PlayerEffect::RunAI);
                self.apply_effect(&effect);
                self.trace.record(&effect, TraceSource::Rule { name: "end_turn" }, self.turn);
            }
            TurnPhase::TickStorm => {
                let effect = Effect::Map(super::effects::MapEffect::TickStorm);
                self.apply_effect(&effect);
                self.trace.record(&effect, TraceSource::Rule { name: "end_turn" }, self.turn);
            }
            TurnPhase::AdvanceTime => {
                // Inline tick_time logic to avoid borrow conflict (rule needs &ctx + &mut rng)
                use super::effects::{MapEffect, RuleOutput};
                let mut effects = Vec::new();
                if self.turn.is_multiple_of(10) {
                    let new_time = (self.world.time_of_day as u32 + 1) % 24;
                    effects.push(Effect::Map(MapEffect::AdvanceTime { new_time }));
                    if new_time == 6 || new_time == 18 {
                        use rand::Rng;
                        let roll = self.rng.gen_range(0..10);
                        let weather = match roll {
                            0..=6 => "clear", 7..=8 => "dusty", 9 => "sandstorm", _ => "clear",
                        };
                        effects.push(Effect::Map(MapEffect::SetWeather { weather: weather.to_string() }));
                    }
                }
                self.apply_and_trace(RuleOutput { effects, presentation: Vec::new() }, "rule_tick_time");
            }
            TurnPhase::UpdateDerives => {
                self.ensure_spatial_index();
                self.update_lighting();
                self.update_fov();
            }
            TurnPhase::CheckEncounters => {
                let ctx = super::effects::context::QueryContext::from_state(self);
                let output = super::rules::turn::rule_check_encounters(&ctx);
                self.apply_and_trace(output, "rule_check_encounters");
            }
        }
    }

    /// Apply a status effect to the player
    pub fn apply_status(&mut self, effect: super::status::StatusEffect) {
        self.log_typed(
            format!("You are {}! ({} turns)", effect.name, effect.duration),
            MsgType::System,
        );
        self.player.status_effects.push(effect);
    }

    /// Auto-end turn if player has no AP left
    pub(crate) fn check_auto_end_turn(&mut self) {
        if self.player.ap <= 0 {
            self.end_turn();
        }
    }

    /// Housekeeping that runs after turn counter advances:
    /// adaptation suppression, triggered effects, decoys, light effects.
    pub(crate) fn tick_turn_housekeeping(&mut self) {
        if self.player.adaptations_hidden_turns > 0 {
            self.player.adaptations_hidden_turns -= 1;
            if self.player.adaptations_hidden_turns == 0 {
                self.log_typed(
                    "The tincture wears off. Your glow returns.",
                    MsgType::Status,
                );
            }
        }
        // Tick down triggered effects
        self.triggered_effects.retain_mut(|e| {
            e.turns_remaining = e.turns_remaining.saturating_sub(1);
            e.turns_remaining > 0
        });
        // Tick down decoys
        self.decoys.retain_mut(|d| {
            d.turns_remaining = d.turns_remaining.saturating_sub(1);
            d.turns_remaining > 0
        });

        // Apply light-based effects
        self.apply_light_effects();
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.log_typed(msg, MsgType::System);
    }

    pub fn log_typed(&mut self, msg: impl Into<String>, msg_type: MsgType) {
        self.messages
            .push(GameMessage::new(msg, msg_type, self.turn));
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }

    /// Log quest completion messages and unlock notifications
    pub fn log_quest_completions(&mut self, completed: &[String]) {
        for quest_id in completed {
            if let Some(def) = crate::game::quest::get_quest_def(quest_id) {
                self.log_typed(format!("Quest completed: {}", def.name), MsgType::System);
                for unlock_id in &def.reward.unlocks_quests {
                    if let Some(unlock_def) = crate::game::quest::get_quest_def(unlock_id) {
                        self.log_typed(
                            format!("New quest available: {}", unlock_def.name),
                            MsgType::System,
                        );
                    }
                }
            }
        }
    }

    /// Execute a debug command
    pub fn debug_command(&mut self, cmd: &str) {
        super::debug_commands::execute(self, cmd);
    }

    // === Visual Effects delegation ===

    pub fn trigger_hit_flash(&mut self, x: i32, y: i32) {
        self.world.visual_effects.trigger_hit_flash(x, y);
    }

    pub fn has_hit_flash(&self, x: i32, y: i32) -> bool {
        self.world.visual_effects.has_hit_flash(x, y)
    }

    pub fn spawn_damage_number(&mut self, x: i32, y: i32, value: i32, is_heal: bool) {
        self.world
            .visual_effects
            .spawn_damage_number(x, y, value, is_heal);
    }

    pub fn spawn_projectile(&mut self, from: (i32, i32), to: (i32, i32), ch: char) {
        self.world.visual_effects.spawn_projectile(from, to, ch);
    }

    pub fn get_projectile_at(&self, x: i32, y: i32) -> Option<char> {
        self.world.visual_effects.get_projectile_at(x, y)
    }

    pub fn spawn_beam(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        beam_type: super::visual_effects::BeamType,
        duration: u32,
    ) {
        self.world
            .visual_effects
            .spawn_beam(from, to, beam_type, duration);
    }

    pub fn get_beam_at(&self, x: i32, y: i32) -> Option<(char, super::visual_effects::BeamType)> {
        self.world.visual_effects.get_beam_at(x, y)
    }

    pub fn tick_hit_flash(&mut self) {
        self.world.visual_effects.tick_hit_flash();
    }

    pub fn tick_damage_numbers(&mut self) {
        self.world.visual_effects.tick_damage_numbers();
    }

    pub fn tick_projectile_trails(&mut self) {
        self.world.visual_effects.tick_projectile_trails();
    }

    pub fn tick_light_beams(&mut self) {
        self.world.visual_effects.tick_light_beams();
    }

    pub fn tick_animation(&mut self) {
        self.world.visual_effects.tick_animation();
    }

    pub fn check_adaptation_threshold(&mut self) {
        // Get all available adaptations sorted by threshold
        let mut available: Vec<(&str, u32)> = super::adaptation::all_adaptation_ids()
            .iter()
            .filter_map(|&id| {
                super::adaptation::get_adaptation_def(id).map(|def| (id, def.threshold))
            })
            .filter(|(id, _)| !self.player.adaptations.iter().any(|a| a.id() == *id))
            .collect();

        available.sort_by_key(|(_, threshold)| *threshold);

        // Find first unlockable adaptation
        if let Some(&(adaptation_id, _threshold)) =
            available.iter().find(|(_, t)| self.player.refraction >= *t)
            && let Some(adaptation) = super::adaptation::Adaptation::from_id(adaptation_id)
        {
            self.player.adaptations.push(adaptation);
            self.log(format!("🧬 You gain {}!", adaptation.name()));
        }
    }

    pub fn has_adaptation(&self, a: Adaptation) -> bool {
        self.player.adaptations.contains(&a)
    }

    pub fn enemy_at(&self, x: i32, y: i32) -> Option<usize> {
        self.spatial.enemy_positions.get(&(x, y)).copied()
    }

    pub fn npc_at(&self, x: i32, y: i32) -> Option<usize> {
        self.spatial.npc_positions.get(&(x, y)).copied()
    }

    /// Auto-explore: find nearest unexplored walkable tile and move toward it
    /// Enhanced with item pickup, danger avoidance, and enemy detection
    pub fn auto_explore(&mut self) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;

        let config = get_auto_explore_config();

        // Check for nearby enemies first
        if config.stop_on_enemies && self.has_nearby_enemies(config.enemy_detection_range) {
            self.messages.push(crate::game::GameMessage::new(
                "Auto-explore stopped: enemy detected nearby".to_string(),
                crate::game::MsgType::System,
                self.turn,
            ));
            return false;
        }

        // Pick up items at current position if configured
        if config.pickup_items {
            self.pickup_filtered_items();
        }

        let start = self.world.map.idx(self.player.x, self.player.y);

        // BFS to find nearest item or unexplored walkable tile
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, vec![start]));
        visited.insert(start);

        let target = loop {
            let (idx, path) = match queue.pop_front() {
                Some(p) => p,
                None => return false, // No items or unexplored tiles reachable
            };

            let mut found_target = false;

            // Check if this tile has items we want to pick up
            if config.pickup_items {
                let x = (idx % self.world.map.width) as i32;
                let y = (idx / self.world.map.width) as i32;

                for item in &self.world.items {
                    if item.x == x && item.y == y && config.should_pickup_item(&item.id) {
                        found_target = true;
                        break;
                    }
                }
            }

            // Check if this tile has an NPC we haven't talked to yet
            if !found_target {
                let x = (idx % self.world.map.width) as i32;
                let y = (idx / self.world.map.width) as i32;

                for npc in &self.world.npcs {
                    // Only target NPCs that haven't been talked to AND don't have ongoing quest interactions
                    if npc.x == x
                        && npc.y == y
                        && !npc.talked
                        && !self.has_interacted_with_npc(&npc.id)
                    {
                        found_target = true;
                        break;
                    }
                }
            }

            // Check if this tile is unexplored
            if !found_target && !self.revealed.contains(&idx) {
                found_target = true;
            }

            if found_target {
                // Return the first step toward this tile
                if path.len() > 1 {
                    break Some(path[1]);
                }
                return false;
            }

            // Add neighbors, but avoid dangerous tiles if configured
            for (next_idx, _) in self.world.map.get_available_exits(idx) {
                if !visited.contains(&next_idx) {
                    // Check if we should avoid this tile due to dangers
                    if config.avoid_dangers && self.is_dangerous_tile(next_idx) {
                        continue;
                    }

                    // Check if there's an NPC we've already talked to on this tile
                    if self.has_talked_npc_at_idx(next_idx)
                        || self.has_interacted_npc_at_idx(next_idx)
                    {
                        continue;
                    }

                    visited.insert(next_idx);
                    let mut new_path = path.clone();
                    new_path.push(next_idx);
                    queue.push_back((next_idx, new_path));
                }
            }
        };

        if let Some(next_idx) = target {
            let nx = (next_idx % self.world.map.width) as i32;
            let ny = (next_idx / self.world.map.width) as i32;
            let dx = nx - self.player.x;
            let dy = ny - self.player.y;

            // Final check: don't move to dangerous tile
            let target_x = self.player.x + dx;
            let target_y = self.player.y + dy;
            let target_idx = self.world.map.idx(target_x, target_y);

            if config.avoid_dangers && self.is_dangerous_tile(target_idx) {
                return false;
            }

            let old_pos = (self.player.x, self.player.y);
            self.dispatch(super::effects::Command::Move { dx, dy });
            self.player.x != old_pos.0 || self.player.y != old_pos.1
        } else {
            false
        }
    }

    /// Check if there are enemies nearby within the given range
    fn has_nearby_enemies(&self, range: i32) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();

        for enemy in &self.world.enemies {
            // Skip dead enemies
            if enemy.hp <= 0 {
                continue;
            }

            let distance = (enemy.x - self.player.x).abs() + (enemy.y - self.player.y).abs();
            if distance <= range {
                // If ignoring weak enemies, check enemy HP
                if config.ignore_weak_enemies && enemy.hp <= config.weak_enemy_threshold {
                    continue;
                }
                return true;
            }
        }
        false
    }

    /// Check if a tile is dangerous based on configuration
    fn is_dangerous_tile(&self, idx: usize) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();

        let tile = &self.world.map.tiles[idx];

        // Check for glass tiles
        if config.is_danger_type("glass") && matches!(tile, crate::game::map::Tile::Glass) {
            return true;
        }

        // Check for glare tiles
        if config.is_danger_type("glare") && matches!(tile, crate::game::map::Tile::Glare) {
            return true;
        }

        false
    }

    /// Check if there's an NPC we've already talked to at the given tile index
    fn has_talked_npc_at_idx(&self, idx: usize) -> bool {
        let x = (idx % self.world.map.width) as i32;
        let y = (idx / self.world.map.width) as i32;

        self.world
            .npcs
            .iter()
            .any(|npc| npc.x == x && npc.y == y && npc.talked)
    }

    /// Check if there's an NPC at this tile that we've interacted with via quest objectives
    fn has_interacted_npc_at_idx(&self, idx: usize) -> bool {
        let x = (idx % self.world.map.width) as i32;
        let y = (idx / self.world.map.width) as i32;

        self.world
            .npcs
            .iter()
            .any(|npc| npc.x == x && npc.y == y && self.has_interacted_with_npc(&npc.id))
    }

    /// Check if we've interacted with an NPC (either talked or has quest progress)
    fn has_interacted_with_npc(&self, npc_id: &str) -> bool {
        // Check if any TalkTo objective for this NPC has been completed in active quests
        for quest in &self.player.quest_log.active {
            if let Some(def) = quest.def() {
                for (i, obj) in def.objectives.iter().enumerate() {
                    if let crate::game::quest::ObjectiveType::TalkTo { npc_id: target } =
                        &obj.objective_type
                        && target == npc_id
                        && quest.objectives[i].completed
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Pick up items at current position, filtered by configuration
    fn pickup_filtered_items(&mut self) {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();

        let player_idx = self.world.map.idx(self.player.x, self.player.y);
        let mut items_to_remove = Vec::new();

        for (i, item) in self.world.items.iter().enumerate() {
            let item_idx = self.world.map.idx(item.x, item.y);
            if item_idx == player_idx && config.should_pickup_item(&item.id) {
                items_to_remove.push(i);
            }
        }

        // Remove items in reverse order to maintain indices
        for &i in items_to_remove.iter().rev() {
            let item = self.world.items.remove(i);
            self.player.inventory.push(item.id.clone());
            self.messages.push(crate::game::GameMessage::new(
                format!("Picked up {}", item.name()),
                crate::game::MsgType::System,
                self.turn,
            ));
        }
    }

    /// Pickup items at player position - delegates to MovementSystem
    pub fn pickup_items(&mut self) {
        MovementSystem::pickup_items(self)
    }

    pub fn can_open_chest(&self, chest_index: usize) -> bool {
        if chest_index >= self.world.chests.len() {
            return false;
        }

        let chest = &self.world.chests[chest_index];
        let player_pos = (self.player.x, self.player.y);
        let chest_pos = (chest.x, chest.y);

        // Check if player is adjacent to chest
        let dx = (player_pos.0 - chest_pos.0).abs();
        let dy = (player_pos.1 - chest_pos.1).abs();
        dx <= 1 && dy <= 1 && (dx + dy) > 0 // Adjacent but not same position
    }

    pub fn open_chest(&mut self, chest_index: usize) -> bool {
        if !self.can_open_chest(chest_index) {
            return false;
        }

        // Check if chest is locked and handle unlocking
        let chest_id = self.world.chests[chest_index].id.clone();
        let is_locked = self.world.chests[chest_index].is_locked();

        if is_locked
            && let Some(def) = super::chest::get_chest_def(&chest_id)
            && let Some(key_id) = &def.key_required
        {
            if self.player.inventory.contains(key_id) {
                self.world.chests[chest_index].unlock();
                self.log(format!("Unlocked {} with {}.", def.name, key_id));
            } else {
                self.log(format!("{} is locked. You need a {}.", def.name, key_id));
                return false;
            }
        }

        self.world.chests[chest_index].opened = true;
        let def = super::chest::get_chest_def(&chest_id);
        let name = def.map(|d| d.name.as_str()).unwrap_or("chest");
        self.log(format!("Opened {}.", name));
        true
    }

    pub fn transfer_to_chest(&mut self, chest_index: usize, inventory_index: usize) -> bool {
        if chest_index >= self.world.chests.len() || inventory_index >= self.player.inventory.len()
        {
            return false;
        }

        let chest = &mut self.world.chests[chest_index];
        if !chest.can_add_item() {
            self.log("Chest is full.");
            return false;
        }

        let item_id = self.player.inventory.remove(inventory_index);
        let item = Item::new(chest.x, chest.y, &item_id);
        chest.add_item(item);

        let item_def = super::item::get_item_def(&item_id);
        let name = item_def.map(|d| d.name.as_str()).unwrap_or(&item_id);
        self.log(format!("Stored {} in chest.", name));
        true
    }

    pub fn transfer_from_chest(&mut self, chest_index: usize, chest_item_index: usize) -> bool {
        if chest_index >= self.world.chests.len() {
            return false;
        }

        let chest = &mut self.world.chests[chest_index];
        if let Some(item) = chest.remove_item(chest_item_index) {
            self.player.inventory.push(item.id.clone());

            let item_def = super::item::get_item_def(&item.id);
            let name = item_def.map(|d| d.name.as_str()).unwrap_or(&item.id);
            self.log(format!("Took {} from chest.", name));
            true
        } else {
            false
        }
    }

    pub fn use_psychic_ability(&mut self, ability_id: &str) {
        match self.player.psychic.use_ability(ability_id) {
            Ok(effect_id) => {
                self.log_typed(format!("You use {}.", ability_id), MsgType::Combat);
                // Apply effect
                match effect_id.as_str() {
                    "stun_aoe" => {
                        // Stun nearby enemies
                        let mut stunned_count = 0;
                        for enemy in &mut self.world.enemies {
                            let dist = ((enemy.x - self.player.x).pow(2)
                                + (enemy.y - self.player.y).pow(2))
                                as f32;
                            if dist <= 25.0 {
                                // Radius 5
                                enemy.apply_status("stun", 2);
                                stunned_count += 1;
                            }
                        }
                        self.log_typed(
                            format!("Stunned {} enemies.", stunned_count),
                            MsgType::Combat,
                        );
                    }
                    "guaranteed_hit" => {
                        self.apply_status_effect("guaranteed_hit", 1);
                    }
                    "phasing" => {
                        self.apply_status_effect("phasing", 5);
                        self.debug.phase = true; // Or handle via status effect check in movement
                    }
                    _ => self.log("Effect not implemented."),
                }
            }
            Err(e) => self.log(e),
        }
    }

    /// Recalculate stats from equipment (called by ItemEffect::RecalcStats apply arm)
    pub(crate) fn recalc_equipment_stats(&mut self) {
        self.player.equipped_weapon = self.player.equipment.weapon.clone();
        self.player.armor = self
            .player
            .equipment
            .jacket
            .as_ref()
            .and_then(|id| get_item_def(id))
            .map(|def| def.armor_value)
            .unwrap_or(0);
    }

    /// Get next tutorial message if conditions are met — returns (id, text)
    pub fn get_next_tutorial_message(&self) -> Option<(String, String)> {
        self.narrative
            .tutorial_progress
            .get_next_message(self)
            .map(|msg| (msg.id.clone(), msg.text.clone()))
    }

    /// Mark a tutorial message as shown
    pub fn dismiss_tutorial_message(&mut self, message_id: &str) {
        self.narrative.tutorial_progress.mark_shown(message_id);
    }

    /// Modify faction reputation (clamped to -100 to +100)
    pub fn modify_reputation(&mut self, faction: &str, delta: i32) {
        let current = self
            .player
            .faction_reputation
            .get(faction)
            .copied()
            .unwrap_or(0);
        let new_rep = (current + delta).clamp(-100, 100);
        self.player
            .faction_reputation
            .insert(faction.to_string(), new_rep);

        if delta != 0 {
            let change_desc = if delta > 0 { "improved" } else { "worsened" };
            self.log_typed(
                format!("Your reputation with {} has {}.", faction, change_desc),
                MsgType::Social,
            );
        }
    }

    /// Get faction reputation (0 if not set)
    pub fn get_reputation(&self, faction: &str) -> i32 {
        self.player
            .faction_reputation
            .get(faction)
            .copied()
            .unwrap_or(0)
    }

    /// Apply status effect to player
    pub fn apply_status_effect(&mut self, effect_id: &str, duration: i32) {
        // Check if effect already exists
        if let Some(existing) = self
            .player
            .status_effects
            .iter_mut()
            .find(|e| e.id == effect_id)
        {
            existing.duration = existing.duration.max(duration); // Take longer duration
            existing.add_stack(5); // Max 5 stacks for most effects
        } else {
            self.player
                .status_effects
                .push(super::status::StatusEffect::new(effect_id, duration));
        }

        self.log_typed(
            format!("You are affected by {}.", effect_id),
            MsgType::Combat,
        );
    }

    /// Check if player has specific status effect
    pub fn has_status_effect(&self, effect_id: &str) -> bool {
        self.player.status_effects.iter().any(|e| e.id == effect_id)
    }

    /// Apply light-based effects (glare damage, visibility modifiers)
    pub fn apply_light_effects(&mut self) {
        if self.debug.disable_glare {
            return;
        }
        let light_level =
            super::lighting::get_light_level(&self.light_map, self.player.x, self.player.y);

        // Glare damage - disabled until lighting system is properly balanced
        // if super::lighting::has_glare(&self.light_map, self.player.x, self.player.y, 250) {
        //     if !self.has_status_effect("glare_protection") {
        //         self.player.hp -= 1;
        //         self.log_typed("The intense light burns your eyes!".to_string(), MsgType::Combat);
        //         self.apply_status_effect("blinded", 2);
        //     }
        // }

        // Light-based item effects
        for item_id in &self.player.inventory.clone() {
            if let Some(def) = super::item::get_item_def(item_id) {
                if def.reveals_storm_timing && light_level > 150 {
                    // Storm Chart works better in bright light
                    if self.rng.gen_range(0..100) < 10 {
                        self.log_typed(
                            "The Storm Chart glows, revealing storm patterns...".to_string(),
                            MsgType::System,
                        );
                    }
                }

                if def.grants_invisibility && light_level < 50 {
                    // Refraction Oil works better in darkness
                    if !self.has_status_effect("invisible") {
                        self.apply_status_effect("invisible", 3);
                        self.log_typed(
                            "You blend into the shadows...".to_string(),
                            MsgType::System,
                        );
                    }
                }
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
        state.update_lighting(); // Recalculate lighting after loading
        Ok(state)
    }
}

impl GameState {
    // Delegation methods for frequently accessed player fields
    pub fn player_x(&self) -> i32 {
        self.player.x
    }
    pub fn player_y(&self) -> i32 {
        self.player.y
    }
    pub fn player_hp(&self) -> i32 {
        self.player.hp
    }
    pub fn player_max_hp(&self) -> i32 {
        self.player.max_hp
    }
    pub fn player_ap(&self) -> i32 {
        self.player.ap
    }
    pub fn player_level(&self) -> u32 {
        self.player.level
    }

    // Delegation methods for frequently accessed world fields
    pub fn map(&self) -> &Map {
        &self.world.map
    }
    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.world.map
    }
    pub fn enemies(&self) -> &Vec<Enemy> {
        &self.world.enemies
    }
    pub fn enemies_mut(&mut self) -> &mut Vec<Enemy> {
        &mut self.world.enemies
    }
    pub fn npcs(&self) -> &Vec<Npc> {
        &self.world.npcs
    }
    pub fn npcs_mut(&mut self) -> &mut Vec<Npc> {
        &mut self.world.npcs
    }
    pub fn items(&self) -> &Vec<Item> {
        &self.world.items
    }
    pub fn items_mut(&mut self) -> &mut Vec<Item> {
        &mut self.world.items
    }
    pub fn chests(&self) -> &Vec<Chest> {
        &self.world.chests
    }
    pub fn chests_mut(&mut self) -> &mut Vec<Chest> {
        &mut self.world.chests
    }
    pub fn interactables(&self) -> &Vec<Interactable> {
        &self.world.interactables
    }
    pub fn interactables_mut(&mut self) -> &mut Vec<Interactable> {
        &mut self.world.interactables
    }
    pub fn storm(&self) -> &Storm {
        &self.world.storm
    }
    pub fn storm_mut(&mut self) -> &mut Storm {
        &mut self.world.storm
    }
    pub fn world_map(&self) -> &Option<WorldMap> {
        &self.world.world_map
    }
    pub fn world_x(&self) -> usize {
        self.world.world_x
    }
    pub fn world_y(&self) -> usize {
        self.world.world_y
    }
    pub fn layer(&self) -> i32 {
        self.world.layer
    }
    pub fn time_of_day(&self) -> u8 {
        self.world.time_of_day
    }
    pub fn weather(&self) -> Weather {
        self.world.weather
    }
    pub fn ambient_light(&self) -> u8 {
        self.world.ambient_light
    }
    pub fn refraction(&self) -> u32 {
        self.player.refraction
    }

    // Narrative delegation methods
    pub fn quest_log(&self) -> &crate::game::narrative_engine::QuestLog {
        &self.narrative.quest_log
    }
    pub fn quest_log_mut(&mut self) -> &mut crate::game::narrative_engine::QuestLog {
        &mut self.narrative.quest_log
    }
    pub fn story_model(&self) -> &crate::game::narrative_engine::StoryModel {
        &self.narrative.story_model
    }
    pub fn tutorial_progress(&self) -> &crate::game::tutorial::TutorialProgress {
        &self.narrative.tutorial_progress
    }
    pub fn world_history(&self) -> &crate::game::narrative_engine::WorldHistory {
        &self.narrative.world_history
    }

    pub fn load_test_tile(&mut self, params: crate::game::generation::tile_generator::TileParams) {
        use crate::game::generation::tile_generator::generate_tile;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let tile = generate_tile(&params);
        let biome = params.biome;
        let terrain = params.terrain;
        let poi = params.poi;
        let level = params.level;
        let walkable = tile.walkable_positions.clone();
        let mut rng = ChaCha8Rng::seed_from_u64(params.seed);

        self.world.map = tile.map;
        self.world.enemies = tile.enemies;
        self.world.items = tile.items;
        self.world.npcs = tile.npcs;
        self.world.chests = tile.chests;
        self.player.x = tile.spawn_pos.0;
        self.player.y = tile.spawn_pos.1;

        // same post-load hooks as travel_to_tile
        crate::game::generation::feature_materializer::materialize_features(
            self, biome, terrain, poi, level,
        );
        if poi == crate::game::world_map::POI::Town {
            self.spawn_crafting_stations(&walkable, &mut rng);
        }
        self.spawn_quest_required_npcs();
        self.update_fov();
        self.rebuild_spatial_index();
        self.update_lighting();
        self.generate_crystal_formations(&biome, &walkable, &mut rng);
        self.log(format!(
            "[TEST] Loaded tile: {:?} {:?} {:?}",
            biome, terrain, poi
        ));
    }
}
