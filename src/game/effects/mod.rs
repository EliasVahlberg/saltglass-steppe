//! VERA — Verified Effect-Rule Architecture
//!
//! Pure rule functions return `Effect` enums, a mechanical `apply` mutates state,
//! and a `Trace` records what happened for verification.

pub mod apply;
pub mod context;
pub mod trace;

pub use context::QueryContext;
pub use trace::{Trace, TraceEntry, TraceSource};

// ---------------------------------------------------------------------------
// Top-level Effect enum — every state mutation goes through this
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Player(PlayerEffect),
    Combat(CombatEffect),
    Item(ItemEffect),
    Map(MapEffect),
    Resource(ResourceEffect),
    Event(EventEffect),
    Quest(QuestEffect),
}

// ---------------------------------------------------------------------------
// Domain-specific effect enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEffect {
    Heal { amount: i32 },
    TakeDamage { amount: i32 },
    SpendAp { amount: i32 },
    SetPosition { x: i32, y: i32 },
    ModifyRefraction { delta: i32 },
    SuppressAdaptations { turns: u32 },
    PlaceDecoy { x: i32, y: i32 },
    ResetWaitCounter,
    GainXp { amount: u32 },
    RecordDamageDealt { amount: u32 },
    ResetAp,
    AdvanceTurn,
    IncrementWaitCounter,
    AllocateStat { stat: String },
    GainSaltScrip { amount: u32 },
    GainSkillPoints { amount: u32 },
    LevelUp,
    ModifyReputation { faction: String, delta: i32 },
    ApplyStatusEffect { effect_id: String, duration: i32 },
    SetPhaseMode { enabled: bool },
    ClearEncounter,
    SetLastFleeAttempt { turn: u32 },
    SetWorldPosition { wx: usize, wy: usize },
    SetLayer { layer: i32 },
    IncrementTilesTraveled,
    TickPsychic,
    TickSkills,
    TickLightSystem,
    TickVoidSystem,
    TickCrystalSystem,
    TickStatusEffects,
    TickHousekeeping,
    GainAdaptation { adaptation_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatEffect {
    DealDamage { enemy_idx: usize, amount: i32 },
    Miss { enemy_idx: usize },
    Kill { enemy_idx: usize, enemy_id: String, x: i32, y: i32 },
    Provoke { enemy_idx: usize },
    StunEnemy { enemy_idx: usize, duration: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemEffect {
    Consume { item_id: String, inventory_index: usize },
    RemoveFromInventory { index: usize },
    Equip { item_id: String, slot: String },
    Unequip { slot: String },
    AddToInventory { item_id: String },
    RecalcStats,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapEffect {
    RevealAll,
    DamageWall { x: i32, y: i32, damage: i32 },
    ClearStormHighlight { tile_index: usize },
    SetWorldPath { path: Vec<(usize, usize)>, target: Option<(usize, usize)> },
    ClearWorldPath,
    AdvanceTime { new_time: u32 },
    SetWeather { weather: String },
    TickEncounterTimer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceEffect {
    GainLightEnergy { amount: u32 },
    GainVoidEnergy { amount: u32 },
    GainVoidExposure { amount: u32 },
    GainResonanceEnergy { amount: u32 },
    PlaceCrystal { x: i32, y: i32, frequency: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventEffect {
    OpenBook { book_id: String },
    /// Bridge: calls LootSystem::drop_loot for the killed enemy
    LootDrop { enemy_id: String, x: i32, y: i32 },
    /// Bridge: calls quest_log methods + check_auto_complete + logs completions
    QuestNotify { kind: QuestNotifyKind },
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestNotifyKind {
    Kill { enemy_id: String },
    Collect { item_id: String },
    Move { x: i32, y: i32 },
    NpcTalk { npc_id: String },
    Interact { target_id: String },
    Examine { target_id: String },
    AriaInterface { item_id: String },
    Turn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestEffect {
    Accept { quest_id: String },
    Complete { quest_id: String },
    SetFactionAlignment { faction: String },
}

// ---------------------------------------------------------------------------
// Presentation — visual feedback only, never traced
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Presentation {
    LogMessage { text: String, msg_type: String },
}

// ---------------------------------------------------------------------------
// RuleOutput — returned by rule functions
// ---------------------------------------------------------------------------

/// Concrete rule output for saltglass-steppe.
pub type RuleOutput = vera_effects::RuleOutput<Effect, Presentation>;

/// Result of rule_move — tells dispatch which derives to run
#[derive(Debug, Clone, PartialEq)]
pub enum MoveResult {
    /// Player moved to new position — run FOV, lighting, pickup, tile transition
    Moved,
    /// NPC at target — dispatch should call legacy NPC interaction
    Npc,
    /// Enemy at target — dispatch should call legacy combat
    Combat,
    /// Movement blocked (wall, no AP, out of bounds)
    Blocked,
}

/// Extended output for movement rules
#[derive(Debug, Clone)]
pub struct MoveOutput {
    pub result: MoveResult,
    pub effects: Vec<Effect>,
    pub presentation: Vec<Presentation>,
}

// ---------------------------------------------------------------------------
// Command — input to dispatch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Command {
    UseItem { index: usize },
    UseItemOnTile { index: usize, x: i32, y: i32 },
    Move { dx: i32, dy: i32 },
    Attack { target_x: i32, target_y: i32 },
    RangedAttack { target_x: i32, target_y: i32 },
    Wait,
    Rest,
    Equip { inv_idx: usize, slot: String },
    Unequip { slot: String },
    AllocateStat { stat: String },
    AcceptQuest { quest_id: String },
    CompleteQuest { quest_id: String },
    Interact { x: i32, y: i32 },
    Examine { x: i32, y: i32 },
    UsePsychic { ability_id: String },
    FleeEncounter,
    WorldMove { new_wx: usize, new_wy: usize },
    WorldMoveSafe { new_wx: usize, new_wy: usize },
    EnterSubterranean,
    ExitSubterranean,
    FollowWorldPath,
    CalculateWorldPath { target_wx: usize, target_wy: usize },
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Command::UseItem { .. } => "rule_use_item",
            Command::UseItemOnTile { .. } => "rule_use_item_on_tile",
            Command::Move { .. } => "rule_move",
            Command::Attack { .. } => "rule_melee_attack",
            Command::RangedAttack { .. } => "rule_ranged_attack",
            Command::Wait => "rule_wait",
            Command::Rest => "rule_rest",
            Command::Equip { .. } => "rule_equip",
            Command::Unequip { .. } => "rule_unequip",
            Command::AllocateStat { .. } => "rule_allocate_stat",
            Command::AcceptQuest { .. } => "rule_accept_quest",
            Command::CompleteQuest { .. } => "rule_complete_quest",
            Command::Interact { .. } => "rule_interact",
            Command::Examine { .. } => "rule_examine",
            Command::UsePsychic { .. } => "rule_use_psychic",
            Command::FleeEncounter => "rule_flee_encounter",
            Command::WorldMove { .. } => "dispatch_world_move",
            Command::WorldMoveSafe { .. } => "dispatch_world_move_safe",
            Command::EnterSubterranean => "dispatch_enter_subterranean",
            Command::ExitSubterranean => "dispatch_exit_subterranean",
            Command::FollowWorldPath => "dispatch_follow_world_path",
            Command::CalculateWorldPath { .. } => "dispatch_calculate_world_path",
        }
    }
}

// ---------------------------------------------------------------------------
// TurnPhase — explicit end_turn sequence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum TurnPhase {
    ResetAp,
    TickStatusEffects,
    TickSubsystems,
    AdvanceTurn,
    RunAI,
    TickStorm,
    AdvanceTime,
    UpdateDerives,
    CheckEncounters,
}

impl TurnPhase {
    pub fn sequence() -> &'static [TurnPhase] {
        &[
            TurnPhase::ResetAp,
            TurnPhase::TickStatusEffects,
            TurnPhase::TickSubsystems,
            TurnPhase::AdvanceTurn,
            TurnPhase::RunAI,
            TurnPhase::TickStorm,
            TurnPhase::AdvanceTime,
            TurnPhase::UpdateDerives,
            TurnPhase::CheckEncounters,
        ]
    }
}
