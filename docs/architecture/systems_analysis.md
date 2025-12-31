# Game Systems Architecture Analysis

## Progress Summary (Updated 2024-12-31)

| Anti-Pattern | Status | Notes |
|-------------|--------|-------|
| 1. God Object | In Progress | Systems extracted: Combat, AI, Movement |
| 2. Unused Event Bus | **FIXED** | Events now processed in `end_turn()` |
| 3. Mixed Abstraction Levels | **FIXED** | `try_move()` extracted to `MovementSystem` |
| 4. Spatial Index Sync | Partially Fixed | Added missing index updates in AI spawning |
| 5. Test Schema Drift | Partially Fixed | New assertions added |
| 6. Hardcoded Entity Checks | **FIXED** | Laser beam is data-driven |
| 7. Panic on Missing Defs | **FIXED** | Safe patterns used |
| 8. Code Duplication | **FIXED** | `process_enemy_death()` extracted |

**Tests**: 68 passing, 10 broken scenarios in `tests/scenarios/broken/`

**Line Counts**:
- `src/game/state.rs`: 3009 lines (down from 3150)
- `src/game/systems/`: 1008 lines (combat: 292, ai: 404, movement: 297)

---

## Current Architecture Overview

The current architecture of Saltglass Steppe revolves around a central "God Object" pattern. The `GameState` struct (defined in `src/game/state.rs`) holds all game data and implements the majority of the game logic.

### Core Components

1.  **GameState (`src/game/state.rs`)**:
    -   **Data**: Holds `player`, `map`, `enemies`, `items`, `storm`, `messages`, etc.
    -   **Logic**: Implements `try_move`, `end_turn`, `process_ai`, `process_enemy_behavior`, `apply_light_effects`, etc.
    -   **Coupling**: Directly depends on almost every other module in `src/game/`.

2.  **Data Definitions**:
    -   Static data is loaded from JSON files into `once_cell::Lazy` static HashMaps/Vecs.
    -   Examples: `src/game/item.rs` (items.json), `src/game/enemy.rs` (enemies.json), `src/game/action.rs` (actions.json).
    -   This is a strong, data-driven foundation.

3.  **Systems (Logic Modules)**:
    -   **Combat**: Split between `src/game/combat.rs` (pure math helpers) and `src/game/combat_actions.rs` (state mutation via `impl GameState`).
    -   **AI**: `src/game/ai.rs` implements `process_ai` as a method on `GameState`.
    -   **Storm**: `src/game/storm.rs` handles forecasting logic, but application logic resides in `GameState`.
    -   **Map**: `src/game/map.rs` defines the grid, but manipulation happens inside `GameState`.

4.  **UI Layer (`src/ui/`)**:
    -   Decoupled from game logic.
    -   Reads `GameState` for rendering.
    -   Calls `GameState` methods for input handling.

5.  **Testing (DES)**:
    -   `src/des/` and `tests/` drive `GameState` directly via JSON scenarios.
    -   Relies on the deterministic nature of `GameState`.

## System Analysis & Coupling

### 1. Combat System
-   **Current**: `GameState` methods (`try_melee_attack`, `try_ranged_attack`) handle everything: checking range, spending AP, rolling dice, applying damage, handling death, dropping loot, awarding XP, and emitting events.
-   **Issues**:
    -   Logic is hardcoded in `GameState`.
    -   "Split on death" and other special behaviors are implemented inline with string checks.
    -   Difficult to extend without modifying `GameState`.

### 2. AI System
-   **Current**: `GameState::process_ai` iterates over enemies.
-   **Issues**:
    -   Hardcoded behavior strings ("suicide_bomber", "teleport").
    -   Directly mutates player HP and map state.
    -   Mixes movement logic, attack logic, and special ability logic in one large loop.

### 3. Storm System
-   **Current**: `Storm` struct tracks timer. `GameState` checks timer and applies effects.
-   **Issues**:
    -   Storm effects (like "Glass" conversion) are likely implemented directly in `GameState` (though not fully verified in this pass, the pattern suggests it).

### 4. State Management
-   **Current**: `GameState` is a massive struct.
-   **Issues**:
    -   Any change to game logic requires recompiling `GameState` and its dependencies.
    -   Testing specific systems in isolation is difficult because they all require a full `GameState`.

## Proposed Architecture Changes

To improve maintainability, extensibility, and testability, we propose the following changes:

### 1. Decouple Systems from GameState

Move logic out of `GameState` into dedicated system structs. `GameState` should primarily be a data container.

**Pattern:**
```rust
struct CombatSystem;

impl CombatSystem {
    pub fn resolve_attack(state: &mut GameState, attacker_id: EntityId, target_id: EntityId) {
        // Logic here
    }
}
```

**Proposed Systems:**
-   `CombatSystem`: Handles attacks, damage calculation, death processing.
-   `AiSystem`: Handles enemy decision making.
-   `MovementSystem`: Handles entity movement and collision.
-   `StormSystem`: Handles storm progression and effect application.
-   `EffectSystem`: Handles status effects and triggered effects.

### 2. Event-Driven Architecture

Expand the use of `GameEvent` to decouple systems.

-   **Current**: `GameState` emits some events.
-   **Proposed**: Systems emit events (e.g., `EntityAttacked`, `EntityMoved`, `StormStarted`). Other systems listen and react.
    -   Example: `CombatSystem` emits `EntityDied`. `LootSystem` listens and drops loot. `QuestSystem` listens and updates objectives.

### 3. Data-Driven Behavior Registry

Replace hardcoded string checks with a registry pattern or strategy pattern.

-   **Current**: `match behavior_type { "suicide_bomber" => ... }`
-   **Proposed**:
    -   Define a `Behavior` trait.
    -   Register behaviors in a `BehaviorRegistry`.
    -   Load behaviors by name from JSON.
    -   `AiSystem` looks up behavior and executes it.

### 4. Component-Like Access

While a full ECS might be overkill, we can structure `GameState` to provide component-like access to data.

-   **Current**: `state.enemies[i].hp`
-   **Proposed**: Helper methods to access "components" of entities (Player, Enemy, NPC) uniformly where possible.

## Good Examples in Codebase

### Crafting System (`src/game/crafting.rs`)
The crafting system is a good example of decoupled design:
-   It defines its own data structures (`Recipe`).
-   It loads data independently.
-   It provides **pure functions** (`can_craft`, `crafting_success_chance`) that take specific data arguments (inventory slice, player level) rather than the entire `GameState`.
-   It does not import `GameState`.

This pattern should be replicated in other systems.

## Implementation Roadmap

### Phase 1: Foundation - The Event Bus & System Trait (COMPLETED)

The current `GameEvent` system is underutilized (events are queued but never drained). We will establish a robust Event Bus to allow systems to communicate without direct coupling.

**1.1 Define the System Trait**
Create a standard interface for all game logic modules.
```rust
pub trait System {
    /// Run the system logic for one turn/frame
    fn update(&self, state: &mut GameState);
    
    /// Handle specific events (optional)
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
```

**1.2 Activate the Event Bus**
Modify the game loop to process events.
```rust
// In main.rs or a new game_loop.rs
let mut event_bus = EventBus::new();
let mut systems: Vec<Box<dyn System>> = vec![
    Box::new(CombatSystem::new()),
    Box::new(AiSystem::new()),
    Box::new(StormSystem::new()),
];

// Game Loop
loop {
    // 1. Process Input -> Actions
    // 2. Update Systems
    for system in &mut systems {
        system.update(&mut state, &mut event_bus);
    }
    // 3. Process Events
    while let Some(event) = event_bus.pop() {
        for system in &mut systems {
            system.on_event(&event, &mut state);
        }
    }
}
```

### Phase 2: Decoupling Combat (The "Action" System) (COMPLETED)

Currently, `GameState` has methods like `try_melee_attack`. These should be moved to a `CombatSystem`.

**2.1 Extract Logic**
Move `src/game/combat_actions.rs` content into a new `CombatSystem` struct.
-   **Input**: `Action::MeleeAttack(x, y)`
-   **Process**: `CombatSystem` validates range, rolls dice.
-   **Output**: Emits `GameEvent::DamageDealt`, `GameEvent::EnemyKilled`.
-   **State Mutation**: `CombatSystem` modifies `state.enemies` and `state.player_hp` directly (for now), but logic is contained.

**2.2 Event-Driven Side Effects**
Instead of `CombatSystem` calling `quest_log.on_enemy_killed()`, it emits `GameEvent::EnemyKilled`.
-   `QuestSystem` listens for `EnemyKilled` and updates quests.
-   `LootSystem` listens for `EnemyKilled` and spawns loot.
-   `ProgressionSystem` listens for `EnemyKilled` and awards XP.

### Phase 3: Data-Driven AI (Behavior Registry) (COMPLETED)

Hardcoded AI behaviors ("suicide_bomber") prevent easy content creation.

**3.1 The Behavior Trait**
```rust
trait AiBehavior: Send + Sync {
    fn execute(&self, entity_idx: usize, state: &mut GameState) -> bool;
}
```

**3.2 The Registry**
Create a `BehaviorRegistry` (singleton/static) that maps string IDs to `Box<dyn AiBehavior>`.
-   "standard_melee" -> `StandardMeleeBehavior` (default, handles most enemies)
-   Future: "suicide_bomber" -> `SuicideBomberBehavior`
-   Future: "healer" -> `HealerBehavior`

**3.3 Refactor AiSystem**
`AiSystem` iterates enemies, looks up their behavior ID in the registry, and delegates execution.

**Implementation Notes:**
-   Created `src/game/systems/ai.rs` with `AiSystem` and `AiBehavior` trait.
-   Moved all AI logic from `src/game/ai.rs` (now deleted) into `StandardMeleeBehavior`.
-   `BEHAVIOR_REGISTRY` uses `once_cell::Lazy<HashMap<String, Box<dyn AiBehavior>>>`.
-   Currently all enemies use "standard_melee" behavior; future work will dispatch to specific behaviors based on enemy def.

### Phase 4: Storm System Unification (TODO)

Combine `src/game/storm.rs` (data/forecasting) with the effect logic currently scattered in `GameState`.

-   `StormSystem` manages the `Storm` struct.
-   It listens for `TurnEnded`.
-   It updates the storm timer.
-   When storm hits, it emits `StormStarted`.
-   It handles `StormEditType` application (map mutation) internally, removing that logic from `GameState`.

---

## Current State (Updated 2024-12-31)

### Completed Work
1.  **System Trait & Module Structure**: `src/game/systems/mod.rs` defines `System` trait.
2.  **CombatSystem**: `src/game/systems/combat.rs` handles melee and ranged attacks.
3.  **AiSystem**: `src/game/systems/ai.rs` handles enemy AI with behavior registry foundation.
4.  **Facade Pattern**: `GameState` methods (`attack_melee`, `try_ranged_attack`, `update_enemies`) delegate to systems.

### Files Changed
-   `src/game/systems/mod.rs` - System trait definition
-   `src/game/systems/combat.rs` - Combat logic
-   `src/game/systems/ai.rs` - AI logic with behavior registry
-   `src/game/combat_actions.rs` - Facade methods on GameState
-   `src/game/mod.rs` - Removed `ai` module, added `systems`
-   Deleted: `src/game/ai.rs`

---

## Future Work & Areas for Improvement

### High Priority

1.  **Complete Behavior Registry**
    -   Currently all enemies use `StandardMeleeBehavior`.
    -   Extract specific behaviors (suicide_bomber, teleport, healer) into separate structs.
    -   Allow enemy definitions in JSON to specify behavior IDs.
    -   Example: `"behavior": "suicide_bomber"` in `data/enemies.json`.

2.  **Event-Driven Side Effects**
    -   `CombatSystem` still directly calls `quest_log.on_enemy_killed()`, `drop_enemy_loot()`, `gain_xp()`.
    -   These should be event listeners reacting to `GameEvent::EnemyKilled`.
    -   Create `LootSystem`, `QuestSystem`, `ProgressionSystem` to handle these.

3.  **Storm System Extraction**
    -   `apply_storm()` logic is still in `GameState`.
    -   Create `src/game/systems/storm.rs`.
    -   Move storm effect application (map mutation) to `StormSystem`.

4.  **Movement System**
    -   `try_move()` is still a large method in `GameState`.
    -   Extract to `MovementSystem` to handle collision, interaction with NPCs/items/chests.

### Medium Priority

5.  **Status Effect System**
    -   Status effect ticking is in `GameState::tick_status_effects()`.
    -   Extract to `EffectSystem` that can handle both player and enemy status effects.

6.  **Unified Entity Model**
    -   Player, Enemy, NPC share similar attributes (position, hp, status effects).
    -   Consider a unified `Entity` trait or struct with component-like access.
    -   Would simplify systems that operate on "any entity".

7.  **Clean Up Unused Imports**
    -   Several warnings about unused imports in `combat_actions.rs` and `systems/ai.rs`.
    -   Run `cargo fix` to clean up.

### Low Priority / Future Considerations

8.  **Activate Event Bus in Game Loop**
    -   Currently `GameState::drain_events()` is never called.
    -   Modify `main.rs` or create `game_loop.rs` to process events each frame.
    -   Allow systems to subscribe to event types.

9.  **Data-Driven Behavior Loading**
    -   Load behavior implementations from a config file or dynamically register them.
    -   This would allow modding/extending AI without recompiling.

10. **Performance Profiling**
    -   The AI system iterates all enemies each turn.
    -   Consider spatial partitioning for large enemy counts.
    -   Profile pathfinding (A*) for bottlenecks.

### Technical Debt

11. **Broken Test Scenarios** (PARTIALLY FIXED 2024-12-31)
    -   10 scenarios moved to `tests/scenarios/broken/` due to schema errors or assertion failures.
    -   Root causes identified:
        -   Missing `turns` field in Wait actions (schema change not reflected in old tests)
        -   Invalid assertion types (`chest_count_min`, `enemy_count_min`, `npc_count_min` don't exist)
        -   Invalid entity types (`chest` not supported in DES entities)
        -   Combat scenarios failing due to map generation blocking player movement
    -   95 scenarios remain working.

12. **Hardcoded Enemy IDs**
    -   `has_laser = self.enemies[i].id == "laser_drone"` is hardcoded in AI.
    -   Should be a flag in `EnemyDef` like `has_laser: bool` or a behavior.

13. **Code Duplication in CombatSystem**
    -   `attack_melee` and `ranged_attack` have duplicated "on death" logic.
    -   Extract into a shared `process_enemy_death()` helper.

14. **Unwrap on Optional Enemy Definitions** (FIXED 2024-12-31)
    -   `state.enemies[i].def().unwrap()` in AI system caused panics.
    -   Fixed with `let Some(def) = state.enemies[i].def() else { return true; };`

15. **Unit Test Fragility**
    -   Tests like `glass_increases_refraction` and `item_removed_after_walking_onto_it` failed because:
        -   Enemies/NPCs could spawn at the test target location
        -   Spatial indices not cleared when entities cleared
    -   Fixed by adding `enemies.clear(); npcs.clear(); rebuild_spatial_index();` before test setup.

---

## Anti-Patterns Identified (Analysis 2024-12-31)

### 1. God Object (GameState)
**Symptom**: `src/game/state.rs` is 3150 lines with 111+ methods.
**Impact**: Hard to understand, test, and modify. Changes cascade unpredictably.
**Mitigation**: Continue extracting logic to Systems (`CombatSystem`, `AiSystem`, etc.) while keeping `GameState` as a facade.

### 2. Unused Event Bus
**Symptom**: `GameEvent` enum defined, `emit()` and `drain_events()` exist, but `drain_events()` is never called.
**Impact**: Side effects are tightly coupled (combat directly calls quest, loot, xp systems).
**Mitigation**: Activate event bus in game loop, convert side effects to event listeners.

### 3. Mixed Abstraction Levels
**Symptom**: `try_move()` handles: collision, NPC dialogue, combat, item pickup, world transitions, glass damage, glare effects.
**Impact**: Single method doing 10+ things makes bugs hard to isolate.
**Mitigation**: Extract to `MovementSystem` that orchestrates smaller, focused handlers.

### 4. Implicit Dependencies via Spatial Indices
**Symptom**: `enemy_positions`, `npc_positions`, `item_positions` HashMaps must be kept in sync with entity vectors.
**Impact**: Tests fail silently when indices are stale. Easy to forget `rebuild_spatial_index()`.
**Mitigation**: Consider on-demand spatial queries or automatic index invalidation.

### 5. Test Scenarios Schema Drift (PARTIALLY FIXED 2024-12-31)
**Symptom**: Old test scenarios use outdated schemas (missing `turns`, wrong assertion types).
**Impact**: `run_all_scenarios` fails on valid scenarios due to parse errors.
**Mitigation**: Add schema versioning or validation. Document required fields.
**Progress**: Added `enemy_count`, `npc_count`, `chest_count` assertions. Added `Chest` entity type.

### 6. Hardcoded Entity Checks (FIXED 2024-12-31)
**Symptom**: `if enemy.id == "laser_drone"` instead of `if enemy.def().has_laser`.
**Impact**: Adding new laser enemies requires code changes, not just data.
**Mitigation**: Move special abilities to enemy definition flags or behavior system.
**Resolution**: Laser beam damage now read from `behaviors` in enemy definition.

### 7. Panic on Missing Definitions (FIXED 2024-12-31)
**Symptom**: `enemy.def().unwrap()` assumes all enemies have valid definitions.
**Impact**: Runtime panics when enemy ID doesn't match any definition.
**Mitigation**: Use `let Some(def) = enemy.def() else { ... };` pattern.
**Resolution**: All `unwrap()` calls on enemy definitions replaced with safe patterns.

### 8. Code Duplication in CombatSystem (FIXED 2024-12-31)
**Symptom**: `attack_melee` and `ranged_attack` had duplicated "on death" logic (~50 lines each).
**Impact**: Bug fixes need to be applied in multiple places. Ranged was missing `on_death` effects.
**Resolution**: Extracted `process_enemy_death()` helper function.

---

## Recommendations for Phase 5+

### Immediate (Next Sprint) - COMPLETED 2024-12-31

1.  ~~**Fix Broken Test Scenarios**~~ - Partially done
    -   10 scenarios moved to `tests/scenarios/broken/`
    -   Root causes identified; some need DES framework changes to fix

2.  ~~**Activate Event Bus**~~ - **COMPLETED**
    -   `end_turn()` now calls `process_events()` to drain and dispatch events
    -   Events logged with `[Event]` prefix for debugging

3.  ~~**Extract MovementSystem**~~ - **COMPLETED**
    -   Created `src/game/systems/movement.rs` (297 lines)
    -   Handles: NPC interaction, enemy combat, tile effects, world transitions, item pickup
    -   `GameState::try_move()` now delegates to `MovementSystem::try_move()`

### Short-term (1-2 Sprints) - REMAINING WORK

4.  ~~**Complete Behavior Registry**~~ - **COMPLETED**
    -   Added `behavior_id` field to `EnemyDef` in `src/game/enemy.rs`
    -   Implemented `SuicideBomberBehavior`, `HealerBehavior`, `RangedOnlyBehavior` as separate structs
    -   AI system now dispatches to specific behaviors based on enemy definition's `behavior_id`
    -   Added `behavior_id` to `glass_bomber` (suicide_bomber) and `storm_archer` (ranged_only)

5.  ~~**Create LootSystem and QuestSystem as Event Listeners**~~ - **COMPLETED**
    -   Created `LootSystem` (`src/game/systems/loot.rs`) that subscribes to `EnemyKilled` events
    -   Created `QuestSystem` (`src/game/systems/quest.rs`) that subscribes to `EnemyKilled`, `ItemPickedUp`
    -   Removed direct calls from `CombatSystem::process_enemy_death()`
    -   Events now dispatched via `process_events()` in `state.rs`

6.  ~~**Add Missing DES Assertions**~~ - **COMPLETED**
    -   Added `enemy_count`, `npc_count`, `chest_count` assertions with `op` comparison
    -   Added `Chest` entity type with inventory support

7.  ~~**Extract StatusEffectSystem**~~ - **COMPLETED**
    -   Created `StatusEffectSystem` (`src/game/systems/status.rs`)
    -   Handles player and enemy status effect ticking
    -   Reads tick_damage from `data/status_effects.json`
    -   Provides helper functions: `player_is_stunned`, `enemy_is_stunned`, `player_accuracy_penalty`
    -   Enemies killed by status effects emit `EnemyKilled` event

8.  ~~**Fix Broken Combat Test Scenarios**~~ - **COMPLETED**
    -   Added `MapSetup` struct to DES framework with:
        -   `clear_radius`: Clear circular area around player
        -   `clear_areas`: Clear specific rectangular regions
        -   `ensure_paths`: Carve walkable paths between points using Bresenham's algorithm
    -   Fixed 6 broken scenarios: combat_basic, item_pickup_basic, combat_kill_xp, combat_ranged_kill, mock_combat, enemy_hp_check
    -   Remaining scenarios in broken/ are feature-specific (equipment, skills, quests)

### Long-term (Future Phases)

9.  ~~**Unified Entity Model**~~ - **COMPLETED**
    -   Created `Entity` trait (`src/game/entity.rs`) with position, hp, status effects
    -   Implemented `Entity` for `Enemy` and `Npc`
    -   Provides: `x()`, `y()`, `set_position()`, `hp()`, `max_hp()`, `status_effects()`, `name()`, `glyph()`
    -   Enables generic systems that operate on any entity type

10. ~~**Storm System Extraction**~~ - **COMPLETED**
    -   Created `StormSystem` (`src/game/systems/storm.rs`, 319 lines)
    -   Moved all storm edit types: Glass, Rotate, Swap, Mirror, Fracture, Crystallize, Vortex
    -   Removed ~277 lines from `state.rs` (now 2689 lines, down from 3150)
    -   Updated DES framework and unit tests to use `StormSystem::apply_storm()`

11. **Performance Optimization**
    -   Profile A* pathfinding for large maps
    -   Consider spatial partitioning for enemy lookups (current: O(n) scan)
    -   Potential: Lazy FOV computation

12. ~~**Spatial Index Improvements**~~ - **PARTIALLY COMPLETED**
    -   Added `spatial_dirty` flag for lazy invalidation
    -   Added `mark_spatial_dirty()` and `ensure_spatial_index()` methods
    -   `ensure_spatial_index()` called at start of `end_turn()` before AI runs
    -   Explicit `rebuild_spatial_index()` still available for backwards compatibility
    -   Note: Query functions still take `&self`, not `&mut self` to avoid breaking changes

---

## Work Completed (2024-12-31 Session)

### Systems Extracted
| System | Lines | Purpose |
|--------|-------|---------|
| `CombatSystem` | 292 | Melee/ranged attacks, death handling |
| `AiSystem` | 404 | Enemy AI, behavior registry foundation |
| `MovementSystem` | 297 | Player movement, NPC/combat triggers, tile effects |
| **Total** | 1008 | ~33% of original `state.rs` logic |

### Anti-Patterns Fixed
- ✅ Unused Event Bus → Events now processed each turn
- ✅ Mixed Abstraction in `try_move()` → Extracted to MovementSystem
- ✅ Hardcoded Entity Checks → Laser beam is data-driven
- ✅ Panic on Missing Defs → Safe patterns used throughout
- ✅ Code Duplication → `process_enemy_death()` shared helper

### Test Scenarios Added
- `laser_beam_behavior.json` - Data-driven laser attack
- `entity_count_assertions.json` - Count assertions
- `chest_spawn_test.json` - Chest entity spawning
- `event_bus_test.json` - Event processing
- `movement_system_test.json` - Glass tile damage

### Code Metrics
- `state.rs`: 3150 → 3009 lines (-141 lines, -4.5%)
- New systems: +1008 lines (well-organized, focused modules)
- Tests: 68 passing (was 55 at session start)

---

## Architectural Patterns Reference

### Impact Analysis
A scan of the codebase reveals that `GameState` methods like `try_move`, `try_ranged_attack`, and `end_turn` are heavily used in `src/main.rs` and the DES testing framework (`src/des/`, `tests/`).

**Risk**: Changing the public API of `GameState` will break the game loop and all tests.

**Mitigation Strategy**:
1.  **Maintain Facade**: Keep the existing `GameState` methods (`try_move`, `end_turn`) as the public API.
2.  **Internal Delegation**: Refactor these methods to delegate to the new systems instead of containing the logic themselves.
    *   `GameState::end_turn` -> calls `systems::ai::update(self)` and `systems::storm::update(self)`.
3.  **Stateless Systems**: To avoid serialization issues (since `GameState` is serialized), systems should be stateless functions or static singletons that accept `&mut GameState` as a parameter. They should not be fields of `GameState`.

## Architectural Patterns Reference

### 1. The "System" Pattern (Data-Oriented Design)
*Concept*: Separate Data (State) from Logic (Systems).
*Fit*: Perfect for Rust. `GameState` is the Data. `CombatSystem`, `AiSystem` are the Logic.
*Benefit*: Solves the "God Object" problem. Allows multiple systems to operate on the same data without owning it.

### 2. Event Bus (Observer Pattern)
*Concept*: Components communicate by broadcasting events, not by calling methods on each other.
*Fit*: Essential for decoupling "Side Effects" (Quests, Achievements, Stats, Logs) from "Core Logic" (Combat, Movement).
*Benefit*: You can add a new `AchievementSystem` without touching `CombatSystem` code.

### 3. Command Pattern
*Concept*: Encapsulate a request as an object (`Action` enum).
*Fit*: Already partially used (`Action` enum). Can be expanded to queue actions for AI and other entities, not just the player.
*Benefit*: Enables features like "Replay System", "Undo", and "Queueing Actions".

### 4. Strategy Pattern (for AI)
*Concept*: Define a family of algorithms (Behaviors), encapsulate each one, and make them interchangeable.
*Fit*: The `BehaviorRegistry` proposal.
*Benefit*: Designers can mix-and-match behaviors in JSON (`"behaviors": ["aggressive", "suicide_bomber"]`) without code changes.
