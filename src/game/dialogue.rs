use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Clone, Debug, Deserialize)]
pub struct DialogueCondition {
    #[serde(default)]
    pub has_currency: Option<u32>,
    #[serde(default)]
    pub faction_reputation: Option<HashMap<String, i32>>,
    #[serde(default)]
    pub has_item: Option<String>,
    #[serde(default)]
    pub player_level: Option<u32>,
    #[serde(default)]
    pub completed_quest: Option<String>,
    #[serde(default)]
    pub has_adaptation: Option<String>,
    #[serde(default)]
    pub area_tier: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DialogueAction {
    #[serde(rename = "type")]
    pub action_type: String, // "trade", "quest", "reputation_change", "give_item", "take_item"
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DialogueOption {
    pub text: String,
    #[serde(default)]
    pub condition: Option<DialogueCondition>,
    #[serde(default)]
    pub response: Option<String>,
    #[serde(default)]
    pub action: Option<DialogueAction>,
    #[serde(default)]
    pub leads_to: Option<String>, // Next dialogue node ID
    #[serde(default)]
    pub ends_conversation: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DialogueNode {
    pub id: String,
    pub speaker: String,
    pub text: String,
    pub options: Vec<DialogueOption>,
    #[serde(default)]
    pub condition: Option<DialogueCondition>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DialogueTree {
    pub npc_id: String,
    pub name: String,
    pub faction: String,
    pub root_node: String,
    pub nodes: Vec<DialogueNode>,
}

#[derive(Deserialize)]
struct DialoguesFile {
    dialogues: Vec<DialogueTree>,
}

static DIALOGUES: Lazy<HashMap<String, DialogueTree>> = Lazy::new(|| {
    let data = include_str!("../../data/dialogues.json");
    let file: DialoguesFile = serde_json::from_str(data).expect("Failed to parse dialogues.json");
    file.dialogues.into_iter().map(|d| (d.npc_id.clone(), d)).collect()
});

/// Current dialogue state for UI
#[derive(Clone, Debug)]
pub struct DialogueState {
    pub npc_id: String,
    pub current_node: String,
    pub speaker: String,
    pub text: String,
    pub options: Vec<DialogueOption>,
}

pub fn get_dialogue_tree(npc_id: &str) -> Option<&'static DialogueTree> {
    DIALOGUES.get(npc_id)
}

pub fn start_dialogue(npc_id: &str, game_state: &crate::game::GameState) -> Option<DialogueState> {
    let tree = get_dialogue_tree(npc_id)?;
    let root_node = tree.nodes.iter().find(|n| n.id == tree.root_node)?;
    
    // Check if root node condition is met
    if let Some(condition) = &root_node.condition {
        if !check_dialogue_condition(game_state, condition) {
            return None;
        }
    }
    
    // Filter available options based on conditions
    let available_options: Vec<DialogueOption> = root_node.options.iter()
        .filter(|opt| {
            if let Some(condition) = &opt.condition {
                check_dialogue_condition(game_state, condition)
            } else {
                true
            }
        })
        .cloned()
        .collect();
    
    Some(DialogueState {
        npc_id: npc_id.to_string(),
        current_node: root_node.id.clone(),
        speaker: root_node.speaker.clone(),
        text: root_node.text.clone(),
        options: available_options,
    })
}

pub fn continue_dialogue(
    dialogue_state: &DialogueState,
    option_index: usize,
    game_state: &mut crate::game::GameState,
) -> Option<DialogueState> {
    let tree = get_dialogue_tree(&dialogue_state.npc_id)?;
    let option = dialogue_state.options.get(option_index)?;
    
    // Execute action if present
    if let Some(action) = &option.action {
        execute_dialogue_action(action, game_state);
    }
    
    // Check if conversation ends
    if option.ends_conversation {
        return None;
    }
    
    // Move to next node if specified
    if let Some(next_node_id) = &option.leads_to {
        let next_node = tree.nodes.iter().find(|n| n.id == *next_node_id)?;
        
        // Check node condition
        if let Some(condition) = &next_node.condition {
            if !check_dialogue_condition(game_state, condition) {
                return None;
            }
        }
        
        // Filter available options
        let available_options: Vec<DialogueOption> = next_node.options.iter()
            .filter(|opt| {
                if let Some(condition) = &opt.condition {
                    check_dialogue_condition(game_state, condition)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        Some(DialogueState {
            npc_id: dialogue_state.npc_id.clone(),
            current_node: next_node.id.clone(),
            speaker: next_node.speaker.clone(),
            text: next_node.text.clone(),
            options: available_options,
        })
    } else {
        None
    }
}

fn check_dialogue_condition(game_state: &crate::game::GameState, condition: &DialogueCondition) -> bool {
    // Check currency requirement
    if let Some(required_currency) = condition.has_currency {
        if game_state.salt_scrip < required_currency {
            return false;
        }
    }

    // Check faction reputation requirements
    if let Some(faction_reqs) = &condition.faction_reputation {
        for (faction, required_rep) in faction_reqs {
            if game_state.get_reputation(faction) < *required_rep {
                return false;
            }
        }
    }

    // Check item requirement
    if let Some(required_item) = &condition.has_item {
        if !game_state.inventory.contains(required_item) {
            return false;
        }
    }

    // Check level requirement
    if let Some(required_level) = condition.player_level {
        if game_state.player_level < required_level {
            return false;
        }
    }

    // Check completed quest
    if let Some(quest_id) = &condition.completed_quest {
        if !game_state.quest_log.completed.contains(quest_id) {
            return false;
        }
    }

    // Check adaptation
    if let Some(adaptation_id) = &condition.has_adaptation {
        if !game_state.adaptations.iter().any(|a| a.id == *adaptation_id) {
            return false;
        }
    }

    // Check area tier
    if let Some(required_tier) = condition.area_tier {
        let current_tier = crate::game::trading::calculate_area_tier(&game_state.enemies);
        if current_tier < required_tier {
            return false;
        }
    }

    true
}

fn execute_dialogue_action(action: &DialogueAction, game_state: &mut crate::game::GameState) {
    match action.action_type.as_str() {
        "trade" => {
            // Set pending trade interface
            if let Some(trader_id) = action.parameters.get("trader_id") {
                if let Some(trader_id_str) = trader_id.as_str() {
                    game_state.pending_trade = Some(trader_id_str.to_string());
                }
            }
        }
        "reputation_change" => {
            if let (Some(faction), Some(change)) = (
                action.parameters.get("faction").and_then(|v| v.as_str()),
                action.parameters.get("change").and_then(|v| v.as_i64())
            ) {
                let current = game_state.faction_reputation.get(faction).unwrap_or(&0);
                game_state.faction_reputation.insert(faction.to_string(), current + change as i32);
            }
        }
        "give_item" => {
            if let Some(item_id) = action.parameters.get("item_id").and_then(|v| v.as_str()) {
                game_state.inventory.push(item_id.to_string());
            }
        }
        "take_item" => {
            if let Some(item_id) = action.parameters.get("item_id").and_then(|v| v.as_str()) {
                if let Some(pos) = game_state.inventory.iter().position(|id| id == item_id) {
                    game_state.inventory.remove(pos);
                }
            }
        }
        "give_currency" => {
            if let Some(amount) = action.parameters.get("amount").and_then(|v| v.as_u64()) {
                game_state.salt_scrip += amount as u32;
            }
        }
        "take_currency" => {
            if let Some(amount) = action.parameters.get("amount").and_then(|v| v.as_u64()) {
                game_state.salt_scrip = game_state.salt_scrip.saturating_sub(amount as u32);
            }
        }
        _ => {} // Unknown action type
    }
}

impl crate::game::state::GameState {
    /// Check if dialogue condition is met
    pub fn check_dialogue_condition(&self, condition: &DialogueCondition) -> bool {
        // Check currency requirement
        if let Some(required_currency) = condition.has_currency {
            if self.salt_scrip < required_currency {
                return false;
            }
        }

        // Check faction reputation requirements
        if let Some(faction_reqs) = &condition.faction_reputation {
            for (faction, required_rep) in faction_reqs {
                if self.get_reputation(faction) < *required_rep {
                    return false;
                }
            }
        }

        // Check item requirement
        if let Some(required_item) = &condition.has_item {
            if !self.inventory.contains(required_item) {
                return false;
            }
        }

        // Check player level requirement
        if let Some(required_level) = condition.player_level {
            if self.player_level < required_level {
                return false;
            }
        }

        true
    }

    /// Get available dialogue options for an NPC
    pub fn get_available_dialogue_options(&self, dialogue: &DialogueTree) -> Vec<&DialogueOption> {
        dialogue.options.iter()
            .filter(|option| {
                if let Some(condition) = &option.condition {
                    self.check_dialogue_condition(condition)
                } else {
                    true
                }
            })
            .collect()
    }
}
