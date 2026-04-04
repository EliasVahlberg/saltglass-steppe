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
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatEffect {
    DealDamage { enemy_idx: usize, amount: i32 },
    Miss { enemy_idx: usize },
    Kill { enemy_idx: usize, enemy_id: String, x: i32, y: i32 },
    Provoke { enemy_idx: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemEffect {
    Consume { item_id: String, inventory_index: usize },
    RemoveFromInventory { index: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapEffect {
    RevealAll,
    DamageWall { x: i32, y: i32, damage: i32 },
    ClearStormHighlight { tile_index: usize },
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
    EmitGameEvent { event_name: String },
}

// ---------------------------------------------------------------------------
// Presentation — visual feedback only, never traced
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Presentation {
    LogMessage { text: String, msg_type: String },
}

// ---------------------------------------------------------------------------
// RuleOutput — what a rule function returns
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct RuleOutput {
    pub effects: Vec<Effect>,
    pub presentation: Vec<Presentation>,
}

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
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Command::UseItem { .. } => "rule_use_item",
            Command::UseItemOnTile { .. } => "rule_use_item_on_tile",
            Command::Move { .. } => "rule_move",
            Command::Attack { .. } => "rule_melee_attack",
            Command::RangedAttack { .. } => "rule_ranged_attack",
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
    ProcessEvents,
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
            TurnPhase::ProcessEvents,
        ]
    }
}
