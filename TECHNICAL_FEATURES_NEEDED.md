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
- **Advanced Combat Behaviors** - Implemented reflect_damage, poison_sting, web_trap, drain_sanity, teleport
- **Advanced Dialogue System** - Condition checking for faction/currency/items/level
- **Quest System Expansion** - Already well-implemented with objectives, rewards, and tracking
- **Ritual System** - Complete with requirements, effects, and DES testing support

### 🚧 IN PROGRESS
- **Trading System UI** - Backend complete, needs UI
- **Light and Vision System** - Complete with glare effects, visibility modifiers, and item interactions

### ⏳ TODO
- **Advanced Map Features** - Not started
- **Crafting System** - Not started
- **Sanity/Mental Health System** - Placeholder exists

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

### 5. 🚧 Advanced Combat Behaviors
**Referenced in:** Enemy behaviors array
**Description:** Complex enemy AI and special attacks
**Implementation needs:**
- `reflect_damage`: Percentage of damage reflected back to attacker
- `teleport`: Enemy can move instantly within range
- `web_trap`: Immobilizes player for X turns
- `drain_sanity`: Reduces player mental health
- `laser_beam`: Ranged attack with specific damage
- `split_on_death`: Enemy spawns smaller versions when killed
- `fear_aura`: Causes player to flee or suffer penalties
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

### 7. Quest System Expansion
**Referenced in:** NPC actions, dialogue
**Description:** Dynamic quest generation and tracking
**Implementation needs:**
- `starts_quest`: NPCs can give quests dynamically
- Quest objectives beyond "kill X enemies"
- Faction-specific questlines
- Quest rewards that affect reputation and unlock content

### 8. Advanced Dialogue System
**Referenced in:** NPC dialogue conditions
**Description:** Complex dialogue trees with multiple conditions
**Implementation needs:**
- `has_currency`: Check if player has enough money
- `faction_reputation`: Dialogue based on faction standing
- `has_item`: Check for specific items in inventory
- Multiple conditions per dialogue option (AND logic)
- Dialogue that consumes items or currency

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
