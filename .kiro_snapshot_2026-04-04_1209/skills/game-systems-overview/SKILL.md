---
name: game-systems-overview
description: Comprehensive guide to Saltglass Steppe's architecture and systems. Use when working on system architecture, understanding how systems interact, adding new content types, or planning new features.
---

# Game Systems Overview

## Quick Reference

| I Want To... | Look At |
|---|---|
| Add a new item | `data/items.json` |
| Add a new enemy | `data/enemies.json` |
| Add a new quest | `data/quests.json` |
| Add a dynamic event | `data/dynamic_events.json` |
| Add a story fragment | `data/narrative_integration.json` |
| Create a test scenario | `tests/scenarios/` |
| Add a new AI behavior | `src/game/systems/ai.rs` |
| Add a new game mechanic | `src/game/systems/` |
| Understand the game loop | `src/main.rs` |

---

## Architecture Principles

1. **Determinism First** — All game logic uses seeded `ChaCha8Rng`. Required for DES testing.
2. **Data-Driven** — Content in `data/*.json`. No code changes needed to add items/enemies/quests.
3. **Strict Layer Separation**:
   - `src/game/` — Pure game logic. Never imports rendering.
   - `src/renderer/` — Reads `GameState`, never modifies it.
   - `src/des/` — Headless test framework, drives `GameState` without rendering.
4. **Event-Driven** — Systems communicate via `GameEvent` bus, not direct calls.

## Directory Structure

```
src/
├── main.rs              # Entry point, game loop, input handling
├── game/                # Pure game logic
│   ├── state.rs         # GameState — single source of truth
│   ├── systems/         # Stateless logic systems (ai, combat, movement, loot, quest, status, storm)
│   ├── generation/      # Procedural generation (20+ modules)
│   └── ...              # enemy, item, npc, quest, map, etc.
├── renderer/            # TUI rendering (ratatui)
├── des/                 # Debug Execution System
└── ui/                  # UI state and input handling
data/                    # JSON content definitions
tests/scenarios/         # DES test scenarios
```

---

## GameState

`src/game/state.rs` — single source of truth. Serializable, deterministic, central hub.

Decomposed into sub-structs for better organization:
- `PlayerState` — Position, stats, inventory, equipment, adaptations, quests, specialized systems
- `WorldState` — Map, entities, environment, encounters, pathfinding
- `NarrativeEngine` — Quests, story, tutorial, history, effects

Key top-level fields:
```rust
pub struct GameState {
    pub player: PlayerState,
    pub world: WorldState,
    pub narrative: NarrativeEngine,
    pub rng: ChaCha8Rng,          // seeded, serde-serialized
    pub event_queue: Vec<GameEvent>,
    pub messages: Vec<GameMessage>,
    pub visible: HashSet<usize>,
    pub revealed: HashSet<usize>,
}
```

Pattern — systems receive `&mut GameState`:
```rust
pub fn attack_melee(state: &mut GameState, target_idx: usize) {
    let damage = state.calc_player_damage();
    state.world.enemies[target_idx].hp -= damage;
    state.event_queue.push(GameEvent::EnemyKilled { ... });
}
```

---

## Systems Layer

Stateless modules that operate on `GameState`. Data lives in state, logic lives in systems.

### Active Systems

| System | File | Responsibility |
|---|---|---|
| `CombatSystem` | `systems/combat.rs` | Attack resolution, damage, death |
| `AiSystem` | `systems/ai.rs` | Enemy decision-making, behavior dispatch |
| `MovementSystem` | `systems/movement.rs` | Player movement, tile effects, NPC triggers |
| `LootSystem` | `systems/loot.rs` | Listens to `EnemyKilled`, drops loot |
| `QuestSystem` | `systems/quest.rs` | Listens to events, updates quest progress |
| `StatusEffectSystem` | `systems/status.rs` | Ticks status effects each turn |
| `StormSystem` | `systems/storm.rs` | Storm progression, map transformations |

### Generation Systems

| System | File | Responsibility |
|---|---|---|
| `TerrainForgeGenerator` | `generation/terrain_forge_adapter.rs` | Tile generation via terrain-forge crate |
| `WorldGenerator` | `generation/world_gen.rs` | World map generation with POI preferences |
| `SpawnSystem` | `generation/spawn.rs` | Weighted entity spawning by biome/level |
| `BiomeSystem` | `generation/biomes.rs` | Biome-specific environmental content |
| `ConnectivitySystem` | `generation/connectivity.rs` | Glass Seam Bridging — ensures map connectivity |
| `EventSystem` | `generation/events.rs` | Dynamic events based on player/world state |
| `NarrativeIntegration` | `generation/narrative.rs` | Story fragment placement |
| `Grammar` | `generation/grammar.rs` | Dynamic text generation with rule expansion |
| `AlgorithmRegistry` | `generation/registry.rs` | Plugin system for generation algorithms |

Adding a new system: create `src/game/systems/my_system.rs`, implement `System` trait, add to `mod.rs`, call from `end_turn()`.

---

## Event Bus

```rust
pub enum GameEvent {
    PlayerDamaged { amount: i32, source: String },
    EnemyKilled { enemy_id: String, x: i32, y: i32 },
    ItemPickedUp { item_id: String },
    AdaptationGained { name: String },
    StormArrived { intensity: u8 },
    LevelUp { level: u32 },
    // ...
}
```

Flow: system pushes event → `end_turn()` calls `process_events()` → each system's `on_event()` reacts.

**Rule**: never call loot/quest logic directly from combat. Emit `EnemyKilled`, let `LootSystem`/`QuestSystem` react.

---

## Data Files

Content is loaded via `once_cell::Lazy` statics using the unified `DataLoader<T>` system. 45 total files organized by category (consolidated from 52):

**Core Game Content (7 files)**: items.json, weapons.json, abilities.json (includes skills, psychic), classes.json, progression.json, adaptations.json, effects.json (includes status_effects, config)

**Enemies & Combat (8 files)**: enemies/common.json, enemies/uncommon.json, enemies/rare.json, enemies/elite.json, enemies/boss.json, factions.json, loot_tables.json, biome_spawn_tables.json

**World Generation (6 files)**: terrain_config.json (active), biome_profiles.json, structure_templates.json, structure_generation.json, microstructures.json, map_features.json, map_elements.json (includes walls, floors, lights)

**Environmental Systems (5 files)**: storm_config.json, encounter_config.json, dynamic_events.json, travel_config.json, auto_explore_config.json

**NPCs & Narrative (6 files)**: npcs.json, dialogues.json (includes ARIA), traders.json, books.json, narrative_templates.json, narrative_integration.json

**Quests & Progression (4 files)**: quests.json, main_questline.json (13-quest main story arc), tutorial.json, recipes.json

**World Objects (4 files)**: chests.json, interactables.json, actions.json, constraint_rules.json

**UI & Rendering (3 files)**: render_config.json, themes.json, keyboard_config.json

**Data Loader**: All files use `src/game/data_loader.rs` with automatic schema validation, duplicate ID detection, and consistent error handling.

---

## Game Loop

```
Main Loop (src/main.rs)
  1. Handle Input → Action enum
  2. update(state, action)
       ├─ Movement → MovementSystem
       ├─ Combat → CombatSystem
       └─ etc.
  3. state.end_turn()
       ├─ AI → AiSystem
       ├─ Status effects tick
       ├─ Storm progresses
       ├─ Dynamic events → EventSystem
       └─ Events processed (LootSystem, QuestSystem react)
  4. travel_to_tile() [on world map movement]
       ├─ Biome content → BiomeSystem
       ├─ Grammar generation
       ├─ Template content → TemplateLibrary
       └─ Narrative fragments → NarrativeIntegration
  5. Render → Renderer reads GameState (never writes)
```

---

## AI System

Strategy pattern via behavior registry:
```rust
pub trait AiBehavior: Send + Sync {
    fn execute(&self, entity_idx: usize, state: &mut GameState) -> bool;
}
```

Built-in behaviors: `StandardMeleeBehavior`, `RangedOnlyBehavior`, `SuicideBomberBehavior`, `HealerBehavior`.

To add: implement `AiBehavior`, register in `BEHAVIOR_REGISTRY`, reference by `behavior_id` in `enemies.json`.

---

## Quest System

Objective types: `Kill`, `Collect`, `Reach`, `TalkTo`, `InterfaceWithAria`.

Features: conditional unlocking, faction reputation thresholds, branching narratives, 4 ending paths.

Quest categories: `main` (Acts I–IV, 13 quests), `side`, `faction`.

---

## Storm System

Map transformation types: `Glass`, `Rotate`, `Swap`, `Mirror`, `Fracture`, `Crystallize`, `Vortex`.

---

## Visual Effects DSL

Syntax: `"EFFECT(@SPEED &COLOR)"` — e.g. `"B(@3 &Cyan)"` = blink speed 3 cyan, `"G(&Yellow)"` = glow yellow.

---

## DES Testing

Headless deterministic testing. Scenarios in `tests/scenarios/*.json`.

```json
{
  "name": "combat_basic",
  "seed": 12345,
  "player": { "x": 10, "y": 10, "inventory": ["sword"] },
  "entities": [{"entity_type": "enemy", "id": "glass_crawler", "x": 11, "y": 10}],
  "mocks": { "combat_always_hit": true, "combat_fixed_damage": 100 },
  "actions": [{"turn": 0, "action": {"type": "attack", "target_x": 11, "target_y": 10}}],
  "assertions": [{"at_end": true, "check": {"type": "enemy_at", "x": 11, "y": 10, "alive": false}}]
}
```

Assertion types: `player_alive/dead`, `player_hp`, `player_position`, `has_item`, `enemy_at`, `enemy_count`, `quest_complete`, `message_contains`.

Run: `cargo test --test des_scenarios`

---

## Adding New Content

- **Item**: add to `data/items.json` — done.
- **Enemy**: add to `data/enemies.json`, optionally add to `data/biome_spawn_tables.json`.
- **Quest**: add to `data/quests.json` or `data/main_questline.json`.
- **Dynamic event**: add to `data/dynamic_events.json` with triggers + consequences.
- **Story fragment**: add to `data/narrative_integration.json` with placement rules.
- **New mechanic**: create system in `src/game/systems/`, add events to `GameEvent`, write DES scenario.

---

## Key Patterns

**Deterministic RNG** — always use `state.rng`, never `thread_rng()`.

**Spatial index** — call `state.ensure_spatial_index()` before querying `state.enemy_positions`.

**Safe def lookup** — use `if let Some(def) = enemy.def()` not `.unwrap()`.
