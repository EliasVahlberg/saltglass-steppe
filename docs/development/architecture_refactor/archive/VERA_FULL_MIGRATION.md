# VERA Full Migration Plan

> Date: 2026-04-04
> Goal: Every GameState mutation during gameplay goes through `dispatch()`.
> Boundary: Generation, rendering, serialization, and keypress→Command mapping stay outside VERA.

---

## The Rule

**If it changes GameState, it goes through dispatch.** No exceptions.

- Player presses a key → UI produces a `Command` → `dispatch()` → rule → effects → apply
- Turn ends → `TurnPhase` sequence → each phase either produces effects or dispatches sub-commands
- System tick → rule function → effects → apply
- Reaction fires → reaction function → effects → apply

Nothing outside `dispatch()` and `apply_effect()` mutates GameState fields.

---

## Migration Batches

### Batch A: Simple player actions (easy, mechanical)

These are direct translations — same pattern as use_item.

| Method | LOC est. | New Command | Notes |
|--------|----------|-------------|-------|
| `equip_item` | ~20 | `Command::Equip` | Inventory shuffle + stat recalc |
| `unequip_slot` | ~10 | `Command::Unequip` | Reverse of equip |
| `allocate_stat` | ~30 | `Command::AllocateStat` | Stat point spending |
| `accept_quest` | ~30 | `Command::AcceptQuest` | Quest activation + faction alignment |
| `complete_quest` | ~30 | `Command::CompleteQuest` | Rewards: XP, items, reputation |
| `wait_turn` | ~15 | `Command::Wait` | Wait counter + end_turn |
| `rest` | ~30 | `Command::Rest` | HP recovery + turn advance |
| `interact_at` | ~40 | `Command::Interact` | NPC/chest/interactable dispatch |
| `examine_at` | ~60 | `Command::Examine` | Inspect tile contents |
| `use_psychic_ability` | ~30 | `Command::UsePsychic` | 3 working effects |
| `attempt_flee_encounter` | ~30 | `Command::FleeEncounter` | Encounter escape |
| `apply_status` | ~15 | Internal effect | StatusEffect application |
| `gain_xp` | ~25 | Internal effect | XP + level-up check |
| `modify_reputation` | ~10 | Internal effect | Faction rep change |

### Batch B: World travel

| Method | LOC est. | New Command | Notes |
|--------|----------|-------------|-------|
| `move_on_world_map` | ~80 | `Command::WorldMove` | World position + encounter check + tile transition |
| `travel_to_tile` | ~150 | Internal | Called by WorldMove — generation is outside VERA, but the state write (setting map, entities, FOV) becomes effects |
| `enter_subterranean` | ~40 | `Command::EnterSubterranean` | Layer transition |
| `exit_subterranean` | ~50 | `Command::ExitSubterranean` | Layer transition |

### Batch C: Turn system ticks (medium — convert system calls to rules)

| System | Current pattern | New pattern |
|--------|----------------|-------------|
| `StatusEffectSystem.update()` | `impl System` with `&mut GameState` | Rule: `rule_tick_status(ctx, rng) → Vec<Effect>` |
| `player.psychic.tick()` | Direct field mutation | Effect: `PlayerEffect::TickPsychic` |
| `player.skills.tick()` | Direct field mutation | Effect: `PlayerEffect::TickSkills` |
| `player.light_system.update()` | Direct field mutation + RNG | Rule: `rule_tick_light(ctx, rng) → Vec<Effect>` |
| `player.void_system.update()` | Direct field mutation + RNG | Rule: `rule_tick_void(ctx, rng) → Vec<Effect>` |
| `player.crystal_system.update()` | Direct field mutation + RNG | Rule: `rule_tick_crystal(ctx, rng) → Vec<Effect>` |
| `tick_time()` | Weather RNG every 10 turns | Rule: `rule_tick_time(ctx, rng) → Vec<Effect>` |
| `check_encounter_completion()` | Direct mutation | Rule or effect |
| `apply_light_effects()` | Glare damage, light-based effects | Rule: `rule_light_effects(ctx, rng) → Vec<Effect>` |
| `check_adaptation_threshold()` | Refraction check → adaptation gain | Reaction to refraction changes |

### Batch D: AI system

| System | New pattern |
|--------|-------------|
| `update_enemies()` | For each enemy: `rule_enemy_turn(enemy_idx, ctx, rng) → Vec<Effect>`. Effects: EnemyMove, EnemyAttack, EnemyHeal, etc. Coarse tracing: one `AiEffect::EnemyActed` per enemy, expandable to fine effects if needed. |

### Batch E: Storm system

| System | New pattern |
|--------|-------------|
| `StormSystem::apply_storm()` | Rule: `rule_storm_tick(ctx, rng) → Vec<Effect>`. Effects: `StormEffect::EditTile`, `StormEffect::SpawnWraith`, etc. |

### Batch F: Event system → Reactions

| System | Current | New |
|--------|---------|-----|
| `LootSystem.on_event(EnemyKilled)` | GameEvent listener | Reaction to `CombatEffect::Kill` |
| `QuestSystem.on_event(*)` | GameEvent listener | Reactions to relevant effects (Kill, Collect, Position, etc.) |
| `process_events()` | Event drain loop | Replaced by `run_reactions()` after each dispatch |

### Batch G: Visual effects and debug

| Method | Decision |
|--------|----------|
| `trigger_hit_flash`, `spawn_damage_number`, `spawn_projectile`, `spawn_beam` | These are Presentation effects — they don't change gameplay state. Move to `Presentation` enum variants. |
| `tick_hit_flash`, `tick_damage_numbers`, `tick_projectile_trails`, `tick_light_beams`, `tick_animation` | Presentation ticks — run in render loop or as a non-traced TurnPhase. |
| `debug_command` | Debug commands dispatch to VERA commands where applicable. |
| `log`, `log_typed` | Become `Presentation::LogMessage`. Already partially done. |

---

## Execution Order

1. **Batch A** (simple player actions) — mechanical, low risk, high coverage
2. **Batch B** (world travel) — medium complexity, important for consistency
3. **Batch C** (turn ticks) — converts end_turn phases from system calls to rules
4. **Batch F** (event→reactions) — replaces process_events with run_reactions
5. **Batch D** (AI) — highest complexity, but well-encapsulated
6. **Batch E** (storm) — well-encapsulated, can be coarse-grained
7. **Batch G** (visual/debug) — cleanup pass

Each batch should be a separate branch, merged after tests pass.

---

## New Effect Variants Needed (estimated)

| Domain | New variants |
|--------|-------------|
| PlayerEffect | TickPsychic, TickSkills, TickLight, TickVoid, TickCrystal, GainAdaptation, AllocateStat |
| ItemEffect | Equip, Unequip, Drop, SpawnOnMap |
| QuestEffect | Activate, Complete, Progress, SetFactionAlignment |
| CombatEffect | EnemyMove, EnemyAttack, EnemyHeal, ApplyStatus |
| MapEffect | WorldMove, SetWeather, AdvanceTime |
| StormEffect | EditTile, SpawnWraith, SetIntensity, AdvanceTimer |
| ResourceEffect | (existing variants sufficient) |
| EventEffect | (may be eliminated — replaced by typed effects) |
| AiEffect | EnemyActed (coarse), or per-action variants |

---

## What Gets Deleted

After full migration:
- `process_events()` — replaced by `run_reactions()`
- `GameEvent` enum — replaced by typed Effect reactions
- All direct `self.player.x = ...` mutations outside apply.rs
- `emit()` and `drain_events()` — no longer needed
- The `event_queue` field on GameState

---

## Success Criteria

- `dispatch()` is the only entry point for GameState mutation
- Every `pub fn` on GameState that mutates state is either deleted or converted to dispatch a Command
- `cargo test` passes, `cargo clippy -- -D warnings` is clean
- SYSTEM_STATUS.md updated for every migrated system
