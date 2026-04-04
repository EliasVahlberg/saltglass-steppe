---
status: current
last_verified: 2026-04-04
commit: e0d1fe7
---

# System Status Registry

> **Purpose**: Single source of truth for what actually works in gameplay. Read this before working on any system.
> **Architecture**: VERA (Verified Effect-Rule Architecture) — see `docs/development/architecture_refactor/FINAL_ARCHITECTURE.md`
> **Last verified**: 2026-04-04 (from codebase health audit)
> **Rule**: If a system isn't marked ✅, don't assume it works. Verify before building on it.

## Status Key

| Icon | Meaning |
|------|---------|
| ✅ | Fully wired: input path → state mutation → observable in gameplay |
| ⚠️ | Partially wired: some paths work, others are broken or incomplete |
| ❌ | Not wired: code exists but is unreachable from gameplay |
| 🗑️ | Dead: should be deleted in cleanup |

---

## Core Gameplay Systems

| System | Input Path | State Mutation | DES Coverage | Status |
|--------|-----------|---------------|-------------|--------|
| **Movement** | Arrow keys → `MovementSystem::try_move` | player.x/y, FOV, lighting, tile effects | Multiple scenarios | ✅ |
| **Melee combat** | Bump-to-attack → `CombatSystem::attack_melee` | enemy.hp, player.ap, XP, loot events | 12+ scenarios | ✅ |
| **Ranged combat** | 'f' key → `CombatSystem::ranged_attack` | enemy.hp, ammo consumed, projectile visual | Some scenarios | ✅ |
| **Item use** | 'u' key → `state.use_item()` | HP, AP, refraction, energy, inventory | Some scenarios | ✅ |
| **Item pickup** | Walk over → `MovementSystem::pickup_items` | inventory, item removed from map | Some scenarios | ✅ |
| **Equipment** | Inventory menu → equip | player.equipped_weapon | Minimal | ✅ |
| **Enemy AI** | `end_turn` → `AiSystem::update` | enemy positions, player.hp (attacks) | Some scenarios | ✅ |
| **Status effects** | Applied by combat/items → `StatusEffectSystem::update` | HP ticks, duration, expiry | Some scenarios | ✅ |
| **Storm system** | `end_turn` → `StormSystem::apply_storm` | map tiles (7 edit types), refraction, wraith spawns | Minimal | ✅ |
| **Quest system** | Events → `QuestSystem::on_event` | quest progress, completion | Some scenarios | ✅ |
| **Loot system** | EnemyKilled event → `LootSystem::on_event` | items spawned on map | Some scenarios | ✅ |
| **Save/load** | Menu → `save_game`/`load_game` | Full GameState serialization | No DES coverage | ⚠️ No tests |
| **World travel** | Map edge → `travel_to_tile` | map regenerated, entities spawned | Minimal | ✅ |
| **NPC dialogue** | Bump NPC → `MovementSystem::handle_npc_interaction` | pending_dialogue, quest events | Some scenarios | ✅ |
| **Trading** | NPC action → `pending_trade` | inventory, salt_scrip | Thin coverage (1-2 scenarios) | ⚠️ |
| **Crafting** | Menu → craft | inventory (consume + produce) | Thin coverage (1-2 scenarios) | ⚠️ |

## Special Systems (Audit Findings)

| System | Input Path | State Mutation | DES Coverage | Status | Audit Reference |
|--------|-----------|---------------|-------------|--------|----------------|
| **Light manipulation** | Menu opens ('g') but **no input handler** | `update()` ticks energy only | 1 fake scenario (player_alive only) | ❌ | Audit §2.1 |
| **Crystal resonance** | Menu opens ('V'), Enter **does nothing** | `update()` ticks energy only | 1 fake scenario (player_alive only) | ❌ | Audit §2.2 |
| **Void energy** | Menu opens, Enter dispatches `UseVoidAbility` | Energy deducted but **PhaseWalk unchecked in movement**, 4 other abilities are `_ => {}` | 1 fake scenario (player_alive only) | ❌ | Audit §2.3 |
| **Psychic abilities** | Full pipeline with cooldowns | Only 3 of N effects work (stun_aoe, guaranteed_hit, phasing). Rest log "not implemented." | No meaningful coverage | ⚠️ | Audit §2.4 |
| **Adaptations** | Refraction threshold → `check_adaptation_threshold` | Adaptation gained, stat modifiers applied | Minimal | ⚠️ |
| **Narrative engine** | None — bridge methods are stubs | `narrative_engine.rs` is a stub state container. `complete()` returns hardcoded rewards. | No coverage | ❌ | Audit §6.7 |
| **Narrative generation** | None — never called from game pipeline | `generation/narrative.rs` (535 LOC) + `narrative_templates.rs` (387 LOC) work in unit tests only | Unit tests only | ❌ | Audit §6.7 |

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
| Loot tables → Items cross-refs | ❌ | 2 dangling: `angle_split_lens`, `prism_shard` |
| Spawn tables → Items cross-refs | ❌ | 16 dangling: `ancient_gear`, `cactus_water`, `crystalline_shard`, `dried_herbs`, `healing_herb`, `prism_shard`, etc. |
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
