# Verified State Store

> Canonical architecture reference for state management in saltglass-steppe.
> Last updated: 2026-04-06

---

## Overview

GameState is a verified data store. It holds all game data, applies mutations with per-field invariant checks, and reports state transitions. It contains zero system logic and zero notification logic.

Three roles:

- **Systems** decide what should change. They receive read-only state and return mutation requests.
- **State** applies mutations, enforces invariants, and reports what changed.
- **Notifications** react to state transitions. They map transitions to reactive mutations.

```
Input (Command)
  │
  ▼
dispatch.rs ─── routes to system
  │
  ▼
System::handle(query, rng) → Vec<Mutation>         ← system owns logic
  │
  ▼
state.apply_mutations(mutations) → Vec<Transition>  ← state owns data + verification
  │
  ▼
notify::on_transitions(transitions, state, rng)     ← external, expandable
  → Vec<Mutation>
  │
  ▼
state.apply_mutations(reactions) → Vec<Transition>   ← cascade (depth-limited to 10)
  │
  ▼
Derives: FOV, lighting, spatial index                ← run once after all cascades settle
```

## File Structure

```
src/game/
├── state.rs          940 LOC  — struct, mutation engine, dispatch API, derives, queries
├── state_init.rs     398 LOC  — new(), new_with_class()
├── dispatch.rs       122 LOC  — route_command(), apply_with_cascade()
├── notify.rs          34 LOC  — on_transitions() reaction layer
├── mutations.rs      170 LOC  — Mutation enum, StateTransition enum, SubsystemId
├── rules/                     — pure rule functions (legacy, being absorbed into systems/)
├── systems/
│   ├── combat.rs              — handle_melee, handle_ranged, on_enemy_hit, on_enemy_killed
│   ├── movement.rs            — dispatch_move (bridge), pickup_items
│   ├── items.rs               — handle_use_item, chest operations
│   ├── world.rs               — world travel, tile generation, encounters
│   ├── player.rs              — handle_wait/rest/equip, adaptation, status effects
│   ├── quest.rs               — handle_accept/complete_quest
│   ├── turn.rs                — end_turn, execute_phase
│   ├── explore.rs             — auto_explore
│   ├── interact.rs            — handle_interact, handle_examine
│   ├── loot.rs                — reaction_loot_drop bridge
│   ├── ai.rs                  — update_enemies (bridge)
│   ├── storm.rs               — StormSystem (bridge)
│   └── status.rs              — StatusEffectSystem (bridge)
├── effects/
│   ├── mod.rs                 — Command enum, Effect enums (legacy, used by rules/)
│   ├── context.rs             — QueryContext (read-only state view)
│   └── trace.rs               — trace recording
```

## Two Tiers of Mutations

The Mutation enum has two kinds of variants that behave differently:

### Atomic mutations

Single field assignment with invariant enforcement. May return a `StateTransition`. Can trigger reactions via `notify::on_transitions`.

```rust
// Examples:
Mutation::SetPlayerHp(i32)          // clamped to [0, max_hp], detects PlayerDied
Mutation::SetEnemyHp { idx, hp }    // detects EnemyHpReachedZero, EnemyHpChanged
Mutation::SetPlayerPosition { x, y } // detects PlayerPositionChanged
Mutation::AddToInventory(String)     // detects ItemAddedToInventory
Mutation::AdvanceTurn                // detects TurnAdvanced
Mutation::SetReputation { .. }       // clamped to [-100, 100]
```

### Bridge mutations

Call imperative code inside `apply_one`. Return no transitions. Cannot trigger reactions. Bypass the invariant layer for the state changes they produce internally.

```rust
// Examples:
Mutation::MovePlayer { dx, dy }     // calls dispatch_move — runs rule, NPC, combat, FOV
Mutation::WorldMove { wx, wy }      // calls dispatch_world_move — regenerates map
Mutation::EndTurn                    // calls end_turn — runs all 9 turn phases
Mutation::RestTick                   // calls tick_turn_housekeeping ×10 + update_enemies
Mutation::UsePsychicAbility { .. }  // calls psychic.use_ability + rule + apply_mutations
Mutation::TickSubsystem(SubsystemId) // calls legacy system code (AI, storm, status, etc.)
```

Bridge mutations exist because decomposing these operations into atomic mutations would require rewriting the underlying systems. They are pragmatic compromises, not the target pattern. New features should prefer atomic mutations.

### Delta mutations (convenience)

Relative mutations for cases where the system doesn't have the current value:

```rust
Mutation::SpendAp(i32)              // equivalent to SetPlayerAp(current - amount)
Mutation::AddHp(i32)                // equivalent to SetPlayerHp(current + amount)
Mutation::AddSaltScrip(u32)         // equivalent to SetPlayerSaltScrip(current + amount)
Mutation::AddRefraction(i32)        // equivalent to SetPlayerRefraction(current + delta)
Mutation::IncrementWaitCounter      // equivalent to SetWaitCounter(current + 1)
```

Prefer `Set*` variants when the system has access to the current value. Delta variants exist for legacy compatibility.

### Wrapper mutations (delegation)

Thin wrappers that delegate to another mutation:

```rust
Mutation::Equip { slot, item_id }   // delegates to SetEquipment { slot, item_id: Some(..) }
Mutation::Unequip(slot)             // delegates to SetEquipment { slot, item_id: None }
Mutation::StunEnemy { idx, dur }    // delegates to AddEnemyStatus { idx, "stun", dur }
Mutation::TickStatusEffects         // delegates to TickSubsystem(Status)
Mutation::RunAI                     // delegates to TickSubsystem(AI)
Mutation::TickStorm                 // delegates to TickSubsystem(Storm)
Mutation::TickHousekeeping          // delegates to TickSubsystem(Housekeeping)
Mutation::ResetAp                   // equivalent to SetPlayerAp(max_ap)
```

These exist for readability in system code. They add no behavior.

## State Transitions

Detected inside `apply_one()` by comparing pre/post values. State reports these but takes no action.

```rust
pub enum StateTransition {
    PlayerPositionChanged { old_x: i32, old_y: i32, new_x: i32, new_y: i32 },
    PlayerApReachedZero,
    PlayerDied,
    EnemyHpChanged { idx: usize, old_hp: i32, new_hp: i32 },
    EnemyHpReachedZero { idx: usize, enemy_id: String, x: i32, y: i32 },
    TurnAdvanced { old_turn: u32, new_turn: u32 },
    ItemAddedToInventory { item_id: String },
    PlayerEnteredWorldTile { wx: usize, wy: usize },
}
```

### Currently wired in notify.rs

| Transition | Reactions |
|-----------|-----------|
| `EnemyHpChanged` | `combat::on_enemy_hit` — swarm aggro, reflect damage, hit flash, damage number |
| `EnemyHpReachedZero` | `combat::on_enemy_killed` — split-on-death, loot drop |

### Detected but not wired (no reactions fire)

| Transition | Intended reactions (not yet implemented) |
|-----------|-----------|
| `PlayerPositionChanged` | Pickup items, world transition, adaptation check |
| `PlayerApReachedZero` | Auto end turn |
| `TurnAdvanced` | Subsystem ticks |
| `ItemAddedToInventory` | Quest collect tracking |
| `PlayerEnteredWorldTile` | Tile generation, encounter check |
| `PlayerDied` | (handled by game loop) |

These transitions are detected by atomic mutations but the corresponding systems currently handle their post-processing through bridge mutations instead of through the notification layer.

## Verification Rules

Each mutable field has an invariant enforced inside `apply_one()`. Only applies to atomic mutations — bridge mutations bypass this.

| Field | Invariant | Enforcement |
|-------|-----------|-------------|
| `player.hp` | 0 ≤ hp ≤ max_hp | Clamp |
| `player.ap` | 0 ≤ ap ≤ max_ap | Clamp |
| `player.max_hp` | ≥ 1 | Clamp(1, ..) |
| `player.max_ap` | ≥ 1 | Clamp(1, ..) |
| `player.xp` | Monotonically increasing | max(old, new) |
| `player.level` | 1 ≤ level ≤ max_level | Clamp |
| `player.salt_scrip` | ≥ 0 | u32 (natural) |
| `faction_reputation` | -100 ≤ rep ≤ 100 | Clamp |
| `enemy[idx].*` | idx in bounds | No-op if out of bounds |
| `world.time_of_day` | 0..24 | Modulo 24 |
| `resonance_energy` | ≤ max_resonance_energy | Clamp |
| Presentation mutations | None | Always applied |

## Notification Module

`notify.rs` is external to state. It maps transitions to reactive mutations.

Currently handles combat reactions only (EnemyHpChanged, EnemyHpReachedZero). Other transitions are detected but produce no reactions — the corresponding post-processing happens inside bridge mutations instead.

Adding a new reaction: add a line to the relevant transition match arm in `notify.rs`.
Adding a new transition type: add a variant to `StateTransition`, add detection in `apply_one()`, add a match arm in `notify.rs`.

Subscribers are static (compile-time match arms, not dynamic registration).

## Dispatch

### dispatch.rs

Routes all 22 Command variants to system handlers. Owns the cascade loop (apply → transitions → notify → apply, depth-limited to 10). Runs derives (FOV, lighting) after cascades settle.

### Command API on GameState

`state.dispatch(command)` is the single entry point. It calls `dispatch::route_command` and `dispatch::apply_with_cascade`.

Three additional methods remain on GameState for commands with return values that the UI needs:
- `dispatch_craft(recipe_id) → bool`
- `dispatch_buy_item(item_id, npc_id) → Result<(), String>`
- `dispatch_sell_item(item_id) → Result<(), String>`

## System Interface

Systems are free functions. Two signatures:

**Command handlers** — called from dispatch, take `&QueryContext` (read-only snapshot):
```rust
pub fn handle_melee(query: &QueryContext, x: i32, y: i32, rng: &mut ChaCha8Rng) -> Vec<Mutation>
```

**Notification handlers** — called from notify.rs, take `&GameState` (read-only, richer access):
```rust
pub fn on_enemy_hit(state: &GameState, idx: usize, old_hp: i32, new_hp: i32) -> Vec<Mutation>
```

Both return `Vec<Mutation>`. Neither mutates state.

## RNG Handling

Systems that need randomness receive `&mut ChaCha8Rng`. The dispatch layer uses clone-writeback to avoid borrow conflicts:

```rust
let mut rng = state.rng.clone();
let mutations = system::handle(&ctx, &mut rng);
state.rng = rng;  // write back to advance canonical rng
```

Bridge mutations that use rng inside `apply_one` (TickSubsystem Light/Void/Crystal, AttemptFlee) access `&mut self.rng` directly. This is a known inconsistency — these paths advance rng inside the mutation engine rather than at the dispatch layer.

## What state.rs Contains (~940 LOC)

| Concern | Role |
|---------|------|
| Data model (~178 LOC) | GameState struct, SpatialIndex, DebugState, PendingUi, MsgType, etc. |
| Mutation engine (~430 LOC) | `apply_one` (exhaustive match), `apply_mutations` |
| Command API (~55 LOC) | `dispatch`, `dispatch_craft/buy/sell` |
| Derives (~55 LOC) | `update_fov`, `update_lighting` |
| Spatial index (~40 LOC) | `ensure_spatial_index`, `rebuild_spatial_index` |
| Queries (~50 LOC) | `effective_ambient_light`, `enemy_at`, `has_status_effect`, etc. |
| Logging (~28 LOC) | `log`, `log_typed`, `log_quest_completions` |
| Misc (~20 LOC) | `save`, `load`, `trigger_effect`, `debug_command` |

Zero system logic. Zero notification logic.
