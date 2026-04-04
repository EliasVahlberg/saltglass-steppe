---
name: codebase-des-system
description: DES (Debug Execution System) interpreter architecture, scenario format, all available actions and assertions. Use when writing new DES test scenarios, debugging failing tests, or extending the DES with new commands.
---

# Codebase: DES System

**Location**: `src/des/mod.rs` (2,325 lines)

## Architecture

DES runs `GameState` headlessly without rendering. Scenarios are JSON files that define initial state, scheduled actions, and assertions.

```
Scenario (JSON)
  → DesExecutor::new(scenario)   // set up GameState
  → DesExecutor::run(scenario)   // execute actions turn by turn
  → ExecutionResult              // success/failure + assertion results
```

## Key Types

```rust
pub struct Scenario {
    pub name: String,
    pub seed: Option<u64>,
    pub mocks: MockSettings,       // force hit/miss, fixed damage
    pub entities: Vec<EntitySpawn>,
    pub player: PlayerSetup,
    pub actions: Vec<ScheduledAction>,
    pub assertions: Vec<Assertion>,
    pub base: Option<String>,      // inherit from another scenario file
    pub variables: HashMap<String, serde_json::Value>,
    pub map_setup: MapSetup,
}

pub struct MockSettings {
    pub combat_always_hit: Option<bool>,
    pub combat_fixed_damage: Option<i32>,
}

pub struct MapSetup {
    pub clear_radius: Option<i32>,    // clear tiles around player
    pub clear_areas: Vec<ClearArea>,  // {x, y, width, height}
    pub ensure_paths: Vec<EnsurePath>, // {from_x, from_y, to_x, to_y}
}
```

## Scenario JSON Format

```json
{
  "name": "my_test",
  "seed": 12345,
  "mocks": { "combat_always_hit": true, "combat_fixed_damage": 10 },
  "map_setup": { "clear_radius": 5 },
  "player": { "x": 10, "y": 10, "hp": 20, "inventory": ["sword"] },
  "entities": [
    { "entity_type": "enemy", "id": "glass_crawler", "x": 11, "y": 10 },
    { "entity_type": "item", "id": "health_potion", "x": 12, "y": 10 },
    { "entity_type": "npc", "id": "dying_pilgrim", "x": 9, "y": 10 }
  ],
  "actions": [
    { "turn": 0, "action": { "type": "attack", "target_x": 11, "target_y": 10 } },
    { "turn": 1, "action": { "type": "move", "dx": 1, "dy": 0 } }
  ],
  "assertions": [
    { "at_end": true, "check": { "type": "enemy_at", "x": 11, "y": 10, "alive": false } },
    { "after_turn": 1, "check": { "type": "player_position", "x": 11, "y": 10 } }
  ]
}
```

## All Actions

```json
{ "type": "move", "dx": 1, "dy": 0 }
{ "type": "teleport", "x": 5, "y": 5 }
{ "type": "attack", "target_x": 11, "target_y": 10 }
{ "type": "ranged_attack", "target_x": 15, "target_y": 10 }
{ "type": "use_item", "item_index": 0 }
{ "type": "use_item_on", "item_index": 0, "x": 5, "y": 5 }
{ "type": "equip", "item_index": 0, "slot": "weapon" }
{ "type": "unequip", "slot": "weapon" }
{ "type": "auto_explore" }
{ "type": "wait", "turns": 3 }
{ "type": "rest" }
{ "type": "end_turn" }
{ "type": "apply_status", "effect": "poison", "duration": 3, "potency": 5 }
{ "type": "accept_quest", "quest_id": "pilgrims_last_angle" }
{ "type": "complete_quest", "quest_id": "pilgrims_last_angle" }
{ "type": "craft", "recipe_id": "glass_shard_blade" }
{ "type": "buy_item", "item_id": "health_potion", "npc_id": "trader_1" }
{ "type": "sell_item", "item_id": "glass_shard" }
{ "type": "interact", "target_x": 5, "target_y": 5 }
{ "type": "examine", "target_x": 5, "target_y": 5 }
{ "type": "trigger_storm", "intensity": 3 }
{ "type": "set_refraction", "value": 50 }
{ "type": "set_faction_rep", "faction": "Mirror Monks", "value": 50 }
{ "type": "set_level", "level": 5 }
{ "type": "set_salt_scrip", "amount": 100 }
{ "type": "set_tile", "x": 5, "y": 5, "tile_type": "floor" }
{ "type": "spawn_enemy", "enemy_id": "glass_crawler", "x": 5, "y": 5, "hp": 10 }
{ "type": "give_adaptation", "adaptation_id": "prismhide" }
{ "type": "unlock_ability", "ability_id": "mind_blast" }
{ "type": "use_ability", "ability_id": "mind_blast" }
{ "type": "allocate_stat", "stat": "max_hp" }
{ "type": "log", "query": { "type": "player_hp" } }
```

## All Assertions

```json
{ "check": { "type": "player_hp", "op": "gt", "value": 0 } }
{ "check": { "type": "player_position", "x": 5, "y": 5 } }
{ "check": { "type": "player_alive" } }
{ "check": { "type": "player_dead" } }
{ "check": { "type": "inventory_contains", "item": "sword" } }
{ "check": { "type": "inventory_size", "op": "eq", "value": 3 } }
{ "check": { "type": "enemy_at", "x": 11, "y": 10, "alive": false } }
{ "check": { "type": "no_enemy_at", "x": 11, "y": 10 } }
{ "check": { "type": "enemy_hp", "id": "glass_crawler", "op": "lt", "value": 10 } }
{ "check": { "type": "enemy_alive", "id": "glass_crawler" } }
{ "check": { "type": "enemy_dead", "id": "glass_crawler" } }
{ "check": { "type": "turn", "op": "eq", "value": 5 } }
{ "check": { "type": "player_has_adaptation", "adaptation": "prismhide" } }
{ "check": { "type": "adaptation_count", "op": "gte", "value": 2 } }
{ "check": { "type": "map_tile_at", "x": 5, "y": 5, "tile": "floor" } }
{ "check": { "type": "refraction", "op": "gte", "value": 10 } }
{ "check": { "type": "player_ap", "op": "eq", "value": 3 } }
{ "check": { "type": "has_status_effect", "effect": "poison" } }
{ "check": { "type": "status_effect_count", "op": "eq", "value": 1 } }
{ "check": { "type": "tile_explored", "x": 5, "y": 5 } }
{ "check": { "type": "explored_count", "op": "gt", "value": 100 } }
{ "check": { "type": "equipped_in_slot", "slot": "weapon", "item": "sword" } }
{ "check": { "type": "player_armor", "op": "gte", "value": 5 } }
```

Comparison ops (`op`): `"eq"`, `"ne"`, `"lt"`, `"lte"`, `"gt"`, `"gte"`

Assertion timing:
- `"at_end": true` — checked after all actions complete
- `"after_turn": N` — checked after turn N executes

## Public API

```rust
// Run from file
pub fn run_scenario(path: impl AsRef<Path>) -> Result<ExecutionResult, String>

// Run from JSON string
pub fn run_scenario_json(json: &str) -> Result<ExecutionResult, String>

// Run multiple in parallel (rayon)
pub fn run_parallel(scenarios: &[Scenario]) -> Vec<ExecutionResult>

// Run with render callback (for visual debugging)
pub fn run_with_render<F>(scenario: &Scenario, render_fn: F) -> ExecutionResult
```

## Scenario Inheritance

```json
{ "base": "tests/scenarios/base_combat.json", "name": "variant_test" }
```

Child scenario inherits all fields from base, then overrides with its own values.

## Running Tests

```bash
cargo test --test des_scenarios              # all scenarios
cargo test --test des_scenarios combat_basic # specific scenario
cargo test --test des_scenarios -- --nocapture # with output
```

## Writing New Scenarios

1. Create `tests/scenarios/my_test.json`
2. Set `seed` for reproducibility
3. Use `map_setup.clear_radius` to ensure walkable space
4. Use `mocks` to control randomness
5. Schedule actions at specific turns
6. Add `at_end` assertions for final state checks
7. Run with `cargo test --test des_scenarios my_test`
