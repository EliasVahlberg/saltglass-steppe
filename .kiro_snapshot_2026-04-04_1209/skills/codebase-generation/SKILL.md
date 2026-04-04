---
name: codebase-generation
description: Procedural generation systems — terrain-forge adapter, world generation, biome system, spawn tables, and all generation modules. Use when working on map generation, adding new biome content, modifying spawn tables, or extending the generation pipeline.
---

# Codebase: Generation Systems

**Location**: `src/game/generation/` (20+ modules)

> ⚠️ Read terrain-forge API documentation before starting any generation work. Many algorithms may already be implemented in the library.

## Core Entry Points

### Tile Generation — `terrain_forge_adapter.rs`

```rust
pub struct TerrainForgeGenerator;

impl TerrainForgeGenerator {
    pub fn generate_tile_with_seed(
        &self,
        biome: Biome,
        terrain: Terrain,
        elevation: Elevation,
        poi: POI,
        seed: u64,
        quest_ids: &[String],
    ) -> (Map, Vec<(i32, i32)>)  // (map, room_centers)
}
```

Uses `terrain_forge` crate (`terrain_forge::Grid`, `SemanticExtractor`, `ops`). Config from `data/terrain_config.json`.

**Biome-driven algorithm selection**: each biome has weighted algorithm preferences (cellular, BSP, rooms) with per-terrain overrides. POI types have separate algorithm overrides that take priority.

### World Map Generation — `world_gen.rs`

```rust
pub struct WorldGenerator;
// Generates WorldMap with layers: biome, terrain, elevation, resources, POIs
// Uses noise + wave function collapse for layers 1-3
// Procedural placement for layers 4-5 (POIs penalize proximity to spread exploration)
```

### Spawn System — `spawn.rs`

```rust
pub fn get_biome_spawn_table(biome: &Biome) -> SpawnTable
// Returns SpawnTable { enemies: Vec<SpawnEntry>, npcs: Vec<SpawnEntry>, items: Vec<SpawnEntry> }

pub fn weighted_pick_by_level_and_tier(
    entries: &[SpawnEntry],
    level: u32,
    rng: &mut ChaCha8Rng,
    allow_boss: bool,
) -> Option<String>

pub fn distribute_points_grid(
    positions: &[(i32, i32)],
    count: usize,
    min_distance: i32,
    rng: &mut ChaCha8Rng,
) -> Vec<(i32, i32)>
```

Data: `data/biome_spawn_tables.json`, `data/npc_spawn_config.json`

### Loot Generation — `loot.rs`

```rust
pub fn generate_loot(table: &str, x: i32, y: i32, rng: &mut ChaCha8Rng) -> Vec<Item>
```

Data: `data/loot_tables.json`

## All Generation Modules

| Module | File | Purpose |
|--------|------|---------|
| `TerrainForgeGenerator` | `terrain_forge_adapter.rs` | Tile generation via terrain-forge crate |
| `WorldGenerator` | `world_gen.rs` | World map generation |
| `SpawnSystem` | `spawn.rs` | Biome-based entity spawning |
| `LootGeneration` | `loot.rs` | Procedural loot from tables |
| `BiomeSystem` | `biomes.rs` | Biome environmental content (stub — not re-implemented) |
| `ConnectivitySystem` | `connectivity.rs` | Glass Seam Bridging — ensures map connectivity |
| `MicroStructures` | `microstructures.rs` | Mini-structure placement |
| `FeatureMaterializer` | `feature_materializer.rs` | Converts terrain-forge markers to game entities |
| `FeatureRegistry` | `feature_registry.rs` | Registry of feature types |
| `Grammar` | `grammar.rs` | Dynamic text generation (stub) |
| `NarrativeIntegration` | `narrative.rs` | Story fragment placement (stub) |
| `NarrativeTemplates` | `narrative_templates.rs` | Template-based narrative (stub) |
| `StorySystem` | `story.rs` | Procedural story generation (stub) |
| `EventSystem` | `events.rs` | Dynamic events (stub — not re-implemented) |
| `ConstraintSystem` | `constraints.rs` | Constraint validation |
| `QuestConstraints` | `quest_constraints.rs` | Quest-driven generation constraints |
| `AlgorithmRegistry` | `algorithm.rs` + `algorithms/` | Plugin system for generation algorithms |
| `SpatialSystem` | `spatial.rs` | Poisson disk sampling, spatial distribution |
| `WeightedTable` | `weighted_table.rs` | Generic weighted random selection |
| `StructureGenerators` | `structures/` | BSP, Cellular Automata, dungeon generators |
| `BracketAdapter` | `adapters/` | bracket-lib integration layer |

## Connectivity System (Glass Seam Bridging)

Ensures generated maps have one connected open area:
1. Flood-fill to identify disconnected floor regions
2. Build connectivity graph with tunnel costs (Manhattan distance + wall-breaking penalty)
3. Find minimum-cost tunnel set via modified Dijkstra's
4. Create tunnels to connect regions
5. Validate coverage threshold (configurable in `data/constraint_rules.json`)

## Feature Materializer

`feature_materializer.rs` — runs after tile generation to convert terrain-forge `SemanticExtractor` markers into game entities:
- Feature IDs → interactables (`data/interactables.json`)
- Feature IDs → props/loot (via spawn system)
- Feature IDs → narrative hooks

This keeps generation deterministic and decoupled from runtime systems.

## Microstructures

Small procedural structures placed within generated tiles. Defined in `data/microstructures.json` and `data/structure_spawn_config.json`.

```rust
pub fn place_microstructures(
    map: &mut Map,
    biome: &str,
    rooms: &[(i32, i32)],
    player_pos: (i32, i32),
    rng: &mut ChaCha8Rng,
) -> (Vec<PlacedMicroStructure>, Vec<Npc>, Vec<Chest>, Vec<Item>)
```

## Data Files

| File | Used By | Status |
|------|---------|--------|
| `terrain_config.json` | `terrain_forge_adapter.rs` | **Active** — current tile generation config |
| `biome_spawn_tables.json` | `spawn.rs` | Active |
| `loot_tables.json` | `loot.rs` | Active |
| `microstructures.json` | `microstructures.rs` | Active |
| `constraint_rules.json` | `constraints.rs` | Active |
| `biome_profiles.json` | `biomes.rs` | **Suspect** — may overlap terrain_config biome_modifiers |
| `structure_generation.json` | `tilegen-tool.rs` only | **Suspect** — only used by CLI test tool |

> ⚠️ Data file audit pending. See ROADMAP.md.

## Testing Tools

```bash
# Test terrain generation
cargo run --bin tilegen-tool tile 12345 town desert
cargo run --bin tilegen-tool tile 12345 shrine saltflat

# Run enhanced evaluation
cargo run --bin tilegen-test-tool -- --config test_config.json

# Generate PNG visualization
cargo test enhanced_tile_generation_test_suite
```

## Adding New Biome Content

1. Add spawn entries to `data/biome_spawn_tables.json`
2. Add algorithm weights to `data/terrain_config.json` biome section
3. Add microstructure definitions to `data/microstructures.json` if needed
4. Write DES scenario to test generation

## Adding New Generation Algorithm

1. Create struct implementing `GenerationAlgorithm` trait in `generation/algorithms/`
2. Register in `AlgorithmRegistry`
3. Reference by name in `data/terrain_config.json`
