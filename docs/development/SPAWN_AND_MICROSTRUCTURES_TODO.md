# Spawn System and Micro-Structures Implementation TODO

**Created:** 2025-12-29  
**Status:** In Progress  

## Overview

Rework mob/NPC spawning system and implement procedural micro-structures with associated NPCs, chests, and loot systems.

## Research Findings

Based on research into games like Caves of Qud, RimWorld, and other roguelikes:
- **Micro-structures** should be small, thematic clusters (3-8 tiles)
- **Spawn distribution** should use spatial algorithms to avoid clustering
- **Loot tables** should be data-driven and contextual to structure type
- **NPCs** should have logical associations with their structures

## Implementation Status

**COMPLETED MAJOR SYSTEMS:**
1. ✅ **Spatial Spawn Distribution** - Fixed enemy clustering with 20-tile minimum separation
2. ✅ **Comprehensive Chest System** - 5 chest types with data-driven loot tables
3. ✅ **Micro-Structures System** - 6 structure types with biome-specific placement
4. ✅ **NPC Integration** - 5 new NPCs with structure-specific roles
5. ✅ **Documentation** - Complete technical documentation for both systems

**REMAINING TASKS:**
- Chest UI system for player interaction
- Advanced NPC behavior patterns
- Structure-specific dialogue integration

## Tasks

### 1. Fix Mob/NPC Spawn Distribution
- [x] **Analyze current spawn system** - Identified clustering due to old room-based logic
- [x] **Implement spatial distribution** - Created Poisson disk sampling and grid-based distribution
- [x] **Add spawn constraints** - Minimum distance between hostile mobs (20 tiles)
- [x] **DES test** - Create test scenario for spawn distribution
- [x] **Document** - Update spawn system documentation

### 2. Implement Chest System
- [x] **Create Chest entity** - Basic chest with inventory storage
- [ ] **Chest UI system** - Transfer interface between player and chest inventory
- [x] **Chest types** - Multiple chest variants (wooden, metal, glass, etc.)
- [x] **Loot table system** - Data-driven loot generation
- [x] **DES test** - Test chest interaction and loot generation
- [x] **Document** - Chest system implementation guide

### 3. Design Micro-Structure Types
Saltglass Steppe-appropriate micro-structures:
- [x] **Scavenger Camp** - Bedroll, fire pit, supply cache (trader NPC)
- [x] **Abandoned Outpost** - Ruined walls, broken equipment (hostile mobs)
- [x] **Glass Garden** - Cultivated glass formations (hermit NPC)
- [x] **Storm Shelter** - Reinforced hut with supplies (neutral NPC)
- [x] **Shrine Remnant** - Broken altar, scripture fragments (monk NPC)
- [x] **Salt Harvester** - Evaporation pools, tools (worker NPC)

### 4. Implement Micro-Structure Generation
- [x] **Structure templates** - Define layouts and tile patterns
- [x] **Placement algorithm** - Spatial distribution avoiding overlap
- [x] **Biome integration** - Structure types appropriate to terrain
- [x] **Data-driven config** - JSON templates for easy content expansion
- [x] **DES test** - Test structure generation and placement
- [x] **Document** - Micro-structure system guide

### 5. Integrate NPCs with Structures
- [x] **NPC-structure associations** - Link NPC types to structure types
- [x] **Contextual spawning** - NPCs spawn with appropriate structures
- [ ] **Behavior patterns** - NPCs interact with their environment
- [ ] **Dialogue integration** - Structure-specific conversation topics
- [x] **DES test** - Test NPC-structure relationships
- [x] **Document** - NPC integration guide

### 6. Loot Table System
- [ ] **Loot table format** - JSON schema for item generation
- [ ] **Contextual loot** - Different tables per structure/chest type
- [ ] **Rarity system** - Weighted item selection
- [ ] **Condition system** - Item quality/durability variants
- [ ] **DES test** - Test loot generation consistency
- [ ] **Document** - Loot system documentation

## Implementation Priority

1. **High Priority:** Fix spawn distribution (affects current gameplay)
2. **High Priority:** Implement chest system (foundation for loot)
3. **Medium Priority:** Basic micro-structures (camps, shelters)
4. **Medium Priority:** NPC integration
5. **Low Priority:** Advanced structures and specialized loot

## Technical Considerations

- **Decoupled systems:** Separate generation, spawning, and interaction logic
- **Data-driven:** Use JSON configs for easy content expansion
- **Deterministic:** Ensure reproducible generation with seeds
- **Performance:** Efficient spatial algorithms for large maps
- **Extensible:** Easy to add new structure types and loot tables

## Success Criteria

- [ ] Mobs no longer cluster in spawn locations
- [ ] Micro-structures generate naturally across terrain
- [ ] Chests provide meaningful loot progression
- [ ] NPCs feel contextually appropriate to their locations
- [ ] System is easily extensible for new content

---

**Next Update:** After completing spawn distribution fix
