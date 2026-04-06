# Domain Decomposition: Implementation Plan

> Date: 2026-04-05
> Status: **SUPERSEDED** by `DISPATCH_EXTRACTION_DESIGN.md`
> This document proposed moving apply + rules into domain modules. After review, the
> lighter-weight dispatch extraction approach was chosen instead: move only orchestration
> out of GameState, keep apply.rs centralized, keep rules/ separate.
> This document is retained for reference.
>
> Original goal: Decompose state.rs (3,680 LOC) into domain modules. state.rs becomes a thin router (~400 LOC).
> Prerequisite: Complete Priority 1 (DES effect assertions) and Priority 2 (workflow doc) first.

---

## The Problem

state.rs is 3,680 LOC — larger than pre-refactor. The VERA migration wrapped each method in Command/Effect/Rule but left everything in state.rs. The result:

1. Every new feature requires touching state.rs
2. Dispatch helpers mix rule calls, post-processing, visual effects, and legacy code
3. Domain logic (combat, movement, items) is interleaved, not separated
4. The constructor (367 LOC) contains hardcoded game content
5. apply.rs is a separate file but conceptually part of the same monolith

## Target Architecture

```
src/game/
├── state.rs              (~400 LOC) — struct, router, VERA infra, derives, accessors
├── state_init.rs         (~200 LOC) — new(), new_with_class() — no hardcoded content
├── domains/
│   ├── mod.rs
│   ├── combat.rs         — melee, ranged, post-processing, visual effects
│   ├── movement.rs       — move, tile effects, pickup triggers
│   ├── items.rs          — use_item, equip, unequip, chest, crafting, trading
│   ├── world.rs          — world_move, travel_to_tile, subterranean, pathing, encounters
│   ├── turn.rs           — end_turn, TurnPhase, tick systems
│   ├── player.rs         — psychic, adaptations, status, reputation, rest, wait, stats
│   ├── quest.rs          — accept, complete, progress, reactions
│   └── explore.rs        — auto_explore + helpers
├── effects/
│   ├── mod.rs            — Effect, Command, Presentation enums (unchanged)
│   ├── trace.rs          — Trace type aliases (unchanged)
│   └── context.rs        — QueryContext, TestContext (unchanged)
├── rules/                — pure rule functions (unchanged, eventually absorbed into domains)
└── systems/              — legacy system impls (AI, storm — eventually absorbed into domains)
```

## Domain Module Interface

Every domain module follows the same pattern:

```rust
// domains/combat.rs

use crate::game::state::GameState;
use crate::game::effects::{Command, Effect, CombatEffect, RuleOutput};
use crate::game::effects::context::QueryContext;

/// Entry point — called by state.rs dispatch()
pub fn dispatch(state: &mut GameState, command: &Command) {
    match command {
        Command::Attack { target_x, target_y } => dispatch_melee(state, *target_x, *target_y),
        Command::RangedAttack { target_x, target_y } => dispatch_ranged(state, *target_x, *target_y),
        _ => unreachable!("combat::dispatch called with non-combat command"),
    }
}

/// Apply a CombatEffect — called by state.rs apply_effect()
pub fn apply(state: &mut GameState, effect: &CombatEffect) {
    match effect {
        CombatEffect::DealDamage { enemy_idx, amount } => { ... }
        CombatEffect::Miss { .. } => {}
        CombatEffect::Kill { .. } => { ... }
        // exhaustive match
    }
}

// --- Internal ---

fn dispatch_melee(state: &mut GameState, target_x: i32, target_y: i32) {
    let output = {
        let ctx = QueryContext::from_state(state);
        rules::rule_melee_attack(target_x, target_y, &ctx, &mut state.rng)
    };
    
    // Inspect effects for post-processing decisions
    let killed = output.effects.iter().any(|e| matches!(e, Effect::Combat(CombatEffect::Kill { .. })));
    let hit = output.effects.iter().any(|e| matches!(e, Effect::Combat(CombatEffect::DealDamage { .. })));
    
    state.apply_and_trace(output, "rule_melee_attack");
    
    // Post-processing (domain-specific, not in rules or apply)
    if hit {
        state.trigger_hit_flash(target_x, target_y);
    }
    if killed {
        // swarm aggro, split-on-death, etc.
    }
}

mod rules {
    // Pure rule functions (moved from src/game/rules/combat.rs)
    pub fn rule_melee_attack(...) -> RuleOutput { ... }
    pub fn rule_ranged_attack(...) -> RuleOutput { ... }
}

#[cfg(test)]
mod tests {
    // Rule unit tests (moved from src/game/rules/combat.rs)
}
```

## state.rs After Decomposition

```rust
// ~400 LOC total

pub struct GameState { ... }  // struct definition (~80 LOC)

impl GameState {
    // VERA router
    pub fn dispatch(&mut self, command: Command) {
        match &command {
            Command::Move { .. } => domains::movement::dispatch(self, &command),
            Command::Attack { .. } | Command::RangedAttack { .. } => domains::combat::dispatch(self, &command),
            Command::UseItem { .. } | Command::UseItemOnTile { .. } => domains::items::dispatch(self, &command),
            Command::Equip { .. } | Command::Unequip { .. } => domains::items::dispatch(self, &command),
            Command::Wait | Command::Rest => domains::player::dispatch(self, &command),
            Command::WorldMove { .. } | Command::EnterSubterranean | Command::ExitSubterranean => domains::world::dispatch(self, &command),
            Command::AcceptQuest { .. } | Command::CompleteQuest { .. } => domains::quest::dispatch(self, &command),
            // ... etc
        }
    }

    // VERA infra
    pub fn apply_and_trace(&mut self, output: RuleOutput, rule_name: &'static str) { ... }
    pub fn apply_effect(&mut self, effect: &Effect) {
        match effect {
            Effect::Player(e) => domains::player::apply(self, e),
            Effect::Combat(e) => domains::combat::apply(self, e),
            Effect::Item(e) => domains::items::apply(self, e),
            Effect::Map(e) => domains::world::apply_map(self, e),
            Effect::Resource(e) => domains::player::apply_resource(self, e),
            Effect::Event(e) => domains::quest::apply_event(self, e),
            Effect::Quest(e) => domains::quest::apply(self, e),
        }
    }
    pub fn run_reactions(&mut self, effects: &[Effect], depth: u32) { ... }

    // Derives
    pub fn update_lighting(&mut self) { ... }
    pub fn update_fov(&mut self) { ... }
    fn ensure_spatial_index(&mut self) { ... }

    // Logging
    pub fn log(&mut self, msg: impl Into<String>) { ... }
    pub fn log_typed(&mut self, msg: impl Into<String>, msg_type: MsgType) { ... }

    // Accessors (30+ getters — keep here, they're thin)
    pub fn player_x(&self) -> i32 { self.player.x }
    // ...

    // Save/load
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> { ... }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> { ... }
}
```

## Domain Mapping

### domains/combat.rs (~350 LOC)

**Absorbs from state.rs:**
- `dispatch_melee_attack` (L715-821, 107 LOC)
- `dispatch_ranged_attack` (L822-914, 93 LOC)
- Visual effect calls: `trigger_hit_flash`, `spawn_damage_number`, `spawn_projectile`, `spawn_beam`

**Absorbs from other files:**
- `combat_actions.rs`: `update_enemies` (AI bridge), `try_break_wall`, `effective_armor`, `effective_reflex`
- `rules/combat.rs`: `rule_melee_attack`, `rule_ranged_attack` + tests
- `effects/apply.rs`: `apply_combat_effect`
- `systems/combat.rs`: `CombatSystem` methods used in post-processing

### domains/movement.rs (~300 LOC)

**Absorbs from state.rs:**
- `dispatch_move` (L645-714, 70 LOC)

**Absorbs from other files:**
- `rules/movement.rs`: `rule_move` + tests
- `systems/movement.rs`: tile effect handling, pickup triggers
- Movement-related apply arms from `apply.rs`

### domains/items.rs (~400 LOC)

**Absorbs from state.rs:**
- `dispatch_craft` (L1291-1323, 33 LOC)
- `dispatch_buy_item` (L1324-1339, 16 LOC)
- `dispatch_sell_item` (L1340-1356, 17 LOC)
- `pickup_items`, `pickup_filtered_items` (L2890-2920, 31 LOC)
- `open_chest`, `can_open_chest`, `transfer_to/from_chest` (L2921-3004, 84 LOC)

**Absorbs from other files:**
- `rules/item.rs`: `rule_use_item`, `rule_use_item_on_tile` + tests
- `rules/economy.rs`: `rule_craft`, `rule_buy`, `rule_sell` + tests
- `effects/apply.rs`: `apply_item_effect`

### domains/world.rs (~700 LOC)

**Absorbs from state.rs:**
- `dispatch_world_move` (L1064-1125, 62 LOC)
- `dispatch_world_move_safe` (L1126-1158, 33 LOC)
- `dispatch_follow_world_path` (L1159-1182, 24 LOC)
- `dispatch_calculate_world_path` (L1183-1218, 36 LOC)
- `travel_to_tile` (L1535-1573, 39 LOC)
- `spawn_crafting_stations` (L1574-1598, 25 LOC)
- `spawn_quest_required_npcs` (L1599-1628, 30 LOC)
- `find_safe_spawn_position` (L1629-1684, 56 LOC)
- `move_on_world_map` (L1685-1766, 82 LOC)
- `travel_to_tile_safe` (L1767-1889, 123 LOC)
- `spawn_encounter_entities` (L1890-2019, 130 LOC)
- `attempt_flee_encounter` (L2020-2056, 37 LOC)
- `check_encounter_completion` (from earlier in file)
- `enter_subterranean` (L2057-2094, 38 LOC)
- `exit_subterranean` (L2095-2144, 50 LOC)
- `calculate_world_path` (L2145-2191, 47 LOC)
- `move_along_path` (L2192-2217, 26 LOC)

**Absorbs from other files:**
- `effects/apply.rs`: `apply_map_effect`

### domains/turn.rs (~250 LOC)

**Absorbs from state.rs:**
- `end_turn` (L2409-2416, 8 LOC)
- `execute_phase` (L2417-2506, 90 LOC)
- `tick_turn_housekeeping` (L2524-2548, 25 LOC)
- `check_auto_end_turn` (L2516-2523, 8 LOC)
- `tick_time` (from earlier)
- `apply_light_effects` (L3129-3171, 43 LOC)

**Absorbs from other files:**
- `rules/turn.rs`: turn tick rules + tests
- Bridge effects: TickStatusEffects, RunAI, TickStorm (stay as bridges, domain owns them)

### domains/player.rs (~350 LOC)

**Absorbs from state.rs:**
- Wait/Rest dispatch arms
- `use_psychic_ability` (L3005-3043, 39 LOC)
- `recalc_equipment_stats` (L3044-3056, 13 LOC)
- `modify_reputation` (L3070-3091, 22 LOC)
- `apply_status` / `apply_status_effect` (L2507-2515, L3101-3123, 32 LOC)
- `check_adaptation_threshold` (L2643-2664, 22 LOC)
- `trigger_effect` (L2401-2408, 8 LOC)
- `allocate_stat` dispatch

**Absorbs from other files:**
- `rules/actions.rs`: rule_wait, rule_rest, rule_equip, etc. + tests
- `effects/apply.rs`: `apply_player_effect`, `apply_resource_effect`

### domains/quest.rs (~150 LOC)

**Absorbs from state.rs:**
- `dispatch_accept_quest` (L915-951, 37 LOC)
- `dispatch_complete_quest` (L952-1002, 51 LOC)
- `log_quest_completions` (L2562-2578, 17 LOC)
- `get_quest_ids_for_location` (L1514-1534, 21 LOC)

**Absorbs from other files:**
- `effects/apply.rs`: `apply_quest_effect`, `apply_event_effect`
- `collect_reactions` match arms for quest progress

### domains/explore.rs (~220 LOC)

**Absorbs from state.rs:**
- `auto_explore` (L2679-2805, 127 LOC)
- `has_nearby_enemies` (L2806-2828, 23 LOC)
- `is_dangerous_tile` (L2829-2848, 20 LOC)
- `has_talked_npc_at_idx` (L2849-2859, 11 LOC)
- `has_interacted_npc_at_idx` (L2860-2870, 11 LOC)
- `has_interacted_with_npc` (L2871-2889, 19 LOC)

---

## What Stays in state.rs

| Concern | LOC | Why it stays |
|---------|-----|-------------|
| GameState struct definition | ~80 | Central data definition |
| dispatch() router | ~30 | Single entry point, routes to domains |
| apply_effect() router | ~15 | Routes to domain apply functions |
| apply_and_trace() | ~20 | Generic VERA infra |
| run_reactions() / collect_reactions() | ~60 | Generic VERA infra (reactions route to domains) |
| Derives: update_lighting, update_fov, spatial index | ~60 | Cross-cutting, used by multiple domains |
| Logging: log, log_typed | ~10 | Used everywhere |
| Accessors: 30+ getters | ~100 | Thin, used by UI/rendering |
| Save/load | ~20 | Serialization |
| **Total** | **~400** | |

## What Gets Deleted

| Item | Action |
|------|--------|
| Constructor `new()` (367 LOC) | Move to `state_init.rs`, remove hardcoded content |
| 15 dispatch_* helpers (788 LOC) | Move to domain modules |
| Visual effect methods (58 LOC) | Move to domains/combat.rs or a visual_effects domain |
| Lore stubs (95 LOC) | Delete (they all return None) |
| Auto-explore (209 LOC) | Move to domains/explore.rs |
| Item/chest operations (100 LOC) | Move to domains/items.rs |
| Player system methods (100 LOC) | Move to domains/player.rs |
| World travel (453 LOC) | Move to domains/world.rs |
| Encounter system (167 LOC) | Move to domains/world.rs |
| Subterranean/pathing (161 LOC) | Move to domains/world.rs |
| effects/apply.rs (409 LOC) | Distribute apply arms to domain modules |

## Migration Phases

### Phase D0: Constructor cleanup (on main)

Move `new()` and `new_with_class()` to `state_init.rs`. Remove hardcoded `dying_pilgrim`, `hand_torch`, `glass_pick` — these should come from the spawn table data, not code. Delete the 6 lore stubs that return None.

**Verify:** `cargo test`, all DES pass. state.rs drops by ~460 LOC.

### Phase D1: Create domains/ skeleton (on main)

Create `src/game/domains/mod.rs` and 8 empty domain files. Each exports `pub fn dispatch(state, command)` and `pub fn apply(state, effect)` that initially just call the existing state.rs methods. This is a routing-only change — zero behavior change.

**Verify:** `cargo test`, all DES pass.

### Phase D2: Move combat domain (branch)

Move `dispatch_melee_attack`, `dispatch_ranged_attack`, post-processing, `apply_combat_effect`, `rules/combat.rs` into `domains/combat.rs`. Delete from state.rs and apply.rs.

**Verify:** `cargo test`, all DES pass. Combat rule unit tests still pass.

### Phase D3: Move items domain (branch)

Move item dispatch, chest operations, crafting, trading, `apply_item_effect`, `rules/item.rs`, `rules/economy.rs` into `domains/items.rs`.

### Phase D4: Move player domain (branch)

Move wait, rest, psychic, adaptations, status, reputation, stat allocation, `apply_player_effect`, `apply_resource_effect`, `rules/actions.rs` into `domains/player.rs`.

### Phase D5: Move world domain (branch)

Move world travel, encounters, subterranean, pathing, `apply_map_effect` into `domains/world.rs`. This is the largest move (~700 LOC).

### Phase D6: Move remaining domains (branch)

Move turn system into `domains/turn.rs`, quest system into `domains/quest.rs`, auto-explore into `domains/explore.rs`, movement into `domains/movement.rs`.

### Phase D7: Delete effects/apply.rs (on main)

After all domains own their apply functions, `effects/apply.rs` is empty. Delete it. The `apply_effect` method on state.rs routes to domain apply functions.

### Phase D8: Absorb rules/ into domains (optional, on main)

Move each `rules/*.rs` file into its corresponding domain module. The rules become private functions inside the domain. This is optional — keeping rules as separate files is also fine if you prefer the separation.

---

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Domain modules take `&mut GameState` — same coupling as before | True, but the coupling is explicit (function parameter) not implicit (self). Domains can't accidentally access each other's internals. |
| Cross-domain effects (combat kill → loot drop) | Handled by reactions in state.rs, which routes to the appropriate domain |
| Large merge conflicts if feature work happens in parallel | Do one domain per branch, merge before starting the next |
| state.rs accessors expose `&mut` references (enemies_mut, map_mut) | Audit after decomposition — some _mut accessors may no longer be needed |

## Success Criteria

- state.rs is ≤400 LOC
- Every domain module is self-contained: dispatch, apply, rules, tests
- No `dispatch_*` methods remain on GameState
- `cargo test` passes, all DES scenarios pass
- An agent adding a combat feature only touches `domains/combat.rs`
