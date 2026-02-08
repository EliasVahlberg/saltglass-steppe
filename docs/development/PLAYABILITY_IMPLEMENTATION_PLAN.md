# Playability Implementation Plan

> **Date**: 2026-02-08
> **Goal**: Bring the game to a semi-playable state where a player can travel the world map, enter meaningfully populated tiles, and progress through quests.
> **Status**: Planning — awaiting sub-agent execution

---

## Table of Contents

1. [Current State Summary](#current-state-summary)
2. [Findings with Evidence](#findings-with-evidence)
3. [Implementation Phases](#implementation-phases)
4. [Phase 1: Fix Critical Blockers](#phase-1-fix-critical-blockers)
5. [Phase 2: Populate Tiles on Travel](#phase-2-populate-tiles-on-travel)
6. [Phase 3: POI Differentiation](#phase-3-poi-differentiation)
7. [Phase 4: Validation & Tuning](#phase-4-validation--tuning)
8. [Sub-Agent Assignment Matrix](#sub-agent-assignment-matrix)
9. [Verification Checklist](#verification-checklist)

---

## Current State Summary

### What Works

| System | Status | Notes |
|--------|--------|-------|
| World map generation | ✅ Working | 64×64 grid, biomes, terrain, POIs, roads, level scaling |
| Tile generation (terrain-forge) | ✅ Working | Cellular/BSP/rooms algorithms, biome-specific walls/floors |
| Initial tile population | ✅ Working | First tile gets microstructures, NPCs, enemies, items, chests |
| Core gameplay loop | ✅ Working | Movement, combat, inventory, equipment, crafting, quests |
| World map travel | ✅ Working | Open world map, select destination, travel |
| Save/load | ✅ Working | Full serialization roundtrip |
| DES testing | ✅ Working | 15/16 scenarios pass, 135/136 unit tests pass |
| Advanced systems | ✅ Exist | Crystal resonance, void energy, light manipulation, psychic, narrative |

### What's Broken or Missing

| # | Severity | Issue | Impact on Playability |
|---|----------|-------|----------------------|
| 1 | 🔴 Critical | Connectivity not enforced on generated tiles | Player may be trapped in disconnected map regions |
| 2 | 🔴 Critical | Feature materializer broken after terrain-forge v0.7.0 | Zero features (lights, loot, NPCs) materialize from forge markers |
| 3 | 🔴 Critical | `is_safe()` bug in `travel_to_tile_safe` | Player may spawn on walls after traveling |
| 4 | 🟠 Major | No microstructures placed on travel tiles | Every tile after the first is barren |
| 5 | 🟠 Major | No NPC spawning on travel tiles | Towns have zero merchants or dialogue NPCs |
| 6 | 🟠 Major | Structure generators not integrated | DungeonGenerator/RuinsGenerator unused in gameplay |
| 7 | 🟠 Major | `npc_spawn_config.json` is dead data | Detailed per-structure NPC rules never loaded |
| 8 | 🟡 Minor | POI-specific markers lost in v0.7.0 | Towns/shrines/dungeons don't get specialized markers |
| 9 | 🟡 Minor | 1 failing test | `feature_materializer::story_hook_materializes` |
| 10 | 🟡 Minor | World map determinism test ignored | Non-deterministic behavior needs investigation |

---

## Findings with Evidence

Each finding below includes the exact file locations and code so any sub-agent can independently verify.

### Finding 1: Connectivity Not Enforced

**Claim**: The Glass Seam Bridging connectivity system exists but is never called during tile generation.

**Evidence**:

The connectivity functions are defined in:
- `src/game/generation/connectivity.rs` — `ensure_connectivity()` at line 135, `check_connectivity()` at line 1105

But searching the entire codebase for calls to these functions:
- `ensure_connectivity` is only called from `src/tilegen_tools/constraints.rs` (a test tool)
- It is **never** called from `terrain_forge_adapter.rs` or `state.rs`

The tile generation entry point `TerrainForgeGenerator::generate_tile_with_seed()` at `src/game/generation/terrain_forge_adapter.rs:67-228` generates the grid, converts to Map, applies POI layout, extracts semantic markers — but never validates or enforces connectivity.

**Risk**: A cellular automata or BSP generation can produce maps with disconnected floor regions. The player spawns in one region but enemies, items, or quest objectives may be in unreachable regions.

---

### Finding 2: Feature Materializer Broken

**Claim**: The feature materializer cannot match terrain-forge v0.7.0 markers to the feature registry.

**Evidence**:

terrain-forge v0.7.0's `SemanticExtractor` produces markers with types from the `MarkerType` enum. The adapter maps these to strings at `src/game/generation/terrain_forge_adapter.rs:206-218`:

```rust
feature_id: match &marker.marker_type {
    terrain_forge::semantic::MarkerType::Custom(s) => s.clone(),
    terrain_forge::semantic::MarkerType::Spawn => "Spawn".to_string(),
    terrain_forge::semantic::MarkerType::Exit => "Exit".to_string(),
    terrain_forge::semantic::MarkerType::Treasure => "Treasure".to_string(),
    terrain_forge::semantic::MarkerType::BossRoom => "BossRoom".to_string(),
    terrain_forge::semantic::MarkerType::SafeZone => "SafeZone".to_string(),
    // ... etc
},
```

But `data/map_features.json` defines handlers for completely different IDs:

```json
{ "id": "light_anchor", "handler": "light" },
{ "id": "loot_slot", "handler": "loot" },
{ "id": "enemy_spawn", "handler": "enemy" },
{ "id": "npc_slot", "handler": "npc" },
{ "id": "shop_slot", "handler": "npc" },
{ "id": "altar", "handler": "interactable" },
{ "id": "boss_core", "handler": "enemy" },
{ "id": "story_hook", "handler": "story" }
```

The materializer at `src/game/generation/feature_materializer.rs:33` does:
```rust
let Some(def) = get_feature_def(&feature.feature_id) else { continue; };
```

Since `"Spawn"` ≠ `"enemy_spawn"`, `"Treasure"` ≠ `"loot_slot"`, etc., every marker is skipped. Zero features materialize.

---

### Finding 3: `is_safe()` Bug

**Claim**: The safe-spawn logic in `travel_to_tile_safe` has inverted Floor tile detection.

**Evidence** at `src/game/state.rs:1285-1294`:

```rust
let is_safe = |map: &Map, enemies: &[Enemy], x: i32, y: i32| -> bool {
    if let Some(tile) = map.get(x, y) {
        if matches!(*tile, Tile::Floor { .. }) {
            return false;  // BUG: Floor tiles return false (unsafe)
        }
    } else {
        return false;
    }
    !enemies.iter().any(|e| e.x == x && e.y == y && e.hp > 0)
};
```

The function returns `false` for Floor tiles, meaning it considers walkable floor as "unsafe". The outer code `if !is_safe(...)` then triggers a search for a "safe" position, which would match non-floor tiles (walls, glass, etc.).

**Fix**: The Floor match should `return true` (after the enemy check), or the logic should be restructured to check `!tile.walkable() { return false; }`.

---

### Finding 4: No Microstructures on Travel

**Claim**: `place_microstructures()` is only called during initial game creation, not when traveling to new tiles.

**Evidence**:

- Called at `src/game/state.rs:650` inside `GameState::new()`:
  ```rust
  place_microstructures(&mut map, biome_str, &rooms, (px, py), &mut rng);
  ```

- **Not called** anywhere in `travel_to_tile()` at `src/game/state.rs:966-1173`. Search for `place_microstructures` in that function returns zero matches.

**Impact**: The initial tile has small structures (camps, shrines, ruins fragments) with associated NPCs and chests. Every subsequent tile is just raw terrain-forge output — empty terrain with enemies scattered randomly.

---

### Finding 5: No NPC Spawning on Travel

**Claim**: `travel_to_tile()` clears all NPCs and only spawns quest-required ones.

**Evidence** at `src/game/state.rs:1120`:

```rust
self.npcs = Vec::new(); // NPCs are tile-specific
```

Then at line 1127:
```rust
self.spawn_quest_required_npcs();
```

`spawn_quest_required_npcs()` at line 1176-1204 only spawns NPCs referenced by active quest `TalkTo` objectives. It does **not** spawn biome NPCs, merchants, or town inhabitants.

Compare with `GameState::new()` at lines 487-530 which spawns the dying pilgrim and biome NPCs from `biome_spawn_tables.json`. This logic is absent from `travel_to_tile()`.

---

### Finding 6: Structure Generators Unused in Gameplay

**Claim**: `DungeonGenerator` and `RuinsGenerator` are only used in CLI test tools.

**Evidence**:

- `DungeonGenerator` is referenced in:
  - `src/game/generation/structures/dungeon_generator.rs` (definition)
  - `src/bin/dungeon-test-tool.rs` (test tool only)

- `RuinsGenerator` is referenced in:
  - `src/game/generation/structures/ruins_generator.rs` (definition)
  - `src/bin/tilegen-tool.rs` (test tool only)

Neither is called from `terrain_forge_adapter.rs`, `state.rs`, or any gameplay code path.

---

### Finding 7: `npc_spawn_config.json` Dead Data

**Claim**: The NPC spawn configuration file exists but is never loaded.

**Evidence**:

- `data/npc_spawn_config.json` contains detailed per-structure NPC rules (nexus_plateau, monastery_ascending_light, new_heliograph_station, etc.)
- Searching the codebase for `npc_spawn_config` returns zero matches in any `.rs` file

---

## Implementation Phases

### Dependency Graph

```
Phase 1 (Critical Blockers)
  ├── Task 1: Fix is_safe() bug          [no dependencies]
  ├── Task 2: Wire connectivity          [no dependencies]
  └── Task 3: Fix feature materializer   [no dependencies]
          │
Phase 2 (Populate Tiles)  ← depends on Phase 1
  ├── Task 4: Microstructures on travel  [depends on Task 2]
  ├── Task 5: NPC spawning on travel     [depends on Task 3]
  └── Task 6: Structure generators       [depends on Task 2]
          │
Phase 3 (POI Differentiation)  ← depends on Phase 2
  ├── Task 7: POI-specific markers       [depends on Task 3, 6]
  └── Task 8: POI layout improvements    [depends on Task 6]
          │
Phase 4 (Validation & Tuning)  ← depends on Phase 2+
  ├── Task 9: DES test scenarios         [depends on Phase 2]
  └── Task 10: Balance tuning            [depends on Phase 2]
```

---

## Phase 1: Fix Critical Blockers

All three tasks are independent and can be executed in parallel.

### Task 1: Fix `is_safe()` Bug

**Agent**: systems-engineer
**Files**: `src/game/state.rs`
**Estimated effort**: Small (< 30 min)

**What to do**:

Replace the `is_safe` closure in `travel_to_tile_safe()` (line ~1285) with correct logic:

```rust
let is_safe = |map: &Map, enemies: &[Enemy], x: i32, y: i32| -> bool {
    if let Some(tile) = map.get(x, y) {
        if !tile.walkable() {
            return false;
        }
    } else {
        return false;
    }
    !enemies.iter().any(|e| e.x == x && e.y == y && e.hp > 0)
};
```

**Verification**: Add a DES scenario or unit test that:
1. Creates a GameState
2. Calls `travel_to_tile_safe()` to a new tile
3. Asserts `state.map.tiles[state.map.idx(state.player_x, state.player_y)].walkable() == true`

---

### Task 2: Wire Connectivity Enforcement

**Agent**: systems-engineer
**Files**: `src/game/generation/terrain_forge_adapter.rs`, `src/game/generation/connectivity.rs`
**Estimated effort**: Medium (1-2 hours)

**What to do**:

In `TerrainForgeGenerator::generate_tile_with_seed()`, after the grid-to-map conversion loop (around line 150) and before POI layout application, add:

```rust
// Ensure map connectivity from center spawn area
use crate::game::generation::connectivity::{ensure_connectivity, GSBParams};
let spawn_pos = (MAP_WIDTH as i32 / 2, MAP_HEIGHT as i32 / 2);
ensure_connectivity(&mut map, spawn_pos, &GSBParams::fast(), &mut rng);
```

**Important considerations**:
- Use `GSBParams::fast()` to keep generation time under 100ms
- The spawn position should be the center of the map (where `find_safe_spawn_position_in_map` looks first)
- Run connectivity BEFORE POI layout application so the clearing doesn't get tunneled through

**Verification**:
```rust
#[test]
fn test_tile_connectivity_enforced() {
    use crate::game::generation::connectivity::check_connectivity;
    let gen = TerrainForgeGenerator::new();
    for seed in [42, 123, 999, 7777, 54321] {
        let (map, _) = gen.generate_tile_with_seed(
            Biome::Saltflat, Terrain::Flat, 128, POI::None, seed, &[],
        );
        let score = check_connectivity(&map, (MAP_WIDTH as i32 / 2, MAP_HEIGHT as i32 / 2));
        assert!(score >= 0.8, "Seed {} connectivity: {}", seed, score);
    }
}
```

---

### Task 3: Fix Feature Materializer

**Agent**: systems-engineer
**Files**: `data/map_features.json`, `src/game/generation/feature_materializer.rs`
**Estimated effort**: Medium (1-2 hours)

**What to do**:

Update `data/map_features.json` to include entries for terrain-forge v0.7.0 marker types:

```json
{
  "features": [
    { "id": "Spawn", "handler": "enemy", "params": { "table": "default" } },
    { "id": "Exit", "handler": "interactable", "params": { "id": "exit_marker" } },
    { "id": "Treasure", "handler": "loot", "params": { "table": "default" } },
    { "id": "BossRoom", "handler": "enemy", "params": { "table": "boss" } },
    { "id": "SafeZone", "handler": "npc", "params": { "table": "default" } },
    { "id": "EncounterZone_1", "handler": "enemy", "params": { "table": "default" } },
    { "id": "EncounterZone_2", "handler": "enemy", "params": { "table": "default" } },
    { "id": "EncounterZone_3", "handler": "enemy", "params": { "table": "boss" } },
    { "id": "LootTier_1", "handler": "loot", "params": { "table": "default" } },
    { "id": "LootTier_2", "handler": "loot", "params": { "table": "default" } },
    { "id": "LootTier_3", "handler": "loot", "params": { "table": "default" } },
    { "id": "QuestStart", "handler": "interactable", "params": { "id": "quest_marker" } },
    { "id": "QuestEnd", "handler": "interactable", "params": { "id": "quest_marker" } },
    { "id": "QuestObjective_1", "handler": "story", "params": { "kind": "quest" } },
    { "id": "QuestObjective_2", "handler": "story", "params": { "kind": "quest" } },
    { "id": "QuestObjective_3", "handler": "story", "params": { "kind": "quest" } },
    { "id": "light_anchor", "handler": "light", "params": { "table": "default" } },
    { "id": "loot_slot", "handler": "loot", "params": { "table": "default" } },
    { "id": "enemy_spawn", "handler": "enemy", "params": { "table": "default", "max_distance_from_player": 8 } },
    { "id": "npc_slot", "handler": "npc", "params": { "table": "default" } },
    { "id": "shop_slot", "handler": "npc", "params": { "table": "merchant" } },
    { "id": "altar", "handler": "interactable", "params": { "id": "altar" } },
    { "id": "boss_core", "handler": "enemy", "params": { "table": "boss" } },
    { "id": "story_hook", "handler": "story", "params": { "kind": "environmental" } }
  ]
}
```

Also fix the failing `story_hook_materializes` test in `src/game/generation/feature_materializer.rs`.

**Verification**: Run `cargo test feature_materializer` — all tests should pass.

---

## Phase 2: Populate Tiles on Travel

These tasks depend on Phase 1 being complete (especially connectivity and feature materializer).

### Task 4: Add Microstructure Placement to `travel_to_tile()`

**Agent**: systems-engineer
**Files**: `src/game/state.rs`
**Estimated effort**: Medium (1-2 hours)

**What to do**:

In `travel_to_tile()` (around line 1110, after map generation and before entity spawning), add microstructure placement:

```rust
// Place microstructures
let biome_str = biome.as_str();
let (microstructures, mut structure_npcs, mut structure_chests, mut structure_items) =
    place_microstructures(&mut map, biome_str, &walkable_positions, (px, py), &mut rng);
```

Then merge the structure entities into the tile's collections (before the `self.map = map` assignment):
- Append `structure_npcs` to `self.npcs`
- Append `structure_chests` to `self.chests`
- Append `structure_items` to `items`

**Note**: The `place_microstructures` function signature takes `rooms: &[(i32, i32)]` — for travel tiles, pass `walkable_positions` (or a sampled subset) since we don't have explicit room centers from terrain-forge.

**Verification**: Travel to 3 different tiles and verify `microstructures.len() > 0` for at least some of them.

---

### Task 5: Add NPC Spawning to `travel_to_tile()`

**Agent**: systems-engineer
**Files**: `src/game/state.rs`
**Estimated effort**: Medium (1-2 hours)

**What to do**:

After the `self.npcs = Vec::new()` line in `travel_to_tile()`, add biome NPC spawning logic similar to `GameState::new()`:

```rust
// Spawn biome NPCs
let table = get_biome_spawn_table(&biome);
for spawn in &table.npcs {
    if rng.gen_ratio(spawn.weight.min(10), 10) {
        if let Some(pos) = take_random_walkable_position(&walkable_positions, &mut rng, px, py, 10) {
            self.npcs.push(Npc::new(pos.0, pos.1, &spawn.id));
        }
    }
}

// For towns: spawn additional merchants
if matches!(poi, POI::Town) {
    // Spawn 2-3 merchants in town center area
    for merchant_id in &["glass_merchant", "water_trader", "salt_hermit"] {
        if let Some(pos) = take_random_walkable_position(&walkable_positions, &mut rng, px, py, 8) {
            self.npcs.push(Npc::new(pos.0, pos.1, merchant_id));
        }
    }
}
```

**Future improvement**: Load and use `data/npc_spawn_config.json` for structure-specific NPC rules. For now, biome spawn tables + POI-based merchant spawning is sufficient for semi-playable state.

**Verification**: Travel to a Town tile and verify `self.npcs.len() > 0`.

---

### Task 6: Integrate Structure Generators

**Agent**: systems-engineer
**Files**: `src/game/generation/terrain_forge_adapter.rs`
**Estimated effort**: Large (2-4 hours)

**What to do**:

In `generate_tile_with_seed()`, after the base terrain-forge generation, optionally overlay structure generator output for specific POI types:

```rust
// For dungeon/archive POIs, overlay DungeonGenerator output
if matches!(poi, POI::Dungeon) {
    let dungeon_gen = DungeonGenerator::with_default();
    let params = StructureParams::new(StructureType::Dungeon, (MAP_WIDTH as u32, MAP_HEIGHT as u32));
    if let Some(structure) = dungeon_gen.generate(&params, &mut rng) {
        // Apply structure rooms and corridors to the map
        apply_structure_to_map(&mut map, &structure, &floor_id, &wall_id);
    }
}
```

Similarly for `POI::Landmark` with `RuinsGenerator`.

**Important**: The structure generators produce `Structure` objects with rooms, corridors, and features. A helper function `apply_structure_to_map()` needs to be written that:
1. Carves rooms and corridors into the existing map
2. Preserves the biome-appropriate wall/floor IDs
3. Adds structure spawn points to `map.features`

**Verification**: Generate a Dungeon tile and verify it has distinct room/corridor structure rather than just cellular automata caves.

---

## Phase 3: POI Differentiation

### Task 7: POI-Specific Marker Injection

**Agent**: systems-engineer
**Files**: `src/game/generation/terrain_forge_adapter.rs`
**Estimated effort**: Medium (1-2 hours)

**What to do**:

After the `SemanticExtractor` runs and markers are collected, inject POI-appropriate custom markers:

```rust
// Inject POI-specific markers
match poi {
    POI::Town => {
        // Add npc_slot markers at SafeZone positions
        // Add shop_slot markers near map center
        inject_poi_markers(&mut map, &walkable_positions, "town", &mut rng);
    }
    POI::Shrine => {
        inject_poi_markers(&mut map, &walkable_positions, "shrine", &mut rng);
    }
    POI::Dungeon => {
        inject_poi_markers(&mut map, &walkable_positions, "dungeon", &mut rng);
    }
    _ => {}
}
```

This ensures the feature materializer can place NPCs in towns, altars in shrines, and bosses in dungeons.

---

### Task 8: POI Layout Improvements

**Agent**: content-writer + systems-engineer
**Files**: `data/terrain_config.json`
**Estimated effort**: Medium (1-2 hours)

**What to do**:

Enhance the POI layout definitions in `terrain_config.json` to create more distinctive tiles:

- **Towns**: Larger central clearing (20+), wall clusters forming "buildings", defined market area
- **Dungeons**: Multiple chambers connected by narrow corridors, dead ends
- **Shrines**: Small open area with a defined altar position, meditation paths
- **Ruins**: Partially collapsed structures, rubble patterns

The current `apply_poi_layout()` function at `terrain_forge_adapter.rs:275-317` only creates a central clearing and random wall clusters. It needs richer patterns.

---

## Phase 4: Validation & Tuning

### Task 9: DES Test Scenarios

**Agent**: qa-tester
**Files**: `tests/des_scenarios.rs`, `tests/scenarios/`
**Estimated effort**: Medium (1-2 hours)

**Write DES scenarios that verify**:

1. **Travel spawn safety**: Player always spawns on a walkable floor tile after travel
2. **Town population**: Traveling to a Town tile results in NPCs being present
3. **Dungeon population**: Traveling to a Dungeon tile results in enemies and loot
4. **Connectivity**: Player can pathfind to at least 80% of floor tiles from spawn
5. **Microstructures**: Non-initial tiles have microstructures placed
6. **Quest NPC spawning**: Active quest TalkTo NPCs appear on travel

### Task 10: Balance Tuning

**Agent**: gameplay-balancer
**Files**: `data/biome_spawn_tables.json`, `data/terrain_config.json`
**Estimated effort**: Medium (1-2 hours)

**Review and tune**:

- Enemy counts per POI type (currently hardcoded: Town=0, Shrine=1, other=4)
- NPC distribution for towns (how many merchants, what types)
- Item/loot spawning rates per biome and level
- Difficulty curve across world map distance (level scaling)

---

## Sub-Agent Assignment Matrix

| Task | Agent | Phase | Dependencies | Est. Effort |
|------|-------|-------|-------------|-------------|
| 1. Fix `is_safe()` bug | systems-engineer | 1 | None | Small |
| 2. Wire connectivity | systems-engineer | 1 | None | Medium |
| 3. Fix feature materializer | systems-engineer | 1 | None | Medium |
| 4. Microstructures on travel | systems-engineer | 2 | Task 2 | Medium |
| 5. NPC spawning on travel | systems-engineer | 2 | Task 3 | Medium |
| 6. Structure generators | systems-engineer | 2 | Task 2 | Large |
| 7. POI-specific markers | systems-engineer | 3 | Tasks 3, 6 | Medium |
| 8. POI layout improvements | content-writer + systems-engineer | 3 | Task 6 | Medium |
| 9. DES test scenarios | qa-tester | 4 | Phase 2 | Medium |
| 10. Balance tuning | gameplay-balancer | 4 | Phase 2 | Medium |

### Parallel Execution Opportunities

- **Phase 1**: Tasks 1, 2, 3 can all run in parallel (no dependencies between them)
- **Phase 2**: Tasks 4 and 5 can run in parallel after Phase 1
- **Phase 3**: Tasks 7 and 8 can run in parallel
- **Phase 4**: Tasks 9 and 10 can run in parallel after Phase 2

---

## Verification Checklist

After all phases are complete, verify:

- [ ] `cargo test` — all tests pass (0 failures)
- [ ] `cargo clippy --all-targets` — no errors
- [ ] Travel to 5 different tiles — player always spawns on floor
- [ ] Travel to a Town — NPCs present, at least 1 merchant
- [ ] Travel to a Dungeon — enemies present, rooms/corridors visible
- [ ] Travel to a Shrine — altar or interactable present
- [ ] Generate 10 random tiles — all have connectivity ≥ 0.8
- [ ] Non-initial tiles have microstructures
- [ ] Feature materializer produces entities from forge markers
- [ ] Main questline first quest (dying pilgrim) is completable

---

## Key File Reference

| File | Role |
|------|------|
| `src/game/state.rs` | GameState, `new()`, `travel_to_tile()`, `travel_to_tile_safe()` |
| `src/game/generation/terrain_forge_adapter.rs` | `TerrainForgeGenerator::generate_tile_with_seed()` |
| `src/game/generation/connectivity.rs` | `ensure_connectivity()`, `check_connectivity()` |
| `src/game/generation/feature_materializer.rs` | `materialize_features()` |
| `src/game/generation/feature_registry.rs` | `get_feature_def()`, loads `data/map_features.json` |
| `src/game/generation/microstructures.rs` | `place_microstructures()` |
| `src/game/generation/structures/dungeon_generator.rs` | `DungeonGenerator` |
| `src/game/generation/structures/ruins_generator.rs` | `RuinsGenerator` |
| `src/game/generation/spawn.rs` | `get_biome_spawn_table()` |
| `data/map_features.json` | Feature ID → handler mapping |
| `data/terrain_config.json` | POI layouts, algorithm selection |
| `data/biome_spawn_tables.json` | Per-biome enemy/NPC/item spawn tables |
| `data/npc_spawn_config.json` | Per-structure NPC rules (currently unused) |

---

_Last updated: 2026-02-08_
