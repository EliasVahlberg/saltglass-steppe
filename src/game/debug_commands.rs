//! Debug command execution system
//!
//! This module contains debug commands extracted from the main GameState
//! to reduce the size of state.rs. All debug functionality is preserved
//! but moved to a dedicated module for better organization.

/// Execute a debug command on the given game state
pub fn execute(state: &mut super::state::GameState, cmd: &str) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    match parts.first().copied() {
        Some("show") if parts.get(1) == Some(&"tile") => {
            state.debug.god_view = true;
            state.log("Debug: God view enabled");
        }
        Some("hide") if parts.get(1) == Some(&"tile") => {
            state.debug.god_view = false;
            state.log("Debug: God view disabled");
        }
        Some("sturdy") => {
            state.player.hp = 9999;
            state.player.max_hp = 9999;
            state.log("Debug: HP set to 9999/9999");
        }
        Some("phase") => {
            state.debug.phase = !state.debug.phase;
            state.log(format!(
                "Debug: Phase {}",
                if state.debug.phase {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
        }
        Some("save_debug") => {
            let filename = if parts.len() > 1 {
                format!("{}.ron", parts[1])
            } else {
                format!("debug_{}.ron", chrono::Utc::now().format("%Y%m%d_%H%M%S"))
            };
            match state.save_debug_state(&filename) {
                Ok(_) => state.log(format!("Debug state saved: {}", filename)),
                Err(e) => state.log(format!("Failed to save debug state: {}", e)),
            }
        }
        Some("load_debug") => {
            if let Some(filename) = parts.get(1) {
                match super::state::GameState::load_debug_state(filename) {
                    Ok(new_state) => {
                        *state = new_state;
                        state.log(format!("Debug state loaded: {}", filename));
                    }
                    Err(e) => state.log(format!("Failed to load debug state: {}", e)),
                }
            } else {
                state.log("Usage: load_debug <filename>");
            }
        }
        Some("list_debug") => match super::state::GameState::list_debug_states() {
            Ok(states) => {
                if states.is_empty() {
                    state.log("No debug states found");
                } else {
                    state.log("Debug states:");
                    for debug_state in states {
                        state.log(format!("  {}", debug_state));
                    }
                }
            }
            Err(e) => state.log(format!("Failed to list debug states: {}", e)),
        },
        Some("debug_info") => {
            let info = state.get_debug_info();
            state.log(format!(
                "Turn: {} | Pos: ({},{}) | HP: {}/{}",
                info.turn, info.player_pos.0, info.player_pos.1, info.player_hp.0, info.player_hp.1
            ));
            state.log(format!(
                "Enemies: {} | Items: {} | Storm: {}/{}",
                info.enemies_count, info.items_count, info.storm_intensity, info.storm_turns
            ));
            state.log(format!(
                "Seed: {} | Memory: {}",
                info.seed, info.memory_usage
            ));
        }
        Some("run_des") => {
            if let Some(filename) = parts.get(1) {
                match super::des_testing::run_des_test_file(filename) {
                    Ok(result) => {
                        state.log(format!(
                            "DES Test '{}': {}",
                            result.test_name,
                            if result.passed { "PASSED" } else { "FAILED" }
                        ));
                        for log_entry in result.execution_log {
                            state.log(format!("  {}", log_entry));
                        }
                        if !result.failed_expectations.is_empty() {
                            state.log("Failed expectations:");
                            for failure in result.failed_expectations {
                                state.log(format!("  - {}", failure));
                            }
                        }
                    }
                    Err(e) => state.log(format!("DES test failed: {}", e)),
                }
            } else {
                state.log("Usage: run_des <filename>");
            }
        }
        Some("complete_quest") => {
            if let Some(quest_id) = parts.get(1) {
                complete_quest_objectives(state, quest_id);
            } else {
                state.log("Usage: complete_quest <quest_id>");
            }
        }
        Some("flee") => {
            match state.attempt_flee_encounter() {
                Ok(()) => {} // Message already logged
                Err(e) => state.log(e),
            }
        }
        Some("help") => {
            state.log("Debug Commands:");
            state.log("  show tile, hide tile - Toggle god view");
            state.log("  sturdy - Set HP to 9999");
            state.log("  phase - Toggle wall phasing");
            state.log("  flee - Attempt to flee current encounter");
            state.log("  save_debug [name] - Save debug state");
            state.log("  load_debug <name> - Load debug state");
            state.log("  list_debug - List saved debug states");
            state.log("  debug_info - Show debug information");
            state.log("  run_des <file> - Run DES test");
            state.log("  complete_quest <quest_id> - Complete all objectives for a quest");
            state.log("");
            state.log("Console Controls:");
            state.log("  ` - Toggle debug console");
            state.log("  Up/Down - Navigate command history");
            state.log("  Tab - Accept current suggestion");
            state.log("  Left/Right - Navigate suggestions");
            state.log("  Esc - Close console");
        }
        _ => state.log(format!(
            "Unknown command: {}. Type 'help' for commands.",
            cmd
        )),
    }
}

/// Debug helper to complete quest objectives
fn complete_quest_objectives(state: &mut super::state::GameState, quest_id: &str) {
    // Find and complete quest objectives
    let mut quest_found = false;
    for quest in &mut state.player.quest_log.active {
        if quest.quest_id == quest_id {
            quest_found = true;
            for objective in &mut quest.objectives {
                objective.current = objective.target;
                objective.completed = true;
            }
            break;
        }
    }

    // Log result after the borrow is released
    if quest_found {
        state.log(format!(
            "Debug: Completed quest objectives for {}",
            quest_id
        ));
    } else {
        state.log(format!("Quest not found: {}", quest_id));
    }
}
