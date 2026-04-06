# State Store Refactor — Reflection

> Written: 2026-04-06
> Covers: VERA migration (Stages 1–5) + state.rs decomposition
> Baseline: 3,195 LOC in state.rs → 940 LOC. Total removed: ~2,255 LOC.

---

## 1. Final State of the Codebase

### File structure

```
src/game/
├── state.rs          940 LOC  — struct, mutation engine, dispatch API, derives, queries
├── state_init.rs     398 LOC  — new(), new_with_class()
├── dispatch.rs       122 LOC  — route_command(), apply_with_cascade()
├── notify.rs          34 LOC  — on_transitions() reaction layer
├── mutations.rs      170 LOC  — Mutation enum, StateTransition enum, SubsystemId
├── rules/
│   ├── actions.rs    413 LOC  — rule_wait, rule_rest, rule_equip, rule_use_psychic, etc.
│   ├── combat.rs     385 LOC  — rule_melee_attack, rule_ranged_attack
│   ├── item.rs       411 LOC  — rule_use_item, rule_use_item_on_tile
│   ├── movement.rs   253 LOC  — rule_move
│   ├── economy.rs    182 LOC  — rule_craft, rule_buy_item, rule_sell_item
│   ├── reactions.rs   62 LOC  — reaction_loot_drop
│   └── turn.rs       114 LOC  — rule_check_encounters, rule_check_adaptation
└── systems/
    ├── ai.rs         762 LOC  — AiBehavior trait, 4 behaviors, update_enemies
    ├── world.rs      580 LOC  — travel_to_tile, world movement, crystal generation
    ├── storm.rs      390 LOC  — StormSystem, 7 edit types
    ├── combat.rs     359 LOC  — handle_melee, handle_ranged, on_enemy_killed
    ├── movement.rs   336 LOC  — MovementSystem, dispatch_move, pickup_items
    ├── explore.rs    245 LOC  — auto_explore, BFS helpers
    ├── items.rs      161 LOC  — handle_use_item, chest operations
    ├── player.rs     121 LOC  — handle_wait/rest/equip, check_adaptation_threshold, apply_status_effect
    ├── turn.rs       118 LOC  — end_turn, execute_phase, tick_turn_housekeeping
    ├── status.rs     146 LOC  — StatusEffectSystem
    ├── interact.rs   101 LOC  — handle_interact, handle_examine
    ├── quest.rs      108 LOC  — handle_accept_quest, handle_complete_quest
    └── loot.rs        63 LOC  — reaction_loot_drop bridge
```

### What lives where

**state.rs** is now exclusively: the `GameState` struct definition, `apply_one` (the exhaustive mutation match), `apply_mutations`, the `dispatch*` command API, `update_fov`/`update_lighting`, `rebuild_spatial_index`, read-only queries (`enemy_at`, `has_status_effect`, `get_reputation`, etc.), logging helpers, and `save`/`load`. No system logic.

**dispatch.rs** routes `Command` variants to system handlers and runs the cascade loop (`apply_with_cascade` → `apply_recursive` → `notify::on_transitions`).

**notify.rs** maps `StateTransition` values to reactive mutations. Currently handles `EnemyHpChanged` and `EnemyHpReachedZero`. Everything else is a stub.

**mutations.rs** defines the full `Mutation` enum (the vocabulary of state changes), `StateTransition` (detected side-effects), and `SubsystemId`.

---

## 2. What the Architecture Enforces vs. What It Doesn't

### What it enforces

**Atomic field mutations with invariants.** Every `Set*` arm in `apply_one` is a single field assignment with a clamp or bounds check. `SetPlayerHp` clamps to `[0, max_hp]` and returns `PlayerDied` if it crosses zero. `SetPlayerAp` returns `PlayerApReachedZero`. `SetEnemyHp` returns `EnemyHpReachedZero`. These transitions are structural — they cannot be bypassed.

**Transition-driven reactions.** The cascade loop in `dispatch.rs` is the only place reactions fire. `notify::on_transitions` is the single registry for cross-system coordination. Adding a new reaction means adding one line there, not hunting for call sites.

**Exhaustive match.** Adding a new `Mutation` variant without handling it in `apply_one` is a compile error. This is the primary structural gate against scaffold-and-abandon.

**Separation of rule and apply.** Rule functions in `rules/` are pure: `(args, &QueryContext, &mut RNG) → Vec<Effect>`. They cannot mutate state. The compiler enforces this because they don't receive `&mut GameState`.

### Where it doesn't hold

**Bridge mutations bypass the invariant layer entirely.** `WorldMove`, `MovePlayer`, `EndTurn`, `RestTick`, `UsePsychicAbility`, `AttemptFlee`, `TickSubsystem` all call back into imperative code that mutates state directly, outside `apply_one`. Those mutations are logged to `mutation_log` for DES tracing, but the state changes they produce are not atomic, not transition-detected, and not cascade-eligible.

**`apply_one` is not re-entrant safely.** `Equip` calls `apply_one(&Mutation::SetEquipment)`. `StunEnemy` calls `apply_one(&Mutation::AddEnemyStatus)`. `TickStatusEffects` calls `apply_one(&Mutation::TickSubsystem(Status))`. `RunAI` calls `apply_one(&Mutation::TickSubsystem(AI))`. These recursive calls work but produce no transitions (the inner call's return value is discarded), so reactions cannot fire on them.

**`SetEquipment` has logic.** It calls `recalc_equipment_stats` as a side effect. This is the only `Set*` arm that does more than a field assignment. It's a small violation but it means equipping an item is not a pure field write.

**`QuestNotify` has 30 LOC of inline dispatch logic** inside `apply_one`. It's a match-within-a-match that calls `quest_log` methods and collects completions. This belongs in a system function.

---

## 3. Known Technical Debt

### Bridge mutations (call imperative code, bypass invariant layer)

| Mutation | Calls | Problem |
|----------|-------|---------|
| `MovePlayer { dx, dy }` | `systems::movement::dispatch_move` | Runs rule_move, NPC interaction, combat, FOV — entire turn's worth of logic |
| `EndTurn` | `self.end_turn()` | Runs all 9 turn phases; no transitions emitted |
| `RestTick` | `self.tick_turn_housekeeping()` ×10 + `self.update_enemies()` | Bypasses turn phase sequence |
| `WorldMove { wx, wy }` | `systems::world::dispatch_world_move` | Regenerates entire map; no transitions |
| `WorldMoveSafe { wx, wy }` | `systems::world::dispatch_world_move_safe` | Same |
| `FollowWorldPath` | `systems::world::dispatch_follow_world_path` | Same |
| `CalculateWorldPath` | `systems::world::dispatch_calculate_world_path` | Same |
| `EnterSubterranean` | `systems::world::enter_subterranean` | Same |
| `ExitSubterranean` | `systems::world::exit_subterranean` | Same |
| `UsePsychicAbility` | `player.psychic.use_ability()` + `rule_use_psychic` + `apply_mutations` | Nested apply_mutations inside apply_one |
| `AttemptFlee` | `encounter::attempt_flee` with `&mut self.rng` | Mutates rng directly, not via clone-writeback |
| `TickSubsystem(AI)` | `self.update_enemies()` | Entire AI tick, no transitions |
| `TickSubsystem(Storm)` | `StormSystem::apply_storm` | Entire storm tick, no transitions |
| `TickSubsystem(Status)` | `StatusEffectSystem::tick_*` | Status ticks, no transitions |
| `TickSubsystem(Light/Void/Crystal)` | `*.update(&mut self.rng)` | Subsystem ticks, rng mutated directly |

### Duplicate / redundant mutation variants

| Redundant | Canonical | Notes |
|-----------|-----------|-------|
| `SpendAp(i32)` | `SetPlayerAp(i32)` | Delta vs absolute; callers must compute final value for Set* |
| `AddHp(i32)` | `SetPlayerHp(i32)` | Same |
| `AddSaltScrip(u32)` | `SetPlayerSaltScrip(u32)` | Same |
| `AddRefraction(i32)` | `SetPlayerRefraction(u32)` | Same |
| `IncrementWaitCounter` | `SetWaitCounter(u32)` | Single-purpose increment |
| `Equip { slot, item_id }` | `SetEquipment { slot, item_id: Some(...) }` | Equip delegates to SetEquipment |
| `Unequip(slot)` | `SetEquipment { slot, item_id: None }` | Same |
| `RecalcStats` | (side effect of SetEquipment) | SetEquipment already calls recalc |
| `StunEnemy { idx, duration }` | `AddEnemyStatus { idx, id: "stun", duration }` | StunEnemy delegates to AddEnemyStatus |
| `TickStatusEffects` | `TickSubsystem(Status)` | TickStatusEffects delegates to TickSubsystem |
| `RunAI` | `TickSubsystem(AI)` | RunAI delegates to TickSubsystem |
| `TickStorm` | `TickSubsystem(Storm)` | TickStorm delegates to TickSubsystem |
| `TickHousekeeping` | `TickSubsystem(Housekeeping)` | TickHousekeeping delegates to TickSubsystem |

### `apply_one` arms with inline logic that should be extracted

| Arm | LOC | What it does | Where it should go |
|-----|-----|-------------|-------------------|
| `QuestNotify` | ~35 | Dispatches to 8 quest_log methods, collects completions, logs them | `systems::quest::handle_quest_notify` |
| `AttemptFlee` | ~20 | Calls `encounter::attempt_flee`, handles Ok/Err, mutates encounter state | `systems::player::handle_attempt_flee` |
| `UsePsychicAbility` | ~14 | Calls `psychic.use_ability`, calls `rule_use_psychic`, calls `apply_mutations` | `systems::player::handle_use_psychic` |
| `DamageWall` | ~16 | Reads tile hp, decrements, conditionally replaces tile | `systems::combat::handle_damage_wall` |
| `SetEquipment` | ~8 | Parses slot, sets equipment, calls recalc | Acceptable as-is; recalc side effect is the only issue |

---

## 4. What Went Well

**The Mutation enum as vocabulary.** Having a single exhaustive enum for all state changes made the codebase greppable in a way it wasn't before. Finding every place that modifies player HP is now `grep "SetPlayerHp\|AddHp"`. Before, it was scattered across 40+ methods.

**StateTransition + notify.rs.** The cascade loop is clean. Adding a new reaction (e.g., "when player gains XP, check for level up") is one line in `notify::on_transitions`. The depth limit prevents infinite loops. This pattern scaled well.

**The decomposition stages.** Moving world travel, constructors, auto-explore, turn system, and movement to separate files was mechanical and low-risk. Each stage compiled and passed tests independently. The LOC reduction was real: 3,195 → 940 with no behavior change.

**DES as a regression harness.** Having 26 DES scenarios meant every structural change could be verified in ~1 second. The scenarios caught several regressions during the decomposition that would have been invisible without them.

**`dispatch.rs` is small and readable.** At 122 LOC it's the complete routing table for all 22 commands. A new developer can read it in 5 minutes and understand the full command surface.

**The `effects/apply.rs` deletion.** Deleting the old apply layer (~300 LOC) was the cleanest moment of the refactor. It removed a parallel path that had been causing confusion about which code was actually running.

---

## 5. What Went Wrong or Was Harder Than Expected

**The Python extraction scripts were fragile.** The approach of extracting line ranges with Python scripts caused two incidents: the `state_init.rs` extraction included `dispatch()` and `dispatch_move()` because they were between `new()` and `new_with_class()` in the file, and the depth-tracking logic failed on multi-line function signatures. Both required manual recovery. A better approach would have been to write the destination file first, then delete from the source — rather than extracting and hoping the range was right.

**The delegation accessor block had 149 call sites.** Deleting 30 wrapper methods (`player_x()`, `map()`, `enemies()`, etc.) required touching 23 files. The sed/regex approach worked but produced two double-replacement bugs (`self.world.visual_effects.world.visual_effects.*` and `self.crate::game::systems::player::check_adaptation_threshold`). These were caught immediately by the compiler but added friction. The lesson: regex-based mass replacement on method calls is error-prone when the method names are substrings of field paths.

**Bridge mutations were the right call but created a two-tier system.** The decision to wrap orchestrators (`WorldMove`, `MovePlayer`, `EndTurn`) as bridge mutations rather than decomposing them into atomic mutations was pragmatic — decomposing `dispatch_move` into atomic mutations would have required understanding the full NPC/combat/movement branch logic. But it means the mutation log is misleading: `MovePlayer { dx: 1, dy: 0 }` appears as one entry but represents potentially 20+ state changes. DES assertions on movement effects are testing the bridge, not the atoms.

**`effects/apply.rs` and `state.rs` were parallel for too long.** During the VERA migration, both `apply_effect` (in `effects/apply.rs`) and `apply_one` (in `state.rs`) existed simultaneously. Some commands went through one, some through the other. This was intentional (incremental migration) but created a period where it was genuinely unclear which path a given command took. The Stage 5 deletion resolved this, but the parallel period was longer than it needed to be.

**The `notify.rs` layer is underused.** It was designed as the single registry for cross-system reactions. Currently it only handles two transitions (`EnemyHpChanged`, `EnemyHpReachedZero`). `PlayerPositionChanged`, `TurnAdvanced`, `ItemAddedToInventory`, and `PlayerEnteredWorldTile` are all detected but produce no reactions. The infrastructure is there; the content isn't.

**If starting over:** Write destination files first, then delete from source. Don't use regex replacement for method-to-field migrations — use the LSP rename tool instead. Decompose bridge mutations into atomic mutations from the start rather than deferring it; the deferral created technical debt that compounds.

---

## 6. The RNG Problem

### Current pattern

`dispatch.rs` uses clone-call-writeback for all rule functions:

```rust
let mut rng = state.rng.clone();
let mutations = combat::handle_melee(x, y, &ctx, &mut rng);
state.rng = rng;
```

This is correct: the rule function advances a clone, and the clone is written back only if the call succeeds. The RNG state after a command is deterministic regardless of whether the command produced effects.

### Where it breaks down

**`AttemptFlee` in `apply_one` uses `&mut self.rng` directly**, not clone-writeback. This means the RNG advances inside `apply_one`, which is called from `apply_mutations`, which is called from `apply_with_cascade`. The RNG advancement is not isolated to the dispatch layer. If `AttemptFlee` is applied as part of a cascade (it isn't currently, but could be), the RNG state would be different from a non-cascade path.

**`TickSubsystem(Light/Void/Crystal)` uses `&mut self.rng` directly.** These subsystem ticks advance the RNG inside `apply_one`. The advancement is deterministic (same mutations → same RNG sequence) but it's not isolated to the dispatch layer. If the order of subsystem ticks changes, RNG sequences diverge.

**`UsePsychicAbility` calls `apply_mutations` inside `apply_one`.** This means a psychic ability use advances the RNG through a nested `apply_mutations` call. The outer `apply_mutations` loop does not know this happened. The RNG state after a psychic ability use is correct but the path is convoluted.

**`rule_use_psychic` takes no RNG argument.** Psychic abilities that need randomness (e.g., stun radius, damage variance) cannot be deterministic because the rule function has no RNG. Currently the 3 implemented psychic effects don't need RNG, so this is latent rather than active.

### Is determinism preserved?

Yes, for all currently tested paths. The DES scenarios pass with fixed seeds. But the RNG handling is inconsistent: some paths use clone-writeback (dispatch layer), some use direct mutation (apply_one bridge arms). A future developer adding a new bridge mutation that uses RNG needs to know which pattern to follow — and the code doesn't make this obvious.

---

## 7. DES and Testing

### How assertions work

DES runs with `state.trace.enabled = true`. Two parallel logs are maintained:

- `state.trace.entries` — records `Effect` variants from the old VERA trace path (populated by `rule_output_to_mutations` when it calls `trace.record`)
- `state.mutation_log` — records `format!("{:?}", mutation)` for every mutation applied

`EffectOccurred { pattern }` checks both: it matches the pattern as a substring against both the trace entries' debug repr and the mutation log entries. This means assertions work regardless of whether a command went through the Effect trace path or the Mutation path.

The dual-log approach is a pragmatic workaround for the migration period. It means `EffectOccurred { pattern: "Heal" }` will match both a `PlayerEffect::Heal` trace entry and a `Mutation::AddHp` log entry. This is useful but imprecise — the same assertion can match two different things.

### Coverage gaps

**Systems with zero unit tests:** All 14 files in `systems/`. The rule functions have 39 unit tests across 7 files, but the system functions (which contain the bridge logic, NPC interaction, world travel, AI) have none. These are tested only through DES scenarios.

**Systems with thin or no DES coverage:**
- `systems/explore.rs` — auto-explore has no DES scenario
- `systems/turn.rs` — turn phase sequence is exercised implicitly by every scenario but never asserted on directly
- `systems/player.rs` — `check_adaptation_threshold`, `apply_light_effects` have no scenarios
- `systems/world.rs` — world travel has minimal coverage (1-2 scenarios)
- `systems/items.rs` — chest operations have no DES coverage (they're UI-driven)
- `state_init.rs` — constructor logic (class starting stats, spawn table items) has no scenario

**The 7 fake DES scenarios** (identified in SYSTEM_STATUS.md) still exist and provide false confidence. They assert `player_alive` after a `wait` action.

**`notify.rs` reactions are undertested.** The loot drop reaction (`EnemyHpReachedZero → reaction_loot_drop`) has 1 rule test and a few DES scenarios. The other 5 `StateTransition` variants produce no reactions and have no tests asserting they don't.

---

## 8. Recommendations for the Next Developer

### Read first

1. `docs/development/SYSTEM_STATUS.md` — authoritative wiring status. Don't assume a system works because it compiles.
2. `src/game/dispatch.rs` — the complete command routing table. 122 LOC. Read it before adding any command.
3. `src/game/mutations.rs` — the full vocabulary of state changes. Read it before adding any mutation.
4. `src/game/notify.rs` — the reaction registry. Read it before adding any cross-system effect.

### Adding a new command

1. Add a variant to `Command` in `effects/mod.rs`
2. Add a handler in the appropriate `systems/` or `rules/` file
3. Add a routing arm in `dispatch::route_command`
4. Add a `Mutation` variant if needed (prefer reusing existing `Set*` variants)
5. Add an `apply_one` arm if you added a mutation
6. Write a DES scenario that asserts on the observable state change
7. Write a rule unit test if the handler is a pure rule function

### Adding a new mutation

Prefer `Set*` (absolute value) over delta variants. The delta variants (`AddHp`, `SpendAp`, `AddRefraction`, `AddSaltScrip`) exist for legacy reasons — systems that don't have the current value available. If you're writing new code, compute the final value in the rule function and emit `SetPlayerHp(final_value)`.

Do not add a new bridge mutation unless the operation genuinely cannot be expressed as a sequence of atomic mutations. Bridge mutations bypass the invariant layer, produce no transitions, and cannot trigger reactions.

### The two-tier mutation system

There are two kinds of mutations and they behave differently:

- **Atomic mutations** (`SetPlayerHp`, `SetEnemyHp`, `AddToInventory`, etc.): single field assignment, may return a `StateTransition`, can trigger reactions via `notify::on_transitions`.
- **Bridge mutations** (`MovePlayer`, `EndTurn`, `WorldMove`, `TickSubsystem`, etc.): call imperative code, return nothing, cannot trigger reactions, bypass the invariant layer.

If you apply a bridge mutation and expect a reaction to fire on the state changes it produces, it won't. The reaction system only sees `StateTransition` values returned by atomic `apply_one` arms.

### RNG

Use the clone-writeback pattern in `dispatch.rs` for any new command handler that needs RNG:

```rust
let mut rng = state.rng.clone();
let mutations = your_handler(args, &ctx, &mut rng);
state.rng = rng;
Some(mutations)
```

Do not pass `&mut self.rng` directly to functions called from inside `apply_one`. The `AttemptFlee` arm does this and it's a known inconsistency.

### Gotchas

**`apply_one` is called recursively.** `Equip` calls `apply_one(SetEquipment)`. `StunEnemy` calls `apply_one(AddEnemyStatus)`. The inner call's `StateTransition` return value is discarded. If you need a reaction to fire on the inner mutation, you need to restructure.

**`UsePsychicAbility` calls `apply_mutations` inside `apply_one`.** This is the only place this happens. It works but it means psychic ability effects are applied in a nested context where the outer cascade loop doesn't know about them.

**The `mutation_log` is only populated when `trace.enabled = true`.** This is set by DES runs. In normal gameplay, `mutation_log` is always empty. Don't use it for anything other than DES assertions.

**`systems/` files have no unit tests.** If you break a system function, the only thing that will catch it is a DES scenario. If there's no DES scenario for the system you're modifying, write one before you change anything.

**`state_init.rs` contains hardcoded content** that was moved from the constructor to spawn table data (`room: first`). If you're adding starting items or NPCs, add them to the spawn tables in `data/`, not to `new_with_class`.

**The 7 fake DES scenarios** in `tests/scenarios/` (listed in SYSTEM_STATUS.md) will pass no matter what you do. Don't use them as evidence that a system works.
