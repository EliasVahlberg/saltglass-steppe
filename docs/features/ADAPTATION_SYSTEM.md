# Adaptation System Design

> Status: Design spec — not yet implemented
> Supersedes: existing `data/adaptations.json` (10 placeholder adaptations)
> Last updated: 2026-04-26

---

## Overview

Adaptations are the signature progression system of Saltglass Steppe. They represent permanent, visible physical transformations caused by prolonged refraction exposure. They are not upgrades — they are *crystallizations of how you've been playing*. Each one meaningfully changes how the next 500+ turns feel.

**Design constraints:**
- A semi-long run yields 3 adaptations. A late-game run yields 4–5.
- Each adaptation changes gameplay in a meaningful, observable way — not just stat deltas.
- The options offered at each threshold are weighted by how you've been playing, but not determined by it.
- Adaptations are visibly strange. NPCs and factions react to them.

---

## How It Works

### Refraction accumulation (unchanged)
Refraction builds from walking on glass tiles and surviving storms. It is the gate — you need enough exposure before your body is ready to change.

### Thresholds (redesigned)
Three tiers, spaced to match run length:

| Tier | Refraction required | Typical turn |
|------|--------------------|----|
| 1 | 150 | ~300 |
| 2 | 400 | ~600 |
| 3 | 800 | ~1100 |

A 4th adaptation is possible at 1400 (late-game only). Current thresholds (25–120) are far too low and will be replaced.

### The choice event
When refraction crosses a tier threshold, the game pauses and presents **3 adaptation options**. The player picks one. The others are gone.

The 3 options are drawn from the full pool using weighted selection:
- Adaptations in your **dominant category** (based on activity counters) have 3× weight.
- Adaptations in a **secondary category** have 1.5× weight.
- Adaptations you already own are excluded.
- **Locked adaptations** only enter the pool if their unlock condition is met.
- You cannot be offered more than 1 adaptation from the same category in a single choice event.

### Soft exclusion between categories
Each adaptation carries a category tag. Owning 2+ adaptations from one category reduces the weight of *other* categories' adaptations by 0.5× in future draws. Builds naturally cohere without hard locks.

---

## Activity Counters

Stored on `PlayerState`. Increment on the relevant game events. Used only for adaptation weighting — not displayed to the player directly.

```
storms_survived          — increments when a storm ends while player is alive
glass_tiles_walked       — increments each step on a glass tile
enemies_killed_melee     — increments on melee kill
enemies_killed_ranged    — increments on ranged/psychic kill  
elite_enemies_killed     — increments on elite/boss kill
items_crafted            — increments on successful craft
items_used               — increments on item use (consumables)
psychic_uses             — increments on psychic ability use
tiles_explored           — increments on first visit to a tile
npcs_talked              — increments on NPC dialogue completion
damage_taken_total       — running total of damage received
```

**Category dominance** is determined by comparing normalized scores:
- Survival score: `storms_survived × 3 + glass_tiles_walked × 0.1 + damage_taken_total × 0.05`
- Predator score: `enemies_killed_melee × 2 + elite_enemies_killed × 5`
- Precision score: `enemies_killed_ranged × 2 + psychic_uses × 3`
- Artificer score: `items_crafted × 4 + items_used × 1`

The highest score is dominant (3× weight). Second highest is secondary (1.5×).

---

## Categories and Adaptations

### Alpha release: 4 categories, 5–7 adaptations each

---

### Survival
*You endure what the steppe throws at you. The glass gets into you and you stop fighting it.*

Unlock condition for all: none (available from tier 1)

| ID | Name | Tier | Effect | Faction visibility |
|----|------|------|--------|-------------------|
| `glass_skin` | Glass Skin | 1 | Immune to glass terrain damage. Glass tiles restore 1 HP per step instead of dealing damage. | Moderate — skin has a faint crystalline sheen |
| `storm_drinker` | Storm Drinker | 1 | During storms, gain +2 AP per turn (storms energize rather than hinder). Refraction gain from storms doubled. | Low — only visible during storms |
| `scar_lattice` | Scar Lattice | 2 | Each time you take damage, gain 1 temporary armor (stacks up to 5, resets each combat). You become harder to kill the longer a fight goes. | High — visible crystalline scarring on skin |
| `void_step` | Void Step | 2 | Movement costs 0 AP every 3rd step. You phase slightly between steps — cannot be hit by ranged attacks on the free step. | Moderate — slight visual shimmer |
| `second_breath` | Second Breath | 3 | When HP drops below 25%, automatically gain 15 HP and +3 AP once per combat. Triggers once per encounter. | Low — internal |
| `fracture_body` | Fracture Body | 3 | **Locked**: requires `storms_survived >= 10`. On death, shatter into 3 glass shards that deal damage to adjacent enemies. Respawn at last safe tile with 10 HP. One use per run. | Extreme — body visibly cracked |

---

### Predator
*You kill things. The steppe notices. Your body optimizes for the hunt.*

| ID | Name | Tier | Effect | Faction visibility | Unlock condition |
|----|------|------|--------|-------------------|-----------------|
| `killing_edge` | Killing Edge | 1 | Melee attacks deal +3 damage. After killing an enemy, next attack this turn costs 0 AP. | Low | None |
| `blood_glass` | Blood Glass | 1 | Enemies you kill have a 25% chance to leave a glass shard on the ground. You can pick these up as thrown weapons. | Moderate — hands have glass deposits | None |
| `apex_scent` | Apex Scent | 2 | Enemies with less than 50% HP are highlighted in your FOV. Elite enemies are always visible regardless of FOV. | Low — internal | `enemies_killed_melee >= 15` |
| `bone_spur` | Bone Spur | 2 | Melee attacks have a 20% chance to cause bleed (3 damage/turn for 3 turns). Bleed stacks. | High — visible bone protrusions | None |
| `predator_reflex` | Predator Reflex | 3 | When an enemy attacks you in melee, 30% chance to automatically counter-attack before their hit resolves. | Moderate | `enemies_killed_melee >= 30` |
| `apex_form` | Apex Form | 3 | **Locked**: requires `elite_enemies_killed >= 3`. +5 damage, +3 armor, +1 AP per turn. You are visibly monstrous — most non-hostile NPCs flee on sight. | Extreme — full physical transformation | `elite_enemies_killed >= 3` |

---

### Precision
*You act at a distance. Psychic, ranged, deliberate. The glass sharpens your mind.*

| ID | Name | Tier | Effect | Faction visibility | Unlock condition |
|----|------|------|--------|-------------------|-----------------|
| `lens_eye` | Lens Eye | 1 | FOV range +4. Ranged attacks never miss targets within 6 tiles. | Low — eyes have a glassy quality | None |
| `refraction_shot` | Refraction Shot | 1 | Ranged attacks can bounce off glass walls once, hitting a second target for 50% damage. | Low | None |
| `still_mind` | Still Mind | 2 | Psychic ability cooldowns reduced by 1 turn. Coherence regenerates 1 point per turn (was 0). | Low — internal | `psychic_uses >= 10` |
| `mirage_step` | Mirage Step | 2 | When you move, leave a decoy at your previous position for 2 turns. Enemies will attack the decoy first. | Moderate — slight visual doubling | None |
| `glass_sight` | Glass Sight | 3 | See through glass walls. Enemies behind glass walls can be targeted with ranged attacks at -2 damage. | High — eyes fully crystalline | `tiles_explored >= 500` |
| `mind_shatter` | Mind Shatter | 3 | **Locked**: requires `psychic_uses >= 25`. Once per combat, release a psychic burst that stuns all enemies in FOV for 2 turns. 20-turn cooldown. | Extreme — visible psychic aura | `psychic_uses >= 25` |

---

### Artificer
*You make things. Use things. The steppe is a resource, not a threat.*

| ID | Name | Tier | Effect | Faction visibility | Unlock condition |
|----|------|------|--------|-------------------|-----------------|
| `salt_sense` | Salt Sense | 1 | Loot containers and item drops are highlighted in FOV. Crafting recipes require 1 fewer ingredient (minimum 1). | Low — internal | None |
| `catalyst_body` | Catalyst Body | 1 | Consumable items have 30% chance to not be consumed on use. Healing items restore 50% more HP. | Low — internal | None |
| `glass_forge` | Glass Forge | 2 | Can craft glass weapons and tools from glass shards found in the world. Unlocks 3 new recipes. | Moderate — hands have forge-calluses | `items_crafted >= 8` |
| `storm_harvest` | Storm Harvest | 2 | During storms, automatically collect 1–3 storm glass shards per turn. Storm glass is a high-value crafting material. | Low | `storms_survived >= 5` |
| `living_inventory` | Living Inventory | 3 | Carry capacity +5. Items in inventory slowly repair themselves (1 durability/10 turns). | Low — internal | `items_used >= 20` |
| `transmutation` | Transmutation | 3 | **Locked**: requires `items_crafted >= 20`. Once per tile, convert any 3 items into 1 random item of higher rarity. | High — hands glow faintly | `items_crafted >= 20` |

---

## Faction Visibility

Each adaptation has a visibility level that affects NPC and faction reactions:

| Level | Description | Faction effect |
|-------|-------------|----------------|
| Low | Subtle, internal, or only visible up close | No passive reaction; dialogue may reference it |
| Moderate | Visible to observant NPCs | Some NPCs comment; Mirror Monks react positively |
| High | Obvious physical transformation | Salt Traders become wary; Mirror Monks revere; Archive Drones want to study |
| Extreme | Unmistakable, monstrous | Most NPCs hostile or flee; Mirror Monks treat as saint; Salt Traders refuse service |

Implementation: faction reaction is a modifier on existing reputation checks, not a separate system. A High-visibility adaptation applies a -10 reputation modifier with Salt Traders and +15 with Mirror Monks, passively, while owned.

---

## UI Flow

1. Refraction crosses tier threshold mid-turn.
2. Current turn completes normally.
3. Screen dims. A full-screen panel appears: **"Your body is ready to change."**
4. Three adaptation cards shown side by side, each with: name, category, description, faction visibility warning.
5. Player navigates with arrow keys, confirms with Enter. Escape is disabled — the choice must be made.
6. Brief visual effect plays (screen flash, particle burst).
7. Adaptation is added to player state. Faction reputation modifiers applied immediately.
8. Game resumes.

---

## Implementation Scope

### Phase 1 — Mechanic (no new adaptations yet)
- Add activity counters to `PlayerState`
- Wire counter increments to existing game events
- Replace `rule_check_adaptation` with new weighted selection logic
- Redesign refraction thresholds (150 / 400 / 800 / 1400)
- Build choice UI panel
- Update `AdaptationDef` schema: add `category`, `unlock_condition`, `faction_visibility`, `tier`

### Phase 2 — New adaptations
- Replace `data/adaptations.json` with the 24 adaptations above
- Wire each adaptation's gameplay effect in code
- Add faction reputation modifiers on adaptation gain
- Add `glass_forge` crafting recipes

### Phase 3 — Polish
- NPC dialogue lines that reference visible adaptations
- Visual indicators on player glyph for High/Extreme visibility adaptations
- DES scenarios for each adaptation category

---

## What Happens to Existing Adaptations

The 10 existing adaptations in `data/adaptations.json` are replaced entirely. The `Adaptation` enum in `src/game/adaptation.rs` will be updated to match the new IDs. Save files from before this change will not be compatible (save version bump required).

`saltblood` (glass immunity) is absorbed into `glass_skin`. `mirage_step` is kept as-is. The rest are retired.

---

## Open Questions

- Should the choice UI show which category each adaptation belongs to, or keep it implicit? (Leaning: show it — players should understand the system)
- Should activity counters be visible to the player anywhere? (Leaning: no — let the adaptation choices feel like the world responding to you, not a checklist)
- Faction visibility modifiers: flat reputation delta, or multiplier on existing reputation? (Leaning: flat delta, simpler to reason about)
