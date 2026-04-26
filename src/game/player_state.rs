use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{adaptation::Adaptation, equipment::Equipment, quest::QuestLog, status::StatusEffect};

/// Tracks player activity for adaptation pool weighting.
/// Higher counts in a category increase the weight of that category's adaptations.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ActivityCounters {
    pub storms_survived: u32,
    pub glass_tiles_walked: u32,
    pub enemies_killed_melee: u32,
    pub enemies_killed_ranged: u32,
    pub elite_enemies_killed: u32,
    pub items_crafted: u32,
    pub items_used: u32,
    pub psychic_uses: u32,
    pub tiles_explored: u32,
    pub npcs_talked: u32,
    pub damage_taken_total: u32,
}

/// Identifies a single activity counter field for the IncrementActivity mutation.
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityField {
    StormsSurvived,
    GlassTilesWalked,
    EnemiesKilledMelee,
    EnemiesKilledRanged,
    EliteEnemiesKilled,
    ItemsCrafted,
    ItemsUsed,
    PsychicUses,
    TilesExplored,
    NpcsTalked,
    DamageTakenTotal(u32),
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
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
    /// Set by killing_edge adaptation: next melee attack costs 0 AP
    #[serde(default)]
    pub kill_ap_refund_active: bool,
    /// Temporary armor stacks from scar_lattice adaptation (resets each combat)
    #[serde(default)]
    pub scar_lattice_armor: i32,
    // Activity counters for adaptation weighting
    pub activity: ActivityCounters,
    // Tracks which adaptation tiers have already triggered a choice (0-indexed: tier 1=0, tier 2=1, etc.)
    pub adaptation_tiers_triggered: Vec<u8>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            name: String::new(),
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
            kill_ap_refund_active: false,
            scar_lattice_armor: 0,
            activity: ActivityCounters::default(),
            adaptation_tiers_triggered: Vec::new(),
        }
    }
}
