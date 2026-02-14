/// Game events for decoupled communication between systems
#[derive(Debug, Clone)]
pub enum GameEvent {
    // --- Combat & Health ---
    PlayerDamaged {
        amount: i32,
        source: String,
    },
    PlayerHealed {
        amount: i32,
    },
    EnemyKilled {
        enemy_id: String,
        x: i32,
        y: i32,
    },

    // --- Items ---
    ItemPickedUp {
        item_id: String,
    },
    ItemUsed {
        item_id: String,
    },

    // --- Progression ---
    AdaptationGained {
        name: String,
    },
    LevelUp {
        level: u32,
    },

    // --- Environment ---
    StormArrived {
        intensity: u8,
    },

    // --- Narrative ---
    StoryHook {
        kind: String,
        x: i32,
        y: i32,
        context: std::collections::HashMap<String, String>,
    },

    // --- Cross-System Events (Phase 3 Batch 3) ---
    StatusEffectApplied {
        effect_id: String,
        duration: i32,
    },
    StatusEffectExpired {
        effect_id: String,
    },
    EnemyDamaged {
        enemy_idx: usize,
        amount: i32,
    },
    TileChanged {
        x: i32,
        y: i32,
    },
    TradeCompleted {
        npc_id: String,
    },
    FactionReputationChanged {
        faction_id: String,
        delta: i32,
    },
    CrystalResonanceChanged {
        frequency: String,
    },
    VoidExposureChanged {
        level: u32,
    },
    DialogueStarted {
        npc_id: String,
    },

    // --- Movement & Interaction (Phase 3 Batch 1+2) ---
    PlayerMoved {
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    },
    NpcTalkedTo {
        npc_id: String,
    },
    InteractableUsed {
        interactable_id: String,
    },
    InteractableExamined {
        interactable_id: String,
    },
    AriaInterfaced {
        item_id: String,
    },
    QuestCompleted {
        quest_id: String,
    },
    TurnEnded {
        turn: u32,
    },
}
