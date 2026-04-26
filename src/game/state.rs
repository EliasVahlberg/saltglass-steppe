use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::{
    adaptation::Adaptation,
    item::get_item_def,
    lighting::{LightMap, LightSource, compute_lighting},
    map_features::MapFeatures,
};
use crate::game::narrative_engine::NarrativeEngine;
use crate::game::player_state::PlayerState;
use crate::game::world_state::WorldState;

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
    /// Adaptation IDs offered to the player at a refraction threshold.
    /// Set by check_adaptation_threshold; consumed by session.rs to open the choice UI.
    pub adaptation_choice: Option<Vec<String>>,
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
        if success {
            self.player.activity.items_crafted += 1;
        }
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



    pub fn has_adaptation(&self, a: Adaptation) -> bool {
        self.player.adaptations.contains(&a)
    }

    pub fn enemy_at(&self, x: i32, y: i32) -> Option<usize> {
        self.spatial.enemy_positions.get(&(x, y)).copied()
    }

    pub fn npc_at(&self, x: i32, y: i32) -> Option<usize> {
        self.spatial.npc_positions.get(&(x, y)).copied()
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


    /// Check if player has specific status effect
    pub fn has_status_effect(&self, effect_id: &str) -> bool {
        self.player.status_effects.iter().any(|e| e.id == effect_id)
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
                crate::game::systems::player::apply_status_effect(self, id, *duration);
            }
            Mutation::SetLastDamageDealt(v) => { self.player.last_damage_dealt = *v; }
            Mutation::IncrementActivity(field) => {
                use crate::game::player_state::ActivityField::*;
                match field {
                    StormsSurvived       => self.player.activity.storms_survived += 1,
                    GlassTilesWalked     => self.player.activity.glass_tiles_walked += 1,
                    EnemiesKilledMelee   => self.player.activity.enemies_killed_melee += 1,
                    EnemiesKilledRanged  => self.player.activity.enemies_killed_ranged += 1,
                    EliteEnemiesKilled   => self.player.activity.elite_enemies_killed += 1,
                    ItemsCrafted         => self.player.activity.items_crafted += 1,
                    ItemsUsed            => self.player.activity.items_used += 1,
                    PsychicUses          => self.player.activity.psychic_uses += 1,
                    TilesExplored        => self.player.activity.tiles_explored += 1,
                    NpcsTalked           => self.player.activity.npcs_talked += 1,
                    DamageTakenTotal(v)  => self.player.activity.damage_taken_total += v,
                }
            }
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
                crate::game::systems::player::recalc_equipment_stats(self);
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
                    crate::game::systems::player::recalc_equipment_stats(self);
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

            // --- Bridge subsystems ---
            Mutation::AddSaltScrip(amount) => {
                self.player.salt_scrip = self.player.salt_scrip.wrapping_add(*amount);
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

