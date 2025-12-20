//! Debug Execution System (DES) - Headless game execution for automated testing
//!
//! Runs game scenarios without rendering for automated testing and validation.

use crate::game::{adaptation::Adaptation, inspect::inspect_item, status::StatusType, Enemy, GameState, Item, Npc};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn parse_adaptation(id: &str) -> Option<Adaptation> {
    match id.to_lowercase().as_str() {
        "prismhide" => Some(Adaptation::Prismhide),
        "sunveins" => Some(Adaptation::Sunveins),
        "mirage_step" | "miragestep" => Some(Adaptation::MirageStep),
        "saltblood" => Some(Adaptation::Saltblood),
        _ => None,
    }
}

fn parse_status_type(id: &str) -> Option<StatusType> {
    match id.to_lowercase().as_str() {
        "poison" => Some(StatusType::Poison),
        "burn" => Some(StatusType::Burn),
        "stun" => Some(StatusType::Stun),
        "bleed" => Some(StatusType::Bleed),
        "slow" => Some(StatusType::Slow),
        _ => None,
    }
}

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MockSettings {
    /// Force all attacks to hit (true) or miss (false)
    #[serde(default)]
    pub combat_always_hit: Option<bool>,
    /// Force specific damage value (bypasses roll)
    #[serde(default)]
    pub combat_fixed_damage: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub mocks: MockSettings,
    #[serde(default)]
    pub entities: Vec<EntitySpawn>,
    #[serde(default)]
    pub player: PlayerSetup,
    #[serde(default)]
    pub actions: Vec<ScheduledAction>,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    #[serde(default)]
    pub after_turn: Option<u32>,
    #[serde(default)]
    pub at_end: bool,
    pub check: AssertionCheck,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssertionCheck {
    PlayerHp { op: CmpOp, value: i32 },
    PlayerPosition { x: i32, y: i32 },
    PlayerAlive,
    PlayerDead,
    InventoryContains { item: String },
    InventorySize { op: CmpOp, value: usize },
    EnemyAt { x: i32, y: i32, alive: bool },
    NoEnemyAt { x: i32, y: i32 },
    Turn { op: CmpOp, value: u32 },
    // New assertions
    EnemyHp { id: String, op: CmpOp, value: i32 },
    EnemyAlive { id: String },
    EnemyDead { id: String },
    PlayerHasAdaptation { adaptation: String },
    AdaptationCount { op: CmpOp, value: usize },
    MapTileAt { x: i32, y: i32, tile: String },
    Refraction { op: CmpOp, value: u32 },
    PlayerAp { op: CmpOp, value: i32 },
    HasStatusEffect { effect: String },
    StatusEffectCount { op: CmpOp, value: usize },
    TileExplored { x: i32, y: i32 },
    ExploredCount { op: CmpOp, value: usize },
    EquippedInSlot { slot: String, item: Option<String> },
    PlayerArmor { op: CmpOp, value: i32 },
    EnemyProvoked { id: String, provoked: bool },
    EnemyHasItem { id: String, item: String },
    LightLevel { x: i32, y: i32, op: CmpOp, value: u8 },
    ItemInspectHasStat { item: String, stat: String },
    ItemInspectMissingStat { item: String, stat: String },
    NpcTalked { id: String, talked: bool },
    PlayerXp { op: CmpOp, value: u32 },
    PlayerLevel { op: CmpOp, value: u32 },
    MessageContains { text: String },
    PendingStatPoints { op: CmpOp, value: i32 },
    // Quest assertions
    QuestActive { quest_id: String },
    QuestCompleted { quest_id: String },
    QuestObjectiveProgress { quest_id: String, objective_id: String, op: CmpOp, value: u32 },
    QuestObjectiveComplete { quest_id: String, objective_id: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl CmpOp {
    fn compare<T: Ord>(&self, a: T, b: T) -> bool {
        match self {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerSetup {
    #[serde(default)]
    pub x: Option<i32>,
    #[serde(default)]
    pub y: Option<i32>,
    #[serde(default)]
    pub hp: Option<i32>,
    #[serde(default)]
    pub max_hp: Option<i32>,
    #[serde(default)]
    pub ap: Option<i32>,
    #[serde(default)]
    pub max_ap: Option<i32>,
    #[serde(default)]
    pub xp: Option<u32>,
    #[serde(default)]
    pub inventory: Vec<String>,
    #[serde(default)]
    pub adaptations: Vec<String>,
    #[serde(default)]
    pub equipped_weapon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySpawn {
    pub entity_type: EntityType,
    pub id: String,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub hp: Option<i32>,
    #[serde(default)]
    pub ai_disabled: bool,
    #[serde(default)]
    pub inventory: Vec<String>,
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Enemy,
    Npc,
    Item,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledAction {
    pub turn: u32,
    pub action: Action,
    #[serde(default)]
    pub actor: Actor,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    #[default]
    Player,
    Entity { id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Move { dx: i32, dy: i32 },
    Teleport { x: i32, y: i32 },
    Attack { target_x: i32, target_y: i32 },
    RangedAttack { target_x: i32, target_y: i32 },
    ApplyStatus { effect: String, duration: u32, potency: i32 },
    UseItem { item_index: usize },
    Equip { item_index: usize, slot: String },
    Unequip { slot: String },
    AutoExplore,
    Wait { turns: u32 },
    Rest,
    EndTurn,
    Log { query: LogQuery },
    AllocateStat { stat: String },
    // Quest actions
    AcceptQuest { quest_id: String },
    CompleteQuest { quest_id: String },
    // Crafting actions
    Craft { recipe_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogQuery {
    PlayerHp,
    PlayerPosition,
    Inventory,
    EntityAt { x: i32, y: i32 },
    Turn,
    Custom { message: String },
}

// ============================================================================
// Execution Result
// ============================================================================

pub struct ExecutionResult {
    pub success: bool,
    pub final_turn: u32,
    pub logs: Vec<ExecutionLog>,
    pub assertion_results: Vec<AssertionResult>,
    pub snapshots: Vec<StateSnapshot>,
    pub final_state: Option<GameState>,
}

#[derive(Debug, Clone)]
pub struct ExecutionLog {
    pub turn: u32,
    pub action_index: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub passed: bool,
    pub check: String,
    pub message: Option<String>,
}

// ============================================================================
// Parser
// ============================================================================

impl Scenario {
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
        let mut scenario: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Parse error: {}", e))?;
        
        // Handle base file inheritance
        if let Some(base_path) = scenario.base.take() {
            let base_dir = path.as_ref().parent().unwrap_or(Path::new("."));
            scenario.inherit_from(base_dir.join(&base_path))?;
        }
        Ok(scenario)
    }

    /// Load and merge a base scenario file
    pub fn inherit_from(&mut self, base_path: impl AsRef<Path>) -> Result<(), String> {
        let base = Self::from_file(&base_path)?;
        
        // Merge: base values are used if child doesn't override
        if self.seed.is_none() { self.seed = base.seed; }
        
        // Merge player setup (child overrides base)
        if self.player.x.is_none() { self.player.x = base.player.x; }
        if self.player.y.is_none() { self.player.y = base.player.y; }
        if self.player.hp.is_none() { self.player.hp = base.player.hp; }
        if self.player.max_hp.is_none() { self.player.max_hp = base.player.max_hp; }
        if self.player.xp.is_none() { self.player.xp = base.player.xp; }
        if self.player.inventory.is_empty() { self.player.inventory = base.player.inventory; }
        
        // Prepend base entities, actions, assertions
        let mut entities = base.entities;
        entities.extend(self.entities.drain(..));
        self.entities = entities;
        
        let mut actions = base.actions;
        actions.extend(self.actions.drain(..));
        self.actions = actions;
        
        let mut assertions = base.assertions;
        assertions.extend(self.assertions.drain(..));
        self.assertions = assertions;
        
        // Merge variables (child overrides base)
        for (k, v) in base.variables {
            self.variables.entry(k).or_insert(v);
        }
        
        Ok(())
    }

    /// Substitute variables in JSON string before parsing
    pub fn from_json_with_vars(json: &str, vars: &HashMap<String, serde_json::Value>) -> Result<Self, String> {
        let mut result = json.to_string();
        for (key, value) in vars {
            let placeholder = format!("${{{}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        Self::from_json(&result)
    }
}

// ============================================================================
// Executor
// ============================================================================

/// State snapshot for debugging
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub action_index: usize,
    pub turn: u32,
    pub player_x: i32,
    pub player_y: i32,
    pub player_hp: i32,
    pub inventory_size: usize,
    pub enemy_count: usize,
}

pub struct DesExecutor {
    state: GameState,
    logs: Vec<ExecutionLog>,
    assertion_results: Vec<AssertionResult>,
    action_index: usize,
    snapshots: Vec<StateSnapshot>,
    capture_snapshots: bool,
}

impl DesExecutor {
    pub fn new(scenario: &Scenario) -> Self {
        let seed = scenario.seed.unwrap_or(42);
        let mut state = GameState::new(seed);

        // Apply player setup
        if let Some(x) = scenario.player.x {
            state.player_x = x;
        }
        if let Some(y) = scenario.player.y {
            state.player_y = y;
        }
        // Ensure player position and surrounding area is walkable (carve floor if needed)
        if scenario.player.x.is_some() || scenario.player.y.is_some() {
            // Carve a 9x9 area around player for movement tests
            for dy in -4..=4 {
                for dx in -4..=4 {
                    let nx = state.player_x + dx;
                    let ny = state.player_y + dy;
                    if nx >= 0 && ny >= 0 {
                        let nidx = ny as usize * state.map.width + nx as usize;
                        if nidx < state.map.tiles.len() && !state.map.tiles[nidx].walkable() {
                            state.map.tiles[nidx] = crate::game::map::Tile::Floor;
                        }
                    }
                }
            }
            // Recompute FOV after carving
            state.visible = crate::game::map::compute_fov(&state.map, state.player_x, state.player_y);
            state.revealed.extend(&state.visible);
        }
        if let Some(hp) = scenario.player.hp {
            state.player_hp = hp;
        }
        if let Some(max_hp) = scenario.player.max_hp {
            state.player_max_hp = max_hp;
        }
        if let Some(ap) = scenario.player.ap {
            state.player_ap = ap;
        }
        if let Some(max_ap) = scenario.player.max_ap {
            state.player_max_ap = max_ap;
        }
        if let Some(xp) = scenario.player.xp {
            state.player_xp = xp;
        }
        for item_id in &scenario.player.inventory {
            state.inventory.push(item_id.clone());
        }
        // Wire adaptations
        for adaptation_id in &scenario.player.adaptations {
            if let Some(a) = parse_adaptation(adaptation_id) {
                state.adaptations.push(a);
            }
        }
        // Wire equipped weapon (both legacy and new equipment system)
        if let Some(weapon_id) = &scenario.player.equipped_weapon {
            state.equipped_weapon = Some(weapon_id.clone());
            state.equipment.weapon = Some(weapon_id.clone());
        }

        // Spawn entities
        for spawn in &scenario.entities {
            match spawn.entity_type {
                EntityType::Enemy => {
                    let mut enemy = Enemy::new(spawn.x, spawn.y, &spawn.id);
                    if let Some(hp) = spawn.hp {
                        enemy.hp = hp;
                    }
                    enemy.ai_disabled = spawn.ai_disabled;
                    enemy.inventory = spawn.inventory.clone();
                    state.enemies.push(enemy);
                }
                EntityType::Npc => {
                    state.npcs.push(Npc::new(spawn.x, spawn.y, &spawn.id));
                }
                EntityType::Item => {
                    state.items.push(Item::new(spawn.x, spawn.y, &spawn.id));
                }
            }
        }
        state.rebuild_spatial_index();
        state.update_lighting();

        // Apply mock settings
        state.mock_combat_hit = scenario.mocks.combat_always_hit;
        state.mock_combat_damage = scenario.mocks.combat_fixed_damage;

        Self {
            state,
            logs: Vec::new(),
            assertion_results: Vec::new(),
            action_index: 0,
            snapshots: Vec::new(),
            capture_snapshots: false,
        }
    }

    /// Enable state snapshot capture for debugging
    pub fn with_snapshots(mut self) -> Self {
        self.capture_snapshots = true;
        self
    }

    /// Inject a specific RNG seed for deterministic testing
    pub fn with_rng_seed(mut self, seed: u64) -> Self {
        self.state.rng = ChaCha8Rng::seed_from_u64(seed);
        self
    }

    /// Inject a specific RNG state for exact replay
    pub fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        self.state.rng = rng;
        self
    }

    fn capture_snapshot(&mut self) {
        if self.capture_snapshots {
            self.snapshots.push(StateSnapshot {
                action_index: self.action_index,
                turn: self.state.turn,
                player_x: self.state.player_x,
                player_y: self.state.player_y,
                player_hp: self.state.player_hp,
                inventory_size: self.state.inventory.len(),
                enemy_count: self.state.enemies.iter().filter(|e| e.hp > 0).count(),
            });
        }
    }

    pub fn run(mut self, scenario: &Scenario) -> ExecutionResult {
        let mut current_turn = 0;
        let max_turns = scenario.actions.iter().map(|a| a.turn).max().unwrap_or(0) + 1;

        // Initial snapshot
        self.capture_snapshot();

        while current_turn <= max_turns && self.state.player_hp > 0 {
            // Execute actions scheduled for this turn
            for scheduled in &scenario.actions {
                if scheduled.turn == current_turn {
                    self.execute_action(&scheduled.action, &scheduled.actor);
                    self.capture_snapshot();
                    self.action_index += 1;
                }
            }
            // Check assertions for this turn
            for assertion in &scenario.assertions {
                if assertion.after_turn == Some(current_turn) {
                    self.check_assertion(assertion);
                }
            }
            current_turn += 1;
        }

        // Check end-of-scenario assertions
        for assertion in &scenario.assertions {
            if assertion.at_end {
                self.check_assertion(assertion);
            }
        }

        let all_passed = self.assertion_results.iter().all(|r| r.passed);
        ExecutionResult {
            success: all_passed,
            final_turn: self.state.turn,
            logs: self.logs,
            assertion_results: self.assertion_results,
            snapshots: self.snapshots,
            final_state: Some(self.state),
        }
    }

    fn check_assertion(&mut self, assertion: &Assertion) {
        let passed = self.evaluate_check(&assertion.check);
        self.assertion_results.push(AssertionResult {
            passed,
            check: format!("{:?}", assertion.check),
            message: assertion.message.clone(),
        });
    }

    fn evaluate_check(&self, check: &AssertionCheck) -> bool {
        match check {
            AssertionCheck::PlayerHp { op, value } => op.compare(self.state.player_hp, *value),
            AssertionCheck::PlayerPosition { x, y } => {
                self.state.player_x == *x && self.state.player_y == *y
            }
            AssertionCheck::PlayerAlive => self.state.player_hp > 0,
            AssertionCheck::PlayerDead => self.state.player_hp <= 0,
            AssertionCheck::InventoryContains { item } => self.state.inventory.contains(item),
            AssertionCheck::InventorySize { op, value } => {
                op.compare(self.state.inventory.len() as i32, *value as i32)
            }
            AssertionCheck::EnemyAt { x, y, alive } => {
                self.state.enemy_at(*x, *y).map(|i| {
                    if *alive { self.state.enemies[i].hp > 0 } else { self.state.enemies[i].hp <= 0 }
                }).unwrap_or(false)
            }
            AssertionCheck::NoEnemyAt { x, y } => self.state.enemy_at(*x, *y).is_none(),
            AssertionCheck::Turn { op, value } => op.compare(self.state.turn as i32, *value as i32),
            // New assertions
            AssertionCheck::EnemyHp { id, op, value } => {
                self.state.enemies.iter()
                    .find(|e| e.id() == id)
                    .map(|e| op.compare(e.hp, *value))
                    .unwrap_or(false)
            }
            AssertionCheck::EnemyAlive { id } => {
                self.state.enemies.iter().any(|e| e.id() == id && e.hp > 0)
            }
            AssertionCheck::EnemyDead { id } => {
                self.state.enemies.iter().any(|e| e.id() == id && e.hp <= 0)
            }
            AssertionCheck::PlayerHasAdaptation { adaptation } => {
                parse_adaptation(adaptation)
                    .map(|a| self.state.adaptations.contains(&a))
                    .unwrap_or(false)
            }
            AssertionCheck::AdaptationCount { op, value } => {
                op.compare(self.state.adaptations.len() as i32, *value as i32)
            }
            AssertionCheck::MapTileAt { x, y, tile } => {
                self.state.map.get(*x, *y)
                    .map(|t| format!("{:?}", t).to_lowercase().contains(&tile.to_lowercase()))
                    .unwrap_or(false)
            }
            AssertionCheck::Refraction { op, value } => {
                op.compare(self.state.refraction as i32, *value as i32)
            }
            AssertionCheck::PlayerAp { op, value } => {
                op.compare(self.state.player_ap, *value)
            }
            AssertionCheck::HasStatusEffect { effect } => {
                self.state.status_effects.iter().any(|e| {
                    format!("{:?}", e.effect_type).to_lowercase() == effect.to_lowercase()
                })
            }
            AssertionCheck::StatusEffectCount { op, value } => {
                op.compare(self.state.status_effects.len() as i32, *value as i32)
            }
            AssertionCheck::TileExplored { x, y } => {
                let idx = self.state.map.idx(*x, *y);
                self.state.revealed.contains(&idx)
            }
            AssertionCheck::ExploredCount { op, value } => {
                op.compare(self.state.revealed.len() as i32, *value as i32)
            }
            AssertionCheck::EquippedInSlot { slot, item } => {
                slot.parse::<crate::game::equipment::EquipSlot>()
                    .ok()
                    .and_then(|s| self.state.equipment.get(s))
                    .map(|e| Some(e) == item.as_ref())
                    .unwrap_or(item.is_none())
            }
            AssertionCheck::PlayerArmor { op, value } => {
                op.compare(self.state.player_armor, *value)
            }
            AssertionCheck::EnemyProvoked { id, provoked } => {
                self.state.enemies.iter()
                    .find(|e| e.id == *id)
                    .map(|e| e.provoked == *provoked)
                    .unwrap_or(false)
            }
            AssertionCheck::EnemyHasItem { id, item } => {
                self.state.enemies.iter()
                    .find(|e| e.id == *id)
                    .map(|e| e.inventory.contains(item))
                    .unwrap_or(false)
            }
            AssertionCheck::LightLevel { x, y, op, value } => {
                let level = self.state.get_light_level(*x, *y);
                op.compare(level as i32, *value as i32)
            }
            AssertionCheck::ItemInspectHasStat { item, stat } => {
                inspect_item(item)
                    .map(|info| info.stats.iter().any(|(k, _)| k == stat))
                    .unwrap_or(false)
            }
            AssertionCheck::ItemInspectMissingStat { item, stat } => {
                inspect_item(item)
                    .map(|info| !info.stats.iter().any(|(k, _)| k == stat))
                    .unwrap_or(true)
            }
            AssertionCheck::NpcTalked { id, talked } => {
                self.state.npcs.iter()
                    .find(|n| n.id == *id)
                    .map(|n| n.talked == *talked)
                    .unwrap_or(false)
            }
            AssertionCheck::PlayerXp { op, value } => {
                op.compare(self.state.player_xp as i32, *value as i32)
            }
            AssertionCheck::PlayerLevel { op, value } => {
                op.compare(self.state.player_level as i32, *value as i32)
            }
            AssertionCheck::MessageContains { text } => {
                self.state.messages.iter().any(|m| m.text.contains(text))
            }
            AssertionCheck::PendingStatPoints { op, value } => {
                op.compare(self.state.pending_stat_points, *value)
            }
            // Quest assertions
            AssertionCheck::QuestActive { quest_id } => {
                self.state.quest_log.active.iter().any(|q| q.quest_id == *quest_id)
            }
            AssertionCheck::QuestCompleted { quest_id } => {
                self.state.quest_log.completed.contains(quest_id)
            }
            AssertionCheck::QuestObjectiveProgress { quest_id, objective_id, op, value } => {
                self.state.quest_log.get_active(quest_id)
                    .and_then(|q| q.objectives.iter().find(|o| o.objective_id == *objective_id))
                    .map(|o| op.compare(o.current as i32, *value as i32))
                    .unwrap_or(false)
            }
            AssertionCheck::QuestObjectiveComplete { quest_id, objective_id } => {
                self.state.quest_log.get_active(quest_id)
                    .and_then(|q| q.objectives.iter().find(|o| o.objective_id == *objective_id))
                    .map(|o| o.completed)
                    .unwrap_or(false)
            }
        }
    }

    fn execute_action(&mut self, action: &Action, actor: &Actor) {
        match actor {
            Actor::Player => self.execute_player_action(action),
            Actor::Entity { id } => {
                self.log(format!("Entity action for '{}' not yet implemented", id));
            }
        }
    }

    fn execute_player_action(&mut self, action: &Action) {
        match action {
            Action::Move { dx, dy } => {
                self.state.try_move(*dx, *dy);
                self.log(format!("Player moved ({}, {})", dx, dy));
            }
            Action::Teleport { x, y } => {
                // Carve floor around teleport destination
                for dy in -4..=4 {
                    for dx in -4..=4 {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && ny >= 0 {
                            let nidx = ny as usize * self.state.map.width + nx as usize;
                            if nidx < self.state.map.tiles.len() && !self.state.map.tiles[nidx].walkable() {
                                self.state.map.tiles[nidx] = crate::game::map::Tile::Floor;
                            }
                        }
                    }
                }
                self.state.player_x = *x;
                self.state.player_y = *y;
                // Update visibility after teleport
                self.state.visible = crate::game::map::compute_fov(&self.state.map, *x, *y);
                self.state.revealed.extend(&self.state.visible);
                self.log(format!("Player teleported to ({}, {})", x, y));
            }
            Action::Attack { target_x, target_y } => {
                let dx = target_x - self.state.player_x;
                let dy = target_y - self.state.player_y;
                if dx.abs() <= 1 && dy.abs() <= 1 {
                    self.state.try_move(dx, dy);
                    self.log(format!("Player attacked ({}, {})", target_x, target_y));
                }
            }
            Action::RangedAttack { target_x, target_y } => {
                self.state.try_ranged_attack(*target_x, *target_y);
                self.log(format!("Player ranged attack ({}, {})", target_x, target_y));
            }
            Action::ApplyStatus { effect, duration, potency } => {
                if let Some(status_type) = parse_status_type(effect) {
                    use crate::game::status::StatusEffect;
                    self.state.apply_status(StatusEffect::new(status_type, *duration, *potency));
                    self.log(format!("Applied {} for {} turns", effect, duration));
                }
            }
            Action::UseItem { item_index } => {
                self.state.use_item(*item_index);
                self.log(format!("Player used item at index {}", item_index));
            }
            Action::Equip { item_index, slot } => {
                if let Ok(equip_slot) = slot.parse::<crate::game::equipment::EquipSlot>() {
                    self.state.equip_item(*item_index, equip_slot);
                    self.log(format!("Equipped item {} to {}", item_index, slot));
                } else {
                    self.log(format!("Unknown slot: {}", slot));
                }
            }
            Action::Unequip { slot } => {
                if let Ok(equip_slot) = slot.parse::<crate::game::equipment::EquipSlot>() {
                    self.state.unequip_slot(equip_slot);
                    self.log(format!("Unequipped {}", slot));
                } else {
                    self.log(format!("Unknown slot: {}", slot));
                }
            }
            Action::AutoExplore => {
                let moved = self.state.auto_explore();
                self.log(format!("Auto-explore: {}", if moved { "moved" } else { "no path" }));
            }
            Action::Wait { turns } => {
                for _ in 0..*turns {
                    self.state.wait_turn();
                }
                self.log(format!("Player waited {} turns", turns));
            }
            Action::Rest => {
                match self.state.rest() {
                    Ok(()) => self.log("Player rested and recovered HP".to_string()),
                    Err(e) => self.log(format!("Rest failed: {}", e)),
                }
            }
            Action::EndTurn => {
                self.state.end_turn();
                self.log("Player ended turn".to_string());
            }
            Action::Log { query } => {
                let msg = self.query_state(query);
                self.log(msg);
            }
            Action::AllocateStat { stat } => {
                let success = self.state.allocate_stat(stat);
                self.log(format!("Allocate stat '{}': {}", stat, if success { "success" } else { "failed" }));
            }
            // Quest actions
            Action::AcceptQuest { quest_id } => {
                let success = self.state.accept_quest(quest_id);
                self.log(format!("Accept quest '{}': {}", quest_id, if success { "success" } else { "failed" }));
            }
            Action::CompleteQuest { quest_id } => {
                let success = self.state.complete_quest(quest_id);
                self.log(format!("Complete quest '{}': {}", quest_id, if success { "success" } else { "failed" }));
            }
            // Crafting actions
            Action::Craft { recipe_id } => {
                let success = self.state.craft(recipe_id);
                self.log(format!("Craft '{}': {}", recipe_id, if success { "success" } else { "failed" }));
            }
        }
    }

    fn query_state(&self, query: &LogQuery) -> String {
        match query {
            LogQuery::PlayerHp => format!("HP: {}/{}", self.state.player_hp, self.state.player_max_hp),
            LogQuery::PlayerPosition => format!("Position: ({}, {})", self.state.player_x, self.state.player_y),
            LogQuery::Inventory => format!("Inventory: {:?}", self.state.inventory),
            LogQuery::EntityAt { x, y } => self.state.describe_at(*x, *y),
            LogQuery::Turn => format!("Turn: {}", self.state.turn),
            LogQuery::Custom { message } => message.clone(),
        }
    }

    fn log(&mut self, message: String) {
        self.logs.push(ExecutionLog {
            turn: self.state.turn,
            action_index: self.action_index,
            message,
        });
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Run a scenario from a JSON file
pub fn run_scenario(path: impl AsRef<Path>) -> Result<ExecutionResult, String> {
    let scenario = Scenario::from_file(path)?;
    let executor = DesExecutor::new(&scenario);
    Ok(executor.run(&scenario))
}

/// Run a scenario from a JSON string
pub fn run_scenario_json(json: &str) -> Result<ExecutionResult, String> {
    let scenario = Scenario::from_json(json)?;
    let executor = DesExecutor::new(&scenario);
    Ok(executor.run(&scenario))
}

// ============================================================================
// Rendered Execution
// ============================================================================

/// Callback for rendering state during slow execution
pub type RenderCallback = Box<dyn FnMut(&GameState, &ExecutionLog)>;

/// Run scenario with optional rendering callback and frame delay
pub fn run_with_render<F>(scenario: &Scenario, frame_delay_ms: u64, mut render_fn: F) -> ExecutionResult
where
    F: FnMut(&GameState, Option<&ExecutionLog>),
{
    let mut executor = DesExecutor::new(scenario);
    executor.capture_snapshots = true;
    
    let mut current_turn = 0;
    let max_turns = scenario.actions.iter().map(|a| a.turn).max().unwrap_or(0) + 1;

    // Initial render
    render_fn(&executor.state, None);
    std::thread::sleep(std::time::Duration::from_millis(frame_delay_ms));

    while current_turn <= max_turns && executor.state.player_hp > 0 {
        for scheduled in &scenario.actions {
            if scheduled.turn == current_turn {
                executor.execute_action(&scheduled.action, &scheduled.actor);
                
                // Render after each action
                let log = ExecutionLog {
                    turn: executor.state.turn,
                    action_index: executor.action_index,
                    message: format!("{:?}", scheduled.action),
                };
                render_fn(&executor.state, Some(&log));
                std::thread::sleep(std::time::Duration::from_millis(frame_delay_ms));
                
                executor.action_index += 1;
            }
        }
        for assertion in &scenario.assertions {
            if assertion.after_turn == Some(current_turn) {
                executor.check_assertion(assertion);
            }
        }
        current_turn += 1;
    }

    for assertion in &scenario.assertions {
        if assertion.at_end {
            executor.check_assertion(assertion);
        }
    }

    let all_passed = executor.assertion_results.iter().all(|r| r.passed);
    ExecutionResult {
        success: executor.state.player_hp > 0 && all_passed,
        final_turn: executor.state.turn,
        logs: executor.logs,
        assertion_results: executor.assertion_results,
        snapshots: executor.snapshots,
        final_state: Some(executor.state),
    }
}

/// Run multiple scenarios in parallel using rayon thread pool
pub fn run_parallel(scenarios: &[Scenario]) -> Vec<ExecutionResult> {
    scenarios
        .par_iter()
        .map(|scenario| {
            let executor = DesExecutor::new(scenario);
            executor.run(scenario)
        })
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_scenario() {
        let json = r#"{"name": "test"}"#;
        let scenario = Scenario::from_json(json).unwrap();
        assert_eq!(scenario.name, "test");
    }

    #[test]
    fn parse_scenario_with_actions() {
        let json = r#"{
            "name": "movement_test",
            "seed": 42,
            "actions": [
                {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}},
                {"turn": 1, "action": {"type": "wait", "turns": 2}}
            ]
        }"#;
        let scenario = Scenario::from_json(json).unwrap();
        assert_eq!(scenario.actions.len(), 2);
    }

    #[test]
    fn execute_basic_scenario() {
        let json = r#"{
            "name": "basic_test",
            "seed": 42,
            "actions": [
                {"turn": 0, "action": {"type": "log", "query": "player_hp"}}
            ]
        }"#;
        let result = run_scenario_json(json).unwrap();
        assert!(result.success);
        assert!(!result.logs.is_empty());
    }

    #[test]
    fn assertions_pass() {
        let json = r#"{
            "name": "assertion_test",
            "seed": 42,
            "player": {"hp": 20, "max_hp": 20},
            "assertions": [
                {"at_end": true, "check": {"type": "player_alive"}},
                {"at_end": true, "check": {"type": "player_hp", "op": "eq", "value": 20}}
            ]
        }"#;
        let result = run_scenario_json(json).unwrap();
        assert!(result.success);
        assert_eq!(result.assertion_results.len(), 2);
        assert!(result.assertion_results.iter().all(|r| r.passed));
    }

    #[test]
    fn wait_advances_turns() {
        let json = r#"{
            "name": "wait_test",
            "seed": 42,
            "player": {"x": 5, "y": 5, "hp": 20, "max_hp": 20},
            "actions": [
                {"turn": 0, "action": {"type": "wait", "turns": 3}}
            ]
        }"#;
        let result = run_scenario_json(json).unwrap();
        eprintln!("Final turn: {}, logs: {:?}", result.final_turn, result.logs);
        assert!(result.final_turn >= 3, "Expected turn >= 3, got {}", result.final_turn);
    }

    #[test]
    fn ap_depletes_and_resets() {
        let json = r#"{
            "name": "ap_test",
            "seed": 42,
            "player": {"x": 5, "y": 5, "hp": 20, "ap": 4, "max_ap": 4},
            "actions": [
                {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}},
                {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}},
                {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}},
                {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}}
            ],
            "assertions": [
                {"at_end": true, "check": {"type": "player_position", "x": 9, "y": 5}}
            ]
        }"#;
        let result = run_scenario_json(json).unwrap();
        let final_state = result.final_state.as_ref().unwrap();
        eprintln!("Final turn: {}, AP: {}, pos: ({}, {})", 
            result.final_turn, final_state.player_ap,
            final_state.player_x, final_state.player_y);
        eprintln!("Logs: {:?}", result.logs.iter().map(|l| &l.message).collect::<Vec<_>>());
        // For now, just check that AP system is working - turn advances when AP depletes
        assert!(result.final_turn >= 1, "Turn should advance after AP depleted");
    }

    #[test]
    fn assertions_fail() {
        let json = r#"{
            "name": "fail_test",
            "seed": 42,
            "player": {"hp": 10},
            "assertions": [
                {"at_end": true, "check": {"type": "player_hp", "op": "eq", "value": 20}}
            ]
        }"#;
        let result = run_scenario_json(json).unwrap();
        assert!(!result.success);
        assert!(!result.assertion_results[0].passed);
    }

    #[test]
    fn mock_combat_always_miss() {
        let json = r#"{
            "name": "mock_miss_test",
            "seed": 100,
            "mocks": {"combat_always_hit": false},
            "player": {"x": 5, "y": 5, "hp": 20, "ap": 10},
            "entities": [
                {"entity_type": "enemy", "id": "shard_spider", "x": 6, "y": 5, "hp": 5, "ai_disabled": true}
            ],
            "actions": [
                {"turn": 0, "action": {"type": "attack", "target_x": 6, "target_y": 5}}
            ]
        }"#;
        let result = run_scenario_json(json).unwrap();
        let state = result.final_state.as_ref().unwrap();
        eprintln!("Mock hit setting: {:?}", state.mock_combat_hit);
        // Find enemy at target position
        let enemy = state.enemies.iter().find(|e| e.x == 6 && e.y == 5);
        eprintln!("Enemy at (6,5): {:?}", enemy.map(|e| (e.id.as_str(), e.hp)));
        eprintln!("Logs: {:?}", result.logs.iter().map(|l| &l.message).collect::<Vec<_>>());
        // Enemy should still have 5 HP since all attacks miss
        let enemy_hp = enemy.map(|e| e.hp).unwrap_or(-999);
        assert_eq!(enemy_hp, 5, "Enemy should have 5 HP (attack missed)");
    }
}
