# Codebase Information

## Project Identity

- **Name**: Saltglass Steppe
- **Type**: Deterministic, data-driven, turn-based TUI roguelike RPG
- **Language**: Rust (Edition 2024)
- **License**: See LICENSE file

## Repository Structure

```
saltglass-steppe/
├── src/                    # Rust source code
│   ├── main.rs             # TUI game loop entry point
│   ├── lib.rs              # Library exports + integration tests
│   ├── cli.rs              # CLI argument parsing
│   ├── session.rs          # Game session orchestration
│   ├── ipc.rs              # Multi-terminal IPC (Unix domain sockets)
│   ├── satellite.rs        # Satellite terminal UI processes
│   ├── game/               # Core gameplay (~50 modules)
│   │   ├── state.rs        # Central GameState hub
│   │   ├── effects/        # VERA effect enums, apply, context, trace
│   │   ├── rules/          # VERA pure rule functions
│   │   ├── systems/        # ECS-style systems (ai, combat, movement, loot, storm, status)
│   │   ├── generation/     # Procedural generation pipeline
│   │   └── [40+ domain modules]
│   ├── ui/                 # TUI screens and input handling (~20 menus)
│   ├── renderer/           # Rendering: tiles, entities, lighting, particles
│   ├── des/                # Debug Execution System interpreter
│   └── bin/                # CLI tools (mapgen-tool, schema_gen)
├── data/                   # 40 JSON data files (game content)
├── schemas/                # 36 JSON schemas (auto-generated)
├── tests/
│   ├── scenarios/          # 153 DES scenario files (JSON)
│   └── des_scenarios.rs    # DES test runner
├── docs/                   # Extensive project documentation
│   ├── architecture/       # System design docs
│   ├── design/             # Game design docs
│   ├── development/        # Dev guides, SYSTEM_STATUS.md, VERA refactor docs
│   ├── features/           # Feature specifications
│   └── narrative/          # World lore
└── .github/workflows/      # CI pipeline
```

## Key Metrics

- **Packages**: 1 (monorepo with 2 binaries)
- **Primary language**: Rust (100%)
- **Data files**: 40 JSON content files, 36 schemas
- **Test scenarios**: 153 DES scenarios + ~200 unit tests
- **Core dependencies**: 27 (see dependencies.md)

## Architecture Pattern

The codebase is migrating to **VERA (Verified Effect-Rule Architecture)**:
- Pure rule functions return Effect enums
- Mechanical apply functions mutate state
- Traces record what happened for verification
- Legacy imperative methods coexist during migration

**Source of truth for system wiring status**: `docs/development/SYSTEM_STATUS.md`

## Build & Run

| Command | Purpose | Agent-safe? |
|---------|---------|-------------|
| `cargo build` | Compile | ✅ |
| `cargo test` | All tests | ✅ |
| `cargo test --test des_scenarios` | DES scenarios only | ✅ |
| `cargo clippy -- -D warnings` | Lint | ✅ |
| `cargo run --bin mapgen-tool` | Map generation tool | ✅ |
| `cargo run` | Launch game TUI | ❌ Locks terminal |
