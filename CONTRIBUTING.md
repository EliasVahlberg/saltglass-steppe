# Contributing to Saltglass Steppe

## Development Setup

1. Install [Rust](https://www.rust-lang.org/tools/install) (1.70+)
2. Clone the repository
3. `cargo build` to verify setup
4. `cargo test` to run the test suite

## Code Style

- **rustfmt** with defaults (4-space indent, trailing commas)
- **clippy** with all warnings as errors
- snake_case for files/functions, UpperCamelCase for types, SCREAMING_SNAKE for constants
- Avoid `unwrap()` in gameplay paths — bubble errors with context
- Public APIs require rustdoc comments

Run before committing:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing

### Running Tests
```bash
cargo test                          # All tests
cargo test --test des_scenarios     # DES scenarios only
./test_all_algorithms.sh            # Procgen regression (before generation changes)
./test_all_configs.sh               # Config validation
```

### Writing DES Scenarios
New gameplay features require DES scenarios in `tests/scenarios/`. See `docs/development/DES_USAGE.md` for full syntax.

Key conventions:
- Inherit from `BASE_*` scenarios when possible (BASE_combat, BASE_empty_room, etc.)
- Use `ai_disabled: true` for deterministic combat tests
- Use mocks (`combat_always_hit`, `combat_fixed_damage`) for predictable outcomes
- Include `at_end: true` assertions for final state checks
- Use descriptive filenames: `<feature>_test.json`

### Determinism
All RNG uses `rand_chacha` with explicit seeds. Tests must use fixed seeds for reproducibility.

## Data File Conventions

All game content lives in `data/*.json`, validated against `schemas/*_v1.json`.

When adding content:
1. Use snake_case for IDs
2. Validate JSON: `jq . data/file.json`
3. Cross-reference dependencies:
   - New items → update `traders.json`, `loot_tables.json`, `recipes.json` as needed
   - New enemies → update `biome_spawn_tables.json`, `loot_tables.json`
   - New NPCs → update `dialogues.json`, `quests.json`
4. Keep lore tone consistent (salt, glass, storms, mutations — avoid modern slang)
5. Run `cargo run --bin schema_gen` after changing Rust data types

## Procedural Generation

- Always use deterministic seeds in tests
- Run `./test_all_algorithms.sh` before committing generation changes
- Document algorithm parameters in `docs/development/PROCEDURAL_GENERATION_COMPREHENSIVE_GUIDE.md`
- Validate generated maps meet connectivity and playability requirements

## Commit Guidelines

Follow Conventional Commits:
- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation
- `chore:` — maintenance

Keep messages imperative and scoped: `feat: add storm hazard timers`

## Pull Request Checklist

- [ ] `cargo fmt --all && cargo clippy --all-targets --all-features` passes
- [ ] `cargo test` passes
- [ ] Relevant `./test_all_*` scripts pass (if applicable)
- [ ] DES scenarios added/updated for gameplay changes
- [ ] Data files validated with `jq`
- [ ] Documentation updated in `docs/` for system changes
- [ ] No `unwrap()` in gameplay paths
- [ ] Public API has documentation comments
- [ ] Deterministic seeds used in all tests

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `src/game/` | Core gameplay systems |
| `src/game/generation/` | Procedural generation pipeline |
| `src/game/systems/` | ECS-style systems (AI, combat, movement, etc.) |
| `src/ui/` | TUI screens and menus |
| `src/renderer/` | Rendering pipeline |
| `src/des/` | Debug Execution System |
| `data/` | JSON game content (~45 files) |
| `schemas/` | JSON Schema validation (~49 files) |
| `tests/scenarios/` | DES test scenarios (~120+ files) |
| `docs/` | Project documentation |

## Key Files

- `src/game/state.rs` — Central GameState (largest file, all systems touch it)
- `src/des/mod.rs` — DES interpreter for automated testing
- `data/biome_profiles.json` — Drives procedural generation algorithm selection
- `data/terrain_config.json` — Terrain generation parameters

## Important Notes

- `cargo run` (default binary) **locks the terminal** for TUI rendering — use DES for testing
- `cargo run --bin mapgen-tool` is safe to run for map generation testing
- The `state.rs` file is intentionally large — it's the coordination point for all systems
- All data files are validated at load time against JSON schemas
