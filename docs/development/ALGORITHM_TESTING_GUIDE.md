# Algorithm Testing and Configuration Guide

This guide explains how to test and configure the procedural map generation algorithms in Saltglass Steppe.

## Quick Start

### Run All Algorithm Tests
```bash
./test_all_algorithms.sh
```

### Run Specific Test Categories
```bash
./test_all_algorithms.sh categories    # Room-based, Organic, Advanced
./test_all_algorithms.sh individual    # Individual algorithm tests
./test_all_algorithms.sh comprehensive # All algorithms combined
```

### Test Individual Algorithms
```bash
cargo test drunkard_walk_algorithm_test_suite
cargo test simple_rooms_algorithm_test_suite
cargo test maze_algorithm_test_suite
cargo test voronoi_algorithm_test_suite
cargo test wave_function_collapse_test_suite
```

## Available Algorithms

### 1. Room-Based Algorithms

#### BSP (Binary Space Partitioning)
**Best for**: Classic dungeons, structured layouts
```json
{
  "algorithm": "bsp",
  "algorithm_params": {
    "min_room_size": [4, 4],
    "max_room_size": [12, 8],
    "corridor_width": 1,
    "max_depth": 5,
    "split_ratio_min": 0.3,
    "split_ratio_max": 0.7
  }
}
```

#### Simple Rooms
**Best for**: Quick dungeon generation, basic layouts
```json
{
  "algorithm": "simple_rooms",
  "algorithm_params": {
    "num_rooms": 8,
    "min_room_size": [4, 4],
    "max_room_size": [10, 8],
    "corridor_width": 1,
    "max_placement_attempts": 100,
    "room_spacing": 2
  }
}
```

#### Maze Generation
**Best for**: Labyrinthine structures, puzzle layouts
```json
{
  "algorithm": "maze",
  "algorithm_params": {
    "cell_size": 3,
    "wall_thickness": 1,
    "algorithm": "recursive_backtracking", // or "kruskal", "prim"
    "add_loops": true,
    "loop_probability": 0.1
  }
}
```

### 2. Organic/Cave Algorithms

#### Cellular Automata
**Best for**: Natural caves, organic chambers
```json
{
  "algorithm": "cellular_automata",
  "algorithm_params": {
    "initial_wall_probability": 0.45,
    "iterations": 5,
    "survival_threshold": 4,
    "birth_threshold": 5,
    "use_moore_neighborhood": true
  }
}
```

#### Drunkard's Walk
**Best for**: Winding tunnels, organic pathways
```json
{
  "algorithm": "drunkard_walk",
  "algorithm_params": {
    "num_walkers": 3,
    "steps_per_walker": 400,
    "change_direction_chance": 0.1,
    "spawn_new_walker_chance": 0.05,
    "max_walkers": 6,
    "carve_radius": 1
  }
}
```

#### Voronoi Diagrams
**Best for**: Territory-based maps, biome boundaries
```json
{
  "algorithm": "voronoi",
  "algorithm_params": {
    "num_sites": 20,
    "relaxation_iterations": 2,
    "cell_type_distribution": {
      "floor": 0.7,
      "wall": 0.2,
      "special": 0.1
    },
    "border_thickness": 1
  }
}
```

### 3. Advanced Algorithms

#### Wave Function Collapse
**Best for**: Complex rule-based generation
```json
{
  "algorithm": "wave_function_collapse",
  "algorithm_params": {
    "tile_size": 3,
    "overlap": 1,
    "max_iterations": 1000,
    "entropy_heuristic": "minimum_entropy",
    "pattern_weights": {
      "floor": 0.6,
      "wall": 0.3,
      "door": 0.1
    }
  }
}
```

## Hybrid Algorithms

### BSP + Cellular Automata
Combines structured rooms with organic interiors:
```json
{
  "algorithm": "hybrid_bsp_cellular",
  "algorithm_params": {
    "primary_algorithm": "bsp",
    "secondary_algorithm": "cellular_automata",
    "bsp_params": { /* BSP parameters */ },
    "cellular_params": { /* Cellular parameters */ },
    "blend_mode": "room_interiors",
    "cellular_room_probability": 0.6
  }
}
```

### Voronoi + Drunkard's Walk
Creates regions connected by organic paths:
```json
{
  "algorithm": "hybrid_voronoi_drunkard",
  "algorithm_params": {
    "primary_algorithm": "voronoi",
    "secondary_algorithm": "drunkard_walk",
    "voronoi_params": { /* Voronoi parameters */ },
    "drunkard_params": { /* Drunkard parameters */ },
    "connection_mode": "inter_region",
    "connection_probability": 0.7
  }
}
```

## Sequential Multi-Algorithm

Apply algorithms in sequence for layered complexity:
```json
{
  "algorithm": "sequential_noise_maze_rooms",
  "algorithm_params": {
    "stage_1": {
      "algorithm": "perlin_noise",
      "params": { /* Noise parameters */ }
    },
    "stage_2": {
      "algorithm": "maze",
      "params": { /* Maze parameters */ },
      "apply_to": "high_elevation_areas"
    },
    "stage_3": {
      "algorithm": "simple_rooms",
      "params": { /* Room parameters */ },
      "apply_to": "maze_dead_ends"
    },
    "blending_mode": "layered_priority",
    "priority_order": ["rooms", "maze", "noise_terrain"]
  }
}
```

## Parameter Tuning Guidelines

### Room-Based Algorithms
- **Room sizes**: 4x4 to 12x8 for playable spaces
- **Corridor widths**: 1-3 tiles for navigation
- **Max depth (BSP)**: 3-6 for balanced complexity

### Organic Algorithms
- **Cellular iterations**: 3-7 for good results
- **Initial probability**: 0.35-0.55 for varied density
- **Walker steps**: 200-1000 depending on map size

### Advanced Algorithms
- **WFC tile size**: 3-5 for good pattern recognition
- **Voronoi sites**: 10-30 for region variety
- **Relaxation iterations**: 1-3 for natural boundaries

## Testing Configurations

### Create New Algorithm Config
```bash
./generate_algorithm_configs.sh my_algorithm 6001
```

### Test Single Configuration
```bash
cargo run --bin tilegen-test-tool -- --config enhanced-tile-test-suite/configs/my_config.json
```

### Batch Test Multiple Configs
```bash
cargo test my_algorithm_test_suite
```

## Output Analysis

### Generated Files
- **PNG Images**: `enhanced-tile-test-suite/pngs/`
- **Text Maps**: `enhanced-tile-test-suite/text/`
- **Evaluations**: `enhanced-tile-test-suite/evaluations/`
- **Reports**: `enhanced-tile-test-suite/TEST_REPORT_*.md`

### Quality Metrics
- **Connectivity**: Percentage of reachable floor tiles
- **Variety**: Distribution of different terrain types
- **Complexity**: Structural complexity score
- **Performance**: Generation time and memory usage

### Visual Analysis
Each PNG shows:
- **Base terrain** in grayscale
- **Structures** in color overlays
- **Connectivity paths** highlighted
- **Quality scores** in filename

## Best Practices

### Algorithm Selection
1. **Dungeons**: BSP or Simple Rooms
2. **Caves**: Cellular Automata + connectivity post-processing
3. **Mazes**: Maze algorithms with loop addition
4. **Natural areas**: Voronoi + Drunkard's Walk
5. **Complex layouts**: Wave Function Collapse or hybrids

### Parameter Tuning
1. Start with default parameters
2. Test with different seeds for consistency
3. Adjust one parameter at a time
4. Use visual output to guide changes
5. Validate connectivity after changes

### Performance Optimization
1. Use simpler algorithms for real-time generation
2. Pre-generate complex layouts
3. Cache results for repeated patterns
4. Profile generation time for large maps

### Debugging
1. Enable all output layers for analysis
2. Use text output for detailed inspection
3. Check evaluation metrics for quality issues
4. Compare with reference configurations

## Troubleshooting

### Common Issues
- **No connectivity**: Increase corridor width or walker steps
- **Too sparse**: Reduce wall probability or increase iterations
- **Too dense**: Increase wall probability or reduce iterations
- **Generation fails**: Check parameter ranges and constraints

### Performance Issues
- **Slow generation**: Reduce iterations or map size
- **Memory usage**: Limit number of sites/rooms
- **Infinite loops**: Set max iterations for iterative algorithms

### Quality Issues
- **Boring layouts**: Add parameter variation or hybrid approaches
- **Unrealistic caves**: Tune cellular automata thresholds
- **Disconnected areas**: Add connectivity post-processing

## Advanced Usage

### Custom Algorithm Implementation
1. Create algorithm in `src/game/generation/structures/algorithms/`
2. Add to `mod.rs` exports
3. Implement in `tilegen-test-tool`
4. Create test configurations
5. Add to test suites

### Algorithm Combinations
1. Define hybrid parameters
2. Implement blending logic
3. Test with various blend modes
4. Validate output quality

### Biome-Specific Tuning
1. Create biome-specific configs
2. Adjust parameters for theme
3. Test with appropriate POI types
4. Validate thematic consistency

---

*For more detailed information, see [ALGORITHM_CATEGORIES.md](ALGORITHM_CATEGORIES.md) and the generated test reports.*
