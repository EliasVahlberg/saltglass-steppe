# Auto-Explore System Enhancement

## Overview

The auto-explore system has been enhanced with intelligent item pickup, danger avoidance, enemy detection, and data-driven configuration. This system allows players to efficiently explore the map while maintaining safety and collecting valuable items automatically.

## Features

### 1. Automatic Item Pickup

The auto-explore system now automatically picks up items from the ground when moving to a new tile.

**Configuration:**
- `pickup_items`: Enable/disable automatic item pickup
- `item_filters`: Configure which items to pick up or ignore

**Behavior:**
- Items are picked up before movement pathfinding
- Pickup messages are displayed in the game log
- Respects item filtering configuration

### 2. Danger Avoidance

Auto-explore now avoids dangerous tiles to prevent player harm.

**Configuration:**
- `avoid_dangers`: Enable/disable danger avoidance
- `danger_types`: List of tile types to avoid (e.g., "glass", "storm_tile", "glare")

**Behavior:**
- Dangerous tiles are excluded from pathfinding
- Player will find alternate routes around hazards
- Final movement check prevents stepping on dangerous tiles

### 3. Enemy Detection

Auto-explore stops when enemies are detected nearby to prevent unwanted combat.

**Configuration:**
- `stop_on_enemies`: Enable/disable enemy detection
- `enemy_detection_range`: Distance to detect enemies (default: 8 tiles)
- `ignore_weak_enemies`: Option to ignore enemies below HP threshold
- `weak_enemy_threshold`: HP threshold for weak enemies (default: 10)

**Behavior:**
- Scans for enemies within detection range before moving
- Displays warning message when enemies are detected
- Can optionally ignore weak enemies based on configuration

### 4. Data-Driven Configuration

All auto-explore behavior is configurable through `data/auto_explore_config.json`.

## Configuration File

**Location:** `data/auto_explore_config.json`

```json
{
  "auto_explore": {
    "pickup_items": true,
    "avoid_dangers": true,
    "stop_on_enemies": true,
    "enemy_detection_range": 8,
    "ignore_weak_enemies": false,
    "weak_enemy_threshold": 10,
    "item_filters": {
      "enabled": false,
      "blacklist": [],
      "whitelist": []
    },
    "danger_types": [
      "glass",
      "storm_tile",
      "glare",
      "hot_light"
    ]
  }
}
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `pickup_items` | boolean | `true` | Automatically pick up items during exploration |
| `avoid_dangers` | boolean | `true` | Avoid dangerous tiles during pathfinding |
| `stop_on_enemies` | boolean | `true` | Stop exploring when enemies are detected |
| `enemy_detection_range` | integer | `8` | Range in tiles to detect enemies |
| `ignore_weak_enemies` | boolean | `false` | Ignore enemies below HP threshold |
| `weak_enemy_threshold` | integer | `10` | HP threshold for considering enemies "weak" |
| `item_filters.enabled` | boolean | `false` | Enable item filtering |
| `item_filters.blacklist` | array | `[]` | Items to never pick up |
| `item_filters.whitelist` | array | `[]` | Only pick up these items (if not empty) |
| `danger_types` | array | See above | List of dangerous tile types to avoid |

## Implementation Details

### Code Structure

**Main Module:** `src/game/auto_explore.rs`
- `AutoExploreConfig`: Configuration structure
- `get_auto_explore_config()`: Access to loaded configuration
- Item filtering logic and danger type checking

**Enhanced Methods in GameState:**
- `auto_explore()`: Main auto-explore method with all enhancements
- `has_nearby_enemies()`: Enemy detection logic
- `is_dangerous_tile()`: Danger checking logic
- `pickup_filtered_items()`: Filtered item pickup logic

### Integration Points

1. **Configuration Loading:** Uses `once_cell::Lazy` for efficient config loading
2. **Game Loop Integration:** Called from main game loop on auto-explore input
3. **Message System:** Integrates with game message system for user feedback
4. **Pathfinding:** Integrates with existing BFS pathfinding system

## Usage Examples

### Basic Auto-Explore
```rust
// Player presses auto-explore key
if state.auto_explore() {
    // Movement successful, items picked up, dangers avoided
} else {
    // No valid path found or enemies detected
}
```

### Custom Configuration
```json
{
  "auto_explore": {
    "pickup_items": true,
    "item_filters": {
      "enabled": true,
      "whitelist": ["storm_glass", "brine_vial"]
    },
    "ignore_weak_enemies": true,
    "weak_enemy_threshold": 15
  }
}
```

## Testing

### DES Test Scenarios

The system includes comprehensive DES (Debug Execution System) tests:

1. **Item Pickup Test** (`auto_explore_item_pickup.json`)
   - Verifies automatic item collection
   - Tests pickup message generation

2. **Enemy Detection Test** (`auto_explore_enemy_detection.json`)
   - Verifies enemy detection stops exploration
   - Tests warning message display

3. **Danger Avoidance Test** (`auto_explore_danger_avoidance.json`)
   - Verifies pathfinding around dangerous tiles
   - Tests alternate route finding

4. **Configuration Test** (`auto_explore_configuration.json`)
   - Tests configuration-based behavior changes
   - Verifies weak enemy ignoring functionality

### Running Tests

```bash
# Run all auto-explore tests
cargo test --test des_scenarios auto_explore

# Run specific test
cargo test --test des_scenarios auto_explore_item_pickup
```

## Future Enhancements

### Potential Improvements

1. **Smart Pathfinding:** Prefer safer routes even when not strictly necessary
2. **Item Priority:** Pick up more valuable items first when multiple items are present
3. **Dynamic Danger Assessment:** Adjust danger avoidance based on player adaptations
4. **Exploration Memory:** Remember previously explored areas for more efficient pathfinding
5. **Combat Integration:** Option to engage weak enemies automatically

### Configuration Extensions

1. **Time-based Filters:** Only pick up items during certain game phases
2. **Inventory Management:** Stop picking up items when inventory is full
3. **Biome-specific Rules:** Different behavior in different biome types
4. **Adaptive Thresholds:** Adjust enemy detection based on player level/equipment

## Troubleshooting

### Common Issues

1. **Auto-explore not moving:** Check for nearby enemies or dangerous tiles
2. **Items not being picked up:** Verify `pickup_items` is enabled and item filters
3. **Avoiding safe tiles:** Check `danger_types` configuration for false positives
4. **Not detecting enemies:** Verify `enemy_detection_range` and `stop_on_enemies` settings

### Debug Information

Enable debug logging to see auto-explore decision making:
- Enemy detection messages
- Danger avoidance decisions
- Item pickup filtering results
- Pathfinding route selection

---

*This enhancement maintains backward compatibility while adding powerful new functionality for improved gameplay experience.*
