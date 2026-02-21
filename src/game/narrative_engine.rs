use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEngine {
    pub quest_log: QuestLog,
    pub story_model: StoryModel,
    pub tutorial_progress: crate::game::tutorial::TutorialProgress,
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
            tutorial_progress: crate::game::tutorial::TutorialProgress::new(),
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
        true
    }

    pub fn get_active(&self, quest_id: &str) -> Option<&String> {
        self.active_quests.iter().find(|q| q.as_str() == quest_id)
    }

    pub fn on_position_changed(&mut self, _x: i32, _y: i32) {}
    pub fn on_enemy_killed(&mut self, _enemy_id: &str) {}
    pub fn check_auto_complete(&mut self) -> Vec<String> { Vec::new() }
    pub fn on_item_collected(&mut self, _item_id: &str) {}
    pub fn on_npc_talked(&mut self, _npc_id: &str) -> Vec<String> { Vec::new() }
    pub fn on_interact(&mut self, _interactable_id: &str) {}
    pub fn on_examine(&mut self, _interactable_id: &str) {}
    pub fn on_aria_interfaced(&mut self, _item_id: &str) {}
    pub fn on_turn_passed(&mut self) {}
    pub fn set_faction_alignment(&mut self, _faction: &str) -> bool { false }

    pub fn complete(&mut self, quest_id: &str) -> Option<crate::game::quest::QuestReward> {
        if let Some(pos) = self.active_quests.iter().position(|q| q == quest_id) {
            let quest = self.active_quests.remove(pos);
            self.completed_quests.push(quest);
            Some(crate::game::quest::QuestReward {
                xp: 100,
                salt_scrip: 50,
                items: Vec::new(),
                unlocks_quests: Vec::new(),
                reputation_rewards: std::collections::HashMap::new(),
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
