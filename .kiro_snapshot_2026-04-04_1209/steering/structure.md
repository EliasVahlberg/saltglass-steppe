# Code Structure & Organization

## Repository Layout

```
saltglass-steppe/
├── src/
│   ├── main.rs              # Game entry point, TUI loop
│   ├── lib.rs               # Library exports
│   ├── game/                # Core gameplay systems
│   │   ├── state.rs         # Main game state (171KB central hub)
│   │   ├── map.rs           # Tile map logic
│   │   ├── world_map.rs     # Overworld map
│   │   ├── generation/      # Procedural generation
│   │   ├── systems/         # ECS-style systems
│   │   └── [30+ modules]    # Combat, quests, items, etc.
│   ├── ui/                  # TUI screens and menus
│   ├── renderer/            # Rendering logic
│   │   ├── mod.rs           # Main renderer
│   │   ├── tiles.rs         # Tile rendering
│   │   ├── entities.rs      # Entity rendering
│   │   ├── lighting.rs      # Dynamic lighting
│   │   ├── particles.rs     # Particle effects
│   │   └── [10+ modules]    # Themes, animations, effects
│   ├── des/                 # Debug Execution System
│   │   └── mod.rs           # DES interpreter (80KB)
│   ├── bin/                 # CLI tools
│   │   ├── mapgen_tool.rs   # Map generation CLI
│   │   └── [5+ tools]       # Testing utilities
│   └── tilegen_tools/       # Procedural generation utilities
├── tests/                   # Integration tests
│   └── scenarios/           # DES test scenarios
├── data/                    # Data-driven configs
│   ├── terrain_config.json  # Terrain generation params
│   └── [configs]            # Factions, items, etc.
├── docs/                    # Comprehensive documentation
│   ├── architecture/        # System design
│   ├── design/              # Game design
│   ├── development/         # Dev guides
│   ├── features/            # Feature specs
│   ├── narrative/           # Lore and story
│   ├── testing/             # QA procedures
│   └── DOCUMENT_DATABASE.md # Doc index
├── releases/                # Release artifacts
├── .kiro/                   # Kiro-CLI configuration
│   ├── agents/              # Agent configs
│   └── steering/            # Steering files
└── [scripts]                # Build and test scripts
```

## Module Organization

### src/game/ (Core Systems)

**Central Hub**:
- `state.rs` (171KB) - Main `GameState` struct, turn loop, event handling

**Map & Generation**:
- `map.rs` - Tile map, pathfinding, FOV
- `world_map.rs` - Overworld navigation
- `generation/` - Procedural generation (world, tiles, structures)

**Gameplay Systems**:
- `combat.rs`, `combat_actions.rs` - Combat mechanics
- `quest.rs` - Quest system
- `item.rs`, `equipment.rs`, `inventory` - Item management
- `npc.rs`, `enemy.rs`, `dialogue.rs` - NPCs and interactions
- `skills.rs`, `progression.rs` - Character advancement
- `storm.rs`, `light.rs`, `lighting.rs` - Environmental systems
- `sanity.rs`, `adaptation.rs`, `void_energy.rs` - Mutation systems
- `ritual.rs`, `psychic.rs`, `crystal_resonance.rs` - Special mechanics
- `trading.rs`, `crafting.rs` - Economy systems

**Testing & Debug**:
- `des_testing.rs` - DES integration
- `qa_tools.rs` - Debug commands

### src/ui/ (TUI Screens)

- `game_view.rs` - Main game viewport
- `hud.rs` - HUD overlay
- `menu.rs` - Main menu
- `inventory_menu.rs`, `skills_menu.rs`, `trade_menu.rs` - Menus
- `debug_menu.rs` - Debug console
- `input.rs` - Input handling (33KB)
- `world_map.rs` - Overworld UI
- `[10+ menus]` - Quest log, wiki, book reader, etc.

### src/renderer/ (Rendering)

- `mod.rs` - Main renderer orchestration
- `tiles.rs` - Tile rendering with themes
- `entities.rs` - Entity rendering
- `lighting.rs` - Dynamic lighting system
- `particles.rs` - Particle effects
- `animations.rs` - Animation system
- `effects.rs`, `procedural.rs` - Visual effects
- `themes.rs` - Color themes
- `config.rs` - Renderer configuration

### src/des/ (Debug Execution System)

- `mod.rs` (80KB) - DES interpreter, scenario execution

### src/bin/ (CLI Tools)

- `mapgen_tool.rs` - Generate world/tile maps
- `tilegen-tool.rs` - Test tile generation
- `bsp-test.rs`, `cellular-automata-test.rs` - Algorithm tests

## Naming Conventions

### Files & Modules
- **snake_case**: `game_state.rs`, `world_map.rs`
- Mirror folder structure: `src/game/systems/` → `mod systems`

### Types
- **UpperCamelCase**: `GameState`, `TileType`, `EntityKind`
- Enums for variants: `BiomeType::SaltFlats`

### Functions & Variables
- **snake_case**: `update_game_state()`, `player_pos`
- Descriptive names: `calculate_fov()` not `calc_fov()`

### Constants
- **SCREAMING_SNAKE_CASE**: `MAX_INVENTORY_SIZE`, `DEFAULT_SEED`

## Code Style

### Formatting
- **rustfmt** with defaults (4-space indent)
- Trailing commas in multi-line lists
- Max line length: 100 chars (flexible)

### Error Handling
- Avoid `unwrap()` in gameplay paths
- Use `Result<T, E>` with context
- Bubble errors with `?` operator
- Log errors to game log

### Documentation
- Public APIs require rustdoc comments
- Module-level docs explain purpose
- Complex algorithms get inline comments

### Testing
- Unit tests in `#[cfg(test)]` modules
- Integration tests in `tests/`
- DES scenarios for gameplay testing
- Descriptive test names: `test_storm_rotates_room_90_degrees()`

## Data-Driven Design

### Configuration Files (data/)
- **JSON** for configs: `terrain_config.json`
- **RON** for complex data: `meta_progress.ron`
- Validate on load, fail fast with clear errors

### Content Separation
- Game logic in Rust
- Content (items, factions, dialogue) in data files
- Balance tuning without recompilation

## Build & Development

### Commands
- `cargo build` - Debug build
- `cargo run` - Run game (locks terminal)
- `cargo run --bin mapgen-tool` - Run CLI tool
- `cargo test` - Run tests
- `cargo fmt --all` - Format code
- `cargo clippy --all-targets --all-features` - Lint

### Scripts
- `./test_all_algorithms.sh` - Algorithm regression
- `./test_all_configs.sh` - Config validation
- `./build-release.sh <version>` - Multi-platform release

## Documentation

### docs/ Structure
- **architecture/** - System design, tech stack
- **design/** - Game concept, mechanics, creative direction
- **development/** - Implementation guides, DES usage
- **features/** - Feature specifications
- **narrative/** - Lore, quests, world-building
- **testing/** - QA procedures, feedback requests
- **DOCUMENT_DATABASE.md** - Complete doc index

### Documentation Standards
- Markdown for all docs
- Keep docs updated with code changes
- Link between related docs
- Use diagrams for complex systems
