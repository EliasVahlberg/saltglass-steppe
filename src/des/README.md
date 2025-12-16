# Debug Execution System (DES)

Headless game execution for automated testing of TUI-RPG.

## Overview

DES runs game scenarios without rendering, enabling automated testing of game mechanics, combat, items, and entity behaviors.

## Quick Start

```rust
use tui_rpg::des::run_scenario_json;

let result = run_scenario_json(r#"{
    "name": "basic_test",
    "seed": 42,
    "actions": [
        {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}},
        {"turn": 1, "action": {"type": "log", "query": "player_hp"}}
    ],
    "assertions": [
        {"at_end": true, "check": {"type": "player_alive"}}
    ]
}"#).unwrap();

assert!(result.success);
```

## Scenario Format

```json
{
    "name": "scenario_name",
    "seed": 42,
    "base": "base_scenario.json",
    "variables": {"item_id": "brine_vial"},
    "player": {
        "x": 10,
        "y": 10,
        "hp": 20,
        "max_hp": 20,
        "inventory": ["brine_vial"],
        "adaptations": []
    },
    "entities": [
        {
            "entity_type": "enemy",
            "id": "salt_crawler",
            "x": 12,
            "y": 10,
            "hp": 5
        }
    ],
    "actions": [
        {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}},
        {"turn": 1, "action": {"type": "attack", "target_x": 12, "target_y": 10}}
    ],
    "assertions": [
        {"after_turn": 1, "check": {"type": "player_hp", "op": "ge", "value": 10}},
        {"at_end": true, "check": {"type": "player_alive"}}
    ]
}
```

## Actions

| Action | Parameters | Description |
|--------|------------|-------------|
| `move` | `dx`, `dy` | Move player relative to current position |
| `teleport` | `x`, `y` | Set player position directly |
| `attack` | `target_x`, `target_y` | Attack adjacent target |
| `use_item` | `item_index` | Use item from inventory |
| `wait` | `turns` | Skip turns |
| `log` | `query` | Log game state |

## Assertions

| Check Type | Parameters | Description |
|------------|------------|-------------|
| `player_hp` | `op`, `value` | Compare player HP |
| `player_position` | `x`, `y` | Check exact position |
| `player_alive` | - | Player HP > 0 |
| `player_dead` | - | Player HP <= 0 |
| `inventory_contains` | `item` | Item in inventory |
| `inventory_size` | `op`, `value` | Compare inventory size |
| `enemy_at` | `x`, `y`, `alive` | Enemy at position |
| `no_enemy_at` | `x`, `y` | No enemy at position |
| `turn` | `op`, `value` | Compare turn number |

Comparison operators (`op`): `eq`, `ne`, `lt`, `le`, `gt`, `ge`

## API

```rust
// From file
let result = des::run_scenario("path/to/scenario.json")?;

// From JSON string
let result = des::run_scenario_json(json_str)?;

// With variable substitution
let scenario = Scenario::from_json_with_vars(json, &vars)?;

// Manual execution with options
let executor = DesExecutor::new(&scenario)
    .with_snapshots()      // Enable state capture
    .with_rng_seed(123);   // Override RNG seed
let result = executor.run(&scenario);

// Parallel execution
let results = des::run_parallel(&scenarios);

// Rendered execution with callback
des::run_with_render(&scenario, 100, |state, log| {
    println!("Turn {}: {:?}", state.turn, log);
});
```

## Base File Inheritance

Scenarios can inherit from base files:

```json
// base.json
{
    "name": "base",
    "seed": 42,
    "player": {"hp": 20, "max_hp": 20}
}

// test.json
{
    "name": "test",
    "base": "base.json",
    "actions": [{"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}}]
}
```

## Variable Substitution

Use `${var}` syntax in JSON:

```rust
let vars = HashMap::from([("item".to_string(), json!("brine_vial"))]);
let scenario = Scenario::from_json_with_vars(r#"{"name": "${item}_test"}"#, &vars)?;
```

## State Snapshots

Enable snapshots for debugging:

```rust
let executor = DesExecutor::new(&scenario).with_snapshots();
let result = executor.run(&scenario);

for snapshot in &result.snapshots {
    println!("Action {}: HP={}, Pos=({},{})", 
             snapshot.action_index, snapshot.player_hp,
             snapshot.player_x, snapshot.player_y);
}
```

## CI Integration

Run all scenarios in `tests/scenarios/`:

```bash
cargo test --test des_scenarios
```

See `.github/workflows/ci.yml` for GitHub Actions configuration.
