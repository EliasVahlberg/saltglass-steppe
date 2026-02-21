# Psychic Menu System Implementation

## Overview

The psychic menu system provides a user interface for accessing and using the 16 psychic abilities available in the game. This addresses the critical gap where psychic abilities existed but were completely inaccessible to players.

## Implementation Details

### Core Components

1. **PsychicMenu struct** (`src/ui/psychic_menu.rs`)
   - Manages menu state (active/inactive, selected index)
   - Provides navigation and selection methods
   - Integrates with existing UI system

2. **UI Integration** (`src/ui/input.rs`, `src/main.rs`)
   - Added 'p' hotkey to open psychic menu
   - Integrated menu input handling into main UI flow
   - Added action types for psychic ability usage

3. **HUD Enhancement** (`src/ui/hud.rs`)
   - Added Coherence bar to main stats display
   - Updated hotkeys panel to show combat actions
   - Coherence displayed alongside HP and AP

### Key Features

- **Category-based organization**: Abilities grouped by type (Telepathy, Probability, Energy, Phasing, Temporal)
- **Resource management**: Shows coherence costs and current coherence
- **Cooldown tracking**: Displays ability cooldowns and availability
- **Visual feedback**: Color-coded abilities based on affordability and status

### Usage

1. Press 'p' to open psychic abilities menu
2. Use ↑↓ or k/j to navigate abilities
3. Press Enter to use selected ability
4. Press Esc or 'p' to close menu

### Integration Points

- **GameState.psychic**: Existing psychic system integration
- **Action handling**: New UsePsychicAbility action type
- **UI rendering**: Integrated into main UI rendering flow

## Testing

- Created DES test scenario for psychic ability usage
- Verified menu navigation and ability activation
- Confirmed coherence cost deduction and cooldown tracking

## Impact

This implementation unlocks the entire psychic combat system for players, transforming 16 hidden abilities into accessible gameplay mechanics. The coherence bar provides essential resource management feedback, and the enhanced hotkeys improve combat action discoverability.

## Future Enhancements

- Ability tooltips with detailed effect descriptions
- Keyboard shortcuts for frequently used abilities
- Visual effects for ability activation
- Integration with targeting system for targeted abilities
