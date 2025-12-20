//! Data-driven quest system with objectives and rewards

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quest objective types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObjectiveType {
    /// Kill N enemies of a specific type
    Kill { enemy_id: String, count: u32 },
    /// Collect N of a specific item
    Collect { item_id: String, count: u32 },
    /// Reach a specific location
    Reach { x: i32, y: i32 },
    /// Talk to a specific NPC
    TalkTo { npc_id: String },
}

/// A single quest objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: String,
    pub description: String,
    #[serde(flatten)]
    pub objective_type: ObjectiveType,
}

/// Quest rewards
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestReward {
    #[serde(default)]
    pub xp: u32,
    #[serde(default)]
    pub items: Vec<String>,
    /// Quest IDs to unlock upon completion
    #[serde(default)]
    pub unlocks_quests: Vec<String>,
    /// Currency reward (salt scrip)
    #[serde(default)]
    pub salt_scrip: u32,
}

/// Quest definition loaded from data file
#[derive(Debug, Clone, Deserialize)]
pub struct QuestDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objectives: Vec<Objective>,
    #[serde(default)]
    pub reward: QuestReward,
    /// Quest IDs that must be completed before this quest becomes available
    #[serde(default)]
    pub requires_quests_completed: Vec<String>,
}

#[derive(Deserialize)]
struct QuestsFile {
    quests: Vec<QuestDef>,
}

static QUEST_DEFS: Lazy<HashMap<String, QuestDef>> = Lazy::new(|| {
    let data = include_str!("../../data/quests.json");
    let file: QuestsFile = serde_json::from_str(data).expect("Failed to parse quests.json");
    file.quests.into_iter().map(|q| (q.id.clone(), q)).collect()
});

pub fn get_quest_def(id: &str) -> Option<&'static QuestDef> {
    QUEST_DEFS.get(id)
}

pub fn all_quest_ids() -> Vec<&'static str> {
    QUEST_DEFS.keys().map(|s| s.as_str()).collect()
}

/// Tracks progress on a single objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveProgress {
    pub objective_id: String,
    pub current: u32,
    pub target: u32,
    pub completed: bool,
}

/// Active quest instance with progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuest {
    pub quest_id: String,
    pub objectives: Vec<ObjectiveProgress>,
}

impl ActiveQuest {
    pub fn new(quest_id: &str) -> Option<Self> {
        let def = get_quest_def(quest_id)?;
        let objectives = def.objectives.iter().map(|obj| {
            let target = match &obj.objective_type {
                ObjectiveType::Kill { count, .. } => *count,
                ObjectiveType::Collect { count, .. } => *count,
                ObjectiveType::Reach { .. } | ObjectiveType::TalkTo { .. } => 1,
            };
            ObjectiveProgress {
                objective_id: obj.id.clone(),
                current: 0,
                target,
                completed: false,
            }
        }).collect();
        Some(Self { quest_id: quest_id.to_string(), objectives })
    }

    pub fn is_complete(&self) -> bool {
        self.objectives.iter().all(|o| o.completed)
    }

    pub fn def(&self) -> Option<&'static QuestDef> {
        get_quest_def(&self.quest_id)
    }

    /// Update progress for kill objectives
    pub fn on_enemy_killed(&mut self, enemy_id: &str) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::Kill { enemy_id: target, .. } = &obj.objective_type {
                    if target == enemy_id && !self.objectives[i].completed {
                        self.objectives[i].current += 1;
                        if self.objectives[i].current >= self.objectives[i].target {
                            self.objectives[i].completed = true;
                        }
                    }
                }
            }
        }
    }

    /// Update progress for collect objectives
    pub fn on_item_collected(&mut self, item_id: &str) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::Collect { item_id: target, .. } = &obj.objective_type {
                    if target == item_id && !self.objectives[i].completed {
                        self.objectives[i].current += 1;
                        if self.objectives[i].current >= self.objectives[i].target {
                            self.objectives[i].completed = true;
                        }
                    }
                }
            }
        }
    }

    /// Update progress for reach objectives
    pub fn on_position_changed(&mut self, x: i32, y: i32) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::Reach { x: tx, y: ty } = &obj.objective_type {
                    if x == *tx && y == *ty && !self.objectives[i].completed {
                        self.objectives[i].current = 1;
                        self.objectives[i].completed = true;
                    }
                }
            }
        }
    }

    /// Update progress for talk objectives
    pub fn on_npc_talked(&mut self, npc_id: &str) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::TalkTo { npc_id: target } = &obj.objective_type {
                    if target == npc_id && !self.objectives[i].completed {
                        self.objectives[i].current = 1;
                        self.objectives[i].completed = true;
                    }
                }
            }
        }
    }
}

/// Quest log tracking active and completed quests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestLog {
    pub active: Vec<ActiveQuest>,
    pub completed: Vec<String>,
}

impl QuestLog {
    /// Check if a quest's prerequisites are satisfied
    pub fn is_quest_available(&self, quest_id: &str) -> bool {
        // Already active or completed? Not available.
        if self.active.iter().any(|q| q.quest_id == quest_id) {
            return false;
        }
        if self.completed.contains(&quest_id.to_string()) {
            return false;
        }

        // Check prerequisites
        if let Some(def) = get_quest_def(quest_id) {
            for required in &def.requires_quests_completed {
                if !self.completed.contains(required) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Get list of all available quest IDs (prerequisites satisfied, not active/completed)
    pub fn get_available_quests(&self) -> Vec<&'static str> {
        all_quest_ids()
            .into_iter()
            .filter(|id| self.is_quest_available(id))
            .collect()
    }

    pub fn accept(&mut self, quest_id: &str) -> bool {
        // Use is_quest_available for comprehensive checks
        if !self.is_quest_available(quest_id) {
            return false;
        }
        if let Some(quest) = ActiveQuest::new(quest_id) {
            self.active.push(quest);
            true
        } else {
            false
        }
    }

    pub fn complete(&mut self, quest_id: &str) -> Option<QuestReward> {
        let idx = self.active.iter().position(|q| q.quest_id == quest_id && q.is_complete())?;
        let quest = self.active.remove(idx);
        self.completed.push(quest.quest_id.clone());
        quest.def().map(|d| d.reward.clone())
    }

    pub fn get_active(&self, quest_id: &str) -> Option<&ActiveQuest> {
        self.active.iter().find(|q| q.quest_id == quest_id)
    }

    pub fn get_active_mut(&mut self, quest_id: &str) -> Option<&mut ActiveQuest> {
        self.active.iter_mut().find(|q| q.quest_id == quest_id)
    }

    /// Notify all active quests of enemy kill
    pub fn on_enemy_killed(&mut self, enemy_id: &str) {
        for quest in &mut self.active {
            quest.on_enemy_killed(enemy_id);
        }
    }

    /// Notify all active quests of item collection
    pub fn on_item_collected(&mut self, item_id: &str) {
        for quest in &mut self.active {
            quest.on_item_collected(item_id);
        }
    }

    /// Notify all active quests of position change
    pub fn on_position_changed(&mut self, x: i32, y: i32) {
        for quest in &mut self.active {
            quest.on_position_changed(x, y);
        }
    }

    /// Notify all active quests of NPC talk
    pub fn on_npc_talked(&mut self, npc_id: &str) {
        for quest in &mut self.active {
            quest.on_npc_talked(npc_id);
        }
    }
}
