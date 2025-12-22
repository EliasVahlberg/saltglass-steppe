use std::fs;
use serde::{Deserialize, Serialize};
use crate::GameState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesTest {
    pub name: String,
    pub description: String,
    pub initial_state_file: String,
    pub actions: Vec<DesAction>,
    pub expected_outcomes: Vec<DesExpectation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesAction {
    pub action_type: String,
    pub parameters: Vec<String>,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesExpectation {
    pub check_type: String,
    pub expected_value: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesTestResult {
    pub test_name: String,
    pub passed: bool,
    pub failed_expectations: Vec<String>,
    pub execution_log: Vec<String>,
    pub final_state_file: Option<String>,
}

impl DesTest {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let test: DesTest = serde_json::from_str(&content)?;
        Ok(test)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn execute(&self) -> Result<DesTestResult, Box<dyn std::error::Error>> {
        let mut result = DesTestResult {
            test_name: self.name.clone(),
            passed: true,
            failed_expectations: Vec::new(),
            execution_log: Vec::new(),
            final_state_file: None,
        };

        // Load initial state
        let mut state = GameState::load_debug_state(&self.initial_state_file)?;
        result.execution_log.push(format!("Loaded initial state: {}", self.initial_state_file));

        // Execute actions
        for (i, action) in self.actions.iter().enumerate() {
            result.execution_log.push(format!("Action {}: {} - {}", i + 1, action.action_type, action.description));
            
            match self.execute_action(&mut state, action) {
                Ok(msg) => result.execution_log.push(format!("  Success: {}", msg)),
                Err(e) => {
                    result.execution_log.push(format!("  Error: {}", e));
                    result.passed = false;
                }
            }
        }

        // Check expectations
        for expectation in &self.expected_outcomes {
            match self.check_expectation(&state, expectation) {
                Ok(true) => {
                    result.execution_log.push(format!("✓ {}", expectation.description));
                }
                Ok(false) => {
                    result.execution_log.push(format!("✗ {}", expectation.description));
                    result.failed_expectations.push(expectation.description.clone());
                    result.passed = false;
                }
                Err(e) => {
                    result.execution_log.push(format!("✗ {} (Error: {})", expectation.description, e));
                    result.failed_expectations.push(format!("{} (Error: {})", expectation.description, e));
                    result.passed = false;
                }
            }
        }

        // Save final state
        let final_state_filename = format!("des_final_{}_{}.ron", self.name, chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        match state.save_debug_state(&final_state_filename) {
            Ok(_) => {
                result.final_state_file = Some(final_state_filename.clone());
                result.execution_log.push(format!("Final state saved: {}", final_state_filename));
            }
            Err(e) => {
                result.execution_log.push(format!("Failed to save final state: {}", e));
            }
        }

        Ok(result)
    }

    fn execute_action(&self, state: &mut GameState, action: &DesAction) -> Result<String, Box<dyn std::error::Error>> {
        match action.action_type.as_str() {
            "move" => {
                if action.parameters.len() >= 2 {
                    let dx: i32 = action.parameters[0].parse()?;
                    let dy: i32 = action.parameters[1].parse()?;
                    let old_pos = (state.player_x, state.player_y);
                    state.try_move(dx, dy);
                    Ok(format!("Moved from ({},{}) to ({},{})", old_pos.0, old_pos.1, state.player_x, state.player_y))
                } else {
                    Err("Move action requires dx and dy parameters".into())
                }
            }
            "use_item" => {
                if action.parameters.len() >= 1 {
                    let idx: usize = action.parameters[0].parse()?;
                    if idx < state.inventory.len() {
                        let item_id = state.inventory[idx].clone();
                        state.use_item(idx);
                        Ok(format!("Used item: {}", item_id))
                    } else {
                        Err(format!("Item index {} out of range", idx).into())
                    }
                } else {
                    Err("Use item action requires item index parameter".into())
                }
            }
            "wait" => {
                if action.parameters.len() >= 1 {
                    let turns: u32 = action.parameters[0].parse()?;
                    for _ in 0..turns {
                        state.end_turn();
                    }
                    Ok(format!("Waited {} turns", turns))
                } else {
                    state.end_turn();
                    Ok("Waited 1 turn".to_string())
                }
            }
            "debug_command" => {
                if action.parameters.len() >= 1 {
                    let cmd = action.parameters.join(" ");
                    state.debug_command(&cmd);
                    Ok(format!("Executed debug command: {}", cmd))
                } else {
                    Err("Debug command action requires command parameter".into())
                }
            }
            _ => Err(format!("Unknown action type: {}", action.action_type).into())
        }
    }

    fn check_expectation(&self, state: &GameState, expectation: &DesExpectation) -> Result<bool, Box<dyn std::error::Error>> {
        match expectation.check_type.as_str() {
            "player_hp" => {
                let expected: i32 = expectation.expected_value.parse()?;
                Ok(state.player_hp == expected)
            }
            "player_position" => {
                let coords: Vec<&str> = expectation.expected_value.split(',').collect();
                if coords.len() == 2 {
                    let x: i32 = coords[0].parse()?;
                    let y: i32 = coords[1].parse()?;
                    Ok(state.player_x == x && state.player_y == y)
                } else {
                    Err("Position expectation must be in format 'x,y'".into())
                }
            }
            "inventory_contains" => {
                Ok(state.inventory.contains(&expectation.expected_value))
            }
            "inventory_count" => {
                let expected: usize = expectation.expected_value.parse()?;
                Ok(state.inventory.len() == expected)
            }
            "turn_number" => {
                let expected: u32 = expectation.expected_value.parse()?;
                Ok(state.turn == expected)
            }
            "enemy_count" => {
                let expected: usize = expectation.expected_value.parse()?;
                Ok(state.enemies.len() == expected)
            }
            _ => Err(format!("Unknown expectation type: {}", expectation.check_type).into())
        }
    }
}

pub fn run_des_test_file(path: &str) -> Result<DesTestResult, Box<dyn std::error::Error>> {
    let test = DesTest::load_from_file(path)?;
    test.execute()
}

pub fn list_des_tests() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut tests = Vec::new();
    
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with("_test.des") || name.ends_with(".des") {
                tests.push(name.to_string());
            }
        }
    }
    
    tests.sort();
    Ok(tests)
}

pub fn create_sample_des_test() -> DesTest {
    DesTest {
        name: "sample_movement_test".to_string(),
        description: "Test basic player movement and position tracking".to_string(),
        initial_state_file: "debug_initial_state.ron".to_string(),
        actions: vec![
            DesAction {
                action_type: "move".to_string(),
                parameters: vec!["1".to_string(), "0".to_string()],
                description: "Move right".to_string(),
            },
            DesAction {
                action_type: "move".to_string(),
                parameters: vec!["0".to_string(), "1".to_string()],
                description: "Move down".to_string(),
            },
            DesAction {
                action_type: "wait".to_string(),
                parameters: vec!["1".to_string()],
                description: "Wait one turn".to_string(),
            },
        ],
        expected_outcomes: vec![
            DesExpectation {
                check_type: "turn_number".to_string(),
                expected_value: "3".to_string(),
                description: "Should be on turn 3".to_string(),
            },
        ],
    }
}
