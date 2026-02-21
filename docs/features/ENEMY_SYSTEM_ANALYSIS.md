# Enemy System Feature Gap Analysis

## Current Enemy Capabilities

### ✅ Implemented Features

**Basic Stats:**
- HP, damage range (min/max), armor, accuracy, reflex
- Sight range, level, XP value
- Glyph and name

**AI Demeanor System:**
- Aggressive (attacks on sight)
- Defensive (takes cover, flees at low HP <30%)
- Neutral (ignores unless provoked)
- Friendly (helps player, becomes defensive if attacked)
- Pacifist (flees when threatened)

**Combat Capabilities:**
- Melee attacks
- Ranged attacks (with attack_range)
- AOE attacks (with warning turns and radius)
- Status effect application (poison, web trap, etc.)

**Special Behaviors:**
- Reflect damage (percentage-based)
- Poison sting (applies poison status)
- Web trap (immobilizes player)
- Teleportation (within range)
- Spawning (is_spawner, spawn_rate, max_spawns, spawn_types)
- Split on death (spawns other enemies)
- Laser beam attacks
- Swarm mechanics (swarm_leader, swarm_id)

**Loot System:**
- Loot tables with weighted drops
- Inventory system for carried items

**Visual Effects:**
- On-hit effects (particle effects)
- On-death effects (particle effects)

**Status Effects:**
- Can have status effects applied to them
- Status effect tracking (id, duration, stacks)

### ❌ Missing Features (High Impact, Low Effort)

#### 1. **Movement Patterns** (Effort: 2-3 hours)
**Current**: Enemies just pathfind directly to player
**Missing**:
- Patrol routes (circular, back-and-forth, random wander)
- Ambush behavior (wait in hiding, strike when player is close)
- Kiting behavior (maintain distance, retreat when player approaches)
- Flanking behavior (try to get behind player)
- Guard behavior (stay near a location/item)

**Implementation**: Add `movement_pattern` field with types:
```json
"movement_pattern": {
  "type": "patrol",
  "waypoints": [[10, 10], [20, 10], [20, 20], [10, 20]],
  "loop": true
}
```

#### 2. **Conditional Behaviors** (Effort: 1-2 hours)
**Current**: Behaviors trigger randomly or on specific events
**Missing**:
- HP threshold triggers ("enrage at <50% HP")
- Turn-based triggers ("every 3 turns, do X")
- Distance-based triggers ("if player within 3 tiles, do Y")
- Ally count triggers ("if alone, flee")

**Implementation**: Enhance existing `condition` field:
```json
"behaviors": [
  {
    "type": "enrage",
    "condition": "hp_below_50",
    "effect": "damage_multiplier_2x"
  }
]
```

#### 3. **Resistances and Vulnerabilities** (Effort: 1-2 hours)
**Current**: Only armor for general damage reduction
**Missing**:
- Damage type resistances (fire, cold, poison, physical)
- Damage type vulnerabilities (2x damage from specific types)
- Status effect immunities (immune to poison, stun, etc.)

**Implementation**: Add fields:
```json
"resistances": {
  "fire": 0.5,
  "cold": 0.5,
  "physical": 0.8
},
"vulnerabilities": {
  "lightning": 2.0
},
"immunities": ["poison", "stun"]
```

#### 4. **Phases/Forms** (Effort: 3-4 hours)
**Current**: Enemies are static throughout combat
**Missing**:
- Phase transitions at HP thresholds
- Form changes (visual and mechanical)
- Different abilities per phase

**Implementation**: Add `phases` array:
```json
"phases": [
  {
    "hp_threshold": 100,
    "glyph": "D",
    "behaviors": ["melee_attack"],
    "armor": 5
  },
  {
    "hp_threshold": 50,
    "glyph": "D",
    "behaviors": ["enrage", "aoe_attack"],
    "armor": 3,
    "damage_multiplier": 1.5
  }
]
```

#### 5. **Summon/Reinforcement** (Effort: 1-2 hours)
**Current**: Spawners exist but are passive
**Missing**:
- Active summoning as a combat action
- Reinforcement calls (summon allies when in danger)
- Summon limits per combat

**Implementation**: Already partially exists, just needs:
```json
"behaviors": [
  {
    "type": "summon",
    "condition": "hp_below_30",
    "spawns": "glass_beetle",
    "count": 2,
    "cooldown": 5
  }
]
```

#### 6. **Loot Tier System** (Effort: 1 hour)
**Current**: Flat loot tables
**Missing**:
- Guaranteed drops vs. chance drops
- Loot quality scaling with enemy level/tier
- Rare drop indicators

**Implementation**: Enhance loot_table:
```json
"loot_table": [
  {
    "item": "glass_shard",
    "weight": 5,
    "guaranteed": false,
    "min_count": 1,
    "max_count": 3
  },
  {
    "item": "rare_crystal",
    "weight": 1,
    "guaranteed": false,
    "quality_tier": "rare"
  }
]
```

#### 7. **Faction/Tag System** (Effort: 1 hour)
**Current**: No grouping or relationships
**Missing**:
- Enemy factions (Archive Drones, Storm Cultists, etc.)
- Tags for categorization (undead, construct, beast, humanoid)
- Faction-based AI (allies help each other, enemies fight each other)

**Implementation**: Add fields:
```json
"faction": "archive_drones",
"tags": ["construct", "mechanical", "ancient"],
"hostile_to": ["storm_cultists", "player"],
"allied_with": ["archive_drones"]
```

#### 8. **Sound/Alert System** (Effort: 2 hours)
**Current**: Enemies only react to sight
**Missing**:
- Sound-based detection (combat alerts nearby enemies)
- Alert states (unaware → alerted → combat)
- Investigation behavior (move to last known player position)

**Implementation**: Add fields:
```json
"hearing_range": 10,
"alert_radius": 8,
"investigation_turns": 5
```

### ⚠️ Missing Features (High Impact, Medium Effort)

#### 9. **Ability Cooldowns** (Effort: 3-4 hours)
**Current**: Behaviors can trigger every turn
**Missing**: Per-ability cooldown tracking

#### 10. **Multi-Target Attacks** (Effort: 2-3 hours)
**Current**: Single target or AOE
**Missing**: Cleave (hit adjacent enemies), chain attacks, etc.

#### 11. **Buff/Debuff Auras** (Effort: 3-4 hours)
**Current**: No passive area effects
**Missing**: Auras that affect nearby allies/enemies

### 🔮 Missing Features (Nice to Have, High Effort)

#### 12. **Advanced AI States** (Effort: 6-8 hours)
- State machines (patrol → investigate → combat → flee)
- Memory of player actions
- Learning behaviors

#### 13. **Environmental Interactions** (Effort: 4-6 hours)
- Knock player into hazards
- Destroy terrain
- Use cover

## Recommendations for Tiered Mob System

### Priority 1 (Implement Now)
1. **Resistances/Vulnerabilities** - Essential for build diversity
2. **Faction/Tag System** - Essential for lore integration
3. **Loot Tier System** - Essential for tiered loot
4. **Conditional Behaviors** - Makes enemies more interesting

### Priority 2 (Implement Soon)
5. **Movement Patterns** - Adds tactical variety
6. **Summon/Reinforcement** - Creates dynamic encounters
7. **Sound/Alert System** - More realistic AI

### Priority 3 (Future)
8. **Phases/Forms** - For boss encounters
9. **Ability Cooldowns** - For balance
10. **Multi-Target Attacks** - For advanced enemies

## Minimal Additions for Maximum Impact

If we want to make enemies interesting with **minimal effort** (4-6 hours total):

1. **Add resistances/vulnerabilities** (1-2 hours)
   - Enables build diversity (fire builds vs. cold builds)
   - Makes enemy choice matter

2. **Add faction/tag system** (1 hour)
   - Enables lore-aligned enemy design
   - Enables faction-based spawning

3. **Add conditional behaviors** (1-2 hours)
   - "Enrage at 50% HP" 
   - "Summon allies when alone"
   - Makes combat more dynamic

4. **Enhance loot tables** (1 hour)
   - Guaranteed drops
   - Count ranges
   - Quality tiers

**Total**: 4-6 hours for significant enemy depth improvement
