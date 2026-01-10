# Tile Generation Pipeline Documentation

**Version:** 2.0  
**Date:** 2026-01-06  
**System:** Bracket-Noise Based Terrain Generation  
**Purpose:** Technical documentation for the enhanced tile generation system

---

## Overview

The tile generation system uses **bracket-noise** (Perlin noise) with multi-layered generation, constraint validation, and biome-specific modifications to create organic, varied terrain for the Saltglass Steppe.

## Pipeline Architecture

```
World Seed → Tile Seed → Multi-Layer Noise → Constraint Validation → Final Map
```

### Key Components
- **TerrainForgeGenerator**: Main generation coordinator
- **Multi-layer noise**: 4 separate Perlin noise layers
- **Constraint system**: Validation and emergency fixes
- **Biome modifiers**: Data-driven terrain customization
- **Glass Seam Bridging**: Connectivity algorithm

---

## Algorithms Used

### 1. Multi-Layer Perlin Noise Generation

| Layer | Purpose | Frequency | Range |
|-------|---------|-----------|-------|
| **Base Terrain** | Primary landscape structure | 1.0 / noise_scale | [-1, 1] |
| **Variation Layer** | Micro-variations and detail | 2.0 / noise_scale | [-1, 1] |
| **Feature Layer** | Large-scale terrain features | 0.5 / noise_scale | [-1, 1] |
| **Glass Noise** | Glass formation placement | 2.0 / noise_scale | [-1, 1] |

### 2. Glass Seam Bridging Algorithm
- **Purpose**: Ensures map connectivity
- **Method**: Modified Dijkstra's algorithm for optimal tunnel networks
- **Validation**: Coverage threshold verification
- **Integration**: Called via `ensure_connectivity()` after base generation

### 3. Constraint Validation System
- **Standard Constraints**: Connectivity, spawn safety, resource distribution
- **Quest Constraints**: Quest-specific terrain requirements
- **Emergency Fixes**: Applied on final attempt if critical constraints fail
- **Max Attempts**: 5 generation attempts before fallback

---

## Parameters and Configuration

### Terrain Configuration (`data/terrain_config.json`)

```rust
struct TerrainConfig {
    floor_threshold: f64,     // Noise threshold for floor vs wall
    glass_density: f64,       // Glass spawn probability  
    noise_scale: f64,         // Noise frequency scale
    wall_type: String,        // Wall material identifier
    floor_type: String,       // Floor material identifier
    feature_weights: HashMap, // Feature spawn weights
}
```

#### Terrain Type Values

| Terrain | Floor Threshold | Glass Density | Noise Scale | Wall Type | Floor Type |
|---------|----------------|---------------|-------------|-----------|------------|
| Canyon | -0.3 | 0.15 | 8.0 | sandstone | dry_soil |
| Mesa | -0.4 | 0.1 | 6.0 | shale | ancient_tile |
| Hills | -0.2 | 0.08 | 10.0 | sandstone | dry_soil |
| Dunes | -0.25 | 0.12 | 12.0 | sandstone | soft_sand |
| Flat | -0.1 | 0.05 | 15.0 | salt_crystal | salt_crust |

### Biome Modifiers

```rust
struct BiomeModifier {
    glass_density_multiplier: Option<f64>,  // 0.3 to 3.0
    floor_threshold_bonus: Option<f64>,     // -0.3 to 0.3
    wall_type_override: Option<String>,     // Material override
    floor_type_override: Option<String>,    // Material override
    unique_features: Option<Vec<String>>,   // Biome-specific features
}
```

#### Biome Modifier Examples

| Biome | Glass Multiplier | Threshold Bonus | Wall Override | Floor Override |
|-------|-----------------|----------------|---------------|----------------|
| Saltflat | 2.5 | +0.2 | salt_crystal | salt_crust |
| Oasis | 0.3 | +0.1 | - | dry_soil |
| Ruins | - | - | old_reinforced_concrete | ancient_tile |
| Desert | - | +0.3 | - | soft_sand |

### Global Parameters
```json
{
  "feature_density": 0.15,        // Overall feature spawn rate
  "variation_intensity": 0.3      // Noise variation strength
}
```

---

## Seeding System

### Hierarchical Seed Generation
```rust
// Primary seed from world coordinates
tile_seed = world_map.tile_seed(world_x, world_y)

// Layer-specific seeds to avoid correlation
base_seed = tile_seed.wrapping_mul(1000)
glass_seed = base_seed.wrapping_add(1)
variation_seed = base_seed.wrapping_add(2)  
feature_seed = base_seed.wrapping_add(3)
```

### Seed Sources
- **Game Runtime**: Uses `world_map.tile_seed(x, y)` for deterministic per-tile generation
- **Tools**: Accept command-line seed parameters for testing
- **Fallback**: Default seed 12345 for development/testing

### Determinism Guarantees
- Same world coordinates always produce identical terrain
- Same seed produces identical results across runs
- Independent of generation order or timing

---

## Core Generation Algorithm

### Terrain Classification Logic
```rust
for each position (x, y) {
    // Sample multi-layer noise
    base_terrain = perlin_noise(x/noise_scale, y/noise_scale)           // [-1, 1]
    variation = variation_noise(x*2/noise_scale, y*2/noise_scale) * 0.3
    terrain_value = base_terrain + variation
    
    // Dynamic threshold with feature variation
    threshold = floor_threshold + (feature_noise * 0.2)
    
    // Terrain classification
    if terrain_value > threshold {
        tile = Floor(floor_type)
        
        // Glass placement check
        glass_value = glass_noise(x*2/noise_scale, y*2/noise_scale)
        pattern_factor = calculate_glass_pattern(x, y, biome, terrain)
        
        if glass_value > (0.0 - glass_density * pattern_factor) {
            tile = Glass
        }
    } else {
        tile = Wall(wall_type, hp=100)
    }
}
```

### Glass Pattern Calculation
Biome-specific glass formation patterns:

- **Saltflat**: Crystalline formations using `sin(x/8) * cos(y/8)` patterns
- **Ruins**: Structural glass from ancient building remnants
- **Desert**: Wind-carved glass formations with directional bias
- **Scrubland**: Scattered glass shards from storm activity

---

## Constraint Evaluation

### Validation Process
```rust
const MAX_ATTEMPTS: usize = 5;

for attempt in 0..MAX_ATTEMPTS {
    let (map, clearings) = generate_base_terrain();
    
    let constraint_results = validate_all_constraints(&map, quest_ids);
    let critical_satisfied = are_critical_constraints_satisfied(&constraint_results);
    
    if critical_satisfied || attempt == MAX_ATTEMPTS - 1 {
        if !critical_satisfied && attempt == MAX_ATTEMPTS - 1 {
            apply_emergency_fixes(&mut map, &constraint_results);
        }
        return (map, clearings);
    }
}
```

### Constraint Types

#### Standard Constraints
1. **Connectivity**: Minimum percentage of floor tiles must be reachable
2. **Spawn Safety**: Safe player spawn positions must exist
3. **Resource Viability**: Resource placement areas must be accessible

#### Quest Constraints (when applicable)
1. **Quest Structure Requirements**: Specific terrain layouts for quest objectives
2. **Item Placement Validation**: Ensures quest items can be placed safely
3. **Enemy Spawn Validation**: Confirms enemy spawn areas are suitable

### Emergency Fixes
Applied when critical constraints fail on final attempt:
- **Connectivity Fixes**: Create tunnels to connect isolated areas
- **Spawn Fixes**: Clear safe areas around spawn points
- **Access Fixes**: Ensure critical areas remain reachable

---

## POI Integration

### POI-Specific Modifications

| POI Type | Central Clearing | Structure Density | Special Features |
|----------|-----------------|-------------------|------------------|
| Town | 15 tiles | 0.3 | market_square, well, watchtower |
| Ruins | 8 tiles | 0.6 | collapsed_dome, archive_fragment |
| Shrine | 6 tiles | 0.2 | altar, prayer_circle, sacred_light |
| Dungeon | 12 tiles | 0.8 | data_core, security_grid |

### Structure Integration Process
1. **Base Terrain Generation**: Create underlying landscape
2. **POI Feature Placement**: Add POI-specific structures
3. **Clearing Identification**: Find and preserve open areas
4. **Quest Structure Addition**: Place quest-specific elements when quest_ids provided

---

## Output Format

### Map Structure
```rust
pub struct Map {
    pub tiles: Vec<Tile>,                    // Generated terrain grid
    pub width: usize,                        // MAP_WIDTH (250)
    pub height: usize,                       // MAP_HEIGHT (110)
    pub lights: Vec<MapLight>,               // Light source positions
    pub inscriptions: Vec<MapInscription>,   // Text elements
    pub area_description: Option<String>,    // Flavor text
    pub metadata: HashMap<String, String>,   // Generation metadata
}
```

### Tile Types Generated
```rust
pub enum Tile {
    Wall { id: String, hp: i32 },    // Solid barriers with material type
    Floor { id: String },            // Walkable surfaces with material type  
    Glass,                           // Dangerous glass terrain
    // ... other tile types
}
```

### Metadata Tracking
Generated maps include metadata for debugging and analysis:
- `generation_seed`: Seed used for this tile
- `biome_type`: Biome identifier
- `terrain_type`: Terrain type used
- `poi_type`: POI type if applicable
- `quest_structure`: Boolean indicating quest-specific generation
- `clearings_count`: Number of open areas found
- `constraint_results`: Constraint validation summary

---

## Integration Points

### Game Integration
```rust
// Called from GameState::travel_to_tile()
let tile_seed = world_map.tile_seed(new_wx, new_wy);
let mut tile_gen = TerrainForgeGenerator::new()?;
let map = tile_gen.generate_enhanced_tile_with_structures_seeded(
    Some(poi), 
    &biome_str, 
    quest_ids, 
    tile_seed
);
```

### Tool Integration
```rust
// tilegen-tool usage
cargo run --bin tilegen-tool tile <seed> [poi] [biome]

// tilegen-test-tool usage  
cargo run --bin tilegen-test-tool --config config.json
```

### Testing Integration
- **DES Scenarios**: Can specify terrain requirements for test scenarios
- **Deterministic Testing**: Same seeds produce identical results for validation
- **Performance Testing**: Generation time and memory usage tracking

---

## Performance Characteristics

### Generation Time
- **Typical**: 10-50ms per tile on modern hardware
- **Worst Case**: 250ms with multiple constraint validation attempts
- **Optimization**: Noise sampling is the primary bottleneck

### Memory Usage
- **Base Map**: ~275KB per tile (250x110 grid)
- **Noise Generators**: ~1KB per generator (4 total)
- **Constraint Data**: ~10KB for validation structures

### Scalability
- **Linear**: Generation time scales linearly with map size
- **Parallel**: Multiple tiles can be generated concurrently
- **Caching**: Generated tiles are cached in GameState

---

## Troubleshooting

### Common Issues

#### All Floor Tiles Generated
- **Cause**: Noise range mismatch with threshold values
- **Fix**: Ensure noise stays in [-1, 1] range, thresholds are negative

#### No Glass Generated  
- **Cause**: Glass density too low or pattern factor issues
- **Fix**: Check biome glass_density_multiplier values

#### Constraint Failures
- **Cause**: Terrain too dense or isolated areas
- **Fix**: Adjust floor_threshold or enable emergency fixes

#### Determinism Broken
- **Cause**: RNG state contamination or seed issues  
- **Fix**: Verify seed propagation and RNG isolation

### Debug Tools
- **tilegen-tool**: Visual terrain inspection
- **Metadata output**: Generation parameter verification
- **Constraint logging**: Validation failure analysis

---

## Future Enhancements

### Planned Improvements
1. **Cellular Automata**: Additional generation algorithm option
2. **BSP Rooms**: Structured room-based generation for dungeons
3. **Heightmaps**: 3D terrain representation for advanced features
4. **Biome Transitions**: Smooth blending between adjacent biomes

### Configuration Expansion
1. **Per-POI Terrain**: POI-specific terrain overrides
2. **Seasonal Variation**: Time-based terrain modifications
3. **Storm Scars**: Persistent terrain changes from glass storms
4. **Player Impact**: Terrain modifications from player actions

---

*This document reflects the current implementation as of 2026-01-06. For implementation details, see `src/game/generation/tile_gen.rs` and related modules.*
