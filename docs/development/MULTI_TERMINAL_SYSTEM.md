# Multi-Terminal UI System Implementation

## Overview

The multi-terminal UI system allows the TUI RPG to run across multiple terminal windows, with the main game in one terminal and satellite terminals displaying specific UI components (log, status, inventory).

## Architecture

### IPC Communication (`src/ipc.rs`)
- **Unix Domain Sockets**: Uses `/tmp/saltglass-steppe.sock` for inter-process communication
- **Message Types**: GameState updates, LogEntry messages, InventoryUpdate, Commands
- **Server/Client Model**: Main game runs IPC server, satellite terminals connect as clients

### Command Line Interface (`src/cli.rs`)
- **Launch Modes**: `--log-ui`, `--status-ui`, `--inventory-ui`
- **Argument Parsing**: Uses `clap` crate for clean CLI interface

### Terminal Spawning (`src/terminal_spawn.rs`)
- **Multi-Terminal Support**: gnome-terminal, konsole, xterm, alacritty, kitty
- **Automatic Detection**: Uses `which` crate to find available terminal emulators
- **Debug Commands**: `spawn <type>` and `terminals` commands

### Satellite UI Components (`src/satellite.rs`)
- **Log UI**: Real-time game log display with scrolling
- **Status UI**: Player stats, HP, refraction, adaptations, storm countdown
- **Inventory UI**: Placeholder for future inventory display

## Usage

### Manual Launch
```bash
# Start main game
cargo run

# In separate terminals:
cargo run -- --log-ui
cargo run -- --status-ui  
cargo run -- --inventory-ui
```

### Automatic Spawn (from game)
```
Press ` to open debug console, then:
spawn log      # Spawns log terminal
spawn status   # Spawns status terminal  
spawn inventory # Spawns inventory terminal
terminals      # Lists available terminal emulators
```

## Technical Details

### IPC Message Flow
1. Main game sends GameState updates after each player action
2. Log messages are sent in real-time as they occur
3. Satellite terminals poll for messages and update their displays
4. All communication is JSON-serialized over Unix domain sockets

### Error Handling
- Graceful fallback if no terminal emulators found
- Connection retry logic for satellite terminals
- Socket cleanup on game exit

### Performance
- Minimal overhead: IPC only sends updates when game state changes
- Efficient message format using serde JSON serialization
- Non-blocking communication to prevent game lag

## Files Modified
- `src/ipc.rs` - IPC communication layer
- `src/cli.rs` - Command-line argument parsing
- `src/satellite.rs` - Satellite terminal UI components
- `src/terminal_spawn.rs` - Terminal spawning functionality
- `src/main.rs` - Integration with main game loop
- `src/game/state.rs` - Debug commands for terminal spawning
- `Cargo.toml` - Added dependencies (clap, which)

## Testing
- DES test scenario: `tests/multi_terminal_ipc.des`
- Tests debug commands and IPC message flow
- Verifies CLI argument parsing works correctly

## Future Enhancements
- Inventory synchronization for satellite inventory UI
- Bidirectional commands (satellite terminals can send actions to main game)
- Configuration file for terminal preferences
- Network support for remote terminals
