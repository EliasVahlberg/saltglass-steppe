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
    /// Interface with ARIA
    InterfaceWithAria { item_required: String },
    /// Interact with a specific object/target
    Interact { target: String },
    /// Collect data points (generic counter)
    CollectData { data_points: u32 },
    /// Wait for a duration (turns)
    Wait { duration: u32 },
    /// Examine a specific object/target
    Examine { target: String },
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
    /// Quest availability criteria
    #[serde(default)]
    pub criteria: QuestCriteria,
    /// Quest category (main, side, faction)
    #[serde(default)]
    pub category: String,
    /// Act number for main questline organization
    #[serde(default)]
    pub act: Option<u32>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestCriteria {
    /// Quest IDs that must be completed before this quest becomes available
    #[serde(default)]
    pub requires_quests_completed: Vec<String>,
    /// Minimum faction reputation requirements (faction_name -> min_reputation)
    #[serde(default)]
    pub min_faction_reputation: HashMap<String, i32>,
    /// Maximum faction reputation requirements (faction_name -> max_reputation)
    #[serde(default)]
    pub max_faction_reputation: HashMap<String, i32>,
    /// Minimum refraction level required
    #[serde(default)]
    pub min_refraction: Option<u32>,
    /// Maximum refraction level allowed
    #[serde(default)]
    pub max_refraction: Option<u32>,
    /// Required adaptations (adaptation names)
    #[serde(default)]
    pub required_adaptations: Vec<String>,
    /// Forbidden adaptations (quest unavailable if player has these)
    #[serde(default)]
    pub forbidden_adaptations: Vec<String>,
    /// Required items in inventory
    #[serde(default)]
    pub required_items: Vec<String>,
    /// Minimum player level/experience
    #[serde(default)]
    pub min_level: Option<u32>,
    /// Custom conditions that can be checked programmatically
    #[serde(default)]
    pub custom_conditions: Vec<String>,
}

#[derive(Deserialize)]
struct QuestsFile {
    quests: Vec<QuestDef>,
}

static QUEST_DEFS: Lazy<HashMap<String, QuestDef>> = Lazy::new(|| {
    let mut quests = HashMap::new();
    
    // Load regular quests
    let data = include_str!("../../data/quests.json");
    let file: QuestsFile = serde_json::from_str(data).expect("Failed to parse quests.json");
    for quest in file.quests {
        quests.insert(quest.id.clone(), quest);
    }
    
    // Load main questline
    let main_data = include_str!("../../data/main_questline.json");
    let main_file: MainQuestlineFile = serde_json::from_str(main_data).expect("Failed to parse main_questline.json");
    for quest in main_file.main_questline {
        quests.insert(quest.id.clone(), quest);
    }
    
    quests
});

static QUEST_CONTENT_INJECTIONS: Lazy<Vec<QuestContentInjection>> = Lazy::new(|| {
    let main_data = include_str!("../../data/main_questline.json");
    let main_file: MainQuestlineFile = serde_json::from_str(main_data).expect("Failed to parse main_questline.json");
    main_file.quest_content_injections
});

#[derive(Deserialize)]
struct MainQuestlineFile {
    main_questline: Vec<QuestDef>,
    quest_content_injections: Vec<QuestContentInjection>,
}

pub fn get_quest_def(id: &str) -> Option<&'static QuestDef> {
    QUEST_DEFS.get(id)
}

pub fn all_quest_ids() -> Vec<&'static str> {
    QUEST_DEFS.keys().map(|s| s.as_str()).collect()
}

/// Get quest content injections for a specific quest
pub fn get_quest_content_injections(quest_id: &str) -> Vec<&'static QuestContentInjection> {
    QUEST_CONTENT_INJECTIONS.iter()
        .filter(|injection| injection.quest_id == quest_id)
        .collect()
}

/// Get all quest content injections
pub fn get_all_quest_content_injections() -> &'static Vec<QuestContentInjection> {
    &QUEST_CONTENT_INJECTIONS
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
                ObjectiveType::CollectData { data_points, .. } => *data_points,
                ObjectiveType::Wait { duration, .. } => *duration,
                ObjectiveType::Reach { .. } | ObjectiveType::TalkTo { .. } | ObjectiveType::InterfaceWithAria { .. } 
                | ObjectiveType::Interact { .. } | ObjectiveType::Examine { .. } => 1,
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
            // Find the next uncompleted talk objective for this NPC
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::TalkTo { npc_id: target } = &obj.objective_type {
                    if target == npc_id && !self.objectives[i].completed {
                        self.objectives[i].current = 1;
                        self.objectives[i].completed = true;
                        break; // Only complete one objective per interaction
                    }
                }
            }
        }
    }

    /// Update progress for ARIA interface objectives
    pub fn on_aria_interfaced(&mut self, item_used: &str) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::InterfaceWithAria { item_required } = &obj.objective_type {
                    if item_required == item_used && !self.objectives[i].completed {
                        self.objectives[i].current = 1;
                        self.objectives[i].completed = true;
                    }
                }
            }
        }
    }

    /// Update progress for interact objectives
    pub fn on_interact(&mut self, target: &str) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::Interact { target: obj_target } = &obj.objective_type {
                    if obj_target == target && !self.objectives[i].completed {
                        self.objectives[i].current = 1;
                        self.objectives[i].completed = true;
                    }
                }
            }
        }
    }

    /// Update progress for examine objectives
    pub fn on_examine(&mut self, target: &str) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::Examine { target: obj_target } = &obj.objective_type {
                    if obj_target == target && !self.objectives[i].completed {
                        self.objectives[i].current = 1;
                        self.objectives[i].completed = true;
                    }
                }
            }
        }
    }

    /// Update progress for collect_data objectives
    pub fn on_data_collected(&mut self) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::CollectData { .. } = &obj.objective_type {
                    if !self.objectives[i].completed {
                        self.objectives[i].current += 1;
                        if self.objectives[i].current >= self.objectives[i].target {
                            self.objectives[i].completed = true;
                        }
                    }
                }
            }
        }
    }

    /// Update progress for wait objectives (call each turn)
    pub fn on_turn_passed(&mut self) {
        if let Some(def) = self.def() {
            for (i, obj) in def.objectives.iter().enumerate() {
                if let ObjectiveType::Wait { .. } = &obj.objective_type {
                    if !self.objectives[i].completed {
                        self.objectives[i].current += 1;
                        if self.objectives[i].current >= self.objectives[i].target {
                            self.objectives[i].completed = true;
                        }
                    }
                }
            }
        }
    }
}

/// Quest content that needs to be injected after procgen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestContentInjection {
    /// Quest ID this content belongs to
    pub quest_id: String,
    /// NPCs to place in the world
    #[serde(default)]
    pub npcs: Vec<QuestNpcPlacement>,
    /// Items to place in the world
    #[serde(default)]
    pub items: Vec<QuestItemPlacement>,
    /// Map modifications (doors, special tiles, etc.)
    #[serde(default)]
    pub map_modifications: Vec<QuestMapModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestNpcPlacement {
    pub npc_id: String,
    pub placement_strategy: PlacementStrategy,
    /// Only place if quest is active or available
    pub condition: PlacementCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestItemPlacement {
    pub item_id: String,
    pub placement_strategy: PlacementStrategy,
    pub condition: PlacementCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestMapModification {
    pub modification_type: String, // "door", "special_tile", "inscription"
    pub placement_strategy: PlacementStrategy,
    pub condition: PlacementCondition,
    pub data: HashMap<String, String>, // Additional data for the modification
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlacementStrategy {
    /// Place at specific world coordinates
    WorldPosition { world_x: i32, world_y: i32 },
    /// Place at specific tile coordinates
    TilePosition { x: i32, y: i32 },
    /// Place in a specific biome type
    BiomeType { biome: String },
    /// Place near a specific POI type
    NearPoi { poi_type: String, max_distance: u32 },
    /// Place in any safe location (floor tile with clearance)
    SafeLocation,
    /// Place in the current player's tile
    CurrentTile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlacementCondition {
    /// Always place
    Always,
    /// Place only if quest is active
    QuestActive,
    /// Place only if quest is available but not started
    QuestAvailable,
    /// Place only if quest is completed
    QuestCompleted,
    /// Place based on faction reputation
    FactionReputation { faction: String, min_rep: i32 },
    /// Place based on custom condition
    CustomCondition { condition: String },
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestLog {
    pub active: Vec<ActiveQuest>,
    pub completed: Vec<String>,
    /// Track major story choices and faction alignments
    #[serde(default)]
    pub story_choices: HashMap<String, String>,
    /// Track quest-related flags and state
    #[serde(default)]
    pub quest_flags: HashMap<String, bool>,
    /// Track faction alignment choices
    #[serde(default)]
    pub faction_alignment: Option<String>,
    /// Current main questline act
    #[serde(default)]
    pub current_act: u32,
}

impl QuestLog {
    /// Check if a quest's prerequisites are satisfied
    pub fn is_quest_available(&self, quest_id: &str, game_state: &super::state::GameState) -> bool {
        // Already active or completed? Not available.
        if self.active.iter().any(|q| q.quest_id == quest_id) {
            return false;
        }
        if self.completed.contains(&quest_id.to_string()) {
            return false;
        }

        // Check all criteria
        if let Some(def) = get_quest_def(quest_id) {
            self.check_quest_criteria(&def.criteria, game_state)
        } else {
            false
        }
    }
    
    /// Check if quest criteria are satisfied
    fn check_quest_criteria(&self, criteria: &QuestCriteria, game_state: &super::state::GameState) -> bool {
        // Check completed quest prerequisites
        for required in &criteria.requires_quests_completed {
            if !self.completed.contains(required) {
                return false;
            }
        }
        
        // Check faction reputation requirements
        for (faction, min_rep) in &criteria.min_faction_reputation {
            if game_state.get_reputation(faction) < *min_rep {
                return false;
            }
        }
        
        for (faction, max_rep) in &criteria.max_faction_reputation {
            if game_state.get_reputation(faction) > *max_rep {
                return false;
            }
        }
        
        // Check refraction level requirements
        if let Some(min_refraction) = criteria.min_refraction {
            if game_state.refraction < min_refraction {
                return false;
            }
        }
        
        if let Some(max_refraction) = criteria.max_refraction {
            if game_state.refraction > max_refraction {
                return false;
            }
        }
        
        // Check adaptation requirements
        let player_adaptations: Vec<String> = game_state.adaptations.iter()
            .map(|a| a.name().to_string())
            .collect();
            
        for required_adaptation in &criteria.required_adaptations {
            if !player_adaptations.contains(required_adaptation) {
                return false;
            }
        }
        
        for forbidden_adaptation in &criteria.forbidden_adaptations {
            if player_adaptations.contains(forbidden_adaptation) {
                return false;
            }
        }
        
        // Check required items
        for required_item in &criteria.required_items {
            if !game_state.inventory.contains(required_item) {
                return false;
            }
        }
        
        // Check minimum level (using player level directly)
        if let Some(min_level) = criteria.min_level {
            if game_state.player_level < min_level {
                return false;
            }
        }
        
        // Check custom conditions
        for condition in &criteria.custom_conditions {
            if !self.check_custom_condition(condition, game_state) {
                return false;
            }
        }
        
        true
    }
    
    /// Check custom quest conditions
    fn check_custom_condition(&self, condition: &str, game_state: &super::state::GameState) -> bool {
        match condition {
            "has_saint_key" => game_state.inventory.iter().any(|item| item.contains("saint_key")),
            "in_deep_archive" => game_state.layer < -2, // Deep underground
            "storm_active" => game_state.storm.turns_until > 0,
            "white_noon_discovered" => self.completed.contains(&"discover_white_noon".to_string()),
            "first_time_player" => self.completed.is_empty(), // No quests completed yet
            "has_faction_alignment" => self.faction_alignment.is_some(),
            _ => {
                // Unknown condition defaults to true to avoid breaking quests
                eprintln!("Unknown quest condition: {}", condition);
                true
            }
        }
    }

    /// Get list of all available quest IDs (prerequisites satisfied, not active/completed)
    pub fn get_available_quests(&self, game_state: &super::state::GameState) -> Vec<&'static str> {
        all_quest_ids()
            .into_iter()
            .filter(|id| self.is_quest_available(id, game_state))
            .collect()
    }

    pub fn accept(&mut self, quest_id: &str, game_state: &super::state::GameState) -> bool {
        // Use is_quest_available for comprehensive checks
        if !self.is_quest_available(quest_id, game_state) {
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
        self.check_auto_complete();
    }

    /// Notify all active quests of ARIA interface
    pub fn on_aria_interfaced(&mut self, item_used: &str) {
        for quest in &mut self.active {
            quest.on_aria_interfaced(item_used);
        }
        self.check_auto_complete();
    }

    /// Notify all active quests of position change
    pub fn on_position_changed(&mut self, x: i32, y: i32) {
        for quest in &mut self.active {
            quest.on_position_changed(x, y);
        }
        self.check_auto_complete();
    }

    /// Notify all active quests of interaction
    pub fn on_interact(&mut self, target: &str) {
        for quest in &mut self.active {
            quest.on_interact(target);
        }
        self.check_auto_complete();
    }

    /// Notify all active quests of examination
    pub fn on_examine(&mut self, target: &str) {
        for quest in &mut self.active {
            quest.on_examine(target);
        }
        self.check_auto_complete();
    }

    /// Notify all active quests of data collection
    pub fn on_data_collected(&mut self) {
        for quest in &mut self.active {
            quest.on_data_collected();
        }
        self.check_auto_complete();
    }

    /// Notify all active quests of turn passing (for wait objectives)
    pub fn on_turn_passed(&mut self) {
        for quest in &mut self.active {
            quest.on_turn_passed();
        }
        self.check_auto_complete();
    }

    /// Notify all active quests of NPC talk
    pub fn on_npc_talked(&mut self, npc_id: &str) -> Vec<String> {
        for quest in &mut self.active {
            quest.on_npc_talked(npc_id);
        }
        self.check_auto_complete()
    }
    
    /// Record a story choice
    pub fn record_story_choice(&mut self, choice_id: &str, choice_value: &str) {
        self.story_choices.insert(choice_id.to_string(), choice_value.to_string());
    }
    
    /// Get a story choice
    pub fn get_story_choice(&self, choice_id: &str) -> Option<&str> {
        self.story_choices.get(choice_id).map(|s| s.as_str())
    }
    
    /// Set a quest flag
    pub fn set_quest_flag(&mut self, flag: &str, value: bool) {
        self.quest_flags.insert(flag.to_string(), value);
    }
    
    /// Check a quest flag
    pub fn get_quest_flag(&self, flag: &str) -> bool {
        self.quest_flags.get(flag).copied().unwrap_or(false)
    }
    
    /// Set faction alignment (can only be done once per playthrough)
    pub fn set_faction_alignment(&mut self, faction: &str) -> bool {
        if self.faction_alignment.is_none() {
            self.faction_alignment = Some(faction.to_string());
            true
        } else {
            false // Already aligned
        }
    }
    
    /// Get current faction alignment
    pub fn get_faction_alignment(&self) -> Option<&str> {
        self.faction_alignment.as_deref()
    }
    
    /// Advance to next act
    pub fn advance_act(&mut self) {
        self.current_act += 1;
    }
    
    /// Get current act
    pub fn get_current_act(&self) -> u32 {
        self.current_act
    }
    
    /// Auto-complete quests that have all objectives done and unlock new quests
    pub fn check_auto_complete(&mut self) -> Vec<String> {
        let mut completed_quests = Vec::new();
        let mut unlocked_quests = Vec::new();
        
        // Find completed quests
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].is_complete() {
                let quest = self.active.remove(i);
                let quest_id = quest.quest_id.clone();
                self.completed.push(quest_id.clone());
                completed_quests.push(quest_id.clone());
                
                // Check for unlocked quests
                if let Some(def) = quest.def() {
                    for unlock_id in &def.reward.unlocks_quests {
                        if self.is_quest_available_simple(unlock_id) {
                            unlocked_quests.push(unlock_id.clone());
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
        
        // Add unlocked quests
        for quest_id in unlocked_quests {
            if let Some(quest) = ActiveQuest::new(&quest_id) {
                self.active.push(quest);
            }
        }
        
        completed_quests
    }
    
    /// Simple quest availability check (no game state needed)
    fn is_quest_available_simple(&self, quest_id: &str) -> bool {
        // Already active or completed? Not available.
        if self.active.iter().any(|q| q.quest_id == quest_id) {
            return false;
        }
        if self.completed.contains(&quest_id.to_string()) {
            return false;
        }
        true
    }
}
