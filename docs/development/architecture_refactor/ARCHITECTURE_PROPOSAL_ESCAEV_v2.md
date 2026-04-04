# Architecture Proposal: ESCAEV v2

**Event-Sourced Command Architecture with Effect Verification**

> Status: PROPOSAL v2 — incorporates LeadDeveloper review
> Authors: Elias + Kiro CLI (system-agent)
> Date: 2026-04-03
> Previous: [v1](ARCHITECTURE_PROPOSAL_ESCAEV.md) · [v1 Review](ESCAEV_REVIEW.md) · [v2 Review](ESCAEV_REVIEW_v2.md)

### Changes from v1

- Corrected `end_turn` fan-out: 19 calls, not 11 (4 are dead stubs → 15 live after cleanup)
- Corrected `use_item` LOC: ~140, not 160
- Added `QueryContext` struct for ergonomic Rule call sites (review §4.1)
- Split Effect into `GameEffect` and `PresentationEffect` (review §3.2)
- Added domain-scoped Effect enums (review §9.3)
- Moved dead code cleanup to Phase 0.5 (review §6)
- Added Reaction cascade depth limit to grammar (review §3.5)
- Added RNG ordering as explicit migration constraint (review §3.6)
- Added sub-state extraction as Phase 3.5 (review §9.8)
- Added save/load compatibility to risks (review §7)
- Added compile-time isolation strategy (review §7)
- Resolved logging: application-layer auto-generation from GameEffects (review §3.4)
- Strengthened self-verification: require both `expect_effects` AND `at_end` assertions (review §5)

---

## 1. Problem Statement

Saltglass Steppe is developed by one person using AI coding agents. The most critical architectural constraint is **verifiability** — AI agents must self-verify their work.

Three structural problems undermine this:

1. **Single orchestrator bottleneck.** `state.rs` (3,525 LOC, 163 methods, 16 concerns) is the only cross-concern coordinator. An AI agent cannot reason about what a change affects without tracing every call chain.

2. **Opaque side effects.** `use_item` (~140 LOC, 12 concerns) and `end_turn` (fan-out 19, of which 4 are dead stubs) mutate state directly. No record of what changed or why. Testing requires full game state setup — expensive, brittle integration tests.

3. **Scaffold-and-abandon pattern.** The codebase health audit identified ~3,600 LOC of dead/half-wired code from AI-generated batch scaffolding. The architecture has no structural gate catching incomplete integration.

DES (Debug Execution System) addresses verifiability at the integration level — 101 of 162 scenarios are meaningful. But there is no unit-test layer for game logic. When a DES scenario fails, the agent must debug the entire turn loop.

### What we want

- Every action produces an inspectable, verifiable record of what changed
- AI agents can unit-test individual game rules without full game state
- New systems cannot be committed without proving gameplay integration
- Emergent behavior arises from composition of simple, testable parts
- Incremental migration — no rewrite

### Why not ECS?

Full ECS was considered and rejected. Behavior emerges from system scheduling and component queries — hard for AI agents to reason about. The codebase already has strong testable primitives (pure decision functions, data-driven content, an underused event system). ESCAEV formalizes what already works.

---

## 2. The Architecture

### 2.1 Vocabulary (8 Atomics)

| # | Atomic | What it is | Current equivalent |
|---|--------|-----------|-------------------|
| 1 | **State Facet** | Typed slice of game state (`player.hp`, `map.tiles`, `quest_log`) | Fields on GameState/PlayerState/WorldState |
| 2 | **Query** | Pure read of facets. RNG is privileged: consumed, recorded in trace. Bundled via `QueryContext`. | `enemy_at()`, `get_item_def()`, `action_cost()` |
| 3 | **Command** | Player intention or system trigger. Tagged with source (Player, System, Reaction). | `Action` enum variants, `end_turn()` call |
| 4 | **Effect** | Atomic state mutation. Split into `GameEffect` (state, traced) and `PresentationEffect` (visual, not traced). Domain-scoped enums. Supports coarse/fine granularity. | `GameEvent` (notification-only currently) |
| 5 | **Rule** | Pure function: `(Command, &QueryContext, &mut RNG) → (Vec<GameEffect>, Vec<PresentationEffect>)`. Deterministic given seed. Free functions, not trait objects. All data in effects must be owned (no borrowed references). | `roll_attack()`, `calc_hit_chance()`, logic inside `use_item` |
| 6 | **Reaction** | Pure function: `(GameEffect, &QueryContext, &mut RNG) → (Vec<GameEffect>, Vec<PresentationEffect>)` or `DeferredCommand`. Timing: immediate or deferred (default: deferred). Cascade depth limit: 10. | `LootSystem.on_event()`, `QuestSystem.on_event()` |
| 7 | **Derive** | Cache recomputation: `(State Facets) → derived state`. Runs post-application. Can also run mid-sequence when spatial indices are needed between sequential Rules. | `update_fov()`, `update_lighting()`, `rebuild_spatial_index()` |
| 8 | **Trace** | Ordered record of all GameEffects + sources. PresentationEffects excluded. Supports coarse/fine zoom. Ephemeral (testing only, not saved). | New — no current equivalent |

### 2.2 Grammar (10 Composition Rules)

1. **Commands produce Effects only through Rules.** No direct state mutation.
2. **Rules are deterministic** given (QueryContext + RNG seed). Each Rule must consume RNG in the same order as the imperative code it replaces.
3. **Effects are applied in order.** Application is mechanical — no logic, no branching. Log messages are auto-generated by the application layer from GameEffects.
4. **Reactions trigger on GameEffects.** Timing is explicit: immediate or deferred (default deferred). Cascade depth limit: 10 levels. Exceeding the limit logs a warning and truncates.
5. **Reactions produce Effects or DeferredCommands.** DeferredCommands are queued for the next orchestration cycle.
6. **Derives run post-application.** Exception: spatial index Derives may run between sequential Rules when subsequent Rules need spatial queries.
7. **Phase Sequence:** Ordered list of Commands/Derives. Each phase sees post-application state of all previous phases.
8. **Sequential Execution:** Rules in a sequence see interleaved post-application state (e.g., AI turn: enemy 0..N).
9. **Priority Chain:** Try Rules in order; first non-empty result wins (e.g., movement: NPC → combat → move).
10. **Trace records all GameEffects.** Coarse effects expand to fine on demand. Nothing happens off-trace.

### 2.3 What the Grammar Forbids

- A Rule calling another Rule directly (must go through Effects → Reactions)
- State mutation outside the application layer
- Logic in the application layer
- Unrecorded state changes

### 2.4 QueryContext (new in v2)

Rules receive read-only state through a bundled reference struct, solving borrow checker ergonomics:

```rust
pub struct QueryContext<'a> {
    pub player: &'a PlayerState,
    pub world: &'a WorldState,
    pub enemy_positions: &'a HashMap<(i32, i32), usize>,
    pub item_positions: &'a HashMap<(i32, i32), Vec<usize>>,
    pub visible: &'a HashSet<usize>,
    pub turn: u32,
    pub mock_combat_hit: Option<bool>,
    pub mock_combat_damage: Option<i32>,
}
```

Call site pattern:
```rust
let ctx = QueryContext::from(&self);  // borrows shared refs
let (game_effects, presentation) = some_rule(args, &ctx, &mut self.rng);
for effect in &game_effects {
    self.apply_game_effect(effect);
    self.trace.record(effect, source);
}
for effect in &presentation {
    self.apply_presentation_effect(effect);
}
```

### 2.5 Effect Architecture (new in v2)

Domain-scoped enums composed into a top-level enum:

```rust
pub enum GameEffect {
    Combat(CombatEffect),
    Item(ItemEffect),
    Movement(MovementEffect),
    Storm(StormEffect),
    Status(StatusEffect),
    Quest(QuestEffect),
    Resource(ResourceEffect),  // HP, AP, XP, refraction, currencies
    Entity(EntityEffect),      // spawn, kill, move, provoke
    Notify(GameEvent),         // existing GameEvent variants, for Reaction dispatch
}

pub enum PresentationEffect {
    Log(String, MsgType),
    HitFlash(i32, i32),
    DamageNumber(i32, i32, i32, bool),
    Projectile { from: (i32, i32), to: (i32, i32), glyph: char },
    Beam { from: (i32, i32), to: (i32, i32), beam_type: BeamType, duration: u32 },
    VisualDSL(String, u32),  // effect DSL string + duration
}
```

Each domain module defines its own effect variants. `apply_game_effect` delegates to domain-specific appliers. PresentationEffects are applied but not traced.

---

## 3. How It Maps to the Current Codebase

### 3.1 What already fits

| Current pattern | ESCAEV atomic | Modules |
|----------------|---------------|---------|
| `DATA-DEF` | State Facet | map.rs, world_map.rs, entity.rs, all data loaders (22 total) |
| `DECISION-FN` | Rule | combat.rs (`roll_attack`, `calc_hit_chance`), encounter.rs, adaptation.rs, progression.rs, travel.rs |
| `EVENT-ROUTER` | Reaction | LootSystem.on_event, QuestSystem.on_event |
| `TICK-SYSTEM` | Rule (per-turn) | StatusEffectSystem, StormSystem tick, psychic/skills ticks |
| `DATA-XFORM` | Rule (generation) | tile_generator.rs, terrain_forge_adapter.rs, connectivity.rs, settlement/*, spawn.rs |
| `RENDER-COMPOSE` | (unchanged) | All renderer/ modules — read-only, outside ESCAEV scope |
| `INPUT-DISPATCH` | Command source | ui/input.rs, all menu input handlers |

Notable: `CombatSystem::process_enemy_death` is already a proto-Reaction (handles XP, loot events, split-on-death) but mutates state directly. Migration target for Phase 2.

### 3.2 What changes

| Current | Problem | ESCAEV change |
|---------|---------|---------------|
| `STATE-ORCHESTRATOR` (state.rs) | All cross-concern logic opaque | Phase Sequences of Commands. Logic → Rules. state.rs → thin orchestrator + mechanical apply. |
| `use_item` (~140 LOC, 12 concerns) | Decides AND mutates | Split: `use_item_rule` (→ effects) + `apply_game_effect` (mechanical) |
| `end_turn` (19 calls, 4 dead) | Direct subsystem calls | Phase Sequence. Each phase independently testable. |
| `GameEvent` (notification only) | Events don't drive mutations | GameEffect replaces direct mutations. GameEvent becomes `GameEffect::Notify(...)` for Reaction dispatch. |
| FOV/lighting | Tangled into effect flow | Derive — explicit post-application cache recomputation |
| Visual feedback | Interleaved with game logic | PresentationEffect — separated, not traced |
| No trace | Can't inspect what happened | Trace records all GameEffects. DES asserts on traces. |

### 3.3 What doesn't change

- Renderer (read-only), data loading (`DataLoader<T>`, schemas, `once_cell`), DES (gains trace assertions, existing scenarios preserved), deterministic RNG (ChaCha8Rng), data-driven content (JSON), multi-terminal IPC (orthogonal)


---

## 4. Concrete Example: `use_item` Refactored

### Current (state.rs, ~140 LOC, direct mutation)
```rust
pub fn use_item(&mut self, idx: usize) -> bool {
    // ~140 lines of interleaved queries, decisions, and mutations
    // touching AP, HP, refraction, adaptations, map, light, void, crystal, inventory, events
}
```

### Proposed (Rule + Application, v2 with QueryContext and split effects)
```rust
pub fn use_item_rule(
    item_idx: usize,
    ctx: &QueryContext,
    rng: &mut ChaCha8Rng,
) -> (Vec<GameEffect>, Vec<PresentationEffect>) {
    let mut game = Vec::new();
    let mut presentation = Vec::new();

    let id = &ctx.player.inventory[item_idx];
    let def = match get_item_def(id) {
        Some(d) if d.usable => d,
        _ => return (game, presentation),
    };
    if ctx.player.ap < action_cost("use_item") {
        return (game, presentation);
    }

    game.push(GameEffect::Resource(DeductAP(Player, action_cost("use_item"))));

    if def.heal > 0 {
        let heal = def.heal.min(ctx.player.max_hp - ctx.player.hp);
        game.push(GameEffect::Resource(HealEntity(Player, heal)));
    }
    if def.void_exposure > 0 {
        game.push(GameEffect::Resource(AddVoidExposure(Player, def.void_exposure)));
    }
    // ... each concern adds effects conditionally
    if def.consumable {
        game.push(GameEffect::Item(RemoveItem(Player, item_idx)));
    }
    (game, presentation)
}
```

### Application layer (auto-generates log messages)
```rust
pub fn apply_game_effect(&mut self, effect: &GameEffect) {
    match effect {
        GameEffect::Resource(HealEntity(_, amount)) => {
            self.player.hp = (self.player.hp + amount).min(self.player.max_hp);
            // Auto-log: application layer generates presentation
            self.log_typed(format!("You heal for {} HP.", amount), MsgType::Loot);
        }
        GameEffect::Item(RemoveItem(_, idx)) => {
            self.player.inventory.remove(*idx);
        }
        // ... mechanical match arms
    }
}
```

### Rule unit test (no full GameState needed)
```rust
#[test]
fn healing_herb_produces_correct_effects() {
    let ctx = QueryContext::test_with(|b| {
        b.player_hp(60).player_max_hp(100).player_ap(4)
         .inventory(vec!["healing_herb".into()])
    });
    let (effects, _) = use_item_rule(0, &ctx, &mut test_rng(42));
    assert_eq!(effects, vec![
        GameEffect::Resource(DeductAP(Player, 1)),
        GameEffect::Resource(HealEntity(Player, 20)),
        GameEffect::Item(RemoveItem(Player, 0)),
    ]);
}
```

### DES trace test (integration level)
```json
{
    "action": { "type": "use_item", "slot": 0 },
    "expect_effects": [
        { "type": "Resource.DeductAP", "amount": 1 },
        { "type": "Resource.HealEntity", "amount": 20 },
        { "type": "Item.RemoveItem", "index": 0 }
    ],
    "expect_no_effects": [
        { "type": "Resource.DamageEntity" }
    ],
    "at_end": { "player_hp": 80 }
}
```

Note: DES scenarios require BOTH `expect_effects` (causal) AND `at_end` (outcome) assertions. The trace proves the Rule ran; the state assertion proves effects were applied correctly. This closes the gap where an agent could wire effects but implement `apply_game_effect` as a no-op.

---

## 5. Stress Test Results

The vocabulary was tested against 5 systems (combat, movement, storm, AI, end_turn). 15 friction points identified.

### Held firm (no changes needed)
- Reaction chains (combat death → loot → quest)
- Behavior dispatch (AI strategy pattern: Query → Rule selection)
- Tile effects (Glass damage, Glare AP loss as Reactions to MoveEntity)
- Item pickup (Reaction to MoveEntity)
- Adaptation threshold (Reaction to AddRefraction)
- Individual enemy decisions (each enemy's turn is a Rule)

### Bent but workable
- **RNG purity**: Privileged facet — consumed, recorded. Deterministic given seed.
- **Conditional branching in Rules**: Deterministic given (queries + seed).
- **Movement dispatcher**: Branching in query results, not hidden state. Priority Chain.
- **System-generated Commands**: Source tags (Player, System, Reaction).
- **Sub-state replacement** (ForecastNextStorm): Atomic effect on storm facet.
- **Parallel tick systems**: Optimization opportunity, not vocabulary issue.

### Required vocabulary additions (all resolved in v2)
- **Derive** (8th atomic): FOV/lighting cache recomputation
- **DeferredCommand**: World transitions from Reactions
- **Effect granularity**: Storm bulk mutations — coarse/fine zoom
- **Sequential Execution**: AI turn interleaved application
- **Priority Chain**: Movement dispatcher
- **Phase Sequence**: end_turn orchestration
- **Reaction timing**: Immediate vs deferred (default deferred)
- **GameEffect/PresentationEffect split** (v2): Visual feedback separated from state mutations
- **Cascade depth limit** (v2): 10 levels, matching existing `process_events` behavior
- **Spatial index Derive exception** (v2): May run mid-sequence between sequential Rules

---

## 6. Testing Architecture

### 6.1 Test Layers

| Layer | Tests | Writer | Shape |
|-------|-------|--------|-------|
| **Rule unit** | Single rule, inputs → expected GameEffects | AI agent | `assert_eq!(rule(ctx, rng), expected)` |
| **Reaction unit** | Trigger effect → expected GameEffects | AI agent | `assert_eq!(reaction(trigger, ctx, rng), expected)` |
| **Application** | Effect applied → facet changed | One-time per effect type | `apply(state, effect); assert_eq!(facet, expected)` |
| **Trace (DES)** | Command → expected trace subset | AI agent or human | `expect_effects` + `expect_no_effects` |
| **Integration (DES)** | Multi-turn → final state | Human-designed | Existing `at_end` assertions (unchanged) |
| **Migration sentinel** | Fixed-seed → exact numeric outcomes | Written once per phase | Catches RNG ordering changes |

### 6.2 Self-Verification Protocol

When an AI agent implements a new Rule:

1. Write the Rule — pure function, `(&QueryContext, &mut RNG) → (Vec<GameEffect>, Vec<PresentationEffect>)`
2. Write Rule unit tests — enumerate key inputs, assert expected GameEffects (ignore PresentationEffects)
3. Write DES scenario with BOTH `expect_effects` AND `at_end` assertions
4. The trace test is the integration gate — unwired Rules produce no trace entries

DES trace assertions should assert on a **subset** of important effects, not the complete trace. This prevents brittleness when implementation adds new effects (e.g., additional log messages).

### 6.3 DES Evolution

DES gains `expect_effects` and `expect_no_effects` assertion types. Existing `at_end` assertions unchanged. Both required for new scenarios:

- `expect_effects`: "did the right things happen?" (causal)
- `at_end`: "is the world correct?" (outcome)
- Together: proves the Rule ran AND the effects were applied correctly


---

## 7. Migration Path

### Principles
- Incremental. No rewrite. Each step compiles and passes existing tests.
- Dead code resolved BEFORE encoding into new architecture.
- Each Rule must consume RNG in the same order as the imperative code it replaces.
- Phase boundaries are hard stops. Don't start Phase N+1 until N is complete and tested.
- Effect enum and apply_game_effect live in a separate module (`src/game/effect.rs` or `src/game/effects/`) to isolate recompilation from state.rs.

### Phase 0: Foundation (no behavior change)
1. Define `GameEffect` enum (domain-scoped) — start with `Resource` and `Item` variants needed by `use_item`
2. Define `PresentationEffect` enum
3. Define `Trace` struct — ordered vec of GameEffects with source tags, behind opt-in flag
4. Define `QueryContext` struct with `From<&GameState>` impl
5. Add `apply_game_effect()` and `apply_presentation_effect()` — mechanical match arms
6. All new code in new modules. Zero changes to existing code.

### Phase 0.5: Dead code triage
1. Delete confirmed dead code: `terminal_spawn.rs`, 4 dead algorithms (bsp, maze, voronoi, wfc), `patterns/special/` duplicates, 7 deprecated schemas
2. Decide fate of half-wired systems (light, crystal, void): keep or remove. If keep → they get wired via ESCAEV in a later phase. If remove → delete now, before encoding into Effect enum.
3. Delete 4 dead stub methods in `end_turn` (`generate_narrative_fragments`, `generate_biome_content`, `generate_template_content`, `check_dynamic_events`)
4. Delete 16 dead pub methods in state.rs identified by the audit
5. Write migration sentinel DES scenarios: fixed seeds, exact numeric assertions, catch RNG ordering changes

### Phase 1: First Rule extraction (`use_item`)
1. Extract `use_item_rule()` — pure function taking `(&QueryContext, &mut ChaCha8Rng)`, returning `(Vec<GameEffect>, Vec<PresentationEffect>)`
2. `use_item()` becomes: construct QueryContext, call rule, apply effects, record trace
3. Write Rule unit tests for `use_item_rule` (per item type)
4. Write DES trace tests with `expect_effects` + `at_end`
5. Verify all existing DES scenarios pass (behavior unchanged)
6. Verify migration sentinel scenarios pass (RNG ordering preserved)

### Phase 2: Movement (easier than combat — second "easy win")
1. Extract `move_to_tile_rule()` — produces MoveEntity, tile effects
2. Item pickup becomes Reaction to `Entity(MoveEntity(player, ...))`
3. Tile effects (Glass, Glare) become Reactions to MoveEntity
4. NPC interaction and bump-to-attack become Priority Chain branches
5. FOV/lighting become Derives (explicit post-application)
6. Write Rule + Reaction unit tests, DES trace tests

### Phase 3: Combat
1. Extract `attack_melee_rule()`, `ranged_attack_rule()`
2. Extract `process_enemy_death` as Reaction to `Entity(KillEnemy(...))`
3. Swarm aggro, reflect damage, on_hit effects become Reactions
4. Write Rule + Reaction unit tests, DES trace tests

### Phase 3.5: Sub-state extraction
1. Extract sub-states from GameState (CombatState, ProgressionState, etc.) based on patterns revealed by Phases 1-3
2. Update QueryContext to borrow sub-states instead of flat fields
3. This makes Phase 4 cleaner — each end_turn phase operates on a specific sub-state

### Phase 4: `end_turn` decomposition
1. Define Phase Sequence for end_turn (15 live phases after Phase 0.5 cleanup)
2. Each phase becomes a Command with its own Rule
3. Tick systems (status, psychic, skills) become Rules returning Effects
4. Storm tick becomes a Rule with coarse effects
5. AI turn becomes Sequential Execution of per-enemy Rules
6. `process_events` becomes the Reaction dispatch loop with cascade depth limit
7. Write phase-level tests

### Phase 5: Trace-based DES
1. Add `expect_effects` and `expect_no_effects` assertion types to DES interpreter
2. Rewrite 7 fake scenarios as real trace tests
3. Add trace tests for systems with zero coverage (encounter, crafting, skills, trading)
4. Establish policy: new DES scenarios require both `expect_effects` AND `at_end`

### What each phase delivers
| Phase | Delivers | Risk |
|-------|----------|------|
| 0 | Infrastructure, zero behavior change | None |
| 0.5 | Cleaner codebase, migration sentinels | Low (deletions only) |
| 1 | Proof of concept — one system fully ESCAEV | Low |
| 2 | Movement cross-concerns explicit, second validation | Medium |
| 3 | Combat testable at Rule level | Medium-High |
| 3.5 | Clean sub-state boundaries for Phase 4 | Medium |
| 4 | Turn loop decomposed, each phase testable | High |
| 5 | DES becomes trace verification tool | Low (additive) |

---

## 8. Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Effect enum explosion** (est. 80-120 variants full game) | Medium | Domain-scoped enums (`CombatEffect`, `ItemEffect`, etc.) keep each domain manageable. Review enum size at each phase. |
| **Effect ordering matters** | Medium | Effects from one Rule applied in list order. Document ordering semantics per Rule. |
| **RNG ordering sensitivity** | High | Migration constraint: Rules consume RNG in same order as original code. Migration sentinel DES scenarios with fixed seeds catch violations. |
| **Borrow checker friction at call sites** | Medium | QueryContext bundles shared refs. RNG is sole mutable borrow. Disjoint field borrows work in Rust. |
| **Spatial index needed mid-sequence** | Medium | Grammar rule 6 exception: spatial Derives may run between sequential Rules. Document which sequences require this. |
| **Migration breaks existing DES** | Low | Each phase preserves behavior. Sentinels catch RNG drift. |
| **Save/load compatibility** | Medium | ESCAEV changes logic flow, not GameState struct (until Phase 3.5). Phase 3.5 sub-state extraction requires SAVE_VERSION bump + migration function. |
| **Compile time impact** | Low | Effect enum and apply functions in separate module, isolating recompilation from state.rs. |
| **Scope creep** | High | Phase boundaries are hard stops. Each phase independently valuable. Don't start N+1 until N is complete. |
| **Vocabulary gaps for future systems** | Low | Vocabulary designed to extend. 15 friction points already resolved. New atomics can be added. |


---

## 9. Success Criteria

1. **AI agent self-verification.** An agent can implement a new game mechanic, write Rule unit tests and a DES trace+state scenario, and verify it works without human testing.
2. **Scaffold-and-abandon structurally prevented.** New systems require DES scenarios with both `expect_effects` and `at_end` assertions. Unwired systems fail the trace test; no-op apply fails the state test.
3. **Cross-concern interactions explicit.** The trace shows exactly which GameEffects were produced by `use_item`, in what order, from which Rule.
4. **Existing gameplay preserved.** All current DES scenarios pass after each migration phase. Migration sentinels catch RNG ordering drift.
5. **state.rs shrinks.** Logic moves into Rules. state.rs becomes orchestration (phase sequences) + application (mechanical effect writes) + QueryContext construction.

---

## 10. Open Questions

1. **Effect granularity for storm edits.** Coarse (`Storm(ApplyEdit(Glass, 4, seed))`) vs fine (`SetTile` × 100). Proposed: default coarse, expand on demand. Validate during Phase 4.

2. **Reaction timing default.** Keep deferred (current behavior). Allow immediate for specific reactions if needed. Validate during Phase 3 (combat death → loot).

3. **Sub-state extraction scope.** Exact boundaries TBD after Phases 1-3 reveal which field groupings Rules actually need. QueryContext usage patterns will guide this.

4. **Trace storage.** Ephemeral (testing only, not saved). Revisit if replay becomes a feature.

5. **IPC effect streams.** Satellites could receive effect streams instead of state snapshots. Deferred to post-Phase 4.

6. **Half-wired system fate.** Light, crystal, void — keep or remove? Must be decided in Phase 0.5 before encoding into Effect enum. Recommendation: remove ability methods (dead), keep resource accumulation (ticks work, items grant energy). This means the Effect enum doesn't need ability variants, only resource variants.

---

## Appendix A: Vocabulary Reference Card

```
ATOMICS (8)
  State Facet      typed slice of game state
  Query            pure read of facets; RNG privileged; bundled via QueryContext
  Command          player intention or system trigger; source-tagged
  Effect           GameEffect (state mutation, traced) + PresentationEffect (visual, not traced)
                   domain-scoped enums; supports coarse/fine granularity
  Rule             (Command, &QueryContext, &mut RNG) → (Vec<GameEffect>, Vec<PresentationEffect>)
                   deterministic given seed; free functions; owned data in effects
  Reaction         (GameEffect, &QueryContext, &mut RNG) → effects or DeferredCommand
                   timing: immediate/deferred (default deferred); cascade limit: 10
  Derive           (State Facets) → recomputed cache; post-application
                   exception: spatial index Derives may run mid-sequence
  Trace            ordered GameEffects + sources; coarse/fine zoom; ephemeral

GRAMMAR (10)
  1. Commands → Effects only through Rules
  2. Rules deterministic given (QueryContext + RNG seed); preserve RNG consumption order
  3. Effects applied in order; application mechanical; logs auto-generated
  4. Reactions trigger on GameEffects; timing explicit; cascade limit 10
  5. Reactions → Effects or DeferredCommands
  6. Derives post-application; spatial index exception for mid-sequence
  7. Phase Sequence: ordered Commands/Derives, each sees prior results
  8. Sequential Execution: interleaved application between Rules
  9. Priority Chain: try Rules in order, first non-empty wins
  10. Trace records all GameEffects; coarse expands to fine on demand
```

## Appendix B: Friction Points from Stress Testing

| # | System | Friction | Resolution | Category |
|---|--------|---------|------------|----------|
| 1 | Combat | RNG breaks Query purity | RNG is privileged facet | Bent |
| 2 | Combat | Conditional branching in Rules | Deterministic given seed | Bent |
| 3 | Combat | Death cascade complexity | Reaction chain | Held |
| 4 | Movement | Dispatcher is a meta-rule | Priority Chain grammar rule | Bent |
| 5 | Movement | FOV/lighting are derived state | Derive atomic | Added |
| 6 | Movement | World transitions are compound | DeferredCommand | Added |
| 7 | Storm | Bulk effects (100+ tiles) | Coarse/fine granularity | Added |
| 8 | Storm | System-generated commands | Command source tags | Bent |
| 9 | Storm | Sub-state replacement | Atomic effect on storm facet | Bent |
| 10 | AI | Sequential entity processing | Sequential Execution rule | Added |
| 11 | AI | Behavior dispatch | Query → Rule selection | Held |
| 12 | AI | StandardMelee is a god-rule | Priority Chain decomposition | Added |
| 13 | end_turn | Not a Rule — Phase Sequence | Phase Sequence rule | Added |
| 14 | end_turn | Parallel tick systems | Optimization, not vocabulary | Bent |
| 15 | end_turn | Deferred vs immediate reactions | Explicit timing in grammar | Added |

## Appendix C: Review Feedback Disposition

| Review Item | Disposition | Where in v2 |
|-------------|------------|-------------|
| end_turn fan-out 19 not 11 | Accepted | §1, §7 |
| use_item ~140 LOC not 160 | Accepted | §1 |
| QueryContext struct | Accepted | §2.4, §4 |
| GameEffect/PresentationEffect split | Accepted | §2.1, §2.5, §4 |
| Domain-scoped Effect enums | Accepted | §2.5 |
| Dead code → Phase 0.5 | Accepted | §7 |
| Cascade depth limit 10 | Accepted | §2.2 rule 4 |
| RNG ordering constraint | Accepted | §2.2 rule 2, §8 |
| Sub-state extraction Phase 3.5 | Accepted | §7 |
| Save/load compatibility | Accepted | §8 |
| Compile-time isolation | Accepted | §7 principles, §8 |
| Logging auto-generation | Accepted | §2.2 rule 3, §4 |
| Both expect_effects AND at_end | Accepted | §6.2, §9 |
| Migration sentinel scenarios | Accepted | §6.1, §7 Phase 0.5 |
| Subset-based trace assertions | Accepted | §6.2 |
| process_enemy_death as migration target | Accepted | §3.1 |
| Movement before combat | Accepted | §7 (Phase 2 = movement, Phase 3 = combat) |
| LegacyMutation escape hatch | Rejected | Introduces opaque mutation the architecture eliminates. Prefer smaller Phase 1 scope. |
| Spatial index mid-sequence Derives | Accepted | §2.2 rule 6 |
