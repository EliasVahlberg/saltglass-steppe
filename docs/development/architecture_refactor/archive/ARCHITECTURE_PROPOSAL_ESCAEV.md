# Architecture Proposal: ESCAEV

**Event-Sourced Command Architecture with Effect Verification**

> Status: PROPOSAL — awaiting review
> Author: Elias + Kiro CLI (system-agent)
> Date: 2026-04-03

---

## 1. Problem Statement

Saltglass Steppe is developed by one person using AI coding agents. The most critical architectural constraint is not runtime performance or feature richness — it is **verifiability**. AI agents must be able to self-verify their work: confirm that new code does what it claims, that existing behavior is preserved, and that cross-system interactions produce correct results.

The current architecture has three structural problems that undermine verifiability:

1. **Single orchestrator bottleneck.** `state.rs` (3,525 LOC, 163 methods, 16 concerns) is the only cross-concern coordinator. All game logic flows through it. An AI agent cannot reason about what a change to state.rs affects without tracing every call chain manually.

2. **Opaque side effects.** Methods like `use_item` (160 LOC, 10+ concerns) and `end_turn` (fan-out 11) mutate state directly and broadly. There is no record of what changed or why. Testing requires setting up full game state, executing actions, and asserting on the resulting state — expensive integration tests that are brittle and hard to maintain.

3. **Scaffold-and-abandon pattern.** The codebase health audit (2026-04-03) identified ~3,600 LOC of dead or half-wired code, all following the same pattern: AI agents generate complete vertical slices (struct + methods + UI + tests) but fail at horizontal integration — threading new systems through existing orchestration code. The architecture provides no structural gate that catches incomplete integration.

The Debug Execution System (DES) was built to address verifiability. It works — 101 of 162 scenarios are meaningful. But DES tests at the integration level only. There is no unit-test layer for game logic. When a DES scenario fails, the agent must debug the entire turn loop to find the cause.

### What we want

An architecture where:
- Every game action produces an inspectable, verifiable record of what it changed
- AI agents can write unit tests for individual game rules without setting up full game state
- New systems cannot be committed without proving they connect to gameplay
- Emergent behavior (the CoQ/DF quality) arises from composition of simple, testable parts
- The existing codebase migrates incrementally — no rewrite

### Why not ECS?

Full ECS (Bevy/Specs-style) was considered. It provides emergent behavior through component composition but reduces testability in this context:
- Behavior emerges from system scheduling order and component queries — hard for an AI agent to reason about
- Debugging requires reconstructing which systems ran in what order on which entity
- Edge cases emerge from component combinations that were never anticipated
- Testing requires either exhaustive scenario coverage or property-based verification

The codebase already has strong testable primitives: pure decision functions (`combat.rs`, `encounter.rs`, `adaptation.rs`), a data-driven content pipeline, and an event system (underused — only LootSystem and QuestSystem subscribe). ESCAEV formalizes and extends what already works.

---

## 2. The Architecture

ESCAEV is defined as a vocabulary (8 atomics) and a grammar (10 composition rules). The vocabulary was stress-tested against 5 real systems in the codebase (combat, movement, storm, AI, end_turn) with 15 friction points identified and resolved.

### 2.1 Vocabulary (Atomics)

| # | Atomic | What it is | Current codebase equivalent |
|---|--------|-----------|---------------------------|
| 1 | **State Facet** | A typed slice of game state. Not "GameState" — a specific concern's data. `player.hp`, `map.tiles`, `quest_log`. | Fields on GameState, PlayerState, WorldState |
| 2 | **Query** | A pure read of one or more facets. Returns data, never mutates. RNG is a privileged facet: consumed, recorded in trace. | `enemy_at()`, `get_item_def()`, `action_cost()` |
| 3 | **Command** | A player intention or system trigger. Contains the *what*, not the *how*. Tagged with source (Player, System, Reaction). | `Action` enum variants, `end_turn()` call |
| 4 | **Effect** | An atomic state mutation. One facet, one change. Supports granularity levels (coarse ↔ fine). | `GameEvent` variants (but currently used for notification, not mutation) |
| 5 | **Rule** | A pure function: `(Command, Queries, RNG) → Vec<Effect>`. Where game logic lives. Deterministic given seed. | `roll_attack()`, `calc_hit_chance()`, the logic inside `use_item` |
| 6 | **Reaction** | A pure function: `(Effect, Queries, RNG) → Vec<Effect> or DeferredCommand`. Where emergence lives. Timing: immediate or deferred. | `LootSystem.on_event()`, `QuestSystem.on_event()` |
| 7 | **Derive** | Cache recomputation: `(State Facets) → derived state`. Runs post-application. Not a mutation — a recomputation. | `update_fov()`, `update_lighting()`, `rebuild_spatial_index()` |
| 8 | **Trace** | The ordered record of all effects + their sources. Supports zoom levels (coarse/fine). The complete causal record. | No current equivalent — this is new |

### 2.2 Grammar (Composition Rules)

1. **Commands produce Effects only through Rules.** A command never mutates state directly.
2. **Rules are deterministic** given (Queries + RNG seed). Same inputs, same outputs, always.
3. **Effects are applied in order.** The application layer is mechanical — no logic, no branching.
4. **Reactions trigger on Effects.** Timing is explicit: immediate (after each effect) or deferred (at phase boundary).
5. **Reactions produce Effects or DeferredCommands.** DeferredCommands are queued for the next orchestration cycle, not executed inline.
6. **Derives run after effect application.** They read state, recompute caches. No game logic.
7. **Phase Sequence:** An ordered list of Commands/Derives. Each phase sees post-application state of all previous phases. (Example: `end_turn` is a 9-phase sequence.)
8. **Sequential Execution:** Rules in a sequence see interleaved post-application state. (Example: AI turn processes enemies 0..N, each seeing the results of previous enemies.)
9. **Priority Chain:** Try rules in order; first rule producing non-empty effects wins. (Example: movement dispatcher tries NPC interaction → combat → actual movement.)
10. **The Trace records everything.** Coarse effects expand to fine effects on demand. Nothing happens off-trace.

### 2.3 What the Grammar Forbids

- A Rule calling another Rule directly (must go through Effects → Reactions)
- State mutation outside the application layer
- Logic in the application layer (it is a mechanical `match effect { ... }` that writes fields)
- Effects that conditionally produce different mutations (effects are atomic and deterministic)
- Unrecorded state changes (everything is on the trace)

---

## 3. How It Maps to the Current Codebase

### 3.1 What already fits

The audit's computational taxonomy classified every module. Many already match ESCAEV atomics:

| Current pattern | ESCAEV atomic | Modules |
|----------------|---------------|---------|
| `DATA-DEF` | State Facet | map.rs, world_map.rs, entity.rs, all data loader modules (22 total) |
| `DECISION-FN` | Rule | combat.rs (`roll_attack`, `calc_hit_chance`), encounter.rs, adaptation.rs, progression.rs, travel.rs |
| `EVENT-ROUTER` | Reaction | LootSystem.on_event, QuestSystem.on_event |
| `TICK-SYSTEM` | Rule (per-turn) | StatusEffectSystem, StormSystem tick, psychic/skills/light/void/crystal ticks |
| `DATA-XFORM` | Rule (generation) | tile_generator.rs, terrain_forge_adapter.rs, connectivity.rs, settlement/*, spawn.rs |
| `RENDER-COMPOSE` | (unchanged) | All renderer/ modules — read-only, outside ESCAEV scope |
| `INPUT-DISPATCH` | Command source | ui/input.rs, all menu input handlers |

### 3.2 What changes

| Current pattern | Problem | ESCAEV change |
|----------------|---------|---------------|
| `STATE-ORCHESTRATOR` (state.rs only) | All cross-concern logic is opaque | Orchestration becomes Phase Sequences of Commands. Logic moves into Rules. state.rs becomes thin: apply effects, run derives, execute phase sequences. |
| `use_item` (160 LOC, 10+ concerns) | Decides AND mutates in one method | Split into Rule (`use_item_rule`: queries → effects) and Application (mechanical writes). |
| `end_turn` (fan-out 11) | Calls 11 subsystems directly | Becomes a Phase Sequence of 9 Commands. Each phase is independently testable. |
| `GameEvent` (notification only) | Events notify but don't drive mutations | Effects replace direct mutations. GameEvent becomes a subset of Effect (the notification kind). |
| FOV/lighting recomputation | Tangled into effect flow | Becomes Derive — explicit post-application cache recomputation. |
| No trace | Can't inspect what happened | Every action produces a Trace. DES asserts on traces, not just final state. |

### 3.3 What doesn't change

- **Renderer** — read-only, outside ESCAEV scope
- **Data loading** — `DataLoader<T>`, JSON schemas, `once_cell` statics stay as-is
- **DES** — gains trace-assertion capability but existing scenarios keep working
- **Deterministic RNG** — ChaCha8Rng stays, becomes a privileged facet in Rules
- **Data-driven content** — JSON configs, no code changes for content, unchanged
- **Multi-terminal IPC** — orthogonal to ESCAEV

---

## 4. Concrete Example: `use_item` Refactored

Current (state.rs, 160 LOC, direct mutation):
```rust
pub fn use_item(&mut self, idx: usize) -> bool {
    // 160 lines of interleaved queries, decisions, and mutations
    // touching HP, AP, refraction, adaptations, map, light, void, crystal, inventory, events
}
```

Proposed (split into Rule + Application):
```rust
// Rule: pure function, no mutation
pub fn use_item_rule(
    item_idx: usize,
    player: &PlayerState,
    item_defs: &DataLoader<ItemDef>,
    rng: &mut ChaCha8Rng,
) -> Vec<Effect> {
    let id = &player.inventory[item_idx];
    let def = item_defs.get(id)?;
    let mut effects = vec![DeductAP(Player, action_cost("use_item"))];

    if def.heal > 0 {
        effects.push(HealEntity(Player, def.heal.min(player.max_hp - player.hp)));
    }
    if def.void_exposure > 0 {
        effects.push(AddVoidExposure(Player, def.void_exposure));
    }
    // ... each concern adds effects conditionally
    if def.consumable {
        effects.push(RemoveItem(Player, item_idx));
    }
    effects
}

// Application: mechanical, no logic
pub fn apply_effect(state: &mut GameState, effect: &Effect) {
    match effect {
        DeductAP(entity, amount) => { /* write to entity.ap */ }
        HealEntity(entity, amount) => { /* write to entity.hp */ }
        AddVoidExposure(entity, amount) => { /* write to entity.void_system */ }
        RemoveItem(entity, idx) => { /* remove from inventory */ }
        // ...
    }
}
```

Test (unit level — no full GameState needed):
```rust
#[test]
fn healing_herb_produces_correct_effects() {
    let player = PlayerState { hp: 60, max_hp: 100, ap: 4, inventory: vec!["healing_herb"], .. };
    let effects = use_item_rule(0, &player, &ITEM_DEFS, &mut rng_from_seed(42));
    assert_eq!(effects, vec![
        DeductAP(Player, 1),
        HealEntity(Player, 20),
        RemoveItem(Player, 0),
    ]);
}
```

Test (trace level — DES integration):
```json
{
    "action": { "type": "use_item", "slot": 0 },
    "expect_effects": [
        { "type": "DeductAP", "entity": "player", "amount": 1 },
        { "type": "HealEntity", "entity": "player", "amount": 20 },
        { "type": "RemoveItem", "entity": "player", "index": 0 }
    ],
    "expect_no_effects": [
        { "type": "DamageEntity" }
    ]
}
```

---

## 5. Stress Test Results

The vocabulary was tested against 5 systems. 15 friction points were identified. Summary:

### Held firm (no vocabulary changes needed)
- Reaction chains (combat death → loot → quest)
- Behavior dispatch (AI strategy pattern)
- Tile effects (Glass damage, Glare AP loss as Reactions to MoveEntity)
- Item pickup (Reaction to MoveEntity)
- Adaptation threshold (Reaction to AddRefraction)
- Individual enemy decisions (each enemy's turn is a Rule)

### Bent but workable (minor adjustments)
- **RNG purity**: RNG is a privileged facet — Rules consume it, trace records consumption. Rules are deterministic given seed.
- **Conditional branching in Rules**: Fine — Rules branch internally but are deterministic given (queries + seed).
- **Movement dispatcher**: Branching is in query results (npc_at? enemy_at? walkable?), not hidden state. Still a Rule.
- **System-generated Commands**: Commands have a source tag (Player, System, Reaction).
- **Sub-state replacement** (ForecastNextStorm): Accepted as an atomic Effect on the storm facet.

### Required vocabulary additions (resolved)
- **Derive** (8th atomic): FOV/lighting are cache recomputation, not effects. Added as explicit post-application phase.
- **DeferredCommand**: World transitions (Reaction producing a Command) resolved by allowing Reactions to queue commands for the next cycle.
- **Effect granularity**: Storm bulk mutations (100+ tile changes) resolved with coarse/fine zoom levels. Coarse: `ApplyStormEdit(Glass, intensity, seed)`. Fine: individual `SetTile` effects.
- **Sequential Execution**: AI turn requires interleaved application between per-enemy Rules. Added as grammar rule.
- **Priority Chain**: Movement dispatcher tries rules in order. Added as grammar rule.
- **Phase Sequence**: `end_turn` is 9 phases. Added as grammar rule.
- **Reaction timing**: Immediate vs deferred processing. Made explicit in grammar.

---

## 6. Testing Architecture Under ESCAEV

### 6.1 Test Layers

| Layer | What it tests | Who writes it | Shape |
|-------|--------------|---------------|-------|
| **Rule unit test** | Single rule, specific inputs → expected effects | AI agent (mechanical) | `assert_eq!(rule(inputs), expected_effects)` |
| **Reaction unit test** | Single reaction, trigger effect → expected effects | AI agent (mechanical) | `assert_eq!(reaction(trigger, queries), expected_effects)` |
| **Application test** | Effect applied → facet changed correctly | One-time, per effect type | `apply(state, effect); assert_eq!(state.facet, expected)` |
| **Trace test (DES)** | Full command → expected trace | AI agent or human | `assert_eq!(trace(command, state), expected_trace)` |
| **Integration test** | Multi-turn scenario → final state | Human-designed, AI-maintained | Existing DES scenarios (unchanged) |

### 6.2 Self-Verification Protocol

When an AI agent implements a new Rule:

1. **Write the Rule** — pure function, queries → effects
2. **Write Rule unit tests** — enumerate key input combinations, assert expected effects
3. **Write at least one DES trace test** — full command through the system, assert trace contains expected effects and does NOT contain unexpected effects
4. **The trace test is the integration gate** — if the trace doesn't contain the expected effects, the Rule isn't wired into the system

This replaces the current pattern where DES scenarios assert on final state (which can pass even when the system under test did nothing, as the fake scenarios demonstrated).

### 6.3 DES Evolution

DES gains a new assertion type: `expect_effects`. Existing `at_end` assertions continue to work. The two complement each other:

- `expect_effects`: "did the right things happen?" (causal verification)
- `at_end` assertions: "is the world in the right state?" (outcome verification)

A scenario that asserts both is stronger than either alone.

---

## 7. Migration Path

### Principles
- Incremental. No rewrite. Each step compiles and passes existing tests.
- Worst offenders first. `use_item` and `end_turn` are the highest-value targets.
- Event system expansion is the prerequisite — it's the mechanism that enables decoupling.

### Phase 0: Foundation (no behavior change)
1. Define the `Effect` enum — start with effects needed by `use_item`
2. Define the `Trace` struct — ordered vec of effects with source tags
3. Add `apply_effect()` — mechanical match on Effect variants
4. Add trace recording to GameState (opt-in, behind a flag for zero-cost when not testing)

### Phase 1: First Rule extraction (`use_item`)
1. Extract `use_item_rule()` — pure function returning `Vec<Effect>`
2. `use_item()` becomes: call rule, apply effects, record trace
3. Write Rule unit tests for `use_item_rule`
4. Write DES trace tests for `use_item`
5. Existing DES scenarios continue to pass (behavior unchanged)

### Phase 2: Expand to combat
1. Extract `attack_melee_rule()`, `ranged_attack_rule()`
2. Extract `process_enemy_death` as a Reaction to `KillEnemy` effect
3. Swarm aggro, reflect damage become Reactions
4. Write Rule + trace tests

### Phase 3: Expand to movement
1. Extract `move_to_tile_rule()` — produces MoveEntity, tile effects
2. Item pickup becomes Reaction to MoveEntity
3. Tile effects (Glass, Glare) become Reactions to MoveEntity
4. FOV/lighting become Derives (explicit post-application)

### Phase 4: `end_turn` decomposition
1. Define the Phase Sequence for end_turn
2. Each phase becomes a Command with its own Rule
3. Storm tick, AI turn, status ticks all become Rules producing Effects
4. `process_events` becomes the Reaction dispatch loop

### Phase 5: Trace-based DES
1. Add `expect_effects` assertion type to DES
2. Rewrite the 7 fake scenarios as real trace tests
3. Add trace tests for systems with zero coverage (encounter, crafting, skills, trading)

### Phase 6: Dead code cleanup
1. Remove dead code identified in the audit (guided by the audit's policy recommendations)
2. Half-wired systems (light, crystal, void) get a decision: wire via ESCAEV Reactions, or remove
3. If wired: each ability becomes a Rule, input dispatch becomes a Command, the missing integration path is explicit

### What each phase delivers
- Phase 0: infrastructure, no behavior change
- Phase 1: proof of concept, one system fully ESCAEV
- Phase 2: combat is testable at the Rule level
- Phase 3: movement cross-concern interactions are explicit
- Phase 4: the turn loop is decomposed and each phase is testable
- Phase 5: DES becomes a trace verification tool
- Phase 6: dead code is resolved, not deferred

---

## 8. Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Effect enum becomes enormous | Medium | Use granularity levels. Domain-specific effects for readability, generic effects for rare cases. Review enum size at each phase. |
| Effect ordering matters (heal then damage ≠ damage then heal) | Medium | Effects from one Rule are applied in list order. Document ordering semantics per Rule. |
| Performance cost of allocating effect vectors | Low | TUI roguelike — negligible. Trace recording is opt-in (testing only). |
| Migration breaks existing DES scenarios | Low | Each phase preserves behavior. Existing assertions test outcomes, not implementation. |
| Vocabulary doesn't cover a future system | Medium | The vocabulary was designed to be extended. New atomics can be added if stress-testing reveals gaps. |
| Refactor scope creep | High | Phase boundaries are hard stops. Each phase is independently valuable. Don't start Phase N+1 until Phase N is complete and tested. |

---

## 9. Success Criteria

The architecture is successful if:

1. **An AI agent can implement a new game mechanic and verify it works without human testing.** The agent writes a Rule, writes Rule unit tests, writes a DES trace test, and all pass.
2. **The scaffold-and-abandon pattern is structurally prevented.** A new system without a trace test that asserts observable effects cannot be considered complete.
3. **Cross-concern interactions are explicit in the trace.** When `use_item` affects HP, refraction, void energy, and inventory, the trace shows exactly which effects were produced and in what order.
4. **Existing gameplay is preserved.** All current DES scenarios pass after each migration phase.
5. **state.rs shrinks.** Logic moves into Rules. state.rs becomes orchestration (phase sequences) and application (mechanical effect writes).

---

## 10. Open Questions

1. **Effect granularity for storm edits.** Coarse (`ApplyStormEdit(Glass, 4, seed)`) vs fine (`SetTile` × 100). Proposed: default coarse, expand on demand. Needs validation during Phase 4.

2. **Reaction timing default.** Immediate (after each effect) or deferred (at phase boundary)? Current code is deferred (process_events at end of turn). Proposed: keep deferred as default, allow immediate for specific reactions. Needs validation during Phase 2 (combat death → loot).

3. **How much of state.rs to decompose.** The audit identified 22 leaf modules consumed only by state.rs. Extracting sub-states (CombatState, NarrativeState) is possible but should follow event expansion, not precede it. Exact decomposition scope TBD after Phase 4.

4. **Trace storage for save/load.** Should traces be persisted? Useful for replay/debugging but increases save size. Proposed: traces are ephemeral (testing only), not saved. Revisit if replay becomes a feature.

5. **Integration with multi-terminal IPC.** Satellite terminals currently receive full state snapshots. Under ESCAEV, they could receive effect streams instead — more efficient, enables real-time effect visualization. Deferred to post-Phase 4.

---

## Appendix A: Vocabulary Reference Card

```
ATOMICS
  State Facet    typed slice of game state (player.hp, map.tiles, quest_log)
  Query          pure read of facets; RNG is privileged (consumed, recorded)
  Command        player intention or system trigger; tagged with source
  Effect         atomic state mutation; supports coarse/fine granularity
  Rule           (Command, Queries, RNG) → Vec<Effect>; deterministic given seed
  Reaction       (Effect, Queries, RNG) → Vec<Effect> | DeferredCommand
  Derive         (State Facets) → recomputed cache; post-application, no logic
  Trace          ordered record of all effects + sources; supports zoom levels

GRAMMAR
  1. Commands → Effects only through Rules
  2. Rules are deterministic given (Queries + RNG seed)
  3. Effects applied in order; application is mechanical
  4. Reactions trigger on Effects; timing explicit (immediate/deferred)
  5. Reactions produce Effects or DeferredCommands
  6. Derives run post-application; read state, recompute caches
  7. Phase Sequence: ordered Commands/Derives, each sees prior results
  8. Sequential Execution: interleaved application between Rules
  9. Priority Chain: try Rules in order, first non-empty wins
  10. Trace records everything; coarse expands to fine on demand
```

## Appendix B: Friction Points from Stress Testing

| # | System | Friction | Resolution | Category |
|---|--------|---------|------------|----------|
| 1 | Combat | RNG breaks Query purity | RNG is privileged facet | Bent |
| 2 | Combat | Conditional branching in Rules | Deterministic given seed | Bent |
| 3 | Combat | Death cascade complexity | Maps cleanly to Reaction chain | Held |
| 4 | Movement | Dispatcher is a meta-rule | Branching is in query results | Bent |
| 5 | Movement | FOV/lighting are derived state | Added Derive atomic | Added |
| 6 | Movement | World transitions are compound | DeferredCommand from Reaction | Added |
| 7 | Storm | Bulk effects (100+ tiles) | Effect granularity levels | Added |
| 8 | Storm | System-generated commands | Command source tags | Bent |
| 9 | Storm | Sub-state replacement | Atomic effect on storm facet | Bent |
| 10 | AI | Sequential entity processing | Sequential Execution grammar rule | Added |
| 11 | AI | Behavior dispatch | Query → Rule selection | Held |
| 12 | AI | StandardMelee is a god-rule | Decompose into Priority Chain | Added |
| 13 | end_turn | Not a Rule — it's a Phase Sequence | Phase Sequence grammar rule | Added |
| 14 | end_turn | Parallel tick systems | Optimization, not vocabulary issue | Bent |
| 15 | end_turn | Deferred vs immediate reactions | Explicit timing in grammar | Added |
