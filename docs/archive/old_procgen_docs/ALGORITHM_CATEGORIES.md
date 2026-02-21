# Procedural Map Generation Algorithm Categories

This document categorizes the available procedural map generation algorithms by their characteristics and use cases.

## Algorithm Categories

### 1. Room-Based Algorithms
**Characteristics**: Generate discrete rooms connected by corridors
**Best for**: Dungeons, buildings, structured layouts

| Algorithm | Complexity | Connectivity | Organic Feel | Performance |
|-----------|------------|--------------|--------------|-------------|
| **BSP (Binary Space Partitioning)** | Medium | Guaranteed | Low | High |
| **Simple Rooms** | Low | Guaranteed | Low | High |
| **Maze Generation** | Medium | Perfect | Low | Medium |

#### BSP (Binary Space Partitioning)
- **Use Case**: Classic dungeon layouts with varied room sizes
- **Strengths**: Guaranteed connectivity, no overlapping rooms, efficient
- **Weaknesses**: Can feel artificial, limited organic variation
- **Parameters**: `min_room_size`, `max_room_size`, `corridor_width`, `max_depth`, `split_ratio`

#### Simple Rooms
- **Use Case**: Basic room-and-corridor dungeons, quick generation
- **Strengths**: Fast, simple, predictable layouts
- **Weaknesses**: Can have overlapping rooms, less sophisticated
- **Parameters**: `num_rooms`, `room_size_range`, `corridor_width`, `room_spacing`

#### Maze Generation
- **Use Case**: Labyrinthine structures, puzzle-like layouts
- **Strengths**: Perfect connectivity, multiple algorithms available
- **Weaknesses**: Can be too uniform, limited room variation
- **Algorithms**: Recursive Backtracking, Kruskal, Prim
- **Parameters**: `cell_size`, `wall_thickness`, `add_loops`, `loop_probability`

### 2. Organic/Cave Algorithms
**Characteristics**: Generate natural, flowing spaces
**Best for**: Caves, natural formations, organic environments

| Algorithm | Complexity | Connectivity | Organic Feel | Performance |
|-----------|------------|--------------|--------------|-------------|
| **Cellular Automata** | Medium | Variable | High | Medium |
| **Drunkard's Walk** | Low | Variable | High | High |
| **Voronoi Diagrams** | High | Variable | Medium | Low |

#### Cellular Automata
- **Use Case**: Natural cave systems, organic chambers
- **Strengths**: Highly organic, configurable density, realistic caves
- **Weaknesses**: Connectivity not guaranteed, can create isolated areas
- **Parameters**: `initial_wall_probability`, `iterations`, `survival_threshold`, `birth_threshold`

#### Drunkard's Walk
- **Use Case**: Winding tunnels, organic pathways, river-like structures
- **Strengths**: Highly organic, guaranteed connectivity, simple
- **Weaknesses**: Can be too sparse, limited room generation
- **Parameters**: `num_walkers`, `steps_per_walker`, `change_direction_chance`, `carve_radius`

#### Voronoi Diagrams
- **Use Case**: Territory-based maps, biome boundaries, organic regions
- **Strengths**: Natural region boundaries, good for biome generation
- **Weaknesses**: Complex implementation, can lack structure
- **Parameters**: `num_sites`, `relaxation_iterations`, `cell_type_distribution`

### 3. Noise-Based Algorithms
**Characteristics**: Use mathematical noise functions for terrain
**Best for**: Heightmaps, terrain generation, natural landscapes

| Algorithm | Complexity | Connectivity | Organic Feel | Performance |
|-----------|------------|--------------|--------------|-------------|
| **Perlin Noise** | Low | Variable | High | High |
| **Simplex Noise** | Low | Variable | High | High |

#### Perlin Noise (Already Implemented)
- **Use Case**: Terrain generation, height maps, natural variation
- **Strengths**: Fast, natural-looking, highly configurable
- **Weaknesses**: May need post-processing for connectivity
- **Parameters**: `scale`, `octaves`, `persistence`, `lacunarity`

### 4. Constraint-Based Algorithms
**Characteristics**: Generate content based on rules and constraints
**Best for**: Complex layouts, puzzle generation, specific requirements

| Algorithm | Complexity | Connectivity | Organic Feel | Performance |
|-----------|------------|--------------|--------------|-------------|
| **Wave Function Collapse** | High | Configurable | Medium | Low |

#### Wave Function Collapse
- **Use Case**: Complex rule-based generation, tile-based layouts
- **Strengths**: Highly configurable, can enforce complex rules
- **Weaknesses**: Complex to set up, can be slow, may fail to solve
- **Parameters**: `tile_size`, `overlap`, `pattern_weights`, `entropy_heuristic`

## Algorithm Combinations

### Hybrid Approaches
Combining multiple algorithms can create more interesting and varied layouts:

1. **BSP + Cellular Automata**: Generate rooms with BSP, then use cellular automata to make room interiors more organic
2. **Simple Rooms + Drunkard's Walk**: Place rooms, then connect with organic tunnels
3. **Voronoi + Maze**: Use Voronoi for region boundaries, maze for internal structure
4. **Perlin Noise + BSP**: Use noise for overall terrain, BSP for structure placement

### Sequential Generation
Apply algorithms in sequence for layered complexity:

1. **Base Structure**: Start with room-based algorithm (BSP/Simple Rooms)
2. **Organic Details**: Apply cellular automata or drunkard's walk for natural variation
3. **Fine Details**: Use noise functions for texture and micro-variation

## Performance Considerations

### Fast Algorithms (Real-time generation)
- Simple Rooms
- Drunkard's Walk  
- Perlin Noise
- Basic Maze (Recursive Backtracking)

### Medium Performance
- BSP
- Cellular Automata
- Advanced Maze (Kruskal, Prim)

### Slow Algorithms (Pre-generation recommended)
- Voronoi with relaxation
- Wave Function Collapse
- Complex hybrid approaches

## Recommended Use Cases

### Dungeon Types
- **Classic Dungeon**: BSP or Simple Rooms
- **Natural Cave**: Cellular Automata + Drunkard's Walk
- **Maze Dungeon**: Maze Generation (any algorithm)
- **Organic Dungeon**: Voronoi + Cellular Automata
- **Complex Dungeon**: Wave Function Collapse

### Terrain Types
- **Desert/Steppe**: Perlin Noise + Voronoi for oases
- **Cave Systems**: Cellular Automata + connectivity post-processing
- **Ruins**: BSP + decay simulation
- **Natural Formations**: Drunkard's Walk + Cellular Automata

### Biome-Specific Recommendations
- **Saltflat**: Voronoi (salt crystal patterns) + Perlin (elevation)
- **Desert**: Drunkard's Walk (wind patterns) + Simple Rooms (oases)
- **Scrubland**: Cellular Automata (vegetation) + Perlin (terrain)
- **Ruins**: BSP (structure) + Cellular Automata (decay)
- **Oasis**: Simple Rooms (water) + Voronoi (vegetation zones)

## Configuration Guidelines

### Parameter Tuning Tips
1. **Start with defaults** and adjust incrementally
2. **Test connectivity** after generation
3. **Balance organic feel vs structure** based on use case
4. **Consider performance** for real-time vs pre-generated content
5. **Use seeds** for reproducible results during testing

### Common Parameter Ranges
- **Room sizes**: 4x4 to 12x8 for playable spaces
- **Corridor widths**: 1-3 tiles for navigation
- **Cellular automata iterations**: 3-7 for good results
- **Walker steps**: 200-1000 depending on map size
- **Noise scales**: 0.1-0.3 for terrain features
