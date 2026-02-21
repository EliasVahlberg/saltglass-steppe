# Procedural Generation Restructure Analysis

## Current State

### Already in Generation Module ✅
- `biomes.rs` - Biome-specific content generation
- `constraints.rs` - Constraint-based generation system
- `events.rs` - Dynamic event generation
- `grammar.rs` - Text generation system
- `microstructures.rs` - Mini-structure placement (just moved)
- `narrative.rs` - Narrative fragment generation
- `pipeline.rs` - Generation pipeline coordination
- `templates.rs` - Template-based content generation
- `tile_gen.rs` - Enhanced tile generation
- `weighted_table.rs` - Weighted selection system
- `world_gen.rs` - Enhanced world generation

### Systems That Should Be Moved 🔄

#### 1. **Spawn System** (`src/game/spawn.rs`)
**Current Location**: `src/game/spawn.rs`
**Recommended**: Move to `src/game/generation/spawn.rs`

**Rationale**:
- Contains weighted spawn table logic
- Handles biome-specific entity spawning
- Uses procedural selection algorithms
- Already imports from generation module (`WeightedTable`)
- Core procedural generation functionality

**Functions**:
- `weighted_pick()` - Weighted random selection
- `weighted_pick_by_level_and_tier()` - Level-based spawn selection
- `get_biome_spawn_table()` - Biome-specific spawn tables
- Enhanced versions using `WeightedTable`

#### 2. **Spatial Distribution System** (`src/game/spatial.rs`)
**Current Location**: `src/game/spatial.rs`
**Recommended**: Move to `src/game/generation/spatial.rs`

**Rationale**:
- Implements Poisson disk sampling for spatial distribution
- Used for procedural placement of entities/features
- Core algorithm for procedural generation
- No dependencies on game logic, pure generation utility

**Functions**:
- `PoissonSampler` - Spatial distribution algorithm
- `distribute_points_grid()` - Grid-based point distribution

#### 3. **Loot Generation System** (`src/game/loot.rs`)
**Current Location**: `src/game/loot.rs`
**Recommended**: Move to `src/game/generation/loot.rs`

**Rationale**:
- Contains procedural loot generation logic
- Uses weighted tables and RNG for content creation
- Already imports from generation module (`WeightedTable`)
- Pure generation functionality

**Functions**:
- `generate_loot()` - Procedural loot generation
- `generate_loot_enhanced()` - Enhanced loot generation
- Loot table management

#### 4. **Narrative Generation System** (`src/game/narrative.rs`)
**Current Location**: `src/game/narrative.rs`
**Recommended**: Move to `src/game/generation/narrative_templates.rs` or merge with existing `narrative.rs`

**Rationale**:
- Contains narrative template system
- Procedural text generation
- Template-based content creation
- Overlaps with existing `generation/narrative.rs`

**Note**: This might need merging rather than moving to avoid duplication.

#### 5. **Story Generation System** (`src/game/story.rs`)
**Current Location**: `src/game/story.rs`
**Recommended**: Move to `src/game/generation/story.rs`

**Rationale**:
- Procedural story and character generation
- Dynamic relationship and event generation
- Uses RNG for procedural content creation
- Pure generation functionality

### Systems That Should Stay 🏠

#### 1. **World Map** (`src/game/world_map.rs`)
**Keep in current location**

**Rationale**:
- Core game data structure
- Contains world state, not just generation
- Used throughout the game for world queries
- Generation logic already moved to `generation/world_gen.rs`

#### 2. **Storm System** (`src/game/systems/storm.rs`)
**Keep in current location**

**Rationale**:
- Game system that reacts to events
- Map transformation logic, not generation
- Part of the systems architecture
- Uses generation but isn't generation itself

#### 3. **State Generation** (`src/game/state.rs`)
**Keep in current location**

**Rationale**:
- Core game state management
- Uses generation systems but isn't a generation system
- Central coordination point for the game

## Recommended Restructure Plan

### Phase 1: Move Core Generation Systems
1. Move `spawn.rs` to `generation/spawn.rs`
2. Move `spatial.rs` to `generation/spatial.rs`  
3. Move `loot.rs` to `generation/loot.rs`

### Phase 2: Consolidate Narrative Systems
1. Analyze overlap between `narrative.rs` and `generation/narrative.rs`
2. Merge or reorganize to avoid duplication
3. Move `story.rs` to `generation/story.rs`

### Phase 3: Update Imports and Dependencies
1. Update all import statements
2. Update module exports
3. Ensure no circular dependencies
4. Test all functionality

## Benefits of Restructuring

### 1. **Improved Organization**
- All procedural generation in one module
- Clear separation of concerns
- Easier to find and maintain generation code

### 2. **Better Reusability**
- Generation systems can be easily shared
- Cleaner API for generation functionality
- Reduced coupling between systems

### 3. **Enhanced Maintainability**
- Single location for generation logic
- Consistent patterns across generation systems
- Easier to add new generation features

### 4. **Performance Benefits**
- Potential for shared caching
- Better optimization opportunities
- Reduced import overhead

## Risk Assessment

### Low Risk ✅
- `spatial.rs` - Pure utility, no game dependencies
- `loot.rs` - Well-defined interface, limited usage

### Medium Risk ⚠️
- `spawn.rs` - Used in multiple places, needs careful import updates
- `story.rs` - May have dependencies on game state

### High Risk ⚡
- `narrative.rs` - Potential duplication with existing generation/narrative.rs
- Import chain updates - Need to ensure no circular dependencies

## Implementation Priority

1. **High Priority**: `spawn.rs`, `spatial.rs`, `loot.rs`
   - Core generation systems
   - Clear benefits
   - Relatively low risk

2. **Medium Priority**: `story.rs`
   - Good candidate for generation module
   - Needs dependency analysis

3. **Low Priority**: `narrative.rs`
   - Requires careful analysis of existing duplication
   - May need merging rather than moving

## Conclusion

Moving the spawn, spatial, and loot systems to the generation module would significantly improve the organization and maintainability of the procedural generation systems. The narrative systems require more careful analysis due to potential duplication.

The restructure should be done incrementally, starting with the lowest-risk, highest-benefit systems first.
