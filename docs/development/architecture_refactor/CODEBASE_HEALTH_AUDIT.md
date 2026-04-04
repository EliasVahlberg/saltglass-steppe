# Codebase Health Audit — 2026-04-03 (Updated 2026-04-04)

## Executive Summary

**Original finding:** AI-generated feature scaffolding that is vertically complete but horizontally disconnected from gameplay. ~3,600 LOC of dead/half-wired code. Root cause: absence of structural gates that catch incomplete integration before code is committed.

**Post-cleanup state:** ~150 source files, ~43,800 LOC. state.rs: 3,185 LOC / 135 methods (118 pub + 17 private). All dead code, orphaned schemas, duplicate patterns, fake DES scenarios, dead algorithms deleted. Structural gates added (SYSTEM_STATUS.md, commit policy, doc triage).

**Remaining issues:**
- 5 data files without schemas (biome_spawn_tables, environmental_props, main_questline, skill_trees, traders)
- 18 dangling data cross-references (16 spawn table + 2 loot table items not in items.json)
- Psychic abilities partially wired (3 of N effects work)
- No CI step for schema-data consistency

---

## Remaining Issues

### Psychic Abilities — partially wired

- Full pipeline with cooldowns, data-driven from abilities.json
- Only 3 hardcoded effect IDs work (stun_aoe, guaranteed_hit, phasing). Everything else logs "Effect not implemented."
- See `docs/development/SYSTEM_STATUS.md` for full wiring status.

### Schema Lifecycle Gap

- 5 data files lack schemas: biome_spawn_tables, environmental_props, main_questline, skill_trees, traders
- No CI step for schema-data consistency

### Data Cross-Reference Integrity

18 dangling references:
- Loot tables → Items: 2 dangling (angle_split_lens, prism_shard)
- Spawn tables → Items: 16 dangling (ancient_gear, cactus_water, crystalline_shard, dried_herbs, healing_herb, prism_shard, etc.)

No runtime validation exists for cross-references. DataLoader validates schema structure but not referential integrity.

### Audit Corrections

Items originally flagged as dead that are actually used:

| Item | Original Assessment | Actual Status |
|------|-------------------|---------------|
| `generation/narrative.rs` (535 LOC) | Dead — never called from game pipeline | **Used** by `map.rs::generate_area_description()` |
| `generation/narrative_templates.rs` (387 LOC) | Dead — tests only | **Used** via narrative.rs → map.rs |
| `structures/dungeon_generator.rs` (236 LOC) | Dead — custom system superseded | **Used** by `terrain_forge_adapter.rs` for POI-specific dungeon generation |
| `structures/ruins_generator.rs` (116 LOC) | Dead — custom system superseded | Used by dungeon_generator.rs |

---

## Architectural Weaknesses

### state.rs God Object

- **3,185 LOC**, **135 methods** (118 pub + 17 private)
- Still 16 distinct concerns, still the sole STATE-ORCHESTRATOR
- 4 files extend it with `impl GameState` blocks (combat_actions.rs, inspect.rs, qa_tools.rs, state.rs itself)
- Cleanup reduced its size but not its structural role

### Structural Gates (added 2026-04-04)

1. **System Status Registry** (`docs/development/SYSTEM_STATUS.md`): Source of truth for what is wired into gameplay.
2. **Commit Policy** (in `AGENTS.md` and `.kiro/steering/tech.md`): Requires proving DES scenario, honest status marking, registry update.
3. **Documentation Triage**: Technical/design docs triaged and archived with front-matter.

---

## Root Cause — The "Scaffold and Abandon" Anti-Pattern

### Git Forensics

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

### The Pattern

```
1. Feature request or design doc describes a system
2. AI generates complete vertical slice (struct + methods + update + UI + tests + docs)
3. Code compiles, trivial tests pass, commit is made
4. Last-mile integration never happens (input dispatch → game effect → movement/combat)
5. Next feature request arrives, repeat
```

The gap is always at the **horizontal integration boundary** — where the new system needs to modify existing complex code (input.rs, state.rs, movement systems).

### Telltale Signs

- Massive batch commits (1,500+ insertions, multiple complete systems)
- Identical boilerplate DES tests that assert nothing meaningful
- Commit messages that overstate integration
- 42 of 468 commits (9%) have 1,000+ insertions

---

## Module Structure Analysis

### Natural Clusters

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

### Key Structural Observations

- **Star topology**: state.rs is the only hub. Almost nothing cross-imports siblings directly.
- **22 leaf modules**: Only consumed by state.rs — they define types/functions, state.rs orchestrates.
- **world_map types are the true foundation**: Biome, POI, Terrain enums have 10 importers.
- **generation/ is already well-isolated**: 14,743 LOC with only 2 GameState touch points.
- **`impl GameState` sprawl**: 4 files extend GameState, creating structural coupling.

---

## Investigation Results

### state.rs Internal Dependency Graph

**Hub methods by fan-out:**
1. `end_turn` (fan-out 11) — turn → status → AI → storm → time → lighting → FOV → encounter → XP → events
2. `travel_to_tile` (fan-out 10, 4 are dead stubs) — generation → quest NPCs → FOV → spatial → lighting → crystal → narrative stubs
3. `use_item` (160 LOC, touches 10+ concerns inline) — AP, HP, refraction, adaptation, map reveal, ARIA, light/void/crystal energy, events, inventory. **Worst cross-concern offender.**
4. `move_on_world_map` / `travel_to_tile_safe` (fan-out 5 each) — travel + encounter wrappers
5. `process_events` (fan-out 3) — drain → LootSystem → QuestSystem → handle_event

**Key finding:** The event system (`emit`/`process_events`) is already a clean boundary. More systems should use it instead of direct method calls.

### encounter.rs / state.rs Coupling

encounter.rs is well-designed — pure functions taking explicit parameters, not `&mut GameState`. The 79 references in state.rs are field accesses and calls to pure functions. The coupling is in state.rs's orchestration code, not in encounter.rs itself. This orchestration could become an `EncounterSystem` in `systems/`.

### combat_actions.rs `impl GameState`

Only 95 LOC, 7 methods. 4 thin delegators, 2 pure reads, 1 cross-concern method (`try_break_wall` at 42 LOC). Organizational, not structural.

### DES Scenario Coverage

162 files total: 101 good, 48 setup-only, 7 fake (now deleted), 2 dead .des (previously deleted).

**Systems with zero real coverage:** sanity, ritual, save/load, encounter, interactable, meta-progression, crystal resonance, void energy, light manipulation, FOV, narrative engine.

**Dangerously thin coverage (1-2 scenarios):** crafting, movement, skills, trading.

### Narrative Subsystem

Three layers, two active:
1. **narrative_engine.rs** (103 LOC) — state container. QuestLog.on_* hooks functional for quest tracking.
2. **generation/narrative.rs** (535 LOC) + **narrative_templates.rs** (387 LOC) — Markov chain + template generation. Used by `map.rs::generate_area_description()`. Not dead.
3. **10 dead bridge methods in state.rs** — deleted.

---

## Meta-Level Computational Taxonomy

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

### Key Observations

1. **state.rs is the only `STATE-ORCHESTRATOR`.** Every cross-concern coordination goes through it.
2. **`DECISION-FN` modules are the cleanest.** Pure functions with explicit inputs/outputs — easiest to test, least likely to accumulate dead code.
3. **22 `DATA-DEF` modules are leaf nodes.** Zero reverse dependencies.
4. **Generation pipeline is a clean `DATA-XFORM` chain** — except tile_generator.rs which writes entities into GameState.
5. **The event system is underused.** Only LootSystem and QuestSystem use `EVENT-ROUTER`. Combat, movement, item use all use direct method calls through the STATE-ORCHESTRATOR.
6. **Two patterns dominated the dead code:** `DATA-XFORM` never connected to orchestrator (algorithms), `RESOURCE-ACCUM` without `INPUT-DISPATCH` path (light/crystal/void abilities).
7. **Architectural patterns suggested:** Command pattern for STATE-ORCHESTRATOR methods, event sourcing for cross-concern coordination, pipeline pattern for generation.

---

## Appendix: Codebase Metrics

| Metric | Before Cleanup | After Cleanup |
|--------|---------------|---------------|
| Source files | ~164 | ~150 |
| Total LOC | ~48,500 | ~43,800 |
| state.rs LOC | 3,525 | 3,185 |
| state.rs methods | 163 | 135 (118 pub + 17 private) |
| Schemas | 47 (11 orphaned) | 36 (0 orphaned) |
| Light system LOC | 313 | 85 |
| Crystal system LOC | 376 | 151 |
| Void system LOC | 318 | 196 |
| Dead code removed | — | ~3,250 LOC, 20+ files |
