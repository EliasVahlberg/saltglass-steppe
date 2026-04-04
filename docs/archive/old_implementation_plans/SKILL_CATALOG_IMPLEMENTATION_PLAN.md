# Skill Catalog Implementation Plan

**Status**: Planned — Next Major Feature  
**Tier**: 1 (foundational — unblocks Adaptations Rework and Storm System)  
**Depends on**: Nothing (standalone)  
**Unlocks**: Adaptations Rework (item 9), faction skill bonuses, class differentiation

---

## Current State

`src/game/skills.rs` (310 lines) has a working foundation:
- `SkillDef` with `prerequisites`, `passive_effects`, `ability_unlock`
- `SkillsState` with `upgrade_skill`, `can_upgrade_skill`, `recalculate_passive_bonuses`
- `PassiveEffect` applies stat modifiers per level
- Skill point allocation UI exists

`data/abilities.json` has 10 skills and 50 abilities, but:
- Only 5 categories with 1–4 skills each (sparse)
- No Psychic category (listed in roadmap, not implemented)
- Passive effects exist but most are not wired into gameplay
- Active abilities exist in data but are not usable in-game
- No skill synergies
- No faction or class starting bonuses
- No alternative skill point sources (only level-up)

---

## Goals

1. **Full skill tree** — 5–6 skills per category, 3 prerequisite tiers max, meaningful choices at every level
2. **Active abilities wired** — usable in combat/exploration with stamina costs and cooldowns
3. **Passive effects wired** — all passive bonuses actually affect gameplay
4. **Faction/class bonuses** — starting skill levels based on class and faction affiliation
5. **Synergy bonuses** — specific skill combinations unlock bonus effects
6. **Alternative point sources** — quests and exploration discoveries award skill points

---

## Skill Categories & Tree Design

### Combat (6 skills)
```
blade_mastery (no prereq)         → melee damage/accuracy
  └─ whirlwind_technique (blade 3) → AoE melee ability unlock
shield_discipline (no prereq)     → block chance, damage reduction
  └─ counter_strike (shield 3)    → riposte ability unlock
glass_fighting (blade 2)          → bonus vs glass/crystal enemies, glass weapon proficiency
```

### Survival (6 skills)
```
desert_conditioning (no prereq)   → heat/cold resistance, stamina regen
  └─ endurance (desert 3)         → max stamina, travel cost reduction
wayfaring (no prereq)             → travel cost reduction, encounter avoidance
  └─ navigation (wayfaring 2)     → world map reveal radius, pathfinding
field_medicine (no prereq)        → healing item effectiveness, bleed resistance
  └─ herbalism (medicine 2)       → craft healing items from biome resources
```

### Psychic (6 skills) — new category
```
storm_sense (no prereq)           → storm forecast accuracy, storm damage resistance
  └─ storm_reading (storm 3)      → predict storm edit types before they happen
void_attunement (no prereq)       → void energy capacity, void ability potency
  └─ void_channeling (void 3)     → unlock void drain ability
refraction_affinity (no prereq)   → refraction gain rate, adaptation threshold reduction
  └─ light_bending (refraction 3) → unlock light deflect ability
```

### Social (5 skills)
```
salt_trading (no prereq)          → buy/sell price improvement
  └─ haggling (trading 3)         → unlock haggle dialogue option
steppe_lore (no prereq)           → faction reputation gain rate, NPC dialogue options
  └─ faction_standing (lore 3)    → unlock faction-exclusive quests
intimidation (no prereq)          → enemy morale checks, flee chance
```

### Crafting (5 skills)
```
glass_working (no prereq)         → craft glass weapons/armor, repair efficiency
  └─ crystal_shaping (glass 3)    → craft crystal resonance items
salt_alchemy (no prereq)          → potion effectiveness, crafting cost reduction
  └─ brine_synthesis (alchemy 3)  → unlock advanced recipes
scavenging (no prereq)            → item find rate, chest loot quality
```

---

## Active Ability System

Active abilities are unlocked by reaching a skill level threshold. They are used via the ability hotbar (UI already exists).

### Ability execution flow
```
Player presses ability hotkey
→ check stamina >= cost
→ check cooldown == 0
→ apply effect (damage, buff, movement, etc.)
→ deduct stamina, set cooldown
→ emit GameEvent::AbilityUsed { id }
```

### Ability effect types (extend existing `AbilityDef`)
Add `effect_type` and `targeting` fields to `AbilityDef`:

```json
{
  "id": "whirlwind_strike",
  "name": "Whirlwind Strike",
  "stamina_cost": 20,
  "cooldown": 5,
  "effect_type": "aoe_damage",
  "targeting": "self_centered",
  "range": 1,
  "damage_multiplier": 0.8
}
```

Effect types to implement: `melee_attack`, `aoe_damage`, `self_buff`, `enemy_debuff`, `movement`, `utility`.

---

## Passive Effect Wiring

Currently `recalculate_passive_bonuses()` computes totals but most aren't read. Wire each stat into the relevant system:

| Stat | Where to read |
|------|--------------|
| `melee_damage_bonus` | `combat.rs` attack calculation |
| `melee_accuracy_bonus` | `combat.rs` hit chance |
| `stamina_regen` | `state.rs` end_turn stamina tick |
| `travel_cost_reduction` | `travel.rs` cost calculation |
| `healing_effectiveness` | item use handler |
| `trade_price_modifier` | `trading.rs` price calculation |
| `storm_resistance` | `systems/storm.rs` damage calculation |
| `void_capacity_bonus` | `void_energy.rs` max capacity |

---

## Faction & Class Starting Bonuses

Add `starting_skill_bonuses: Vec<(String, u32)>` to `ClassDef` and `FactionDef`.

Examples:
- Wanderer class: `wayfaring +1`, `desert_conditioning +1`
- Mirror Monks faction: `refraction_affinity +1`, `steppe_lore +1`
- Salt Traders faction: `salt_trading +2`
- Sand Engineers faction: `glass_working +1`, `crafting +1`

Applied in `GameState::new_with_class()` after skill state initialization.

---

## Skill Synergies

Synergy bonuses activate when the player has both skills at a minimum level. Checked in `recalculate_passive_bonuses()`.

```json
{
  "synergies": [
    {
      "id": "glass_warrior",
      "requires": [{"skill": "blade_mastery", "level": 3}, {"skill": "glass_fighting", "level": 2}],
      "bonus": {"stat": "melee_damage_bonus", "value": 0.15}
    },
    {
      "id": "desert_wanderer",
      "requires": [{"skill": "wayfaring", "level": 3}, {"skill": "desert_conditioning", "level": 3}],
      "bonus": {"stat": "travel_cost_reduction", "value": 0.25}
    }
  ]
}
```

Store synergies in `abilities.json` under a `"synergies"` key. Check in `recalculate_passive_bonuses()`.

---

## Alternative Skill Point Sources

Add `bonus_skill_points: u32` to `PlayerState`. Sources:
- Quest completion reward (add `skill_points: u32` field to quest reward schema)
- Exploration discovery (rare interactable: "Ancient Training Ground" → +1 skill point)
- Faction standing milestone (reaching Honored with a faction → +1 skill point)

---

## Implementation Order

### Phase 1 — Wire existing passives (small, high impact)
1. Read `passive_bonuses` in `combat.rs`, `travel.rs`, `trading.rs`
2. Wire `stamina_regen` into end_turn
3. All existing 10 skills now have real gameplay effect

### Phase 2 — Expand skill tree content
1. Add Psychic category to `abilities.json`
2. Expand each category to 5–6 skills with prerequisites
3. Add synergy definitions
4. `cargo test --lib` must pass

### Phase 3 — Active ability execution
1. Add `effect_type` + `targeting` to `AbilityDef`
2. Implement `execute_ability(state, ability_id)` in `skills.rs`
3. Wire into input handler (ability hotkeys)
4. Emit `GameEvent::AbilityUsed`

### Phase 4 — Faction/class bonuses + alternative point sources
1. Add `starting_skill_bonuses` to class/faction defs
2. Apply in `new_with_class()`
3. Add `bonus_skill_points` to quest rewards and exploration

---

## Files Touched

| File | Change |
|------|--------|
| `data/abilities.json` | Expand to ~28 skills, add Psychic category, add synergies |
| `src/game/skills.rs` | Add synergy checking, `execute_ability()`, alternative point sources |
| `src/game/systems/combat.rs` | Read `melee_damage_bonus`, `melee_accuracy_bonus` |
| `src/game/travel.rs` | Read `travel_cost_reduction` |
| `src/game/trading.rs` | Read `trade_price_modifier` |
| `src/game/state.rs` | Wire `stamina_regen` in end_turn, apply class/faction bonuses |
| `src/game/systems/storm.rs` | Read `storm_resistance` |
| `data/classes.json` | Add `starting_skill_bonuses` |
| `data/factions.json` | Add `starting_skill_bonuses` |

---

## Success Criteria

- All 5 categories have 5–6 skills with meaningful prerequisites
- Every passive effect is read by at least one gameplay system
- At least 3 active abilities are usable in combat
- 3+ distinct viable builds exist (glass warrior, desert wanderer, storm caller, trader)
- Faction/class choice has visible skill impact from turn 1
- `cargo test --lib` passes
