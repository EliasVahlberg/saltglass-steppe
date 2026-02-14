use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEngine {
    // Core systems (serialized)
    pub quest_log: QuestLog,
    pub story_model: StoryModel,
    pub tutorial_progress: TutorialProgress,
    
    // Generation systems (not serialized - recreated on load)
    #[serde(skip)]
    pub narrative_generator: NarrativeGenerator,
    #[serde(skip)]
    pub event_system: EventSystem,
    #[serde(skip)]
    pub narrative_integration: NarrativeIntegration,
    #[serde(skip)]
    pub grammar_system: GrammarSystem,
    #[serde(skip)]
    pub template_library: TemplateLibrary,
    #[serde(skip)]
    pub biome_system: BiomeSystem,
    #[serde(skip)]
    pub constraint_system: ConstraintSystem,
    
    // State tracking (serialized)
    pub world_history: WorldHistory,
    pub triggered_effects: TriggeredEffects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestLog {
    pub active_quests: Vec<String>,
    pub completed_quests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryModel {
    pub current_chapter: String,
    pub story_flags: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialProgress {
    pub completed_steps: Vec<String>,
    pub current_step: Option<String>,
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct NarrativeGenerator;

#[derive(Debug, Clone, Default)]
pub struct EventSystem;

#[derive(Debug, Clone, Default)]
pub struct NarrativeIntegration;

#[derive(Debug, Clone, Default)]
pub struct GrammarSystem;

#[derive(Debug, Clone, Default)]
pub struct TemplateLibrary;

#[derive(Debug, Clone, Default)]
pub struct BiomeSystem;

#[derive(Debug, Clone, Default)]
pub struct ConstraintSystem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldHistory {
    pub events: Vec<String>,
    pub timeline: HashMap<u32, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredEffects {
    pub active_effects: Vec<String>,
    pub effect_timers: HashMap<String, u32>,
}

impl Default for NarrativeEngine {
    fn default() -> Self {
        Self {
            quest_log: QuestLog::default(),
            story_model: StoryModel::default(),
            tutorial_progress: TutorialProgress::default(),
            narrative_generator: NarrativeGenerator,
            event_system: EventSystem,
            narrative_integration: NarrativeIntegration,
            grammar_system: GrammarSystem,
            template_library: TemplateLibrary,
            biome_system: BiomeSystem,
            constraint_system: ConstraintSystem,
            world_history: WorldHistory::default(),
            triggered_effects: TriggeredEffects::default(),
        }
    }
}

impl Default for QuestLog {
    fn default() -> Self {
        Self {
            active_quests: Vec::new(),
            completed_quests: Vec::new(),
        }
    }
}

impl QuestLog {
    pub fn is_quest_available(&self, _quest_id: &str, _state: &crate::game::state::GameState) -> bool {
        // Placeholder implementation
        true
    }

    pub fn get_active(&self, quest_id: &str) -> Option<&String> {
        self.active_quests.iter().find(|q| q.as_str() == quest_id)
    }

    pub fn on_position_changed(&mut self, _x: i32, _y: i32) {
        // Placeholder implementation
    }

    pub fn on_enemy_killed(&mut self, _enemy_id: &str) {
        // Placeholder implementation
    }

    pub fn check_auto_complete(&mut self) -> Vec<String> {
        // Placeholder implementation
        Vec::new()
    }

    pub fn on_item_collected(&mut self, _item_id: &str) {
        // Placeholder implementation
    }

    pub fn on_npc_talked(&mut self, _npc_id: &str) -> Vec<String> {
        // Placeholder implementation
        Vec::new()
    }

    pub fn on_interact(&mut self, _interactable_id: &str) {
        // Placeholder implementation
    }

    pub fn on_examine(&mut self, _interactable_id: &str) {
        // Placeholder implementation
    }

    pub fn on_aria_interfaced(&mut self, _item_id: &str) {
        // Placeholder implementation
    }

    pub fn on_turn_passed(&mut self) {
        // Placeholder implementation
    }

    pub fn set_faction_alignment(&mut self, _faction: &str) -> bool {
        // Placeholder implementation
        false
    }

    pub fn complete(&mut self, quest_id: &str) -> Option<crate::game::quest::QuestReward> {
        // Move quest from active to completed
        if let Some(pos) = self.active_quests.iter().position(|q| q == quest_id) {
            let quest = self.active_quests.remove(pos);
            self.completed_quests.push(quest);
            // Return placeholder reward
            Some(crate::game::quest::QuestReward {
                xp: 100,
                salt_scrip: 50,
                items: Vec::new(),
                unlocks_quests: Vec::new(),
            })
        } else {
            None
        }
    }
}

impl Default for StoryModel {
    fn default() -> Self {
        Self {
            current_chapter: "prologue".to_string(),
            story_flags: HashMap::new(),
        }
    }
}

impl Default for TutorialProgress {
    fn default() -> Self {
        Self {
            completed_steps: Vec::new(),
            current_step: None,
        }
    }
}

impl TutorialProgress {
    pub fn get_next_message(&self, _state: &crate::game::state::GameState) -> Option<String> {
        // Placeholder implementation
        None
    }

    pub fn mark_shown(&mut self, _message_id: &str) {
        // Placeholder implementation
    }
}

impl Default for WorldHistory {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            timeline: HashMap::new(),
        }
    }
}

impl Default for TriggeredEffects {
    fn default() -> Self {
        Self {
            active_effects: Vec::new(),
            effect_timers: HashMap::new(),
        }
    }
}