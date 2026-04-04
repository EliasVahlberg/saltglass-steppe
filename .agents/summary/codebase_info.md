# codebase_info.md

## Project

- **Name**: Saltglass Steppe
- **Type**: Deterministic, data-driven, turn-based TUI roguelike RPG
- **Language**: Rust (Edition 2024, minimum 1.70+)

## Entry Points

| Binary | Path | Safe to Run? | Notes |
|--------|------|:---:|-------|
| `saltglass-steppe` (default) | `src/main.rs` | ❌ | Locks terminal — cannot run in agent/CI context |
| `mapgen-tool` | `src/bin/mapgen_tool.rs` | ✅ | `cargo run --bin mapgen-tool world\|tile [seed] [poi]` |
| `schema_gen` | `src/bin/schema_gen.rs` | ✅ | Regenerates `schemas/` from Rust types |

## Directory Map

| Directory | Purpose |
|-----------|---------|
| `src/game/state.rs` | Central `GameState` hub — all systems touch this |
| `src/game/systems/` | ECS-style systems: ai, combat, movement, loot, storm, status, quest |
| `src/game/generation/` | Procedural generation pipeline (terrain-forge, GSB, structures, settlement) |
| `src/ui/` | TUI screens, menus, input handling |
| `src/renderer/` | Rendering pipeline: tiles, entities, lighting, particles, effects |
| `src/des/mod.rs` | Debug Execution System interpreter |
| `data/` | JSON/RON data-driven content configs |
| `schemas/` | Auto-generated JSON schemas for data validation |
| `tests/scenarios/` | DES scenario files (JSON + `.des`) |
| `data/enemies/` | Enemy definitions split by rarity: common, uncommon, rare, elite, boss |
| `data/structures/patterns/` | Prefab structure patterns: core, ruins, special |

## Tech Stack

| Category | Crates |
|----------|--------|
| UI | ratatui 0.28, crossterm 0.28 |
| Procgen | terrain-forge 0.7.0, bracket-noise 0.8.7, bracket-pathfinding 0.8, bracket-geometry 0.8.7 |
| Data | serde 1.0, serde_json 1.0, ron 0.8, jsonschema 0.17, schemars 0.8 |
| RNG | rand 0.8, rand_chacha 0.3 (seeded determinism) |
| Utilities | clap 4.0, once_cell 1.0, rayon 1.11, image 0.25, smallvec 1.15, md5 0.8 |

## CI Pipeline

GitHub Actions (`.github/workflows/ci.yml`):

1. **test** job: `cargo build` → `cargo test` → `cargo clippy -- -D warnings` → `cargo fmt -- --check`
2. **des-scenarios** job (depends on test): `cargo test --test des_scenarios`

## Key Rules

- **Deterministic**: All RNG uses `rand_chacha` with explicit seeds. Same seed = same result.
- **No `unwrap()` in gameplay paths**: Bubble errors with context.
- **Data-driven**: Content in `data/*.json`, validated against `schemas/` at load time.
- **DES for testing**: Write DES scenarios instead of manual TUI testing. See `docs/development/DES_USAGE.md`.
- **System status registry**: Read `docs/development/SYSTEM_STATUS.md` before working on any gameplay system.
