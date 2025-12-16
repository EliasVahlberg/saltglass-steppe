//! Debug Execution System (DES) - Headless game execution for automated testing
//!
//! Runs game scenarios without rendering for automated testing and validation.

use crate::game::{Enemy, GameState, Item, Npc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub entities: Vec<EntitySpawn>,
    #[serde(default)]
    pub player: PlayerSetup,
    #[serde(default)]
    pub actions: Vec<ScheduledAction>,
    #[serde(default)]
    pub base: Option<String>, // BLOCKED: inheritance not implemented
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
    pub inventory: Vec<String>,
    #[serde(default)]
    pub adaptations: Vec<String>,
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
    UseItem { item_index: usize },
    Wait { turns: u32 },
    Log { query: LogQuery },
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
    pub final_state: Option<GameState>,
}

#[derive(Debug, Clone)]
pub struct ExecutionLog {
    pub turn: u32,
    pub action_index: usize,
    pub message: String,
}

// ============================================================================
// Parser
// ============================================================================

impl Scenario {
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        Self::from_json(&content)
    }

    /// BLOCKED: Base file inheritance not implemented
    pub fn inherit_from(&mut self, _base_path: &str) -> Result<(), String> {
        unimplemented!("BLOCKED: Base file inheritance requires variable system implementation")
    }
}

// ============================================================================
// Executor
// ============================================================================

pub struct DesExecutor {
    state: GameState,
    logs: Vec<ExecutionLog>,
    action_index: usize,
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
        if let Some(hp) = scenario.player.hp {
            state.player_hp = hp;
        }
        if let Some(max_hp) = scenario.player.max_hp {
            state.player_max_hp = max_hp;
        }
        for item_id in &scenario.player.inventory {
            state.inventory.push(item_id.clone());
        }

        // Spawn entities
        for spawn in &scenario.entities {
            match spawn.entity_type {
                EntityType::Enemy => {
                    let mut enemy = Enemy::new(spawn.x, spawn.y, &spawn.id);
                    if let Some(hp) = spawn.hp {
                        enemy.hp = hp;
                    }
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

        Self {
            state,
            logs: Vec::new(),
            action_index: 0,
        }
    }

    pub fn run(mut self, scenario: &Scenario) -> ExecutionResult {
        let mut current_turn = 0;
        let max_turns = scenario.actions.iter().map(|a| a.turn).max().unwrap_or(0) + 1;

        while current_turn <= max_turns && self.state.player_hp > 0 {
            // Execute actions scheduled for this turn
            for scheduled in &scenario.actions {
                if scheduled.turn == current_turn {
                    self.execute_action(&scheduled.action, &scheduled.actor);
                    self.action_index += 1;
                }
            }
            current_turn += 1;
        }

        ExecutionResult {
            success: self.state.player_hp > 0,
            final_turn: self.state.turn,
            logs: self.logs,
            final_state: Some(self.state),
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
                self.state.player_x = *x;
                self.state.player_y = *y;
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
            Action::UseItem { item_index } => {
                self.state.use_item(*item_index);
                self.log(format!("Player used item at index {}", item_index));
            }
            Action::Wait { turns } => {
                for _ in 0..*turns {
                    self.state.try_move(0, 0);
                }
                self.log(format!("Player waited {} turns", turns));
            }
            Action::Log { query } => {
                let msg = self.query_state(query);
                self.log(msg);
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
// BLOCKED: Dummy Implementations
// ============================================================================

/// BLOCKED: Run multiple scenarios in parallel
pub fn run_parallel(_scenarios: &[Scenario]) -> Vec<ExecutionResult> {
    unimplemented!("BLOCKED: Parallel execution requires thread-safe DES executor")
}

/// BLOCKED: Run with system mocking
pub fn run_with_mocks(_scenario: &Scenario, _mocks: &HashMap<String, serde_json::Value>) -> ExecutionResult {
    unimplemented!("BLOCKED: System mocking requires dependency injection in GameState")
}

/// BLOCKED: Run with rendering and slow execution
pub fn run_rendered(_scenario: &Scenario, _frame_delay_ms: u64) -> ExecutionResult {
    unimplemented!("BLOCKED: Rendered execution requires UI decoupling and frame control")
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
}
