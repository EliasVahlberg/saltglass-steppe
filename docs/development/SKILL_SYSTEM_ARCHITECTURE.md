---
status: current
last_verified: 2026-04-04
commit: e0d1fe7
---

# Skill System Architecture

> Last updated: 2026-03-07

## Executive Summary

The current skill system is structurally sound. `passive_bonuses: HashMap<String, f32>` is a good backing store; `recalculate_passive_bonuses()` works correctly; `SkillDef` already supports prerequisites. The system does not need a rewrite.

Four targeted changes prevent the technical debt explosion that would otherwise occur as the skill tree grows to 50+ skills:

1. **Typed accessors** on `SkillsState` — replace raw string lookups in consuming systems
2. **`SkillSystem`** implementing the `System` trait — handles event-triggered skill effects
3. **Effect-type dispatch** for active skills — replaces hardcoded match arms in `skills_menu.rs`
4. **Enemy type tags** in JSON — enables skills that target specific enemy categories

Everything else stays as-is. No big refactors.

---

## Current State Assessment

### What's working well

| Component | Status | Notes |
|---|---|---|
| `passive_bonuses: HashMap<String, f32>` | ✅ Keep | Clean hub-and-spoke model |
| `recalculate_passive_bonuses()` | ✅ Keep | Correct, fast enough for 50+ skills |
| `SkillDef.prerequisites` | ✅ Keep | Already supports cross-tree prereqs |
| `can_upgrade_skill()` | ✅ Keep | Prerequisite checking works |
| `crafting.rs` recipe system | ✅ Keep | `skill_required`, `station_required`, `faction_required` all wired |
| `StatusEffectSystem` | ✅ Keep | Clean event-driven design |
| `trading.rs` passive keys | ✅ Keep | `buy_price_reduction`, `sell_price_bonus` already consumed |
| `available_stations()` | ✅ Keep | Proximity-based, supports new station types via JSON |

### What's fragile

| Problem | Location | Risk |
|---|---|---|
| Raw string passive bonus lookups | `systems/combat.rs` lines 177, 178, 321, 322 | Silent failure on typo; no compile-time safety |
| Hardcoded active skill dispatch | `ui/skills_menu.rs` | Every new active skill requires a new match arm |
| No event-triggered skill hooks | — | Skills like Fracture Strike, Corpse Reading have nowhere to live |
| `SkillCategory` enum has 5 entries | `skills.rs` | Doesn't match the 7-tree design |
| No enemy type tags | `enemies/*.json` | Skills targeting glass/crystal enemies can't check type |
| No `blocked` flag on `SkillDef` | `skills.rs` | Can't mark storm/hunger-dependent skills as coming-soon |
| `AbilitiesFile` parses skills and abilities from same JSON | `skills.rs` | Fine now, but conflates two different concepts |

---

## The Passive Bonus Problem

### Current pattern (fragile)

```rust
// systems/combat.rs — 4 raw string lookups
let accuracy_bonus = state.player.skills.passive_bonuses
    .get("melee_accuracy_bonus").copied().unwrap_or(0.0);
let damage_bonus = state.player.skills.passive_bonuses
    .get("melee_damage_bonus").copied().unwrap_or(0.0);
```

As skills expand, every new system that consumes a passive bonus will add more raw string lookups scattered across the codebase. A typo silently returns 0.0 — no error, no warning.

### Fix: typed accessors on `SkillsState`

Add a thin accessor layer. The HashMap stays unchanged; the accessors are just named wrappers:

```rust
impl SkillsState {
    pub fn melee_damage_bonus(&self) -> f32 {
        self.passive_bonuses.get("melee_damage_bonus").copied().unwrap_or(0.0)
    }
    pub fn ranged_damage_bonus(&self) -> f32 {
        self.passive_bonuses.get("ranged_damage_bonus").copied().unwrap_or(0.0)
    }
    pub fn melee_accuracy_bonus(&self) -> f32 {
        self.passive_bonuses.get("melee_accuracy_bonus").copied().unwrap_or(0.0)
    }
    pub fn ranged_accuracy_bonus(&self) -> f32 {
        self.passive_bonuses.get("ranged_accuracy_bonus").copied().unwrap_or(0.0)
    }
    // Add one accessor per passive key as skills are implemented
    pub fn dodge_bonus(&self) -> f32 { ... }
    pub fn glass_enemy_damage_bonus(&self) -> f32 { ... }
    // etc.
}
```

**Rule**: no system outside `skills.rs` should call `.passive_bonuses.get(...)` directly. All consumption goes through typed accessors.

**Why not an enum?** An enum for effect types would require recompilation to add new skill effects. The String-keyed HashMap stays data-driven; the accessors provide the type safety layer without losing flexibility.

### Passive bonus consumption map

This is the authoritative list of where each passive key is (or will be) consumed. Add to this as skills are implemented.

| Passive Key | Consumed In | Status |
|---|---|---|
| `melee_damage_bonus` | `systems/combat.rs` | ✅ Wired |
| `ranged_damage_bonus` | `systems/combat.rs` | ✅ Wired |
| `melee_accuracy_bonus` | `systems/combat.rs` | ✅ Wired |
| `ranged_accuracy_bonus` | `systems/combat.rs` | ✅ Wired |
| `healing_bonus` | item use path | ✅ Wired |
| `buy_price_reduction` | `trading.rs` | ✅ Wired |
| `sell_price_bonus` | `trading.rs` | ✅ Wired |
| `detection_reduction` | `systems/movement.rs` | ✅ Wired |
| `encounter_reduction` | `encounter.rs` | ✅ Wired |
| `flee_bonus` | AI flee logic | ✅ Wired |
| `map_reveal` | map render | ✅ Wired |
| `craft_success` | `crafting.rs` | ✅ Wired |
| `craft_quality` | `crafting.rs` | ✅ Wired |
| `resource_efficiency` | `crafting.rs`, loot | ✅ Wired |
| `armor_bonus` | `systems/combat.rs` incoming | ✅ Wired |
| `dodge_bonus` | `systems/combat.rs` incoming | 🔲 Add when Vortex Footwork implemented |
| `glass_enemy_damage_bonus` | `systems/combat.rs` melee | 🔲 Add when Glass Fighting implemented |
| `armor_shred_chance` | `systems/combat.rs` melee | 🔲 Add when Seam Breaker implemented |
| `known_weakness_damage_bonus` | `systems/combat.rs` | 🔲 Add when Anatomy Knowledge implemented |
| `ambush_damage_bonus` | `systems/combat.rs` | 🔲 Add when Ambush Predator implemented |
| `max_range_damage_bonus` | `systems/combat.rs` ranged | 🔲 Add when Sniper's Eye implemented |
| `wet_enemy_damage_bonus` | `systems/combat.rs` | 🔲 Add when Corrosive Rounds implemented |
| `ammo_conservation_chance` | `systems/combat.rs` ranged | 🔲 Add when Ammo Conservation implemented |
| `reputation_gain_bonus` | `state.rs::modify_reputation()` | 🔲 Add when Faction Expertise implemented |
| `expert_scavenge_chance` | loot resolution | 🔲 Add when Expert Scavenging implemented |
| `triage_active` | enemy inspect UI | 🔲 Add when Triage implemented |
| `stimulant_tolerance` | item use / status expiry | 🔲 Add when Stimulant Use implemented |
| `ammo_loot_bonus` | loot resolution | 🔲 Add when Scavenger's Arsenal implemented |
| `dust_walk_detection_reduction` | `systems/movement.rs` | 🔲 Add when Dust Walking implemented |
| `phantom_step` | `systems/movement.rs` | 🔲 Add when Phantom Step implemented |
| `gossip_chance` | NPC dialogue | 🔲 Add when Gossip & Rumors implemented |
| `outcast_signs` | dialogue system | 🔲 Add when Outcast Signs implemented |
| `monk_rhetoric` | dialogue system | 🔲 Add when Monk Rhetoric implemented |
| `faction_lore` | reputation UI | 🔲 Add when Faction Lore implemented |
| `faction_insider` | crafting/trading faction check | 🔲 Add when Faction Insider implemented |
| `black_market_access` | trading faction check | 🔲 Add when Black Market Access implemented |
| `masterwork_mod_chance` | mod application | 🔲 Add when Masterwork Modding implemented |
| `medicine_understanding` | item inspect UI | 🔲 Add when Medicine Understanding implemented |
| `brine_splash_on_hit` | `systems/combat.rs` melee | 🔲 Add when Brine Splash implemented |
| `suppressive_on_hit` | `systems/combat.rs` ranged | 🔲 Add when Suppressive Shards implemented |
| `brine_volley_on_hit` | `systems/combat.rs` ranged | 🔲 Add when Brine Volley implemented |
| `void_strike_chance` | `systems/combat.rs` melee | 🔲 Add when Void Strike implemented |
| `reputation_weaving` | `state.rs::modify_reputation()` | 🔲 Wire when rival bleed implemented |
| `item_duration_bonus` | item use / status application | 🔲 Add when Salt Preservation implemented |

**Rule**: when implementing a skill, add its passive key to this table and add the consumption point in the same PR. Never add a passive key to `abilities.json` without a corresponding consumption point.


---

## The Active Skill Problem

### Current pattern (hardcoded)

```rust
// ui/skills_menu.rs — grows unboundedly with each new active skill
match skill_id {
    "field_medicine" => {
        let heal = (game_state.player.max_hp / 4).max(5);
        game_state.player.hp = (game_state.player.hp + heal).min(game_state.player.max_hp);
    }
    // Every new active skill adds another arm here
}
```

With 20+ active skills, this becomes a 200-line match statement that's hard to test and impossible to make data-driven.

### Fix: effect-type dispatch

Active skills define their `effect_type` in `abilities.json`. The dispatch matches on `effect_type`, not `skill_id`. Adding a new active skill with an existing effect type requires **zero Rust changes**.

```rust
// abilities.json
{
  "id": "wound_packing",
  "active": true,
  "effect_type": "remove_status",
  "effect_data": { "status_id": "bleeding" },
  "ap_cost": 1
}

{
  "id": "salt_flurry",
  "active": true,
  "effect_type": "multi_attack",
  "effect_data": { "hits": 2, "damage_multiplier": 0.8 },
  "ap_cost": 2
}
```

```rust
// ui/skills_menu.rs — dispatch on effect_type, not skill_id
match def.effect_type.as_deref() {
    Some("remove_status") => handle_remove_status(state, &def.effect_data),
    Some("multi_attack")  => handle_multi_attack(state, &def.effect_data),
    Some("instant_heal")  => handle_instant_heal(state, &def.effect_data),
    Some("apply_status")  => handle_apply_status(state, &def.effect_data),
    Some("craft_unlock")  => handle_craft_unlock(state, &def.effect_data),
    _ => { /* unknown effect type — log warning */ }
}
```

**Effect types to implement** (covers all active skills in the skill tree):

| Effect Type | Skills Using It | Handler Complexity |
|---|---|---|
| `remove_status` | Wound Packing, Glass Extraction | Low |
| `instant_heal` | (future healing actives) | Low |
| `apply_status` | Berate, Brine Splash, Suppressive Shards | Low |
| `multi_attack` | Salt Flurry | Medium |
| `aimed_shot` | Aimed Shot | Medium |
| `area_attack` | Glass Barrage, Fracture Strike (passive trigger) | High |
| `pierce_shot` | Void Barrage | High |
| `stabilize_npc` | Field Surgery | High (blocked) |

New active skills that fit an existing effect type: add to `abilities.json` only.
New effect types: add one handler function + one match arm.

---

## The Event Hook Problem

### The problem

Some skills need to fire on game events, not on player input:
- **Fracture Strike** — triggers on `EnemyKilled` if enemy has glass tag
- **Corpse Reading** — triggers on `EnemyKilled` to record enemy weakness
- **Expert Salvage** — triggers on chest/container open to add rare loot
- **Anatomy Knowledge** — modifies damage in combat based on known weaknesses

These don't fit the passive bonus model (they're not flat multipliers) and don't fit the active skill model (player doesn't trigger them). They need event hooks.

### Fix: `SkillSystem` implementing `System` trait

Add a new `src/game/systems/skill_effects.rs` that implements the `System` trait. It handles event-triggered skill effects by checking relevant passive keys.

```rust
// src/game/systems/skill_effects.rs
pub struct SkillEffectSystem;

impl System for SkillEffectSystem {
    fn update(&self, _state: &mut GameState) {}

    fn on_event(&self, state: &mut GameState, event: &GameEvent) {
        match event {
            GameEvent::EnemyKilled { enemy_id, x, y } => {
                Self::on_enemy_killed(state, enemy_id, *x, *y);
            }
            // Add more event handlers as skills require them
            _ => {}
        }
    }
}

impl SkillEffectSystem {
    fn on_enemy_killed(state: &mut GameState, enemy_id: &str, x: i32, y: i32) {
        // Corpse Reading: record enemy weakness
        if state.player.skills.passive_bonuses.get("corpse_reading").copied().unwrap_or(0.0) > 0.0 {
            if let Some(def) = get_enemy_def(enemy_id) {
                state.player.known_weaknesses.insert(def.weakness_type.clone());
            }
        }

        // Fracture Strike: AoE on glass enemy kill
        if state.player.skills.passive_bonuses.get("fracture_strike").copied().unwrap_or(0.0) > 0.0 {
            if let Some(def) = get_enemy_def(enemy_id) {
                if def.tags.contains(&"glass".to_string()) {
                    // apply AoE damage to adjacent enemies
                }
            }
        }
    }
}
```

Register in `systems/mod.rs` alongside the other systems. Called from `end_turn()` event processing loop.

**Rule**: event-triggered skill effects live in `SkillEffectSystem::on_event()`. No other system should check passive bonus keys in its event handler for skill-triggered effects.

---

## Systems That Need Changes

### 1. `src/game/skills.rs` — Low risk, immediate

**Changes:**
- Update `SkillCategory` enum to match 7-tree design:
  ```rust
  pub enum SkillCategory {
      SaltAlchemy, Crafting, Social, Survival, Medical, MeleeCombat, RangedCombat,
  }
  ```
- Add fields to `SkillDef`:
  ```rust
  pub struct SkillDef {
      // existing fields...
      #[serde(default)]
      pub tree_parent: Option<String>,   // parent skill ID in the tree
      #[serde(default)]
      pub blocked: bool,                 // true = not yet implementable
      #[serde(default)]
      pub active: bool,                  // true = has active use handler
  }
  ```
- Add typed accessor methods (see Passive Bonus section above)
- Add `known_weaknesses: HashSet<String>` to `PlayerState` for Corpse Reading

**Does not change:** `passive_bonuses` HashMap, `recalculate_passive_bonuses()`, `can_upgrade_skill()`, `upgrade_skill()`.

### 2. `src/game/systems/combat.rs` — Low risk, immediate

**Changes:**
- Replace 4 raw string lookups with typed accessor calls:
  ```rust
  // Before
  let accuracy_bonus = state.player.skills.passive_bonuses.get("melee_accuracy_bonus").copied().unwrap_or(0.0);
  // After
  let accuracy_bonus = state.player.skills.melee_accuracy_bonus();
  ```
- Add consumption points for new passive keys as skills are implemented (see consumption map above)

**Does not change:** combat logic, damage calculation, hit resolution.

### 3. `src/ui/skills_menu.rs` — Medium risk, do after typed accessors

**Changes:**
- Refactor active skill dispatch from skill-id match to effect-type dispatch (see Active Skill section above)
- Add `blocked` skill display (greyed out with reason)
- Add tree structure rendering (parent/child relationships)

**Does not change:** skill point allocation, skill level display, prerequisite checking.

### 4. `src/game/systems/skill_effects.rs` — New file, low risk

**Changes:**
- Create new file implementing `SkillEffectSystem`
- Register in `systems/mod.rs`
- Start with `EnemyKilled` handler for Corpse Reading and Fracture Strike

### 5. `data/enemies/*.json` — Data only, zero risk

**Changes:**
- Add `"tags": ["glass"]` (or `"salt"`, `"void"`, `"armored"`, etc.) to enemy definitions
- Used by Glass Fighting, Fracture Strike, Counter-Refraction

**Does not change:** any Rust code. Enemy loading already uses `serde(default)` for unknown fields.

### 6. `data/abilities.json` — Data only, zero risk

**Changes:**
- Add `tree_parent`, `blocked`, `active`, `effect_type`, `effect_data` fields to skill definitions
- Add all 90+ skills from the skill tree design
- Blocked skills included but marked `"blocked": true`

---

## Systems That Are Stable — Do Not Touch

| System | Reason |
|---|---|
| `passive_bonuses` HashMap | Good design. Typed accessors are additive, not a replacement. |
| `recalculate_passive_bonuses()` | Correct. Performance is fine for 50+ skills. |
| `crafting.rs` | Already supports `skill_required`, `station_required`, `faction_required`. |
| `StatusEffectSystem` | Clean event-driven design. Skills apply statuses via existing path. |
| `trading.rs` | Passive keys already wired. No changes needed. |
| `available_stations()` | Proximity-based detection works. New stations = JSON only. |
| `encounter.rs` | `encounter_reduction` passive key already consumed. |
| `LootSystem` | Reacts to `EnemyKilled`. New loot modifiers = new passive key + consumption point. |
| `faction_reputation` | `modify_reputation()` is stable. Rival bleed is a future addition, not a rework. |


---

## Roadmap Alignment

The skill system is a **Tier 1 blocker** for most of Tier 2+. Getting the architecture right now prevents rework later.

### How each roadmap item interacts with skills

| Roadmap Item | Skill Dependency | Risk if Skills Not Stable |
|---|---|---|
| **Item 3 — Skill Catalog** (Tier 1) | This is the skill system | — |
| **Item 6 — Spawn Table Update** (Tier 2) | Enemy tags needed for Glass Fighting, etc. | Low — tags are additive |
| **Item 7 — Tiered Mob Overhaul** (Tier 2) | Expert Salvage checks enemy tier | Medium — tier field needed on enemy |
| **Item 8 — Tiered Loot** (Tier 2) | Scavenger's Arsenal modifies loot weights | Low — passive key + loot table |
| **Item 9 — Adaptations Rework** (Tier 2) | Adaptation Tinctures reads `adaptations.len()` | Low — read-only, survives rework |
| **Item 10 — Storm System Rework** (Tier 2) | Storm Stance, Storm Reading blocked until this lands | None — blocked skills defined but not wired |
| **Item 11 — Ranged Weapon Overhaul** (Tier 3) | Ranged skills (Trick Shot, Void Barrage) need projectile system | Medium — define skills now, wire after |
| **Item 12 — Trader Overhaul** (Tier 3) | Social skills (Bartering, Black Market) hook into trading | Low — passive keys already wired |
| **Item 13 — Main Questline** (Tier 4) | Social skills affect dialogue branches | Low — passive key checks in dialogue |

### What the skill system must NOT depend on

These systems are being reworked. Skills that depend on them are marked `blocked: true` in `abilities.json` and have no integration code until the rework lands:

| Blocked Dependency | Affected Skills | Rework Item |
|---|---|---|
| Storm state API | Storm Stance, Storm Reading, Storm Sense, Shard Storm | Item 10 |
| Adaptation tree structure | Adaptation Resistance (rework-dependent variant) | Item 9 |
| Hunger/thirst system | Salt Hermit Ways | Not yet scheduled |
| Rival faction bleed | Reputation Weaving | Not yet scheduled |
| Projectile pierce/ricochet | Trick Shot, Void Barrage | Item 11 |
| NPC companion system | Field Surgery, Inspiring Presence | Not yet scheduled |
| AoE attack patterns | Glass Barrage | Item 11 (partial) |

**Rule**: a skill is only wired when its dependency is stable. Define it in JSON now (so the tree is complete and visible), but do not add integration code until the dependency lands.

---

## Implementation Order

### Phase 0 — Data (no Rust changes, do first)
1. Add `tags` to enemy definitions in `enemies/*.json`
2. Add `tree_parent`, `blocked`, `active` fields to skill definitions in `abilities.json`
3. Add all 90+ skills from `SKILL_TREE_DESIGN.md` to `abilities.json` (blocked ones included)
4. Add new recipes to `recipes.json` for crafting skills

### Phase 1 — Typed Accessors (minimal Rust, low risk)
1. Update `SkillCategory` enum in `skills.rs`
2. Add `tree_parent`, `blocked`, `active` fields to `SkillDef`
3. Add typed accessor methods to `SkillsState`
4. Replace 4 raw string lookups in `combat.rs` with accessor calls
5. Add `known_weaknesses: HashSet<String>` to `PlayerState`

### Phase 2 — SkillEffectSystem (new file, low risk)
1. Create `src/game/systems/skill_effects.rs`
2. Implement `on_enemy_killed` for Corpse Reading and Fracture Strike
3. Register in `systems/mod.rs`

### Phase 3 — Active Skill Dispatch (medium risk, do last)
1. Add `effect_type` and `effect_data` to `AbilityDef`
2. Refactor `skills_menu.rs` dispatch to effect-type based
3. Implement effect type handlers: `remove_status`, `apply_status`, `multi_attack`, `aimed_shot`
4. Update blocked skill display in skills UI

### Phase 4 — Wire Safe Skills
Wire integration for all ✅ Safe skills from `SKILL_TREE_DESIGN.md`, one passive key + consumption point at a time. Follow the consumption map table above.

---

## Architectural Rules (enforce going forward)

1. **No raw `passive_bonuses.get(...)` outside `skills.rs`** — use typed accessors
2. **No skill logic in `state.rs`** — skills live in `SkillsState`, event effects in `SkillEffectSystem`
3. **One passive key = one consumption point** — add both in the same commit
4. **Blocked skills are defined in JSON, not wired in Rust** — `blocked: true` in `abilities.json`, no integration code
5. **Active skills dispatch on `effect_type`, not `skill_id`** — new skills with existing effect types require zero Rust
6. **Event-triggered effects go through `SkillEffectSystem::on_event()`** — not scattered across other systems
7. **Enemy type checks use `def.tags`** — not hardcoded enemy ID strings

---

## Related Documents

- `docs/design/SKILL_TREE_DESIGN.md` — full skill tree with all 90+ skills, blockers, and balancing notes
- `docs/development/SKILL_SYSTEM_IMPLEMENTATION_PLAN.md` — phased implementation plan
- `docs/development/ROADMAP.md` — feature roadmap (items 3, 6–13 affected by skill system)
- `src/game/skills.rs` — `SkillsState`, `SkillDef`, `recalculate_passive_bonuses()`
- `src/game/systems/combat.rs` — current passive bonus consumption (4 raw string lookups to replace)
- `data/abilities.json` — skill and ability definitions
