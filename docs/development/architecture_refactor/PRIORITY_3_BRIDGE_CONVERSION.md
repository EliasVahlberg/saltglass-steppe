# Priority 3: Selective Bridge Conversion

> Effort: 1-2 days
> Impact: Testable loot, status, and quest logic
> Files: `rules/`, `effects/apply.rs`, `systems/`

## Problem

Five bridge effects call legacy code from apply arms, bypassing the "apply is mechanical" principle. The trace says "status effects ticked" but not what happened. These systems can't be unit tested.

## Which bridges to convert

| Bridge | Apply arm calls | LOC | Modify often? | Convert? |
|--------|----------------|-----|---------------|----------|
| `PlayerEffect::TickStatusEffects` | `StatusEffectSystem::tick_player_effects/tick_enemy_effects` | ~80 | Yes — new status effects | **Yes** |
| `EventEffect::LootDrop` | `LootSystem::handle_enemy_death` | ~60 | Yes — new enemies need loot | **Yes** |
| `EventEffect::QuestNotify` | `quest_log.on_enemy_killed/on_item_collected/etc` | ~40 | Yes — new quests | **Yes** |
| `PlayerEffect::RunAI` | `self.update_enemies()` | ~400 | Rarely — behaviors are data-driven | **No** |
| `MapEffect::TickStorm` | `storm.tick() + StormSystem::apply_storm()` | ~300 | Rarely — storm edits are stable | **No** |

## StatusEffectSystem conversion

**Current:** `StatusEffectSystem::tick_player_effects(state)` iterates player status effects, decrements durations, applies damage (poison/burn), removes expired.

**New rule:** `src/game/rules/status.rs`
```rust
fn rule_tick_status(ctx: &QueryContext) -> RuleOutput<Effect, Presentation> {
    // For each active status effect:
    //   - If expired: StatusEffect::Remove { id }
    //   - If damage type: PlayerEffect::TakeDamage { amount }
    //   - Decrement: StatusEffect::Tick { id }
}
```

**New effects:**
- `StatusEffect::Tick { effect_id: String }` — decrement duration
- `StatusEffect::Remove { effect_id: String }` — remove expired
- Add `Status(StatusEffect)` domain to the top-level `Effect` enum

**Tests:** tick decrements duration, expired effect produces Remove, poison produces TakeDamage.

## LootSystem conversion

**Current:** `LootSystem::handle_enemy_death(state, enemy_id, x, y)` looks up loot table, rolls RNG, spawns items on map.

**New reaction:** `src/game/rules/reactions.rs`
```rust
fn reaction_loot_drop(enemy_id: &str, x: i32, y: i32, ctx: &QueryContext, rng: &mut ChaCha8Rng) -> RuleOutput<Effect, Presentation> {
    // Look up loot table for enemy_id
    // Roll RNG for each entry
    // Return ItemEffect::SpawnOnMap { item_id, x, y } for each drop
}
```

**New effects:**
- `ItemEffect::SpawnOnMap { item_id: String, x: i32, y: i32 }`

**RNG ordering:** This reaction currently fires from `collect_reactions` on `CombatEffect::Kill`. The RNG consumption order changes from end-of-turn (old event system) to kill-time (VERA reaction). This was accepted in Batch F — verify existing DES scenarios still pass.

**Tests:** enemy with loot table produces SpawnOnMap effects, enemy without loot table produces nothing.

## QuestSystem conversion

**Current:** `EventEffect::QuestNotify` calls `quest_log.on_enemy_killed()`, `on_item_collected()`, etc.

**New reaction functions:**
```rust
fn reaction_quest_kill(enemy_id: &str, ctx: &QueryContext) -> RuleOutput
fn reaction_quest_collect(item_id: &str, ctx: &QueryContext) -> RuleOutput
fn reaction_quest_position(x: i32, y: i32, ctx: &QueryContext) -> RuleOutput
```

**New effects:**
- `QuestEffect::Progress { quest_id: String, objective_type: String }`
- `QuestEffect::AutoComplete { quest_id: String }`

**Tests:** killing quest target produces Progress effect, collecting quest item produces Progress effect.

## What to delete after conversion

- `EventEffect::LootDrop` variant and its apply arm
- `EventEffect::QuestNotify` variant, `QuestNotifyKind` enum, and apply arm
- `PlayerEffect::TickStatusEffects` variant and its apply arm
- `LootSystem::handle_enemy_death` (or keep as internal helper called by the reaction)
- `StatusEffectSystem::tick_player_effects` / `tick_enemy_effects`

## Verify

`cargo test`, `cargo clippy -- -D warnings`, all DES scenarios pass. New rule unit tests for all three conversions.
