---
status: current
last_verified: 2026-04-06
commit: c0401a0
---

# System Status Registry

> **Purpose**: Single source of truth for what actually works in gameplay. Read this before working on any system.
> **Architecture**: Verified State Store — see `docs/development/architecture_refactor/VERIFIED_STATE_STORE.md`
> **Last verified**: 2026-04-06 (state store refactor complete)
> **Rule**: If a system isn't marked ✅, don't assume it works. Verify before building on it.

## Architecture Summary

| Metric | Value |
|--------|-------|
| state.rs LOC | 940 |
| Command variants | 22 (all routed through dispatch.rs) |
| System modules | 13 (systems/*.rs) |
| Mutation variants | ~70 (atomic + bridge + delta + wrapper) |
| Rule functions | 20 (legacy, in rules/, being absorbed into systems/) |
| Rule unit tests | 39 |
| DES scenarios | 26 (7 are thin/false-positive — see DES section) |

**Verified State Store complete.** All commands route through `dispatch.rs` → system → `Vec<Mutation>` → `state.apply_mutations` (verified). Bridge mutations exist for complex systems (movement, world travel, turn, AI, storm). See `VERIFIED_STATE_STORE.md` for the two-tier mutation model.

## Status Key

| Icon | Meaning |
|------|---------|
| ✅ | Fully wired: input path → state mutation → observable in gameplay |
| ⚠️ | Partially wired: some paths work, others are broken or incomplete |
| ❌ | Not wired: code exists but is unreachable from gameplay |
| 🗑️ | Dead: should be deleted in cleanup |

---

## Core Gameplay Systems

| System | Input Path | State Mutation | DES Coverage | Status | VERA |
|--------|-----------|---------------|-------------|--------|------|
| **Movement** | Arrow keys → `Command::Move` → `rule_move` | player.x/y, FOV, lighting, tile effects | Multiple scenarios + 7 rule tests | ✅ | ⚠️ Pure rule; NPC/combat branches delegate to legacy bridge |
| **Melee combat** | Bump-to-attack → `Command::Attack` → `rule_melee_attack` | enemy.hp, player.ap, XP, loot events | 12+ scenarios + 4 rule tests | ✅ | ⚠️ Pure rule; post-processing (swarm aggro, reflect, split) still imperative |
| **Ranged combat** | 'f' key → `Command::RangedAttack` → `rule_ranged_attack` | enemy.hp, ammo consumed, projectile visual | Some scenarios + 3 rule tests | ✅ | ⚠️ Pure rule; same caveats as melee |
| **Item use** | 'u' key → `Command::UseItem` → `rule_use_item` | HP, AP, refraction, energy, inventory | 12 DES scenarios + 7 rule tests | ✅ | ✅ Pure rule |
| **Item pickup** | Walk over → `MovementSystem::pickup_items` | inventory, item removed from map | Some scenarios | ✅ | — Legacy bridge |
| **Equipment** | `Command::Equip/Unequip` → `rule_equip/rule_unequip` | player.equipment, equipped_weapon, armor | Minimal + 2 rule tests | ✅ | ✅ Pure rule |
| **Interact / Examine** | 'e'/'x' → `Command::Interact/Examine` → `rule_interact/rule_examine` | quest progress, log messages | Some scenarios | ✅ | ✅ Pure rule |
| **Crafting** | Menu → `Command::Craft` → `rule_craft` | inventory (consume + produce) | Thin (1-2 scenarios) + 1 rule test | ⚠️ | ✅ Pure rule |
| **Trading** | NPC action → `Command::BuyItem/SellItem` → `rule_buy/sell_item` | inventory, salt_scrip | Thin (1-2 scenarios) + 2 rule tests | ⚠️ | ✅ Pure rule |
| **Quest accept/complete** | `Command::AcceptQuest/CompleteQuest` → dispatch helpers | quest log, XP, items, reputation | Some scenarios | ✅ | ✅ Pure rule (effects only) |
| **Wait / Rest** | Space/r → `Command::Wait/Rest` → `rule_wait/rule_rest` | AP, HP, wait_counter | Some scenarios + 4 rule tests | ✅ | ✅ Pure rule |
| **Enemy AI** | `end_turn` → `TurnPhase::RunAI` → `PlayerEffect::RunAI` | enemy positions, player.hp | Some scenarios | ✅ | ⚠️ Bridge effect (calls `update_enemies()`) |
| **Status effects** | `end_turn` → `TurnPhase::TickStatusEffects` → `PlayerEffect::TickStatusEffects` | HP ticks, duration, expiry | Some scenarios | ✅ | ⚠️ Bridge effect (calls `StatusEffectSystem`) |
| **Loot drops** | `CombatEffect::Kill` → `reaction_loot_drop` → `ItemEffect::SpawnOnMap` | items spawned on map | Some scenarios + 1 rule test | ✅ | ✅ Pure reaction |
| **Storm system** | `end_turn` → `TurnPhase::TickStorm` → `MapEffect::TickStorm` | map tiles (7 edit types), refraction, wraith spawns | Minimal | ✅ | ⚠️ Bridge effect (calls `StormSystem`) |
| **Turn subsystems** | `end_turn` → `TurnPhase::TickSubsystems` → 5 bridge effects | psychic, skills, light, void, crystal ticks | Every scenario (implicit) | ✅ | ⚠️ Bridge effects |
| **Time / weather** | `end_turn` → `TurnPhase::AdvanceTime` → `MapEffect::AdvanceTime/SetWeather` | time_of_day, weather | Every scenario (implicit) + 2 rule tests | ✅ | ✅ Pure rule (inlined in execute_phase) |
| **Encounters** | `end_turn` → `TurnPhase::CheckEncounters` → `rule_check_encounters` | encounter_state, ClearEncounter | Some scenarios + 1 rule test | ✅ | ✅ Pure rule |
| **Adaptations** | Refraction threshold → `rule_check_adaptation` | adaptation gained, stat modifiers | Minimal + 1 rule test | ⚠️ | ✅ Pure rule |
| **Psychic abilities** | `Command::UsePsychic` → `dispatch_use_psychic` → `rule_use_psychic` | cooldowns; only 3 of N effects work | No meaningful coverage | ⚠️ | ⚠️ Pure rule; most effects unimplemented |
| **World travel** | Map edge → `Command::WorldMove` → `dispatch_world_move` | map regenerated, entities spawned | Minimal | ✅ | — Orchestrator (not VERA-migratable) |
| **Subterranean travel** | Stairs → `Command::EnterSubterranean` → `dispatch_enter_subterranean` | map regenerated, layer changed | Minimal | ✅ | — Orchestrator |
| **NPC dialogue** | Bump NPC → `handle_npc_interaction_legacy` | pending_ui.dialogue, quest events | Some scenarios | ✅ | — Legacy bridge from rule_move |
| **Save/load** | Menu → `save`/`load` | Full GameState serialization | No DES coverage | ⚠️ | — Legacy |
| **Quest progress** | `EventEffect::QuestNotify` → apply arm | quest objectives, auto-complete | Some scenarios | ✅ | ⚠️ Bridge effect (calls quest_log methods directly) |

## Procedural Generation

| System | Used By | Status | Notes |
|--------|---------|--------|-------|
| **terrain-forge adapter** | `tile_generator.rs` → game | ✅ | Active generation pipeline |
| **World generation** | `world_gen.rs` → game | ✅ | |
| **Connectivity (GSB)** | `connectivity.rs` → tile_generator | ✅ | |
| **Settlement generation** | `settlement/` → tile_generator | ✅ | |
| **Spawn system** | `spawn.rs` → tile_generator | ✅ | |
| **Microstructures** | `microstructures.rs` → tile_generator | ✅ | |
| **Environmental props** | `environmental_props.rs` → tile_generator | ✅ | |
| **Constraint validation** | `constraints.rs` → tile_generator | ✅ | |
| **Custom BSP algorithm** | Nothing | 🗑️ | Dead — superseded by terrain-forge |
| **Custom Maze algorithm** | Nothing | 🗑️ | Dead |
| **Custom Voronoi algorithm** | Nothing | 🗑️ | Dead |
| **Custom WFC algorithm** | Nothing | 🗑️ | Dead |
| **Custom Cellular Automata** | Test files only | 🗑️ | Tests validate dead code |
| **Custom Drunkard Walk** | Test files only | 🗑️ | Tests validate dead code |
| **Custom Simple Rooms** | Test files only | 🗑️ | Tests validate dead code |
| **Algorithm registry** | **Does not exist** | 🗑️ | Summary claims it does — false |

## Data Integrity

| Check | Status | Details |
|-------|--------|---------|
| Traders → Items cross-refs | ✅ | 0 dangling references |
| Spawn tables → Enemies cross-refs | ✅ | 0 dangling references |
| Loot tables → Items cross-refs | ✅ | 0 dangling (fixed: `angle_split_lens`→`angle_lens`, `prism_shard`→`prism_core`) |
| Spawn tables → Items cross-refs | ✅ | 0 dangling (fixed: 16 refs remapped to existing items) |
| Hardcoded constructor content | ✅ | `dying_pilgrim`, `hand_torch`, `glass_pick` moved to spawn table data (`room: first`) |
| Schema coverage | ⚠️ | 5 data files without schemas: biome_spawn_tables, environmental_props, main_questline, skill_trees, traders |
| Orphaned schemas | ❌ | 11 schemas with no matching data file |
| Runtime cross-ref validation | ❌ | DataLoader validates schema structure but not referential integrity |

## DES Test Coverage

> Last verified: 2026-04-06 (commit d73fd75)

| Category | Count | Notes |
|----------|-------|-------|
| Passing | 117 | Run with `cargo test --test des_scenarios` |
| Ignored — broken test format | 7 | Parse errors: invalid action/assertion types in JSON |
| Ignored — real code failures | 26 | Genuine bugs, see breakdown below |
| Ignored — flaky (storm RNG) | 2 | `storm_glass_drops`, `storm_intensity_scaling_test` |
| **Total registered** | **152** | |

### Ignored: broken test format (fix the JSON, not the code)

| Scenario | Problem |
|----------|---------|
| `bracket_lib_pathfinding_test` | Uses `check: {type: "custom"}` — not a valid assertion type |
| `terrain_variety_test` | Same |
| `bsp_algorithm_test` | Same |
| `main_questline_architect` | Uses `{type: "talk"}` action — not a valid DES action |
| `settlement_generation_test` | Same |
| `trading_system_test` | Uses `{type: "interact_npc"}` action — not valid |
| `procedural_structure_generation_test` | Invalid assertion op format |

### Ignored: real code failures (bugs to fix)

| Scenario | Failure | Root cause |
|----------|---------|-----------|
| `quest_reach_objective` | Reach objective not completing | `teleport` DES action calls `on_position_changed` but not `check_auto_complete` |
| `quest_chain_unlocking` | Same | Same |
| `quest_npc_spawning` | `NpcExists { npc_id: "test_npc" }` fails | `test_npc` doesn't exist in data |
| `dungeon_connectivity_test` | `ConnectivityRatio >= 0.8` fails | Glass Seam Bridging not achieving connectivity guarantee |
| `dungeon_comprehensive_validation` | Same | Same |
| `dungeon_deterministic_test` | Same | Same |
| `dungeon_quest_accessibility_test` | Same | Same |
| `archive_dungeon_test` | Same | Same |
| `connectivity_validation` | Same | Same |
| `shrine_connectivity_test` | Same | Same |
| `organic_cave_test` | `ConnectivityRatio >= 0.7` fails | Same |
| `progression` | `PlayerXp == 10` fails | `shard_spider.xp_value` is 8, test expects 10 |
| `level_up_stat_allocation` | `PlayerXp == 100` fails | Same XP mismatch; also `player_level: 0` is invalid (levels start at 1) |
| `shop_trading` | `SaltScrip == 40` fails | Scenario doesn't set `salt_scrip` in player setup |
| `npc_dialogue` | `NpcTalked` not set | `dying_pilgrim` bump interaction not marking `talked` flag |
| `loot_system_event_test` | `MessageContains "[Event]"` fails | Old GameEvent system deleted; `[Event]` prefix no longer emitted |
| `event_bus_test` | Same | Same |
| `laser_beam_behavior` | `PlayerHp < 50` fails | `laser_drone` enemy doesn't exist in data |
| `behavior_registry_test` | Bomber doesn't explode | `glass_bomber` enemy doesn't exist in data |
| `microstructures_on_travel` | `MicrostructureCount >= 1` fails | Microstructures not placed in DES world-coordinate tile load path |
| `auto_explore_fixes_test` | Wrong player position | Position assertion too brittle — depends on exact map layout |
| `auto_explore_danger_avoidance` | Wrong player position | Same |
| `dialogue_item_condition` | `MessageContains "UNAUTHORIZED"` fails | Dialogue condition check not logging to message log |
| `progression` | `PlayerLevel == 0` fails | Level 0 is invalid; levels start at 1 |
| `shop_trading` | `SaltScrip` wrong | Missing `salt_scrip` in scenario player setup |
| `npc_dialogue` | NPC not marked talked | Bump-to-talk path not setting `talked` flag |

### False positives — passing tests that verify nothing

20 scenarios pass with only `player_alive` as their assertion. They catch crashes but not logic errors.

`animation_effects_test`, `basic_movement`, `biome_system_basic`, `constraint_system_basic`, `effects_config_test`, `event_system_basic`, `generation_pipeline_basic`, `grammar_generation_basic`, `microstructures_test`, `narrative_integration_basic`, `procedural_effects_test`, `spawn_distribution_test`, `storm_timer_countdown`, `system_integration_test`, `template_system_basic`, `test_renderer_frame`, `theme_system_test`, `tutorial_messages_display`, `world_tile_transition` (no actions, no assertions — delete), `base_empty_room` (no assertions).

### Systems with zero meaningful DES coverage

Save/load, encounter (flee path), interactable, meta-progression, crystal resonance, void energy, light manipulation, FOV, narrative engine.

### Systems with thin coverage (1-2 scenarios)

Crafting, movement, skills, trading.

## Dead Code Summary

| Item | LOC | Status |
|------|-----|--------|
| `terminal_spawn.rs` | 52 | 🗑️ Never called |
| 4 dead algorithms (bsp, maze, voronoi, wfc) | ~1,300 | 🗑️ Zero usage |
| 3 test-only algorithms (cellular_automata, drunkard_walk, simple_rooms) | ~900 | 🗑️ Tests validate dead code |
| 15 dead methods in state.rs | ~300 est. | 🗑️ Never called |
| 4 dead stubs in end_turn | ~20 | 🗑️ Empty method bodies |
| Dead UI exports (render_map, render_inventory_bar) | ~50 est. | 🗑️ Never called |
| ViewportCuller | ~30 est. | 🗑️ Result unused |
| `data/structures/patterns/special/` | 7 files | 🗑️ Exact duplicates of `patterns/ruins/` |
| 11 orphaned schemas | 11 files | 🗑️ No matching data |
| `structure_generation.json` | 1 file | 🗑️ Only used by deprecated tool |
| 7 fake DES scenarios | 7 files | 🗑️ or rewrite |
| 2 dead .des files | 2 files | 🗑️ Never executed |

---

## How to Use This Document

**Before implementing a feature that depends on another system:**
1. Check this table for the system's status
2. If ❌ or ⚠️, do NOT assume it works — verify with a DES scenario or manual test
3. If 🗑️, the code should be deleted, not built upon

**After wiring a new system or fixing a broken one:**
1. Update this table with the new status
2. Add the DES scenario that proves it works
3. Record the date of verification

**For AI agents:**
This document overrides `.agents/summary/` claims. If the summary says a system is functional but this registry says ❌, trust this registry.
