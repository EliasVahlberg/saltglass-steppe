# ESCAEV Architecture Proposal — Lead Developer Review

> Reviewer: LeadDeveloper (system-agent)
> Date: 2026-04-03
> Status: REVIEW COMPLETE — actionable feedback

---

## 1. Accuracy of Claims

The proposal's claims about the current codebase are **largely correct**, with a few nuances worth noting.

### Verified correct

- **state.rs as god object**: Confirmed. 3,525 LOC, 163 methods, 16 concerns. The struct definition alone (state.rs lines 97–170) has ~30 fields spanning player state, world state, spatial indices, event queues, debug flags, mock combat state, pending UI state, and narrative engine. It is genuinely the only cross-concern coordinator.

- **`use_item` complexity**: Confirmed. Starting at line 2639, it runs ~140 lines touching AP, HP, refraction, adaptations, map reveal, ARIA events, light energy, void exposure, void energy, crystal resonance, crystal frequency placement, and inventory removal. The proposal says "160 LOC, 10+ concerns" — the actual count is ~140 LOC and 12 distinct concerns. Close enough.

- **`end_turn` fan-out**: Confirmed at line 1741. The method calls: `ensure_spatial_index`, AP reset, `StatusEffectSystem.update`, `psychic.tick`, `skills.tick`, `light_system.update`, `void_system.update`, `crystal_system.update`, `tick_turn`, `update_enemies`, `StormSystem::apply_storm` (conditional), `tick_time`, `update_lighting`, `update_fov`, `check_encounter_completion`, encounter tick, `check_dynamic_events`, event emit, `process_events`. That's 19 distinct calls, not 11. The proposal **understates** the fan-out.

- **Pure decision functions exist**: Confirmed. `combat.rs` has `roll_attack`, `calc_hit_chance`, `calc_damage` as pure functions (lines 63–99). `encounter.rs` and `adaptation.rs` are similarly pure per the audit. These are genuine ESCAEV Rule candidates.

- **Event system is underused**: Confirmed. `process_events` (line 1793) only dispatches to `LootSystem`, `QuestSystem`, and `handle_event` (which is just logging). The `GameEvent` enum (event.rs) has 20 variants but they're notification-only — none drive mutations.

- **Scaffold-and-abandon pattern**: The audit's git forensics are thorough and the 3,600 LOC dead code figure is well-documented. The proposal correctly identifies this as the core problem ESCAEV aims to prevent.

### Minor inaccuracies

- **"fan-out 11" for end_turn**: Actually 19 distinct calls. The audit's own section 6.1 says fan-out 11, but counting the actual method body shows more. This matters because it means Phase 4 (end_turn decomposition) is bigger than estimated.

- **combat_actions.rs characterization**: The proposal doesn't mention it, but the audit says "95 LOC, 7 methods, 4 thin delegators." Actual file is 95 LOC with 7 methods — confirmed. But `attack_melee` and `try_ranged_attack` are pure delegators to `CombatSystem`, while `update_enemies` delegates to `AiSystem`. The `try_break_wall` method (42 LOC) is the only one with real cross-concern logic. This is relevant because it means combat extraction (Phase 2) has less `impl GameState` sprawl to deal with than expected.

- **CombatSystem already partially fits ESCAEV**: `systems/combat.rs` has `process_enemy_death` as a static method taking `&mut GameState` — it handles XP, loot events, split-on-death spawning, visual effects, and meta-progress. This is already a proto-Rule/Reaction, but it mutates state directly rather than returning effects. The proposal doesn't call this out as a specific migration target, but it should.

---

## 2. Feasibility of Incremental Migration

The proposal claims incremental migration with no rewrite. This is **mostly feasible but harder than presented**.

### What works about the incremental approach

- **Phase 0 (foundation) is genuinely zero-risk.** Defining an `Effect` enum, a `Trace` struct, and an `apply_effect` function adds new code without touching existing code. This is the right starting point.

- **Phase 1 (use_item) is a good proof-of-concept target.** `use_item` is self-contained — it reads state, makes decisions, mutates state, and returns. It doesn't call other orchestrator methods. The proposed split into `use_item_rule` returning `Vec<Effect>` is mechanically straightforward.

- **The existing event system provides a migration bridge.** `GameEvent` variants already exist for most cross-system notifications. During migration, a Rule can return Effects while the old code path still emits GameEvents. Both can coexist.

### What's harder than presented

- **The `&mut GameState` problem.** Every system method takes `&mut GameState`. A Rule that needs to query player HP, item definitions, and map state needs read access to three different parts of GameState simultaneously. In the current struct layout, you can't borrow `self.player` and `self.world.map` separately through `&GameState` — you need the whole thing. The proposal's `use_item_rule` example takes `&PlayerState` and `&DataLoader<ItemDef>` separately, which is correct, but extracting these sub-borrows from the call site in `state.rs` requires careful restructuring. Every Rule call site will need to destructure GameState into its components.

- **`apply_effect` needs `&mut GameState` too.** The mechanical application layer matches on Effect variants and writes to specific fields. But in Rust, `apply_effect(&mut self, effect: &Effect)` is a method on GameState — and if you're iterating over a `Vec<Effect>` returned by a Rule, you need to make sure the Rule's return value doesn't borrow anything from GameState. This is fine if Rules take owned/cloned data or references to `DataLoader` statics, but it constrains the Rule signature.

- **Phase 2 (combat) is significantly harder than Phase 1.** Look at `CombatSystem::attack_melee` (systems/combat.rs, line 131): it reads enemy position, checks AP, sets provoked flag, checks swarm behavior, reads weapon def, reads enemy stats, reads skill bonuses, rolls attack, applies mocks, reads enemy name, computes direction, applies damage, emits events, triggers visual effects, spawns damage numbers, checks on_hit effects, handles reflect_damage behavior, records last_damage_dealt, and conditionally calls `process_enemy_death` (which itself does XP, loot events, split spawning, meta-progress). Converting this to a Rule returning `Vec<Effect>` means the Effect enum needs variants for: DeductAP, SetProvoked, TriggerSwarmAggro, DamageEnemy, EmitEvent, TriggerVisualEffect, SpawnDamageNumber, SetLastDamageDealt, KillEnemy — plus all the effects from process_enemy_death. That's a lot of Effect variants for one action, and the ordering matters (damage before death check, death before loot).

- **Phase 4 (end_turn) is the real test.** The 19-call fan-out means 19 potential phase steps. Some of these (like `light_system.update`, `void_system.update`, `crystal_system.update`) are tick systems on sub-state structs that take `&mut self` + `&mut rng`. Converting these to Rules requires either: (a) making them return Effects that get applied to the sub-state, or (b) treating the entire sub-state update as a single coarse Effect. Option (b) is pragmatic but undermines the "inspectable trace" goal. Option (a) means every tick system needs its own Effect variants.

### Verdict on feasibility

Phases 0–1: **High confidence.** Straightforward, low risk.
Phase 2: **Medium confidence.** Combat is complex but bounded. Expect 2–3x the estimated effort.
Phase 3: **Medium confidence.** Movement is already well-structured in `MovementSystem` (separate file, clean methods). The `handle_tile_effects` and `pickup_items` methods are natural Reaction candidates.
Phase 4: **Lower confidence.** end_turn decomposition touches everything. This is where the migration will either prove itself or stall.
Phases 5–6: **Depends on 0–4.** If the pattern works, these are mechanical.

---

## 3. Risks the Proposal Misses

### 3.1 Effect enum explosion

The proposal acknowledges this as "Medium" risk but underestimates it. Let's count:

`use_item` alone needs: DeductAP, HealEntity, ReduceRefraction, SuppressAdaptations, RevealMap, EmitAriaEvent, AddLightEnergy, TeachLightManipulation, AddVoidExposure, AddVoidEnergy, TeachCrystalResonance, AddResonanceEnergy, PlaceCrystal, RemoveItem, OpenBook, LogMessage. That's 16 variants for one action.

Combat adds: SetProvoked, TriggerSwarmAggro, DamageEnemy, SpawnDamageNumber, TriggerHitFlash, TriggerOnHitEffect, ReflectDamage, SetLastDamageDealt, KillEnemy, AwardXP, SpawnSplitEnemies, TriggerOnDeathEffect, DiscoverEnemy. Another 13.

Movement adds: ResetWaitCounter, MoveEntity, ClearStormHighlight, PickupItem, SpawnDecoy, DamageFromGlass, AddRefraction, ReduceAP (from Glare), CheckAdaptationThreshold, TravelToTile. Another 10.

We're at ~40 variants before touching storm, AI, status effects, quests, or skills. A realistic Effect enum for the full game is 80–120 variants. This is manageable in Rust (enums are cheap), but it creates a maintenance burden: every new game mechanic requires adding Effect variants, updating `apply_effect`, updating trace serialization, and updating DES assertion matching.

**Mitigation the proposal should add:** Consider a hybrid approach where some Effects are domain-specific enums (CombatEffect, MovementEffect, ItemEffect) composed into a top-level Effect enum. This keeps each domain's variants co-located with the Rules that produce them.

### 3.2 Visual/UI effects in the trace

The current code interleaves game logic with visual feedback: `trigger_hit_flash`, `spawn_damage_number`, `spawn_projectile`, `log_typed`. Under ESCAEV, these become Effects. But visual effects are fundamentally different from state mutations — they're ephemeral, they don't affect game logic, and they shouldn't be in the same trace as gameplay effects.

The proposal doesn't distinguish between "state-mutating effects" and "presentation effects." This matters for testing: a Rule unit test shouldn't need to assert on damage number positions. And it matters for the trace: a trace that mixes `DamageEnemy(5)` with `SpawnDamageNumber(x=10, y=5, amount=5, is_crit=false)` is noisy.

**Suggestion:** Split Effects into `GameEffect` (state mutations, traced) and `PresentationEffect` (visual feedback, not traced). Rules return both, but only GameEffects go into the Trace.

### 3.3 The spatial index problem

GameState maintains several spatial indices: `enemy_positions`, `npc_positions`, `item_positions`, `chest_positions`, `interactable_positions`. These are rebuilt by `rebuild_spatial_index()` and are `#[serde(skip)]` — they're derived state.

Under ESCAEV, these should be Derives. But many Rules need to query spatial indices (e.g., `enemy_at(x, y)` is used everywhere in combat and movement). If a Rule produces a `MoveEnemy` effect and a subsequent Rule in the same phase needs `enemy_at()`, the spatial index must be updated between Rules. This creates a dependency: some Derives must run mid-phase, not just post-phase.

The proposal's grammar says "Derives run after effect application" (rule 6). But Sequential Execution (rule 8) says "Rules in a sequence see interleaved post-application state." If spatial indices are Derives, they need to run after each effect application in a sequence, not just at phase boundaries. This is a real tension in the grammar.

### 3.4 Logging as a side effect

The current code calls `self.log()` and `self.log_typed()` extensively — there are probably 100+ log calls across the codebase. Under ESCAEV, every log message becomes an Effect (or a PresentationEffect per suggestion 3.2). This means every Rule must include log messages in its return value, which makes Rules verbose and harder to read.

Alternative: Rules return game effects only. The application layer generates log messages based on the effects it applies. `apply_effect(HealEntity(Player, 20))` automatically logs "You heal for 20 HP." This keeps Rules clean but means log messages are coupled to the application layer, not the Rules.

### 3.5 The `process_events` cascade loop

`process_events` (state.rs line 1793) loops up to 10 times, draining events and dispatching them to LootSystem and QuestSystem, which may emit more events. Under ESCAEV, this becomes the Reaction dispatch loop. But the proposal doesn't specify a cascade depth limit for Reactions. If Reaction A produces Effect X, which triggers Reaction B, which produces Effect Y, which triggers Reaction A again — you have infinite recursion.

The current code has a hard limit of 10 iterations. The proposal should specify an equivalent limit for Reaction cascades, and define what happens when the limit is hit (error? truncate? log warning?).

### 3.6 Test migration burden

The proposal says "existing DES scenarios continue to pass." This is true if the refactored code produces identical observable behavior. But DES scenarios assert on specific state values (`player.hp == 80`, `enemies.len() == 0`). If the migration changes the order of operations even slightly (e.g., applying effects in a different order than the original imperative code), deterministic RNG consumption may shift, producing different random outcomes. This would break DES scenarios that depend on specific RNG sequences.

This is the most insidious risk: the migration is supposed to be behavior-preserving, but Rust's deterministic RNG means even reordering two `rng.gen_range()` calls changes all subsequent random values. Every Rule must consume RNG in exactly the same order as the original code.

---

## 4. Rust-Specific Concerns

### 4.1 The borrow checker will fight Rule signatures

The proposal's example Rule signature:
```rust
pub fn use_item_rule(
    item_idx: usize,
    player: &PlayerState,
    item_defs: &DataLoader<ItemDef>,
    rng: &mut ChaCha8Rng,
) -> Vec<Effect>
```

This works because `PlayerState` and `DataLoader<ItemDef>` are separate types. But consider `attack_melee_rule`:
```rust
pub fn attack_melee_rule(
    target_x: i32, target_y: i32,
    player: &PlayerState,        // need player.ap, player.equipped_weapon, player.skills, player.adaptations
    enemies: &[Enemy],           // need enemy stats, behaviors
    enemy_positions: &HashMap<(i32, i32), usize>,  // need spatial lookup
    rng: &mut ChaCha8Rng,
) -> Vec<Effect>
```

At the call site in `state.rs`, you need:
```rust
let effects = attack_melee_rule(
    x, y,
    &self.player,           // borrows self.player
    &self.world.enemies,    // borrows self.world.enemies
    &self.enemy_positions,  // borrows self.enemy_positions
    &mut self.rng,          // mutably borrows self.rng
);
```

This actually works in Rust because these are disjoint field borrows. But it gets ugly fast. For `end_turn` phases, a Rule might need `&self.player`, `&self.world.enemies`, `&self.world.map`, `&self.world.storm`, `&self.world.npcs`, `&self.enemy_positions`, and `&mut self.rng` — seven arguments destructured from GameState.

**Practical solution:** Define a `QueryContext` struct that bundles read-only references:
```rust
struct QueryContext<'a> {
    player: &'a PlayerState,
    world: &'a WorldState,
    enemy_positions: &'a HashMap<(i32, i32), usize>,
    // ... other read-only state
}
```
Rules take `(command_args, &QueryContext, &mut ChaCha8Rng) -> Vec<Effect>`. The call site constructs QueryContext once per command dispatch. This is ergonomic and borrow-checker friendly because QueryContext holds only shared references while rng is the sole mutable borrow.

### 4.2 Effect enum ownership

Effects need to carry data. Some data is cheap to clone (i32, usize, bool), some is not (String, Vec). Consider:
```rust
enum Effect {
    LogMessage(String, MsgType),        // String allocation per log
    SpawnEnemy(String, i32, i32),       // enemy_id clone
    PlaceCrystal(i32, i32, CrystalFrequency),  // fine, Copy types
}
```

Every `use_item` call that logs a message allocates a String in the Effect, then the application layer reads it and pushes it to the message log. This is fine for a TUI roguelike (performance is irrelevant), but it means Effects own their data. Rules can't return references into the query state because the Effects outlive the query borrows.

This is a non-issue for performance but affects API design: all Effect variants must use owned types. No `&str`, no `&[Enemy]`. The proposal's examples already do this implicitly, but it should be stated as a design constraint.

### 4.3 Trait objects vs enums for Rules

The proposal uses free functions for Rules. This is the right call for this codebase. Trait objects (`Box<dyn Rule>`) would add indirection, make testing harder, and fight the borrow checker (trait objects with lifetime parameters are painful). Enums for command dispatch + free functions for rule logic is idiomatic Rust.

### 4.4 The `&mut self` to sub-state extraction path

The proposal mentions extracting sub-states (CombatState, NarrativeState) but defers it to after Phase 4. This is correct — premature sub-state extraction would create a second migration front. But the proposal should note that sub-state extraction is what makes the QueryContext pattern (4.1) truly clean. Without sub-states, QueryContext just borrows fields from the flat GameState struct. With sub-states, QueryContext borrows entire sub-state structs, which is more natural.

Recommended order: Phase 0–1 (prove pattern) → Phase 2–3 (expand) → extract sub-states → Phase 4 (end_turn, now with clean sub-state borrows).

---

## 5. Testing Claims

### Can AI agents actually write Rule unit tests?

**Yes, and this is the proposal's strongest claim.** A Rule is a pure function: given these inputs, assert these outputs. AI agents are excellent at generating exhaustive input/output test cases for pure functions. The `use_item_rule` test example in the proposal is realistic — an agent can enumerate item types and assert expected effects mechanically.

### Can AI agents write DES trace tests?

**Partially.** Trace assertions (`expect_effects`) are more powerful than state assertions (`at_end`), but they're also more brittle. If the implementation changes which Effects are produced (e.g., adding a new log message effect), trace tests break even if behavior is preserved. The proposal should recommend asserting on a subset of expected effects (the important ones) rather than the complete trace.

### Will the self-verification protocol prevent scaffold-and-abandon?

**Mostly.** The key insight is correct: if a DES trace test must assert that specific Effects were produced, an unwired system will fail the trace test. But there's a gap: an AI agent could write a Rule that produces the right Effects, wire it into the command dispatch, but implement `apply_effect` for those Effects as no-ops. The trace would show the Effects, the test would pass, but nothing actually happens in the game.

**Mitigation:** Require that DES scenarios for new systems include both `expect_effects` (trace) AND `at_end` (state) assertions. The trace proves the Rule ran; the state assertion proves the Effects were applied correctly.

### The fake scenario problem

The proposal correctly identifies the 7 fake DES scenarios as a symptom. ESCAEV's trace assertions would catch this — a scenario that only asserts `player_alive` would need to also assert specific Effects. But this only works if there's a policy requiring trace assertions for new scenarios. The structural gate is the policy, not the architecture.

---

## 6. Migration Ordering

### The proposed order is mostly correct, with one adjustment

The proposal orders: Foundation → use_item → combat → movement → end_turn → DES traces → dead code.

**Problem:** Dead code cleanup (Phase 6) should come before Phase 1, not after Phase 5. Here's why:

1. The 3,600 LOC of dead code includes half-wired systems (light, crystal, void) that `use_item` currently integrates with (lines 2720–2780 in state.rs). If you extract `use_item_rule` in Phase 1, you're encoding the dead light/crystal/void energy effects into the new Effect enum. Then in Phase 6, you decide to remove those systems — and now you have to remove Effect variants, Rule logic, and tests you just wrote.

2. The 4 dead algorithms, orphaned schemas, and duplicate pattern files are noise that makes the codebase harder to navigate during migration. Removing them first reduces cognitive load.

**Recommended order:**
- Phase 0: Foundation (Effect enum, Trace, apply_effect)
- Phase 0.5: Dead code triage — delete confirmed dead code, mark half-wired systems with `#[deprecated]`, decide keep/remove for light/crystal/void BEFORE encoding them into Effects
- Phase 1: use_item extraction (now without dead system baggage)
- Phase 2: Combat
- Phase 3: Movement
- Phase 3.5: Sub-state extraction (CombatState, etc.) — this makes Phase 4 cleaner
- Phase 4: end_turn decomposition
- Phase 5: DES trace assertions

### Phase 2 and 3 could be swapped

Movement (`MovementSystem`) is already better structured than combat. It's in its own file, has clean method boundaries (`handle_npc_interaction`, `handle_enemy_combat`, `handle_movement`, `handle_tile_effects`, `pickup_items`), and each method maps naturally to a Rule or Reaction. Combat is messier — `attack_melee` in `systems/combat.rs` is 80 lines of interleaved logic.

Doing movement before combat would give a second "easy win" after use_item, building confidence in the pattern before tackling the hardest system.

---

## 7. Scope Assessment

### Is this too ambitious?

**The vocabulary and grammar are about right.** 8 atomics and 10 composition rules is a reasonable formalism for this codebase. The stress testing against 5 systems with 15 friction points shows genuine rigor.

**The migration scope is ambitious but not unreasonable** — IF phases are treated as hard stops with independent value. The proposal says this, but it needs to be enforced. The real risk is Phase 4 (end_turn) expanding to touch everything.

### What's missing from scope

1. **Save/load compatibility.** The proposal doesn't address whether saves from pre-ESCAEV code will load in post-ESCAEV code. If GameState's struct changes (e.g., sub-state extraction), old saves break. This needs a migration strategy or a version field in saves.

2. **IPC impact.** The proposal mentions IPC could receive effect streams instead of state snapshots (Section 10, question 5). But during migration, the IPC protocol must continue working. If state.rs changes how it updates fields, the IPC serialization must still produce valid snapshots. This is probably fine but should be explicitly verified.

3. **Compile time impact.** A 80–120 variant Effect enum with `match` in `apply_effect` will increase compile times for state.rs (which is already the largest file). Consider putting the Effect enum and apply_effect in a separate module to isolate recompilation.

---

## 8. Alternative Approaches

### 8.1 Lightweight alternative: just expand the event system

The simplest path to the same goals: expand `GameEvent` to carry mutation data, make `process_events` the central dispatch loop, and have systems return events instead of mutating state directly. This is ESCAEV without the formalism — no Trace, no Rule/Reaction distinction, no Phase Sequences.

**Pros:** Much less new infrastructure. Builds on existing code. Faster to implement.
**Cons:** No trace (can't inspect what happened). No Rule unit testing (events still go through the full dispatch loop). Doesn't prevent scaffold-and-abandon (no structural gate).

**Verdict:** This gets you 40% of ESCAEV's value for 20% of the effort. Worth considering as a stepping stone if Phase 1 proves harder than expected.

### 8.2 Command pattern without full event sourcing

Define Commands (player actions) and a `dispatch(command) -> Vec<Mutation>` function. Mutations are applied mechanically. No Reactions, no Derives, no Traces. Just "action in, mutations out."

**Pros:** Simpler than ESCAEV. Gets you Rule unit testing. Prevents direct state mutation.
**Cons:** No emergence from Reactions. No trace for DES. Cross-system interactions (combat death → loot → quest) must be handled explicitly in the dispatch function, recreating the orchestrator problem.

**Verdict:** This is ESCAEV Phase 1 without the rest. If you're going to do Phase 1, you might as well design for the full architecture.

### 8.3 Actual ECS (Bevy/hecs)

The proposal dismisses ECS. I agree with the dismissal for this project. The codebase is 48.5k LOC with a working game. Migrating to Bevy's ECS would be a rewrite, not a refactor. And the testability argument is valid — ECS system scheduling is hard for AI agents to reason about.

---

## 9. Specific Suggestions

### 9.1 Add a QueryContext struct (see section 4.1)

This is the single most important ergonomic improvement. Without it, every Rule call site will be a wall of destructured field borrows.

### 9.2 Split Effects into GameEffect and PresentationEffect (see section 3.2)

Keep the trace clean. Don't mix `DamageEnemy(5)` with `SpawnDamageNumber(10, 5, 5, false)`.

### 9.3 Domain-scoped Effect enums

```rust
enum Effect {
    Combat(CombatEffect),
    Movement(MovementEffect),
    Item(ItemEffect),
    Storm(StormEffect),
    // ...
}
```

Each domain defines its own variants. `apply_effect` delegates to domain-specific appliers. This keeps the top-level enum manageable and co-locates Effect definitions with the Rules that produce them.

### 9.4 Move dead code cleanup to Phase 0.5

Don't encode dead systems into the new architecture. Decide their fate first.

### 9.5 Add a Reaction cascade depth limit to the grammar

Specify: "Reaction cascades are limited to N levels (default 10). Exceeding the limit logs a warning and truncates." This matches the existing `process_events` behavior.

### 9.6 Specify RNG consumption ordering as a migration constraint

Add to the migration principles: "Each Rule must consume RNG in the same order as the original imperative code it replaces. DES scenarios with deterministic seeds are the regression test for this."

### 9.7 Start with a smaller Effect enum

Phase 1 doesn't need 16 Effect variants for use_item. Start with the 5 most common (DeductAP, HealEntity, RemoveItem, AddRefraction, LogMessage) and handle the rest as a single `LegacyMutation(Box<dyn FnOnce(&mut GameState)>)` escape hatch. This lets you prove the pattern without boiling the ocean on Effect variants. Replace LegacyMutation variants with proper Effects incrementally.

### 9.8 Consider sub-state extraction between Phase 3 and Phase 4

The proposal defers sub-state extraction to "after Phase 4." But Phase 4 (end_turn) is where sub-states would help the most — each phase in the end_turn sequence operates on a specific sub-state. Extracting sub-states before Phase 4 makes the decomposition cleaner.

### 9.9 Add a "migration test" DES scenario category

Create a set of DES scenarios specifically designed to catch RNG ordering changes. These scenarios use fixed seeds and assert on specific numeric outcomes (exact damage values, exact enemy positions). If any migration phase changes RNG consumption order, these scenarios fail immediately.

---

## 10. Summary Verdict

The ESCAEV proposal is **well-researched, honest about the problem, and architecturally sound**. The vocabulary and grammar are the right level of abstraction for this codebase. The stress testing against real systems shows it wasn't designed in a vacuum.

**Strengths:**
- Correctly identifies the core problem (verifiability, not performance)
- Builds on what already works (pure decision functions, event system, DES)
- Incremental migration with hard phase boundaries
- The testing architecture is the most valuable part — Rule unit tests are a genuine improvement

**Weaknesses:**
- Underestimates the borrow checker friction at Rule call sites (fixable with QueryContext)
- Doesn't address Effect enum scaling (fixable with domain-scoped enums)
- Mixes presentation effects with game effects in the trace
- Dead code cleanup should precede Rule extraction, not follow it
- end_turn fan-out is understated (19 calls, not 11) — Phase 4 is bigger than estimated
- RNG ordering sensitivity during migration is unaddressed
- No save/load compatibility strategy

**Recommendation:** Proceed with the proposal, incorporating the adjustments above. Start with Phase 0 + Phase 0.5 (foundation + dead code triage). If Phase 1 (use_item) succeeds cleanly, the architecture has proven itself and the remaining phases are execution, not design risk.

The biggest risk is not the architecture — it's scope creep. The proposal's own advice ("Phase boundaries are hard stops") is the most important sentence in the document. Enforce it.
