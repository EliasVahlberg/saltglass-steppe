# Terrain Forge Spawn Integration Plan

## Objective
Leverage terrain-forge 0.3.1 semantic outputs (`SemanticExtractor` → regions/markers/masks/connectivity) to drive deterministic, data-driven spawning of items, enemies, NPCs, lights, and interactables in Saltglass Steppe without coupling generation to gameplay logic.

## Scope
- Adapter changes: read terrain-forge semantic outputs and map them into `Map.features` + metadata.
- Materialization pass: convert feature markers into runtime entities using existing spawn tables/registries.
- Tests: DES scenario(s) and unit coverage for marker-to-spawn mapping.
- Docs: update architecture/generation docs to reflect the new flow.

## Assumptions (aligned to terrain-forge 0.3.1)
- Use `SemanticExtractor::{for_rooms, for_caves, for_mazes}` (or custom `SemanticConfig`) on the generated `Grid<Tile>` to retrieve `SemanticLayers { regions, markers, masks, connectivity }`.
- `Marker` has `{x, y, tag: String, weight: f32, region_id: Option<u32>, metadata: HashMap<String,String>}`; tags are arbitrary strings we control via `SemanticConfig::marker_types`.
- `Masks` expose `walkable` and `no_spawn` bool grids to constrain placements.
- Game keeps selection logic (which enemy/item/light) in its own spawn tables keyed by marker tag + biome/poi/region tags.

## Work Breakdown
1) Adapter update (`src/game/generation/terrain_forge_adapter.rs`)
   - Use `terrain-forge = "0.3.1"` and call `SemanticExtractor` after base tile generation. Pick extractor per algo/poi: `for_rooms()` for BSP/rooms/town/archive, `for_caves()` for cellular/dungeon, `for_mazes()` for maze/ruins where appropriate. Override `SemanticConfig::marker_types` to our tags (`light_anchor`, `loot_slot`, `npc_slot`, `enemy_patrol`, `story_hook`, etc.) instead of the defaults (`PlayerStart`, `Exit`, …).
   - Map returned `markers` into `Map.features` with `{x,y,feature_id=marker.tag,source="forge_marker"}` and copy marker metadata/region_id into feature metadata (e.g., `region_kind`, `marker_weight`).
   - Optionally store region/connectivity counts in `map.metadata` for debugging/QA and hold `Masks` in a lightweight cache during materialization (no need to persist if not serialized).
   - Preserve deterministic seeding via tile seed (share the same RNG seed for extractor and materializer).

2) Feature registry (`data/map_features.json`, new)
   - Define marker tags and handlers: `light_anchor`, `loot_slot`, `npc_slot`, `enemy_patrol`, `interactable:altar`, `story_hook`, `boss_core`, etc.
   - Include optional filters (biome/poi/region kind/tags) and weights for spawn tables.
   - Allow defaults to map old-style tags (`PlayerStart`, `Exit`, `Treasure`, `Enemy`, `Furniture`, `Trap`, `Crystal`) to sensible handlers for backward compatibility.

3) Materialization pass (new module, e.g., `src/game/generation/feature_materializer.rs`)
   - Input: `&mut GameState`, `&Map.features`, biome/poi/level/quest tags, RNG, and optional `Masks` for spawn validation.
   - For each feature:
     - Resolve handler by feature id → action (spawn item/enemy/NPC/light/add interactable).
     - Use existing tables (`generation::spawn`, `loot`, `light_defs`, `interactables`) for actual picks.
     - Respect constraints: reachability (walkable mask), distance-from-player, per-type caps, density, and `no_spawn` mask from terrain-forge.
   - Emit `GameEvent` if needed (e.g., story hooks) rather than direct cross-system calls.

4) Entry points
   - Call materializer after `Map::generate_from_world_with_poi` in `GameState::new` and travel code.
   - Make materialization opt-in for tests/tools via a flag if needed.

5) Tests
   - Add a DES scenario using a deterministic seed + known marker tags; assert an expected light/interactable/item spawns at marker coords.
   - Unit test: feed synthetic `Map.features` (and optional Masks) to the materializer and assert outputs (counts/placements; mask compliance).

6) Docs
   - Expand `docs/architecture/game_systems_overview.md` with the marker → materializer → spawn flow and note `SemanticExtractor` as the source.
   - Add a short developer note in `docs/development/` on how to add new feature handlers and how to tune `SemanticConfig::marker_types`.

## Risks & Mitigations
- Missing markers in some algos → keep fallback spawn (current spawn tables) when no feature matched.
- Overcrowding/density → enforce per-type caps and mask-based sampling in the materializer.
- Backward compatibility → preserve existing generation API; gate new materializer behind a feature flag/config if needed for legacy scenarios.

## Deliverables
- Updated adapter to ingest forge semantic layers.
- New feature registry data file.
- Materializer module wired into map generation flow.
- DES + unit tests covering marker-to-spawn mapping.
- Documentation updates.
