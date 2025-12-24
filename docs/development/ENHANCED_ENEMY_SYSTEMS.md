# Enhanced Enemy Systems Implementation

## Overview

This document describes the implementation of enhanced enemy systems including enemy variation, ranged attacks, AOE attacks, swarm mechanics, enemy spawners, friendly NPCs, and enhanced debug capabilities.

## Features Implemented

### 1. Enemy Variation and New Types

**New Enemy Types Added:**
- **Shard Nest** - Spawner enemy that creates shard spiders
- **Storm Archer** - Ranged attacker with bow attacks
- **Glass Bomber** - AOE suicide bomber with warning system
- **Psychic Wraith** - Telepathic enemy with mind attacks
- **Adaptation Horror** - Boss-type enemy with AOE mutation attacks
- **Friendly NPCs** - Mirror Monk, Sand Engineer, and Glassborn variants

**Enhanced Enemy Capabilities:**
- Ranged attack system with configurable range
- AOE attack system with visual warnings
- Spawner mechanics with spawn limits and cooldowns
- Swarm coordination and group behavior
- Advanced AI behaviors (kiting, charging, fleeing)

### 2. Ranged Attack System

**Implementation:**
- `ranged_attack` and `attack_range` fields in enemy definitions
- Visual beam effects for ranged attacks (Arrow beam type)
- Smart AI that maintains distance for ranged enemies
- Line-of-sight requirements for ranged attacks

**Usage Example:**
```json
{
  "id": "storm_archer",
  "ranged_attack": true,
  "attack_range": 6,
  "behaviors": [
    { "type": "kite_enemy", "range": 6 },
    { "type": "precise_shot", "accuracy_bonus": 20 }
  ]
}
```

### 3. AOE Attack System

**Implementation:**
- `aoe_attack`, `aoe_radius`, and `aoe_warning_turns` fields
- Visual warning system with countdown
- Area damage calculation with player position checking
- Warning message system (MsgType::Warning)

**Usage Example:**
```json
{
  "id": "glass_bomber",
  "aoe_attack": true,
  "aoe_radius": 2,
  "aoe_warning_turns": 2,
  "behaviors": [
    { "type": "suicide_bomber", "damage": 8, "radius": 2 }
  ]
}
```

### 4. Swarm Enemy System

**Implementation:**
- `swarm_id` and `swarm_leader` fields for coordination
- Group spawning with `spawn_swarm` debug command
- Swarm behavior coordination (future enhancement)

**Usage Example:**
```json
{
  "id": "shard_spider",
  "swarm": true,
  "behaviors": [
    { "type": "on_hit_refraction", "value": 2 }
  ]
}
```

### 5. Enemy Spawner System

**Implementation:**
- `is_spawner`, `spawn_rate`, `max_spawns`, `spawn_types` fields
- Cooldown-based spawning system
- Spawn location finding algorithm
- Visual effects for spawning events

**Usage Example:**
```json
{
  "id": "shard_nest",
  "is_spawner": true,
  "spawn_rate": 3,
  "max_spawns": 8,
  "spawn_types": ["shard_spider"]
}
```

### 6. Friendly NPC System

**Implementation:**
- `demeanor: "friendly"` for non-hostile NPCs
- Faction-specific friendly variants
- Special interaction behaviors
- Spawn table integration

**Available Friendly NPCs:**
- **Mirror Monk Friendly** - Offers blessings and wisdom
- **Sand Engineer Friendly** - Provides repairs and maps
- **Glassborn Friendly** - Offers transformation guidance

### 7. Enhanced Debug System

**New Debug Commands:**
- `add_adaptation <id>` - Add adaptation to player
- `list_adaptations` - List all available adaptations
- `add_psychic <id>` - Add psychic ability to player
- `list_psychic` - List all available psychic abilities
- `set_coherence <amount>` - Set psychic coherence level
- `spawn_enemy <id> [x] [y]` - Spawn enemy at position
- `spawn_swarm <id> <count>` - Spawn enemy swarm

### 8. Psychic Abilities System

**Implementation:**
- 15 psychic abilities across 5 categories
- Coherence-based resource system
- Cooldown mechanics
- Category-based organization (Telepathy, Probability, Energy, Phasing, Temporal)

**Categories:**
- **Telepathy** - Mind probe, telepathic strike, collective link
- **Probability** - Probability shift, quantum dodge, reality anchor
- **Energy** - Energy drain, storm channel, psychic shield
- **Phasing** - Phase step, ghost walk, dimensional rift
- **Temporal** - Temporal glimpse, time dilation

## Technical Architecture

### Enhanced Enemy Struct

```rust
pub struct Enemy {
    // Existing fields...
    pub spawned_count: u32,
    pub last_spawn_turn: u32,
    pub aoe_target: Option<(i32, i32)>,
    pub aoe_warning_turns: u32,
    pub swarm_leader: bool,
    pub swarm_id: Option<String>,
}
```

### Enhanced EnemyDef Struct

```rust
pub struct EnemyDef {
    // Existing fields...
    pub is_spawner: bool,
    pub spawn_rate: u32,
    pub max_spawns: u32,
    pub spawn_types: Vec<String>,
    pub ranged_attack: bool,
    pub attack_range: u32,
    pub aoe_attack: bool,
    pub aoe_radius: u32,
    pub aoe_warning_turns: u32,
}
```

### Enhanced AI System

The AI system now handles:
- Two-pass enemy updates (spawners/AOE first, then movement)
- Ranged attack logic with distance management
- AOE warning and execution system
- Spawner cooldown and location finding
- Advanced behavior patterns

## Data Files

### Updated Files:
- `data/enemies.json` - Enhanced with new enemy types and capabilities
- `data/psychic_abilities.json` - New file with 15 psychic abilities
- `data/biome_spawn_tables.json` - Updated with new enemies and friendly NPCs

### New Test Files:
- `tests/scenarios/enhanced_enemy_systems_test.json` - Comprehensive test scenario

## Visual Enhancements

### New Beam Types:
- **Arrow** (Green) - For ranged attacks
- Enhanced beam visualization system

### New Message Types:
- **Warning** (Light Red) - For AOE attack warnings and critical alerts

### Visual Effects:
- AOE warning indicators
- Spawn effect animations
- Enhanced beam effects for different attack types

## Performance Considerations

- Efficient spawner cooldown tracking
- Optimized AOE distance calculations
- Smart AI behavior selection to prevent excessive computation
- Lazy loading of enemy definitions

## Future Enhancements

1. **Advanced Swarm AI** - Coordinated group tactics
2. **Dynamic Spawner Behavior** - Adaptive spawn rates based on player actions
3. **Faction-Specific Abilities** - Unique abilities for different enemy factions
4. **Environmental Interactions** - Enemies that interact with map features
5. **Player Psychic Ability Usage** - Integration with combat system

## Testing

The implementation includes comprehensive testing through:
- Unit tests for core functionality
- DES (Debug Execution System) integration tests
- Manual testing scenarios
- Debug command validation

All existing tests pass (56/56) with the new implementation, ensuring backward compatibility while adding significant new functionality.

## Usage Examples

### Spawning Enemies via Debug Console:
```
/spawn_enemy storm_archer 15 10
/spawn_swarm shard_spider 5
```

### Adding Player Abilities:
```
/add_adaptation quantum_entanglement
/add_psychic mind_probe
/set_coherence 150
```

### Testing AOE Mechanics:
```
/spawn_enemy glass_bomber 12 12
# Move close and observe warning system
```

This implementation provides a solid foundation for complex enemy encounters while maintaining the game's performance and architectural principles.
