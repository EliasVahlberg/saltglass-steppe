# Priority 1: DES Effect Assertions

> Effort: Half day
> Impact: Completes the VERA verification story
> Files: `src/des/mod.rs`

## Problem

The FINAL_ARCHITECTURE.md promised three DES assertion types that were never built. The trace is enabled during DES runs and dumps on failure, but no scenario actually asserts on effects. All 22 scenarios use state assertions only.

Without effect assertions, DES can't distinguish between "the rule produced the wrong effects" and "the apply arm has a bug."

## What to build

Add 3 new variants to `AssertionCheck` enum in `src/des/mod.rs` (line ~112):

```rust
EffectOccurred { effect_pattern: String },
EffectNotOccurred { effect_pattern: String },
EffectCount { effect_pattern: String, op: CmpOp, value: usize },
```

The `effect_pattern` is a substring match against the `Debug` representation of effects. This avoids needing to deserialize full Effect enums in JSON — patterns like `"PlayerEffect::Heal"` or `"CombatEffect::Kill"` are sufficient.

Add to `evaluate_check` (line ~1278):

```rust
AssertionCheck::EffectOccurred { effect_pattern } => {
    self.state.trace.entries.iter()
        .any(|e| format!("{:?}", e.effect).contains(effect_pattern))
}
AssertionCheck::EffectNotOccurred { effect_pattern } => {
    !self.state.trace.entries.iter()
        .any(|e| format!("{:?}", e.effect).contains(effect_pattern))
}
AssertionCheck::EffectCount { effect_pattern, op, value } => {
    let count = self.state.trace.entries.iter()
        .filter(|e| format!("{:?}", e.effect).contains(effect_pattern))
        .count();
    op.compare(count as i32, *value as i32)
}
```

## DES JSON usage

```json
{
    "assertions": [
        {"at_end": true, "check": {"type": "effect_occurred", "effect_pattern": "PlayerEffect::Heal"}},
        {"at_end": true, "check": {"type": "effect_not_occurred", "effect_pattern": "CombatEffect::Kill"}},
        {"at_end": true, "check": {"type": "effect_count", "effect_pattern": "PlayerEffect::SpendAp", "op": "eq", "value": 3}}
    ]
}
```

## Tests

1. Add 3 unit tests in the `mod tests` block at the bottom of `src/des/mod.rs`:
   - `effect_occurred_assertion_passes` — use_item produces Heal, assert effect_occurred
   - `effect_not_occurred_assertion_passes` — wait produces no Kill, assert effect_not_occurred
   - `effect_count_assertion_passes` — multiple actions, assert effect_count

2. Update 2-3 existing DES scenarios to add effect assertions alongside their state assertions. Good candidates:
   - A combat scenario: assert `effect_occurred` for `CombatEffect::DealDamage`
   - An item use scenario: assert `effect_count` for `PlayerEffect::SpendAp`

## Verify

`cargo test`, `cargo clippy -- -D warnings`, all 22+ DES scenarios pass.
