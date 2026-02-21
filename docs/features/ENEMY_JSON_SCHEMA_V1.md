# Enemy JSON Schema Reference (v1)

> **JSON Schema**: `schemas/enemies_v1.json`  
> This document provides human-readable documentation for the schema.

## Schema Version

All enemy JSON files MUST include a schema version field:

```json
{
  "schema": "enemies_v1",
  "enemies": [...]
}
```

## File Organization

Enemies can be split across multiple JSON files for organization:

```
data/enemies/
  ├── common_enemies.json      (schema: enemies_v1)
  ├── elite_enemies.json        (schema: enemies_v1)
  ├── boss_enemies.json         (schema: enemies_v1)
  ├── biome_saltflats.json      (schema: enemies_v1)
  └── biome_glassreefs.json     (schema: enemies_v1)
```

All files with `"schema": "enemies_v1"` will be loaded and merged.

## Enemy Definition Schema

### Required Fields

```json
{
  "id": "string",           // Unique identifier (snake_case)
  "name": "string",         // Display name
  "glyph": "string",        // Single character for rendering
  "max_hp": number,         // Maximum hit points (positive integer)
  "damage_min": number,     // Minimum damage (positive integer)
  "damage_max": number      // Maximum damage (>= damage_min)
}
```

### Optional Fields (Core Stats)

```json
{
  "sight_range": number,    // Default: 6, Range: 1-20
  "reflex": number,         // Default: 0, Dodge chance modifier
  "armor": number,          // Default: 0, Damage reduction
  "accuracy": number,       // Default: 0, Hit chance modifier
  "level": number,          // Default: 1, Enemy level for scaling
  "xp_value": number,       // Default: 0, XP granted on death
  "description": "string"   // Default: "", Lore description
}
```

### Optional Fields (AI Behavior)

```json
{
  "demeanor": "string",     // Default: "aggressive"
                            // Options: "aggressive", "defensive", "neutral", "friendly", "pacifist"
  
  "faction": "string",      // Default: null, Faction identifier
                            // Examples: "archive_drones", "storm_cultists", "salt_traders"
  
  "tags": ["string"],       // Default: [], Categorization tags
                            // Examples: ["construct", "mechanical"], ["beast", "predator"]
  
  "hostile_to": ["string"], // Default: ["player"], Faction IDs this enemy attacks
  "allied_with": ["string"] // Default: [], Faction IDs this enemy helps
}
```

### Optional Fields (Combat Abilities)

```json
{
  "ranged_attack": boolean, // Default: false, Can attack at range
  "attack_range": number,   // Default: 1, Max attack distance (if ranged)
  
  "aoe_attack": boolean,    // Default: false, Has area-of-effect attack
  "aoe_radius": number,     // Default: 0, AOE attack radius
  "aoe_warning_turns": number, // Default: 0, Turns before AOE triggers
  
  "resistances": {          // Default: {}, Damage type multipliers
    "fire": number,         // 0.5 = 50% damage, 0.0 = immune
    "cold": number,
    "poison": number,
    "physical": number,
    "lightning": number
  },
  
  "vulnerabilities": {      // Default: {}, Damage type multipliers
    "fire": number,         // 2.0 = 200% damage (double damage)
    "cold": number,
    "poison": number,
    "physical": number,
    "lightning": number
  },
  
  "immunities": ["string"]  // Default: [], Status effect IDs enemy is immune to
                            // Examples: ["poison", "stun", "blinded"]
}
```

### Optional Fields (Spawning)

```json
{
  "is_spawner": boolean,    // Default: false, Can spawn other enemies
  "spawn_rate": number,     // Default: 0, Turns between spawns
  "max_spawns": number,     // Default: 0, Max total spawns
  "spawn_types": ["string"], // Default: [], Enemy IDs to spawn
  
  "swarm": boolean,         // Default: false, Part of swarm mechanics
  "spawns_during_storm": boolean // Default: false, Only spawns during storms
}
```

### Optional Fields (Loot)

```json
{
  "loot_table": [
    {
      "item": "string",     // Required: Item ID to drop
      "weight": number,     // Required: Drop weight (higher = more common)
      "guaranteed": boolean, // Default: false, Always drops
      "min_count": number,  // Default: 1, Minimum drop count
      "max_count": number,  // Default: 1, Maximum drop count
      "quality_tier": "string" // Default: null, Loot tier override
                            // Options: "common", "uncommon", "rare", "epic", "legendary"
    }
  ]
}
```

### Optional Fields (Behaviors)

```json
{
  "behaviors": [
    {
      "type": "string",     // Required: Behavior type
      "condition": "string", // Optional: When behavior triggers
      "percent": number,    // Optional: Percentage value
      "value": number,      // Optional: Generic value parameter
      "range": number,      // Optional: Range parameter
      "damage": number,     // Optional: Damage parameter
      "duration": number,   // Optional: Duration parameter
      "spawns": "string",   // Optional: Enemy ID to spawn
      "count": number,      // Optional: Count parameter
      "cooldown": number    // Optional: Turns between uses
    }
  ]
}
```

**Behavior Types:**
- `reflect_damage` - Reflects damage back to attacker (requires `percent`)
- `poison_sting` - Applies poison status (requires `value` for duration)
- `web_trap` - Immobilizes target (requires `value` for duration)
- `teleport` - Teleports within range (requires `value` for range)
- `split_on_death` - Spawns enemies on death (requires `spawns`, `count`)
- `laser_beam` - Laser attack (requires `value` for damage)
- `summon` - Summons allies (requires `spawns`, `count`, optional `cooldown`)
- `enrage` - Increases damage at low HP (requires `condition`)
- `pack_coordination` - Buffs nearby allies
- `heat_vision` - Enhanced detection (requires `range`)

**Condition Formats:**
- `hp_below_X` - Triggers when HP drops below X%
- `hp_above_X` - Triggers when HP is above X%
- `turn_multiple_X` - Triggers every X turns
- `distance_less_X` - Triggers when player within X tiles
- `distance_more_X` - Triggers when player beyond X tiles
- `ally_count_less_X` - Triggers when fewer than X allies nearby
- `player_adaptations >= X` - Triggers based on player adaptations
- `player_has_item:item_id` - Triggers if player has specific item

### Optional Fields (Visual Effects)

```json
{
  "effects": [
    {
      "condition": "string", // Required: When effect triggers
                            // Options: "on_hit", "on_death", "on_spawn"
      "effect": "string"    // Required: Effect code
                            // Format: "TYPE(@radius &Color1 &Color2)"
                            // Types: W (wave), D (dust), S (sparkle)
    }
  ]
}
```

### Optional Fields (Phases)

```json
{
  "phases": [
    {
      "hp_threshold": number, // Required: HP percentage to trigger phase
      "glyph": "string",      // Optional: New glyph for this phase
      "behaviors": ["string"], // Optional: Behaviors active in this phase
      "armor": number,        // Optional: Armor override for this phase
      "damage_multiplier": number, // Optional: Damage multiplier for this phase
      "resistances": {},      // Optional: Resistance overrides
      "vulnerabilities": {}   // Optional: Vulnerability overrides
    }
  ]
}
```

## Complete Example (Tiered Enemy)

```json
{
  "schema": "enemies_v1",
  "enemies": [
    {
      "id": "glass_wraith_common",
      "name": "Glass Wraith",
      "glyph": "w",
      "max_hp": 15,
      "damage_min": 2,
      "damage_max": 4,
      "sight_range": 8,
      "reflex": 5,
      "armor": 2,
      "accuracy": 10,
      "level": 2,
      "xp_value": 25,
      "description": "A shimmering apparition formed from shattered glass and salt wind. Its touch drains warmth.",
      
      "demeanor": "aggressive",
      "faction": "glass_spirits",
      "tags": ["spirit", "glass", "undead"],
      "hostile_to": ["player", "salt_traders"],
      "allied_with": ["glass_spirits"],
      
      "resistances": {
        "physical": 0.5,
        "cold": 0.3
      },
      "vulnerabilities": {
        "fire": 1.5
      },
      "immunities": ["poison", "bleed"],
      
      "behaviors": [
        {
          "type": "teleport",
          "condition": "hp_below_30",
          "value": 5,
          "cooldown": 3
        },
        {
          "type": "drain_warmth",
          "condition": "distance_less_2",
          "damage": 3,
          "duration": 2
        }
      ],
      
      "loot_table": [
        {
          "item": "glass_shard",
          "weight": 5,
          "guaranteed": true,
          "min_count": 1,
          "max_count": 3
        },
        {
          "item": "wraith_essence",
          "weight": 2,
          "guaranteed": false,
          "quality_tier": "uncommon"
        }
      ],
      
      "effects": [
        {
          "condition": "on_hit",
          "effect": "W(@3 &Cyan &White)"
        },
        {
          "condition": "on_death",
          "effect": "S(@5 &LightCyan &White)"
        }
      ]
    }
  ]
}
```

## Validation Rules

### ID Naming
- Must be unique across all enemy files
- Use snake_case
- Include tier suffix for variants: `_common`, `_uncommon`, `_rare`, `_elite`, `_boss`
- Example: `glass_wraith_common`, `glass_wraith_elite`

### Stat Ranges
- `max_hp`: 1-1000
- `damage_min`: 0-100
- `damage_max`: >= damage_min, 1-100
- `sight_range`: 1-20
- `level`: 1-50
- `xp_value`: 0-10000

### Glyph Guidelines
- Single ASCII character
- Lowercase for common enemies
- Uppercase for elite/boss variants
- Avoid: `@` (player), `#` (wall), `.` (floor), `>` (stairs)

### Faction Naming
- Use snake_case
- Examples: `archive_drones`, `storm_cultists`, `salt_traders`, `glass_spirits`, `void_touched`

### Tag Guidelines
- Use lowercase, singular form
- Common tags: `construct`, `mechanical`, `beast`, `humanoid`, `undead`, `spirit`, `elemental`
- Material tags: `glass`, `salt`, `crystal`, `metal`
- Behavior tags: `predator`, `scavenger`, `guardian`, `swarm`

## Common Mistakes to Avoid

❌ **Don't**: Forget schema version
```json
{
  "enemies": [...]  // Missing "schema": "enemies_v1"
}
```

✅ **Do**: Always include schema
```json
{
  "schema": "enemies_v1",
  "enemies": [...]
}
```

---

❌ **Don't**: Use damage_max < damage_min
```json
{
  "damage_min": 5,
  "damage_max": 3  // Invalid!
}
```

✅ **Do**: Ensure damage_max >= damage_min
```json
{
  "damage_min": 3,
  "damage_max": 5
}
```

---

❌ **Don't**: Use multi-character glyphs
```json
{
  "glyph": "WR"  // Invalid!
}
```

✅ **Do**: Use single character
```json
{
  "glyph": "W"
}
```

---

❌ **Don't**: Forget required loot_table fields
```json
{
  "loot_table": [
    {
      "item": "glass_shard"  // Missing weight!
    }
  ]
}
```

✅ **Do**: Include all required fields
```json
{
  "loot_table": [
    {
      "item": "glass_shard",
      "weight": 5
    }
  ]
}
```

---

❌ **Don't**: Use undefined behavior types
```json
{
  "behaviors": [
    {
      "type": "super_attack"  // Not a valid behavior type!
    }
  ]
}
```

✅ **Do**: Use documented behavior types
```json
{
  "behaviors": [
    {
      "type": "enrage",
      "condition": "hp_below_50"
    }
  ]
}
```

## Tier Guidelines

### Common (Levels 1-5)
- HP: 10-30
- Damage: 1-5
- Simple behaviors (1-2 max)
- Common loot only
- Lowercase glyph

### Uncommon (Levels 3-10)
- HP: 25-60
- Damage: 3-10
- 2-3 behaviors
- Mix of common/uncommon loot
- Lowercase glyph, may use special characters

### Rare (Levels 8-20)
- HP: 50-120
- Damage: 8-20
- 3-4 behaviors, may have phases
- Uncommon/rare loot
- Uppercase glyph or special character

### Elite (Levels 15-35)
- HP: 100-250
- Damage: 15-40
- 4-5 behaviors, phases likely
- Rare/epic loot
- Uppercase glyph, distinctive

### Boss (Levels 25-50)
- HP: 200-1000
- Damage: 30-100
- 5+ behaviors, multiple phases
- Epic/legendary loot guaranteed
- Unique glyph, very distinctive

## Lore Integration Checklist

When creating enemies, ensure:
- [ ] Name fits Saltglass Steppe mythology (glass, salt, light, void themes)
- [ ] Description references world lore
- [ ] Faction aligns with established factions
- [ ] Tags reflect material/nature (glass, salt, crystal, etc.)
- [ ] Behaviors match thematic identity (glass enemies shatter, salt enemies corrode)
- [ ] Loot drops make sense (glass enemies drop glass shards)
- [ ] Visual effects match theme (glass = cyan/white, salt = yellow/white)
