# Saltglass Steppe

A deterministic, data-driven, turn-based TUI RPG set in the harsh, crystalline wastelands of the Saltglass Steppe. Built with Rust, it features tactical combat, environmental survival mechanics, glass storms that physically rewrite maps, and a custom Debug Execution System (DES) for automated scenario testing.

## Documentation

📚 **[Complete Documentation](docs/)** — Comprehensive project documentation

### Quick Links
- **[Architecture](docs/architecture/)** — Technical architecture and system design
- **[Design](docs/design/)** — Game design and creative vision
- **[Development](docs/development/)** — Development guides, roadmap, codebase health audit
- **[Features](docs/features/)** — Feature specifications and implementation
- **[Narrative](docs/narrative/)** — World lore and storytelling
- **[Testing](docs/testing/)** — QA procedures and guidelines
- **[Document Database](docs/DOCUMENT_DATABASE.md)** — Complete document listing
- **[System Status](docs/development/SYSTEM_STATUS.md)** — Source of truth for system wiring status

### Research Papers
- **[Glass Seam Bridging Algorithm](docs/papers/glass_seam_bridging_paper.pdf)** — Novel algorithm for procedural map connectivity

## Tech Stack

- **Language**: Rust (Edition 2024)
- **UI Framework**: `ratatui` 0.28
- **Terminal Backend**: `crossterm` 0.28
- **Terrain Generation**: `terrain-forge` 0.7.0
- **Pathfinding/Noise**: `bracket-pathfinding` 0.8, `bracket-noise` 0.8.7
- **Data Serialization**: `serde`, `serde_json`, `ron`
- **RNG**: `rand` 0.8 + `rand_chacha` 0.3 (seeded for full determinism)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

## Setup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo build
```

## Run

```bash
cargo run
```

## Architecture

The codebase uses the **VERA (Verified Effect-Rule Architecture)** pattern: pure rule functions return Effect enums, a mechanical apply function mutates state, and traces record what happened for verification.

```
Input → Command → Rule(ctx, rng) → Vec<Effect> → apply(state) → Trace
```

See [AGENTS.md](AGENTS.md) for the full architecture guide and [FINAL_ARCHITECTURE.md](docs/development/architecture_refactor/FINAL_ARCHITECTURE.md) for the design document.

## Multi-Terminal UI System

The game supports running across multiple terminal windows for enhanced gameplay:

```bash
# Main game
cargo run

# In separate terminals:
cargo run -- --log-ui        # Real-time game log
cargo run -- --status-ui     # Player stats and status
cargo run -- --inventory-ui  # Inventory display
```

Features: real-time IPC via Unix domain sockets, automatic terminal emulator detection, non-blocking updates.

**Supported Terminals**: gnome-terminal, konsole, xterm, alacritty, kitty

## Testing Map Generation

```bash
# World map (default seed 12345)
cargo run --bin mapgen-tool world

# World map with custom seed
cargo run --bin mapgen-tool world 42

# Tile map with POI
cargo run --bin mapgen-tool tile 42 town     # Town with central clearing
cargo run --bin mapgen-tool tile 42 shrine   # Shrine with smaller clearing
cargo run --bin mapgen-tool tile 42 landmark # Landmark (ruins) with structures
cargo run --bin mapgen-tool tile 42 dungeon  # Dungeon (archive) with chambers
```

### Terrain Generation

Noise-based tile generation creates organic, biome-appropriate landscapes:

- **Biome-specific terrain**: Different wall types and glass densities per biome
- **Terrain variety**: Canyon, Mesa, Hills, Dunes, and Flat with unique characteristics
- **POI integration**: Towns, Shrines, Landmarks, and Dungeons add specific features
- **Data-driven configuration**: Modify parameters via `data/terrain_config.json`
- **Deterministic**: Same seed always produces identical results

## QA / Debug Commands

Press `/` during gameplay to open the debug console:

| Command     | Description                                                        |
| ----------- | ------------------------------------------------------------------ |
| `show tile` | Enables "God View", revealing the entire map and all entities      |
| `hide tile` | Disables "God View", returning to normal line-of-sight visibility  |
| `sturdy`    | Sets player HP to 9999/9999 (God Mode)                             |
| `phase`     | Toggles "Phase Mode", allowing movement through walls              |
| `help`      | Lists available debug commands in the game log                     |

## Testing

```bash
cargo test                        # All tests
cargo test --test des_scenarios   # DES scenario tests only
```

DES (Debug Execution System) enables headless, deterministic gameplay testing via JSON scenarios in `tests/scenarios/`. See [AGENTS.md](AGENTS.md#testing-with-des) for scenario format and examples.

## Cross-compile for Windows (from Linux)

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```

## License

See [LICENSE](LICENSE)
