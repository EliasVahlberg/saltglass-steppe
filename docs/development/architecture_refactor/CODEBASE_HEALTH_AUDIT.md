# Codebase Health Audit — 2026-04-03

## Executive Summary

An audit of the `.agents/summary/` system descriptions against the actual codebase reveals a consistent pattern: **AI-generated feature scaffolding that is vertically complete but horizontally disconnected from gameplay**. Roughly 3,000–4,000 LOC exists as dead or half-wired code. The root cause is not carelessness — it's the absence of structural gates that catch incomplete integration before code is committed.

This report documents all findings, identifies areas needing further investigation, and proposes structural changes that address root causes while supporting future development.

---

## Part 1: Dead Code Inventory

### 1.1 Fully Dead Files

| File | LOC | Status |
|------|-----|--------|
| `terminal_spawn.rs` | 52 | Declared in lib.rs, never called. Debug console lists `spawn`/`terminals` commands but no handler exists. |
| `ritual.rs` | 0 | **Does not exist.** Summary claims it does — false. |
| `structures/algorithms/bsp.rs` | ~350 | Zero usage in game or tests. |
| `structures/algorithms/maze.rs` | ~300 | Zero usage in game or tests. |
| `structures/algorithms/voronoi.rs` | ~250 | Zero usage in game or tests. |
| `structures/algorithms/wave_function_collapse.rs` | ~400 | Zero usage in game or tests. |

### 1.2 Dead Methods in state.rs (15 confirmed)

10 narrative methods (never called from anywhere):
`get_area_description`, `add_story_event`, `story_model_mut`, `world_history_mut`, `generate_npc_backstories`, `get_character_relationships`, `get_world_history`, `get_artifact_inscription`, `generate_flavor_text`, `get_shrine_text`

5 other dead methods:
`calculate_price` (trading), `tutorial_progress_mut`, `decoy_at` (combat), `visible_adaptation_count`, `get_adaptation_visual_effects`

### 1.3 Dead UI Exports

- `render_map` in ui/mod.rs — superseded by Renderer, never called
- `dim_color` in ui/mod.rs — only used internally in game_view.rs
- `render_inventory_bar` in ui/mod.rs — never called

### 1.4 Orphaned Data Artifacts

**11 orphaned schemas** (no matching data file):
- 7 deprecated (data merged/renamed): aria_dialogues, floors, walls, lights, effects_config, skills, psychic_abilities, status_effects
- 3 mismatched names: spawn_tables (→biome_spawn_tables), save_meta (runtime-only), enemies (split into subdirectory — actually valid)
- 1 truly unused: structures_unified

**7 orphaned pattern files** — entire `data/structures/patterns/special/` directory contains exact duplicates of files in `patterns/ruins/`.

**5 data files without schemas**: biome_spawn_tables, environmental_props, main_questline, skill_trees, traders.

**`structure_generation.json`** — only loaded by deprecated tilegen-tool binary, not by the game.

### 1.5 Dead Renderer Code

`ViewportCuller` in renderer/performance.rs — instantiated but result assigned to `_viewport_bounds`. `is_in_bounds()` never called.

---

## Part 2: Broken / Half-Wired Systems

These are the most concerning findings — real implementations that appear functional but don't work end-to-end.

### 2.1 Light Manipulation (313 LOC + UI)

- **What works**: `update()` runs every turn, energy gained from items, menu renders with 'g' key
- **What's broken**: Menu has NO input handler — display only. None of the 10+ ability methods (focus_beam, create_prism, absorb_light, trace_beam) are callable from gameplay.
- **Break point**: No `handle_light_menu_input` function exists. No `UseLightAbility` action variant.

### 2.2 Crystal Resonance (376 LOC + UI)

- **What works**: `update()` runs, crystals added during mapgen, energy from items, menu renders with 'V' key
- **What's broken**: Menu only handles Esc/Up/Down — Enter does nothing. 7+ ability methods unreachable.
- **Break point**: No Enter handler in `handle_crystal_menu_input()`. No `UseCrystalAbility` action variant.

### 2.3 Void Energy (318 LOC + UI)

- **What works**: Full UI pipeline — menu opens, Enter dispatches `Action::UseVoidAbility`, energy deducted
- **What's broken**: `use_ability()` only handles PhaseWalk internally, but `can_phase_walk()` is never checked in movement code. Other 4 abilities (VoidStep, RealityRend, VoidShield, VoidDrain) hit `_ => {}` — energy spent, nothing happens.
- **Break point**: Movement system doesn't check `can_phase_walk()`. Other ability match arms are empty.

### 2.4 Psychic Abilities (144 LOC + UI)

- **What works**: Full pipeline with cooldowns, data-driven from abilities.json
- **What's broken**: Only 3 hardcoded effect IDs work (stun_aoe, guaranteed_hit, phasing). Everything else logs "Effect not implemented."
- **Break point**: Effect dispatch is a hardcoded match, not data-driven despite the data-driven loading.

### 2.5 Fake DES Scenarios

All three special system scenarios (`crystal_resonance_basic.json`, `void_energy_basic.json`, `light_manipulation_basic.json`) are **byte-for-byte identical** — spawn player, wait 1 turn, assert `player_alive`. They test nothing about the actual systems. They provide false confidence.

---

## Part 3: Architectural Weaknesses

### 3.1 state.rs God Object

- **3,525 LOC**, 163 methods, 16 distinct concerns
- 25+ sibling module imports
- 4 files extend it with `impl GameState` blocks (combat_actions.rs, inspect.rs, qa_tools.rs, state.rs itself)
- 22 of 50 game modules have zero reverse dependencies — only consumed by state.rs
- The god object makes it impossible to tell what's connected without tracing every call chain

### 3.2 Dual Generation System

Two parallel generation systems that don't interact:
- **System A (active)**: terrain-forge pipeline via `terrain_forge_adapter.rs` — handles all game terrain
- **System B (dead)**: Custom `StructureGenerator` trait with 7 algorithm implementations (~45KB) — only used in deprecated tilegen-tool and 3 test files

The custom system was the original approach, superseded by terrain-forge. The adapter function `generate_with_dungeon_generator()` is misleadingly named — it calls terrain-forge, not the custom DungeonGenerator.

### 3.3 No Algorithm Registry

The summary claims `generation/registry.rs` provides a plugin system. **This file does not exist.** Algorithm selection happens via string names in `terrain_forge_adapter.rs` dispatched to the external crate.

### 3.4 Schema Lifecycle Gap

- `schema_gen.rs` uses a manual type list (only 7 types registered)
- No CI step runs schema_gen or checks schema-data consistency
- No cleanup mechanism when data files are renamed/merged/split
- Bidirectional problem: orphaned schemas AND unvalidated data files

### 3.5 Summary Documentation Drift

The `.agents/summary/` files contain multiple false claims:
- `ritual.rs` exists (it doesn't)
- Algorithm registry exists (it doesn't)
- All special systems are functional (they're half-wired)
- 7 structure algorithms are selectable (none are used in game)

---

## Part 4: Root Cause — The "Scaffold and Abandon" Anti-Pattern

### 4.1 Git Forensics

The dead code was not from planned features that lost momentum. It's a systematic pattern of AI-generated batch scaffolding:

**Three special systems** — single commit `1e40f72` (2026-01-02):
- "Implement missing systems from content expansion"
- 12 files, 1,593 insertions — light.rs, crystal_resonance.rs, void_energy.rs all in one shot
- Commit message: "All systems integrated into GameState and game loop" — partially true at best

**Four dead algorithms** — single commit `4316e7c` (2026-01-07):
- "feat: add comprehensive algorithm library for procedural generation"
- 12 files, 1,538 insertions — 8 algorithms in one commit
- None were ever called from the game pipeline

**Narrative dead methods** — two commits on same day (2025-12-24):
- "Implement Procedural Narrative System inspired by Caves of Qud" — 1,450 insertions
- Methods added to state.rs but never called from the game loop

**UI menus** — added later in audit commit `acd82d7` (2026-02-14):
- "feat: maintainability audit phases 3-4, event system expansion & feature integration"
- Added void_menu.rs, crystal_menu.rs, light_menu.rs with keybindings

### 4.2 The Pattern

```
1. Feature request or design doc describes a system
2. AI generates complete vertical slice (struct + methods + update + UI + tests + docs)
3. Code compiles, trivial tests pass, commit is made
4. Last-mile integration never happens (input dispatch → game effect → movement/combat)
5. Next feature request arrives, repeat
```

The gap is always at the **horizontal integration boundary** — where the new system needs to modify existing complex code (input.rs, state.rs, movement systems). The vertical slice is easy to generate; threading through existing code is hard.

### 4.3 Telltale Signs

- Massive batch commits (1,500+ insertions, multiple complete systems)
- Identical boilerplate DES tests that assert nothing meaningful
- Commit messages that overstate integration ("All systems integrated")
- 42 of 468 commits (9%) have 1,000+ insertions

---

## Part 5: Module Structure Analysis

### 5.1 Natural Clusters (from dependency graph analysis)

| Cluster | Files | LOC | Boundary Quality | Notes |
|---------|-------|-----|-----------------|-------|
| A: Map & Terrain | 5 | ~1,218 | Excellent | Pure data types, no GameState dependency |
| B: Generation | 49 | ~14,743 | Excellent | Already isolated, only 2 entry points touch GameState |
| C: Entity Definitions | 7 | ~1,500 | Good | Pure data + DataLoader, leaf modules |
| D: Combat | 4 | ~1,384 | Moderate | combat_actions.rs has `impl GameState` |
| E: Narrative | 6 | ~1,955 | Moderate | encounter.rs deeply woven into state.rs |
| F: Character/Progression | 7 | ~1,562 | Good | Mostly leaf modules, 0 importers |
| G: Environment | 4 | ~552 | Good | Clean data types |
| H: Infrastructure | 10 | ~1,603 | Mixed | data_loader is foundational; others are grab-bag |
| I: Debug/QA | 4 | ~894 | Poor | Tightly coupled to GameState internals |
| J: Economy | 2 | ~421 | Good | Self-contained leaf modules |

### 5.2 Key Structural Observations

- **Star topology**: state.rs is the only hub. Almost nothing cross-imports siblings directly.
- **22 leaf modules**: Only consumed by state.rs — they define types/functions, state.rs orchestrates.
- **world_map types are the true foundation**: Biome, POI, Terrain enums have 10 importers — most cross-cutting types.
- **generation/ is already well-isolated**: 14,743 LOC with only 2 GameState touch points.
- **`impl GameState` sprawl**: 4 files extend GameState, creating structural coupling that prevents extraction.

---

## Part 6: Areas Requiring Further Investigation

### 6.1 state.rs Internal Dependency Graph (NOT YET DONE)

We know state.rs has 163 methods across 16 concerns, but we haven't mapped which methods call which other methods internally. This is critical for decomposition planning:
- Which concern categories have zero cross-dependencies? (safe to extract)
- Which methods are "hub methods" touching 3+ concerns? (must stay central or become orchestrators)
- Are there circular call chains within state.rs?

### 6.2 The encounter.rs / state.rs Coupling

encounter.rs is referenced 15 times from state.rs — the heaviest dependency after world_map. Need to understand:
- What does encounter logic do inside state.rs?
- Can encounter resolution be extracted into a system?
- Is the coupling structural or incidental?

### 6.3 combat_actions.rs `impl GameState` Block

This file extends GameState with combat action methods. Need to understand:
- How many methods does it add?
- Do they access fields from multiple concerns, or just combat-related state?
- Could they become free functions taking specific sub-states instead of &mut GameState?

### 6.4 DES Scenario Coverage Gaps

We know 3 scenarios are fake. But what's the overall coverage picture?
- How many DES scenarios exercise actual gameplay effects vs just asserting player_alive?
- Are there systems with zero meaningful test coverage?
- What would a "minimum viable DES scenario" look like for each system?

### 6.5 Data Cross-Reference Integrity

The summary shows a data dependency graph (items→traders, enemies→spawn_tables, etc.). Need to verify:
- Are all cross-references valid? (e.g., does every item_id in traders.json exist in items.json?)
- Are there dangling references?
- Is there a validation step for this, or is it manual?

### 6.6 The 3 Test-Only Algorithms

cellular_automata, drunkard_walk, and simple_rooms are used in test files but not in the game. Need to decide:
- Are these regression tests for terrain-forge's equivalent algorithms?
- Or are they testing the custom implementations that nothing uses?
- If the latter, the tests themselves are dead weight.

### 6.7 Narrative Subsystem Intent

10 dead narrative methods suggest a planned feature. Before deleting:
- What was the design vision? (check docs/narrative/, commit messages)
- Is narrative generation still on the roadmap?
- Should the methods be preserved behind a feature flag, or removed entirely?

---

## Part 7: Structural Recommendations (Draft)

These are preliminary. Final recommendations depend on the investigations in Part 6.

### 7.1 Integration Gates (Prevent Future Scaffolding)

The core problem: code can be committed without proving it connects to gameplay. Proposed gates:

1. **Meaningful DES requirement**: Every new system must have a DES scenario that exercises the actual gameplay effect — not just `player_alive`. For combat abilities: assert damage dealt. For movement abilities: assert position changed. For resource systems: assert resource consumed AND effect applied.

2. **"Wired" checklist in PR template**: Before merging a new system, verify:
   - [ ] Input action variant exists and is dispatched
   - [ ] Effect is observable in game state (not just energy deducted)
   - [ ] At least one DES scenario asserts the observable effect
   - [ ] No `_ => {}` catch-all in ability dispatch

3. **Batch commit size limit**: Flag commits with >500 insertions for review. Not a hard block, but a signal that integration may be incomplete.

### 7.2 state.rs Decomposition (Reduce Coupling Surface)

The god object makes it hard to see what's connected. Proposed approach:

1. **Extract foundational types first** (Tier 1 from cluster analysis): Map, Tile, Biome, POI, Terrain into a `types` module. Zero risk, high value — these are pure data types with no GameState dependency.

2. **Extract sub-states with accessor traits**: Instead of one GameState with 163 methods, define sub-state structs (CombatState, NarrativeState, ProgressionState) that own their fields and methods. GameState becomes a composition of sub-states with thin delegation.

3. **Eliminate `impl GameState` sprawl**: combat_actions.rs, inspect.rs, qa_tools.rs should become free functions or methods on sub-states, not extensions of GameState.

4. **Defer the hard parts**: encounter.rs coupling and hub methods (end_turn, etc.) are the hardest to extract. Do them last, after the easy extractions prove the pattern works.

### 7.3 Dead Code Policy

Rather than a one-time cleanup, establish ongoing policy:

1. **Immediate**: Delete `patterns/special/` (duplicates), `terminal_spawn.rs` (dead), 4 dead algorithms with zero test usage (bsp, maze, voronoi, wfc).
2. **Mark explicitly**: Add `#[deprecated(note = "Not wired to gameplay — see CODEBASE_HEALTH_AUDIT.md")]` to light, crystal, void ability methods. This makes the status visible to anyone reading the code.
3. **Schema cleanup**: Delete the 7 deprecated schemas. Add a CI check that warns on schema-data mismatches.
4. **Decide, don't defer**: For each half-wired system, make an explicit decision: finish wiring it, or remove it. Don't leave it in limbo.

### 7.4 Generation System Consolidation

1. **Remove the custom StructureGenerator trait and 7 algorithm implementations** — terrain-forge has superseded them.
2. **Rename `generate_with_dungeon_generator()`** to reflect that it uses terrain-forge.
3. **Keep the 3 test-only algorithms only if** investigation 6.6 confirms they test something meaningful. Otherwise delete.
4. **Remove `structure_generation.json`** and the deprecated tilegen-tool, or update them to use terrain-forge.

### 7.5 Summary Documentation Accuracy

The `.agents/summary/` files need correction. But more importantly, they need a maintenance strategy:
- Tie summary updates to the same PR that changes the code
- Add a "last verified" date to each summary file
- Consider generating parts of the summary from code analysis (e.g., module list, dependency counts)

---

## Part 6: Investigation Results (Completed)

### 6.1 state.rs Internal Dependency Graph

**Hub methods by fan-out:**
1. `end_turn` (fan-out 11) — turn → status → AI → storm → time → lighting → FOV → encounter → XP → events
2. `travel_to_tile` (fan-out 10, 4 are dead stubs) — generation → quest NPCs → FOV → spatial → lighting → crystal → narrative stubs
3. `use_item` (160 LOC, touches 10+ concerns inline) — AP, HP, refraction, adaptation, map reveal, ARIA, light/void/crystal energy, events, inventory. **Worst cross-concern offender.**
4. `move_on_world_map` / `travel_to_tile_safe` (fan-out 5 each) — travel + encounter wrappers
5. `process_events` (fan-out 3) — drain → LootSystem → QuestSystem → handle_event

**API surface:** 141 pub, 1 pub(crate), 21 private. 16 pub methods are dead (zero callers). 9 pub methods should be private (only called internally).

**Key finding:** The event system (`emit`/`process_events`) is already a clean boundary. More systems should use it instead of direct method calls — this would decouple `use_item` and `end_turn`.

### 6.2 encounter.rs / state.rs Coupling

encounter.rs is actually well-designed — pure functions taking explicit parameters, not `&mut GameState`. The 79 references in state.rs are field accesses (`encounter_state`, `encounter_history`) and calls to these pure functions. The coupling is in state.rs's orchestration code (`move_on_world_map`, `travel_to_tile_safe`, `check_encounter_completion`, `spawn_encounter_entities`), not in encounter.rs itself. This orchestration could become an `EncounterSystem` in `systems/`.

### 6.3 combat_actions.rs `impl GameState`

Only 95 LOC, 7 methods. 4 thin delegators, 2 pure reads, 1 cross-concern method (`try_break_wall` at 42 LOC touching inventory + AP + map + turn). Not a major coupling problem — organizational, not structural.

### 6.4 DES Scenario Coverage

162 files total: 101 good, 48 setup-only, 7 fake (wait + player_alive), 2 dead .des files.

**Systems with zero real coverage:** sanity, ritual, save/load, encounter, interactable, meta-progression, crystal resonance, void energy, light manipulation, FOV, narrative engine.

**Dangerously thin coverage (1-2 scenarios):** crafting, movement, skills, trading.

**Fake scenarios** (identical boilerplate, test nothing): crystal_resonance_basic, void_energy_basic, light_manipulation_basic, enhanced_enemy_systems_test, fov_system_test, narrative_system_test, story_model_test.

### 6.5 Data Cross-Reference Integrity

18 dangling references total:
- Traders → Items: **clean** (0 dangling)
- Spawn tables → Enemies: **clean** (0 dangling)
- Loot tables → Items: **2 dangling** (angle_split_lens, prism_shard)
- Spawn tables → Items: **16 dangling** (ancient_gear, cactus_water, crystalline_shard, dried_herbs, healing_herb, prism_shard, etc.)

No runtime validation exists for cross-references. DataLoader validates schema structure but not referential integrity.

### 6.6 Test-Only Algorithms

The 3 test-only algorithms (cellular_automata, drunkard_walk, simple_rooms) are tested by smoke tests that only verify `generate()` returns non-empty output. They test the **custom** implementations, not terrain-forge's equivalents. `des_scenarios.rs` also has 2 pure Rust tests for BSP and CellularAutomata (same smoke pattern). These tests are dead weight — they validate code the game never uses.

### 6.7 Narrative Subsystem Design Intent

Three disconnected layers:
1. **narrative_engine.rs** (130 LOC) — state container with stub methods. QuestLog.on_* are all no-ops. complete() returns hardcoded rewards.
2. **generation/narrative.rs** (535 LOC) + **narrative_templates.rs** (387 LOC) — real Markov chain + template generation code, but only used in `generation/tests.rs`. Never called from the game pipeline.
3. **10 dead methods in state.rs** — were supposed to bridge state → generation. All say "Removed: generation systems not yet re-implemented."

The generation code exists and works (per unit tests) but was never wired into the game loop. Total: ~1,050 LOC exercised only by unit tests.

---

## Part 6.5: Meta-Level Computational Taxonomy

Each module classified by the abstract problem it solves, independent of game domain. Inputs/outputs specified — "mutated game state" is broken down into which specific state facets change.

### Pattern Tags

| Tag | Description |
|-----|-------------|
| `DATA-DEF` | Type definitions + JSON deserialization. No behavior. |
| `DATA-XFORM` | Deterministic transformation pipeline: input data → output data |
| `STATE-ORCHESTRATOR` | Coordinates mutations across multiple state facets |
| `DECISION-FN` | Pure function: parameters → decision/result, no side effects |
| `EVENT-ROUTER` | Dispatches events to handlers that produce side effects |
| `TICK-SYSTEM` | Per-turn update on a single concern |
| `RENDER-COMPOSE` | Reads state, produces visual output |
| `INPUT-DISPATCH` | Maps user input → action enum |
| `SPATIAL-ALGO` | Computes spatial relationships (visibility, paths, regions) |
| `WEIGHTED-SELECT` | Chooses from weighted options using RNG |
| `RESOURCE-ACCUM` | Tracks and modifies numeric resources |
| `PERSISTENCE` | Serialization / deserialization of state |

### Module Classification

#### Foundation Layer (no GameState dependency)

| Module | Pattern | Input | Output |
|--------|---------|-------|--------|
| map.rs | `DATA-DEF` + `SPATIAL-ALGO` | Tile grid, position | FOV set, pathfinding result, tile lookups |
| world_map.rs | `DATA-DEF` | — | Biome, POI, Terrain enums (used by 10+ modules) |
| constants.rs | `DATA-DEF` | — | MAP_WIDTH, MAP_HEIGHT |
| entity.rs | `DATA-DEF` | — | Entity trait, position types |
| map_elements.rs | `DATA-DEF` | JSON | Wall/floor/light definitions |
| map_features.rs | `DATA-DEF` | JSON | Feature definitions |

#### Data Loading Layer

| Module | Pattern | Input | Output |
|--------|---------|-------|--------|
| data_loader.rs | `DATA-XFORM` | JSON file path + schema | Typed, validated, cached data |
| enemy.rs | `DATA-DEF` | JSON (5 files) | EnemyDef lookup by ID |
| item.rs | `DATA-DEF` | JSON | ItemDef lookup by ID |
| npc.rs | `DATA-DEF` | JSON | NpcDef lookup by ID |
| chest.rs | `DATA-DEF` | JSON | ChestDef lookup |
| interactable.rs | `DATA-DEF` | JSON | InteractableDef lookup |
| effect.rs | `DATA-DEF` | JSON | Effect/StatusEffect definitions |
| status.rs | `DATA-DEF` | — | StatusEffect types |
| action.rs | `DATA-DEF` | JSON | AP cost lookup by action name |
| keyboard_config.rs | `DATA-DEF` | JSON | Key binding lookup |

#### Decision / Pure Function Layer

| Module | Pattern | Input | Output |
|--------|---------|-------|--------|
| combat.rs | `DECISION-FN` | Attacker stats, defender stats, RNG | CombatResult (hit, damage, status) |
| encounter.rs | `DECISION-FN` + `WEIGHTED-SELECT` | World position, danger, travel count, RNG | bool (trigger?), EncounterState |
| adaptation.rs | `DECISION-FN` | Adaptation list | Stat modifiers, immunities, abilities |
| progression.rs | `DECISION-FN` | XP, level | Level-up threshold, stat gains |
| travel.rs | `DECISION-FN` | World positions, biome | Travel cost, adjacency check |

#### Generation Pipeline (DATA-XFORM, largest subsystem)

| Module | Pattern | Input | Output |
|--------|---------|-------|--------|
| tile_generator.rs | `STATE-ORCHESTRATOR` | Seed, biome, POI, config | Complete tile map with entities |
| terrain_forge_adapter.rs | `DATA-XFORM` | Seed, biome profile, algorithm layers | Raw terrain grid |
| connectivity.rs | `SPATIAL-ALGO` | Map with regions | Connected map (tunnels carved) |
| settlement/* | `DATA-XFORM` + `WEIGHTED-SELECT` | Seed, tier, structures | Settlement layout (buildings, roads) |
| spawn.rs | `WEIGHTED-SELECT` | Biome, level, spawn tables, RNG | Entity positions + types |
| loot.rs (generation) | `WEIGHTED-SELECT` | Loot table, RNG | Item list |
| world_gen.rs | `DATA-XFORM` | Seed, config | World map grid |
| microstructures.rs | `DATA-XFORM` | Map, biome | Map with small features added |
| environmental_props.rs | `DATA-XFORM` | Map, biome, config | Map with floor decorations |
| structure_library.rs | `DATA-XFORM` | Structure patterns, map | Map with prefabs stamped |
| constraints.rs | `DECISION-FN` | Generated map, rules | Satisfaction score, pass/fail |
| narrative.rs | `DATA-XFORM` | Templates, context, RNG | Generated text fragments |
| narrative_templates.rs | `DATA-XFORM` | Template defs, variables | Filled text strings |

#### Tick Systems (per-turn, single concern)

| Module | Pattern | Input | Mutated State Facet |
|--------|---------|-------|---------------------|
| systems/status.rs | `TICK-SYSTEM` | Enemy/player status lists | HP (damage ticks), status durations |
| systems/storm.rs | `TICK-SYSTEM` + `DATA-XFORM` | Storm state, map | Map tiles (7 edit types), enemy spawns |
| systems/quest.rs | `EVENT-ROUTER` | GameEvent | Quest progress counters |
| systems/loot.rs | `EVENT-ROUTER` | EnemyKilled event | Item spawns on map |
| systems/ai.rs | `DECISION-FN` + `STATE-ORCHESTRATOR` | Enemy positions, player position, behaviors | Enemy positions, attack actions |
| systems/combat.rs | `DECISION-FN` + `STATE-ORCHESTRATOR` | Attack params | HP (damage), status effects, events emitted |
| systems/movement.rs | `STATE-ORCHESTRATOR` | Move direction | Player position, tile effects, NPC triggers, FOV |
| light.rs | `TICK-SYSTEM` + `RESOURCE-ACCUM` | Light system state | Beam decay, energy ticks |
| crystal_resonance.rs | `TICK-SYSTEM` + `RESOURCE-ACCUM` | Crystal system state | Crystal timers, energy ticks |
| void_energy.rs | `TICK-SYSTEM` + `RESOURCE-ACCUM` | Void system state | Exposure decay, energy ticks |
| psychic.rs | `TICK-SYSTEM` | Psychic state | Cooldown timers |

#### State Orchestration (cross-concern coordination)

| Method | Pattern | Input | Mutated State Facets |
|--------|---------|-------|---------------------|
| end_turn | `STATE-ORCHESTRATOR` | — | Status, AI, storm, time, lighting, FOV, encounters, XP, events |
| travel_to_tile | `STATE-ORCHESTRATOR` | World position | Map (regenerated), entities, FOV, spatial index, lighting, crystals |
| use_item | `STATE-ORCHESTRATOR` | Item ID | HP, AP, refraction, adaptations, FOV, light/void/crystal energy, inventory, events |
| move_on_world_map | `STATE-ORCHESTRATOR` | Direction | World position, encounters, map (via travel_to_tile) |
| complete_quest | `STATE-ORCHESTRATOR` | Quest ID | Quest log, XP, currency, inventory, faction reputation |

#### Rendering Layer

| Module | Pattern | Input | Output |
|--------|---------|-------|--------|
| renderer/mod.rs | `RENDER-COMPOSE` | GameState, Camera, Frame | Composed terminal frame |
| renderer/tiles.rs | `RENDER-COMPOSE` | Map, theme, lighting | Tile spans |
| renderer/entities.rs | `RENDER-COMPOSE` | Entities, lighting | Entity spans |
| renderer/lighting.rs | `RENDER-COMPOSE` + `SPATIAL-ALGO` | Light sources, map | Light intensity grid |
| renderer/particles.rs | `TICK-SYSTEM` + `RENDER-COMPOSE` | Particle state | Updated particles + spans |
| renderer/animations.rs | `TICK-SYSTEM` + `RENDER-COMPOSE` | Animation state | Updated animations + style modifiers |
| renderer/procedural.rs | `TICK-SYSTEM` + `RENDER-COMPOSE` | Weather state | Weather particle spans |

#### UI Layer

| Module | Pattern | Input | Output |
|--------|---------|-------|--------|
| ui/input.rs | `INPUT-DISPATCH` | Keypress, UI state | Action enum |
| ui/game_view.rs | `RENDER-COMPOSE` | GameState | Damage numbers, death screen, debug console |
| ui/hud.rs | `RENDER-COMPOSE` | GameState | Side panel, bottom panel |
| ui/*_menu.rs (×15) | `INPUT-DISPATCH` + `RENDER-COMPOSE` | GameState, keypress | Menu frame + Action |

#### Infrastructure

| Module | Pattern | Input | Output |
|--------|---------|-------|--------|
| save.rs | `PERSISTENCE` | GameState ↔ JSON file | Saved/loaded state |
| event.rs | `DATA-DEF` | — | GameEvent enum |
| ipc.rs | `PERSISTENCE` (streaming) | GameState → socket | IPC messages to satellites |
| debug_commands.rs | `INPUT-DISPATCH` + `STATE-ORCHESTRATOR` | Command string | Debug state mutations |

### Observations from the Taxonomy

**1. state.rs is the only `STATE-ORCHESTRATOR` in the codebase.** Every cross-concern coordination goes through it. The systems/ modules are `TICK-SYSTEM` or `EVENT-ROUTER` — they handle single concerns. Nobody else orchestrates.

**2. The `DECISION-FN` modules are the cleanest.** encounter.rs, combat.rs, adaptation.rs, progression.rs, travel.rs — all pure functions with explicit inputs/outputs. These are the easiest to test and the least likely to accumulate dead code.

**3. The `DATA-DEF` modules are leaf nodes.** 22 modules with zero reverse dependencies. They define types and load data. They don't need GameState.

**4. The generation pipeline is a clean `DATA-XFORM` chain** — except for tile_generator.rs which is a `STATE-ORCHESTRATOR` (it writes entities into GameState). If tile_generator returned a struct instead of mutating state, the entire generation pipeline would be pure.

**5. The half-wired systems (light, crystal, void) are `TICK-SYSTEM` + `RESOURCE-ACCUM`** — they tick correctly but their `DECISION-FN` abilities are never invoked because no `INPUT-DISPATCH` → `STATE-ORCHESTRATOR` path exists for them.

**6. The event system is underused.** Only `LootSystem` and `QuestSystem` use `EVENT-ROUTER`. Combat, movement, item use, and encounter resolution all use direct method calls through the `STATE-ORCHESTRATOR`. If `use_item` emitted events instead of calling 10+ methods directly, each concern could react independently.

**7. Two patterns dominate the dead code:**
- `DATA-XFORM` code that was never connected to a `STATE-ORCHESTRATOR` (narrative generation, custom algorithms)
- `RESOURCE-ACCUM` systems whose `DECISION-FN` abilities lack an `INPUT-DISPATCH` path (light, crystal, void)

**8. Potential architectural patterns suggested by the taxonomy:**
- **Command pattern** for `STATE-ORCHESTRATOR` methods — `use_item` becomes a command that emits effects, not a method that calls 10 things
- **Event sourcing** for cross-concern coordination — `end_turn` emits phase events, systems react
- **Pipeline pattern** for generation — tile_generator returns data, caller writes to state
- **Strategy pattern** already used well in systems/ai.rs — could extend to ability dispatch

---

## Appendix: Quantified Impact

| Category | Dead/Broken LOC | Files Affected |
|----------|----------------|----------------|
| Dead algorithms (4 fully dead) | ~1,300 | 4 |
| Dead algorithms (3 test-only) | ~900 | 3 |
| Half-wired special systems | ~1,007 | 3 (+3 UI files) |
| Dead state.rs methods | ~300 est. | 1 |
| Dead UI exports | ~50 est. | 1 |
| Orphaned schemas | 11 files | 11 |
| Orphaned pattern files | 7 files | 7 |
| Dead terminal_spawn.rs | 52 | 1 |
| **Total estimated dead/broken** | **~3,600 LOC + 19 files** | |
