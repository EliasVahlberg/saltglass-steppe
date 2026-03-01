# Tile Generation Refactor + Tile-Gen Tester

**Status**: Planned  
**Prerequisite for**: Tile-gen testing feature, easier generation iteration

---

## Problem

`travel_to_tile()` in `state.rs` (~240 lines) does two unrelated things:
1. **Generates** a tile (map + enemies + NPCs + items + settlement)
2. **Applies** the result to `GameState` and runs post-load hooks

Generation logic belongs in `src/game/generation/`, not in `state.rs`. This makes generation untestable without a full `GameState`, and is why the tile tester doesn't exist yet.

---

## Part 1: Refactor — Extract `generate_tile()`

### New file: `src/game/generation/tile_generator.rs`

```rust
pub struct TileParams {
    pub seed: u64,
    pub biome: Biome,
    pub terrain: Terrain,
    pub elevation: u8,
    pub poi: POI,
    pub level: u32,
    pub faction_control: Vec<(String, f32)>,
    pub quest_ids: Vec<String>,
}

pub struct GeneratedTile {
    pub map: Map,
    pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub chests: Vec<Chest>,
    pub spawn_pos: Option<(i32, i32)>,  // None = caller picks
}

pub fn generate_tile(params: &TileParams) -> GeneratedTile { ... }
```

Move the pure generation body of `travel_to_tile` here:
- `TerrainForgeGenerator::generate_tile_with_seed()`
- Enemy spawning
- NPC spawning (including settlement NPCs)
- Item/chest placement
- Settlement generation + stamping + decorations

### `travel_to_tile()` becomes a thin wrapper

```rust
pub fn travel_to_tile(&mut self, new_wx: usize, new_wy: usize) {
    let params = TileParams::from_world_state(self, new_wx, new_wy);
    let tile = generate_tile(&params);
    self.apply_tile(new_wx, new_wy, tile);
    // post-load hooks that need &mut self stay here:
    // materialize_features, spawn_crafting_stations,
    // spawn_quest_required_npcs, generate_narrative_fragments,
    // generate_biome_content, generate_crystal_formations,
    // generate_template_content, update_fov, rebuild_spatial_index, etc.
}
```

### What stays in `state.rs`
Everything that reads or writes `self` after the tile is applied:
- `materialize_features()`
- `spawn_crafting_stations()`
- `spawn_quest_required_npcs()`
- `generate_narrative_fragments()`
- `generate_biome_content()`
- `generate_crystal_formations()`
- `generate_template_content()`
- `update_fov()`, `rebuild_spatial_index()`, `update_lighting()`

These are post-load hooks, not generation — they stay where they are.

### Risk
`travel_to_tile` reads from `self` in a few places during generation:
- `self.get_quest_ids_for_location(wx, wy)` → becomes a parameter in `TileParams`
- `self.find_safe_spawn_position_in_map(&map)` → moves into `generate_tile`, returns `spawn_pos`
- `self.world.storm.intensity` (for template context) → stays in post-load, not generation

Mitigation: extract incrementally, run `cargo test` after each move.

---

## Part 2: Tile-Gen Tester

### Test configs: `data/tile_tests/*.json`

```json
{
  "name": "mirrormonks_town",
  "biome": "Saltflat",
  "terrain": "Flat",
  "elevation": 100,
  "poi": "Town",
  "level": 1,
  "faction_territory": "MirrorMonks",
  "seed": 12345
}
```

One file per scenario. Filenames are the test name. Seed is optional — omit to use a default.

Provide configs for:
- All 5 biomes × common terrain types (flat, canyon, dunes)
- All POI types (Town, Dungeon, Shrine, Landmark, None)
- All 7 factions as `faction_territory`
- Edge cases: no faction, high elevation, level 10

### CLI: `mapgen-tool tile-test [name] [--seed N]`

```
cargo run --bin mapgen-tool tile-test                  # list all configs
cargo run --bin mapgen-tool tile-test mirrormonks_town # run one
cargo run --bin mapgen-tool tile-test mirrormonks_town --seed 99999
```

Output per run:
1. ASCII map render (same as existing tile command)
2. Entity summary:
   ```
   NPCs (4): merchant @ (12,8), guard @ (15,9), ...
   Enemies (3): glass_crawler @ (40,20), ...
   Items (2): salt_shard @ (30,15), ...
   Chests (1): @ (55,30)
   ```
3. Settlement summary (if POI=Town):
   ```
   Settlement: Town, 12 buildings, faction=MirrorMonks
     [0] mirror_monks_light_temple @ (8,6)
     ...
   ```

### In-game tester (later, lower priority)

A menu option "Test Tile Generation" → config picker → calls `generate_tile(&params)` → loads result into `GameState` as if you traveled there. Lets you walk around and interact. Deferred until CLI version is validated.

---

## Implementation Order

1. **Refactor** `travel_to_tile` → `generate_tile()` in `tile_generator.rs`
   - Move pure generation body
   - `travel_to_tile` becomes wrapper
   - All existing tests must pass
   - No behaviour change

2. **Add** `data/tile_tests/` with ~15 configs covering all biomes/POIs/factions

3. **Add** `mapgen-tool tile-test` command using `generate_tile()` directly

4. **[Later]** In-game config picker

---

## Files Touched

| File | Change |
|------|--------|
| `src/game/generation/tile_generator.rs` | New — core generation logic |
| `src/game/generation/mod.rs` | Add `pub mod tile_generator` |
| `src/game/state.rs` | `travel_to_tile` becomes thin wrapper |
| `src/bin/mapgen_tool.rs` | Add `tile-test` subcommand |
| `data/tile_tests/*.json` | New — test configs |
