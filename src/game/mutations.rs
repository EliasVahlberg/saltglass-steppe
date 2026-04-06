use crate::game::{
    encounter::EncounterState,
    map::Tile,
    state::MsgType,
    world_state::Weather,
};

// ---------------------------------------------------------------------------
// SubsystemId — identifies bridge subsystems for TickSubsystem mutations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum SubsystemId {
    Psychic,
    Skills,
    Light,
    Void,
    Crystal,
    Status,
    AI,
    Storm,
    Housekeeping,
}

// ---------------------------------------------------------------------------
// Mutation — every state change goes through one of these
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Mutation {
    // Player vitals
    SetPlayerHp(i32),
    SetPlayerMaxHp(i32),
    SetPlayerAp(i32),
    SetPlayerMaxAp(i32),
    SetPlayerPosition { x: i32, y: i32 },
    SetPlayerReflex(i32),
    SetPlayerArmor(i32),

    // Player progression
    SetPlayerXp(u32),
    SetPlayerLevel(u32),
    SetPlayerStatPoints(i32),
    SetPlayerSkillPoints(u32),
    SetPlayerSaltScrip(u32),

    // Player state
    SetPlayerRefraction(u32),
    SetWaitCounter(u32),
    SetAdaptationsHidden(u32),
    AddAdaptation(String),
    AddStatusEffect { id: String, duration: i32 },
    SetLastDamageDealt(u32),
    AllocateStat(String),
    SuppressAdaptations { turns: u32 },
    SetPhaseMode(bool),
    /// Equipment
    Equip { slot: String, item_id: String },
    Unequip(String),
    RecalcStats,
    /// Combat
    StunEnemy { idx: usize, duration: i32 },
    DamageWall { x: i32, y: i32, damage: i32 },

    // Inventory & equipment
    AddToInventory(String),
    RemoveFromInventory(usize),
    SetEquipment { slot: String, item_id: Option<String> },
    SpawnItemOnMap { item_id: String, x: i32, y: i32 },

    // Enemies
    SetEnemyHp { idx: usize, hp: i32 },
    SetEnemyProvoked { idx: usize, provoked: bool },
    AddEnemyStatus { idx: usize, id: String, duration: i32 },
    RemoveEnemy { idx: usize, x: i32, y: i32 },
    SpawnEnemy { id: String, x: i32, y: i32 },

    // World state
    SetWorldPosition { wx: usize, wy: usize },
    SetLayer(i32),
    SetTimeOfDay(u8),
    SetWeather(Weather),
    IncrementTilesTraveled,
    AdvanceTurn,

    // Map
    SetTile { idx: usize, tile: Tile },
    RevealTile(usize),
    RevealAll,
    ClearStormHighlight(usize),
    SetWorldPath { path: Vec<(usize, usize)>, target: Option<(usize, usize)> },
    ClearWorldPath,

    // Encounter
    SetEncounterState(Option<Box<EncounterState>>),
    IncrementEncounterTimer,
    SetLastFleeAttempt(u32),

    // Faction & quest
    SetReputation { faction: String, value: i32 },
    AcceptQuest(String),
    CompleteQuest(String),
    SetFactionAlignment(String),
    /// Bridge: calls quest_log notification methods
    QuestNotify(crate::game::effects::QuestNotifyKind),

    // Resources
    SetLightEnergy(u32),
    AddVoidEnergy(u32),
    AddVoidExposure(u32),
    SetResonanceEnergy(u32),
    PlaceCrystal { x: i32, y: i32, frequency: String },

    // Presentation (no verification, no transitions)
    LogMessage { text: String, msg_type: MsgType },
    OpenBook(String),
    PlaceDecoy { x: i32, y: i32 },
    HitFlash { x: i32, y: i32 },
    DamageNumber { x: i32, y: i32, value: i32, is_heal: bool },
    SpawnProjectile { from: (i32, i32), to: (i32, i32), ch: char },
    TriggerEffect { effect: String, duration: u32 },

    // Subsystem ticks (bridge — subsystem handles internally)
    TickSubsystem(SubsystemId),
    ResetAp,
    TickStatusEffects,
    TickHousekeeping,
    RunAI,
    TickStorm,
    AdvanceTime { new_time: u32 },
    /// Bridge: calls psychic.use_ability then applies the resulting effect
    UsePsychicAbility { ability_id: String },
    /// Delta mutations (used when system doesn't have current value)
    AddSaltScrip(u32),
    SpendAp(i32),
    AddHp(i32),
    AddRefraction(i32),
    IncrementWaitCounter,
    /// World travel bridges (orchestrators — call travel_to_tile internally)
    WorldMove { wx: usize, wy: usize },
    WorldMoveSafe { wx: usize, wy: usize },
    FollowWorldPath,
    CalculateWorldPath { target: (usize, usize) },
    EnterSubterranean,
    ExitSubterranean,
    /// Bridge: runs rule_move and handles all branches (NPC, combat, move, blocked)
    MovePlayer { dx: i32, dy: i32 },
    /// Bridge: calls end_turn() (runs all turn phases)
    EndTurn,
    /// Bridge: calls update_enemies() + tick_turn_housekeeping() x10 (for Rest)
    RestTick,
}

// ---------------------------------------------------------------------------
// StateTransition — detected by apply_one, reported to notify layer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum StateTransition {
    PlayerPositionChanged { old_x: i32, old_y: i32, new_x: i32, new_y: i32 },
    PlayerApReachedZero,
    PlayerDied,
    EnemyHpChanged { idx: usize, old_hp: i32, new_hp: i32 },
    EnemyHpReachedZero { idx: usize, enemy_id: String, x: i32, y: i32 },
    TurnAdvanced { old_turn: u32, new_turn: u32 },
    ItemAddedToInventory { item_id: String },
    PlayerEnteredWorldTile { wx: usize, wy: usize },
}
