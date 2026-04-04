---
name: codebase-state-structure
description: Structure of GameState and its sub-structs (PlayerState, WorldState, NarrativeEngine). Use when navigating the data model, adding new fields, or understanding what data lives where.
---

# Codebase: State Structure

> `state.rs` contains the central GameState and has been decomposed into sub-structs for better organization.

## GameState (`src/game/state.rs`)

Single source of truth. Serialized with RON via `serde`.

```rust
pub struct GameState {
    // Sub-structs (decomposed for organization)
    pub player: PlayerState,       // player_state.rs
    pub world: WorldState,         // world_state.rs
    pub narrative: NarrativeEngine, // narrative_engine.rs

    // FOV
    pub visible: HashSet<usize>,   // currently visible tile indices
    pub revealed: HashSet<usize>,  // all ever-seen tile indices

    // Lighting (recomputed, not saved)
    #[serde(skip)] pub light_map: LightMap,

    // Messages (capped at 5)
    pub messages: Vec<GameMessage>,
    pub turn: u32,

    // Seeded RNG (custom serde via rng_serde module)
    pub rng: ChaCha8Rng,

    // Active effects
    pub triggered_effects: Vec<TriggeredEffect>,
    pub decoys: Vec<Decoy>,        // mirage_step adaptation decoys

    // Spatial index (not saved — rebuilt on load)
    #[serde(skip)] pub enemy_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)] pub npc_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)] pub item_positions: HashMap<(i32, i32), Vec<usize>>,
    #[serde(skip)] pub chest_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)] pub interactable_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)] spatial_dirty: bool,

    // Event bus (not saved)
    #[serde(skip)] pub event_queue: Vec<GameEvent>,

    // DES mocks (not saved)
    #[serde(skip)] pub mock_combat_hit: Option<bool>,
    #[serde(skip)] pub mock_combat_damage: Option<i32>,

    // Pending UI actions (not saved)
    #[serde(skip)] pub pending_trade: Option<String>,
    #[serde(skip)] pub pending_dialogue: Option<(String, String)>,
    #[serde(skip)] pub pending_aria_dialogue: Option<(String, Vec<String>)>,
    #[serde(skip)] pub pending_book_open: Option<String>,

    // Meta (not saved per-game)
    #[serde(skip)] pub meta: MetaProgress,

    pub wait_counter: u32,
    pub seed: u64,

    // Debug flags (not saved)
    #[serde(skip)] pub debug_god_view: bool,
    #[serde(skip)] pub debug_phase: bool,
    #[serde(skip)] pub debug_disable_glare: bool,
}
```

---

## PlayerState (`src/game/player_state.rs`)

```rust
pub struct PlayerState {
    // Position
    pub x: i32, pub y: i32, pub layer: u8,

    // Core stats
    pub hp: i32, pub max_hp: i32,
    pub ap: i32, pub max_ap: i32,
    pub reflex: i32, pub armor: i32,

    // Progression
    pub xp: u32, pub level: u32,
    pub pending_stat_points: u32,
    pub salt_scrip: i32,

    // Inventory
    pub inventory: Vec<String>,       // item IDs
    pub equipped_weapon: Option<String>,
    pub equipment: Equipment,         // all equipment slots

    // Adaptations
    pub refraction: i32,
    pub adaptations: Vec<Adaptation>,
    pub adaptations_hidden_turns: u32,
    pub status_effects: Vec<StatusEffect>,

    // Faction
    pub faction_reputation: HashMap<String, i32>,

    // Quest system (canonical quest log)
    pub quest_log: QuestLog,

    // Specialized systems
    pub psychic: PsychicState,
    pub skills: SkillState,
    pub light_system: LightSystem,
    pub void_system: VoidSystem,
    pub crystal_system: CrystalSystem,

    pub last_damage_dealt: i32,
}
```

---

## WorldState (`src/game/world_state.rs`)

```rust
pub struct WorldState {
    // World navigation
    pub world_map: WorldMap,
    pub world_x: usize, pub world_y: usize,
    pub layer: u8,

    // Current tile
    pub map: Map,
    pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub chests: Vec<Chest>,
    pub interactables: Vec<Interactable>,
    pub microstructures: Vec<Microstructure>,

    // Environmental
    pub storm: Storm,
    pub time_of_day: TimeOfDay,
    pub weather: Weather,
    pub ambient_light: u8,

    // Visual effects
    pub visual_effects: Vec<VisualEffect>,
    pub light_map: LightMap,

    // Encounters
    pub encounter_state: EncounterState,
    pub encounter_history: Vec<EncounterRecord>,
    pub total_tiles_traveled: u32,

    // Pathfinding
    pub world_map_target: Option<(usize, usize)>,
    pub world_map_path: Vec<(usize, usize)>,

    // Spatial indices (computed on load)
    pub enemy_positions: HashMap<(i32, i32), usize>,
    pub npc_positions: HashMap<(i32, i32), usize>,
    pub item_positions: HashMap<(i32, i32), Vec<usize>>,
    pub chest_positions: HashMap<(i32, i32), usize>,
    pub interactable_positions: HashMap<(i32, i32), usize>,
}
```

---

## NarrativeEngine (`src/game/narrative_engine.rs`)

```rust
pub struct NarrativeEngine {
    pub quest_log: QuestLog,
    pub story_model: StoryModel,
    pub tutorial_progress: TutorialProgress,
    pub world_history: WorldHistory,
    pub triggered_effects: TriggeredEffects,
}
```

---

## Key Patterns

**Spatial index** — lazy rebuild via dirty flag:
```rust
state.ensure_spatial_index();  // rebuilds if spatial_dirty == true
state.mark_spatial_dirty();    // call after moving entities
state.rebuild_spatial_index(); // force rebuild
```

**Delegation methods** — `GameState` exposes convenience accessors:
```rust
state.player_x(), state.player_y(), state.player_hp()
state.map(), state.enemies(), state.npcs()
state.quest_log(), state.story_model()
```

**Messages** — capped at 5, oldest removed:
```rust
state.log("text");                        // MsgType::System
state.log_typed("text", MsgType::Combat); // typed
```

**Event emission**:
```rust
state.emit(GameEvent::EnemyKilled { ... }); // push to event_queue
// processed in end_turn() → process_events()
```

---

## Adding New Fields

1. Add to the appropriate sub-struct (`PlayerState`, `WorldState`, or `GameState` directly)
2. If it should not be saved: add `#[serde(skip)]`
3. If it needs a default: add `#[serde(default)]`
4. If it's a spatial index: add to `rebuild_spatial_index_internal()`
5. Bump `SAVE_VERSION` in `src/game/save.rs` if the layout change breaks existing saves
