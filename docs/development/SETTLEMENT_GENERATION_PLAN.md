# Settlement Generation Implementation Plan

**Date**: 2026-02-21  
**Estimate**: 15-20 hours  
**Dependencies**: Faction system ✅, Overworld travel ✅  
**Status**: Planning

---

## Executive Summary

Implement procedural settlement generation (villages, towns, cities) integrated with the faction system and world map. Settlements will be generated on-demand when the player enters a POI marked as "Town" on the world map, using terrain-forge algorithms and faction-specific theming.

---

## Research Findings

### Terrain-Forge Capabilities

**Available Algorithms** (from v0.7.0):
- `bsp` - Binary Space Partitioning (structured rooms/blocks)
- `rooms` - Simple room placement
- `voronoi` - Voronoi diagram regions
- `cellular` - Organic caves (not suitable for settlements)
- `maze` - Perfect mazes (not suitable)
- `room_accretion` - Brogue-style organic dungeons

**Relevant Features**:
- **Semantic Layers**: Markers for entity spawning (PlayerStart, Exit, Treasure, etc.)
- **Prefab System**: JSON-defined building templates with transformations
- **Requirements System**: Generate maps meeting specific constraints
- **Connectivity Utilities**: Glass Seam Bridging for connecting regions
- **Grid Operations**: Flood fill, region labeling, pathfinding

**Best Fit for Settlements**:
1. **BSP** - For structured city blocks and street grids
2. **Voronoi** - For organic village layouts with central plaza
3. **Prefab System** - For faction-specific buildings
4. **Semantic Markers** - For NPC spawns, shops, quest locations

### Settlement Generation Patterns (Research)

**Recursive Subdivision Algorithm** (from roguelike community):
1. Start with empty map
2. Pick center point, draw cross streets (horizontal + vertical)
3. Recursively subdivide each block with more streets
4. Stop at minimum block size
5. Place buildings in final blocks

**Voronoi-Based Approach**:
1. Place seed points for important buildings (town hall, temple, market)
2. Generate Voronoi regions around seeds
3. Connect regions with roads
4. Fill regions with smaller buildings

**Agent-Based Generation**:
1. Place initial buildings (town center)
2. Roads attract settlers
3. New houses lead to road extensions
4. Iterative growth simulation

**Recommendation**: Hybrid approach using BSP for structured towns/cities and Voronoi for organic villages.

---

## Design Specification

### Settlement Tiers

| Tier | Size | Buildings | Population | Faction Control |
|------|------|-----------|------------|-----------------|
| **Village** | 40×30 | 5-10 | 20-50 | Single faction dominant |
| **Town** | 80×60 | 15-30 | 100-300 | 1-2 factions, contested |
| **City** | 120×90 | 40-80 | 500+ | Multi-faction, complex |

### Building Types

**Core Buildings** (all settlements):
- Town Hall / Meeting House
- General Store / Trader
- Inn / Tavern
- Temple / Shrine
- Residential Houses

**Faction-Specific Buildings**:
- **Mirror Monks**: Light Temple, Meditation Chamber, Scripture Archive
- **Glassborn**: Crystal Forge, Transformation Clinic, Shimmer Gallery
- **Sand Engineers**: Workshop, Foundry, Mechanical Lab
- **Salt Traders**: Trading Post, Caravan Station, Brine Warehouse
- **Storm Cults**: Storm Shrine, Ritual Circle, Tempest Observatory
- **Refraction Outcasts**: Hidden Safehouse, Mutation Clinic, Underground Market
- **Archive Drones**: Data Vault, Preservation Chamber, Knowledge Repository

**Tier-Specific Buildings**:
- **Village**: Basic services only (store, inn, temple)
- **Town**: + Blacksmith, Healer, Quest Board, Faction Outpost
- **City**: + Multiple faction buildings, Arena, Library, Market District

### Generation Algorithm

**Phase 1: Layout Generation**
1. Determine settlement tier from world map POI
2. Select algorithm based on tier:
   - Village: Voronoi (organic)
   - Town: BSP (structured)
   - City: BSP with districts
3. Generate base layout using terrain-forge
4. Extract semantic markers for building placement

**Phase 2: Faction Integration**
1. Query world map for faction territory
2. Determine faction control (dominant faction + minorities)
3. Assign building types based on faction presence
4. Apply faction-specific theming (colors, descriptions)

**Phase 3: Building Placement**
1. Place core buildings at semantic markers
2. Place faction buildings based on control percentage
3. Fill remaining spaces with residential buildings
4. Add decorative elements (gardens, fountains, statues)

**Phase 4: NPC Population**
1. Spawn NPCs based on building types
2. Assign faction affiliations
3. Generate dialogue trees
4. Place quest givers and vendors

**Phase 5: Connectivity**
1. Ensure all buildings are accessible
2. Add roads/paths between buildings
3. Place entrance/exit markers
4. Connect to world map

---

## Technical Implementation

### File Structure

```
src/game/generation/
├── settlement/
│   ├── mod.rs              # Public API
│   ├── layout.rs           # Layout generation (BSP/Voronoi)
│   ├── buildings.rs        # Building placement and types
│   ├── faction_theme.rs    # Faction-specific theming
│   ├── population.rs       # NPC spawning
│   └── prefabs/            # Building prefab definitions
│       ├── core.json       # Core buildings
│       ├── mirror_monks.json
│       ├── glassborn.json
│       └── [other factions].json

data/
├── settlement_config.json  # Settlement generation parameters
└── building_types.json     # Building definitions
```

### Data Structures

```rust
// src/game/generation/settlement/mod.rs

pub struct SettlementConfig {
    pub tier: SettlementTier,
    pub size: (usize, usize),
    pub faction_control: HashMap<String, f32>, // faction_id -> control %
    pub biome: Biome,
    pub seed: u64,
}

pub enum SettlementTier {
    Village,
    Town,
    City,
}

pub struct Settlement {
    pub map: Map, // Reuse existing Map type
    pub buildings: Vec<Building>,
    pub npcs: Vec<NPC>,
    pub faction_control: HashMap<String, f32>,
}

pub struct Building {
    pub building_type: BuildingType,
    pub bounds: Rectangle,
    pub faction_affiliation: Option<String>,
    pub entrance: (i32, i32),
    pub interior_map: Option<Map>, // For enterable buildings
}

pub enum BuildingType {
    TownHall,
    GeneralStore,
    Inn,
    Temple,
    Residential,
    FactionSpecific(String), // faction_id
}
```

### Integration Points

**World Map** (`src/game/world_map.rs`):
- Add `settlement_tier` field to POI
- Generate settlement tier based on faction control and biome

**Map Generation** (`src/game/map.rs`):
- Add `generate_settlement()` function
- Hook into existing `generate_from_world()` when POI is Town

**Game State** (`src/game/state.rs`):
- Track current settlement
- Handle building entry/exit
- Manage settlement NPCs

**Faction System** (`src/game/faction.rs`):
- Query faction control for tile
- Apply reputation modifiers in settlements
- Faction-specific NPC behavior

---

## Implementation Phases

### Phase 1: Core Infrastructure (4-5 hours)

**Tasks**:
1. Create `src/game/generation/settlement/` module structure
2. Define data structures (`SettlementConfig`, `Settlement`, `Building`)
3. Create `data/settlement_config.json` with tier parameters
4. Create `data/building_types.json` with building definitions
5. Integrate terrain-forge BSP and Voronoi algorithms
6. Write basic layout generation (no buildings yet)

**Deliverables**:
- Empty settlement maps generated with correct size
- BSP/Voronoi layout selection based on tier
- Semantic markers extracted

**Testing**:
- CLI tool: `cargo run --bin mapgen-tool settlement <seed> <tier>`
- DES scenario: Generate settlement and verify layout

### Phase 2: Building Placement (3-4 hours)

**Tasks**:
1. Implement building placement at semantic markers
2. Create core building prefabs (town hall, store, inn, temple)
3. Add residential building generation
4. Implement building bounds and entrance detection
5. Add roads/paths between buildings
6. Ensure connectivity (all buildings accessible)

**Deliverables**:
- Settlements with placed buildings
- Roads connecting buildings
- Entrance markers for each building

**Testing**:
- Visual inspection via mapgen-tool
- DES scenario: Verify all buildings accessible

### Phase 3: Faction Integration (3-4 hours)

**Tasks**:
1. Query faction control from world map
2. Implement faction-specific building selection
3. Create faction building prefabs (7 factions × 3 buildings = 21 prefabs)
4. Apply faction theming (colors, descriptions)
5. Adjust building distribution based on faction control

**Deliverables**:
- Faction-specific buildings in settlements
- Faction control affects building types
- Faction theming applied

**Testing**:
- Generate settlements in different faction territories
- Verify faction buildings appear correctly
- DES scenario: Test faction control percentages

### Phase 4: NPC Population (2-3 hours)

**Tasks**:
1. Implement NPC spawning based on building types
2. Assign faction affiliations to NPCs
3. Generate basic dialogue trees
4. Place vendors and quest givers
5. Add NPC schedules (optional, if time permits)

**Deliverables**:
- NPCs spawned in settlements
- Faction-affiliated NPCs
- Vendors and quest givers functional

**Testing**:
- Enter settlement and verify NPCs present
- Talk to NPCs and verify dialogue
- DES scenario: Test NPC spawning

### Phase 5: Polish & Integration (3-4 hours)

**Tasks**:
1. Add decorative elements (gardens, fountains, statues)
2. Implement building interiors (for key buildings)
3. Add settlement-specific encounters
4. Integrate with save/load system
5. Write comprehensive documentation
6. Create DES test scenarios
7. Balance and tuning

**Deliverables**:
- Polished settlement generation
- Building interiors for key buildings
- Documentation in `docs/features/SETTLEMENT_GENERATION.md`
- DES test suite

**Testing**:
- Full playthrough: Travel to settlement, enter buildings, talk to NPCs
- Save/load in settlement
- DES regression suite

---

## Data File Specifications

### `data/settlement_config.json`

```json
{
  "tiers": {
    "village": {
      "size": [40, 30],
      "min_buildings": 5,
      "max_buildings": 10,
      "algorithm": "voronoi",
      "core_buildings": ["town_hall", "general_store", "inn", "temple"],
      "residential_ratio": 0.6
    },
    "town": {
      "size": [80, 60],
      "min_buildings": 15,
      "max_buildings": 30,
      "algorithm": "bsp",
      "core_buildings": ["town_hall", "general_store", "inn", "temple", "blacksmith", "healer"],
      "residential_ratio": 0.5,
      "faction_building_ratio": 0.2
    },
    "city": {
      "size": [120, 90],
      "min_buildings": 40,
      "max_buildings": 80,
      "algorithm": "bsp",
      "core_buildings": ["town_hall", "general_store", "inn", "temple", "blacksmith", "healer", "arena", "library"],
      "residential_ratio": 0.4,
      "faction_building_ratio": 0.3
    }
  },
  "faction_buildings": {
    "MirrorMonks": ["light_temple", "meditation_chamber", "scripture_archive"],
    "Glassborn": ["crystal_forge", "transformation_clinic", "shimmer_gallery"],
    "SandEngineers": ["workshop", "foundry", "mechanical_lab"],
    "SaltTraders": ["trading_post", "caravan_station", "brine_warehouse"],
    "StormCults": ["storm_shrine", "ritual_circle", "tempest_observatory"],
    "RefractionOutcasts": ["hidden_safehouse", "mutation_clinic", "underground_market"],
    "ArchiveDrones": ["data_vault", "preservation_chamber", "knowledge_repository"]
  }
}
```

### `data/building_types.json`

```json
{
  "buildings": [
    {
      "id": "town_hall",
      "name": "Town Hall",
      "size": [10, 8],
      "entrance_side": "south",
      "has_interior": true,
      "npc_count": 3,
      "npc_types": ["mayor", "clerk", "guard"]
    },
    {
      "id": "general_store",
      "name": "General Store",
      "size": [8, 6],
      "entrance_side": "south",
      "has_interior": true,
      "npc_count": 1,
      "npc_types": ["merchant"]
    },
    {
      "id": "inn",
      "name": "Inn",
      "size": [12, 10],
      "entrance_side": "south",
      "has_interior": true,
      "npc_count": 2,
      "npc_types": ["innkeeper", "patron"]
    }
  ]
}
```

---

## Terrain-Forge Integration

### Using BSP for Structured Layouts

```rust
use terrain_forge::{Grid, ops, algorithms::Bsp};

fn generate_town_layout(seed: u64, size: (usize, usize)) -> Grid {
    let mut grid = Grid::new(size.0, size.1);
    
    // Generate BSP layout
    ops::generate("bsp", &mut grid, Some(seed), None).unwrap();
    
    // Extract semantic markers for building placement
    let semantic = SemanticExtractor::for_rooms().extract(&grid, &mut rng);
    
    grid
}
```

### Using Voronoi for Organic Villages

```rust
use terrain_forge::{Grid, ops};

fn generate_village_layout(seed: u64, size: (usize, usize)) -> Grid {
    let mut grid = Grid::new(size.0, size.1);
    
    // Generate Voronoi layout
    ops::generate("voronoi", &mut grid, Some(seed), None).unwrap();
    
    grid
}
```

### Using Prefabs for Buildings

```rust
use terrain_forge::algorithms::{Prefab, PrefabLibrary, PrefabPlacer};

fn place_buildings(grid: &mut Grid, buildings: Vec<BuildingType>, seed: u64) {
    let mut library = PrefabLibrary::new();
    
    // Load building prefabs
    for building in buildings {
        let prefab = load_building_prefab(building);
        library.add_prefab(prefab);
    }
    
    // Place buildings
    let placer = PrefabPlacer::new(PrefabConfig::default(), library);
    placer.generate(grid, seed);
}
```

---

## Risks & Mitigations

### Risk 1: Terrain-Forge Learning Curve
**Impact**: Medium  
**Likelihood**: Medium  
**Mitigation**: 
- Study terrain-forge examples and documentation
- Start with simple BSP/Voronoi before advanced features
- Allocate extra time for experimentation

### Risk 2: Faction Integration Complexity
**Impact**: High  
**Likelihood**: Low  
**Mitigation**:
- Faction system already complete and tested
- Use existing faction query functions
- Keep faction theming simple initially

### Risk 3: Building Interior Generation
**Impact**: Medium  
**Likelihood**: Medium  
**Mitigation**:
- Make interiors optional for Phase 1-4
- Reuse existing dungeon generation for interiors
- Limit interiors to key buildings only

### Risk 4: Performance with Large Cities
**Impact**: Medium  
**Likelihood**: Low  
**Mitigation**:
- Generate settlements on-demand (not all at once)
- Cache generated settlements in save file
- Use terrain-forge's optimized algorithms

### Risk 5: NPC Dialogue Complexity
**Impact**: Low  
**Likelihood**: Medium  
**Mitigation**:
- Reuse existing dialogue system
- Start with simple generic dialogue
- Faction-specific dialogue can be added later

---

## Success Criteria

### Minimum Viable Product (MVP)
- [ ] Generate village, town, and city layouts
- [ ] Place core buildings (town hall, store, inn, temple)
- [ ] Faction-specific buildings appear in faction territories
- [ ] NPCs spawn in settlements
- [ ] Player can enter/exit settlements from world map
- [ ] Save/load works with settlements

### Stretch Goals
- [ ] Building interiors for all key buildings
- [ ] NPC schedules (day/night routines)
- [ ] Settlement-specific quests
- [ ] Dynamic settlement growth over time
- [ ] Faction wars affect settlement control

---

## Testing Strategy

### Unit Tests
- Settlement tier determination
- Building placement logic
- Faction control calculation
- NPC spawning logic

### Integration Tests
- Generate settlement from world map POI
- Enter/exit settlement
- Save/load with settlement data
- Faction reputation affects NPC behavior

### DES Scenarios
- `settlement_generation_test.des` - Generate all tiers
- `faction_settlement_test.des` - Verify faction buildings
- `settlement_npc_test.des` - Test NPC spawning
- `settlement_save_load_test.des` - Save/load round-trip

### Manual Testing
- Visual inspection of generated settlements
- Playthrough: Travel to settlement, explore, interact
- Verify faction theming is consistent
- Check performance with large cities

---

## Documentation Deliverables

1. **Feature Documentation**: `docs/features/SETTLEMENT_GENERATION.md`
   - User-facing feature description
   - How to find and enter settlements
   - Building types and NPCs

2. **Implementation Summary**: `docs/features/SETTLEMENT_GENERATION_SUMMARY.md`
   - Technical implementation details
   - Code changes and file modifications
   - Testing results

3. **API Documentation**: Rustdoc comments in code
   - Public functions and structs
   - Usage examples

4. **Roadmap Update**: Mark settlement generation as complete

---

## Open Questions

1. **Should settlements be persistent or regenerated each visit?**
   - Persistent: Better for player memory, requires save storage
   - Regenerated: Simpler, but confusing for players
   - **Recommendation**: Persistent (cache in save file)

2. **How many faction buildings per settlement?**
   - Proportional to faction control percentage
   - Minimum 1 if faction has >25% control
   - **Recommendation**: 1-3 buildings per faction based on control

3. **Should all buildings have interiors?**
   - Full interiors: More immersive, more work
   - Key buildings only: Faster implementation
   - **Recommendation**: Key buildings only (town hall, inn, faction buildings)

4. **How to handle multi-faction settlements?**
   - Separate districts per faction
   - Mixed buildings throughout
   - **Recommendation**: Mixed with faction clustering (similar factions near each other)

5. **Should settlements have walls/defenses?**
   - Adds visual interest and realism
   - Requires additional generation logic
   - **Recommendation**: Add in polish phase if time permits

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Adjust estimates** based on feedback
3. **Create TODO list** with all tasks
4. **Begin Phase 1** implementation
5. **Iterate** based on testing results

---

## References

- [Terrain-Forge Documentation](https://docs.rs/terrain-forge)
- [Terrain-Forge Usage Guide](../../../terrain-forge/USAGE.md)
- [Roguelike Town Generation Algorithm](https://groups.google.com/g/rec.games.roguelike.development/c/rIgTZhg3D0E)
- [Procedural Generation of Villages on Arbitrary Terrains](https://www.researchgate.net/publication/257406695_Procedural_Generation_of_Villages_on_Arbitrary_Terrains)
- [Saltglass Steppe Faction System](../features/FACTION_SYSTEM.md)
- [Saltglass Steppe Roadmap](ROADMAP.md)
