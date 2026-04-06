# Trial Refactor Plan: VERA Phase 0 + Phase 1

> Date: 2026-04-04
> Ref: FINAL_ARCHITECTURE.md (VERA — Verified Effect-Rule Architecture)
> Goal: Prove the VERA pattern works on the worst cross-concern method (use_item)

---

## Scope

- **Phase 0** (foundation): On main. New modules, zero behavior change. ~200 LOC.
- **Phase 1** (use_item): On branch `refactor/use-item-effects`. Convert `use_item` (140 LOC, 12 concerns) and `use_item_on_tile` (60 LOC) to rule functions. Delete old methods. All 12 existing DES scenarios must pass.

## Success criteria

Phase 0:
- `cargo build` and `cargo test` pass with no behavior change
- New `src/game/effects/` module exists with all types

Phase 1:
- All 12 existing use_item DES scenarios pass through the new dispatch path
- At least 5 rule unit tests exist (using TestContext, no GameState)
- Old `use_item` and `use_item_on_tile` methods deleted from state.rs
- Trace dumps on DES failure show useful debugging info

## Retrospective questions (answer after Phase 1)

1. Did QueryContext work ergonomically, or did you fight the borrow checker?
2. How many Effect variants were needed? (prediction: ~15)
3. Did the rule function feel like natural Rust, or like fighting a framework?
4. Could an AI agent write meaningful unit tests from the rule signature alone?
5. Did the trace dump actually help debug a failing scenario?
6. How much did state.rs shrink? Was the complexity moved or eliminated?

If any answer is clearly negative, stop and reassess before Phase 2.

---

## Phase 0: Foundation (on main)

### Step 0.1: Create effects module structure

```
src/game/effects/
├── mod.rs          # Re-exports, Effect enum, Presentation enum, Target enum, RuleOutput
├── trace.rs        # Trace, TraceEntry, TraceSource
├── context.rs      # QueryContext, TestContext builder
└── apply.rs        # GameState::apply_effect (initially empty match arms)
```

### Step 0.2: Define core types in `effects/mod.rs`

From FINAL_ARCHITECTURE.md §2.1-2.2:
- `Effect` enum with domain variants: `Combat(CombatEffect)`, `Item(ItemEffect)`, `Player(PlayerEffect)`, `Map(MapEffect)`, `Resource(ResourceEffect)`, `Event(EventEffect)`
- `Presentation` enum (visual-only, not traced)
- `Target` enum: `Player`, `Enemy { index }`, `EnemyAt { x, y }`, `Npc { index }`
- `RuleOutput` struct: `{ effects: Vec<Effect>, presentation: Vec<Presentation> }`
- Only define variants needed for use_item initially. Other domains (Quest, Storm, Status, Combat) get empty enums with a placeholder variant or are omitted until Phase 2-3.

Effect variants needed for use_item:
```
PlayerEffect::Heal { amount }
PlayerEffect::SpendAp { amount }
PlayerEffect::ModifyRefraction { delta }
PlayerEffect::SuppressAdaptations { turns }
ItemEffect::Consume { item_id, inventory_index }
MapEffect::RevealAll
MapEffect::SetTile { x, y, tile_type }
MapEffect::DamageWall { x, y, damage }
ResourceEffect::GainLightEnergy { amount }
ResourceEffect::GainVoidEnergy { amount }
ResourceEffect::GainVoidExposure { amount }
ResourceEffect::GainResonanceEnergy { amount }
ResourceEffect::PlaceCrystal { x, y, frequency }
EventEffect::Log { message, msg_type }
EventEffect::OpenBook { book_id }
EventEffect::EmitGameEvent { event_name, data }
```

That's 16 variants across 5 domains. Close to the prediction of ~15.

### Step 0.3: Define QueryContext in `effects/context.rs`

From FINAL_ARCHITECTURE.md §2.3:
- `QueryContext<'a>` with borrowed refs to PlayerState, WorldState, spatial indices
- `QueryContext::from_state(&GameState) -> QueryContext`
- Convenience methods: `item_def()`, `enemy_at()`, `enemy()`
- `TestContext` builder for unit tests: `TestContext::new().with_player_hp(100).with_inventory(vec![...]).build()`

### Step 0.4: Define Trace in `effects/trace.rs`

From FINAL_ARCHITECTURE.md §2.5:
- `Trace { entries: Vec<TraceEntry>, enabled: bool }`
- `TraceEntry { turn, source, effect }`
- `TraceSource::Rule { name }` and `TraceSource::Reaction { name, trigger }`
- Query methods: `contains()`, `from_rule()`, `effects_matching()`

### Step 0.5: Stub apply in `effects/apply.rs`

- `impl GameState { pub fn apply_effect(&mut self, effect: &Effect) { match ... } }`
- Each domain arm delegates to a domain-specific apply function
- Arms for use_item-relevant effects are implemented
- Other arms are `todo!()` or empty — they'll be filled in Phase 2-3

### Step 0.6: Wire into GameState

- Add `pub mod effects;` to `src/game/mod.rs`
- Add `trace: Trace` field to `GameState` (with `#[serde(skip)]`, default disabled)
- Add `pub fn dispatch(&mut self, command: Command)` stub to state.rs (initially empty, filled in Phase 1)

### Step 0.7: Verify

- `cargo build` — compiles
- `cargo test` — all existing tests pass
- `cargo clippy` — no warnings
- No behavior change whatsoever

---

## Phase 1: use_item extraction (on branch)

### Step 1.1: Create branch

```bash
git checkout -b refactor/use-item-effects
```

### Step 1.2: Write rule_use_item

Create `src/game/rules/mod.rs` and `src/game/rules/item.rs`.

`rule_use_item(item_index: usize, ctx: &QueryContext, rng: &mut ChaCha8Rng) -> RuleOutput`

Translate the 140 LOC of `GameState::use_item` into a pure function that returns effects. The logic is identical — same if-chains, same calculations — but instead of `self.player.hp += heal` it pushes `Effect::Player(PlayerEffect::Heal { amount: heal })`.

Key translation points:
- `self.player.ap -= cost` → `effects.push(Effect::Player(PlayerEffect::SpendAp { amount: cost }))`
- `self.player.hp += heal` → `effects.push(Effect::Player(PlayerEffect::Heal { amount: heal }))`
- `self.log_typed(...)` → `presentation.push(Presentation::LogMessage { ... })`
- `self.emit(GameEvent::...)` → `effects.push(Effect::Event(EventEffect::EmitGameEvent { ... }))`
- `self.revealed.insert(idx)` → `effects.push(Effect::Map(MapEffect::RevealAll))`
- `self.player.inventory.remove(idx)` → `effects.push(Effect::Item(ItemEffect::Consume { ... }))`

Similarly write `rule_use_item_on_tile` for the wall-breaking path.

### Step 1.3: Implement apply arms

Fill in `apply_effect` match arms for all 16 effect variants. Each arm is a direct field assignment — no logic, no conditionals.

Example:
```rust
PlayerEffect::Heal { amount } => {
    self.player.hp = (self.player.hp + amount).min(self.player.max_hp);
}
```

### Step 1.4: Wire dispatch

In `GameState::dispatch`:
```rust
Command::UseItem { index } => {
    let ctx = QueryContext::from_state(self);
    let output = rule_use_item(index, &ctx, &mut self.rng);
    self.apply_and_trace(output, "rule_use_item");
}
Command::UseItemOnTile { index, x, y } => {
    let ctx = QueryContext::from_state(self);
    let output = rule_use_item_on_tile(index, x, y, &ctx, &mut self.rng);
    self.apply_and_trace(output, "rule_use_item_on_tile");
}
```

Helper:
```rust
fn apply_and_trace(&mut self, output: RuleOutput, rule_name: &'static str) {
    for effect in &output.effects {
        self.apply_effect(effect);
        self.trace.record(effect, TraceSource::Rule { name: rule_name }, self.turn);
    }
    for p in &output.presentation {
        self.apply_presentation(p);
    }
}
```

### Step 1.5: Update DES interpreter

In `execute_player_action`, change:
```rust
Action::UseItem { item_index } => {
    // OLD: self.state.use_item(*item_index);
    self.state.dispatch(Command::UseItem { index: *item_index });
    self.log(format!("Player used item at index {}", item_index));
}
```

Enable trace at DES startup:
```rust
pub fn new(scenario: Scenario) -> Self {
    let mut executor = /* ... existing setup ... */;
    executor.state.trace.enabled = true;
    executor
}
```

### Step 1.6: Write rule unit tests

In `src/game/rules/item.rs` (or a `tests` submodule):

Minimum 5 tests:
1. `use_healing_item_produces_heal_and_consume` — healing salve → Heal + SpendAp + Consume
2. `use_item_with_no_ap_produces_nothing` — 0 AP → empty effects
3. `use_item_invalid_index_produces_nothing` — out of bounds → empty
4. `use_non_usable_item_produces_log_only` — non-usable → only presentation log
5. `use_map_reveal_item_produces_reveal_effect` — cartographer's lens → RevealAll
6. `use_book_produces_open_book_effect` — book item → OpenBook
7. `use_void_item_produces_resource_effects` — void shard → GainVoidEnergy + GainVoidExposure

### Step 1.7: Run all existing DES scenarios

```bash
cargo test --test des_scenarios
```

All 12 use_item scenarios must pass. All other scenarios must pass unchanged (they don't touch the new dispatch path).

### Step 1.8: Delete old methods

Remove `GameState::use_item` and `GameState::use_item_on_tile` from state.rs. This is ~200 LOC removed.

### Step 1.9: Add trace dump on DES failure

When a DES assertion fails, print the trace:
```rust
if !assertion_passed {
    if self.state.trace.enabled {
        eprintln!("TRACE:");
        for entry in &self.state.trace.entries {
            eprintln!("  [{}] {:?}", entry.source, entry.effect);
        }
    }
}
```

### Step 1.10: Verify and retrospective

- `cargo test` — all pass
- `cargo clippy` — clean
- Answer the 6 retrospective questions
- If positive: merge branch, plan Phase 2
- If negative: document what went wrong, delete branch, reassess

---

## RNG Ordering Verification

During Step 1.7, add a temporary check: run the old `use_item` path and the new `dispatch` path with the same seed, compare RNG state after. If they diverge, the rule consumes RNG in a different order.

Note: `use_item` currently does NOT consume RNG (no random rolls in item use). This makes it an ideal first target — no RNG ordering risk. Combat (Phase 3) is where RNG ordering becomes critical.

---

## Files touched

### Phase 0 (new files only, no existing files modified except mod.rs and state.rs)
- `src/game/effects/mod.rs` — NEW
- `src/game/effects/trace.rs` — NEW
- `src/game/effects/context.rs` — NEW
- `src/game/effects/apply.rs` — NEW
- `src/game/mod.rs` — add `pub mod effects;`
- `src/game/state.rs` — add `trace: Trace` field, `dispatch()` stub

### Phase 1 (branch)
- `src/game/rules/mod.rs` — NEW
- `src/game/rules/item.rs` — NEW (rule functions + unit tests)
- `src/game/effects/apply.rs` — fill in match arms
- `src/game/state.rs` — wire dispatch, delete use_item/use_item_on_tile (~200 LOC removed)
- `src/des/mod.rs` — update execute_player_action, enable trace, add trace dump

### Not touched
- All DES scenario JSON files (they call `use_item` action which the DES interpreter translates)
- Renderer, UI, generation, data loaders, all other systems
- Save format (trace is `#[serde(skip)]`)
