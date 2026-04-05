# Dispatch Extraction Design

> Date: 2026-04-05
> Status: Design — not yet implemented
> Prerequisite: Complete VERA_REMAINING_TASKS.md (soft-migration completion) first
> Supersedes: DOMAIN_DECOMPOSITION_PLAN.md (which proposed moving apply + rules into domain modules)

---

## Problem Statement

state.rs is 3,297 LOC. The VERA soft-migration wrapped each GameState method in Command→Rule→Effect→Apply but left all orchestration as `&mut self` methods on GameState. The result:

1. Every new feature requires adding a dispatch method to GameState
2. Dispatch helpers mix three concerns: context resolution, rule invocation, and post-processing
3. GameState is simultaneously a data container, a command router, and every domain's orchestrator
4. Bridge effects (AI, storm, loot, status) exist because extracting them as pure rules while keeping everything in state.rs was too invasive

## Chosen Approach: Dispatch Extraction

Move orchestration out of GameState into free functions in `src/game/dispatch/`. GameState keeps only:
- Struct definition
- `apply_effect` / `apply_and_trace` / `run_reactions` (VERA infrastructure)
- Derives (FOV, lighting, spatial index)
- Accessors
- Save/load

The key insight: dispatch functions don't need `&mut self` — they need `&mut GameState` passed as a parameter. This is a structural change, not an architectural redesign. Rules, effects, and apply stay exactly where they are.

## Target Structure

```
src/game/
├── state.rs              — struct definition, VERA infra, derives, accessors, save/load
├── state_init.rs         — new(), new_with_class() (from Task 3 of soft-migration)
├── dispatch/
│   ├── mod.rs            — pub fn dispatch(state, command) router
│   ├── combat.rs         — handle_melee, handle_ranged
│   ├── movement.rs       — handle_move
│   ├── items.rs          — handle_use_item, handle_equip, handle_craft, handle_buy, handle_sell
│   ├── world.rs          — handle_world_move, handle_world_move_safe, handle_follow_path,
│   │                       handle_calculate_path, handle_enter_sub, handle_exit_sub
│   ├── player.rs         — handle_wait, handle_rest, handle_psychic, handle_allocate_stat,
│   │                       handle_flee_encounter
│   ├── quest.rs          — handle_accept_quest, handle_complete_quest
│   ├── interact.rs       — handle_interact, handle_examine
│   └── reactions.rs      — collect_reactions (free function, already pure)
├── effects/              — UNCHANGED (Effect/Command enums, apply.rs, context.rs, trace.rs)
├── rules/                — UNCHANGED (pure rule functions)
└── systems/              — UNCHANGED (legacy system impls)
```

## What Stays in state.rs

| Concern | Current LOC | Notes |
|---------|-------------|-------|
| Struct definition + field sub-structs | ~80 | GameState, SpatialIndex, DebugState, PendingUi |
| `dispatch()` | ~5 | Becomes a one-liner delegating to `dispatch::dispatch(self, command)` |
| `apply_effect()` | ~10 | Routes to apply.rs (unchanged) |
| `apply_and_trace()` | ~20 | Generic VERA infra |
| `run_reactions()` | ~20 | Calls `dispatch::reactions::collect_reactions()` |
| Derives: `update_fov`, `update_lighting`, `rebuild_spatial_index` | ~60 | Cross-cutting, used by multiple dispatch functions |
| `end_turn` + `execute_phase` | ~100 | Turn system — moves to `dispatch/turn.rs` or stays |
| Logging: `log`, `log_typed` | ~10 | Used everywhere |
| Visual effects: `trigger_hit_flash`, `spawn_damage_number`, etc. | ~58 | Used by combat dispatch, stays as utility methods |
| Accessors: 30+ getters | ~105 | Thin, used by UI/rendering |
| Save/load | ~20 | Serialization |
| `check_auto_end_turn`, `tick_turn_housekeeping` | ~33 | Turn utilities |
| **Total** | **~520** | Down from 3,297 |

## What Moves to dispatch/

### dispatch/mod.rs (~40 LOC)

The router. Replaces the current `dispatch()` match body.

```rust
use crate::game::state::GameState;
use crate::game::effects::Command;

pub mod combat;
pub mod movement;
pub mod items;
pub mod world;
pub mod player;
pub mod quest;
pub mod interact;
pub mod reactions;

pub fn dispatch(state: &mut GameState, command: Command) {
    match command {
        Command::Move { dx, dy } => movement::handle_move(state, dx, dy),
        Command::Attack { target_x, target_y } => combat::handle_melee(state, target_x, target_y),
        Command::RangedAttack { target_x, target_y } => combat::handle_ranged(state, target_x, target_y),
        Command::Wait => player::handle_wait(state),
        Command::Rest => player::handle_rest(state),
        Command::Equip { inv_idx, slot } => items::handle_equip(state, inv_idx, &slot),
        Command::Unequip { slot } => items::handle_unequip(state, &slot),
        Command::AllocateStat { stat } => player::handle_allocate_stat(state, &stat),
        Command::AcceptQuest { quest_id } => quest::handle_accept(state, &quest_id),
        Command::CompleteQuest { quest_id } => quest::handle_complete(state, &quest_id),
        Command::Interact { x, y } => interact::handle_interact(state, x, y),
        Command::Examine { x, y } => interact::handle_examine(state, x, y),
        Command::UsePsychic { ability_id } => player::handle_psychic(state, &ability_id),
        Command::FleeEncounter => player::handle_flee(state),
        Command::WorldMove { new_wx, new_wy } => world::handle_world_move(state, new_wx, new_wy),
        Command::WorldMoveSafe { new_wx, new_wy } => world::handle_world_move_safe(state, new_wx, new_wy),
        Command::EnterSubterranean => world::handle_enter_subterranean(state),
        Command::ExitSubterranean => world::handle_exit_subterranean(state),
        Command::FollowWorldPath => world::handle_follow_path(state),
        Command::CalculateWorldPath { target_wx, target_wy } => world::handle_calculate_path(state, (target_wx, target_wy)),
        Command::UseItem { index } => items::handle_use_item(state, index),
        Command::UseItemOnTile { index, x, y } => items::handle_use_item_on_tile(state, index, x, y),
    }
}
```

### dispatch/combat.rs (~220 LOC)

Absorbs from state.rs:
- `dispatch_melee_attack` (107 LOC) → `handle_melee(state, x, y)`
- `dispatch_ranged_attack` (93 LOC) → `handle_ranged(state, x, y)`

Pattern — each function is a direct move, replacing `self.` with `state.`:

```rust
pub fn handle_melee(state: &mut GameState, target_x: i32, target_y: i32) {
    state.ensure_spatial_index();
    let output = {
        let ctx = QueryContext::from_state(state);  // or manual construction for rng
        rules::rule_melee_attack(target_x, target_y, &ctx, &mut state.rng)
    };

    // Inspect effects for post-processing
    let killed_idx = /* ... */;
    let hit = /* ... */;

    let applied = output.effects.clone();
    state.apply_and_trace(output, "rule_melee_attack");
    state.run_reactions(&applied, 0);

    // Post-processing (visual effects, swarm, reflect, split, on-death)
    if hit { state.trigger_hit_flash(target_x, target_y); }
    // ... rest of post-processing unchanged
}
```

### dispatch/movement.rs (~80 LOC)

Absorbs: `dispatch_move` (70 LOC) → `handle_move(state, dx, dy)`

### dispatch/items.rs (~100 LOC)

Absorbs:
- Inline dispatch arms for UseItem, UseItemOnTile, Equip, Unequip (18 LOC)
- `dispatch_craft` (33 LOC) → `handle_craft(state, recipe_id) -> bool`
- `dispatch_buy_item` (16 LOC) → `handle_buy(state, item_id, npc_id) -> Result<(), String>`
- `dispatch_sell_item` (17 LOC) → `handle_sell(state, item_id) -> Result<(), String>`

### dispatch/world.rs (~250 LOC)

Absorbs:
- `dispatch_world_move` (62 LOC)
- `dispatch_world_move_safe` (33 LOC)
- `dispatch_follow_world_path` (24 LOC)
- `dispatch_calculate_world_path` (36 LOC)
- EnterSubterranean passthrough (currently 1 LOC, will become proper dispatch)
- ExitSubterranean passthrough (currently 1 LOC, will become proper dispatch)

### dispatch/player.rs (~100 LOC)

Absorbs:
- Inline dispatch arms for Wait (5 LOC), Rest (10 LOC), AllocateStat (4 LOC)
- `dispatch_use_psychic` (13 LOC)
- `dispatch_flee_encounter` (48 LOC)

### dispatch/quest.rs (~100 LOC)

Absorbs:
- `dispatch_accept_quest` (37 LOC)
- `dispatch_complete_quest` (51 LOC)

### dispatch/interact.rs (~80 LOC)

Absorbs:
- `dispatch_interact` (28 LOC)
- `dispatch_examine` (45 LOC)

### dispatch/reactions.rs (~40 LOC)

Absorbs:
- `collect_reactions` (32 LOC) — becomes a free function (already doesn't use `self`)
- `run_reactions` stays on GameState (it calls `apply_effect` and `trace.record`)

## What Does NOT Change

| Component | Location | Why unchanged |
|-----------|----------|---------------|
| Effect/Command enums | `effects/mod.rs` | Stable, well-defined |
| apply_effect + domain apply fns | `effects/apply.rs` | Mechanical, centralized — single source of truth for mutations |
| QueryContext / TestContext | `effects/context.rs` | Used by rules, unchanged |
| Trace type aliases | `effects/trace.rs` | Thin re-exports |
| Pure rule functions | `rules/*.rs` | Already correct — pure functions taking &QueryContext |
| Legacy systems | `systems/*.rs` | Bridge effects call these from apply arms |
| Rule unit tests | `rules/*.rs` | Already correct |
| DES scenarios | `tests/des/` | Unchanged — they call `dispatch()` which delegates |

## Migration Approach

### Principle: One file at a time, zero behavior change

Each dispatch file is created by:
1. Copy the method body from state.rs
2. Replace `self.` with `state.`
3. Replace `&mut self.rng` with `&mut state.rng`
4. Add the function signature: `pub fn handle_X(state: &mut GameState, ...)`
5. Delete the method from state.rs
6. Update `dispatch()` to call the new free function
7. `cargo build` — compiler catches any missed references
8. `cargo test` — all tests pass

### Phase order

```
Phase E0: Create dispatch/mod.rs skeleton
  - Create dispatch/ directory and mod.rs
  - dispatch() on GameState becomes: dispatch::dispatch(self, command)
  - All dispatch_* methods still on GameState, called from dispatch/mod.rs via state.method()
  - Zero behavior change. cargo test passes.

Phase E1: Extract combat (largest, most complex post-processing)
  - Move dispatch_melee_attack → dispatch/combat.rs::handle_melee
  - Move dispatch_ranged_attack → dispatch/combat.rs::handle_ranged
  - Delete from state.rs
  - cargo test passes.

Phase E2: Extract world (largest by LOC)
  - Move dispatch_world_move, dispatch_world_move_safe, dispatch_follow_world_path,
    dispatch_calculate_world_path → dispatch/world.rs
  - Move enter/exit_subterranean passthroughs → dispatch/world.rs
  - Delete from state.rs
  - cargo test passes.

Phase E3: Extract quest + interact
  - Move dispatch_accept_quest, dispatch_complete_quest → dispatch/quest.rs
  - Move dispatch_interact, dispatch_examine → dispatch/interact.rs
  - Delete from state.rs
  - cargo test passes.

Phase E4: Extract items + player
  - Move dispatch_craft, dispatch_buy_item, dispatch_sell_item → dispatch/items.rs
  - Move inline Wait/Rest/Equip/Unequip/AllocateStat/UsePsychic/FleeEncounter → dispatch/player.rs + dispatch/items.rs
  - Delete from state.rs
  - cargo test passes.

Phase E5: Extract reactions
  - Move collect_reactions → dispatch/reactions.rs as free function
  - run_reactions stays on GameState, calls dispatch::reactions::collect_reactions()
  - Delete collect_reactions from state.rs
  - cargo test passes.

Phase E6: Fix external callers
  - main.rs L352: state.dispatch_craft() → dispatch::items::handle_craft(&mut state)
  - des/mod.rs L1889: state.dispatch_craft() → dispatch::items::handle_craft(&mut state)
  - des/mod.rs L1897: state.dispatch_buy_item() → dispatch::items::handle_buy(&mut state)
  - des/mod.rs L1901: state.dispatch_sell_item() → dispatch::items::handle_sell(&mut state)
  - des/mod.rs L1804: state.apply_status() → state.dispatch(Command equivalent) or keep as apply
  - cargo test passes.
```

### Estimated LOC movement

| From state.rs | To | LOC |
|---------------|-----|-----|
| dispatch_melee_attack, dispatch_ranged_attack | dispatch/combat.rs | 200 |
| dispatch_world_move, _safe, _follow, _calculate, enter/exit_sub | dispatch/world.rs | 220 |
| dispatch_accept_quest, dispatch_complete_quest | dispatch/quest.rs | 88 |
| dispatch_interact, dispatch_examine | dispatch/interact.rs | 73 |
| dispatch_craft, dispatch_buy_item, dispatch_sell_item | dispatch/items.rs | 66 |
| Wait, Rest, AllocateStat, UsePsychic, FleeEncounter inline arms | dispatch/player.rs | 80 |
| UseItem, Equip, Unequip inline arms | dispatch/items.rs | 24 |
| collect_reactions | dispatch/reactions.rs | 32 |
| dispatch() match body | dispatch/mod.rs | 30 |
| **Total moved** | | **~813** |

state.rs after: ~3,297 - 813 = **~2,484 LOC**

That's still large. The remaining bulk is:
- Constructor: ~367 LOC (moves to state_init.rs in soft-migration Task 3)
- World travel/generation: ~453 LOC (travel_to_tile, spawn helpers — generation pipeline, not dispatch)
- Encounter system: ~167 LOC (spawn_encounter_entities, attempt_flee — called by dispatch/world.rs)
- Auto-explore: ~242 LOC (auto_explore + helpers)
- Chest operations: ~84 LOC
- Player system methods: ~124 LOC (psychic, status, reputation — legacy methods, deleted in soft-migration Task 4)
- Turn system: ~131 LOC (end_turn, execute_phase, tick_turn_housekeeping)
- Visual effects: ~58 LOC
- Accessors: ~105 LOC
- Lighting/FOV: ~60 LOC
- Lore stubs: ~95 LOC (deleted in soft-migration Task 3)

After soft-migration cleanup (Tasks 3+4) removes ~590 LOC and dispatch extraction removes ~813 LOC:
**state.rs target: ~1,900 LOC**

The remaining ~1,900 LOC is:
- World travel/generation (~453) — generation pipeline, not dispatch
- Encounter system (~167) — called by dispatch, could move later
- Auto-explore (~242) — self-contained, could move later
- Turn system (~131) — could move to dispatch/turn.rs
- Chest operations (~84) — could move to dispatch/items.rs
- Visual effects (~58) — utility methods
- Derives (~60) — cross-cutting
- Accessors (~105) — thin getters
- Struct definition (~80) — core
- VERA infra (~50) — apply_and_trace, run_reactions, dispatch stub
- Logging (~10) — utility
- Save/load (~20) — serialization
- Spatial index (~40) — utility

This is a reasonable size for a central state file. Further extraction (world travel, encounters, auto-explore, turn system) can happen incrementally if/when those areas need modification.

## Reactions Design

`collect_reactions` becomes a free function in `dispatch/reactions.rs`:

```rust
use crate::game::effects::{CombatEffect, Effect, EventEffect, QuestNotifyKind};

/// Pure function: given applied effects, produce reaction effects.
/// No state access needed — only pattern-matches on effect variants.
pub fn collect_reactions(
    effects: &[Effect],
) -> Vec<(Effect, &'static str, Effect)> {
    let mut results = Vec::new();
    for effect in effects {
        if let Effect::Combat(CombatEffect::Kill { enemy_id, x, y, .. }) = effect {
            results.push((
                Effect::Event(EventEffect::LootDrop {
                    enemy_id: enemy_id.clone(),
                    x: *x,
                    y: *y,
                }),
                "reaction_loot_drop",
                effect.clone(),
            ));
            results.push((
                Effect::Event(EventEffect::QuestNotify {
                    kind: QuestNotifyKind::Kill {
                        enemy_id: enemy_id.clone(),
                    },
                }),
                "reaction_quest_kill",
                effect.clone(),
            ));
        }
    }
    results
}
```

`run_reactions` stays on GameState because it calls `apply_effect` and `trace.record`:

```rust
// In state.rs
pub fn run_reactions(&mut self, effects: &[Effect], depth: u32) {
    if depth >= 10 { return; }
    let reaction_effects = dispatch::reactions::collect_reactions(effects);
    if reaction_effects.is_empty() { return; }
    let turn = self.turn;
    for (effect, source_name, trigger) in &reaction_effects {
        self.apply_effect(effect);
        self.trace.record(effect, TraceSource::Reaction {
            name: source_name,
            trigger: Box::new(trigger.clone()),
        }, turn);
    }
    let next: Vec<_> = reaction_effects.into_iter().map(|(e, _, _)| e).collect();
    self.run_reactions(&next, depth + 1);
}
```

New reactions are added by adding match arms to `collect_reactions`. The function is:
- Pure (no state access)
- Exhaustive-match-friendly (compiler catches unhandled patterns if you use `match` instead of `if let`)
- Testable in isolation (pass in effects, assert on output)
- The single place where cross-domain effect chains are defined

## Verification Properties Preserved

| Property | Before | After |
|----------|--------|-------|
| Rules are pure | `rules/*.rs` take `&QueryContext` | Unchanged |
| Effects are exhaustive | `apply.rs` matches all variants | Unchanged — apply.rs untouched |
| Trace records everything | `apply_and_trace` on GameState | Unchanged |
| Reactions are explicit | `collect_reactions` hardcoded match | Same match, now a free function |
| DES tests work | Call `state.dispatch(Command)` | `dispatch()` on GameState delegates to `dispatch::dispatch(self, command)` — same API |
| Rule unit tests work | Use TestContext | Unchanged — rules untouched |

## What This Does NOT Solve

1. **apply.rs growth** — As effects grow, apply.rs grows. This is a future problem (currently 409 LOC, manageable).
2. **World travel/generation in state.rs** — `travel_to_tile` and spawn helpers are generation pipeline code. They should eventually move to the generation module, but that's a separate concern from dispatch extraction.
3. **Bridge effect opacity** — RunAI, TickStorm, LootDrop still call legacy code from apply arms. Soft-migration Task 5 addresses status/loot/quest; AI and storm stay as bridges.
4. **Auto-explore complexity** — 242 LOC of pathfinding/decision logic. Could move to `dispatch/explore.rs` but it's not a dispatch helper — it's a compound action that issues Commands.

## Success Criteria

- No `dispatch_*` methods remain on GameState
- No inline dispatch logic in `dispatch()` body on GameState
- `collect_reactions` is a free function
- state.rs is under 2,000 LOC (after soft-migration cleanup + dispatch extraction)
- `cargo test` passes, all DES scenarios pass, 0 clippy warnings
- External callers (main.rs, des/mod.rs) updated
