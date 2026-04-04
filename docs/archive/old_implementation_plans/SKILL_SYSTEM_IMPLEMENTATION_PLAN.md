# Skill System Implementation Plan

## Overview

This document covers the implementation of the skill tree system and establishes the scalable pattern for adding the remaining ~55 skills in the catalog.

See **[SKILL_TREE_DESIGN.md](SKILL_TREE_DESIGN.md)** for the full tree structure, ~90 skill definitions, blocker analysis, and balancing notes across all 7 categories.

The existing passive effect system (`passive_bonuses: HashMap<String, f32>` in `SkillsState`)
is sound but has two problems at scale: string-based lookups scattered across systems, and no
consistent naming convention. This plan addresses both without a full rewrite.

---

## Summary Table

| Phase | Goal | Files | Risk | Status |
|-------|------|-------|------|--------|
| 0 | Add skill definitions to JSON | `data/skill_trees.json` | Low | ✓ Done — 35 skills across 7 categories |
| 1 | Standardize naming + add typed accessors to `SkillsState` | `src/game/skills.rs` | Low | ✓ Done |
| 2 | Wire accessors into gameplay systems | combat | Medium | ✓ Done (combat.rs — 4 lookups migrated) |
| 3 | DES test coverage | `tests/scenarios/skill_tree_upgrade_test.json` | Low | ✓ Done — 15/16 DES tests pass |

---

## Architecture Decision: Typed Accessors (not raw HashMap)

The system-architect reviewed the existing code and identified the core scaling problem:
every system that consumes a passive effect does a raw string lookup:

```rust
// Current pattern — scattered, typo-prone, no discoverability
let bonus = state.player.skills.passive_bonuses.get("melee_damage_bonus").copied().unwrap_or(0.0);
```

**Recommended pattern**: keep the HashMap as the backing store (no migration needed), but
expose typed accessor methods on `SkillsState` that encapsulate the lookups. Systems call
methods, not string keys.

```rust
// Target pattern — one place to change if naming shifts
impl SkillsState {
    pub fn melee_damage_multiplier(&self) -> f32 { ... }
    pub fn storm_forecast_depth(&self) -> u32 { ... }
    pub fn adaptation_damage_bonus(&self, adaptation_count: usize) -> f32 { ... }
}
```

**Why not `ComputedStats` struct yet?** A pre-computed struct (recomputed on skill change) is
the right long-term target, but requires threading `adaptation_count` and other dynamic values
into the recomputation. With 9 skills it's premature. Migrate to `ComputedStats` when the
accessor list grows unwieldy (~20+ skills) or when profiling shows HashMap lookup cost.

**Naming convention** (standardize now, before the catalog grows):

```
{system}_{stat}_{modifier_type}
```

Examples: `melee_damage_multiplier`, `storm_forecast_depth`, `refraction_gain_multiplier`,
`hunger_rate_multiplier`. All multipliers are additive deltas (0.0 = no change, -0.25 = 25%
reduction), applied as `base * (1.0 + delta)`.

---

## Skills Being Implemented

### Foundation Skills (prerequisites, no passive effects)

| ID | Name | Category | Prereq | Purpose |
|----|------|----------|--------|---------|
| `salt_alchemy` | Salt Alchemy | crafting | `crafting` lv2 | Gates Adaptation Tinctures |
| `faction_lore` | Faction Lore | social | `bartering` lv2 | Gates Reputation Weaving |

### Target Skills

| ID | Name | Category | Prereq | Key Effect |
|----|------|----------|--------|------------|
| `storm_stance` | Storm Stance | combat | `melee_combat` lv2 | Damage/accuracy shifts with storm state |
| `crucible_technique` | Crucible Technique | combat | `melee_combat` lv3 | Damage += adaptation_count × 5%/level |
| `storm_reading` | Storm Reading | survival | `survival` lv2 | Extends storm forecast depth and detail |
| `adaptation_resistance` | Adaptation Resistance | survival | `survival` lv2 | Reduces refraction gain rate |
| `salt_hermit_ways` | Salt Hermit Ways | survival | `survival` lv3 | Hunger/thirst rate reduction |
| `adaptation_tinctures` | Adaptation Tinctures | crafting | `salt_alchemy` lv3 | Unlocks adaptation recipe tier |
| `reputation_weaving` | Reputation Weaving | social | `faction_lore` lv4 | Reduces rival faction rep bleed |

---

## Phase 0 — Data Definitions

**File**: `data/abilities.json` — append to the `skills` array.

Effect type naming follows the convention above. All new effect types are listed in Phase 1.

### Foundation Skills

```json
{
  "id": "salt_alchemy",
  "name": "Salt Alchemy",
  "category": "crafting",
  "description": "Knowledge of crystalline chemistry. Prerequisite for advanced tincture work.",
  "max_level": 5,
  "base_cost": 8,
  "prerequisites": [{ "skill_id": "crafting", "required_level": 2 }],
  "passive_effects": []
},
{
  "id": "faction_lore",
  "name": "Faction Lore",
  "category": "social",
  "description": "Understanding of political dynamics and faction rivalries across the Steppe.",
  "max_level": 6,
  "base_cost": 6,
  "prerequisites": [{ "skill_id": "bartering", "required_level": 2 }],
  "passive_effects": []
}
```

### Combat Skills

```json
{
  "id": "storm_stance",
  "name": "Storm Stance",
  "category": "combat",
  "description": "Your fighting style shifts with the storm. Near active storms: +20% damage, -15% accuracy. In calm: +15% accuracy, +10% dodge.",
  "max_level": 3,
  "base_cost": 10,
  "prerequisites": [{ "skill_id": "melee_combat", "required_level": 2 }],
  "passive_effects": [
    { "effect_type": "storm_stance_damage_delta", "target": "melee", "value_per_level": 0.067, "max_value": 0.2 },
    { "effect_type": "storm_stance_accuracy_delta", "target": "melee", "value_per_level": 0.05, "max_value": 0.15 }
  ]
},
{
  "id": "crucible_technique",
  "name": "Crucible Technique",
  "category": "combat",
  "description": "Named for Vex Crucible. Each refraction adaptation adds +5% melee damage per skill level.",
  "max_level": 3,
  "base_cost": 12,
  "prerequisites": [{ "skill_id": "melee_combat", "required_level": 3 }],
  "passive_effects": [
    { "effect_type": "adaptation_damage_per_stack", "target": "melee", "value_per_level": 0.05, "max_value": 0.15 }
  ]
}
```

### Survival Skills

```json
{
  "id": "storm_reading",
  "name": "Storm Reading",
  "category": "survival",
  "description": "Read the salt and glass for storm signs. Lv1: +5 turns warning. Lv2: reveals intensity. Lv3: reveals edit type.",
  "max_level": 3,
  "base_cost": 8,
  "prerequisites": [{ "skill_id": "survival", "required_level": 2 }],
  "passive_effects": [
    { "effect_type": "storm_forecast_turns", "target": "storm", "value_per_level": 5.0, "max_value": 15.0 },
    { "effect_type": "storm_forecast_detail", "target": "storm", "value_per_level": 1.0, "max_value": 3.0 }
  ]
},
{
  "id": "adaptation_resistance",
  "name": "Adaptation Resistance",
  "category": "survival",
  "description": "Slow your body's drift toward glass. Reduces refraction gain rate by 15% per level.",
  "max_level": 3,
  "base_cost": 10,
  "prerequisites": [{ "skill_id": "survival", "required_level": 2 }],
  "passive_effects": [
    { "effect_type": "refraction_gain_multiplier", "target": "adaptation", "value_per_level": -0.15, "max_value": -0.45 }
  ]
},
{
  "id": "salt_hermit_ways",
  "name": "Salt Hermit Ways",
  "category": "survival",
  "description": "The hermits survive because they're already half-salt. Hunger/thirst rate -25% per level.",
  "max_level": 3,
  "base_cost": 12,
  "prerequisites": [{ "skill_id": "survival", "required_level": 3 }],
  "passive_effects": [
    { "effect_type": "hunger_rate_multiplier", "target": "survival", "value_per_level": -0.25, "max_value": -0.75 },
    { "effect_type": "thirst_rate_multiplier", "target": "survival", "value_per_level": -0.25, "max_value": -0.75 }
  ]
}
```

### Crafting & Social Skills

```json
{
  "id": "adaptation_tinctures",
  "name": "Adaptation Tinctures",
  "category": "crafting",
  "description": "Craft items that control refraction: accelerators, suppressors, stabilizers.",
  "max_level": 3,
  "base_cost": 15,
  "prerequisites": [{ "skill_id": "salt_alchemy", "required_level": 3 }],
  "passive_effects": [
    { "effect_type": "adaptation_recipe_tier", "target": "crafting", "value_per_level": 1.0, "max_value": 3.0 }
  ]
},
{
  "id": "reputation_weaving",
  "name": "Reputation Weaving",
  "category": "social",
  "description": "Navigate faction politics without burning bridges. Rival faction rep loss reduced 20% per level.",
  "max_level": 3,
  "base_cost": 12,
  "prerequisites": [{ "skill_id": "faction_lore", "required_level": 4 }],
  "passive_effects": [
    { "effect_type": "rival_rep_loss_multiplier", "target": "faction", "value_per_level": -0.2, "max_value": -0.6 }
  ]
}
```

---

## Phase 1 — Typed Accessors in `skills.rs`

**File**: `src/game/skills.rs`

Add typed accessor methods to `SkillsState`. These wrap the existing `passive_bonuses` HashMap
and are the **only** way systems should read skill effects going forward.

Also migrate the two existing raw lookups in `combat.rs` to use the new accessors (see Phase 2).

```rust
impl SkillsState {
    // --- Existing (rename to match convention if needed) ---
    pub fn melee_damage_multiplier(&self) -> f32 {
        self.passive_bonuses.get("melee_damage_bonus").copied().unwrap_or(0.0)
    }
    pub fn melee_accuracy_bonus(&self) -> f32 {
        self.passive_bonuses.get("melee_accuracy_bonus").copied().unwrap_or(0.0)
    }

    // --- Storm Stance ---
    // Returns (damage_delta, accuracy_delta) based on storm state.
    // storm_active=true: +damage_delta, -accuracy_delta
    // storm_active=false: -damage_delta*0.75, +accuracy_delta
    pub fn storm_stance_bonus(&self, storm_active: bool) -> (f32, f32) {
        let dmg = self.passive_bonuses.get("storm_stance_damage_delta").copied().unwrap_or(0.0);
        let acc = self.passive_bonuses.get("storm_stance_accuracy_delta").copied().unwrap_or(0.0);
        if storm_active {
            (dmg, -acc * 0.75)
        } else {
            (-dmg * 0.75, acc)
        }
    }

    // --- Crucible Technique ---
    pub fn adaptation_damage_bonus(&self, adaptation_count: usize) -> f32 {
        let per_stack = self.passive_bonuses.get("adaptation_damage_per_stack").copied().unwrap_or(0.0);
        per_stack * adaptation_count as f32
    }

    // --- Storm Reading ---
    // Returns (extra_turns_warning, detail_level 0-3)
    pub fn storm_forecast_bonus(&self) -> (i32, u32) {
        let turns = self.passive_bonuses.get("storm_forecast_turns").copied().unwrap_or(0.0) as i32;
        let detail = self.passive_bonuses.get("storm_forecast_detail").copied().unwrap_or(0.0) as u32;
        (turns, detail)
    }

    // --- Adaptation Resistance ---
    pub fn refraction_gain_multiplier(&self) -> f32 {
        1.0 + self.passive_bonuses.get("refraction_gain_multiplier").copied().unwrap_or(0.0)
    }

    // --- Salt Hermit Ways ---
    pub fn hunger_rate_multiplier(&self) -> f32 {
        1.0 + self.passive_bonuses.get("hunger_rate_multiplier").copied().unwrap_or(0.0)
    }
    pub fn thirst_rate_multiplier(&self) -> f32 {
        1.0 + self.passive_bonuses.get("thirst_rate_multiplier").copied().unwrap_or(0.0)
    }

    // --- Adaptation Tinctures ---
    pub fn adaptation_recipe_tier(&self) -> u32 {
        self.passive_bonuses.get("adaptation_recipe_tier").copied().unwrap_or(0.0) as u32
    }

    // --- Reputation Weaving ---
    pub fn rival_rep_loss_multiplier(&self) -> f32 {
        1.0 + self.passive_bonuses.get("rival_rep_loss_multiplier").copied().unwrap_or(0.0)
    }
}
```

**Rule going forward**: every new skill adds one or more accessor methods here. Systems never
touch `passive_bonuses` directly.

---

## Phase 2 — System Integration

One integration point per skill. Each is a small, targeted change.

### 2.1 Storm Stance + Crucible Technique → `src/game/systems/combat.rs`

In the melee damage/accuracy calculation:

```rust
let storm_active = state.world.storm.intensity > 0;
let (storm_dmg_delta, storm_acc_delta) = state.player.skills.storm_stance_bonus(storm_active);
let adapt_bonus = state.player.skills.adaptation_damage_bonus(state.player.adaptations.len());

// Apply to existing damage calc
let total_dmg_mult = 1.0 + state.player.skills.melee_damage_multiplier() + storm_dmg_delta + adapt_bonus;
dmg = (dmg as f32 * total_dmg_mult) as i32;

// Apply to existing accuracy calc
let total_acc_bonus = state.player.skills.melee_accuracy_bonus() + storm_acc_delta;
// (use total_acc_bonus where accuracy is currently calculated)
```

### 2.2 Storm Reading → `src/game/systems/storm.rs`

Where the storm forecast message is generated, extend it based on skill level:

```rust
let (extra_turns, detail_level) = state.player.skills.storm_forecast_bonus();

// Extend the warning window
forecast.turns_until_arrival += extra_turns;

// Add detail based on level
if detail_level >= 1 { forecast.show_intensity = true; }
if detail_level >= 2 { forecast.show_edit_type = true; }
if detail_level >= 3 { forecast.show_duration = true; }
```

### 2.3 Adaptation Resistance → `src/game/systems/storm.rs`

Where refraction is applied during storm exposure:

```rust
let gain = (base_refraction_gain as f32 * state.player.skills.refraction_gain_multiplier())
    .max(0.0) as u32;
state.player.refraction += gain;
```

### 2.4 Salt Hermit Ways → `src/game/state.rs` (resource tick in `end_turn`)

```rust
state.player.hunger = (state.player.hunger as f32 * state.player.skills.hunger_rate_multiplier())
    .max(0.0) as u32;
state.player.thirst = (state.player.thirst as f32 * state.player.skills.thirst_rate_multiplier())
    .max(0.0) as u32;
```

### 2.5 Adaptation Tinctures → `src/game/crafting.rs`

In the recipe availability check:

```rust
let tier = state.player.skills.adaptation_recipe_tier();
if tier >= 1 { available.push("adaptation_accelerator"); }
if tier >= 2 { available.push("adaptation_suppressor"); }
if tier >= 3 { available.push("adaptation_stabilizer"); }
```

Also add the three recipes to `data/recipes.json`.

### 2.6 Reputation Weaving → faction reputation change function

Wherever rival faction rep loss is applied:

```rust
let reduced_loss = (base_loss as f32 * state.player.skills.rival_rep_loss_multiplier())
    .max(0.0) as i32;
```

---

## Phase 3 — DES Tests

Create `tests/scenarios/skills/` directory. One scenario per skill.

| File | Asserts |
|------|---------|
| `storm_stance_test.des` | Damage higher during storm than calm; accuracy lower during storm |
| `crucible_technique_test.des` | Damage increases as adaptations are added |
| `storm_reading_test.des` | Forecast turns extend per level; detail flags set correctly |
| `adaptation_resistance_test.des` | Refraction gain after storm is lower than baseline |
| `salt_hermit_ways_test.des` | Hunger/thirst after N turns is higher than baseline (less consumed) |
| `adaptation_tinctures_test.des` | Recipes available at correct tier thresholds |
| `reputation_weaving_test.des` | Rival rep loss is reduced when gaining rep with a faction |

Each scenario should use `set_skill <id> <level>` to set up the player state, and compare
against a baseline run without the skill.

---

## Scalability Notes

### Adding future skills (the pattern)

1. **JSON entry** in `data/abilities.json` — 5 min
2. **Accessor method** in `skills.rs` — 2 min (or reuse existing if effect type already exists)
3. **Integration point** — 5–15 min (often reuses an existing integration site)
4. **DES scenario** — 5 min

Most skills will reuse existing integration sites (combat damage calc, resource tick, etc.).
New integration sites are only needed when a skill touches a system that has never had skill
effects before.

### When to migrate to `ComputedStats`

Migrate when either:
- The accessor list in `skills.rs` exceeds ~25 methods, or
- Profiling shows HashMap lookup cost in hot paths (unlikely before 50+ skills)

The migration is mechanical: replace `passive_bonuses.get(...)` calls inside accessors with
field reads from a pre-computed struct. No call sites change.

### Effect types already defined (do not duplicate)

| Effect type | Accessor | Used by |
|-------------|----------|---------|
| `melee_damage_bonus` | `melee_damage_multiplier()` | combat.rs |
| `melee_accuracy_bonus` | `melee_accuracy_bonus()` | combat.rs |
| `ranged_damage_bonus` | *(add accessor)* | combat.rs |
| `ranged_accuracy_bonus` | *(add accessor)* | combat.rs |
| `armor_bonus` | *(add accessor)* | combat.rs |
| `resource_efficiency` | *(add accessor)* | state.rs |
| `healing_bonus` | *(add accessor)* | item use |
| `detection_reduction` | *(add accessor)* | ai.rs |
| `encounter_reduction` | *(add accessor)* | movement.rs |
| `craft_success` | *(add accessor)* | crafting.rs |

The *(add accessor)* entries are existing effect types that currently use raw HashMap lookups.
Wrap them in Phase 1 as part of the accessor migration.

---

## Commit Plan

| Commit | Contents |
|--------|----------|
| `feat: add skill definitions for storm_stance, crucible_technique, storm_reading, adaptation_resistance, salt_hermit_ways, adaptation_tinctures, reputation_weaving` | Phase 0 JSON only |
| `refactor: add typed skill accessors to SkillsState, migrate existing raw lookups` | Phase 1 |
| `feat: wire skill passive effects into combat, storm, survival, crafting, faction systems` | Phase 2 |
| `test: DES scenarios for 7 new skills` | Phase 3 |
