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
    ]
}"#).unwrap();

assert!(result.success);
```

## Scenario Format

```json
{
    "name": "scenario_name",
    "seed": 42,
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

## Log Queries

- `player_hp` - Current HP
- `player_position` - Current coordinates
- `inventory` - Inventory contents
- `turn` - Current turn number
- `entity_at` - Entity at coordinates (`{"entity_at": {"x": 5, "y": 5}}`)
- `custom` - Custom message (`{"custom": {"message": "test"}}`)

## Entity Types

- `enemy` - Hostile entities
- `npc` - Non-player characters
- `item` - Pickupable items

## API

```rust
// From file
let result = des::run_scenario("path/to/scenario.json")?;

// From JSON string
let result = des::run_scenario_json(json_str)?;

// Manual execution
let scenario = Scenario::from_json(json_str)?;
let executor = DesExecutor::new(&scenario);
let result = executor.run(&scenario);
```

## Blocked Features

These features have dummy implementations that panic if called:

- `run_parallel()` - Parallel test execution
- `run_with_mocks()` - System mocking
- `run_rendered()` - Slow rendered execution
- `Scenario::inherit_from()` - Base file inheritance

See `design_docs/DES_TODO.md` for implementation roadmap.
