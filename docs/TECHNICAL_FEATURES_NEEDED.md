# Technical Features Needed for Full Content Implementation

This document lists technical systems that need to be implemented to support the expanded content in the data files. These features are referenced in the JSON but not yet coded.

## Progress Tracking

### ✅ COMPLETED

- **Currency System (Salt Scrip)** - Full implementation with add/spend/display methods
- **Faction Reputation System** - Core reputation tracking with modify/get methods and price modifiers
- **Equipment System Expansion** - Already fully implemented with all slots
- **Advanced Item Properties** - Added missing properties (invisibility, reveals, stackable, etc.)
- **Status Effects System** - Complete framework with tick/apply/check methods
- **Basic Trading System** - Price calculation with reputation modifiers
- **Trading System UI** - Full UI with Buy/Sell modes and merchant interaction
- **Light and Vision System** - Complete with glare effects, visibility modifiers, and item interactions
- **Advanced Combat Behaviors** - Implemented reflect_damage, poison_sting, web_trap, drain_sanity, teleport
- **Advanced Dialogue System** - Condition checking for faction/currency/items/level
- **Quest System Expansion** - Already well-implemented with objectives, rewards, and tracking
- **Ritual System** - Complete with requirements, effects, and DES testing support
- **Advanced Map Features** - Hidden locations, safe routes, storm damage tracking
- **Crafting System** - Recipe system, stations, skill requirements
- **Sanity/Mental Health System** - Sanity meter, mental effects, hallucinations

### 🚧 IN PROGRESS

### ⏳ TODO

## Implementation Summary

**Core Systems Implemented:**

1. ✅ Faction reputation with -100 to +100 range
2. ✅ Currency (salt scrip) with trading mechanics
3. ✅ Equipment system with 12 slots
4. ✅ Advanced item properties (8 new properties)
5. ✅ Status effects with duration and stacking
6. ✅ Combat behaviors (5 types implemented)
7. ✅ Dialogue conditions (4 types)
8. ✅ Quest system with multiple objective types

**Ready for Content Integration:**

- All expanded items (89 total) can now use new properties
- All expanded enemies (31 total) can use new behaviors
- All expanded NPCs (20 total) can use dialogue conditions
- Faction system ready for 5 factions
- Quest system ready for 8 complex questlines

## Priority: CRITICAL (Required for Core Gameplay)

### 1. ✅ Faction Reputation System

**Referenced in:** NPCs, quests, dialogue conditions
**Description:** Track player standing with each faction (-100 to +100)
**Implementation needs:**

- ✅ `faction_reputation: HashMap<String, i32>` in GameState
- ✅ Functions: `modify_reputation(faction, delta)`, `get_reputation(faction)`
- ⏳ Dialogue condition: `{"faction_reputation": {"monks": 20}}`
- ✅ Shop pricing modifiers based on reputation

### 2. ✅ Currency System (Salt Scrip)

**Referenced in:** Items, NPCs, shop actions
**Description:** Universal currency for trading
**Implementation needs:**

- ✅ `salt_scrip: u32` field in Player
- ⏳ Display in HUD: "Scrip: 127"
- ✅ Shop buying/selling mechanics
- ⏳ Quest rewards that give currency

### 3. ✅ Equipment System Expansion

**Referenced in:** Items with `equip_slot` field
**Description:** Full equipment slots beyond basic weapon
**Implementation needs:**

- ✅ Equipment slots: head, jacket, boots, gloves, accessory, necklace, backpack, ranged_weapon
- ✅ Stat bonuses from equipped items
- ✅ Visual representation in character screen
- ⏳ Equipment durability and repair mechanics

### 4. ✅ Advanced Item Properties

**Referenced in:** Various items
**Description:** Special item behaviors beyond basic heal/use
**Implementation needs:**

- ✅ `suppresses_adaptations`: Veil Tincture hides adaptations temporarily
- ✅ `reveals_storm_timing`: Storm Chart shows next storm details
- ✅ `grants_invisibility`: Refraction Oil makes player harder to detect
- ✅ `breaks_walls`: Glass Pick can destroy wall tiles
- ✅ `reveals_locations`: Pilgrim Journal shows hidden areas on map
- ✅ `stackable`: Items that can be grouped (Salt Scrip, Glass Dust)
- ✅ `grows_over_time`: Storm Seed becomes Storm Glass after time
- ✅ `light_source`: Items that provide illumination radius

## Priority: HIGH (Enhances Core Systems)

### 5. ✅ Advanced Combat Behaviors

**Referenced in:** Enemy behaviors array
**Description:** Complex enemy AI and special attacks
**Implementation needs:**

- ✅ `reflect_damage`: Percentage of damage reflected back to attacker
- ✅ `teleport`: Enemy can move instantly within range
- ✅ `web_trap`: Immobilizes player for X turns
- ✅ `drain_sanity`: Reduces player mental health
- ✅ `laser_beam`: Ranged attack with specific damage
- ✅ `split_on_death`: Enemy spawns smaller versions when killed
- ✅ `fear_aura`: Causes player to flee or suffer penalties (Implemented as `swarm` aggro)
- `multiple_attacks`: Enemy attacks multiple times per turn

### 6. ✅ Ritual System

**Referenced in:** NPC actions
**Description:** Special ceremonies that transform the player
**Implementation needs:**

- ✅ `triggers_ritual`: "storm_walk", "crucible_transformation"
- ✅ Ritual locations and requirements
- ✅ Permanent effects on player (new adaptations, stat changes)
- ✅ Faction reputation consequences
- ✅ DES testing support with assertions

### 7. ✅ Quest System Expansion

**Referenced in:** NPC actions, dialogue
**Description:** Dynamic quest generation and tracking
**Implementation needs:**

- ✅ `starts_quest`: NPCs can give quests dynamically
- ✅ Quest objectives beyond "kill X enemies" (Added `InterfaceWithAria`)
- ✅ Faction-specific questlines
- ✅ Quest rewards that affect reputation and unlock content

### 8. ✅ Advanced Dialogue System

**Referenced in:** NPC dialogue conditions
**Description:** Complex dialogue trees with multiple conditions
**Implementation needs:**

- ✅ `has_currency`: Check if player has enough money
- ✅ `faction_reputation`: Dialogue based on faction standing
- ✅ `has_item`: Check for specific items in inventory
- ✅ Multiple conditions per dialogue option (AND logic)
- ✅ Dialogue that consumes items or currency

## Priority: MEDIUM (Quality of Life)

### 9. Trading System

**Referenced in:** NPC shop_inventory, trade actions
**Description:** Full merchant interaction system
**Implementation needs:**

- Shop UI with buy/sell interface
- Dynamic pricing based on faction reputation
- Merchant inventory that restocks over time
- Special items only available to certain factions

### 10. ✅ Light and Vision System

**Referenced in:** Items with light_source, enemy behaviors
**Description:** Dynamic lighting affects gameplay
**Implementation needs:**

- ✅ Light radius affects FOV
- ✅ Glare tiles that damage or blind
- ✅ Light-based puzzles and navigation
- ✅ Storm effects on visibility
- ✅ Item interactions based on light level
- ✅ Visibility modifiers for different light levels

### 11. Status Effects System

**Referenced in:** Enemy behaviors, item effects
**Description:** Temporary conditions affecting player
**Implementation needs:**

- Poison, blindness, fear, invisibility
- Duration tracking and visual indicators
- Stacking effects and interactions
- Cures and immunities

## Priority: LOW (Polish and Immersion)

### 12. Advanced Map Features ✅ **COMPLETED**

**Referenced in:** Items that reveal locations
**Description:** Dynamic map discovery and annotation
**Implementation:** ✅ **COMPLETE**

- ✅ Hidden locations revealed by items
- ✅ Safe routes marked by NPCs
- ✅ Storm damage tracking on world map
- ✅ Player annotations and waypoints
- ✅ Data-driven MapFeatures system with full serialization support
- ✅ DES testing support with comprehensive assertions

### 13. Crafting System ✅ **COMPLETED**

**Referenced in:** Items marked as crafting components
**Description:** Combine items to create new ones
**Implementation:** ✅ **COMPLETE**

- ✅ Recipe system using JSON data
- ✅ Crafting stations and requirements
- ✅ Skill-based success rates
- ✅ Faction-specific recipes
- ✅ Enhanced with skill requirements, station dependencies, and faction restrictions
- ✅ Success rate calculation based on player skill vs recipe requirements
- ✅ DES testing support with crafting assertions

### 14. Sanity/Mental Health System ✅ **COMPLETED**

**Referenced in:** Enemy drain_sanity behavior
**Description:** Psychological effects of transformation
**Implementation:** ✅ **COMPLETE**

- ✅ Sanity meter alongside HP (0-100 range)
- ✅ Effects of low sanity (hallucinations, poor decisions, paranoia, despair)
- ✅ Ways to restore mental health (rest, social interaction, meditation)
- ✅ Adaptation effects on sanity with tolerance building
- ✅ Mental effects system with duration and intensity
- ✅ Decision penalties based on sanity level
- ✅ Hallucination generation system
- ✅ DES testing support with comprehensive sanity assertions

## Data-Driven Implementation Notes

All these systems should be implemented with data-driven design principles:

1. **Configuration in JSON**: Behavior parameters, costs, durations should be in data files
2. **Modular Design**: Each system should be a separate module that can be enabled/disabled
3. **Event System**: Use events to decouple systems (e.g., "player_gained_adaptation" event)
4. **Save/Load Support**: All new systems must serialize properly
5. **Testing Support**: Each system should have DES test scenarios

## Implementation Order Recommendation

1. **Currency + Basic Trading** - Enables economic gameplay loop
2. **Equipment System** - Makes loot meaningful
3. **Faction Reputation** - Enables social consequences
4. **Advanced Item Properties** - Makes items feel unique
5. **Combat Behaviors** - Makes enemies interesting
6. **Dialogue System** - Enables rich NPC interactions
7. **Quest System** - Provides structured goals
8. **Ritual System** - Enables transformation mechanics

This order ensures each system builds on previous ones and provides immediate gameplay value.

## NEW SYSTEMS FOR LORE-BASED CONTENT

### Priority: CRITICAL (Required for New Lore Content)

#### 15. Quantum Consciousness System

**Referenced in:** Psychic Codex, new items, advanced NPCs
**Description:** Framework for psychic abilities and quantum-based powers
**Implementation needs:**

- Five categories of psychic abilities (telepathy, probability, energy, phasing, temporal)
- Stage-based development tied to adaptation levels
- Quantum coherence as resource system
- Psychic ability cooldowns and limitations
- Mental strain and quantum psychosis mechanics

#### 16. ARIA Communication System

**Referenced in:** Archive Consciousness lore, ARIA Interface item
**Description:** Direct dialogue system with the distributed AI
**Implementation needs:**

- Multi-personality AI responses based on Archive facility type
- Dynamic dialogue that changes based on player's adaptation level
- ARIA influence on world events and other NPCs
- Archive facility control through ARIA interface
- AI memory of previous interactions

#### 17. Cosmic Entity Encounters

**Referenced in:** Quantum Collective Scout, Void Swimmer enemies
**Description:** Special encounter system for otherworldly beings
**Implementation needs:**

- Rare spawn conditions tied to player adaptation level
- Reality distortion effects during encounters
- Cosmic entity communication attempts
- Dimensional attack patterns and defenses
- Consequences for cosmic contact events

#### 18. Consciousness Backup System

**Referenced in:** Consciousness Backup item, Archive facilities
**Description:** Mind uploading and digital consciousness storage
**Implementation needs:**

- Player consciousness scanning and storage
- Backup restoration after death (with consequences)
- Digital consciousness interaction with ARIA
- Memory fragmentation and identity issues
- Archive facility integration for backup storage

### Priority: HIGH (Enhances Lore Integration)

#### 19. Null Field Technology

**Referenced in:** Iron Covenant faction, Null Field Generator item
**Description:** Anti-quantum technology that suppresses abilities
**Implementation needs:**

- Null field area effects that disable psychic abilities
- Adaptation suppression and reversal mechanics
- Iron Covenant equipment with null field properties
- Null field resistance for highly adapted characters
- Environmental null field zones

#### 20. Forced Adaptation System

**Referenced in:** Glass Prophet faction, Ascension Catalyst item
**Description:** Accelerated transformation mechanics
**Implementation needs:**

- Rapid adaptation progression with risks
- Forced adaptation events and consequences
- Adaptation resistance and acceptance mechanics
- Transformation trauma and psychological effects
- Emergency adaptation in extreme situations

#### 21. Memory Crystal Technology

**Referenced in:** Wandering Court faction, Memory Crystal item
**Description:** Cultural preservation and memory sharing system
**Implementation needs:**

- Pre-storm memory storage and playback
- Cultural knowledge database access
- Memory sharing between characters
- Lost knowledge recovery mechanics
- Nostalgia and identity preservation themes

#### 22. Quantum Field Manipulation

**Referenced in:** Various lore documents, storm mechanics
**Description:** Environmental quantum field effects
**Implementation needs:**

- Quantum field strength measurement and display
- Field manipulation through items and abilities
- Environmental hazards from quantum instability
- Quantum field interactions with technology
- Storm prediction through field analysis

### Priority: MEDIUM (Atmospheric and Immersion)

#### 23. Timeline Perception System

**Referenced in:** Psychic Codex temporal abilities
**Description:** Time-based psychic abilities and consequences
**Implementation needs:**

- Precognition mechanics for seeing future events
- Retrocognition for viewing past events
- Temporal displacement risks and effects
- Timeline stability and paradox prevention
- Temporal anchor items for protection

#### 24. Collective Consciousness Network

**Referenced in:** Glassborn faction, hive mind mechanics
**Description:** Shared consciousness between adapted individuals
**Implementation needs:**

- Mental network joining and leaving mechanics
- Shared knowledge and experience pools
- Individual identity preservation vs. collective merger
- Network communication and coordination
- Collective decision-making processes

#### 25. Reality Editing System

**Referenced in:** ARIA Manifestation, storm mechanics
**Description:** Direct manipulation of physical reality
**Implementation needs:**

- Terrain modification during encounters
- Reality stability measurement
- Editing resistance and backlash effects
- Environmental storytelling through edits
- Player reality manipulation abilities

#### 26. Cosmic Contact Protocol

**Referenced in:** Heliograph Expedition lore
**Description:** Framework for interaction with alien intelligences
**Implementation needs:**

- Communication attempt mechanics
- Cosmic entity relationship tracking
- Contact preparation and readiness systems
- Cosmic threat assessment and response
- Post-contact world state changes

### Priority: LOW (Polish and Depth)

#### 27. Cultural Preservation Mechanics

**Referenced in:** Wandering Court, pre-storm artifacts
**Description:** Maintaining human culture through transformation
**Implementation needs:**

- Cultural knowledge skill system
- Tradition preservation activities
- Cultural artifact collection and study
- Identity crisis and resolution mechanics
- Cultural exchange between factions

#### 28. Quantum Biology Research

**Referenced in:** Dr. Kira Thorne, research collaboration
**Description:** Scientific study of adaptation processes
**Implementation needs:**

- Research project system with NPCs
- Data collection and analysis mechanics
- Scientific breakthrough discoveries
- Research collaboration benefits
- Ethical considerations in experimentation

## IMPLEMENTATION PRIORITY FOR LORE CONTENT

1. **Quantum Consciousness System** - Enables psychic abilities from lore
2. **ARIA Communication System** - Allows interaction with Archive AI
3. **Null Field Technology** - Enables Iron Covenant faction mechanics
4. **Forced Adaptation System** - Supports Glass Prophet faction
5. **Cosmic Entity Encounters** - Adds otherworldly threat level
6. **Memory Crystal Technology** - Enables cultural preservation themes
7. **Consciousness Backup System** - Archive-based resurrection mechanics
8. **Quantum Field Manipulation** - Environmental storytelling tool

This order ensures each system builds on previous ones and provides immediate gameplay value.

## NEW SYSTEMS FOR HIGH-PRIORITY LORE DOCUMENTS

### Priority: CRITICAL (Required for Atlas of Glass)

#### 29. Regional Navigation System

**Referenced in:** Atlas of Glass, quantum coordinates, trade routes
**Description:** Advanced navigation and location discovery system
**Implementation needs:**

- Quantum coordinate system (QC format) for stable location references
- Regional stability ratings (Stable, Semi-Stable, Variable, Chaotic)
- Dynamic location accessibility based on seasons and storms
- Trade route system with waypoints and travel time calculations
- Hazard zone detection and warning systems

#### 30. World Map Overhaul

**Referenced in:** Atlas locations, territorial boundaries, faction zones
**Description:** Enhanced world map with detailed regional information
**Implementation needs:**

- Faction territory boundaries and control zones
- Major landmark placement (Nexus Plateau, Glass Gardens, etc.)
- Settlement locations with population and services data
- Trade route visualization with safety ratings
- Seasonal map changes and storm damage tracking

#### 31. Travel Time and Distance System

**Referenced in:** Atlas travel times, route planning
**Description:** Realistic travel mechanics with environmental factors
**Implementation needs:**

- Distance calculation between major locations
- Travel speed modifiers (terrain, weather, equipment)
- Route planning with multiple path options
- Travel hazard encounters and random events
- Supply consumption during long journeys

### Priority: CRITICAL (Required for Technical Codex)

#### 32. Technology Classification System

**Referenced in:** Technical Codex equipment categories
**Description:** Comprehensive technology identification and interaction
**Implementation needs:**

- Technology types (Legacy, Hybrid, Native, Archive, Quantum)
- Equipment compatibility matrix and interaction rules
- Maintenance requirements and degradation mechanics
- Power consumption tracking (Storm Glass grades)
- Technology malfunction and repair systems

#### 33. Saint-Key Authentication System

**Referenced in:** Technical Codex, Archive access protocols
**Description:** Credential-based access control for Archive systems
**Implementation needs:**

- Saint-Key item with access level properties (1-5)
- Biometric verification and consciousness scanning
- Archive facility access restrictions and permissions
- Emergency override protocols for crisis situations
- Key bonding process and security features

#### 34. Storm Glass Processing System

**Referenced in:** Technical Codex, Merchant Protocols economy
**Description:** Storm Glass refinement and grading mechanics
**Implementation needs:**

- Raw Storm Glass collection and sorting
- Processing equipment and facility requirements
- Grading system (1-4 Thorne Units) with quality assessment
- Purification and stabilization procedures
- Quality control and contamination detection

### Priority: HIGH (Required for Living Steppe)

#### 35. Quantum Flora and Fauna System

**Referenced in:** Living Steppe species guide, ecological interactions
**Description:** Advanced creature and plant behavior system
**Implementation needs:**

- Quantum-adapted species with special properties
- Ecological relationships and food chain mechanics
- Seasonal behavior changes and migration patterns
- Resource harvesting from plants and creatures
- Conservation mechanics and sustainability tracking

#### 36. Environmental Phenomena System

**Referenced in:** Living Steppe weather patterns, quantum effects
**Description:** Dynamic environmental events and hazards
**Implementation needs:**

- Probability storms with multiple reality states
- Consciousness fog causing psychic interference
- Reality ripples with space-time distortions
- Environmental hazard detection and protection
- Seasonal environmental changes and adaptations

#### 37. Resource Management System

**Referenced in:** Living Steppe harvesting, conservation practices
**Description:** Sustainable resource extraction and ecosystem health
**Implementation needs:**

- Renewable resource tracking (plants, water, materials)
- Harvesting quotas and regeneration timers
- Ecosystem health monitoring and consequences
- Conservation practices and environmental protection
- Resource quality assessment and contamination detection

### Priority: HIGH (Required for Medical Compendium)

#### 38. Advanced Medical System

**Referenced in:** Medical Compendium, adaptation physiology
**Description:** Comprehensive healthcare for transformed humans
**Implementation needs:**

- Adaptation stage tracking (0-100% transformation)
- Medical conditions specific to quantum adaptation
- Treatment protocols for consciousness disorders
- Medical facility specializations and capabilities
- Emergency medical procedures and triage systems

#### 39. Consciousness Disorder System

**Referenced in:** Medical Compendium, quantum poisoning, fragmentation
**Description:** Mental health mechanics for transformed individuals
**Implementation needs:**

- Consciousness fragmentation and identity disorders
- Quantum poisoning symptoms and treatment
- Reality displacement and temporal confusion
- Consciousness stabilization and anchoring techniques
- Medical ethics and consent in transformation medicine

#### 40. Specialized Medical Facilities

**Referenced in:** Medical Compendium facility descriptions
**Description:** Faction-specific healthcare systems and capabilities
**Implementation needs:**

- Archive Medical Division with AI assistance
- Sand-Engineer medical stations with technology focus
- Mirror Monk healing centers with spiritual approach
- Glassborn healing circles with transformation expertise
- Medical facility access requirements and services

### Priority: MEDIUM (Required for Calendar of Light)

#### 41. Temporal System Overhaul

**Referenced in:** Calendar of Light, seasonal cycles, storm timing
**Description:** Comprehensive time and calendar system
**Implementation needs:**

- 365-day calendar with three seasons (Storm, Calm, Flux)
- Storm prediction system with 47-day cycles
- Seasonal activity restrictions and opportunities
- Historical timeline tracking (Year 0-23 PT)
- Temporal phenomena and reality distortion effects

#### 42. Storm Prediction and Warning System

**Referenced in:** Calendar storm cycles, prediction methods
**Description:** Advanced storm forecasting and community alerts
**Implementation needs:**

- Storm intensity levels (1-5) with different effects
- Prediction accuracy based on methods and equipment
- Community warning protocols and communication networks
- Environmental indicators and biological responses
- Storm preparation activities and safety measures

#### 43. Faction Calendar Integration

**Referenced in:** Calendar observances, cultural events
**Description:** Faction-specific holidays and cultural practices
**Implementation needs:**

- Universal observances (White Noon Memorial, Storm's End)
- Faction-specific holidays and celebrations
- Cultural event scheduling and community participation
- Religious and spiritual observance mechanics
- Social bonding and community relationship effects

### Priority: LOW (Polish and Immersion)

#### 44. Location-Based Services System

**Referenced in:** Atlas settlements, facility capabilities
**Description:** Dynamic services available at different locations
**Implementation needs:**

- Settlement service availability (medical, trade, repair)
- Facility specialization and unique capabilities
- Service quality ratings and reputation effects
- Access requirements and faction restrictions
- Service pricing and availability fluctuations

#### 45. Equipment Maintenance System

**Referenced in:** Technical Codex maintenance requirements
**Description:** Technology upkeep and degradation mechanics
**Implementation needs:**

- Equipment durability and wear tracking
- Maintenance schedules and requirements
- Repair services and facility capabilities
- Component replacement and upgrade systems
- Technology failure consequences and emergency procedures

#### 46. Ecological Research System

**Referenced in:** Living Steppe research priorities, citizen science
**Description:** Scientific study and documentation of transformed ecosystem
**Implementation needs:**

- Species observation and documentation mechanics
- Research collaboration with NPCs and factions
- Data collection and analysis activities
- Conservation project participation
- Scientific discovery and knowledge advancement

## IMPLEMENTATION PRIORITY FOR NEW LORE CONTENT

**Phase 1 (Critical Systems):**

1. **Regional Navigation System** - Enables Atlas-based exploration
2. **Technology Classification System** - Supports Technical Codex equipment
3. **Advanced Medical System** - Enables healthcare mechanics
4. **Temporal System Overhaul** - Implements Calendar of Light

**Phase 2 (High Priority Systems):** 5. **World Map Overhaul** - Visual representation of Atlas locations 6. **Saint-Key Authentication System** - Archive access mechanics 7. **Quantum Flora and Fauna System** - Living Steppe creatures 8. **Storm Prediction and Warning System** - Calendar storm mechanics

**Phase 3 (Enhancement Systems):** 9. **Environmental Phenomena System** - Living Steppe weather 10. **Consciousness Disorder System** - Medical Compendium conditions 11. **Storm Glass Processing System** - Technical Codex equipment 12. **Faction Calendar Integration** - Cultural observances

**Phase 4 (Polish Systems):** 13. **Travel Time and Distance System** - Atlas navigation mechanics 14. **Resource Management System** - Living Steppe conservation 15. **Specialized Medical Facilities** - Medical Compendium locations 16. **Location-Based Services System** - Atlas settlement services

This implementation order ensures that the most critical systems for gameplay are developed first, with each phase building on the previous one to create a comprehensive and immersive world system.
