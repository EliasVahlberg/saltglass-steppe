use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::{
    adaptation::Adaptation,
    chest::Chest,
    enemy::Enemy,
    interactable::Interactable,
    item::{Item, get_item_def},
    lighting::{LightMap, LightSource, compute_lighting},
    map::Map,
    map_features::MapFeatures,
    npc::Npc,
    storm::Storm,
    systems::movement::MovementSystem,
    world_map::WorldMap,
};
use crate::game::narrative_engine::NarrativeEngine;
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
    /// Mutation trace — debug strings of Mutation variants, for DES verification
    #[serde(skip)]
    pub mutation_log: Vec<String>,
}

pub(crate) fn msg_type_from_str(s: &str) -> MsgType {
    match s {
        "combat" => MsgType::Combat,
        "loot" => MsgType::Loot,
        "warning" => MsgType::Warning,
        "status" => MsgType::Status,
        "dialogue" => MsgType::Dialogue,
        "social" => MsgType::Social,
        _ => MsgType::System,
    }
}

impl GameState {
    pub fn dispatch(&mut self, command: super::effects::Command) {
        self.ensure_spatial_index();
        if let Some(mutations) = super::dispatch::route_command(&command, self) {
            super::dispatch::apply_with_cascade(self, mutations);
            self.check_auto_end_turn();
        }
    }

    pub fn dispatch_craft(&mut self, recipe_id: &str) -> bool {
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
        let mutations = super::systems::rule_output_to_mutations(output, msg_type_from_str);
        self.apply_mutations(mutations);
        success
    }

    pub fn dispatch_buy_item(&mut self, item_id: &str, npc_id: &str) -> Result<(), String> {
        let ctx = super::effects::context::QueryContext::from_state(self);
        let output = super::rules::economy::rule_buy_item(item_id, npc_id, &ctx);
        if output.effects.is_empty() {
            let msg = output.presentation.first()
                .map(|p| match p { super::effects::Presentation::LogMessage { text, .. } => text.clone() })
                .unwrap_or_else(|| "Cannot buy item.".into());
            return Err(msg);
        }
        let mutations = super::systems::rule_output_to_mutations(output, msg_type_from_str);
        self.apply_mutations(mutations);
        Ok(())
    }

    pub fn dispatch_sell_item(&mut self, item_id: &str) -> Result<(), String> {
        let inv_idx = self.player.inventory.iter().position(|id| id == item_id)
            .ok_or_else(|| "You don't have that item.".to_string())?;
        let ctx = super::effects::context::QueryContext::from_state(self);
        let output = super::rules::economy::rule_sell_item(inv_idx, &ctx);
        if output.effects.is_empty() {
            let msg = output.presentation.first()
                .map(|p| match p { super::effects::Presentation::LogMessage { text, .. } => text.clone() })
                .unwrap_or_else(|| "Cannot sell item.".into());
            return Err(msg);
        }
        let mutations = super::systems::rule_output_to_mutations(output, msg_type_from_str);
        self.apply_mutations(mutations);
        Ok(())
    }

    pub(crate) fn ensure_spatial_index(&mut self) {
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

    /// Generate crystal formations for appropriate biomes
    pub(crate) fn generate_crystal_formations(
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
    pub fn apply_presentation(&mut self, p: &super::effects::Presentation) {
        let super::effects::Presentation::LogMessage { text, msg_type } = p;
        self.log_typed(text.clone(), msg_type_from_str(msg_type));
    }

    pub fn log(&mut self, msg: impl Into<String>) {        self.log_typed(msg, MsgType::System);
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

    /// Get faction reputation (0 if not set)
    pub fn get_reputation(&self, faction: &str) -> i32 {
        self.player
            .faction_reputation
            .get(faction)
            .copied()
            .unwrap_or(0)
    }

    /// Apply status effect to player (merges stacks/duration if already present)
    pub(crate) fn apply_status_effect(&mut self, effect_id: &str, duration: i32) {
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

    // -----------------------------------------------------------------------
    // Mutation interface (Stage 1 — verified state store)
    // -----------------------------------------------------------------------

    /// Apply a single mutation, enforce invariants, return a transition if one occurred.
    pub fn apply_one(&mut self, mutation: &super::mutations::Mutation) -> Option<super::mutations::StateTransition> {
        use super::mutations::{Mutation, StateTransition, SubsystemId};
        use super::progression::max_level;

        match mutation {
            // --- Player vitals ---
            Mutation::SetPlayerHp(v) => {
                let old = self.player.hp;
                self.player.hp = (*v).clamp(0, self.player.max_hp);
                if self.player.hp <= 0 && old > 0 {
                    return Some(StateTransition::PlayerDied);
                }
            }
            Mutation::SetPlayerMaxHp(v) => {
                self.player.max_hp = (*v).max(1);
            }
            Mutation::SetPlayerAp(v) => {
                let old = self.player.ap;
                self.player.ap = (*v).clamp(0, self.player.max_ap);
                if self.player.ap == 0 && old > 0 {
                    return Some(StateTransition::PlayerApReachedZero);
                }
            }
            Mutation::SetPlayerMaxAp(v) => {
                self.player.max_ap = (*v).max(1);
            }
            Mutation::SetPlayerPosition { x, y } => {
                let (old_x, old_y) = (self.player.x, self.player.y);
                self.player.x = *x;
                self.player.y = *y;
                if old_x != *x || old_y != *y {
                    return Some(StateTransition::PlayerPositionChanged {
                        old_x, old_y, new_x: *x, new_y: *y,
                    });
                }
            }
            Mutation::SetPlayerReflex(v) => { self.player.reflex = *v; }
            Mutation::SetPlayerArmor(v) => { self.player.armor = *v; }

            // --- Player progression ---
            Mutation::SetPlayerXp(v) => {
                self.player.xp = self.player.xp.max(*v);
            }
            Mutation::SetPlayerLevel(v) => {
                self.player.level = (*v).clamp(1, max_level());
            }
            Mutation::SetPlayerStatPoints(v) => { self.player.pending_stat_points = *v; }
            Mutation::SetPlayerSkillPoints(v) => { self.player.skills.skill_points = *v; }
            Mutation::SetPlayerSaltScrip(v) => { self.player.salt_scrip = *v; }

            // --- Player state ---
            Mutation::SetPlayerRefraction(v) => { self.player.refraction = *v; }
            Mutation::SetWaitCounter(v) => { self.wait_counter = *v; }
            Mutation::SetAdaptationsHidden(v) => { self.player.adaptations_hidden_turns = *v; }
            Mutation::AddAdaptation(id) => {
                if let Some(a) = super::adaptation::Adaptation::from_id(id) {
                    self.player.adaptations.push(a);
                }
            }
            Mutation::AddStatusEffect { id, duration } => {
                self.apply_status_effect(id, *duration);
            }
            Mutation::SetLastDamageDealt(v) => { self.player.last_damage_dealt = *v; }
            Mutation::AllocateStat(stat) => {
                if self.player.pending_stat_points > 0 {
                    match stat.as_str() {
                        "max_hp" => { self.player.max_hp += 5; self.player.hp += 5; }
                        "max_ap" => { self.player.max_ap += 1; self.player.ap += 1; }
                        "reflex" => { self.player.reflex += 1; }
                        _ => {}
                    }
                    self.player.pending_stat_points -= 1;
                }
            }
            Mutation::SuppressAdaptations { turns } => {
                self.player.adaptations_hidden_turns = *turns;
            }
            Mutation::SetPhaseMode(enabled) => {
                self.debug.phase = *enabled;
            }
            Mutation::Equip { slot, item_id } => {
                self.apply_one(&Mutation::SetEquipment { slot: slot.clone(), item_id: Some(item_id.clone()) });
            }
            Mutation::Unequip(slot) => {
                self.apply_one(&Mutation::SetEquipment { slot: slot.clone(), item_id: None });
            }
            Mutation::RecalcStats => {
                self.recalc_equipment_stats();
            }
            Mutation::StunEnemy { idx, duration } => {
                self.apply_one(&Mutation::AddEnemyStatus { idx: *idx, id: "stun".into(), duration: *duration });
            }
            Mutation::DamageWall { x, y, damage } => {
                let tile_idx = (*y * self.world.map.width as i32 + *x) as usize;
                if tile_idx < self.world.map.tiles.len() {
                    let mut broken = false;
                    if let super::map::Tile::Wall { hp, .. } = &mut self.world.map.tiles[tile_idx] {
                        *hp -= damage;
                        if *hp <= 0 { broken = true; }
                    }
                    if broken {
                        self.apply_one(&Mutation::SetTile {
                            idx: tile_idx,
                            tile: super::map::Tile::floor("stone"),
                        });
                    }
                }
            }
            // --- Inventory & equipment ---
            Mutation::AddToInventory(item_id) => {
                self.player.inventory.push(item_id.clone());
                return Some(StateTransition::ItemAddedToInventory { item_id: item_id.clone() });
            }
            Mutation::RemoveFromInventory(idx) => {
                if *idx < self.player.inventory.len() {
                    self.player.inventory.remove(*idx);
                }
            }
            Mutation::SetEquipment { slot, item_id } => {
                if let Ok(equip_slot) = slot.parse::<super::equipment::EquipSlot>() {
                    let old = self.player.equipment.set(equip_slot, item_id.clone());
                    if let Some(old_item) = old {
                        self.player.inventory.push(old_item);
                    }
                    self.recalc_equipment_stats();
                }
            }
            Mutation::SpawnItemOnMap { item_id, x, y } => {
                self.world.items.push(super::item::Item::new(*x, *y, item_id));
                self.rebuild_spatial_index();
            }

            // --- Enemies ---
            Mutation::SetEnemyHp { idx, hp } => {
                if let Some(enemy) = self.world.enemies.get_mut(*idx) {
                    let old_hp = enemy.hp;
                    enemy.hp = *hp;
                    if old_hp != *hp {
                        let enemy_id = enemy.id.clone();
                        let (ex, ey) = (enemy.x, enemy.y);
                        if *hp <= 0 && old_hp > 0 {
                            return Some(StateTransition::EnemyHpReachedZero {
                                idx: *idx, enemy_id, x: ex, y: ey,
                            });
                        }
                        return Some(StateTransition::EnemyHpChanged {
                            idx: *idx, old_hp, new_hp: *hp,
                        });
                    }
                }
            }
            Mutation::SetEnemyProvoked { idx, provoked } => {
                if let Some(enemy) = self.world.enemies.get_mut(*idx) {
                    enemy.provoked = *provoked;
                }
            }
            Mutation::AddEnemyStatus { idx, id, duration } => {
                if let Some(enemy) = self.world.enemies.get_mut(*idx) {
                    enemy.apply_status(id, *duration);
                }
            }
            Mutation::RemoveEnemy { idx, x, y } => {
                self.spatial.enemy_positions.remove(&(*x, *y));
                if *idx < self.world.enemies.len() {
                    let enemy_id = self.world.enemies[*idx].id.clone();
                    self.meta.discover_enemy(&enemy_id);
                }
            }
            Mutation::SpawnEnemy { id, x, y } => {
                self.world.enemies.push(super::enemy::Enemy::new(*x, *y, id));
                self.rebuild_spatial_index();
            }

            // --- World state ---
            Mutation::SetWorldPosition { wx, wy } => {
                self.world.world_x = *wx;
                self.world.world_y = *wy;
                return Some(StateTransition::PlayerEnteredWorldTile { wx: *wx, wy: *wy });
            }
            Mutation::SetLayer(v) => { self.world.layer = *v; }
            Mutation::SetTimeOfDay(v) => { self.world.time_of_day = *v % 24; }
            Mutation::SetWeather(w) => { self.world.weather = *w; }
            Mutation::IncrementTilesTraveled => { self.world.total_tiles_traveled += 1; }
            Mutation::AdvanceTurn => {
                let old = self.turn;
                self.turn += 1;
                return Some(StateTransition::TurnAdvanced { old_turn: old, new_turn: self.turn });
            }

            // --- Map ---
            Mutation::SetTile { idx, tile } => {
                if *idx < self.world.map.tiles.len() {
                    self.world.map.tiles[*idx] = tile.clone();
                }
            }
            Mutation::RevealTile(idx) => { self.revealed.insert(*idx); }
            Mutation::RevealAll => {
                for i in 0..self.world.map.tiles.len() { self.revealed.insert(i); }
            }
            Mutation::ClearStormHighlight(idx) => {
                self.world.visual_effects.storm_changed_tiles.remove(idx);
            }
            Mutation::SetWorldPath { path, target } => {
                self.world.world_map_path = path.clone();
                self.world.world_map_target = *target;
            }
            Mutation::ClearWorldPath => {
                self.world.world_map_path.clear();
                self.world.world_map_target = None;
            }

            // --- Encounter ---
            Mutation::SetEncounterState(s) => {
                self.world.encounter_state = s.as_ref().map(|b| *b.clone());
            }
            Mutation::IncrementEncounterTimer => {
                if let Some(enc) = &mut self.world.encounter_state {
                    enc.turns_in_encounter += 1;
                }
            }
            Mutation::SetLastFleeAttempt(turn) => {
                if let Some(enc) = &mut self.world.encounter_state {
                    enc.last_flee_attempt = *turn;
                }
            }

            // --- Faction & quest ---
            Mutation::SetReputation { faction, value } => {
                let clamped = (*value).clamp(-100, 100);
                self.player.faction_reputation.insert(faction.clone(), clamped);
            }
            Mutation::AcceptQuest(quest_id) => {
                if let Some(quest) = super::quest::ActiveQuest::new(quest_id) {
                    self.player.quest_log.active.push(quest);
                }
            }
            Mutation::CompleteQuest(quest_id) => {
                let _ = self.player.quest_log.complete(quest_id);
            }
            Mutation::SetFactionAlignment(faction) => {
                self.player.quest_log.set_faction_alignment(faction);
            }
            Mutation::QuestNotify(kind) => {
                use super::effects::QuestNotifyKind;
                let completed = match kind {
                    QuestNotifyKind::Kill { enemy_id } => {
                        self.player.quest_log.on_enemy_killed(enemy_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Collect { item_id } => {
                        self.player.quest_log.on_item_collected(item_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Move { x, y } => {
                        self.player.quest_log.on_position_changed(*x, *y);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::NpcTalk { npc_id } => self.player.quest_log.on_npc_talked(npc_id),
                    QuestNotifyKind::Interact { target_id } => {
                        self.player.quest_log.on_interact(target_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Examine { target_id } => {
                        self.player.quest_log.on_examine(target_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::AriaInterface { item_id } => {
                        self.player.quest_log.on_aria_interfaced(item_id);
                        self.player.quest_log.check_auto_complete()
                    }
                    QuestNotifyKind::Turn => {
                        self.player.quest_log.on_turn_passed();
                        self.player.quest_log.check_auto_complete()
                    }
                };
                self.log_quest_completions(&completed);
            }

            // --- Resources ---
            Mutation::SetLightEnergy(v) => { self.player.light_system.light_energy = *v; }
            Mutation::AddVoidEnergy(v) => { self.player.void_system.gain_energy(*v); }
            Mutation::AddVoidExposure(v) => { self.player.void_system.add_exposure(*v); }
            Mutation::SetResonanceEnergy(v) => {
                self.player.crystal_system.resonance_energy =
                    (*v).min(self.player.crystal_system.max_resonance_energy);
            }
            Mutation::PlaceCrystal { x, y, frequency } => {
                let freq = match frequency.as_str() {
                    "alpha" => super::crystal_resonance::CrystalFrequency::Alpha,
                    "beta"  => super::crystal_resonance::CrystalFrequency::Beta,
                    "gamma" => super::crystal_resonance::CrystalFrequency::Gamma,
                    "delta" => super::crystal_resonance::CrystalFrequency::Delta,
                    "epsilon" => super::crystal_resonance::CrystalFrequency::Epsilon,
                    _ => super::crystal_resonance::CrystalFrequency::Alpha,
                };
                self.player.crystal_system.add_crystal(*x, *y, freq);
            }

            // --- Presentation (no verification, no transitions) ---
            Mutation::LogMessage { text, msg_type } => {
                self.log_typed(text.clone(), *msg_type);
            }
            Mutation::OpenBook(id) => { self.pending_ui.book_open = Some(id.clone()); }
            Mutation::PlaceDecoy { x, y } => {
                self.decoys.push(Decoy { x: *x, y: *y, turns_remaining: 3 });
            }
            Mutation::HitFlash { x, y } => { self.world.visual_effects.trigger_hit_flash(*x, *y); }
            Mutation::DamageNumber { x, y, value, is_heal } => {
                self.world.visual_effects.spawn_damage_number(*x, *y, *value, *is_heal);
            }
            Mutation::SpawnProjectile { from, to, ch } => {
                self.world.visual_effects.spawn_projectile(*from, *to, *ch);
            }
            Mutation::TriggerEffect { effect, duration } => {
                self.trigger_effect(effect, *duration);
            }
            Mutation::UsePsychicAbility { ability_id } => {
                match self.player.psychic.use_ability(ability_id) {
                    Ok(effect_id) => {
                        self.log_typed(format!("You use {}.", ability_id), MsgType::Combat);
                        let output = {
                            let ctx = super::effects::context::QueryContext::from_state(self);
                            super::rules::rule_use_psychic(&effect_id, &ctx)
                        };
                        let mutations = super::systems::rule_output_to_mutations(output, msg_type_from_str);
                        self.apply_mutations(mutations);
                    }
                    Err(e) => self.log(e),
                }
            }
            Mutation::AttemptFlee { turn } => {
                let encounter = self.world.encounter_state.clone()?;
                match super::encounter::attempt_flee(
                    self.player.x, self.player.y,
                    &self.world.enemies, &encounter.spawned_enemies,
                    &mut self.rng,
                    self.player.skills.get_skill_level("wayfaring"),
                ) {
                    Ok(()) => {
                        self.world.encounter_state = None;
                        self.log_typed("You successfully flee the encounter!".to_string(), MsgType::Status);
                    }
                    Err(e) => {
                        if let Some(enc) = &mut self.world.encounter_state {
                            enc.last_flee_attempt = *turn;
                        }
                        self.log_typed(e, MsgType::Warning);
                    }
                }
            }

            // --- Bridge subsystems ---
            Mutation::AddSaltScrip(amount) => {
                self.player.salt_scrip += amount;
            }
            Mutation::SpendAp(amount) => {
                self.player.ap = (self.player.ap - amount).clamp(0, self.player.max_ap);
            }
            Mutation::AddHp(amount) => {
                self.player.hp = (self.player.hp + amount).clamp(0, self.player.max_hp);
            }
            Mutation::AddRefraction(delta) => {
                self.player.refraction = (self.player.refraction as i32 + delta).max(0) as u32;
            }
            Mutation::IncrementWaitCounter => {
                self.wait_counter += 1;
            }
            Mutation::WorldMove { wx, wy } => { super::systems::world::dispatch_world_move(self, *wx, *wy); }
            Mutation::WorldMoveSafe { wx, wy } => { super::systems::world::dispatch_world_move_safe(self, *wx, *wy); }
            Mutation::FollowWorldPath => { super::systems::world::dispatch_follow_world_path(self); }
            Mutation::CalculateWorldPath { target } => { super::systems::world::dispatch_calculate_world_path(self, *target); }
            Mutation::EnterSubterranean => { super::systems::world::enter_subterranean(self); }
            Mutation::ExitSubterranean => { super::systems::world::exit_subterranean(self); }
            Mutation::MovePlayer { dx, dy } => { super::systems::movement::dispatch_move(self, *dx, *dy); }
            Mutation::EndTurn => { self.end_turn(); }
            Mutation::RestTick => {
                for _ in 0..10 { self.tick_turn_housekeeping(); }
                self.update_enemies();
            }
            Mutation::TickSubsystem(id) => match id {
                SubsystemId::Psychic     => { self.player.psychic.tick(); }
                SubsystemId::Skills      => { self.player.skills.tick(); }
                SubsystemId::Light       => { self.player.light_system.update(&mut self.rng); }
                SubsystemId::Void        => { self.player.void_system.update(&mut self.rng); }
                SubsystemId::Crystal     => { self.player.crystal_system.update(&mut self.rng); }
                SubsystemId::Status      => {
                    super::systems::StatusEffectSystem::tick_player_effects(self);
                    super::systems::StatusEffectSystem::tick_enemy_effects(self);
                }
                SubsystemId::AI          => { self.update_enemies(); }
                SubsystemId::Storm       => {
                    if self.world.storm.tick() {
                        super::systems::StormSystem::apply_storm(self);
                    }
                }
                SubsystemId::Housekeeping => { self.tick_turn_housekeeping(); }
            },
            Mutation::ResetAp => {
                self.player.ap = self.player.max_ap;
            }
            Mutation::TickStatusEffects => {
                self.apply_one(&Mutation::TickSubsystem(SubsystemId::Status));
            }
            Mutation::TickHousekeeping => {
                self.tick_turn_housekeeping();
            }
            Mutation::RunAI => {
                self.apply_one(&Mutation::TickSubsystem(SubsystemId::AI));
            }
            Mutation::TickStorm => {
                self.apply_one(&Mutation::TickSubsystem(SubsystemId::Storm));
            }
            Mutation::AdvanceTime { new_time } => {
                self.world.time_of_day = *new_time as u8;
            }
        }
        None
    }

    /// Apply a batch of mutations, collect all transitions.
    pub fn apply_mutations(&mut self, mutations: Vec<super::mutations::Mutation>) -> Vec<super::mutations::StateTransition> {
        let mut transitions = Vec::new();
        for m in &mutations {
            if self.trace.enabled {
                self.mutation_log.push(format!("{:?}", m));
            }
            if let Some(t) = self.apply_one(m) {
                transitions.push(t);
            }
        }
        transitions
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
            super::systems::world::spawn_crafting_stations(self, &walkable, &mut rng);
        }
        super::systems::world::spawn_quest_required_npcs(self);
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
