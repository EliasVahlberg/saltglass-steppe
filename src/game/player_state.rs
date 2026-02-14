use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    adaptation::Adaptation,
    equipment::Equipment,
    quest::QuestLog,
    status::StatusEffect,
};

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    // Position
    pub x: i32,
    pub y: i32,
    pub layer: i32,

    // Core stats
    pub hp: i32,
    pub max_hp: i32,
    pub ap: i32,
    pub max_ap: i32,
    pub reflex: i32,
    pub armor: i32,

    // Progression
    pub xp: u32,
    pub level: u32,
    pub pending_stat_points: i32,
    pub salt_scrip: u32,

    // Inventory and equipment
    pub inventory: Vec<String>,
    pub equipped_weapon: Option<String>,
    pub equipment: Equipment,

    // Refraction and adaptations
    pub refraction: u32,
    pub adaptations: Vec<Adaptation>,
    pub adaptations_hidden_turns: u32,
    pub status_effects: Vec<StatusEffect>,

    // Faction relationships
    pub faction_reputation: HashMap<String, i32>,

    // Quest system
    pub quest_log: QuestLog,

    // Specialized systems
    pub psychic: super::psychic::PsychicState,
    pub skills: super::skills::SkillsState,
    pub light_system: super::light::LightSystem,
    pub void_system: super::void_energy::VoidSystem,
    pub crystal_system: super::crystal_resonance::CrystalSystem,

    // Combat tracking
    pub last_damage_dealt: u32,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            layer: 0,
            hp: 100,
            max_hp: 100,
            ap: super::action::default_player_ap(),
            max_ap: super::action::default_player_ap(),
            reflex: 0,
            armor: 0,
            xp: 0,
            level: 1,
            pending_stat_points: 0,
            salt_scrip: 0,
            inventory: Vec::new(),
            equipped_weapon: None,
            equipment: Equipment::default(),
            refraction: 0,
            adaptations: Vec::new(),
            adaptations_hidden_turns: 0,
            status_effects: Vec::new(),
            faction_reputation: HashMap::new(),
            quest_log: QuestLog::default(),
            psychic: super::psychic::PsychicState::default(),
            skills: super::skills::SkillsState::default(),
            light_system: super::light::LightSystem::default(),
            void_system: super::void_energy::VoidSystem::default(),
            crystal_system: super::crystal_resonance::CrystalSystem::default(),
            last_damage_dealt: 0,
        }
    }
}