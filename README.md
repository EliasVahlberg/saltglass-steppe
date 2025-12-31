# Saltglass Steppe


A deterministic, data-driven, turn-based TUI RPG set in the harsh, crystalline wastelands of the Saltglass Steppe. Built with Rust, it features tactical combat, environmental survival mechanics, and a custom Debug Execution System (DES) for automated scenario testing.

## Documentation

📚 **[Complete Documentation](docs/)** - Comprehensive project documentation

### Quick Links
- **[Architecture](docs/architecture/)** - Technical architecture and system design
- **[Design](docs/design/)** - Game design and creative vision
- **[Development](docs/development/)** - Development guides and tools
- **[Features](docs/features/)** - Feature specifications and implementation
- **[Narrative](docs/narrative/)** - World lore and storytelling
- **[Testing](docs/testing/)** - QA procedures and guidelines
- **[Document Database](docs/DOCUMENT_DATABASE.md)** - Complete document listing

## Tech Stack

- **Language**: Rust
- **UI Framework**: `ratatui`
- **Terminal Backend**: `crossterm`
- **Data Serialization**: `serde`, `serde_json`
- **RNG**: `rand_chacha` (Seeded for full determinism)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

## Setup

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build
```

## Run

```bash
cargo run
```

## Multi-Terminal UI System

The game supports running across multiple terminal windows for enhanced gameplay experience:

### Manual Launch
```bash
# Start main game
cargo run

# In separate terminals:
cargo run -- --log-ui      # Real-time game log display
cargo run -- --status-ui   # Player stats and status
cargo run -- --inventory-ui # Inventory display (placeholder)
```

### Automatic Spawn (from game)
Press `` ` `` to open debug console, then:
- `spawn log` - Spawns log terminal
- `spawn status` - Spawns status terminal  
- `spawn inventory` - Spawns inventory terminal
- `terminals` - Lists available terminal emulators

**Supported Terminals**: gnome-terminal, konsole, xterm, alacritty, kitty

**Features**:
- Real-time IPC communication via Unix domain sockets
- Automatic terminal emulator detection
- Non-blocking updates to prevent game lag
- JSON-serialized message format for reliability

## Testing Map Generation

A dedicated tool is available for testing world and tile generation without running the full game:

```bash
# Generate and display a world map with default seed (12345)
cargo run --bin mapgen-tool world

# Generate and display a world map with custom seed
cargo run --bin mapgen-tool world 42

# Generate and display a tile map with default seed (12345)
cargo run --bin mapgen-tool tile

# Generate and display a tile map with custom seed
cargo run --bin mapgen-tool tile 42

# Generate tile map with specific Point of Interest (POI)
cargo run --bin mapgen-tool tile 42 town     # Town with central clearing
cargo run --bin mapgen-tool tile 42 shrine   # Shrine with smaller clearing
cargo run --bin mapgen-tool tile 42 landmark # Landmark (ruins) with structures
cargo run --bin mapgen-tool tile 42 dungeon  # Dungeon (archive) with chambers
```

The tool displays ASCII representations of the generated maps:

- **World Map**: Shows biomes, terrain, and points of interest across a 192x64 grid
- **Tile Map**: Shows detailed organic terrain using Perlin noise generation (250x110 grid)
  - Uses data-driven terrain configuration for different biomes and terrain types
  - Generates natural, flowing landscapes instead of geometric rooms
  - Supports POI-specific features like central clearings and structures
  - Glass shards are placed naturally based on noise patterns

### Terrain Generation Features

The new noise-based tile generation creates organic, Saltglass Steppe-appropriate landscapes:

- **Biome-specific terrain**: Different wall types and glass densities based on biome
- **Terrain variety**: Canyon, Mesa, Hills, Dunes, and Flat terrain with unique characteristics
- **POI integration**: Towns, Shrines, Landmarks, and Dungeons add specific features
- **Data-driven configuration**: Easy to modify terrain parameters via `data/terrain_config.json`
- **Deterministic generation**: Same seed always produces identical results

Use different seeds to test various generation patterns and ensure deterministic behavior.

## QA / Debug Commands

During gameplay, you can open the debug console by pressing `/`. The following commands are available to assist with testing:

| Command     | Description                                                        |
| ----------- | ------------------------------------------------------------------ |
| `show tile` | Enables "God View", revealing the entire map and all entities.     |
| `hide tile` | Disables "God View", returning to normal line-of-sight visibility. |
| `sturdy`    | Sets player HP to 9999/9999 (God Mode).                            |
| `phase`     | Toggles "Phase Mode", allowing movement through walls.             |
| `help`      | Lists available debug commands in the game log.                    |

## Cross-compile for Windows (from Linux)

```bash
# Install Windows target and cross-compiler
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64

# Build Windows executable
cargo build --release --target x86_64-pc-windows-gnu

# Package for distribution
mkdir dist
cp target/x86_64-pc-windows-gnu/release/saltglass-steppe.exe dist/
cp -r data dist/
zip -r saltglass-steppe-windows.zip dist
```

The tester should extract the zip and run `saltglass-steppe.exe` from Command Prompt (not by double-clicking).
