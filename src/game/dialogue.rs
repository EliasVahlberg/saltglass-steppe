use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

#[derive(Clone, Debug, Deserialize)]
pub struct DialogueOption {
    pub text: String,
    #[serde(default)]
    pub condition: Option<DialogueCondition>,
    #[serde(default)]
    pub response: Option<String>,
    #[serde(default)]
    pub action: Option<String>, // "trade", "quest", "reputation_change"
}

#[derive(Clone, Debug, Deserialize)]
pub struct DialogueTree {
    pub greeting: String,
    pub options: Vec<DialogueOption>,
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
