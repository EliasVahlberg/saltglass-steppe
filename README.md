# tui-rpg
A simple TUI RPG.

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

## Cross-compile for Windows (from Linux)

```bash
# Install Windows target and cross-compiler
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64

# Build Windows executable
cargo build --release --target x86_64-pc-windows-gnu

# Package for distribution
mkdir dist
cp target/x86_64-pc-windows-gnu/release/tui-rpg.exe dist/
cp -r data dist/
zip -r tui-rpg-windows.zip dist
```

The tester should extract the zip and run `tui-rpg.exe` from Command Prompt (not by double-clicking).
