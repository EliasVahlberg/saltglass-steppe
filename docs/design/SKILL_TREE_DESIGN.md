# Skill Tree Design

> The steppe does not teach gently. Every scar is a lesson; every lesson, a scar.

## Overview

Seven root categories. Each is an independent tree. Skills within a tree have prerequisites; some skills have cross-tree prerequisites (noted explicitly). Tiers T1–T4 represent depth, not power gating — a T4 skill is transformative, not mandatory.

**Blocker legend:**
- ✅ Safe — implement now, no rework risk
- ⚠️ Partial — define in JSON now, wire integration later
- ❌ Blocked — depends on unimplemented or rework-scheduled system

**Complexity legend (implementation effort):**
- Low — JSON only (`abilities.json` / `recipes.json`)
- Medium — JSON + minimal Rust (new passive key consumed in one system)
- High — new mechanic, new entity type, or new system hook

---

## Summary Table

| Skill | Category | Tier | Status | Complexity |
|---|---|---|---|---|
| Crucible Techniques | Salt Alchemy | T1 | ✅ | Low |
| Adaptation Tinctures | Salt Alchemy | T2 | ✅ | Medium |
| Poison Creation | Salt Alchemy | T2 | ✅ | Low |
| Contact Poison | Salt Alchemy | T3 | ⚠️ | Medium |
| Brine Distillation | Salt Alchemy | T1 | ✅ | Low |
| Salt Preservation | Salt Alchemy | T2 | ✅ | Medium |
| Void Reagents | Salt Alchemy | T2 | ✅ | Low |
| Void Glass Synthesis | Salt Alchemy | T3 | ✅ | Low |
| Medicine Synthesis | Salt Alchemy | T1 | ✅ | Low |
| Advanced Medicine Synthesis | Salt Alchemy | T2 | ✅ | Low |
| Medicine Fabrication Bench | Salt Alchemy | T3 | ✅ | Medium |
| Scrap Salvage | Crafting | T1 | ✅ | Low |
| Expert Salvage | Crafting | T2 | ✅ | Medium |
| Weapon Modding | Crafting | T1 | ✅ | Low |
| Advanced Modding | Crafting | T2 | ✅ | Low |
| Masterwork Modding | Crafting | T3 | ✅ | Low |
| Mechanical Fabrication | Crafting | T1 | ✅ | Low |
| Clockwork Devices | Crafting | T2 | ⚠️ | High |
| Trap Setting | Crafting | T2 | ⚠️ | High |
| Advanced Traps | Crafting | T3 | ⚠️ | High |
| Ammo Fabrication | Crafting | T1 | ✅ | Low |
| Specialty Ammo | Crafting | T2 | ✅ | Low |
| Lab Equipment | Crafting | T1 | ✅ | Low |
| Prefab Shelter | Crafting | T1 | ❌ | High |
| Fortified Shelter | Crafting | T2 | ❌ | High |
| Faction Lore | Social | T1 | ✅ | Low |
| Faction Expertise | Social | T2 | ✅ | Medium |
| Faction Insider | Social | T3 | ✅ | Low |
| Outcast Signs | Social | T2 | ✅ | Medium |
| Reputation Weaving | Social | T3 | ⚠️ | Medium |
| Gossip & Rumors | Social | T1 | ✅ | Medium |
| Intelligence Network | Social | T2 | ✅ | Low |
| Bartering | Social | T1 | ✅ | Low |
| Hard Bargaining | Social | T2 | ✅ | Low |
| Black Market Access | Social | T3 | ✅ | Low |
| Trader's Tongue | Social | T2 | ⚠️ | Medium |
| Intimidation | Social | T1 | ✅ | Low |
| Berate | Social | T2 | ✅ | Medium |
| Inspiring Presence | Social | T1 | ⚠️ | High |
| Pilgrim Courtesy | Social | T1 | ✅ | Low |
| Monk Rhetoric | Social | T2 | ✅ | Medium |
| Scavenging | Survival | T1 | ✅ | Low |
| Expert Scavenging | Survival | T2 | ✅ | Medium |
| Corpse Reading | Survival | T1 | ✅ | Medium |
| Anatomy Knowledge | Survival | T2 | ✅ | Medium |
| Weak Point Analysis | Survival | T3 | ✅ | Medium |
| Pickpocket | Survival | T1 | ⚠️ | High |
| Master Thief | Survival | T2 | ⚠️ | High |
| Track & Trace | Survival | T1 | ✅ | Low |
| Ambush Predator | Survival | T2 | ✅ | Medium |
| Dust Walking | Survival | T1 | ✅ | Low |
| Ghost Step | Survival | T2 | ✅ | Low |
| Biome Lore | Survival | T1 | ✅ | Low |
| Biome Mastery | Survival | T2 | ✅ | Low |
| Storm Reading | Survival | T1 | ❌ | Medium |
| Storm Sense | Survival | T2 | ❌ | Medium |
| Salt Hermit Ways | Survival | T1 | ❌ | High |
| Basic Medical Practice | Medical | T1 | ✅ | Low |
| Wound Packing | Medical | T2 | ✅ | Medium |
| Glass Extraction | Medical | T2 | ✅ | Medium |
| Triage | Medical | T2 | ✅ | Medium |
| Intermediary Medical Practice | Medical | T3 | ✅ | Low |
| Wound Sutures | Medical | T3 | ✅ | Medium |
| Medicine Understanding | Medical | T3 | ✅ | Low |
| Advanced Medical Practice | Medical | T4 | ✅ | Low |
| Self-Operating | Medical | T4 | ⚠️ | High |
| Field Surgery | Medical | T4 | ⚠️ | High |
| Stimulant Use | Medical | T1 | ✅ | Medium |
| Stimulant Synthesis | Medical | T2 | ✅ | Low |
| Glass Fighting | Melee Combat | T1 | ✅ | Medium |
| Seam Breaker | Melee Combat | T2 | ✅ | Medium |
| Fracture Strike | Melee Combat | T3 | ✅ | Medium |
| Counter-Refraction | Melee Combat | T2 | ✅ | Medium |
| Angle Reading | Melee Combat | T1 | ✅ | Low |
| Void Strike (melee) | Melee Combat | T2 | ✅ | Medium |
| Vortex Footwork | Melee Combat | T1 | ✅ | Low |
| Evasive Maneuvers | Melee Combat | T2 | ⚠️ | High |
| Phantom Step | Melee Combat | T3 | ✅ | Low |
| Salt Flurry | Melee Combat | T1 | ✅ | Medium |
| Brine Splash (melee) | Melee Combat | T2 | ✅ | Medium |
| Storm Stance | Melee Combat | T1 | ❌ | High |
| Draw a Bead | Ranged Combat | T1 | ✅ | Low |
| Sniper's Eye | Ranged Combat | T2 | ✅ | Medium |
| Trick Shot | Ranged Combat | T3 | ⚠️ | High |
| Aimed Shot | Ranged Combat | T2 | ✅ | Medium |
| Suppressive Shards | Ranged Combat | T1 | ✅ | Medium |
| Glass Barrage | Ranged Combat | T2 | ⚠️ | High |
| Shard Storm | Ranged Combat | T3 | ❌ | High |
| Brine Volley | Ranged Combat | T1 | ✅ | Medium |
| Corrosive Rounds | Ranged Combat | T2 | ✅ | Medium |
| Ammo Conservation | Ranged Combat | T1 | ✅ | Medium |
| Scavenger's Arsenal | Ranged Combat | T2 | ✅ | Medium |
| Void Aim | Ranged Combat | T1 | ✅ | Low |
| Void Barrage | Ranged Combat | T2 | ⚠️ | High |


---

## 1. Salt Alchemy

> The steppe's minerals are not inert. They remember what they were before the glass came.

```
Salt Alchemy (root)
├── Crucible Techniques (T1)
│   ├── Adaptation Tinctures (T2)
│   └── Poison Creation (T2)
│       └── Contact Poison (T3)
├── Brine Distillation (T1)
│   ├── Salt Preservation (T2)
│   └── Void Reagents (T2)
│       └── Void Glass Synthesis (T3)
└── Medicine Synthesis (T1)          [requires: Medicine Understanding]
    └── Advanced Medicine Synthesis (T2)
        └── Medicine Fabrication Bench (T3)
```

### Crucible Techniques (T1)
**Description:** You have learned to work the crucible — to coax reactions from salt, glass, and brine that others call impossible.
**Effect:** +10% `craft_success` for alchemy recipes. Unlocks tincture crafting recipes in `recipes.json`.
**Blocker:** ✅ Safe
**Implementation:** Add passive effect `craft_success` to `abilities.json` entry. Add tincture recipes to `data/recipes.json` with `skill_required: 1` and `station_required: "crucible"`. Add `crucible` to `data/interactables.json` and `spawn_crafting_stations()`.
**Balance:** Low SP cost (1). Broad unlock — intentionally a gateway node.

### Adaptation Tinctures (T2)
**Description:** Distilled from the bodies of adapted creatures, these tinctures grant brief echoes of their mutations.
**Effect:** Unlocks tincture recipes that grant temporary adaptation-like status effects (e.g. glass resistance for 5 turns). Bonus scales with `player.adaptations.len()` — more adaptations = longer duration.
**Blocker:** ✅ Safe — reads `adaptations.len()` only, does not modify the adaptation system.
**Implementation:** New recipes in `recipes.json`. Duration scaling: read `adaptations.len()` in item use handler in `skills_menu.rs` or item use path.
**Balance:** T2 cost (2 SP). Tincture duration 3–8 turns. Requires materials that are moderately rare.

### Poison Creation (T2)
**Description:** Salt and void-glass, combined with patience, yield compounds that linger in the blood.
**Effect:** Unlocks poison recipes. Poisons apply `poisoned` status effect via existing `effects.json`.
**Blocker:** ✅ Safe
**Implementation:** Add recipes to `recipes.json`. `poisoned` status effect already exists or add to `data/effects.json`.
**Balance:** T2 cost (2 SP). Poisons deal 2–4 damage/turn for 4–6 turns. Requires rare reagents to prevent trivialization.

### Contact Poison (T3)
**Description:** The poison does not wait to be swallowed. It waits on the blade.
**Effect:** Poisons can be applied to weapons. Attacks have a chance to apply `poisoned` to the target.
**Blocker:** ⚠️ Partial — needs a `weapon_coated: Option<String>` field on player equipment state.
**Implementation:** Add `weapon_coated` field to `PlayerState`. Check in `systems/combat.rs` melee hit path. Wire when equipment state is stable.
**Balance:** T3 cost (3 SP). Coating consumed after 3–5 hits. Proc chance 40%.

### Brine Distillation (T1)
**Description:** Brine is not merely water. Properly distilled, it becomes a solvent, a preservative, a weapon.
**Effect:** +15% yield on brine-based crafting recipes (`resource_efficiency` passive for brine recipes).
**Blocker:** ✅ Safe
**Implementation:** New passive key `brine_craft_efficiency` in `abilities.json`. Consume in `crafting.rs` yield calculation, or use existing `resource_efficiency` key with a recipe tag filter.
**Balance:** T1 cost (1 SP). Yield bonus is minor — value is in unlocking the subtree.

### Salt Preservation (T2)
**Description:** Properly salted, a remedy lasts through three storms. Improperly salted, it lasts through none.
**Effect:** Crafted consumables have +50% duration (status effects they apply last longer).
**Blocker:** ✅ Safe — new passive key, consumed in item use path.
**Implementation:** New passive key `item_duration_bonus` in `abilities.json`. Read in item use handler when applying timed status effects.
**Balance:** T2 cost (2 SP). Significant for tinctures and poisons. Does not affect instant-use items.

### Void Reagents (T2)
**Description:** Void-glass is not merely dangerous. In the right hands, it is a component.
**Effect:** Unlocks void-glass crafting recipes (void-glass tools, components).
**Blocker:** ✅ Safe
**Implementation:** Add recipes to `recipes.json`. Void-glass as a material already exists in loot tables.
**Balance:** T2 cost (2 SP). Recipes require rare void-glass drops.

### Void Glass Synthesis (T3)
**Description:** You can now shape void-glass into weapons and instruments that others dare not touch.
**Effect:** Unlocks T2 void-glass recipes (weapons, specialized tools with unique properties).
**Blocker:** ✅ Safe
**Implementation:** Add recipes to `recipes.json` with `skill_required: 3` and `station_required: "crucible"`.
**Balance:** T3 cost (3 SP). End-game crafting branch. Void-glass weapons should have unique passive effects.

### Medicine Synthesis (T1)
**Description:** The body is a system. Knowing its inputs, you can craft its remedies.
**Effect:** Unlocks T1 medicine recipes (bandages, antiseptic, basic stimulants).
**Prerequisite:** Medicine Understanding (Medical tree T3) — cross-tree unlock.
**Blocker:** ✅ Safe
**Implementation:** Add recipes to `recipes.json` with `skill_required: 1`. Cross-tree prerequisite: add `requires_skill: "medicine_understanding"` field to the skill definition in `abilities.json`; check in `SkillsState::can_learn()`.
**Balance:** T1 cost (1 SP) but gated behind Medical T3 — represents a meaningful investment.

### Advanced Medicine Synthesis (T2)
**Description:** Fewer materials. Stronger compounds. The same result, achieved with less waste.
**Effect:** Medicine recipes require 25% fewer materials. +`craft_quality` for medicine items.
**Blocker:** ✅ Safe
**Implementation:** `craft_quality` passive in `abilities.json`. Material reduction: apply `resource_efficiency` passive in `crafting.rs` for medicine-tagged recipes.
**Balance:** T2 cost (2 SP). Significant late-game value for medicine-focused builds.

### Medicine Fabrication Bench (T3)
**Description:** A bench of your own construction, calibrated to your methods. No apothecary's table compares.
**Effect:** Unlocks the recipe to craft a Medicine Fabrication Bench (placeable interactable). Required station for T2+ medicine recipes.
**Blocker:** ✅ Safe — `station_required` in `crafting.rs` and proximity-based `available_stations()` in `state.rs` already support this.
**Implementation:** Add `medicine_fabrication_bench` to `data/interactables.json`. Add to `spawn_crafting_stations()` in `state.rs`. Add bench crafting recipe to `recipes.json` with `station_required: "crucible"`.
**Balance:** T3 cost (3 SP). The bench is a significant investment but enables the most powerful medicine recipes.


---

## 2. Crafting

> The steppe provides nothing finished. Everything useful was made by someone who refused to accept what they were given.

```
Crafting (root)
├── Scrap Salvage (T1)
│   └── Expert Salvage (T2)
├── Weapon Modding (T1)
│   └── Advanced Modding (T2)
│       └── Masterwork Modding (T3)
├── Mechanical Fabrication (T1)
│   ├── Clockwork Devices (T2)          [⚠️ Partial]
│   └── Trap Setting (T2)              [⚠️ Partial]
│       └── Advanced Traps (T3)        [⚠️ Partial]
├── Ammo Fabrication (T1)
│   └── Specialty Ammo (T2)
├── Lab Equipment (T1)
│   └── Medicine Fabrication Bench (T2) [requires: Advanced Medicine Synthesis]
├── Prefab Shelter (T1)                 [❌ Blocked]
└── Fortified Shelter (T2)             [❌ Blocked]
```

### Scrap Salvage (T1)
**Description:** Where others see wreckage, you see components.
**Effect:** +20% materials recovered from salvage and loot containers. `resource_efficiency` passive.
**Blocker:** ✅ Safe
**Implementation:** `resource_efficiency` passive key already exists in `abilities.json`. Consume in loot/salvage paths.
**Balance:** T1 cost (1 SP). Broad utility — intentional. Does not affect enemy drops, only containers.

### Expert Salvage (T2)
**Description:** The rarest components hide in the most ruined places.
**Effect:** Chance (20%) to find a rare component in otherwise empty containers. Hooks into `EnemyKilled` loot table and chest loot.
**Blocker:** ✅ Safe
**Implementation:** Check `passive_bonuses["expert_salvage_chance"]` in loot resolution. Add rare component loot table entries.
**Balance:** T2 cost (2 SP). 20% proc rate. Rare components should be meaningful but not game-breaking.

### Weapon Modding (T1)
**Description:** A weapon is not finished when it leaves the forge. It is finished when it fits your hand.
**Effect:** Unlocks weapon mod recipes (damage mod, accuracy mod, range mod). Requires `crafting_table` station.
**Blocker:** ✅ Safe
**Implementation:** Add mod recipes to `recipes.json` with `station_required: "crafting_table"` and `skill_required: 1`. Mods are items applied via inventory action.
**Balance:** T1 cost (1 SP). Mods should be consumable (one-time application) to prevent stacking.

### Advanced Modding (T2)
**Description:** The second layer of modification is where weapons become instruments.
**Effect:** Unlocks T2 mod recipes (elemental damage mods, special effect mods).
**Blocker:** ✅ Safe
**Implementation:** Add T2 mod recipes to `recipes.json` with `skill_required: 2`.
**Balance:** T2 cost (2 SP). T2 mods should have meaningful tradeoffs (e.g. +damage but -durability).

### Masterwork Modding (T3)
**Description:** Occasionally, the work exceeds the plan. You have learned to recognize when that is happening.
**Effect:** 15% chance any mod application produces a masterwork result (+50% mod effect).
**Blocker:** ✅ Safe
**Implementation:** New passive key `masterwork_mod_chance`. Check in mod application handler.
**Balance:** T3 cost (3 SP). 15% proc rate. Masterwork mods are not re-applicable — the weapon keeps the bonus permanently.

### Mechanical Fabrication (T1)
**Description:** Gears, springs, and tension. The steppe's machines are crude but they work.
**Effect:** Unlocks mechanical component recipes (gears, springs, frames).
**Blocker:** ✅ Safe
**Implementation:** Add component recipes to `recipes.json`. Components are intermediate crafting materials.
**Balance:** T1 cost (1 SP). Gateway to the mechanical subtree.

### Clockwork Devices (T2)
**Description:** A machine that acts without you. The steppe has few of these. You intend to change that.
**Effect:** Craft automated devices (alarm tripwires, auto-turrets). Devices are placed as interactables.
**Blocker:** ⚠️ Partial — needs new interactable entity type with autonomous behavior.
**Implementation:** Define in `abilities.json` and `recipes.json` now. Wire when interactable AI behavior is implemented. Turret behavior can reuse `HealerBehavior` pattern from `systems/ai.rs`.
**Balance:** T2 cost (2 SP). Devices should have limited durability and require maintenance.

### Trap Setting (T2)
**Description:** Patience is a weapon. You have learned to leave it behind you.
**Effect:** Craft and place traps (glass spike trap, brine snare). Traps trigger on enemy movement.
**Blocker:** ⚠️ Partial — needs new interactable type with trigger-on-step logic.
**Implementation:** Define recipes now. Wire trigger logic in `systems/movement.rs` enemy movement path.
**Balance:** T2 cost (2 SP). Traps are single-use. Placement limited to 3 active traps.

### Advanced Traps (T3)
**Description:** The trap that kills is less useful than the trap that teaches.
**Effect:** Traps can apply status effects (poisoned, suppressed, burning). Unlocks T2 trap recipes.
**Blocker:** ⚠️ Partial — depends on Trap Setting.
**Implementation:** Add status-effect traps to `recipes.json`. Wire status application in trap trigger handler.
**Balance:** T3 cost (3 SP). Status traps deal less direct damage but have lasting effects.

### Ammo Fabrication (T1)
**Description:** You do not wait for traders to restock. You make what you need.
**Effect:** Unlocks basic ammo crafting recipes. `resource_efficiency` passive for ammo crafting.
**Blocker:** ✅ Safe
**Implementation:** Add ammo recipes to `recipes.json`. Use existing `resource_efficiency` passive.
**Balance:** T1 cost (1 SP). Crafted ammo should be slightly less efficient than purchased to preserve trader value.

### Specialty Ammo (T2)
**Description:** Glass rounds for armored targets. Brine bolts for those who burn.
**Effect:** Unlocks specialty ammo recipes: glass rounds (+20% vs armored enemies), brine bolts (apply `wet` status).
**Blocker:** ✅ Safe — new recipes + existing status system.
**Implementation:** Add recipes to `recipes.json`. Glass rounds: new passive key `glass_round_armor_bonus` checked in combat. Brine bolts: apply `wet` status on hit via `effects.json`.
**Balance:** T2 cost (2 SP). Specialty ammo costs more materials than basic ammo. Situational value.

### Lab Equipment (T1)
**Description:** The right tools make the right work possible.
**Effect:** Unlocks lab equipment recipes (distillation coil, reaction vessel). Required for advanced alchemy stations.
**Blocker:** ✅ Safe
**Implementation:** Add equipment recipes to `recipes.json`. Equipment items are used to craft stations.
**Balance:** T1 cost (1 SP). Gateway to the Medicine Fabrication Bench.

### Prefab Shelter (T1)
**Description:** A shelter you carry. A shelter you build. The storm does not care which.
**Effect:** Craft portable shelters that provide storm protection when deployed.
**Blocker:** ❌ Blocked — shelter system is part of Storm System Rework (Roadmap Tier 2, item 10).
**Implementation:** Define in `abilities.json` and `recipes.json` now with a `blocked: true` flag. Wire when storm rework lands.
**Balance:** T1 cost (1 SP). Shelters should be fragile and single-storm use.

### Fortified Shelter (T2)
**Description:** A shelter that lasts more than one storm. A rare thing.
**Effect:** Craft reinforced shelters with higher storm resistance and multi-use durability.
**Blocker:** ❌ Blocked — depends on Prefab Shelter and storm rework.
**Implementation:** Define now. Wire with storm rework.
**Balance:** T2 cost (2 SP). Requires significantly more materials than Prefab Shelter.


---

## 3. Social

> The steppe is not empty. It is full of people who have learned not to trust each other. Knowing this is the first skill.

```
Social (root)
├── Faction Lore (T1)
│   ├── Faction Expertise (T2)
│   │   └── Faction Insider (T3)
│   └── Outcast Signs (T2)
│       └── Reputation Weaving (T3)    [⚠️ Partial]
├── Gossip & Rumors (T1)
│   └── Intelligence Network (T2)
├── Bartering (T1)
│   ├── Hard Bargaining (T2)
│   │   └── Black Market Access (T3)
│   └── Trader's Tongue (T2)           [⚠️ Partial]
├── Intimidation (T1)
│   └── Berate (T2)
├── Inspiring Presence (T1)            [⚠️ Partial]
└── Pilgrim Courtesy (T1)
    └── Monk Rhetoric (T2)
```

### Faction Lore (T1)
**Description:** You know who holds power here, and what they want from those who pass through.
**Effect:** Reveals faction reputation thresholds in the UI. Passive `map_reveal` for faction-controlled zones.
**Blocker:** ✅ Safe — JSON only.
**Implementation:** Add to `abilities.json`. `map_reveal` passive already exists. UI threshold display: check `passive_bonuses["faction_lore"]` in the reputation UI render path.
**Balance:** T1 cost (1 SP). Informational — no direct power, but enables better decision-making.

### Faction Expertise (T2)
**Description:** You know not just who they are, but what they respond to.
**Effect:** +5 reputation gain with all factions on positive interactions.
**Blocker:** ✅ Safe
**Implementation:** New passive key `reputation_gain_bonus`. Read in `modify_reputation()` in `state.rs` when delta is positive.
**Balance:** T2 cost (2 SP). +5 flat is meaningful early, less so late. Consider scaling.

### Faction Insider (T3)
**Description:** They have stopped treating you as an outsider. This is either an honor or a trap.
**Effect:** Access to faction-exclusive recipes and trader inventories. Hooks into `faction_required` in `crafting.rs` and `trading.rs`.
**Blocker:** ✅ Safe — `faction_required` field already exists in recipe and trader systems.
**Implementation:** Add `faction_required` to relevant recipes/traders in JSON. Check `passive_bonuses["faction_insider"]` to bypass the reputation threshold check.
**Balance:** T3 cost (3 SP). Exclusive content should be meaningful but not mandatory.

### Outcast Signs (T2)
**Description:** The Refraction Outcasts speak in marks left on stone. You have learned to read them.
**Effect:** Can communicate with Refraction Outcasts regardless of current reputation. Unlocks Outcast dialogue branches.
**Blocker:** ✅ Safe
**Implementation:** Check `passive_bonuses["outcast_signs"] > 0` in dialogue system when loading Outcast NPC dialogue options.
**Balance:** T2 cost (2 SP). Niche but valuable for Outcast-aligned builds.

### Reputation Weaving (T3)
**Description:** You have learned to give with one hand without taking with the other.
**Effect:** Gaining reputation with one faction does not bleed reputation from rival factions.
**Blocker:** ⚠️ Partial — `modify_reputation()` exists but rival faction bleed logic is not yet implemented. Define now; wire when bleed is added.
**Implementation:** Add to `abilities.json`. When rival bleed is implemented in `modify_reputation()`, check `passive_bonuses["reputation_weaving"] > 0` to skip the bleed.
**Balance:** T3 cost (3 SP). Powerful for multi-faction builds. Intentionally late-tree.

### Gossip & Rumors (T1)
**Description:** The traders know more than they sell. You have learned to ask the right questions.
**Effect:** Chance (30%) to learn enemy locations or upcoming events from NPC dialogue.
**Blocker:** ✅ Safe
**Implementation:** New passive key `gossip_chance`. Check in NPC dialogue resolution to optionally reveal nearby enemy positions or trigger a dynamic event hint.
**Balance:** T1 cost (1 SP). Proc rate 30%. Information-based — no direct combat value.

### Intelligence Network (T2)
**Description:** You no longer need to ask. The information finds you.
**Effect:** `encounter_reduction` passive — fewer random ambushes.
**Blocker:** ✅ Safe — `encounter_reduction` passive key already exists.
**Implementation:** Add to `abilities.json`. Already consumed in encounter system.
**Balance:** T2 cost (2 SP). -20% encounter rate. Significant for exploration-focused builds.

### Bartering (T1)
**Description:** Everything has a price. You have learned that the first price is never the real one.
**Effect:** `buy_price_reduction` and `sell_price_bonus` passives.
**Blocker:** ✅ Safe — both passive keys already exist and are consumed in `trading.rs`.
**Implementation:** Add to `abilities.json`. Already wired.
**Balance:** T1 cost (1 SP). -10% buy / +10% sell. Stacks with Hard Bargaining.

### Hard Bargaining (T2)
**Description:** You have stopped pretending the negotiation is friendly.
**Effect:** Larger price bonuses: -20% buy / +20% sell (cumulative with Bartering).
**Blocker:** ✅ Safe
**Implementation:** Increase passive values in `abilities.json`. Cumulative via `recalculate_passive_bonuses()`.
**Balance:** T2 cost (2 SP). Total with Bartering: -30% buy / +30% sell. Significant but not game-breaking.

### Black Market Access (T3)
**Description:** There is a market for everything. You have found the one that does not advertise.
**Effect:** Access to black market trader inventory (rare/illegal items). Hooks into `faction_required` in `traders.json`.
**Blocker:** ✅ Safe
**Implementation:** Add black market trader entries to `traders.json` with `faction_required: "black_market"`. Check `passive_bonuses["black_market_access"] > 0` to bypass the faction check.
**Balance:** T3 cost (3 SP). Black market items should be powerful but expensive.

### Trader's Tongue (T2)
**Description:** A trade is not just an exchange. It is a conversation. You have learned to listen.
**Effect:** NPCs occasionally offer side quests or rumors during trade interactions.
**Blocker:** ⚠️ Partial — NPC quest-offering during trade needs a new dialogue hook.
**Implementation:** Define in `abilities.json`. Wire in trade dialogue resolution when NPC quest-offering is implemented.
**Balance:** T2 cost (2 SP). Informational/narrative value. No direct power.

### Intimidation (T1)
**Description:** Some negotiations end before they begin.
**Effect:** `flee_bonus` passive — enemies have higher chance to flee combat.
**Blocker:** ✅ Safe — `flee_bonus` passive key already exists.
**Implementation:** Add to `abilities.json`. Already consumed in AI flee logic.
**Balance:** T1 cost (1 SP). +15% enemy flee chance. More useful against weak enemies.

### Berate (T2)
**Description:** Words can wound. You have learned the ones that do.
**Effect:** Active skill — apply `demoralized` status to one enemy (reduced accuracy and damage for 3 turns).
**Blocker:** ✅ Safe — hooks into existing status system.
**Implementation:** Add `demoralized` to `data/effects.json`. Add active handler in `skills_menu.rs` following `field_medicine` pattern.
**Balance:** T2 cost (2 SP). AP cost: 1. `demoralized`: -15% accuracy, -10% damage. 3-turn duration.

### Inspiring Presence (T1)
**Description:** You carry yourself like someone who has survived worse. Others notice.
**Effect:** Nearby allied NPCs gain +10% combat effectiveness.
**Blocker:** ⚠️ Partial — NPC combat effectiveness is not fully implemented.
**Implementation:** Define in `abilities.json`. Wire when NPC combat system is more developed.
**Balance:** T1 cost (1 SP). Niche — only valuable in NPC-companion scenarios.

### Pilgrim Courtesy (T1)
**Description:** The old roads have old customs. You have learned to observe them.
**Effect:** +10 reputation with Mirror Monks on first interaction at shrines.
**Blocker:** ✅ Safe
**Implementation:** Check `passive_bonuses["pilgrim_courtesy"] > 0` in shrine interaction handler. Call `modify_reputation("mirror_monks", 10)` once per shrine.
**Balance:** T1 cost (1 SP). One-time bonus per shrine. Meaningful for Monk-aligned builds.

### Monk Rhetoric (T2)
**Description:** The Monks speak in paradox. You have learned to answer in kind.
**Effect:** Unlocks Mirror Monk exclusive dialogue branches and quest options.
**Blocker:** ✅ Safe
**Implementation:** Check `passive_bonuses["monk_rhetoric"] > 0` in dialogue system when loading Mirror Monk NPC options.
**Balance:** T2 cost (2 SP). Narrative/quest value. Unlocks unique quest lines.


---

## 4. Survival

> The steppe kills the unprepared and the unlucky in equal measure. Skill only improves your odds.

```
Survival (root)
├── Scavenging (T1)
│   └── Expert Scavenging (T2)
├── Corpse Reading (T1)
│   └── Anatomy Knowledge (T2)
│       └── Weak Point Analysis (T3)
├── Pickpocket (T1)                    [⚠️ Partial]
│   └── Master Thief (T2)             [⚠️ Partial]
├── Track & Trace (T1)
│   └── Ambush Predator (T2)
├── Dust Walking (T1)
│   └── Ghost Step (T2)
├── Biome Lore (T1)
│   └── Biome Mastery (T2)
├── Storm Reading (T1)                 [❌ Blocked]
│   └── Storm Sense (T2)             [❌ Blocked]
└── Salt Hermit Ways (T1)             [❌ Blocked]
```

### Scavenging (T1)
**Description:** The steppe's dead leave more behind than their bones.
**Effect:** +15% items found in chests and containers. `resource_efficiency` passive.
**Blocker:** ✅ Safe
**Implementation:** `resource_efficiency` passive in `abilities.json`. Consume in chest/container loot resolution.
**Balance:** T1 cost (1 SP). Broad utility. Does not affect enemy drops.

### Expert Scavenging (T2)
**Description:** You have learned to look where others stop looking.
**Effect:** 20% chance to find a rare item in otherwise empty containers.
**Blocker:** ✅ Safe
**Implementation:** New passive key `expert_scavenge_chance`. Check in container loot resolution after normal loot roll fails.
**Balance:** T2 cost (2 SP). 20% proc on empty containers only. Rare items from a curated "scavenge_rare" loot table.

### Corpse Reading (T1)
**Description:** The dead are honest about what killed them. You have learned to ask.
**Effect:** On enemy kill, learn that enemy type's damage weakness. Revealed in enemy inspect UI.
**Blocker:** ✅ Safe — hooks into `EnemyKilled` event.
**Implementation:** New passive key `corpse_reading`. In `EnemyKilled` handler, add enemy type to `player.known_weaknesses: HashSet<String>`. Display in enemy inspect UI.
**Balance:** T1 cost (1 SP). Informational. Value compounds over a run as more enemy types are learned.

### Anatomy Knowledge (T2)
**Description:** Knowing where it hurts is half the battle.
**Effect:** +10% damage against enemies whose weakness is known (via Corpse Reading).
**Blocker:** ✅ Safe
**Implementation:** New passive key `known_weakness_damage_bonus`. Check `player.known_weaknesses.contains(&enemy.type_id)` in `systems/combat.rs` damage calculation.
**Balance:** T2 cost (2 SP). +10% is meaningful but requires prior Corpse Reading investment.

### Weak Point Analysis (T3)
**Description:** You no longer aim at the enemy. You aim at the gap.
**Effect:** Critical hits against known-weakness enemies ignore armor entirely.
**Blocker:** ✅ Safe
**Implementation:** New passive key `weak_point_armor_pierce`. Check in crit damage path in `systems/combat.rs`.
**Balance:** T3 cost (3 SP). Powerful but requires Corpse Reading + Anatomy Knowledge investment. Crits are not guaranteed.

### Pickpocket (T1)
**Description:** The hand is faster than the eye. You have practiced until this is true.
**Effect:** Steal items from NPC inventories during dialogue or proximity.
**Blocker:** ⚠️ Partial — needs new interaction type in NPC/dialogue system.
**Implementation:** Define in `abilities.json`. Wire when NPC interaction system supports item theft. Success chance based on `passive_bonuses["pickpocket_skill"]`.
**Balance:** T1 cost (1 SP). Failure should have reputation consequences with the relevant faction.

### Master Thief (T2)
**Description:** Equipped items are just items that haven't been stolen yet.
**Effect:** Can steal equipped items from NPCs. Higher success chance.
**Blocker:** ⚠️ Partial — depends on Pickpocket implementation.
**Implementation:** Define now. Wire with Pickpocket.
**Balance:** T2 cost (2 SP). Stealing equipped items should have higher failure risk and larger reputation penalty.

### Track & Trace (T1)
**Description:** Everything that moves leaves a mark. You have learned to read them.
**Effect:** `detection_reduction` passive. Reveal enemy patrol paths on the map.
**Blocker:** ✅ Safe — `detection_reduction` passive key already exists.
**Implementation:** Add to `abilities.json`. Patrol path reveal: check `passive_bonuses["track_and_trace"] > 0` in map render to show enemy movement indicators.
**Balance:** T1 cost (1 SP). -15% detection. Patrol reveal is informational.

### Ambush Predator (T2)
**Description:** The first strike from shadow is the only one that matters.
**Effect:** +25% damage on the first attack made from stealth.
**Blocker:** ✅ Safe
**Implementation:** New passive key `ambush_damage_bonus`. Check `player.is_stealthed` flag in `systems/combat.rs` first-attack path.
**Balance:** T2 cost (2 SP). +25% is significant. Only applies to the first attack — encourages stealth approach, not sustained stealth combat.

### Dust Walking (T1)
**Description:** Sand remembers footsteps. You have learned to leave none.
**Effect:** `detection_reduction` bonus on sand and dust tile types.
**Blocker:** ✅ Safe
**Implementation:** New passive key `dust_walk_detection_reduction`. Check tile type in `systems/movement.rs` detection calculation.
**Balance:** T1 cost (1 SP). Biome-specific — most valuable in Salt Flats and Singing Dunes.

### Ghost Step (T2)
**Description:** You move as if the ground is not there.
**Effect:** No movement sound on any tile type. Increased `detection_reduction` passive.
**Blocker:** ✅ Safe
**Implementation:** Increase `detection_reduction` value in `abilities.json`. Sound suppression: check `passive_bonuses["ghost_step"] > 0` in movement sound trigger.
**Balance:** T2 cost (2 SP). Cumulative with Dust Walking. Strong for stealth builds.

### Biome Lore (T1)
**Description:** Each biome has its rules. You have learned to read them before they kill you.
**Effect:** Reveal biome-specific hazards and resource nodes on the map. `map_reveal` passive.
**Blocker:** ✅ Safe — `map_reveal` passive key already exists.
**Implementation:** Add to `abilities.json`. Biome hazard reveal: check `passive_bonuses["biome_lore"] > 0` in map generation to mark hazard tiles as revealed.
**Balance:** T1 cost (1 SP). Informational. Pairs well with Biome Mastery.

### Biome Mastery (T2)
**Description:** You do not merely survive the biome. You use it.
**Effect:** `encounter_reduction` in known biomes. Small stat bonus (armor, accuracy) in the player's most-visited biome.
**Blocker:** ✅ Safe — `encounter_reduction` already exists.
**Implementation:** Track `player.biome_visit_counts: HashMap<String, u32>`. Apply bonus in the most-visited biome. Check in `systems/combat.rs` and encounter system.
**Balance:** T2 cost (2 SP). Rewards players who specialize in one region.

### Storm Reading (T1)
**Description:** The glass shifts before the storm arrives. You have learned to watch for it.
**Effect:** Advance warning of incoming storms. Reduced storm damage.
**Blocker:** ❌ Blocked — Storm System Rework (Roadmap Tier 2, item 10). Storm types, forecast system, and intensity scaling will all change.
**Implementation:** Define in `abilities.json` with `blocked: true`. Wire after storm rework.
**Balance:** T1 cost (1 SP). Significant survival value once storms are more complex.

### Storm Sense (T2)
**Description:** You feel the storm before the glass does.
**Effect:** Predict storm type and intensity. Immunity to minor storm effects.
**Blocker:** ❌ Blocked — depends on Storm Reading and storm rework.
**Balance:** T2 cost (2 SP).

### Salt Hermit Ways (T1)
**Description:** You have learned to need less. The steppe respects this.
**Effect:** Reduced hunger and thirst consumption. Bonus to solo survival checks.
**Blocker:** ❌ Blocked — hunger/thirst system does not exist (0 matches in codebase).
**Implementation:** Define in `abilities.json` with `blocked: true`. Wire when hunger/thirst is implemented.
**Balance:** T1 cost (1 SP). Foundational survival skill once the system exists.


---

## 5. Medical

> The body is not sacred here. It is a tool. You have learned to maintain it.

```
Medical (root)
├── Basic Medical Practice (T1)
│   ├── Wound Packing (T2)
│   ├── Glass Extraction (T2)
│   └── Triage (T2)
│       └── Intermediary Medical Practice (T3)
│           ├── Wound Sutures (T3)
│           ├── Medicine Understanding (T3)
│           │   └── → unlocks Medicine Synthesis (Salt Alchemy T1)
│           └── Advanced Medical Practice (T4)
│               ├── Self-Operating (T4)    [⚠️ Partial]
│               └── Field Surgery (T4)    [⚠️ Partial]
└── Stimulant Use (T1)
    └── Stimulant Synthesis (T2)          [requires: Brine Distillation]
```

### Basic Medical Practice (T1)
**Description:** You know which wounds kill quickly and which kill slowly. This is the beginning of medicine.
**Effect:** `healing_bonus` +10%. Unlocks basic medical item use (bandages, antiseptic).
**Blocker:** ✅ Safe — `healing_bonus` passive key already exists.
**Implementation:** Add to `abilities.json`. `healing_bonus` already consumed in healing paths.
**Balance:** T1 cost (1 SP). Gateway to the entire Medical tree.

### Wound Packing (T2)
**Description:** Salt and cloth, applied with pressure. It is not elegant. It works.
**Effect:** Active skill — instantly remove `bleeding` status effect. No materials required.
**Blocker:** ✅ Safe — hooks into existing status system.
**Implementation:** Add `bleeding` to `data/effects.json` if not present. Add active handler in `skills_menu.rs`: remove `bleeding` from `player.status_effects`.
**Balance:** T2 cost (2 SP). AP cost: 1. Situational but potentially life-saving. No cooldown — balanced by AP cost.

### Glass Extraction (T2)
**Description:** Glass in the wound is not a wound. It is a countdown.
**Effect:** Active skill — remove `glass_shards` debuff without taking the normal removal damage.
**Blocker:** ✅ Safe
**Implementation:** Add `glass_shards` status to `data/effects.json` if not present. Active handler in `skills_menu.rs`: remove status without triggering damage.
**Balance:** T2 cost (2 SP). AP cost: 1. Removes a debuff that would otherwise require taking damage to clear.

### Triage (T2)
**Description:** You can read a body the way others read a map.
**Effect:** See enemy HP values when inspecting enemies.
**Blocker:** ✅ Safe
**Implementation:** New passive key `triage_active`. Check in enemy inspect UI render: if `passive_bonuses["triage_active"] > 0`, display `enemy.hp / enemy.max_hp`.
**Balance:** T2 cost (2 SP). Informational. Significant tactical value.

### Intermediary Medical Practice (T3)
**Description:** You have moved past stopping the bleeding. You have begun to understand why it started.
**Effect:** `healing_bonus` +20% cumulative. Unlocks T2 medical recipes.
**Blocker:** ✅ Safe
**Implementation:** Increase `healing_bonus` value in `abilities.json`. Add T2 medical recipes to `recipes.json` with `skill_required: 3`.
**Balance:** T3 cost (3 SP). Significant healing investment. Total `healing_bonus` with Basic: +30%.

### Wound Sutures (T3)
**Description:** Closed properly, a wound becomes a scar. Scars do not reopen.
**Effect:** Active skill — apply `regenerating` status (heal 30% max HP over 3 turns).
**Blocker:** ✅ Safe — hooks into status system.
**Implementation:** Add `regenerating` to `data/effects.json` with `heal_per_turn` value. Active handler in `skills_menu.rs`: apply status. Requires suture materials (1 use).
**Balance:** T3 cost (3 SP). AP cost: 2. Requires materials. Total heal: 30% max HP over 3 turns. Powerful but not instant.

### Medicine Understanding (T3)
**Description:** You can read the label now. More importantly, you understand what it means.
**Effect:** Reveals full stats for medicine items on inspect. Cross-tree unlock: enables Medicine Synthesis in Salt Alchemy tree.
**Blocker:** ✅ Safe
**Implementation:** New passive key `medicine_understanding`. Check in item inspect UI to show full medicine stats. Used as prerequisite check in `SkillsState::can_learn()` for Medicine Synthesis.
**Balance:** T3 cost (3 SP). Primarily a cross-tree enabler. The stat reveal is a bonus.

### Advanced Medical Practice (T4)
**Description:** You have stopped thinking of the body as something that breaks. You think of it as something that can be rebuilt.
**Effect:** `healing_bonus` +35% cumulative. Unlocks T3 medical recipes.
**Blocker:** ✅ Safe
**Implementation:** Increase `healing_bonus` in `abilities.json`. Total with all tiers: +65%.
**Balance:** T4 cost (4 SP). Significant investment. Healing-focused builds become very durable.

### Self-Operating (T4)
**Description:** The surgeon who operates on themselves has a fool for a patient. You have decided to be that fool.
**Effect:** Can perform surgery on self — remove unwanted adaptations, treat severe wounds that would otherwise be permanent.
**Blocker:** ⚠️ Partial — adaptation removal not yet implemented. Severe wound system not yet implemented.
**Implementation:** Define in `abilities.json`. Wire when adaptation removal and severe wound systems are implemented.
**Balance:** T4 cost (4 SP). Transformative — enables build correction mid-run.

### Field Surgery (T4)
**Description:** You can stabilize someone who should be dead. Whether they stay alive is another matter.
**Effect:** Stabilize dying allied NPCs. Prevents NPC permadeath in combat.
**Blocker:** ⚠️ Partial — NPC permadeath system not fully implemented.
**Implementation:** Define in `abilities.json`. Wire when NPC companion system is more developed.
**Balance:** T4 cost (4 SP). High value for companion-focused builds.

### Stimulant Use (T1)
**Description:** The stims work. The side effects are someone else's problem.
**Effect:** Use stimulant items without negative side effects (removes the `stimulant_crash` debuff on expiry).
**Blocker:** ✅ Safe — hooks into item use and status system.
**Implementation:** New passive key `stimulant_tolerance`. In stimulant item expiry handler, check `passive_bonuses["stimulant_tolerance"] > 0` before applying `stimulant_crash`.
**Balance:** T1 cost (1 SP). Enables aggressive stimulant use without the downside.

### Stimulant Synthesis (T2)
**Description:** You no longer need to find them. You make them.
**Effect:** Unlocks stimulant crafting recipes.
**Prerequisite:** Brine Distillation (Salt Alchemy T1) — cross-tree.
**Blocker:** ✅ Safe
**Implementation:** Add stimulant recipes to `recipes.json` with `skill_required: 2`. Cross-tree prerequisite check in `can_learn()`.
**Balance:** T2 cost (2 SP). Crafted stims should be slightly weaker than found stims to preserve loot value.


---

## 6. Melee Combat

> Close enough to see their eyes. Close enough to be seen. The steppe fighters say this is where the real work begins.

```
Melee Combat (root)
├── Glass Fighting (T1)
│   ├── Seam Breaker (T2)
│   │   └── Fracture Strike (T3)
│   └── Counter-Refraction (T2)
├── Angle Reading (T1)
│   └── Void Strike (T2)
├── Vortex Footwork (T1)
│   └── Evasive Maneuvers (T2)         [⚠️ Partial]
│       └── Phantom Step (T3)
├── Salt Flurry (T1)
│   └── Brine Splash (T2)
└── Storm Stance (T1)                  [❌ Blocked]
```

### Glass Fighting (T1)
**Description:** Glass-armored enemies are not invulnerable. They are just armored in the wrong direction.
**Effect:** +10% melee damage against glass/crystal enemy types.
**Blocker:** ✅ Safe — enemy type IDs are stable.
**Implementation:** New passive key `glass_enemy_damage_bonus`. In `systems/combat.rs` melee damage calc, check if `enemy.type_tags.contains("glass")` and apply bonus.
**Balance:** T1 cost (1 SP). Situational but the most common enemy type in the steppe. Intentionally strong.

### Seam Breaker (T2)
**Description:** Every armor has a seam. You have learned to find it before the fight ends.
**Effect:** Melee attacks against armored enemies have a 25% chance to reduce their armor by 1 for the remainder of combat.
**Blocker:** ✅ Safe
**Implementation:** New passive key `armor_shred_chance`. Check in `systems/combat.rs` hit resolution. Track `enemy.armor_shred_stacks` (temporary combat field).
**Balance:** T2 cost (2 SP). 25% proc. Stacks up to 3 times. Significant against heavily armored enemies.

### Fracture Strike (T3)
**Description:** When a glass enemy dies, it does not simply fall. It teaches everyone nearby a lesson.
**Effect:** On killing a glass/crystal enemy, they shatter — dealing 50% of their max HP as AoE damage to adjacent enemies.
**Blocker:** ✅ Safe — hooks into `EnemyKilled` event.
**Implementation:** New passive key `fracture_strike`. In `EnemyKilled` handler, check enemy type tags and `passive_bonuses["fracture_strike"]`. Apply AoE damage to adjacent enemies.
**Balance:** T3 cost (3 SP). Powerful in dense glass-enemy encounters. No effect against non-glass enemies.

### Counter-Refraction (T2)
**Description:** Enemies who have been changed by the glass fight differently. You have learned how.
**Effect:** -20% damage taken from enemies with refraction adaptations.
**Blocker:** ✅ Safe — reads `enemy.adaptations.len()` (stable).
**Implementation:** New passive key `refraction_damage_reduction`. In `systems/combat.rs` incoming damage calc, check if attacker has adaptations.
**Balance:** T2 cost (2 SP). Niche but very strong in late-game where adapted enemies are common.

### Angle Reading (T1)
**Description:** You have learned to watch where the weapon is going, not where it is.
**Effect:** +8% melee accuracy. `accuracy_bonus` passive.
**Blocker:** ✅ Safe — `accuracy_bonus` passive key already exists.
**Implementation:** Add to `abilities.json`. Already consumed in combat.
**Balance:** T1 cost (1 SP). Flat accuracy — reliable, not flashy.

### Void Strike (T2)
**Description:** Some wounds do not bleed. They simply... open.
**Effect:** Melee attacks have a 20% chance to apply `void_touched` status (reduces enemy max HP by 10% for 5 turns).
**Blocker:** ✅ Safe — hooks into status system.
**Implementation:** Add `void_touched` to `data/effects.json`. New passive key `void_strike_chance`. Check in `systems/combat.rs` hit resolution.
**Balance:** T2 cost (2 SP). 20% proc. `void_touched` is a debuff, not direct damage — synergizes with sustained combat.

### Vortex Footwork (T1)
**Description:** You do not stand where you were. This is the first lesson of the steppe fighter.
**Effect:** +10% dodge chance.
**Blocker:** ✅ Safe
**Implementation:** New passive key `dodge_bonus`. Check in `systems/combat.rs` incoming attack resolution before damage is applied.
**Balance:** T1 cost (1 SP). +10% dodge. Stacks with Evasive Maneuvers.

### Evasive Maneuvers (T2)
**Description:** A dodge is not a retreat. It is a repositioning.
**Effect:** Successful dodge grants a free counter-attack at 50% damage.
**Blocker:** ⚠️ Partial — counter-attack on dodge needs a new hook in the combat resolution flow.
**Implementation:** Define in `abilities.json`. Wire when dodge resolution is refactored to support callbacks.
**Balance:** T2 cost (2 SP). Counter-attack at 50% damage. Powerful but requires the dodge to proc first.

### Phantom Step (T3)
**Description:** Movement is detection. You have learned to move without being.
**Effect:** Movement does not trigger enemy detection checks.
**Blocker:** ✅ Safe
**Implementation:** New passive key `phantom_step`. Check in `systems/movement.rs` detection trigger: skip if `passive_bonuses["phantom_step"] > 0`.
**Balance:** T3 cost (3 SP). Powerful for stealth builds. Does not affect detection from line-of-sight — only movement-triggered detection.

### Salt Flurry (T1)
**Description:** Two strikes where one was expected. The steppe teaches economy; you have learned excess.
**Effect:** Active skill — attack twice in one action at -20% damage each.
**Blocker:** ✅ Safe — hooks into combat action system.
**Implementation:** Add active handler in `skills_menu.rs`. Call melee attack twice with damage multiplier. AP cost: 2 (same as two normal attacks but in one action).
**Balance:** T1 cost (1 SP). AP cost: 2. Net damage: 160% of one attack. Slight efficiency gain — value is in applying on-hit effects twice.

### Brine Splash (T2)
**Description:** The brine does not kill. It prepares.
**Effect:** Melee attacks apply `wet` status (target takes +15% damage from all sources for 2 turns).
**Blocker:** ✅ Safe — hooks into status system.
**Implementation:** Add `wet` to `data/effects.json` with `incoming_damage_multiplier: 1.15`. New passive key `brine_splash_on_hit`. Check in `systems/combat.rs` hit resolution.
**Balance:** T2 cost (2 SP). `wet` is a universal damage amplifier — synergizes with any damage source. 2-turn duration keeps it from being permanent.

### Storm Stance (T1)
**Description:** The storm fighter does not resist the storm. They become it.
**Effect:** During storms, gain bonus melee damage and reduced incoming damage.
**Blocker:** ❌ Blocked — Storm System Rework (Roadmap Tier 2, item 10). Storm state API will change.
**Implementation:** Define in `abilities.json` with `blocked: true`. Wire after storm rework.
**Balance:** T1 cost (1 SP). Significant value once storms are more complex and frequent.


---

## 7. Ranged Combat

> Distance is not safety. It is time. You have learned to use it.

```
Ranged Combat (root)
├── Draw a Bead (T1)
│   ├── Sniper's Eye (T2)
│   │   └── Trick Shot (T3)            [⚠️ Partial]
│   └── Aimed Shot (T2)
├── Suppressive Shards (T1)
│   └── Glass Barrage (T2)             [⚠️ Partial]
│       └── Shard Storm (T3)           [❌ Blocked]
├── Brine Volley (T1)
│   └── Corrosive Rounds (T2)
├── Ammo Conservation (T1)
│   └── Scavenger's Arsenal (T2)
└── Void Aim (T1)
    └── Void Barrage (T2)              [⚠️ Partial]
```

### Draw a Bead (T1)
**Description:** You have learned to breathe before you shoot. The target does not move while you breathe.
**Effect:** +10% ranged accuracy. `accuracy_bonus` passive (ranged).
**Blocker:** ✅ Safe — `accuracy_bonus` passive key already exists.
**Implementation:** Add to `abilities.json` targeting ranged accuracy. Already consumed in `systems/combat.rs`.
**Balance:** T1 cost (1 SP). Flat accuracy. Reliable foundation for ranged builds.

### Sniper's Eye (T2)
**Description:** The farther the target, the more time you have to aim. You have learned to use that time.
**Effect:** +20% ranged damage when attacking at maximum weapon range.
**Blocker:** ✅ Safe
**Implementation:** New passive key `max_range_damage_bonus`. In `systems/combat.rs` ranged damage calc, check if attack distance equals weapon max range.
**Balance:** T2 cost (2 SP). +20% at max range only. Rewards positioning. Does not apply at close range.

### Trick Shot (T3)
**Description:** The glass wall is not an obstacle. It is a second shooter.
**Effect:** Ranged attacks can ricochet off glass walls, hitting targets behind cover.
**Blocker:** ⚠️ Partial — needs new projectile path logic with reflection calculation.
**Implementation:** Define in `abilities.json`. Wire when projectile system supports reflection. Use existing glass tile detection in `map.rs`.
**Balance:** T3 cost (3 SP). Situational but powerful in glass-heavy environments. Ricochet damage: 60% of original.

### Aimed Shot (T2)
**Description:** You have learned that speed and accuracy are not the same thing.
**Effect:** Active skill — spend 2 AP for a guaranteed hit dealing +50% damage.
**Blocker:** ✅ Safe — hooks into combat action system.
**Implementation:** Add active handler in `skills_menu.rs`. Set `combat_always_hit: true` for this attack and apply damage multiplier. AP cost: 2.
**Balance:** T2 cost (2 SP). AP cost: 2. Guaranteed hit + 50% damage. High value against high-evasion targets.

### Suppressive Shards (T1)
**Description:** You do not need to hit them. You need them to stop moving.
**Effect:** Ranged attacks apply `suppressed` status (-20% accuracy for 2 turns).
**Blocker:** ✅ Safe — hooks into status system.
**Implementation:** Add `suppressed` to `data/effects.json`. New passive key `suppressive_on_hit`. Check in `systems/combat.rs` ranged hit resolution.
**Balance:** T1 cost (1 SP). `suppressed` is a debuff, not damage. 2-turn duration. Proc on every hit — balanced by the debuff being accuracy-only.

### Glass Barrage (T2)
**Description:** Not one shard. All of them.
**Effect:** Active skill — fire a cone of shards hitting all enemies in a 3-tile arc.
**Blocker:** ⚠️ Partial — needs new cone/AoE attack pattern in combat system.
**Implementation:** Define in `abilities.json`. Wire when AoE attack patterns are implemented. AP cost: 3.
**Balance:** T2 cost (2 SP). AP cost: 3. Each shard deals 60% normal damage. Powerful in tight corridors.

### Shard Storm (T3)
**Description:** You have learned to throw the storm itself.
**Effect:** Massive AoE shard attack that interacts with storm conditions.
**Blocker:** ❌ Blocked — depends on Glass Barrage (⚠️) and Storm System Rework.
**Implementation:** Define now. Wire after both dependencies are resolved.
**Balance:** T3 cost (3 SP). Signature ability for storm+ranged builds.

### Brine Volley (T1)
**Description:** Brine in the eyes. Brine in the wounds. The steppe provides.
**Effect:** Ranged attacks apply `wet` status (target takes +15% damage from all sources for 2 turns).
**Blocker:** ✅ Safe — hooks into status system. `wet` defined in Brine Splash (Melee).
**Implementation:** New passive key `brine_volley_on_hit`. Check in `systems/combat.rs` ranged hit resolution. Apply `wet` status.
**Balance:** T1 cost (1 SP). Same `wet` debuff as Brine Splash but on ranged attacks. Synergizes with any damage source.

### Corrosive Rounds (T2)
**Description:** Wet targets corrode. You have learned to make them wet first.
**Effect:** +25% ranged damage against `wet` enemies.
**Blocker:** ✅ Safe
**Implementation:** New passive key `wet_enemy_damage_bonus`. Check `enemy.has_status("wet")` in `systems/combat.rs` ranged damage calc.
**Balance:** T2 cost (2 SP). +25% vs wet only. Strong synergy with Brine Volley — intentional combo.

### Ammo Conservation (T1)
**Description:** Every shot that doesn't leave the weapon is a shot you still have.
**Effect:** 20% chance not to consume ammo on a ranged attack.
**Blocker:** ✅ Safe
**Implementation:** New passive key `ammo_conservation_chance`. Check in `systems/combat.rs` ranged attack ammo consumption path.
**Balance:** T1 cost (1 SP). 20% proc. Significant over a long run. Does not apply to thrown weapons.

### Scavenger's Arsenal (T2)
**Description:** You find ammunition where others find nothing.
**Effect:** Ammo appears more frequently in loot containers and enemy drops.
**Blocker:** ✅ Safe — hooks into loot tables.
**Implementation:** New passive key `ammo_loot_bonus`. Check in loot resolution to increase ammo drop weight.
**Balance:** T2 cost (2 SP). Reduces ammo scarcity for ranged builds. Does not make ammo infinite.

### Void Aim (T1)
**Description:** You have learned to aim at what is not quite there.
**Effect:** +10% ranged damage. `damage_bonus` passive (ranged).
**Blocker:** ✅ Safe — `damage_bonus` passive key already exists.
**Implementation:** Add to `abilities.json` targeting ranged damage. Already consumed in `systems/combat.rs`.
**Balance:** T1 cost (1 SP). Flat damage. Reliable foundation.

### Void Barrage (T2)
**Description:** The shot does not stop at the first target.
**Effect:** Active skill — charged ranged attack that pierces through enemies in a line.
**Blocker:** ⚠️ Partial — needs pierce/line projectile logic in combat system.
**Implementation:** Define in `abilities.json`. Wire when projectile pierce is implemented. AP cost: 3.
**Balance:** T2 cost (2 SP). AP cost: 3. Hits all enemies in a line at 80% damage each. Powerful in corridors.


---

## Global Implementation Notes

### Adding a pure passive skill
1. Add entry to `data/abilities.json` with `passive_effects` array
2. `recalculate_passive_bonuses()` picks it up automatically on skill-up
3. If the passive key is new, add one consumption point in the relevant system (combat.rs, movement.rs, etc.)

### Adding an active skill
1. Add entry to `data/abilities.json` with `active: true`
2. Add handler in `src/ui/skills_menu.rs` following the `field_medicine` pattern
3. Active skills should have an AP cost enforced in the handler

### Adding a new recipe
1. Add to `data/recipes.json` with `skill_required`, `station_required`, `faction_required` as needed
2. No Rust changes needed

### Adding a new crafting station
1. Add to `data/interactables.json`
2. Add one line to `spawn_crafting_stations()` in `state.rs`
3. `available_stations()` picks it up automatically via proximity detection

### Cross-tree prerequisites
Add a `requires_skill: "skill_id"` field to the skill definition in `abilities.json`. Check in `SkillsState::can_learn()` before allowing the skill to be learned.

### Blocked skills
Define in `abilities.json` with a `blocked: true` flag. The UI should display these as greyed out with a lore-appropriate reason (e.g. "The storm system is not yet understood"). This keeps the tree complete and visible without wiring broken integrations.

---

## Global Balancing Notes

| Tier | SP Cost | Bonus Range | Mechanic Type |
|---|---|---|---|
| T1 | 1–2 | 10–15% flat | Passive or simple active |
| T2 | 2–3 | 20–25% flat or new mechanic | Passive + proc, or active with AP cost |
| T3 | 3–4 | 30%+ or powerful new mechanic | Active with significant AP cost, or strong passive |
| T4 | 4–5 | Transformative | Rare, build-defining |

- Active skills should cost AP to prevent spam. T1 actives: 1 AP. T2: 2 AP. T3+: 3 AP.
- Proc-based passives (armor shred, fracture strike, etc.): 15–25% proc rate at T2, 30–40% at T3.
- Cross-tree prerequisites create interesting build decisions. Keep them to 1–2 per skill maximum.
- Blocked skills should be visible in the UI — players should be able to plan for them.
- Synergies (e.g. Brine Volley + Corrosive Rounds, Corpse Reading + Anatomy Knowledge) are intentional. They should be discoverable, not mandatory.
- No skill should be strictly mandatory for any playstyle. Every tree should have multiple viable paths.

---

## Related Documents

- `docs/development/SKILL_SYSTEM_IMPLEMENTATION_PLAN.md` — implementation phases and coding patterns
- `docs/design/SKILL_CATALOG_SUGGESTIONS.md` — original brainstorm catalog
- `src/game/skills.rs` — `SkillsState`, `PassiveEffect`, `recalculate_passive_bonuses()`
- `data/abilities.json` — skill and ability definitions
- `data/recipes.json` — crafting recipes
