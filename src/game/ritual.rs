use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct RitualRequirement {
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub faction_reputation: Option<HashMap<String, i32>>,
    #[serde(default)]
    pub adaptations_count: Option<usize>,
    #[serde(default)]
    pub location_type: Option<String>, // "shrine", "ruins", etc.
}

#[derive(Clone, Debug, Deserialize)]
pub struct RitualEffect {
    #[serde(default)]
    pub add_adaptation: Option<String>,
    #[serde(default)]
    pub stat_changes: Option<HashMap<String, i32>>,
    #[serde(default)]
    pub reputation_changes: Option<HashMap<String, i32>>,
    #[serde(default)]
    pub unlock_areas: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RitualDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requirements: RitualRequirement,
    pub effects: RitualEffect,
    pub confirmation_text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletedRitual {
    pub ritual_id: String,
    pub completed_at: u64, // Game turn
}

impl crate::game::state::GameState {
    /// Check if player meets ritual requirements
    pub fn can_perform_ritual(&self, ritual: &RitualDef) -> (bool, Vec<String>) {
        let mut missing = Vec::new();

        // Check item requirements
        for required_item in &ritual.requirements.items {
            if !self.inventory.contains(required_item) {
                if let Some(def) = super::item::get_item_def(required_item) {
                    missing.push(format!("Missing: {}", def.name));
                } else {
                    missing.push(format!("Missing: {}", required_item));
                }
            }
        }

        // Check faction reputation requirements
        if let Some(faction_reqs) = &ritual.requirements.faction_reputation {
            for (faction, required_rep) in faction_reqs {
                let current_rep = self.get_reputation(faction);
                if current_rep < *required_rep {
                    missing.push(format!("Need {} reputation with {} (have {})", 
                        required_rep, faction, current_rep));
                }
            }
        }

        // Check adaptation count requirement
        if let Some(required_count) = ritual.requirements.adaptations_count {
            if self.adaptations.len() < required_count {
                missing.push(format!("Need {} adaptations (have {})", 
                    required_count, self.adaptations.len()));
            }
        }

        (missing.is_empty(), missing)
    }

    /// Perform a ritual and apply its effects
    pub fn perform_ritual(&mut self, ritual_id: &str) -> Result<String, String> {
        // Load ritual definition (placeholder - would load from data file)
        let ritual = match ritual_id {
            "storm_walk" => RitualDef {
                id: "storm_walk".to_string(),
                name: "Storm Walk Ritual".to_string(),
                description: "Embrace the storm's power".to_string(),
                requirements: RitualRequirement {
                    items: vec!["storm_glass".to_string()],
                    faction_reputation: None,
                    adaptations_count: Some(1),
                    location_type: Some("shrine".to_string()),
                },
                effects: RitualEffect {
                    add_adaptation: Some("storm_touched".to_string()),
                    stat_changes: Some([("reflex".to_string(), 2)].into()),
                    reputation_changes: Some([("storm_walkers".to_string(), 10)].into()),
                    unlock_areas: vec!["storm_nexus".to_string()],
                },
                confirmation_text: "The storm's energy courses through you...".to_string(),
            },
            "crucible_transformation" => RitualDef {
                id: "crucible_transformation".to_string(),
                name: "Crucible Transformation".to_string(),
                description: "Undergo the sacred transformation".to_string(),
                requirements: RitualRequirement {
                    items: vec!["saint_key".to_string(), "scripture_shard".to_string()],
                    faction_reputation: Some([("monks".to_string(), 25)].into()),
                    adaptations_count: Some(3),
                    location_type: Some("archive".to_string()),
                },
                effects: RitualEffect {
                    add_adaptation: Some("sanctified".to_string()),
                    stat_changes: Some([("max_hp".to_string(), 10)].into()),
                    reputation_changes: Some([("monks".to_string(), 20)].into()),
                    unlock_areas: vec!["inner_sanctum".to_string()],
                },
                confirmation_text: "Sacred light fills your being...".to_string(),
            },
            _ => return Err(format!("Unknown ritual: {}", ritual_id)),
        };

        let (can_perform, missing) = self.can_perform_ritual(&ritual);
        if !can_perform {
            return Err(format!("Cannot perform ritual: {}", missing.join(", ")));
        }

        // Consume required items
        for item in &ritual.requirements.items {
            if let Some(pos) = self.inventory.iter().position(|x| x == item) {
                self.inventory.remove(pos);
            }
        }

        // Apply effects
        if let Some(adaptation_id) = &ritual.effects.add_adaptation {
            if let Some(adaptation) = super::adaptation::Adaptation::from_id(adaptation_id) {
                self.adaptations.push(adaptation);
                self.log_typed(format!("Gained adaptation: {}", adaptation_id), super::state::MsgType::System);
            }
        }

        if let Some(stat_changes) = &ritual.effects.stat_changes {
            for (stat, change) in stat_changes {
                match stat.as_str() {
                    "max_hp" => {
                        self.player_max_hp += change;
                        self.player_hp += change; // Also heal
                    },
                    "reflex" => self.player_reflex += change,
                    "armor" => self.player_armor += change,
                    _ => {}
                }
            }
        }

        if let Some(rep_changes) = &ritual.effects.reputation_changes {
            for (faction, change) in rep_changes {
                self.modify_reputation(faction, *change);
            }
        }

        // Record completion
        self.completed_rituals.push(CompletedRitual {
            ritual_id: ritual.id.clone(),
            completed_at: self.turn as u64,
        });

        self.log_typed(format!("Ritual completed: {}", ritual.name), super::state::MsgType::System);
        Ok(ritual.confirmation_text)
    }

    /// Check if ritual has been completed
    pub fn has_completed_ritual(&self, ritual_id: &str) -> bool {
        self.completed_rituals.iter().any(|r| r.ritual_id == ritual_id)
    }
}
