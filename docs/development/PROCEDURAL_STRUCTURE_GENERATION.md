# Procedural Structure Generation System

**Version:** 1.0  
**Date:** 2026-01-04  
**Owner:** Lead Developer  
**Purpose:** Design and implement a data-driven, decoupled system for procedurally generating dungeons, towns/cities, shrines/monuments, and ruins.

---

## Executive Summary

This document outlines the implementation of a unified procedural structure generation system for Saltglass Steppe. Based on research into established algorithms and best practices, we propose a modular, data-driven approach that can generate diverse structure types while maintaining the game's thematic consistency and technical requirements.

**Key Goals:**
- **Decoupled Architecture**: Each structure type uses specialized generators with shared foundational systems
- **Data-Driven Configuration**: Structure parameters, rules, and templates defined in JSON files
- **Deterministic Generation**: All generation uses seeded RNG for reproducible results
- **Thematic Consistency**: Generated structures fit the Saltglass Steppe aesthetic and lore

---

## Research Summary

### Established Algorithms

Based on industry research, the following algorithms are proven effective for different structure types:

| Algorithm | Best For | Characteristics |
|-----------|----------|-----------------|
| **BSP (Binary Space Partitioning)** | Dungeons, Large Buildings | Hierarchical room division, guaranteed connectivity |
| **Cellular Automata** | Organic Caves, Natural Ruins | Organic shapes, erosion simulation |
| **Wave Function Collapse** | Complex Buildings, Settlements | Constraint-based, pattern matching |
| **Random Walk** | Corridors, Paths | Simple, maze-like structures |
| **Voronoi Diagrams** | City Districts, Territory Division | Natural boundaries, organic regions |
| **L-Systems** | Organic Growth, Tree-like Structures | Fractal patterns, natural branching |

### Structure Type Analysis

**Dungeons:**
- Primary: BSP for room layout + Cellular Automata for organic walls
- Secondary: Random Walk for connecting corridors
- Examples: Brogue, Crypt of the NecroDancer

**Towns/Cities:**
- Primary: Voronoi for districts + Road network algorithms
- Secondary: Wave Function Collapse for building placement
- Examples: Dwarf Fortress, Banished

**Shrines/Monuments:**
- Primary: Template-based with L-Systems for decorative elements
- Secondary: Symmetry algorithms for formal layouts
- Examples: Minecraft temples, procedural architecture

**Ruins:**
- Primary: Generate complete structure + Cellular Automata for decay
- Secondary: Erosion algorithms for realistic deterioration
- Examples: Fallout settlement ruins, archaeological sites

---

## System Architecture

### Core Components

```
ProceduralStructureSystem
├── StructureGenerator (trait)
│   ├── DungeonGenerator
│   ├── TownGenerator  
│   ├── ShrineGenerator
│   └── RuinsGenerator
├── AlgorithmLibrary
│   ├── BSPAlgorithm
│   ├── CellularAutomata
│   ├── WaveFunctionCollapse
│   ├── RandomWalk
│   ├── VoronoiDiagram
│   └── LSystemGrowth
├── StructureTemplates (data-driven)
├── ValidationSystem
└── PostProcessing
```

### Data Flow

```
1. Structure Request (type, size, theme, constraints)
   ↓
2. Load Template & Configuration
   ↓
3. Select Appropriate Generator
   ↓
4. Apply Primary Algorithm
   ↓
5. Apply Secondary Algorithms (refinement)
   ↓
6. Validate Constraints (connectivity, accessibility)
   ↓
7. Post-Process (theming, decay, population)
   ↓
8. Return Generated Structure
```

---

## Implementation Plan

### Phase 1: Foundation (Week 1-2)

**1.1 Core Trait System**
```rust
// src/game/generation/structures/mod.rs
pub trait StructureGenerator {
    fn generate(&self, params: &StructureParams, rng: &mut ChaCha8Rng) -> Structure;
    fn validate(&self, structure: &Structure) -> ValidationResult;
    fn get_supported_types(&self) -> Vec<StructureType>;
}

pub struct StructureParams {
    pub structure_type: StructureType,
    pub size: (u32, u32),
    pub theme: String,
    pub constraints: Vec<Constraint>,
    pub biome: String,
    pub difficulty_level: u32,
}
```

**1.2 Algorithm Library Foundation**
```rust
// src/game/generation/structures/algorithms/mod.rs
pub trait Algorithm {
    fn apply(&self, grid: &mut Grid, params: &AlgorithmParams, rng: &mut ChaCha8Rng);
}

pub struct BSPAlgorithm;
pub struct CellularAutomata;
pub struct RandomWalk;
// ... other algorithms
```

**1.3 Data Configuration System**
```json
// data/structure_templates.json
{
  "dungeon_templates": {
    "glass_cavern": {
      "size_range": {"min": [40, 30], "max": [80, 60]},
      "room_count_range": [5, 12],
      "algorithms": [
        {"type": "BSP", "weight": 0.7, "params": {"min_room_size": [6, 6]}},
        {"type": "CellularAutomata", "weight": 0.3, "params": {"iterations": 5}}
      ],
      "themes": ["glass", "crystal", "ancient"],
      "required_features": ["entrance", "exit", "treasure_room"]
    }
  }
}
```

### Phase 2: Dungeon Generator (Week 3-4)

**2.1 BSP Implementation**
- Recursive space partitioning
- Room placement within partitions
- Corridor generation between rooms
- Connectivity validation

**2.2 Cellular Automata Integration**
- Wall smoothing and organic shapes
- Cave-like chamber generation
- Natural corridor widening

**2.3 Dungeon-Specific Features**
- Entrance/exit placement
- Treasure room generation
- Trap and secret area placement
- Enemy spawn point distribution

### Phase 3: Town Generator (Week 5-6)

**3.1 District Generation**
- Voronoi diagram for district boundaries
- Road network using pathfinding algorithms
- Central plaza/market placement

**3.2 Building Placement**
- Wave Function Collapse for building types
- Zoning rules (residential, commercial, industrial)
- Building size and style variation

**3.3 Town-Specific Features**
- Wall and gate placement
- Well and resource building placement
- NPC house assignment
- Trade route connections

### Phase 4: Shrine/Monument Generator (Week 7)

**4.1 Template-Based Generation**
- Symmetrical layouts for formal structures
- Sacred geometry patterns
- Decorative element placement

**4.2 L-System Integration**
- Organic decorative growth
- Pillar and arch generation
- Natural integration with landscape

### Phase 5: Ruins Generator (Week 8)

**5.1 Decay Simulation**
- Generate complete structure first
- Apply cellular automata for erosion
- Structural collapse simulation

**5.2 Archaeological Features**
- Buried sections and excavation sites
- Artifact scatter patterns
- Overgrowth and reclamation

### Phase 6: Integration & Polish (Week 9-10)

**6.1 Quest Integration**
- Structure generation based on quest requirements
- Guaranteed item and NPC placement
- Accessibility validation

**6.2 Performance Optimization**
- Caching of generated structures
- Lazy loading for large structures
- Memory management

---

## Detailed Algorithm Specifications

### BSP Algorithm

**Purpose:** Generate hierarchical room layouts with guaranteed connectivity

**Implementation:**
```rust
pub struct BSPNode {
    pub bounds: Rectangle,
    pub room: Option<Rectangle>,
    pub left: Option<Box<BSPNode>>,
    pub right: Option<Box<BSPNode>>,
}

impl BSPAlgorithm {
    fn partition(&self, node: &mut BSPNode, min_size: (u32, u32), rng: &mut ChaCha8Rng) {
        // Recursive partitioning logic
        // Split horizontally or vertically based on aspect ratio
        // Continue until minimum size reached
    }
    
    fn place_rooms(&self, node: &mut BSPNode, rng: &mut ChaCha8Rng) {
        // Place rooms within leaf nodes
        // Ensure rooms don't touch partition boundaries
    }
    
    fn connect_rooms(&self, node: &BSPNode, grid: &mut Grid) {
        // Create corridors between sibling rooms
        // Use L-shaped or straight corridors as appropriate
    }
}
```

**Configuration:**
```json
{
  "bsp_params": {
    "min_room_size": [6, 6],
    "max_room_size": [20, 15],
    "split_ratio_range": [0.3, 0.7],
    "corridor_width": 1,
    "room_padding": 1
  }
}
```

### Cellular Automata Algorithm

**Purpose:** Create organic, cave-like structures and smooth artificial layouts

**Implementation:**
```rust
pub struct CellularAutomata {
    pub birth_limit: u8,
    pub death_limit: u8,
    pub iterations: u32,
}

impl Algorithm for CellularAutomata {
    fn apply(&self, grid: &mut Grid, params: &AlgorithmParams, rng: &mut ChaCha8Rng) {
        // Initialize with random noise or existing structure
        // Apply birth/death rules for specified iterations
        // Smooth walls and create organic shapes
    }
}
```

**Rules:**
- Birth: Empty cell becomes wall if >= birth_limit wall neighbors
- Death: Wall cell becomes empty if < death_limit wall neighbors
- Typical values: birth_limit=4, death_limit=3

### Wave Function Collapse Algorithm

**Purpose:** Generate complex structures following pattern constraints

**Implementation:**
```rust
pub struct WFCTile {
    pub tile_type: String,
    pub constraints: HashMap<Direction, Vec<String>>, // Valid neighbors
}

pub struct WaveFunctionCollapse {
    pub tiles: Vec<WFCTile>,
    pub patterns: Vec<Pattern>,
}

impl Algorithm for WaveFunctionCollapse {
    fn apply(&self, grid: &mut Grid, params: &AlgorithmParams, rng: &mut ChaCha8Rng) {
        // Initialize superposition of all possible states
        // Iteratively collapse cells with lowest entropy
        // Propagate constraints to neighbors
        // Backtrack if contradictions arise
    }
}
```

---

## Data-Driven Configuration

### Structure Templates

**File:** `data/structure_templates.json`

```json
{
  "templates": {
    "glass_dungeon": {
      "type": "dungeon",
      "size_range": {"min": [30, 20], "max": [60, 40]},
      "algorithms": [
        {
          "name": "BSP",
          "weight": 0.6,
          "params": {
            "min_room_size": [4, 4],
            "max_room_size": [12, 8],
            "corridor_width": 1
          }
        },
        {
          "name": "CellularAutomata", 
          "weight": 0.4,
          "params": {
            "iterations": 3,
            "birth_limit": 4,
            "death_limit": 3
          }
        }
      ],
      "features": {
        "entrance": {"required": true, "placement": "edge"},
        "exit": {"required": true, "placement": "opposite_edge"},
        "treasure_room": {"required": true, "placement": "deepest"},
        "secret_areas": {"count_range": [0, 2], "placement": "hidden"}
      },
      "theming": {
        "wall_types": ["glass_wall", "crystal_wall"],
        "floor_types": ["glass_floor", "sand_floor"],
        "decorations": ["crystal_formation", "glass_shard_pile"]
      }
    },
    
    "salt_town": {
      "type": "town",
      "size_range": {"min": [50, 50], "max": [100, 80]},
      "algorithms": [
        {
          "name": "Voronoi",
          "weight": 0.8,
          "params": {
            "district_count_range": [3, 6],
            "site_distribution": "poisson"
          }
        },
        {
          "name": "WFC",
          "weight": 0.2,
          "params": {
            "building_patterns": "salt_town_buildings"
          }
        }
      ],
      "districts": {
        "residential": {"weight": 0.4, "building_types": ["house", "apartment"]},
        "commercial": {"weight": 0.3, "building_types": ["shop", "tavern", "market"]},
        "industrial": {"weight": 0.2, "building_types": ["workshop", "warehouse"]},
        "civic": {"weight": 0.1, "building_types": ["town_hall", "temple"]}
      }
    }
  }
}
```

### Algorithm Parameters

**File:** `data/algorithm_configs.json`

```json
{
  "algorithms": {
    "BSP": {
      "default_params": {
        "min_room_size": [6, 6],
        "max_room_size": [20, 15],
        "split_ratio_range": [0.3, 0.7],
        "corridor_width": 1,
        "max_depth": 6
      },
      "variants": {
        "compact": {"max_room_size": [12, 10], "corridor_width": 1},
        "spacious": {"min_room_size": [8, 8], "corridor_width": 2}
      }
    },
    
    "CellularAutomata": {
      "default_params": {
        "iterations": 5,
        "birth_limit": 4,
        "death_limit": 3,
        "initial_density": 0.45
      },
      "variants": {
        "cave_like": {"birth_limit": 5, "death_limit": 2},
        "maze_like": {"birth_limit": 3, "death_limit": 4}
      }
    }
  }
}
```

---

## Integration with Existing Systems

### Quest System Integration

```rust
// Enhanced quest constraint system
pub struct StructureConstraint {
    pub structure_type: StructureType,
    pub required_features: Vec<String>,
    pub accessibility_requirements: AccessibilityRequirements,
    pub item_placement: Vec<ItemPlacement>,
    pub npc_placement: Vec<NpcPlacement>,
}

impl QuestConstraintSystem {
    pub fn generate_structure_for_quest(
        &self, 
        quest_id: &str, 
        location: (i32, i32)
    ) -> Option<Structure> {
        let constraints = self.get_structure_constraints(quest_id)?;
        let generator = self.select_generator(&constraints.structure_type);
        
        let params = StructureParams {
            structure_type: constraints.structure_type,
            size: self.calculate_size(&constraints),
            constraints: constraints.to_generation_constraints(),
            // ... other params
        };
        
        generator.generate(&params, &mut self.rng)
    }
}
```

### Biome Integration

```rust
// Biome-specific structure modifications
pub struct BiomeModifier {
    pub biome: String,
    pub material_substitutions: HashMap<String, String>,
    pub decoration_additions: Vec<String>,
    pub environmental_effects: Vec<EnvironmentalEffect>,
}

// Example: Saltflat biome modifications
{
  "biome": "saltflat",
  "material_substitutions": {
    "stone_wall": "salt_crystal_wall",
    "wooden_door": "glass_door"
  },
  "decoration_additions": ["salt_deposit", "glass_shard_scatter"],
  "environmental_effects": ["heat_shimmer", "salt_corrosion"]
}
```

### Storm System Integration

```rust
// Structures can be modified by glass storms
impl StormSystem {
    pub fn apply_storm_to_structure(&mut self, structure: &mut Structure, intensity: u8) {
        match intensity {
            1..=3 => self.apply_minor_glass_effects(structure),
            4..=6 => self.apply_major_glass_transformation(structure),
            7..=10 => self.apply_complete_vitrification(structure),
            _ => {}
        }
    }
}
```

---

## Performance Considerations

### Memory Management

- **Lazy Loading**: Generate structure sections as player explores
- **Caching**: Store frequently accessed structures in memory
- **Streaming**: Load/unload structure data based on player proximity

### Generation Speed

- **Incremental Generation**: Generate structures in stages over multiple frames
- **Precomputation**: Pre-generate common structure patterns
- **Optimization**: Use efficient algorithms for real-time generation

### Determinism

- **Seeded RNG**: All generation uses ChaCha8Rng with consistent seeds
- **Reproducibility**: Same seed + parameters = identical structure
- **Save Compatibility**: Structure generation remains consistent across game versions

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bsp_connectivity() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let generator = DungeonGenerator::new();
        let params = StructureParams::default();
        
        let structure = generator.generate(&params, &mut rng);
        
        // Verify all rooms are connected
        assert!(structure.validate_connectivity());
    }
    
    #[test]
    fn test_deterministic_generation() {
        let seed = 42;
        let params = StructureParams::default();
        
        let structure1 = generate_with_seed(seed, &params);
        let structure2 = generate_with_seed(seed, &params);
        
        assert_eq!(structure1, structure2);
    }
}
```

### Integration Tests

- **Quest Integration**: Verify structures meet quest requirements
- **Biome Compatibility**: Test structure generation in all biomes
- **Performance**: Measure generation time for various structure sizes
- **Visual Validation**: Generate test structures and verify visual output

### DES Scenarios

```json
{
  "name": "structure_generation_test",
  "seed": 12345,
  "actions": [
    {"turn": 0, "action": {"type": "generate_structure", "structure_type": "dungeon", "size": [40, 30]}}
  ],
  "assertions": [
    {"at_end": true, "check": {"type": "structure_connectivity", "expected": true}},
    {"at_end": true, "check": {"type": "structure_size", "min": [35, 25], "max": [45, 35]}}
  ]
}
```

---

## Migration Plan

### Phase 1: Replace Existing Vitrified Library Ruins

1. **Create DungeonGenerator** with BSP + Cellular Automata
2. **Replace hardcoded structure** in `place_vitrified_library_ruins()`
3. **Test quest integration** to ensure "The Broken Key" still works
4. **Validate performance** and generation time

### Phase 2: Expand to Other Structure Types

1. **Implement TownGenerator** for settlement POIs
2. **Create ShrineGenerator** for shrine POIs  
3. **Add RuinsGenerator** for landmark POIs
4. **Update POI system** to use procedural generators

### Phase 3: Data-Driven Configuration

1. **Move all parameters** to JSON configuration files
2. **Create structure template system**
3. **Add runtime configuration** validation
4. **Enable mod support** through data files

---

## Future Enhancements

### Advanced Features

- **Multi-Level Structures**: Dungeons with multiple floors
- **Dynamic Structures**: Buildings that change over time
- **Player Modifications**: Structures that respond to player actions
- **Faction Influence**: Structures that reflect faction control

### Algorithm Improvements

- **Hybrid Algorithms**: Combine multiple algorithms for better results
- **Machine Learning**: Train models on hand-crafted structures
- **Constraint Solving**: Advanced constraint satisfaction for complex requirements
- **Optimization**: Genetic algorithms for structure optimization

### Content Expansion

- **Structure Variants**: Multiple templates per structure type
- **Seasonal Changes**: Structures that change with game time
- **Cultural Variations**: Faction-specific architectural styles
- **Historical Layers**: Structures showing multiple time periods

---

## Conclusion

This procedural structure generation system will provide Saltglass Steppe with diverse, thematically consistent, and technically robust structures. The data-driven approach ensures easy content expansion, while the modular architecture allows for future enhancements and optimizations.

The system maintains the game's core principles of determinism, data-driven design, and decoupled architecture while providing the procedural variety needed for long-term replayability.

**Next Steps:**
1. Review and approve this design document
2. Begin Phase 1 implementation with foundation systems
3. Create initial data configuration files
4. Implement and test the dungeon generator
5. Integrate with existing quest and biome systems

---

*This document serves as the technical specification for the procedural structure generation system. All implementation should follow these guidelines to ensure consistency and maintainability.*
