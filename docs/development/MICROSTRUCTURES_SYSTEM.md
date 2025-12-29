# Micro-Structures System

**Version:** 1.0  
**Date:** 2025-12-29  
**Status:** Implemented  

## Overview

The micro-structures system adds small, thematic structures to the procedurally generated world, creating points of interest, NPC encounters, and loot opportunities that enhance exploration and world-building in the Saltglass Steppe.

## Design Philosophy

### Core Principles
- **Thematic Consistency**: All structures fit the Saltglass Steppe setting
- **Gameplay Purpose**: Each structure serves a mechanical function
- **Spatial Distribution**: Structures are placed with proper spacing
- **Data-Driven**: Easy to add new structures via JSON configuration

### Setting Integration
Micro-structures reflect the harsh, post-apocalyptic world where survivors create small outposts, camps, and specialized facilities to survive the glass storms and dangerous environment.

## Structure Types

### 1. Scavenger Camp
**Purpose**: Trading and basic supplies  
**Size**: 5×4 tiles  
**Features**:
- Scavenger Trader NPC (80% chance)
- Supply Crate chest (90% chance)
- Basic items: torch, brine vial

**Biome Distribution**:
- Saltflat: 30 weight
- Scrubland: 25 weight
- Ruins: 15 weight
- Oasis: 10 weight

### 2. Abandoned Outpost
**Purpose**: High-risk, high-reward exploration  
**Size**: 6×5 tiles  
**Features**:
- Fortified walls with breach
- Metal Strongbox (70% chance, locked)
- Valuable loot: storm glass, scripture shards

**Biome Distribution**:
- Ruins: 40 weight (most common in ruins)
- Saltflat: 20 weight
- Scrubland: 15 weight

### 3. Glass Garden
**Purpose**: Glass-related items and hermit encounters  
**Size**: 4×4 tiles  
**Features**:
- Glass Hermit NPC (90% chance)
- Glass Cache chest (80% chance)
- Specialized loot: storm glass, angle-split lens
- Glass tile perimeter

**Biome Distribution**:
- Saltflat: 35 weight
- Ruins: 25 weight
- Scrubland: 20 weight

### 4. Storm Shelter
**Purpose**: Safe haven and basic supplies  
**Size**: 4×3 tiles  
**Features**:
- Storm Watcher NPC (70% chance)
- Supply Crate (80% chance)
- Reinforced walls for protection

**Biome Distribution**:
- Scrubland: 30 weight
- Saltflat: 25 weight
- Oasis: 15 weight
- Ruins: 10 weight

### 5. Shrine Remnant
**Purpose**: Religious lore and scripture items  
**Size**: 3×3 tiles  
**Features**:
- Wandering Monk NPC (60% chance)
- Scripture Shards (80% chance)
- Saints Tear (30% chance, rare)
- Broken altar pattern

**Biome Distribution**:
- Ruins: 35 weight
- Oasis: 25 weight
- Saltflat: 20 weight
- Scrubland: 15 weight

### 6. Salt Harvester
**Purpose**: Industrial salt extraction  
**Size**: 5×3 tiles  
**Features**:
- Salt Worker NPC (80% chance)
- Supply Crate (70% chance)
- Glass pools (evaporation ponds)
- Salt-related items: brine vial, salt poultice

**Biome Distribution**:
- Saltflat: 45 weight (most appropriate)
- Oasis: 20 weight
- Scrubland: 15 weight

## Technical Implementation

### Core Components

#### 1. Structure Definition (`MicroStructureDef`)
```rust
pub struct MicroStructureDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<MicroStructureTile>,
    pub spawns: Vec<MicroStructureSpawn>,
    pub biome_weights: HashMap<String, u32>,
    pub min_distance_from_player: u32,
    pub min_distance_between: u32,
}
```

#### 2. Tile Patterns (`MicroStructureTile`)
Defines the physical layout:
- Position relative to structure origin
- Tile type (wall, floor, glass)
- Optional visual customization

#### 3. Entity Spawning (`MicroStructureSpawn`)
Defines what spawns in the structure:
- Spawn type (NPC, chest, item)
- Entity ID
- Position within structure
- Probability chance

#### 4. Placement Algorithm
```rust
pub fn place_microstructures(
    map: &mut Map,
    biome: &str,
    clearings: &[(i32, i32)],
    player_pos: (i32, i32),
    rng: &mut ChaCha8Rng,
) -> (Vec<PlacedMicroStructure>, Vec<Npc>, Vec<Chest>, Vec<Item>)
```

### Placement Logic

#### Spatial Constraints
- **Player Distance**: 20+ tiles from player spawn
- **Structure Separation**: Varies by structure type (30-50 tiles)
- **Map Boundaries**: Structures must fit within map bounds
- **Terrain Suitability**: 60% of area must be walkable

#### Biome Selection
1. Filter structures by biome compatibility
2. Weight selection by biome-specific values
3. Random selection using weighted probability
4. Fallback to default if no matches

#### Placement Process
1. **Candidate Filtering**: Remove positions too close to player
2. **Structure Selection**: Choose structure type by biome weights
3. **Position Validation**: Check spatial constraints and terrain
4. **Map Modification**: Place tiles on map
5. **Entity Spawning**: Create NPCs, chests, and items
6. **Tracking**: Record placed structure for save/load

## Data Configuration

### Structure Definition Format
File: `data/microstructures.json`

```json
{
  "id": "structure_id",
  "name": "Display Name",
  "description": "Flavor text",
  "width": 5,
  "height": 4,
  "tiles": [
    {"x": 0, "y": 0, "tile_type": "wall"},
    {"x": 1, "y": 1, "tile_type": "floor"}
  ],
  "spawns": [
    {"spawn_type": "npc", "id": "trader", "x": 2, "y": 2, "chance": 0.8}
  ],
  "biome_weights": {
    "saltflat": 30,
    "ruins": 20
  },
  "min_distance_from_player": 25,
  "min_distance_between": 40
}
```

### NPC Integration
New NPCs added for structures:
- `scavenger_trader`: Basic trading
- `glass_hermit`: Glass cultivation wisdom
- `storm_watcher`: Storm forecasting
- `wandering_monk`: Religious lore
- `salt_worker`: Industrial operations

## Gameplay Impact

### Exploration Rewards
- **Discovery**: Structures provide landmarks and goals
- **Loot**: Contextual items appropriate to structure type
- **NPCs**: Social interaction and trading opportunities
- **Lore**: Environmental storytelling through structure design

### Strategic Considerations
- **Risk/Reward**: Some structures are more dangerous but offer better loot
- **Resource Management**: Structures provide supplies and trading
- **Information**: NPCs offer world knowledge and quest hooks
- **Safe Havens**: Some structures provide rest and healing

## Performance Characteristics

### Generation Time
- **Structure Placement**: O(n×m) where n=structures, m=positions
- **Spatial Validation**: O(1) with grid indexing
- **Entity Creation**: O(k) where k=spawned entities

### Memory Usage
- **Structure Definitions**: Loaded once at startup
- **Placed Structures**: Minimal tracking data
- **Generated Entities**: Standard NPC/chest/item overhead

### Determinism
- **Seed-Based**: Same seed produces identical structures
- **Position Consistency**: Structures appear in same locations
- **Entity Spawning**: Deterministic based on structure seed

## Integration Points

### Map Generation
- Called during tile generation after basic terrain
- Integrates with existing clearing detection
- Modifies map tiles directly

### Entity Systems
- Creates NPCs using existing NPC system
- Generates chests with loot table integration
- Places items using standard item system

### Save/Load
- Structures are regenerated from seed
- Placed structure tracking for state consistency
- Entity references maintained through indices

## Testing and Validation

### DES Test Coverage
File: `tests/scenarios/microstructures_test.json`

Tests:
- Structure generation
- Entity spawning
- Spatial distribution
- Biome appropriateness

### Manual Testing Checklist
- [ ] Structures appear in appropriate biomes
- [ ] Proper spacing between structures
- [ ] NPCs spawn with correct dialogue
- [ ] Chests contain appropriate loot
- [ ] No structures overlap or clip

## Future Enhancements

### Planned Features
1. **Dynamic Structures**: Structures that change over time
2. **Player Construction**: Allow players to build structures
3. **Faction Ownership**: Structures controlled by different factions
4. **Seasonal Variation**: Structures affected by weather/storms

### Content Expansion
- **More Structure Types**: Laboratories, workshops, monuments
- **Larger Structures**: Multi-room complexes
- **Interactive Elements**: Workbenches, altars, machinery
- **Quest Integration**: Structures as quest locations

### Technical Improvements
- **Visual Enhancements**: Custom glyphs and colors
- **Procedural Variation**: Randomized structure layouts
- **Performance Optimization**: Spatial indexing improvements
- **Modding Support**: External structure definition loading

## Troubleshooting

### Common Issues

**Structures not appearing**:
- Check biome weights in structure definitions
- Verify minimum distance constraints
- Ensure sufficient clearings exist

**NPCs missing from structures**:
- Check spawn chance probabilities
- Verify NPC IDs exist in NPC definitions
- Check entity spawning logic

**Structures overlapping**:
- Increase `min_distance_between` values
- Check spatial validation logic
- Verify placement algorithm

**Performance issues**:
- Reduce maximum structures per tile
- Optimize spatial constraint checking
- Profile structure placement algorithm

---

*The micro-structures system brings the Saltglass Steppe to life with meaningful points of interest that enhance both gameplay and world-building while maintaining the game's data-driven, deterministic architecture.*
