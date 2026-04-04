# Technical Stack

## Language & Edition

**Rust 2024** (minimum 1.70+)

Chosen for:
- Determinism (strict ownership prevents hidden state bugs)
- Performance (FOV, pathfinding, storm edits, AI scheduling)
- Serialization ecosystem (serde for save/load)
- No GC pauses during gameplay
- Strong TUI libraries

## Core Dependencies

### UI & Terminal
- **ratatui** (0.28) - TUI framework for multi-panel layouts
- **crossterm** (0.28) - Cross-platform terminal backend

### Data & Serialization
- **serde** (1.0) - Serialization framework
- **ron** (0.8) - Rusty Object Notation for data files
- **serde_json** (1.0) - JSON for configs

### Procedural Generation
- **terrain-forge** (0.3.1) - 2D procedural terrain generation library
- **bracket-noise** (0.8.7) - Perlin/simplex noise
- **bracket-pathfinding** (0.8) - A* pathfinding
- **bracket-geometry** (0.8.7) - Geometric utilities
- **rand** (0.8) + **rand_chacha** (0.3) - Seeded RNG for determinism

### Utilities
- **once_cell** (1.0) - Lazy static initialization
- **rayon** (1.11) - Data parallelism
- **textwrap** (0.16) - Text wrapping for UI
- **chrono** (0.4) - Date/time for saves
- **clap** (4.0) - CLI argument parsing
- **which** (6.0) - Terminal emulator detection
- **image** (0.25) - Image processing (map exports)
- **smallvec** (1.15) - Stack-allocated vectors

## Build Configuration

### Release Profile
```toml
[profile.release]
codegen-units = 1        # Better optimization
lto = true              # Link-time optimization
opt-level = 3           # Maximum speed for TUI performance
strip = true            # Remove debug symbols
panic = "abort"         # Reduce binary size
```

### Cross-Compilation
- Linux → Windows: `x86_64-pc-windows-gnu` with mingw-w64
- Zig toolchain for multi-target releases

## Architecture

```
┌─────────────────────────────────────────┐
│         UI Layer (ratatui)              │
│  game_view, hud, menus, debug_menu      │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      Game Simulation (src/game/)        │
│  state, map, combat, quests, systems    │
│  Deterministic RNG, event queue         │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│    Renderer (src/renderer/)             │
│  tiles, entities, lighting, effects     │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      Data Layer (data/*.json)           │
│  terrain_config, factions, items, etc.  │
└─────────────────────────────────────────┘
```

## Module Organization

- **src/main.rs** - Game entry point, TUI loop
- **src/game/** - Core gameplay systems
  - `state.rs` - Main game state (171KB, central hub)
  - `map.rs` - Tile map and world map
  - `generation/` - Procedural generation
  - `systems/` - ECS-style systems
- **src/ui/** - TUI screens and menus
- **src/renderer/** - Rendering logic (tiles, entities, lighting, particles)
- **src/des/** - Debug Execution System for automated testing
- **src/bin/** - CLI tools (mapgen-tool, tilegen-tool, etc.)
- **tests/** - Integration tests and DES scenarios
- **data/** - JSON/RON data files

## Testing Infrastructure

### Debug Execution System (DES)
- Custom scripting language for reproducible gameplay scenarios
- Scenarios in `tests/scenarios/*.des`
- Commands: `spawn`, `move`, `attack`, `assert`, `wait`
- Enables automated regression testing without manual play

### Test Scripts
- `./test_all_algorithms.sh` - Algorithm regression suite
- `./test_all_configs.sh` - Config validation
- `cargo test` - Rust unit/integration tests

### CLI Tools
- `mapgen-tool` - Generate world/tile maps for validation
- `tilegen-tool` - Test tile generation algorithms
- Multi-terminal UI system for live debugging

## Performance Targets

- **TUI Rendering**: 60fps minimum
- **Map Generation**: <100ms for 250×110 tile maps
- **FOV Calculation**: <5ms per turn
- **Save/Load**: <1s for full game state

## CI/CD

- GitHub Actions for multi-platform builds
- Automated testing on push
- Release packaging with `build-release.sh`

## Development Tools

- **rustfmt** - Code formatting (4-space indent)
- **clippy** - Linting (all warnings as errors)
- **cargo-watch** - Auto-rebuild during development

## System Wiring Policy

**Read `docs/development/SYSTEM_STATUS.md` before working on any gameplay system.** This registry is the source of truth for what is wired into gameplay.

Before committing a new or modified system:
1. Confirm the full path: input action → state mutation → observable gameplay effect
2. Write a DES scenario that exercises the system through its input path (not just `player_alive`)
3. Update `docs/development/SYSTEM_STATUS.md` with honest status (✅ ⚠️ ❌)
4. Check data cross-references if modifying `data/*.json`

Do not commit batch scaffolding without proving each system is wired.
