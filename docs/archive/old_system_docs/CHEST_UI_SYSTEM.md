# Chest UI System

**Version:** 1.0  
**Date:** 2025-12-29  
**Status:** Implemented  

## Overview

The Chest UI system provides a user-friendly interface for players to interact with chests, allowing them to transfer items between their inventory and chest storage. The system supports multiple chest types with different capacities and loot tables.

## Features

### Core Functionality
- **Dual-Panel Interface**: Side-by-side view of chest contents and player inventory
- **Item Transfer**: Move items between chest and inventory with Enter key
- **Panel Navigation**: Switch between chest and inventory panels with Tab
- **Keyboard Controls**: Full keyboard navigation (↑↓ to navigate, Tab to switch panels)
- **Capacity Display**: Shows current chest usage vs. maximum capacity

### Chest Types Supported
- **Wooden Chest**: Basic storage (8 items)
- **Metal Strongbox**: Secure storage (6 items, locked)
- **Glass Cache**: Specialized storage (4 items)
- **Archive Container**: High-capacity storage (10 items, locked)
- **Supply Crate**: Large storage (12 items)

### Locking System
- **Locked Chests**: Require specific keys to open
- **Key Validation**: Checks player inventory for required keys
- **Unlock Messages**: Provides feedback when unlocking chests

## User Interface

### Layout
```
┌─ Chest Name - Description ─────────────────────────────────────┐
│                                                                │
└────────────────────────────────────────────────────────────────┘
┌─ Chest (2/8) ──────────────┐┌─ Inventory ────────────────────┐
│ 1. Brine Vial              ││ 1. Hand Torch                  │
│ 2. Storm Glass             ││ 2. Salt Poultice               │
│                            ││ 3. Glass Pick                  │
│                            ││                                │
└────────────────────────────┘└────────────────────────────────┘
┌─ Controls ─────────────────────────────────────────────────────┐
│ ↑↓ Navigate  Tab Switch Panel  Enter Transfer  Esc Close      │
└────────────────────────────────────────────────────────────────┘
```

### Visual Indicators
- **Active Panel**: Highlighted in yellow
- **Inactive Panel**: Grayed out
- **Selected Item**: Highlighted with yellow background
- **Capacity Display**: Shows current/maximum items in chest

## Controls

| Key | Action |
|-----|--------|
| `Shift+C` | Open chest (when adjacent) |
| `↑` / `k` | Navigate up |
| `↓` / `j` | Navigate down |
| `Tab` | Switch between chest and inventory panels |
| `Enter` | Transfer selected item |
| `Esc` | Close chest interface |

## Technical Implementation

### Core Components

#### ChestUI State
```rust
pub struct ChestUI {
    pub chest_index: usize,
    pub chest_list_state: ListState,
    pub inventory_list_state: ListState,
    pub selected_panel: ChestPanel,
}
```

#### Panel Management
- **ChestPanel::ChestInventory**: Focus on chest contents
- **ChestPanel::PlayerInventory**: Focus on player inventory
- **Panel Switching**: Tab key toggles between panels

#### Item Transfer Logic
- **From Chest**: Removes item from chest, adds to player inventory
- **To Chest**: Removes item from player inventory, adds to chest
- **Capacity Checking**: Prevents overfilling chests
- **Validation**: Ensures valid indices and chest availability

### Integration Points

#### Game State Methods
```rust
pub fn open_chest(&mut self, chest_index: usize) -> bool
pub fn transfer_to_chest(&mut self, chest_index: usize, inventory_index: usize) -> bool
pub fn transfer_from_chest(&mut self, chest_index: usize, chest_item_index: usize) -> bool
```

#### Input Handling
- **Action::OpenChest**: Triggered by Shift+C when adjacent to chest
- **Action::ChestTransfer**: Handles item transfer between containers
- **Action::CloseChest**: Closes the chest interface

#### UI State Management
- **ui.chest_ui**: Optional ChestUI state in main UI state
- **Spatial Indexing**: Uses chest_positions HashMap for quick lookup
- **Adjacency Check**: Validates player is next to chest before opening

## Interaction Flow

### Opening a Chest
1. Player moves adjacent to chest
2. Player presses `Shift+C`
3. System checks if chest is locked
4. If locked, validates player has required key
5. Opens chest UI with dual-panel interface

### Transferring Items
1. Player navigates to desired item
2. Player presses `Enter`
3. System validates transfer (capacity, valid indices)
4. Moves item between containers
5. Updates UI display and provides feedback

### Closing Interface
1. Player presses `Esc`
2. System closes chest UI
3. Returns to normal game view

## Error Handling

### Common Scenarios
- **No Chest Present**: "No chest here." message
- **Locked Chest**: "Chest is locked. You need a [key]." message
- **Full Chest**: "Chest is full." message
- **Invalid Transfer**: Silently fails with no action

### Validation Checks
- **Adjacency**: Player must be within 1 tile of chest
- **Capacity**: Chest must have space for new items
- **Index Bounds**: Prevents out-of-bounds access
- **Key Requirements**: Validates unlock conditions

## Performance Characteristics

### Memory Usage
- **UI State**: Minimal overhead (list states + panel selection)
- **Rendering**: Efficient list widgets with stateful selection
- **Item Display**: On-demand item name resolution

### Responsiveness
- **Instant Navigation**: Immediate response to arrow keys
- **Fast Transfer**: Single-frame item movement
- **Smooth Panel Switching**: Immediate Tab response

## Future Enhancements

### Planned Features
1. **Drag & Drop**: Mouse-based item transfer
2. **Bulk Transfer**: Move multiple items at once
3. **Item Sorting**: Organize chest contents
4. **Search/Filter**: Find specific items quickly

### UI Improvements
- **Item Icons**: Visual representation of items
- **Tooltips**: Detailed item information on hover
- **Categories**: Group items by type
- **Quick Actions**: Right-click context menus

### Gameplay Features
- **Chest Naming**: Player-customizable chest labels
- **Shared Storage**: Multi-player chest access
- **Chest Upgrades**: Increase capacity or add features
- **Auto-Sort**: Automatic organization options

## Testing

### DES Test Coverage
File: `tests/scenarios/chest_ui_test.json`

Tests:
- Chest presence and accessibility
- Player positioning relative to chests
- Basic interaction validation

### Manual Testing Checklist
- [ ] Can open chest when adjacent
- [ ] Cannot open chest when not adjacent
- [ ] Locked chests require correct keys
- [ ] Item transfer works in both directions
- [ ] Capacity limits are enforced
- [ ] UI navigation is responsive
- [ ] Panel switching works correctly
- [ ] Closing interface returns to game

## Integration with Existing Systems

### Micro-Structures
- Chests spawn naturally in micro-structures
- Structure-appropriate loot tables
- Contextual chest types per structure

### Save/Load System
- Chest states persist across sessions
- Item contents maintained
- Lock states preserved

### Spatial System
- Chest positions tracked in spatial index
- Efficient adjacency checking
- Integration with movement system

---

*The Chest UI system provides intuitive, keyboard-driven interaction with the game's storage mechanics while maintaining the clean, efficient TUI aesthetic of the Saltglass Steppe.*
