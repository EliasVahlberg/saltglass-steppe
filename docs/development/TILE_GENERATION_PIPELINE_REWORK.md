# Tile Generation Pipeline Rework

**Date:** 2026-01-06  
**Status:** In Progress  
**Priority:** High  

## Overview

Rework the tile generation pipeline to use existing, proven implementations instead of custom algorithms. This will reduce development time, debugging effort, and leverage optimized, well-tested code.

## Selected Libraries

**Primary: bracket-lib (0.8.7)**
- Comprehensive roguelike toolkit
- Pathfinding (A*, Dijkstra), FOV, noise generation
- Designed for deterministic gameplay
- Well-documented and maintained

**Secondary: mapgen (0.6.0)**
- Filter-based map generation pipeline
- BSP, cellular automata, noise generators
- Good for specific algorithm implementations

## Implementation Plan

### Phase 1: Library Integration & Evaluation (1-2 days)

**Task 1.1: Add bracket-lib dependency**
```toml
[dependencies]
bracket-pathfinding = "0.8.7"
bracket-noise = "0.8.7"
bracket-algorithm-traits = "0.8.7"
```

**Task 1.2: Create adapter layer**
- Implement `BaseMap`, `Algorithm2D` traits for our `Map` struct
- Create `src/game/generation/adapters/bracket_adapter.rs`
- Minimal integration to test compatibility

**Task 1.3: DES test bracket-lib integration**
- Create test scenario validating basic pathfinding works
- Verify FOV calculations match expectations
- Document compatibility issues

### Phase 2: Replace Core Algorithms (2-3 days)

**Task 2.1: Replace pathfinding systems**
- Replace custom A* with `bracket_pathfinding::a_star_search`
- Replace Dijkstra with `bracket_pathfinding::DijkstraMap`
- Update AI system to use bracket-lib pathfinding

**Task 2.2: Replace FOV system**
- Replace custom FOV with `bracket_pathfinding::field_of_view`
- Update renderer to use bracket FOV results
- Maintain existing visibility mechanics

**Task 2.3: DES test core algorithms**
- Validate pathfinding produces same results
- Test FOV accuracy and performance
- Ensure deterministic behavior with seeded RNG

### Phase 3: Map Generation Overhaul (3-4 days)

**Task 3.1: Evaluate mapgen vs bracket-lib for generation**
- Test both libraries with our requirements
- Compare: BSP, Cellular Automata, Noise generation
- Choose best fit for each algorithm type

**Task 3.2: Replace BSP implementation**
- Use bracket-lib or mapgen BSP instead of custom
- Maintain data-driven configuration
- Preserve existing room/corridor logic

**Task 3.3: Replace Cellular Automata**
- Integrate library cellular automata
- Keep existing birth/death rule configuration
- Maintain Moore/Von Neumann neighborhood options

**Task 3.4: Replace Perlin noise**
- Use `bracket-noise` for terrain generation
- Maintain existing biome-specific parameters
- Keep deterministic seeded generation

### Phase 4: Post-Processing Pipeline (2-3 days)

**Task 4.1: Create post-processing framework**
- Design pipeline for Saltglass-specific modifications
- Glass storm effects, connectivity validation
- POI placement and biome-specific features

**Task 4.2: Integrate Glass Seam Bridging**
- Keep existing connectivity algorithm
- Apply as post-processing step after library generation
- Maintain coverage threshold validation

**Task 4.3: Preserve unique features**
- Storm map editing capabilities
- Biome-specific terrain generation
- Narrative fragment placement

### Phase 5: Integration & Optimization (2-3 days)

**Task 5.1: Update TileGenerator**
- Refactor to use library algorithms + post-processing
- Maintain existing data-driven configuration
- Keep POI-specific generation logic

**Task 5.2: Performance optimization**
- Profile new vs old performance
- Optimize hot paths (FOV, pathfinding)
- Ensure no regression in generation speed

**Task 5.3: Comprehensive DES testing**
- Test all generation scenarios
- Validate deterministic behavior
- Ensure no gameplay regressions

### Phase 6: Documentation & Cleanup (1 day)

**Task 6.1: Update documentation**
- Document new library dependencies
- Update architecture diagrams
- Create migration guide

**Task 6.2: Remove deprecated code**
- Clean up old custom implementations
- Update imports and dependencies
- Final code review and cleanup

## Implementation Principles

1. **Adapter Pattern**: Create thin adapters to integrate libraries
2. **Gradual Migration**: Replace one system at a time
3. **Preserve Uniqueness**: Keep Saltglass-specific features
4. **Data-Driven**: Maintain JSON configuration
5. **Post-Processing**: Libraries for base generation, custom for game-specific

## Work Process

For each task:
1. Implement
2. DES test
3. [If fail] troubleshoot/fix/test
4. [If working] Document
5. Commit

## Risk Mitigation

- Keep old implementations until new ones are fully tested
- Benchmark each replacement for performance
- Extensive determinism validation
- Small, incremental commits

## Current Status

- [✓] Phase 1: Library Integration & Evaluation
- [✓] Phase 2: Replace Core Algorithms  
- [ ] Phase 3: Map Generation Overhaul
- [ ] Phase 4: Post-Processing Pipeline
- [ ] Phase 5: Integration & Optimization
- [ ] Phase 6: Documentation & Cleanup

### Phase 2 Complete ✅

**Accomplished:**
- Replaced custom FOV implementation with bracket-lib's optimized field_of_view
- Replaced custom BFS pathfinding in constraints system with bracket-lib's A*
- Added idx_to_pos method to Map for index-to-position conversion
- Updated GameState::update_fov() to use bracket-lib directly
- Maintained backward compatibility with existing player_fov field
- All bracket-lib integration tests passing
- Clean build with no compilation errors

**Technical Details:**
- FOV now uses bracket-lib's field_of_view algorithm via compute_fov()
- Constraint validation uses A* pathfinding instead of custom BFS
- Removed unused is_tile_walkable method and cleaned up imports
- Added proper idx_to_pos conversion method to Map struct

**Next: Begin Phase 3 - Map Generation Overhaul**
