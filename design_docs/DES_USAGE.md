# Debug Execution System (DES) - Usage Guide

## Overview

DES enables automated testing of game mechanics by running scenarios headlessly. Tests are defined in JSON files that specify initial state, actions, and assertions.

## Quick Start

```json
{
  "name": "my_test",
  "seed": 42,
  "player": {"x": 5, "y": 5, "hp": 20, "max_hp": 20},
  "actions": [
    {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}}
  ],
  "assertions": [
    {"at_end": true, "check": {"type": "player_position", "x": 6, "y": 5}}
  ]
}
```

## Scenario Schema

### Root Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Scenario identifier |
| `seed` | u64 | no | RNG seed for determinism |
| `base` | string | no | Path to base scenario for inheritance |
| `mocks` | MockSettings | no | Override game systems for testing |
| `variables` | object | no | Variables for substitution |
| `player` | PlayerSetup | no | Player initial state |
| `entities` | EntitySpawn[] | no | Entities to spawn |
| `actions` | ScheduledAction[] | no | Actions to execute |
| `assertions` | Assertion[] | no | Conditions to verify |

### MockSettings

Override game systems for deterministic testing:

```json
{
  "mocks": {
    "combat_always_hit": true,    // Force all attacks to hit (true) or miss (false)
    "combat_fixed_damage": 5      // Force specific damage value
  }
}
```

### PlayerSetup

```json
{
  "x": 5, "y": 5,
  "hp": 20, "max_hp": 20,
  "ap": 4, "max_ap": 4,
  "xp": 90,
  "inventory": ["brine_vial", "salt_knife"],
  "adaptations": ["prismhide", "saltblood"],
  "equipped_weapon": "salt_knife"
}
```

### EntitySpawn

```json
{
  "entity_type": "enemy",  // "enemy", "npc", or "item"
  "id": "mirage_hound",    // Entity definition ID
  "x": 6, "y": 5,
  "hp": 10,                // Optional: override HP
  "ai_disabled": true,     // Optional: disable AI
  "inventory": ["brine_vial", "salt_knife"]  // Optional: items carried
}
```

### ScheduledAction

```json
{"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}}
```

## Actions

| Action | Parameters | Description |
|--------|------------|-------------|
| `move` | `dx`, `dy` | Move relative to current position |
| `teleport` | `x`, `y` | Set absolute position |
| `attack` | `target_x`, `target_y` | Melee attack adjacent tile |
| `ranged_attack` | `target_x`, `target_y` | Ranged attack |
| `use_item` | `item_index` | Use item from inventory |
| `equip` | `item_index`, `slot` | Equip item to slot |
| `unequip` | `slot` | Unequip slot |
| `apply_status` | `effect`, `duration`, `potency` | Apply status effect |
| `auto_explore` | - | Auto-explore one step |
| `wait` | `turns` | Wait N turns |
| `end_turn` | - | End current turn |
| `allocate_stat` | `stat` | Allocate pending stat point (max_hp/max_ap/reflex) |
| `log` | `query` | Log state (player_hp, player_position, inventory, turn) |

### Equipment Slots

Available slots for `equip` and `unequip` actions:
- `weapon` - Primary weapon
- `jacket` - Body armor (provides armor value)
- `accessory` - Accessories (e.g., hand torch for light)
- `boots` - Footwear
- `gloves` - Hand protection
- `backpack` - Storage
- `necklace` - Jewelry

## Assertions

### Timing

- `after_turn: N` - Check after turn N completes
- `at_end: true` - Check at scenario end

### Comparison Operators

`op`: `eq`, `ne`, `lt`, `le`, `gt`, `ge`

### Assertion Types

#### Player State
```json
{"type": "player_hp", "op": "eq", "value": 20}
{"type": "player_position", "x": 5, "y": 5}
{"type": "player_alive"}
{"type": "player_dead"}
{"type": "player_ap", "op": "ge", "value": 2}
{"type": "player_armor", "op": "eq", "value": 3}
{"type": "player_xp", "op": "ge", "value": 100}
{"type": "player_level", "op": "eq", "value": 1}
{"type": "pending_stat_points", "op": "eq", "value": 0}
```

#### Messages
```json
{"type": "message_contains", "text": "CREDENTIAL VALID"}
```

#### Inventory & Equipment
```json
{"type": "inventory_contains", "item": "brine_vial"}
{"type": "inventory_size", "op": "eq", "value": 3}
{"type": "equipped_in_slot", "slot": "weapon", "item": "salt_knife"}
{"type": "item_inspect_has_stat", "item": "salt_knife", "stat": "damage"}
{"type": "item_inspect_missing_stat", "item": "scripture_shard", "stat": "value"}
```

#### Enemies
```json
{"type": "enemy_at", "x": 6, "y": 5, "alive": true}
{"type": "no_enemy_at", "x": 6, "y": 5}
{"type": "enemy_hp", "id": "mirage_hound", "op": "lt", "value": 5}
{"type": "enemy_alive", "id": "mirage_hound"}
{"type": "enemy_dead", "id": "mirage_hound"}
{"type": "enemy_provoked", "id": "salt_pilgrim", "provoked": true}
{"type": "enemy_has_item", "id": "mirage_hound", "item": "brine_vial"}
```

#### NPCs
```json
{"type": "npc_talked", "id": "dying_pilgrim", "talked": true}
```

#### Adaptations & Status
```json
{"type": "player_has_adaptation", "adaptation": "prismhide"}
{"type": "adaptation_count", "op": "ge", "value": 2}
{"type": "has_status_effect", "effect": "poison"}
{"type": "status_effect_count", "op": "eq", "value": 0}
```

#### Map & Exploration
```json
{"type": "map_tile_at", "x": 5, "y": 5, "tile": "floor"}
{"type": "tile_explored", "x": 10, "y": 10}
{"type": "explored_count", "op": "gt", "value": 50}
{"type": "light_level", "x": 5, "y": 5, "op": "gt", "value": 0}
```

#### Turn
```json
{"type": "turn", "op": "ge", "value": 5}
```

## Inheritance

Use `base` to inherit from another scenario:

```json
{
  "name": "combat_test",
  "base": "BASE_combat.json",
  "actions": [
    {"turn": 0, "action": {"type": "attack", "target_x": 6, "target_y": 5}}
  ],
  "assertions": [
    {"at_end": true, "check": {"type": "enemy_dead", "id": "mirage_hound"}}
  ]
}
```

Inheritance merges:
- `seed`, `player` fields: child overrides base
- `entities`, `actions`, `assertions`: base prepended to child

## Base Scenarios

Located in `tests/scenarios/BASE_*.json`:

| File | Setup |
|------|-------|
| `BASE_empty_room.json` | Player at (5,5), 20 HP, 4 AP |
| `BASE_combat.json` | Player + adjacent enemy (AI disabled) |
| `BASE_equipped_player.json` | Player with salt_knife + salt_vest |
| `BASE_npc.json` | Player + adjacent NPC |
| `BASE_items.json` | Player (10 HP) + items on ground |
| `BASE_progression.json` | High-stat player + weak enemy |

## Best Practices

1. **Use deterministic seeds** - Always set `seed` for reproducible tests
2. **Disable AI for combat tests** - Set `ai_disabled: true` to control enemy behavior
3. **Use BASE_* files** - Inherit common setups to reduce duplication
4. **Test one thing** - Each scenario should verify a single mechanic
5. **Use meaningful names** - Scenario names should describe what's being tested
6. **Add messages** - Include `message` in assertions for clear failure output

## Running Tests

```bash
# Run all DES scenarios
cargo test run_all_scenarios

# Run specific scenario test
cargo test basic_movement_scenario
```

## Programmatic Usage

```rust
use tui_rpg::des::{run_scenario, Scenario, DesExecutor};

// From file
let result = run_scenario("tests/scenarios/my_test.json")?;
assert!(result.success);

// From JSON string
let result = run_scenario_json(r#"{"name": "test", ...}"#)?;

// With custom RNG
let scenario = Scenario::from_file("test.json")?;
let executor = DesExecutor::new(&scenario).with_rng_seed(12345);
let result = executor.run(&scenario);
```
