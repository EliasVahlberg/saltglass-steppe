# Game Systems Overview

> **Purpose**: A comprehensive guide for developers working on Saltglass Steppe. Start here to understand how systems interact and where to add new content.

## Quick Reference

| I Want To...                     | Look At                                                    |
| -------------------------------- | ---------------------------------------------------------- |
| Add a new item                   | `data/items.json`, see [Data Files](#data-files)           |
| Add a new enemy                  | `data/enemies.json`, see [Enemy System](#enemy-system)     |
| Add a new quest                  | `data/quests.json`, see [Quest System](#quest-system)      |
| Create a test scenario           | `tests/scenarios/`, see [DES Testing](#des-testing-system) |
| Add a new AI behavior            | `src/game/systems/ai.rs`, see [AI System](#ai-system)      |
| Add a new game mechanic          | `src/game/systems/`, see [Systems Layer](#systems-layer)   |
| Understand the game loop         | `src/main.rs`, see [Game Loop](#game-loop)                 |
| Add visual effects               | `data/effects_config.json`, see [Effects DSL](#visual-effects-dsl) |

---

## Architecture Philosophy

### Core Principles

1. **Determinism First**: All game logic uses seeded RNG (`ChaCha8Rng`) for 100% reproducibility. This enables automated testing via the Debug Execution System (DES).

2. **Data-Driven Design**: Game content lives in JSON files under `data/`. Adding items, enemies, quests, etc. requires *no code changes* unless you're adding new mechanics.

3. **Strict Layer Separation**:
   - `src/game/` — Pure game logic. **Never** imports rendering libraries.
   - `src/renderer/` — All rendering via `ratatui`. Reads `GameState`, never modifies game logic.
   - `src/des/` — Headless test framework. Drives `GameState` without rendering.

4. **Event-Driven Communication**: Systems communicate through a `GameEvent` bus rather than direct method calls, enabling loose coupling.

### Directory Structure

```
src/
├── main.rs              # Entry point, game loop, input handling
├── lib.rs               # Library exports
├── game/                # Pure game logic (THE CORE)
│   ├── mod.rs           # Module re-exports
│   ├── state.rs         # GameState - the single source of truth
│   ├── systems/         # Extracted logic systems
│   │   ├── ai.rs        # Enemy AI and behavior registry
│   │   ├── combat.rs    # Attack resolution, damage, death
│   │   ├── movement.rs  # Player movement, tile effects
│   │   ├── loot.rs      # Event-driven loot drops
│   │   ├── quest.rs     # Event-driven quest progress
│   │   ├── status.rs    # Status effect ticking
│   │   └── storm.rs     # Glass storm map transformations
│   ├── enemy.rs         # Enemy definitions & data loading
│   ├── item.rs          # Item definitions & data loading
│   ├── npc.rs           # NPC definitions & data loading
│   ├── quest.rs         # Quest definitions & data loading
│   └── ...              # Other game modules
├── renderer/            # TUI rendering (ratatui)
├── des/                 # Debug Execution System
└── ui/                  # UI state and input handling
data/                    # JSON content definitions
tests/
└── scenarios/           # DES test scenarios (JSON)
```

---

## The GameState Model

`GameState` (`src/game/state.rs`) is the **single source of truth** for all game data. It is:

- **Serializable**: Supports save/load via `serde`
- **Deterministic**: Uses `ChaCha8Rng` for all randomness
- **Central Hub**: All systems read from and write to `GameState`

### Key Fields

```rust
pub struct GameState {
    // Player state
    pub player_x: i32, pub player_y: i32,
    pub player_hp: i32, pub player_max_hp: i32,
    pub inventory: Vec<Item>,
    pub equipment: Equipment,
    pub adaptations: Vec<Adaptation>,
    
    // World state
    pub map: Map,
    pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,  // Ground items
    pub storm: Storm,
    
    // Progression
    pub quest_log: QuestLog,
    pub turn: u32,
    pub xp: u32, pub level: u32,
    
    // Events & Messages
    pub events: Vec<GameEvent>,
    pub messages: Vec<GameMessage>,
    
    // Seeded RNG (critical for determinism)
    #[serde(with = "rng_serde")]
    pub rng: ChaCha8Rng,
}
```

### Pattern: Accessing GameState

Systems receive `&mut GameState` and operate on it:

```rust
// In src/game/systems/combat.rs
impl CombatSystem {
    pub fn attack_melee(state: &mut GameState, target_idx: usize) {
        // Read from state
        let damage = state.calc_player_damage();
        // Modify state
        state.enemies[target_idx].hp -= damage;
        // Emit event
        state.events.push(GameEvent::EnemyKilled { ... });
    }
}
```

---

## Systems Layer

Systems are stateless modules that operate on `GameState`. They follow the **Data-Oriented Design** pattern: data lives in `GameState`, logic lives in systems.

### System Trait

```rust
// src/game/systems/mod.rs
pub trait System {
    fn update(&self, state: &mut GameState);
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
```

### Active Systems

| System               | File                          | Responsibility                              |
| -------------------- | ----------------------------- | ------------------------------------------- |
| `CombatSystem`       | `systems/combat.rs`           | Attack resolution, damage, death processing |
| `AiSystem`           | `systems/ai.rs`               | Enemy decision-making, behavior dispatch    |
| `MovementSystem`     | `systems/movement.rs`         | Player movement, tile effects, NPC triggers |
| `LootSystem`         | `systems/loot.rs`             | Listens to `EnemyKilled`, drops loot        |
| `QuestSystem`        | `systems/quest.rs`            | Listens to events, updates quest progress   |
| `StatusEffectSystem` | `systems/status.rs`           | Ticks status effects each turn              |
| `StormSystem`        | `systems/storm.rs`            | Storm progression, map transformations      |

### Adding a New System

1. Create `src/game/systems/my_system.rs`
2. Implement the `System` trait
3. Add `pub mod my_system;` to `src/game/systems/mod.rs`
4. Call from `GameState::end_turn()` or relevant trigger point

---

## Event Bus

Systems communicate through events to avoid tight coupling.

### GameEvent Enum

```rust
// src/game/event.rs
pub enum GameEvent {
    PlayerDamaged { amount: i32, source: String },
    PlayerHealed { amount: i32 },
    EnemyKilled { enemy_id: String, x: i32, y: i32 },
    ItemPickedUp { item_id: String },
    ItemUsed { item_id: String },
    AdaptationGained { name: String },
    StormArrived { intensity: u8 },
    LevelUp { level: u32 },
}
```

### Event Flow

```
1. System emits event → state.events.push(GameEvent::EnemyKilled {...})
2. end_turn() calls → process_events()
3. Each system's on_event() is called → LootSystem drops loot, QuestSystem updates progress
```

### Pattern: Event-Driven Side Effects

Instead of `CombatSystem` directly calling loot logic:

```rust
// BAD: Tight coupling
fn kill_enemy(state: &mut GameState, idx: usize) {
    drop_loot(state, idx);       // Direct call
    update_quests(state, idx);   // Direct call
}

// GOOD: Event-driven
fn kill_enemy(state: &mut GameState, idx: usize) {
    state.events.push(GameEvent::EnemyKilled { ... });
    // LootSystem and QuestSystem will react in on_event()
}
```

---

## Data Loading Pattern

All game content is defined in JSON and loaded via `once_cell::Lazy` statics.

### Standard Pattern

```rust
// src/game/item.rs
use once_cell::sync::Lazy;

static ITEMS: Lazy<HashMap<String, ItemDef>> = Lazy::new(|| {
    let data = include_str!("../../data/items.json");
    let file: ItemsFile = serde_json::from_str(data).expect("Failed to parse");
    file.items.into_iter().map(|i| (i.id.clone(), i)).collect()
});

pub fn get_item_def(id: &str) -> Option<&'static ItemDef> {
    ITEMS.get(id)
}

pub fn all_item_ids() -> Vec<&'static str> {
    ITEMS.keys().map(|s| s.as_str()).collect()
}
```

### Data Files

| File                     | Rust Module     | Contains                        |
| ------------------------ | --------------- | ------------------------------- |
| `items.json`             | `item.rs`       | Items, equipment, consumables   |
| `enemies.json`           | `enemy.rs`      | Enemy stats, behaviors, loot    |
| `npcs.json`              | `npc.rs`        | NPCs, merchants, dialogue refs  |
| `quests.json`            | `quest.rs`      | Quest definitions, objectives   |
| `adaptations.json`       | `adaptation.rs` | Player mutations/upgrades       |
| `dialogues.json`         | `dialogue.rs`   | Conversation trees              |
| `recipes.json`           | `crafting.rs`   | Crafting recipes                |
| `storm_config.json`      | `storm.rs`      | Storm timing and effects        |
| `loot_tables.json`       | `loot.rs`       | Weighted loot distributions     |
| `biome_spawn_tables.json`| `spawn.rs`      | Per-biome enemy spawns          |
| `status_effects.json`    | `status.rs`     | Status effect definitions       |

---

## Key Systems Detail

### Combat System

**Location**: `src/game/systems/combat.rs`, `src/game/combat.rs`

**Flow**:
```
Player Input (Attack) → CombatSystem::attack_melee/ranged()
  → roll_attack() (hit/miss/crit)
  → calc_damage() (base + modifiers)
  → Apply damage to enemy
  → If HP <= 0: process_enemy_death()
    → Emit EnemyKilled event
    → Award XP
    → Trigger on_death effects
```

**Adding Combat Mechanics**: Modify `roll_attack()` or `calc_damage()` in `src/game/combat.rs`. These are pure functions that don't depend on `GameState`.

### AI System

**Location**: `src/game/systems/ai.rs`

**Pattern**: Strategy Pattern via Behavior Registry

```rust
pub trait AiBehavior: Send + Sync {
    fn execute(&self, entity_idx: usize, state: &mut GameState) -> bool;
}

static BEHAVIOR_REGISTRY: Lazy<HashMap<&str, Box<dyn AiBehavior>>> = ...;
```

**Built-in Behaviors**:
- `StandardMeleeBehavior` — Chase and melee attack
- `RangedOnlyBehavior` — Keep distance, ranged attacks
- `SuicideBomberBehavior` — Explode on contact
- `HealerBehavior` — Heal nearby allies

**Adding a Behavior**:
1. Create struct implementing `AiBehavior`
2. Register in `BEHAVIOR_REGISTRY`
3. Reference by `behavior_id` in `data/enemies.json`

### Enemy System

**Location**: `src/game/enemy.rs`

**Data Structure** (in `data/enemies.json`):
```json
{
  "id": "glass_crawler",
  "name": "Glass Crawler",
  "glyph": "c",
  "max_hp": 15,
  "damage_min": 2, "damage_max": 5,
  "sight_range": 8,
  "behaviors": [
    {"type": "split_on_death", "condition": "glass_shard", "value": 2}
  ],
  "behavior_id": "standard_melee",
  "xp_value": 25
}
```

### Quest System

**Location**: `src/game/quest.rs`, `src/game/systems/quest.rs`

**Objective Types**:
- `Kill { enemy_id, count }`
- `Collect { item_id, count }`
- `Reach { x, y }`
- `TalkTo { npc_id }`

**Quest Chains**: Quests can unlock other quests via `reward.unlocks_quests`.

### Storm System

**Location**: `src/game/storm.rs`, `src/game/systems/storm.rs`

**Edit Types** (map transformations):
- `Glass` — Convert tiles to glass
- `Rotate` — Rotate map section
- `Swap` — Swap two areas
- `Mirror` — Mirror a section
- `Fracture` — Create cracks
- `Crystallize` — Create crystal formations
- `Vortex` — Spiral pattern transformation

---

## Visual Effects DSL

Effects use a string-based DSL defined in `data/effects_config.json`.

**Syntax**: `"EFFECT(@SPEED &COLOR)"`

**Examples**:
```
"B(@3 &Cyan)"      — Blink at speed 3, cyan color
"G(&Yellow)"       — Glow yellow
"P(@2 &Red)"       — Pulse at speed 2, red
"S(@1 &White,Blue)"— Shimmer between white and blue
```

**Usage**: Effects are attached to entities and tiles via the `effects` field in data files.

---

## Entity Trait

**Location**: `src/game/entity.rs`

Unified interface for enemies and NPCs:

```rust
pub trait Entity {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn hp(&self) -> Option<i32>;
    fn status_effects(&self) -> &[StatusEffect];
    fn name(&self) -> &str;
    fn glyph(&self) -> char;
}
```

**Purpose**: Enables generic systems that operate on any entity type without type-specific code.

---

## DES Testing System

**Location**: `src/des/mod.rs`, `tests/scenarios/`

The Debug Execution System enables headless, deterministic testing.

### Scenario Structure

```json
{
  "name": "combat_basic",
  "seed": 12345,
  "map_setup": {
    "clear_radius": 5,
    "ensure_paths": [{"from_x": 10, "from_y": 10, "to_x": 11, "to_y": 10}]
  },
  "player": { "x": 10, "y": 10, "inventory": ["sword"] },
  "entities": [
    {"entity_type": "enemy", "id": "glass_crawler", "x": 11, "y": 10}
  ],
  "mocks": {
    "combat_always_hit": true,
    "combat_fixed_damage": 100
  },
  "actions": [
    {"turn": 0, "action": {"type": "attack", "target_x": 11, "target_y": 10}}
  ],
  "assertions": [
    {"at_end": true, "check": {"type": "enemy_at", "x": 11, "y": 10, "alive": false}}
  ]
}
```

### Running Tests

```bash
# Run all DES scenarios
cargo test --test des_scenarios

# Run specific scenario
cargo test --test des_scenarios combat_basic

# Run with output
cargo test --test des_scenarios -- --nocapture
```

### Writing Test Scenarios

1. Create `tests/scenarios/my_test.json`
2. Define initial state: `player`, `entities`, `map_setup`
3. Define `actions` to execute each turn
4. Define `assertions` to verify outcomes
5. Use `mocks` to control randomness if needed

### Assertion Types

- `player_alive` / `player_dead`
- `player_hp { op, value }` — Compare player HP
- `player_position { x, y }`
- `has_item { item_id }`
- `enemy_at { x, y, alive }`
- `enemy_count { op, value }`
- `quest_complete { quest_id }`
- `message_contains { text }`

---

## Game Loop

**Location**: `src/main.rs`

```
┌─────────────────────────────────────────────┐
│                  Main Loop                   │
├─────────────────────────────────────────────┤
│  1. Handle Input → Action enum               │
│  2. update(state, action)                    │
│     ├─ Movement → MovementSystem            │
│     ├─ Combat → CombatSystem                │
│     └─ etc.                                 │
│  3. state.end_turn()                        │
│     ├─ AI runs → AiSystem                   │
│     ├─ Status effects tick                  │
│     ├─ Storm progresses                     │
│     └─ Events processed                     │
│  4. Render → Renderer reads GameState       │
└─────────────────────────────────────────────┘
```

---

## Decoupled Design Example: Crafting

`src/game/crafting.rs` demonstrates ideal decoupling:

```rust
// Pure function — doesn't need GameState
pub fn can_craft(recipe: &Recipe, inventory: &[String]) -> bool {
    for (item_id, &required) in &recipe.materials {
        let count = inventory.iter().filter(|id| *id == item_id).count();
        if count < required as usize { return false; }
    }
    true
}
```

**Why It's Good**:
- Takes specific data, not entire `GameState`
- Easy to test in isolation
- No side effects
- Can be called from UI without game logic coupling

---

## Adding New Content

### New Item

1. Add to `data/items.json`:
```json
{
  "id": "prism_shard",
  "name": "Prism Shard",
  "glyph": "*",
  "description": "A crystalline fragment that refracts light.",
  "value": 50,
  "usable": true,
  "heal": 10
}
```

2. That's it! Item is now spawnable and usable.

### New Enemy

1. Add to `data/enemies.json`:
```json
{
  "id": "salt_wraith",
  "name": "Salt Wraith",
  "glyph": "W",
  "max_hp": 30,
  "damage_min": 5, "damage_max": 10,
  "behavior_id": "standard_melee",
  "xp_value": 50
}
```

2. Add to spawn tables in `data/biome_spawn_tables.json` if needed.

### New Quest

1. Add to `data/quests.json`:
```json
{
  "id": "hunt_wraiths",
  "name": "Wraith Hunter",
  "description": "Eliminate the salt wraiths.",
  "objectives": [
    {"id": "kill_wraiths", "description": "Kill 5 salt wraiths",
     "type": "kill", "enemy_id": "salt_wraith", "count": 5}
  ],
  "reward": {"xp": 200, "salt_scrip": 100}
}
```

### New Mechanic

1. Consider if it should be a new System or extend existing
2. Create/modify system in `src/game/systems/`
3. Add any new events to `GameEvent` enum
4. Write DES test scenarios
5. Update `GameState` if new data fields needed

---

## Common Patterns

### Safe Definition Lookup

```rust
// BAD: Panics if enemy def missing
let damage = enemy.def().unwrap().damage_max;

// GOOD: Handle missing gracefully
let Some(def) = enemy.def() else {
    log::warn!("Missing enemy def: {}", enemy.id);
    return;
};
let damage = def.damage_max;
```

### Deterministic Randomness

```rust
// BAD: Non-deterministic
use rand::thread_rng;
let roll = thread_rng().gen_range(1..=20);

// GOOD: Uses seeded RNG from state
let roll = state.rng.gen_range(1..=20);
```

### Spatial Index Usage

```rust
// Ensure index is fresh before queries
state.ensure_spatial_index();
if let Some(&enemy_idx) = state.enemy_positions.get(&(x, y)) {
    // Enemy found at position
}
```

---

## Related Documentation

- [ARCHITECTURE_AUDIT.md](./ARCHITECTURE_AUDIT.md) — Technical audit and recommendations
- [systems_analysis.md](./systems_analysis.md) — Detailed refactoring history and anti-patterns
- [SCALABILITY_AUDIT.md](./SCALABILITY_AUDIT.md) — Performance considerations
- [TECH_STACK.md](./TECH_STACK.md) — Technology choices

---

## Glossary

| Term       | Definition                                                              |
| ---------- | ----------------------------------------------------------------------- |
| DES        | Debug Execution System — headless testing framework                     |
| GameState  | Central data struct holding all game state                              |
| System     | Stateless module that operates on GameState                             |
| Adaptation | Player mutation/upgrade that grants abilities                           |
| Storm      | Glass storm event that transforms the map                               |
| Entity     | Trait unifying Enemy/NPC with common interface                          |
| Behavior   | AI behavior strategy for enemies                                        |
| DSL        | Domain-Specific Language (used for visual effects)                      |
