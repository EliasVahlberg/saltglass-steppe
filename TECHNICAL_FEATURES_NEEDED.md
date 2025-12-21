# Technical Features Needed for Full Content Implementation

This document lists technical systems that need to be implemented to support the expanded content in the data files. These features are referenced in the JSON but not yet coded.

## Priority: CRITICAL (Required for Core Gameplay)

### 1. Faction Reputation System
**Referenced in:** NPCs, quests, dialogue conditions
**Description:** Track player standing with each faction (-100 to +100)
**Implementation needs:**
- `faction_reputation: HashMap<String, i32>` in GameState
- Functions: `modify_reputation(faction, delta)`, `get_reputation(faction)`
- Dialogue condition: `{"faction_reputation": {"monks": 20}}`
- Shop pricing modifiers based on reputation

### 2. Currency System (Salt Scrip)
**Referenced in:** Items, NPCs, shop actions
**Description:** Universal currency for trading
**Implementation needs:**
- `salt_scrip: u32` field in Player
- Display in HUD: "Scrip: 127"
- Shop buying/selling mechanics
- Quest rewards that give currency

### 3. Equipment System Expansion
**Referenced in:** Items with `equip_slot` field
**Description:** Full equipment slots beyond basic weapon
**Implementation needs:**
- Equipment slots: head, jacket, boots, gloves, accessory, necklace, backpack, ranged_weapon
- Stat bonuses from equipped items
- Visual representation in character screen
- Equipment durability and repair mechanics

### 4. Advanced Item Properties
**Referenced in:** Various items
**Description:** Special item behaviors beyond basic heal/use
**Implementation needs:**
- `suppresses_adaptations`: Veil Tincture hides adaptations temporarily
- `reveals_storm_timing`: Storm Chart shows next storm details
- `grants_invisibility`: Refraction Oil makes player harder to detect
- `breaks_walls`: Glass Pick can destroy wall tiles
- `reveals_locations`: Pilgrim Journal shows hidden areas on map
- `stackable`: Items that can be grouped (Salt Scrip, Glass Dust)
- `grows_over_time`: Storm Seed becomes Storm Glass after time
- `light_source`: Items that provide illumination radius

## Priority: HIGH (Enhances Core Systems)

### 5. Advanced Combat Behaviors
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

### 6. Ritual System
**Referenced in:** NPC actions
**Description:** Special ceremonies that transform the player
**Implementation needs:**
- `triggers_ritual`: "storm_walk", "crucible_transformation"
- Ritual locations and requirements
- Permanent effects on player (new adaptations, stat changes)
- Faction reputation consequences

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

### 10. Light and Vision System
**Referenced in:** Items with light_source, enemy behaviors
**Description:** Dynamic lighting affects gameplay
**Implementation needs:**
- Light radius affects FOV
- Glare tiles that damage or blind
- Light-based puzzles and navigation
- Storm effects on visibility

### 11. Status Effects System
**Referenced in:** Enemy behaviors, item effects
**Description:** Temporary conditions affecting player
**Implementation needs:**
- Poison, blindness, fear, invisibility
- Duration tracking and visual indicators
- Stacking effects and interactions
- Cures and immunities

## Priority: LOW (Polish and Immersion)

### 12. Advanced Map Features
**Referenced in:** Items that reveal locations
**Description:** Dynamic map discovery and annotation
**Implementation needs:**
- Hidden locations revealed by items
- Safe routes marked by NPCs
- Storm damage tracking on world map
- Player annotations and waypoints

### 13. Crafting System
**Referenced in:** Items marked as crafting components
**Description:** Combine items to create new ones
**Implementation needs:**
- Recipe system using JSON data
- Crafting stations or requirements
- Skill-based success rates
- Faction-specific recipes

### 14. Sanity/Mental Health System
**Referenced in:** Enemy drain_sanity behavior
**Description:** Psychological effects of transformation
**Implementation needs:**
- Sanity meter alongside HP
- Effects of low sanity (hallucinations, poor decisions)
- Ways to restore mental health
- Adaptation effects on sanity

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
