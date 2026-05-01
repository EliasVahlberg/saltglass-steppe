# System Design Philosophy

> Added: 2026-05-01
> Context: Emerged from the StatEffect refactor and adaptation system design sessions.

This document captures design principles that apply across all game systems — not just adaptations. Read before designing a new system or refactoring an existing one.

---

## The core mistake to avoid: designing systems around player-facing concepts

Game systems have two faces: how they appear to the player, and how they work in code. These should be designed independently.

**Wrong**: "Adaptations are a system, so I'll build an isolated adaptation module that handles everything adaptations do."

**Right**: "Adaptations are a player-facing concept. In code, they are a bundle of effects that happen to be granted together. Each effect belongs to whatever system owns that kind of effect."

The practical consequence: `prismhide` granting +2 armor and a `jacket` granting +2 armor are the same thing in code. They should share the same code path. The fact that one is called an "adaptation" and the other is called "equipment" is a UI concern, not an implementation concern.

This mistake is easy to make in OOP because classes encourage you to model the world as the player sees it. Resist it.

---

## Stat effects: the unified mechanism

All sources of stat modification (adaptations, equipment, status effects, skills, terrain) produce `StatEffect` entries. `resolve_stat()` collapses them to a single value at query time.

```rust
// src/game/stat_effect.rs
struct StatEffect {
    stat: &'static str,   // "armor", "damage_bonus", "fov", ...
    op: StatOp,           // Add(f32) | Multiply(f32)
    priority: f32,        // lower = applied first
    source_id: String,    // "prismhide", "jacket_01", "blinded"
}
```

**Resolution**: sort by priority ascending, fold each operation sequentially. `(base + Add) * Multiply` in priority order.

**The single collection point**: `collect_player_stat_effects(&PlayerState) -> Vec<StatEffect>` in `stat_effect.rs`. All sources are collected here. Query functions (`effective_armor()`, `effective_reflex()`, etc.) call this and resolve.

### What belongs in StatEffect

- Always-on numeric bonuses: armor, damage, reflex, FOV range
- Conditional numeric bonuses: accuracy penalty from status effects, ingredient reduction from adaptations
- Boolean capabilities expressed as 0/1: `blocks_healing`, `grants_invisibility` (value 1.0 = active)

### What does NOT belong in StatEffect

Event reactions — effects that fire when something happens and produce mutations. These are rules, not stats. See below.

### Adding a new stat-modifying source

1. Add the stat name to the known stats list in `stat_effect.rs` (comment only — no code change needed)
2. Add the value to the source's data (JSON `stat_modifiers` field, or `effects` array with the stat name)
3. `collect_player_stat_effects` picks it up automatically

No code changes required for pure stat effects. This is the goal.

---

## Two categories of effects, not one

### Category 1: Stat effects (query-time)

Pure functions of game state. No side effects. Computed when needed, not stored.

- Resolved by `resolve_stat()` at the call site
- Can reference other base stats (but not other derived stats — no circular resolution)
- Testable in isolation with no game state

**Layering rule to prevent circular resolution**: resolvers can only read base stats (hp, ap, level, etc.), not other derived stats. `effective_armor` can read `player.armor` (base) but not `effective_reflex` (derived).

### Category 2: Event reactions (mutation-time)

Fire when something happens, produce mutations. These are rules.

- Belong in the mutation pipeline, not in stat resolvers
- The adaptation data specifies *parameters*, the code specifies the *rule shape*
- Adding a new adaptation with an existing rule shape = data change only
- Adding a new rule shape = one new hook in the relevant system

**Current event reactions in adaptations** (hardcoded by ID — to be replaced with parameterized registry, see issue #6):
- `bone_spur`: on melee hit → 20% bleed
- `killing_edge`: on melee kill → grant AP refund
- `scar_lattice`: on damage taken → stack temp armor
- `storm_drinker`: on storm fire → grant AP

---

## The write-on-change anti-pattern

**Symptom**: a stat is computed and stored when something changes, then read from storage later.

**Examples of this pattern** (eliminated or being eliminated):
- `player.armor` written by `recalc_equipment_stats()` on equip/unequip ✅ eliminated
- `SkillsState.passive_bonuses` HashMap written by `recalculate_passive_bonuses()` on skill upgrade ⚠️ issue #5

**Why it's bad**:
- The stored value can become stale if any contributing source changes without triggering the recalc
- Multiple recalc triggers are easy to miss (equip, unequip, level up, status effect applied...)
- The stored value and the computed value can diverge silently

**The fix**: compute on query, not on change. Stats are cheap to recompute. Staleness bugs are expensive to debug.

---

## The "personal teleporter / recall" principle

If two player-facing features do the same thing mechanically, they should share the same code path.

A `personal_teleporter` item and a `recall` adaptation both teleport the player. They should both call `dispatch(Command::Teleport {...})`. The adaptation system's job is to deliver the effect to the right system — not to re-implement teleportation.

Before writing new code for an adaptation effect, ask: does this mechanic already exist somewhere? If yes, call into that path.

---

## What to check before designing a new system

1. **Is this a stat effect?** If it modifies a numeric value that other systems query, it belongs in `StatEffect`. Add it to `collect_player_stat_effects`.

2. **Is this an event reaction?** If it fires when something happens and produces mutations, it's a rule. Find the event site in the mutation pipeline and add a hook there. Check if a similar rule already exists.

3. **Does this already exist?** Check `stat_effect.rs`, `systems/`, `rules/`, and `notify.rs` before writing new code. The mechanic you need may already be implemented for a different source.

4. **Am I modeling the player's view or the code's view?** If you find yourself naming a module after a player-facing concept (AdaptationSystem, ItemSystem, SkillSystem), stop and ask whether the underlying mechanics are actually distinct or just presented differently to the player.

---

## Stat naming convention

Stats are `snake_case` strings. Known stats as of 2026-05-01:

| Stat | Type | Sources |
|------|------|---------|
| `armor` | Add | adaptations, equipment, scar_lattice_armor |
| `damage_bonus` | Add | adaptations, status effects (negative) |
| `reflex` | Add | adaptations, base player stat |
| `fov` | Add | adaptations (lens_eye) |
| `ranged_accuracy_bonus` | Add | adaptations (lens_eye) |
| `craft_ingredient_reduction` | Add | adaptations (salt_sense) |
| `accuracy_penalty` | Add | status effects (blinded) |
| `blocks_healing` | Add (0/1) | status effects (bleed) |

When adding a new stat, add it to this table.
