# Interaction System Documentation

## Overview

The interaction system allows players to interact with and examine objects in the game world. This system is data-driven and supports quest integration.

## Key Components

### Actions
- **Interact (E key)**: Interact with objects at the player's position
- **Examine (X key)**: Examine objects at the player's position for detailed information

### Data Structure

Interactable objects are defined in `data/interactables.json`:

```json
{
  "interactables": [
    {
      "id": "lever",
      "name": "Lever",
      "glyph": "L",
      "description": "A mechanical lever that can be pulled.",
      "interaction_type": "toggle",
      "states": ["off", "on"],
      "messages": {
        "interact": "You pull the lever.",
        "examine": "A sturdy metal lever. It appears to be {state}."
      }
    }
  ]
}
```

### Interaction Types

- **toggle**: Cycles through states (e.g., off → on → off)
- **press**: Sets to "pressed" state when interacted with

### State Management

Each interactable has:
- `current_state`: Index into the `states` array
- Dynamic state transitions based on `interaction_type`
- Message templates with `{state}` placeholder for current state

## Implementation Details

### Core Files

1. **`src/game/interactable.rs`**: Core interactable logic and data structures
2. **`src/game/state.rs`**: `interact_at()` and `examine_at()` methods
3. **`src/ui/input.rs`**: Key bindings and action definitions
4. **`src/main.rs`**: Action handling in main game loop

### Spatial Indexing

Interactables are spatially indexed for efficient lookup:
- `GameState.interactable_positions: HashMap<(i32, i32), usize>`
- Updated automatically when spatial index is rebuilt

### Quest Integration

The system integrates with the quest system:
- `quest_log.on_interact(target_id)` called when interacting
- `quest_log.on_examine(target_id)` called when examining
- Supports quest objectives of type `Interact` and `Examine`

## Usage

### Adding New Interactables

1. Add definition to `data/interactables.json`
2. Spawn in game world using `Interactable::new(id, x, y)`
3. Add to `GameState.interactables` vector

### Key Bindings

- **E**: Interact with object at player position
- **X**: Examine object at player position

### Fallback Behavior

If no interactable is found at the position:
- **Interact**: Checks for NPCs, then chests, then displays "nothing to interact with"
- **Examine**: Checks for enemies, NPCs, items, chests, then describes the tile

## Testing

The system includes comprehensive DES (Debug Execution System) tests:

- **Test File**: `tests/scenarios/interaction_system_test.json`
- **Test Runner**: `tests/des_scenarios.rs`
- **Run Command**: `cargo test interaction_system_test --test des_scenarios`

### Test Coverage

- Interactable spawning
- Interact action execution
- Examine action execution
- State transitions
- Message generation

## Future Enhancements

- Support for more interaction types (e.g., "use_item_on")
- Conditional interactions based on player state
- Multi-step interaction sequences
- Visual feedback for interactable objects
- Range-based interactions (not just at player position)

## Example Usage in Quests

```json
{
  "objectives": [
    {
      "type": "Interact",
      "target": "lever",
      "description": "Pull the lever to open the door"
    },
    {
      "type": "Examine", 
      "target": "ancient_console",
      "description": "Examine the console for clues"
    }
  ]
}
```

The system automatically tracks these interactions and updates quest progress accordingly.
