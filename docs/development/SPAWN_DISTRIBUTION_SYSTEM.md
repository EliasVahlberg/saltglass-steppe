# Spawn Distribution System

**Version:** 1.0  
**Date:** 2025-12-29  
**Status:** Implemented  

## Overview

The spawn distribution system ensures that enemies, NPCs, and other entities are placed with proper spatial distribution across the game world, preventing clustering and creating more natural, tactical gameplay scenarios.

## Problem Solved

Previously, the spawn system was based on old room-based generation logic that caused enemies to cluster together in predictable patterns. This made combat less interesting and reduced the tactical depth of encounters.

## Implementation

### Core Components

#### 1. Spatial Distribution Module (`src/game/spatial.rs`)

**PoissonSampler**: Advanced spatial distribution using Poisson disk sampling
- Ensures minimum distance between spawn points
- Uses grid-based optimization for performance
- Supports configurable minimum distances

**distribute_points_grid**: Simpler grid-based distribution as fallback
- Shuffles candidate positions
- Applies minimum distance constraints
- More predictable but still effective

#### 2. Integration with Spawn System

The spawn system now uses spatial distribution in two key areas:

**Enemy Spawning** (`src/game/state.rs`):
```rust
let distributed_positions = super::spatial::distribute_points_grid(
    &safe_rooms, 
    max_enemies, 
    20, // Minimum distance between enemies
    &mut rng
);
```

**Parameters**:
- `safe_rooms`: Candidate spawn positions (clearings far from player)
- `max_enemies`: Maximum number of enemies to spawn (8)
- `min_distance`: Minimum tiles between enemies (20)
- `rng`: Seeded random number generator for determinism

### Key Features

#### Spatial Constraints
- **Player Safety Zone**: 15-tile minimum distance from player spawn
- **Enemy Separation**: 20-tile minimum distance between enemies
- **Deterministic**: Same seed produces identical spawn patterns

#### Performance Optimizations
- Grid-based spatial indexing for O(1) distance checks
- Early termination when no valid positions remain
- Configurable attempt limits to prevent infinite loops

#### Fallback Systems
- If Poisson sampling fails, falls back to grid-based distribution
- If no valid positions exist, reduces spawn count gracefully
- Maintains game balance even in constrained scenarios

## Configuration

### Spawn Parameters

| Parameter | Value | Purpose |
|-----------|-------|---------|
| `max_enemies` | 8 | Limit total enemies per tile |
| `safe_distance` | 15 | Minimum distance from player |
| `min_distance` | 20 | Minimum distance between enemies |
| `max_attempts` | 20 | Attempts to find valid position |

### Biome Integration

The system works with existing biome-based spawn tables:
- Uses `get_biome_spawn_table()` for enemy types
- Respects biome-specific spawn weights
- Maintains level-based tier restrictions

## Testing

### DES Test Scenario
File: `tests/scenarios/spawn_distribution_test.json`

Tests:
- Minimum enemy count (2+)
- Player survival
- Spatial distribution validation

### Manual Testing
1. Generate multiple tiles with same seed
2. Verify consistent enemy placement
3. Check minimum distance constraints
4. Validate player safety zone

## Benefits

### Gameplay Improvements
- **Tactical Depth**: Enemies spread across battlefield
- **Exploration**: Encourages movement and positioning
- **Balance**: Prevents overwhelming clusters
- **Predictability**: Consistent challenge level

### Technical Benefits
- **Performance**: Efficient spatial algorithms
- **Determinism**: Reproducible with seeds
- **Extensibility**: Easy to add new constraints
- **Maintainability**: Decoupled from map generation

## Future Enhancements

### Planned Improvements
1. **Advanced Patterns**: Support for formation-based enemy groups
2. **Terrain Awareness**: Consider line-of-sight and cover
3. **Dynamic Adjustment**: Adapt to player level and equipment
4. **Faction Behavior**: Different spatial patterns per faction

### Configuration Expansion
- Per-biome distance parameters
- Enemy-type specific spacing rules
- Player progression scaling
- Difficulty-based adjustments

## Integration Points

### Map Generation
- Called during `GameState::new()`
- Integrates with existing room/clearing system
- Works with both regular and POI-based generation

### Save/Load System
- Spawn positions are deterministic from seed
- No additional save data required
- Consistent across game sessions

### Debug Tools
- Compatible with existing debug spawn commands
- Maintains spatial constraints in debug mode
- Supports manual enemy placement

## Code Examples

### Basic Usage
```rust
use super::spatial::distribute_points_grid;

let positions = distribute_points_grid(
    &candidate_positions,
    max_count,
    min_distance,
    &mut rng
);
```

### Advanced Poisson Sampling
```rust
use super::spatial::PoissonSampler;

let mut sampler = PoissonSampler::new(width, height, min_distance);
let positions = sampler.sample_points(&candidates, max_count, &mut rng);
```

## Performance Characteristics

### Time Complexity
- Grid-based: O(n²) where n = candidate positions
- Poisson sampling: O(n) average case
- Spatial indexing: O(1) distance checks

### Memory Usage
- Grid storage: O(w×h) where w,h = map dimensions
- Position tracking: O(n) where n = spawn count
- Minimal overhead for typical map sizes

## Troubleshooting

### Common Issues

**No enemies spawning**:
- Check candidate position count
- Verify minimum distance constraints
- Ensure player safety zone allows spawns

**Enemies still clustering**:
- Increase `min_distance` parameter
- Check spatial algorithm selection
- Verify grid cell size calculation

**Performance issues**:
- Reduce `max_attempts` parameter
- Use grid-based instead of Poisson sampling
- Optimize candidate position filtering

---

*This system provides the foundation for tactical, well-distributed encounters while maintaining the deterministic, data-driven architecture of the game.*
