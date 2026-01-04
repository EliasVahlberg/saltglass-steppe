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

## Composite Procedural Generation Architecture

### Integration with Existing Systems

The new structure generation system integrates with existing procedural generation as a **layered composite system**:

```
Composite Procedural Generation Pipeline
├── Layer 1: World Generation (existing)
│   ├── WorldGenerator - POI placement and biome assignment
│   └── BiomeSystem - Environmental content generation
├── Layer 2: Tile Foundation (existing)
│   ├── TileGenerator - Base terrain using Perlin noise
│   ├── ConnectivitySystem - Glass Seam Bridging Algorithm
│   └── ConstraintSystem - Validation and accessibility
├── Layer 3: Structure Generation (NEW)
│   ├── StructureGenerator (trait)
│   │   ├── DungeonGenerator (BSP + Cellular Automata)
│   │   ├── TownGenerator (Voronoi + WFC)
│   │   ├── ShrineGenerator (Templates + L-Systems)
│   │   └── RuinsGenerator (Decay simulation)
│   └── AlgorithmLibrary
│       ├── BSPAlgorithm
│       ├── CellularAutomata
│       ├── WaveFunctionCollapse
│       ├── VoronoiDiagram
│       └── LSystemGrowth
├── Layer 4: Content Population (existing + enhanced)
│   ├── SpawnSystem - Entity placement within structures
│   ├── LootGeneration - Item placement with structure context
│   ├── MicroStructures - Small decorative elements
│   └── QuestConstraintSystem - Quest-specific requirements
├── Layer 5: Narrative Integration (existing)
│   ├── NarrativeIntegration - Story fragment placement
│   ├── EventSystem - Dynamic events within structures
│   └── Grammar - Procedural descriptions
└── Layer 6: Post-Processing (existing + enhanced)
    ├── StormSystem - Glass storm modifications
    ├── ValidationSystem - Final connectivity and accessibility
    └── MetadataSystem - Structure information storage
```

### Layered Generation Flow

```
1. World Layer: Determine POI type and biome context
   ↓
2. Tile Foundation: Generate base terrain + ensure connectivity
   ↓
3. Structure Generation: Create major structures based on POI type
   ↓ (Glass Seam Bridging ensures structure connectivity)
4. Content Population: Place entities, items, and microstructures
   ↓
5. Narrative Integration: Add story elements and dynamic events
   ↓
6. Post-Processing: Apply environmental effects and final validation
```

### Enhanced Integration Points

**Glass Seam Bridging Algorithm Integration:**
- **Structure Connectivity**: Ensures all rooms within generated structures are reachable
- **Multi-Structure Connectivity**: Connects separate structures within the same tile
- **Terrain Integration**: Bridges structures to natural terrain features
- **Quest Accessibility**: Guarantees quest objectives are reachable from structure entrances

**Existing System Enhancements:**

1. **TileGenerator Integration**
   ```rust
   impl TileGenerator {
       pub fn generate_with_structures(&mut self, poi_type: POIType, biome: &str) -> Map {
           // 1. Generate base terrain (existing)
           let mut map = self.generate_base_terrain(biome);
           
           // 2. Generate major structures (NEW)
           if let Some(structure) = self.generate_poi_structure(poi_type, biome) {
               self.integrate_structure(&mut map, structure);
           }
           
           // 3. Apply Glass Seam Bridging (existing, enhanced)
           self.connectivity_system.ensure_structure_connectivity(&mut map);
           
           // 4. Place microstructures (existing)
           self.place_microstructures(&mut map);
           
           map
       }
   }
   ```

2. **Quest Constraint System Enhancement**
   ```rust
   impl QuestConstraintSystem {
       pub fn generate_quest_structure(&self, quest_id: &str) -> Option<Structure> {
           let constraints = self.get_structure_constraints(quest_id)?;
           
           // Select appropriate generator based on quest requirements
           let generator = match constraints.structure_type {
               StructureType::Dungeon => &self.dungeon_generator,
               StructureType::Town => &self.town_generator,
               StructureType::Shrine => &self.shrine_generator,
               StructureType::Ruins => &self.ruins_generator,
           };
           
           // Generate with quest-specific parameters
           let structure = generator.generate(&constraints.to_params(), &mut self.rng)?;
           
           // Validate quest requirements are met
           self.validate_quest_requirements(&structure, &constraints)?;
           
           Some(structure)
       }
   }
   ```

3. **Spawn System Integration**
   ```rust
   impl SpawnSystem {
       pub fn populate_structure(&mut self, structure: &Structure, biome: &str) {
           // Use structure context for enhanced spawning
           for room in &structure.rooms {
               let spawn_context = SpawnContext {
                   room_type: room.room_type.clone(),
                   structure_type: structure.structure_type,
                   biome: biome.to_string(),
                   depth_level: room.depth_from_entrance,
               };
               
               self.spawn_entities_in_room(room, &spawn_context);
           }
       }
   }
   ```

### Composite Generation Phases

**Phase 1: Foundation (Existing Systems)**
- World generation determines POI placement and biome context
- Base tile generation creates natural terrain using Perlin noise
- Glass Seam Bridging ensures basic terrain connectivity

**Phase 2: Structure Generation (NEW)**
- POI-specific structure generators create major architectural features
- Algorithms selected based on structure type and biome context
- Glass Seam Bridging extended to ensure structure internal connectivity

**Phase 3: Integration (Enhanced Existing)**
- Structures integrated with natural terrain seamlessly
- Microstructures placed to complement major structures
- Spawn system uses structure context for appropriate entity placement

**Phase 4: Content & Narrative (Existing + Enhanced)**
- Quest constraints ensure required items/NPCs are placed correctly
- Narrative fragments placed with structure context awareness
- Dynamic events configured based on structure type and contents

**Phase 5: Environmental Effects (Existing)**
- Storm system can modify structures over time
- Biome effects applied to structure materials and decorations
- Final validation ensures all systems work together correctly

---

## Refined Implementation Plan

### Phase 1: Foundation Integration (Week 1)

**1.1 Structure Generation Trait System**
```rust
// src/game/generation/structures/mod.rs
pub trait StructureGenerator {
    fn generate(&self, params: &StructureParams, rng: &mut ChaCha8Rng) -> Structure;
    fn get_supported_poi_types(&self) -> Vec<POIType>;
    fn estimate_generation_time(&self, params: &StructureParams) -> Duration;
}

pub struct Structure {
    pub structure_type: StructureType,
    pub bounds: Rectangle,
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub features: Vec<StructureFeature>,
    pub spawn_points: Vec<SpawnPoint>,
    pub metadata: HashMap<String, String>,
}
```

**1.2 Integration with TileGenerator**
```rust
// Enhance existing src/game/generation/tile_gen.rs
impl TileGenerator {
    pub fn generate_enhanced_tile_with_structures(
        &mut self, 
        poi_type: Option<POIType>, 
        biome: &str,
        quest_ids: Vec<String>
    ) -> Map {
        // 1. Generate base terrain (existing Perlin noise system)
        let mut map = self.generate_base_terrain(biome);
        
        // 2. Generate major structure if POI present (NEW)
        if let Some(poi) = poi_type {
            if let Some(structure) = self.generate_poi_structure(poi, biome, &quest_ids) {
                self.integrate_structure_with_terrain(&mut map, structure);
            }
        }
        
        // 3. Apply enhanced Glass Seam Bridging (existing + structure connectivity)
        self.connectivity_system.ensure_full_connectivity(&mut map);
        
        // 4. Place microstructures (existing, but avoid structure areas)
        self.place_microstructures_around_structures(&mut map);
        
        map
    }
}
```

**1.3 Enhanced Glass Seam Bridging**
```rust
// Enhance existing src/game/generation/connectivity.rs
impl ConnectivitySystem {
    pub fn ensure_full_connectivity(&mut self, map: &mut Map) {
        // 1. Existing terrain connectivity
        self.ensure_terrain_connectivity(map);
        
        // 2. NEW: Structure internal connectivity
        self.ensure_structure_connectivity(map);
        
        // 3. NEW: Structure-to-terrain connectivity
        self.ensure_structure_terrain_connectivity(map);
    }
    
    fn ensure_structure_connectivity(&mut self, map: &mut Map) {
        for structure in &map.structures {
            // Apply Glass Seam Bridging within each structure
            self.bridge_structure_rooms(map, structure);
        }
    }
}
```

### Phase 2: Dungeon Generator Implementation (Week 2)

**2.1 BSP Algorithm Implementation**
```rust
// src/game/generation/structures/algorithms/bsp.rs
pub struct BSPAlgorithm {
    pub min_room_size: (u32, u32),
    pub max_room_size: (u32, u32),
    pub corridor_width: u32,
}

impl Algorithm for BSPAlgorithm {
    fn apply(&self, grid: &mut Grid, params: &AlgorithmParams, rng: &mut ChaCha8Rng) {
        let root = BSPNode::new(grid.bounds());
        self.partition_recursive(&root, params, rng);
        self.place_rooms(&root, grid, rng);
        self.connect_rooms(&root, grid);
    }
}
```

**2.2 Dungeon Generator**
```rust
// src/game/generation/structures/dungeon_generator.rs
pub struct DungeonGenerator {
    bsp_algorithm: BSPAlgorithm,
    cellular_automata: CellularAutomata,
}

impl StructureGenerator for DungeonGenerator {
    fn generate(&self, params: &StructureParams, rng: &mut ChaCha8Rng) -> Structure {
        let mut grid = Grid::new(params.size);
        
        // 1. Apply BSP for room layout
        self.bsp_algorithm.apply(&mut grid, &params.to_algorithm_params(), rng);
        
        // 2. Apply Cellular Automata for organic walls
        if params.organic_walls {
            self.cellular_automata.apply(&mut grid, &params.to_ca_params(), rng);
        }
        
        // 3. Convert grid to Structure
        self.grid_to_structure(grid, params)
    }
}
```

**2.3 Replace Existing Vitrified Library Ruins**
```rust
// Update src/game/generation/tile_gen.rs
impl TileGenerator {
    pub fn place_vitrified_library_ruins(&mut self, map: &mut Map, quest_ids: &[String]) {
        // Replace hardcoded structure with procedural generation
        let params = StructureParams {
            structure_type: StructureType::Ruins,
            size: (25, 20),
            theme: "vitrified_library".to_string(),
            quest_requirements: quest_ids.to_vec(),
            biome_context: "ruins".to_string(),
        };
        
        let generator = RuinsGenerator::new();
        if let Some(structure) = generator.generate(&params, &mut self.rng) {
            self.integrate_structure_with_terrain(map, structure);
        }
    }
}
```

### Phase 3: DES Testing Framework (Week 2)

**3.1 Structure Generation Tests**
```json
// tests/scenarios/structure_generation_basic.json
{
  "name": "dungeon_generation_basic",
  "seed": 12345,
  "map_setup": {
    "poi_type": "dungeon",
    "biome": "ruins",
    "size": [40, 30]
  },
  "actions": [
    {"turn": 0, "action": {"type": "generate_structure", "structure_type": "dungeon"}}
  ],
  "assertions": [
    {"at_end": true, "check": {"type": "structure_connectivity", "expected": true}},
    {"at_end": true, "check": {"type": "room_count", "min": 3, "max": 8}},
    {"at_end": true, "check": {"type": "entrance_exists", "expected": true}},
    {"at_end": true, "check": {"type": "quest_item_reachable", "item": "broken_saint_key"}}
  ]
}
```

**3.2 Integration Tests**
```json
// tests/scenarios/quest_structure_integration.json
{
  "name": "broken_key_quest_structure",
  "seed": 42,
  "player": {"x": 50, "y": 50},
  "quests": ["the_broken_key"],
  "actions": [
    {"turn": 0, "action": {"type": "travel_to_tile", "x": 50, "y": 50}}
  ],
  "assertions": [
    {"at_end": true, "check": {"type": "structure_generated", "structure_type": "ruins"}},
    {"at_end": true, "check": {"type": "quest_item_present", "item": "broken_saint_key"}},
    {"at_end": true, "check": {"type": "structure_accessible", "from_entrance": true}}
  ]
}
```

### Phase 4: Data-Driven Configuration (Week 3)

**4.1 Structure Templates**
```json
// data/structure_generation.json
{
  "generators": {
    "dungeon": {
      "class": "DungeonGenerator",
      "algorithms": [
        {
          "name": "BSP",
          "weight": 0.7,
          "params": {
            "min_room_size": [4, 4],
            "max_room_size": [12, 8],
            "corridor_width": 1,
            "max_depth": 5
          }
        },
        {
          "name": "CellularAutomata",
          "weight": 0.3,
          "params": {
            "iterations": 3,
            "birth_limit": 4,
            "death_limit": 3
          }
        }
      ]
    }
  },
  
  "poi_mappings": {
    "dungeon": "dungeon",
    "town": "town",
    "shrine": "shrine", 
    "landmark": "ruins"
  },
  
  "biome_modifiers": {
    "ruins": {
      "material_substitutions": {
        "stone_wall": "cracked_stone_wall",
        "wooden_door": "broken_door"
      },
      "decay_factor": 0.3
    }
  }
}
```

### Phase 5: Performance & Integration (Week 4)

**5.1 Lazy Structure Generation**
```rust
// src/game/generation/structures/lazy_generator.rs
pub struct LazyStructureGenerator {
    generators: HashMap<StructureType, Box<dyn StructureGenerator>>,
    cache: LRUCache<StructureKey, Structure>,
}

impl LazyStructureGenerator {
    pub fn generate_or_cache(&mut self, params: &StructureParams) -> Option<Structure> {
        let key = StructureKey::from_params(params);
        
        if let Some(cached) = self.cache.get(&key) {
            return Some(cached.clone());
        }
        
        let generator = self.generators.get(&params.structure_type)?;
        let structure = generator.generate(params, &mut self.rng)?;
        
        self.cache.put(key, structure.clone());
        Some(structure)
    }
}
```

**5.2 Integration with Existing Systems**
```rust
// Update src/game/state.rs travel_to_tile method
impl GameState {
    pub fn travel_to_tile(&mut self, x: i32, y: i32) {
        // ... existing code ...
        
        // Enhanced structure generation
        let quest_ids = self.get_quest_ids_for_location(x, y);
        let poi_type = self.world_map.get_poi_at(x, y);
        let biome = self.world_map.get_biome_at(x, y);
        
        // Generate map with integrated structure system
        self.map = self.tile_generator.generate_enhanced_tile_with_structures(
            poi_type, 
            &biome, 
            quest_ids
        );
        
        // ... rest of existing code ...
    }
}
```

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
