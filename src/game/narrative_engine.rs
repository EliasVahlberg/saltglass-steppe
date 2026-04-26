use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Narrative state. Only `tutorial_progress` is active — the other fields are
/// legacy stubs kept for save-file compatibility (old saves contain them).
/// Real quest logic lives in `PlayerState.quest_log` (`src/game/quest.rs`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEngine {
    #[serde(default, rename = "quest_log")]
    _quest_log: LegacyQuestLog,
    #[serde(default, rename = "story_model")]
    _story_model: LegacyStoryModel,
    #[serde(default, rename = "world_history")]
    _world_history: LegacyWorldHistory,
    #[serde(default, rename = "triggered_effects")]
    _triggered_effects: LegacyTriggeredEffects,
    pub tutorial_progress: crate::game::tutorial::TutorialProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LegacyQuestLog {
    #[serde(default)]
    active_quests: Vec<String>,
    #[serde(default)]
    completed_quests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyStoryModel {
    #[serde(default = "default_chapter")]
    current_chapter: String,
    #[serde(default)]
    story_flags: HashMap<String, bool>,
}

fn default_chapter() -> String {
    "prologue".to_string()
}

impl Default for LegacyStoryModel {
    fn default() -> Self {
        Self {
            current_chapter: default_chapter(),
            story_flags: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LegacyWorldHistory {
    #[serde(default)]
    events: Vec<String>,
    #[serde(default)]
    timeline: HashMap<u32, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LegacyTriggeredEffects {
    #[serde(default)]
    active_effects: Vec<String>,
    #[serde(default)]
    effect_timers: HashMap<String, u32>,
}

impl Default for NarrativeEngine {
    fn default() -> Self {
        Self {
            _quest_log: LegacyQuestLog::default(),
            _story_model: LegacyStoryModel::default(),
            _world_history: LegacyWorldHistory::default(),
            _triggered_effects: LegacyTriggeredEffects::default(),
            tutorial_progress: crate::game::tutorial::TutorialProgress::new(),
        }
    }
}
