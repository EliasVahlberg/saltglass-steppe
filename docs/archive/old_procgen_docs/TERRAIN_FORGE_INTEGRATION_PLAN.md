# Terrain Forge Integration Plan

Goal: replace all legacy tile generation paths with the external `terrain-forge` engine (https://github.com/EliasVahlberg/terrain-forge). Legacy components (e.g., `TileGenerator` in `src/game/generation/tile_gen.rs`, `tilegen-tool`, `tilegen-test-tool`, `GameState` tilegen calls) are deprecated and should be removed after migration.

## Phases

1) Dependency + bootstrap
- Add `terrain-forge = "0.1.0"` (crates.io) and expose a thin adapter module (`src/game/generation/terrain_forge_adapter.rs`). If pinning to a specific git rev, keep Cargo.toml in sync.
- Define conversion helpers between `terrain-forge` outputs and local types (`Map`, `Tile`, biome/POI enums, light/inscription where applicable).

2) Configuration + data mapping
- Map current JSON configs (`data/terrain_config.json`, wall/floor IDs, POI layouts) to terrain-forge inputs. If needed, add a translation layer that builds terrain-forge configs from existing data files to avoid duplicating content.
- Ensure RNG seeding flows through the adapter so determinism matches existing seeds.
 - Use terrain-forge `Grid<Tile>` (binary wall/floor) and post-map to game tiles: assign wall/floor IDs from `walls.json`/`floors.json`, apply biome modifiers (`terrain_config.json`), glass density, and POI clearings.
 - Preserve structures/templates: after forge generation, reapply `structure_templates.json`, `structure_spawn_config.json`, `microstructures.json`, and quest/POI constraints to stamp features.

3) Call site replacement
- `src/game/state.rs`: swap `TileGenerator::generate_enhanced_tile_with_structures_seeded` with terrain-forge calls via the adapter.
- CLI tools (`src/bin/tilegen-tool.rs`, `src/bin/tilegen-test-tool.rs`): switch to adapter, preserve CLI flags/seeds/output formats.
- Tests in `src/game/generation/tile_gen.rs` and integration tests expecting `TileGenerator` outputs: replace/port to terrain-forge outputs or remove legacy-only expectations.

4) Validation
- Rebuild existing deterministic tests: world map → tile generation → quest placement. Add adapter-specific tests to assert stable outputs for fixed seeds/POIs/biomes.
- Rework failing tests (event triggers, shrine features) to match terrain-forge semantics, or gate them until feature parity is reached.
- Manual QA: run `cargo run --bin tilegen-tool` and in-game `GameState` tile loads with multiple biomes/POIs; compare with legacy screenshots if available.

5) Cleanup
- Remove deprecated files (`tile_gen.rs`, CLI tooling) once all call sites use terrain-forge.
- Drop legacy configs that are fully mirrored in terrain-forge, or keep them only as translation sources.
- Update docs: root README run instructions, testing docs, and DES scenarios if tile layouts change.

## Current data/systems impact (what to adapt)
- Tile data: `terrain_config.json` (terrain_types, biome_modifiers, poi_layouts), `walls.json`, `floors.json`, `constraint_rules.json`, `structure_templates.json`, `structure_spawn_config.json`, `microstructures.json`, `tiles.json`, `themes.json`.
- Legacy generator: `src/game/generation/tile_gen.rs` and CLIs (`tilegen-tool`, `tilegen-test-tool`) are deprecated. `game/state.rs` calls TileGenerator directly.
- Rendering/meta: `src/game/map.rs` loads `walls/floors/terrain_config` for glyphs/colors and needs a mapping from forge grid → game Tile ids/HP/glass/glare.
- Post-processing: after forge outputs floor/wall, apply biome overrides, glass density, POI clearings, and stamp structures/microstructures/quests.

## Terrain Forge snapshot (local repo `/home/elias/Documents/my_repos/terrain-forge`)
- Core types: `Grid<Tile::{Wall, Floor}>`, `Algorithm` trait, registry `algorithms::get("bsp" | "cellular" | ... | "glass_seam")`.
- Constraints: basic connectivity/density/border validators; no game-specific semantics.
- No built-in biomes/POIs/wall IDs; requires adapter to translate from Saltglass data and back.
