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

One file per scenario. `name` is the display label in the menu. `seed` is optional — omit to use a hash of the filename.

Provide configs for:
- All 5 biomes × common terrain types
- All POI types (Town, Dungeon, Shrine, Landmark, None)
- All 7 factions as `faction_territory`

### Main menu integration

Add a new `MenuAction::TileTest(TileTestConfig)` variant and a "Test Tile Generation" option to the main menu. Selecting it opens a sub-menu that lists all configs loaded from `data/tile_tests/`. Selecting a config starts the tester session.

```
Main Menu
├── New Game
├── Load Game
├── Test Tile Generation   ← new
│   ├── mirrormonks_town
│   ├── saltflat_dungeon
│   └── ...
├── Controls
└── Quit
```

### GameState for tester sessions

Create a minimal `GameState` using the existing new-game constructor (default player, no world map needed). Then call a new method:

```rust
impl GameState {
    pub fn load_test_tile(&mut self, params: TileParams) {
        // same as travel_to_tile but takes params directly
        // skips world map lookup
        let tile = generate_tile(&params);
        self.apply_tile(0, 0, tile);
        // same post-load hooks as travel_to_tile
    }
}
```

Add a `test_mode: bool` flag to `GameState`. When `true`:
- `handle_world_transition()` in `movement.rs` is a no-op (blocks exits)
- Save is disabled — `MenuAction::Save` returns early, autosave skipped
- A visible indicator in the HUD ("TEST MODE") so it's clear

### `main.rs` wiring

```rust
MenuAction::TileTest(config) => {
    let params = TileParams::from_test_config(&config);
    let mut state = GameState::new_default();
    state.test_mode = true;
    state.load_test_tile(params);
    run_game_loop(&mut state)?;
}
```

After the game loop exits (player quits), return to the main menu as normal.

---

## Implementation Order

1. **Refactor** `travel_to_tile` → `generate_tile()` in `tile_generator.rs`
   - Move pure generation body
   - `travel_to_tile` becomes wrapper calling `generate_tile` + `apply_tile`
   - All existing tests must pass, no behaviour change

2. **Add** `data/tile_tests/` with configs covering all biomes/POIs/factions

3. **Add** `test_mode: bool` to `GameState`, block exits + saves when set

4. **Add** `MenuAction::TileTest`, sub-menu in `menu.rs`, wiring in `main.rs`

5. **[Future]** Player save state selection — load a saved player into the test session instead of default player

---

## Files Touched

| File | Change |
|------|--------|
| `src/game/generation/tile_generator.rs` | New — core generation logic |
| `src/game/generation/mod.rs` | Add `pub mod tile_generator` |
| `src/game/state.rs` | `travel_to_tile` becomes thin wrapper, add `test_mode`, add `load_test_tile` |
| `src/game/systems/movement.rs` | No-op `handle_world_transition` when `test_mode` |
| `src/ui/menu.rs` | Add `TileTest` action + sub-menu |
| `src/main.rs` | Handle `MenuAction::TileTest`, disable save in test mode |
| `data/tile_tests/*.json` | New — test configs |
