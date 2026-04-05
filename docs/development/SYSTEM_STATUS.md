---
status: current
last_verified: 2026-04-05
commit: dddd97b
---

# System Status Registry

> **Purpose**: Single source of truth for what actually works in gameplay. Read this before working on any system.
> **Architecture**: VERA (Verified Effect-Rule Architecture) — see `docs/development/architecture_refactor/FINAL_ARCHITECTURE.md`
> **Last verified**: 2026-04-05 (VERA soft-migration complete)
> **Rule**: If a system isn't marked ✅, don't assume it works. Verify before building on it.

## VERA Migration Summary

| Metric | Value |
|--------|-------|
| state.rs LOC | 3,195 |
| Command variants | 22 |
| Rule modules | 7 (actions, combat, economy, item, movement, reactions, turn) |
| Rule functions | 20 |
| Rule unit tests | 39 |
| DES scenarios | 157 |

**Soft-migration complete.** All gameplay actions go through `dispatch()`. Legacy bypass paths deleted. Next: domain decomposition of state.rs (see `STATE_RS_MIGRATION_PLAN.md`).

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

| Category | Count | Notes |
|----------|-------|-------|
| Good scenarios (exercise real gameplay) | 101 | |
| Setup-only (no meaningful assertions) | 48 | |
| Fake (identical boilerplate, test nothing) | 7 | crystal_resonance_basic, void_energy_basic, light_manipulation_basic, enhanced_enemy_systems_test, fov_system_test, narrative_system_test, story_model_test |
| Dead .des files (never executed) | 2 | skill_progression_test.des, faction_system_test.des |
| **Total** | **162** | |

### Systems with zero meaningful DES coverage

Sanity, ritual (doesn't exist), save/load, encounter, interactable, meta-progression, crystal resonance, void energy, light manipulation, FOV, narrative engine.

### Systems with dangerously thin coverage (1-2 scenarios)

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
