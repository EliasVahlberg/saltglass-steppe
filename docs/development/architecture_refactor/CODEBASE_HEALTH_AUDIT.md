# Codebase Health Audit — 2026-04-03 (Updated 2026-04-04)

## Executive Summary

> **Update 2026-04-04:** Phase 0.5 dead code cleanup has been completed. Technical/design documentation has been triaged and archived. This audit has been updated to reflect the current codebase state. Items marked ✅ CLEANED have been resolved. Remaining items are listed under "Remaining Issues."

**Original finding:** AI-generated feature scaffolding that is vertically complete but horizontally disconnected from gameplay. ~3,600 LOC of dead/half-wired code. Root cause: absence of structural gates that catch incomplete integration before code is committed.

**Post-cleanup state:** 150 source files, 43,822 LOC (down from ~164 files, ~48,500 LOC). state.rs reduced from 3,525 LOC / 163 methods to 3,185 LOC / 135 methods. Orphaned schemas cleaned. Dead algorithms, dead methods, dead stubs, dead UI exports, ViewportCuller all removed. Structural gates added: `SYSTEM_STATUS.md` registry, commit policy in AGENTS.md and steering, documentation triage completed.

**Remaining issues:** 7 duplicate pattern files, 7 fake DES scenarios, 2 dead .des files, 5 data files without schemas, 18 dangling data cross-references, empty `algorithms/mod.rs`.

---

## Part 1: Dead Code Inventory

### 1.1 Fully Dead Files — ✅ CLEANED

| File | LOC | Status |
|------|-----|--------|
| `terminal_spawn.rs` | 52 | ✅ Deleted |
| `structures/algorithms/bsp.rs` | ~350 | ✅ Deleted |
| `structures/algorithms/maze.rs` | ~300 | ✅ Deleted |
| `structures/algorithms/voronoi.rs` | ~250 | ✅ Deleted |
| `structures/algorithms/wave_function_collapse.rs` | ~400 | ✅ Deleted |
| `structures/algorithms/cellular_automata.rs` | ~300 | ✅ Deleted (test-only) |
| `structures/algorithms/drunkard_walk.rs` | ~300 | ✅ Deleted (test-only) |
| `structures/algorithms/simple_rooms.rs` | ~300 | ✅ Deleted (test-only) |
| `structure_generation.json` | — | ✅ Deleted |

**Remaining:** `structures/algorithms/mod.rs` is now empty (0 bytes). Should be deleted along with the `algorithms/` directory.

**Correction from original audit:** `ritual.rs` was listed as "does not exist, summary claims it does." This was a summary documentation error, not dead code. Summary files have been corrected.

### 1.2 Dead Methods in state.rs — ✅ CLEANED

All 15 dead methods removed. 4 dead end_turn stubs removed. state.rs now has 118 pub + 17 private methods (down from 163 total).

### 1.3 Dead UI Exports — ✅ CLEANED

`render_map`, `dim_color`, `render_inventory_bar` removed.

### 1.4 Dead Renderer Code — ✅ CLEANED

`ViewportCuller` removed.

### 1.5 Orphaned Data Artifacts — PARTIALLY CLEANED

**Orphaned schemas:** ✅ All 11 orphaned schemas deleted. All 36 remaining schemas now have matching data files.

**Orphaned pattern files:** ❌ Still present. `data/structures/patterns/special/` contains 7 files that are byte-for-byte duplicates of files in `patterns/ruins/`: glass_foundry.txt, monastery_ascending_light.txt, new_heliograph_station.txt, nexus_plateau.txt, prism_cathedral_natural.txt, prism_city.txt, salt_harbor.txt.

**Data files without schemas:** ❌ Still 5: biome_spawn_tables, environmental_props, main_questline, skill_trees, traders.

### 1.6 Remaining Dead Code

| Item | Status | Notes |
|------|--------|-------|
| `structures/algorithms/mod.rs` | Empty file (0 bytes) | Delete with directory |
| `patterns/special/` (7 files) | Duplicates of `patterns/ruins/` | Delete directory |
| 7 fake DES scenarios | Still exist | crystal_resonance_basic, void_energy_basic, light_manipulation_basic, enhanced_enemy_systems_test, fov_system_test, narrative_system_test, story_model_test |
| 2 dead .des files | Still exist | skill_progression_test.des, faction_system_test.des |

### 1.7 Audit Corrections

Items originally flagged as dead that are actually used:

| Item | Original Assessment | Actual Status |
|------|-------------------|---------------|
| `generation/narrative.rs` (535 LOC) | Dead — never called from game pipeline | **Used** by `map.rs::generate_area_description()` for area descriptions |
| `generation/narrative_templates.rs` (387 LOC) | Dead — tests only | **Used** via narrative.rs → map.rs |
| `structures/dungeon_generator.rs` (236 LOC) | Dead — custom system superseded | **Used** by `terrain_forge_adapter.rs` for POI-specific dungeon generation |
| `structures/ruins_generator.rs` (116 LOC) | Dead — custom system superseded | Used by dungeon_generator.rs |

## Part 2: Broken / Half-Wired Systems — PARTIALLY RESOLVED

> **Update 2026-04-04:** Decision made to remove ability methods from light/crystal/void, keeping resource accumulation only. Psychic abilities remain partially wired (3 of N effects work). See `docs/development/SYSTEM_STATUS.md` for the full wiring status of all systems.

### 2.1 Light Manipulation — RESOLVED (ability methods removed)

- **Previous:** 313 LOC, menu with no input handler, 10+ unreachable ability methods
- **Current:** 85 LOC. Resource accumulation only (`LightSystem` tracks light level). Ability methods deleted. Menu still renders ('g' key) but is display-only by design.
- **Decision:** Resource tracking kept for future use. Abilities will be re-implemented properly when the system is designed with full wiring.

### 2.2 Crystal Resonance — RESOLVED (ability methods removed)

- **Previous:** 376 LOC, menu Enter does nothing, 7+ unreachable ability methods
- **Current:** 151 LOC. Resource tracking + crystal placement on map. Ability methods deleted.
- **Decision:** Same as light — resource tracking kept, abilities deferred.

### 2.3 Void Energy — RESOLVED (ability methods removed)

- **Previous:** 318 LOC, PhaseWalk unchecked in movement, 4 abilities hit `_ => {}`
- **Current:** 196 LOC. Resource accumulation only. All ability methods deleted including the broken PhaseWalk.
- **Decision:** Same as light/crystal.

### 2.4 Psychic Abilities — UNCHANGED (partially wired)

- **What works:** Full pipeline with cooldowns, data-driven from abilities.json
- **What's broken:** Only 3 hardcoded effect IDs work (stun_aoe, guaranteed_hit, phasing). Everything else logs "Effect not implemented."
- **Status:** ⚠️ Partially wired. Kept because the 3 working effects are functional in gameplay.

### 2.5 Fake DES Scenarios — NOT CLEANED

All 7 fake scenarios still exist:
- `crystal_resonance_basic.json`, `void_energy_basic.json`, `light_manipulation_basic.json` — identical boilerplate (spawn, wait, assert player_alive)
- `enhanced_enemy_systems_test.json`, `fov_system_test.json`, `narrative_system_test.json`, `story_model_test.json` — test nothing meaningful

2 dead `.des` files also remain: `skill_progression_test.des`, `faction_system_test.des`.

---

## Part 3: Architectural Weaknesses

### 3.1 state.rs God Object

- **3,185 LOC** (down from 3,525), **135 methods** (118 pub + 17 private, down from 163)
- Still 16 distinct concerns, still the sole STATE-ORCHESTRATOR
- 4 files extend it with `impl GameState` blocks (combat_actions.rs, inspect.rs, qa_tools.rs, state.rs itself)
- The god object remains the primary architectural problem — cleanup reduced its size but not its role

### 3.2 Generation System — PARTIALLY RESOLVED

- **System A (active):** terrain-forge pipeline via `terrain_forge_adapter.rs` — handles all game terrain
- **System B (partially dead):** Custom `StructureGenerator` trait. 7 algorithm implementations deleted. `dungeon_generator.rs` (236 LOC) and `ruins_generator.rs` (116 LOC) remain and ARE used by `terrain_forge_adapter.rs` for POI-specific dungeon generation. `algorithms/mod.rs` is empty and should be deleted.
- **Correction:** The original audit classified the entire custom system as dead. `dungeon_generator.rs` is actively used — `terrain_forge_adapter.rs` calls `generate_with_dungeon_generator()` for POI types that need dungeon layouts.

### 3.3 No Algorithm Registry — CONFIRMED

The summary claim about `generation/registry.rs` was false. This has been corrected in the `.agents/summary/` files.

### 3.4 Schema Lifecycle Gap — PARTIALLY RESOLVED

- Orphaned schemas cleaned (all 36 schemas now have matching data files)
- 5 data files still lack schemas: biome_spawn_tables, environmental_props, main_questline, skill_trees, traders
- No CI step for schema-data consistency (unchanged)

### 3.5 Summary Documentation Drift — RESOLVED

`.agents/summary/` files regenerated after cleanup with corrections documented in `review_notes.md`. False claims about ritual.rs, algorithm registry, and special system functionality corrected.

### 3.6 Structural Gates — NEW (added 2026-04-04)

The following gates have been added to prevent future scaffold-and-abandon:

1. **System Status Registry** (`docs/development/SYSTEM_STATUS.md`): Source of truth for what is wired into gameplay. Overrides `.agents/summary/` claims.
2. **Commit Policy** (in `AGENTS.md` Custom Instructions and `.kiro/steering/tech.md`): Requires proving DES scenario, honest status marking, and registry update before committing new systems.
3. **Documentation Triage**: Technical/design docs triaged and archived. Front-matter with `status`, `last_verified`, `commit` added to current docs.

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

## Part 7: Structural Recommendations — STATUS

### 7.1 Integration Gates — ✅ IMPLEMENTED

1. **System Status Registry:** `docs/development/SYSTEM_STATUS.md` — created, covers all systems.
2. **Commit Policy:** Added to `AGENTS.md` (Custom Instructions) and `.kiro/steering/tech.md`. Requires proving DES scenario, honest status marking, registry update.
3. **Batch commit awareness:** Covered by commit policy ("do not commit batch scaffolding without proving each system is wired").

### 7.2 state.rs Decomposition — PENDING (architecture refactor)

This is the core of the ESCAEV / Elm Architecture refactor. state.rs is still 3,185 LOC with 135 methods across 16 concerns. The cleanup reduced its size but not its structural role as the sole STATE-ORCHESTRATOR. Decomposition approach depends on the architectural direction chosen.

### 7.3 Dead Code Policy — ✅ MOSTLY IMPLEMENTED

1. **Immediate deletions:** ✅ Done (algorithms, terminal_spawn, dead methods, ViewportCuller, orphaned schemas, structure_generation.json).
2. **Half-wired systems decision:** ✅ Done (ability methods removed from light/crystal/void, resource accumulation kept).
3. **Schema cleanup:** ✅ Orphaned schemas deleted. ❌ CI check not yet added.
4. **Remaining:** patterns/special/ duplicates, fake DES scenarios, dead .des files, empty algorithms/mod.rs.

### 7.4 Generation System Consolidation — ✅ MOSTLY DONE

1. **Custom algorithms removed:** ✅ All 7 deleted.
2. **dungeon_generator.rs kept:** Correction — it IS used by terrain_forge_adapter.rs. Not dead.
3. **structure_generation.json removed:** ✅ Deleted.
4. **Empty algorithms/mod.rs:** ❌ Still exists, should be deleted with directory.

### 7.5 Summary Documentation Accuracy — ✅ DONE

`.agents/summary/` files regenerated with corrections. `review_notes.md` documents all corrections. System Status Registry provides ongoing accuracy for gameplay system wiring.

---

## Part 6: Investigation Results (Completed)

### 6.1 state.rs Internal Dependency Graph

**Hub methods by fan-out:**
1. `end_turn` (fan-out 11) — turn → status → AI → storm → time → lighting → FOV → encounter → XP → events
2. `travel_to_tile` (fan-out 10, 4 are dead stubs) — generation → quest NPCs → FOV → spatial → lighting → crystal → narrative stubs
3. `use_item` (160 LOC, touches 10+ concerns inline) — AP, HP, refraction, adaptation, map reveal, ARIA, light/void/crystal energy, events, inventory. **Worst cross-concern offender.**
4. `move_on_world_map` / `travel_to_tile_safe` (fan-out 5 each) — travel + encounter wrappers
5. `process_events` (fan-out 3) — drain → LootSystem → QuestSystem → handle_event

**API surface:** 118 pub, 17 private (down from 141 pub + 1 pub(crate) + 21 private after removing 15 dead pub methods and 4 dead stubs).

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

### 6.6 Test-Only Algorithms — ✅ RESOLVED

All 3 test-only algorithms (cellular_automata, drunkard_walk, simple_rooms) deleted along with the 4 fully dead algorithms. The smoke tests that validated them are also gone. `algorithms/mod.rs` remains as an empty file — should be deleted with the directory.

### 6.7 Narrative Subsystem Design Intent — CORRECTED

> **Update 2026-04-04:** Original audit classified `generation/narrative.rs` and `narrative_templates.rs` as dead. This was incorrect — they are used by `map.rs::generate_area_description()` for procedural area descriptions during map generation. The 10 dead bridge methods in state.rs were correctly identified as dead and have been removed.

Three layers, two of which are active:

1. **narrative_engine.rs** (103 LOC, down from 130) — state container. QuestLog.on_* hooks are functional for quest tracking. `complete()` returns hardcoded rewards. This is a thin orchestration layer, not dead.

2. **generation/narrative.rs** (535 LOC) + **narrative_templates.rs** (387 LOC) — Markov chain + template generation. **Actually used** by `map.rs::generate_area_description()` which is called during map initialization. Not dead.

3. **10 dead bridge methods in state.rs** — ✅ Removed. These were supposed to connect state → generation but were never wired.

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

**5. The half-wired systems (light, crystal, void) are now `RESOURCE-ACCUM` only** — ability methods removed. They tick correctly and accumulate resources. When abilities are re-implemented, they will need `INPUT-DISPATCH` → `STATE-ORCHESTRATOR` paths.

**6. The event system is underused.** Only `LootSystem` and `QuestSystem` use `EVENT-ROUTER`. Combat, movement, item use, and encounter resolution all use direct method calls through the `STATE-ORCHESTRATOR`. If `use_item` emitted events instead of calling 10+ methods directly, each concern could react independently.

**7. Two patterns dominated the dead code (now mostly cleaned):**
- `DATA-XFORM` code that was never connected to a `STATE-ORCHESTRATOR` (custom algorithms — deleted)
- `RESOURCE-ACCUM` systems whose `DECISION-FN` abilities lacked an `INPUT-DISPATCH` path (light, crystal, void — ability methods deleted, resource tracking kept)

**8. Potential architectural patterns suggested by the taxonomy:**
- **Command pattern** for `STATE-ORCHESTRATOR` methods — `use_item` becomes a command that emits effects, not a method that calls 10 things
- **Event sourcing** for cross-concern coordination — `end_turn` emits phase events, systems react
- **Pipeline pattern** for generation — tile_generator returns data, caller writes to state
- **Strategy pattern** already used well in systems/ai.rs — could extend to ability dispatch

---

## Appendix: Quantified Impact

### Cleanup Completed (Phase 0.5)

| Category | LOC Removed | Files Removed |
|----------|-------------|---------------|
| Dead algorithms (4 fully dead + 3 test-only) | ~2,200 | 7 |
| Dead state.rs methods (15 + 4 stubs) | ~320 est. | 0 (in-file) |
| Half-wired ability methods (light/crystal/void) | ~600 est. | 0 (in-file) |
| Dead UI exports | ~50 est. | 0 (in-file) |
| ViewportCuller | ~30 est. | 0 (in-file) |
| terminal_spawn.rs | 52 | 1 |
| Orphaned schemas | — | 11 |
| structure_generation.json | — | 1 |
| **Total removed** | **~3,250 LOC** | **20 files** |

### Remaining Issues

| Category | Impact | Files |
|----------|--------|-------|
| `patterns/special/` duplicates | 7 duplicate files | 7 |
| Fake DES scenarios | False test confidence | 7 |
| Dead .des files | Clutter | 2 |
| Data files without schemas | No validation | 5 |
| Dangling data cross-references | Runtime errors possible | 18 refs across 2 files |
| Empty `algorithms/mod.rs` | Dead directory | 1 |

### Current Codebase Metrics

| Metric | Before Cleanup | After Cleanup |
|--------|---------------|---------------|
| Source files | ~164 | 150 |
| Total LOC | ~48,500 | 43,822 |
| state.rs LOC | 3,525 | 3,185 |
| state.rs methods | 163 | 135 (118 pub + 17 private) |
| Schemas | 47 (11 orphaned) | 36 (0 orphaned) |
| Light system LOC | 313 | 85 |
| Crystal system LOC | 376 | 151 |
| Void system LOC | 318 | 196 |
