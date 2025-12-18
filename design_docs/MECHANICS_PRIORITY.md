# Core Mechanics Priority Analysis

Ranked by gameplay importance and implementation difficulty.
Scale: 1-5 (1=low, 5=high)

## Priority Matrix

| Rank | Mechanic | Importance | Difficulty | Current State | Dependencies |
|------|----------|------------|------------|---------------|--------------|
| 1 | Turn-Based System | 5 | 2 | 80% | None |
| 2 | Combat System | 5 | 4 | 60% | Turn system |
| 3 | Player Movement & Interaction | 5 | 2 | 50% | Map system |
| 4 | Map and Environment | 5 | 4 | 40% | None |
| 5 | Enemy AI and NPCs | 4 | 4 | 40% | Combat, Map |
| 6 | UI and HUD | 4 | 3 | 30% | All systems |
| 7 | Inventory and Equipment | 4 | 2 | 60% | Items data |
| 8 | Character Progression | 3 | 3 | 0% | Combat |
| 9 | Exploration | 3 | 2 | 50% | Map, FOV |
| 10 | Quests and Storytelling | 3 | 4 | 0% | NPCs, Map |
| 11 | Crafting System | 2 | 3 | 0% | Inventory |
| 12 | Modularity/Modding | 2 | 3 | 30% | All systems |
| 13 | Audio and Sound | 1 | 2 | 0% | None |

---

## Detailed Analysis

### 1. Turn-Based System (Importance: 5, Difficulty: 2)

**Current State:** AP system implemented. Actions consume AP, auto-end turn when depleted.

**Completed:**
- ✅ Action Points (AP) system - actions cost variable AP
- ✅ AP costs defined in data/actions.json
- ✅ End turn early option (resets AP)
- ✅ Auto-end turn when AP depleted

**Missing:**
- Initiative rolls for encounter start
- Interruptions/reactions

**Implementation Approach:**
```
1. Add AP field to Player and Enemy structs
2. Define AP costs in data/actions.json
3. Modify action handlers to check/deduct AP
4. Add turn_end() that resets AP and triggers enemy turns
5. Initiative: sort actors by RE stat + roll at encounter start
```

**DES Testing:** Already have turn tracking. Add AP assertions.

---

### 2. Combat System (Importance: 5, Difficulty: 4)

**Current State:** Combat math, ranged attacks, and status effects implemented.

**Completed:**
- ✅ Hit chance: accuracy - target_reflex - cover_bonus (clamped 5-95%)
- ✅ Damage: base_damage - armor (minimum 1)
- ✅ Critical hits: 5% chance for 2x damage
- ✅ Ranged attacks with range check, LOS, ammo consumption
- ✅ Status effects: Poison, Burn, Stun, Bleed, Slow with duration/potency
- ✅ Weapons defined in data/weapons.json (6 weapons)
- ✅ Player stats: reflex, armor, equipped_weapon
- ✅ Enemy stats: reflex, armor, accuracy

**Missing:**
- Body part targeting
- Cover mechanics (hit chance reduction from cover)
- Psychic abilities (PSY resource)
- Dodge/Reflex evasion rolls

**Implementation Approach:**
```
Phase 1 - Core combat math:
  - Implement hit chance: base + weapon_acc - target_RE - cover_bonus
  - Implement damage: weapon_dmg + stat_bonus - target_AR
  - Add miss/hit/crit outcomes

Phase 2 - Ranged:
  - Add weapon range field
  - Line-of-sight check for ranged attacks
  - Ammo tracking

Phase 3 - Status effects:
  - StatusEffect enum with duration
  - Apply/tick/remove in turn system
  - Effects modify stats or deal DoT

Phase 4 - Body targeting:
  - BodyPart enum (head, torso, arms, legs)
  - Hit chance modifiers per part
  - Damage multipliers and status triggers
```

**DES Testing:** Use ai_disabled enemies for deterministic combat tests.

---

### 3. Player Movement & Interaction (Importance: 5, Difficulty: 2)

**Current State:** 8-directional movement, item pickup, basic FOV.

**Missing:**
- NPC interaction/dialogue
- Object interaction (doors, levers, containers)
- Inspection mode (ASCII art views)
- Cover system (crouch behind objects)
- Stealth (noise, lighting detection)

**Implementation Approach:**
```
Phase 1 - Interactions:
  - Add Interactable trait/component
  - Context menu on 'e' key near interactables
  - Door: toggle walkable, update FOV blockers
  - Container: open inventory transfer UI

Phase 2 - NPC dialogue:
  - Dialogue tree in data/dialogues/*.json
  - DialogueState tracks conversation progress
  - Condition checks (items, adaptations, flags)

Phase 3 - Stealth:
  - Noise value per action
  - Enemy hearing range
  - Light level per tile
  - Detection threshold calculation
```

**DES Testing:** Add Interact action type, dialogue assertions.

---

### 4. Map and Environment (Importance: 5, Difficulty: 4)

**Current State:** Single-tile procgen with rooms, corridors, FOV, storm glass conversion. World map generation implemented.

**Completed:**
- ✅ World map generation (`WorldMap` struct with 64x64 grid)
- ✅ Biome layer (Desert, Saltflat, Scrubland, Oasis, Ruins) via Perlin noise
- ✅ Terrain layer (Flat, Hills, Dunes, Canyon, Mesa) via Perlin noise
- ✅ Elevation layer (0-255) via Perlin noise
- ✅ POI placement (Town, Dungeon, Landmark, Shrine) with distance penalty
- ✅ Deterministic generation from seed
- ✅ `tile_seed()` for deriving per-tile seeds
- ✅ `Map::generate_from_world()` - tile generation from biome/terrain/elevation
  - Wall type varies by biome (Saltflat→salt_crystal, Ruins→shale, else→sandstone)
  - Room count varies by terrain (Canyon: 3-5, Mesa: 4-6, Hills: 5-8, Dunes: 4-7, Flat: 6-10)
  - Glass density varies by biome (Saltflat: 20-35, Oasis: 5-10, else: 10-20)

**Missing:**
- Lazy tile generation when player enters world tile
- Subterranean layers
- Time of day affecting ambient light
- Environmental hazards (fire spread, acid pools)
- Liquid system

**Lighting System (Implemented):**
- `LightSource` struct with position, radius, intensity
- `compute_lighting()` calculates light levels for all tiles
- Player has torch (radius 8, intensity 150)
- Ambient light baseline (default 100)
- Light map updated on player movement

**Implementation Approach:**
```
Phase 1 - World map:
  - WorldMap struct with biome/terrain/elevation grids
  - Noise-based generation for layers 1-3
  - POI placement with distance penalties

Phase 2 - Tile generation:
  - Derive tile seed from world_seed + coords
  - Biome-specific room/corridor templates
  - Resource spawning based on world layer 4

Phase 3 - Lighting:
  - Light sources emit radius
  - Combine with FOV for visibility
  - Time system affects ambient light

Phase 4 - Liquids/hazards (complex):
  - Tile liquid layer (type, depth)
  - Flow simulation on turn tick
  - Interaction effects (fire+water, acid+metal)
```

**DES Testing:** MapTileAt assertion exists. Add lighting/liquid assertions.

---

### 5. Enemy AI and NPCs (Importance: 4, Difficulty: 4)

**Current State:** Demeanor system implemented with multiple behavior types.

**Completed:**
- ✅ AI demeanor types (aggressive, defensive, neutral, pacifist)
- ✅ Neutral: ignore unless attacked (provoked flag)
- ✅ Pacifist: flee when threatened
- ✅ Defensive: flee when HP drops below 30%
- ✅ Flee behavior (move away from player)

**Missing:**
- Patrol routes
- Aggro system with target switching
- Item usage by enemies
- Cover-seeking behavior

**Implementation Approach:**
```
Phase 1 - Demeanor:
  - AIDemeanor enum in enemy data
  - Behavior tree per demeanor type
  - Neutral: ignore unless attacked
  - Pacifist: flee when threatened

Phase 2 - Aggro:
  - aggro_target and aggro_values HashMap
  - Update aggro on damage/heal/ability
  - Switch target when another exceeds threshold

Phase 3 - Tactical AI:
  - Evaluate cover positions
  - Ranged enemies maintain distance
  - Use items when HP low
```

**DES Testing:** ai_disabled flag exists. Add demeanor-specific scenarios.

---

### 6. UI and HUD (Importance: 4, Difficulty: 3)

**Current State:** Basic TUI rendering exists.

**Missing:**
- Full HUD layout per mockup
- Side panel with player ASCII art
- Bottom panel with hotkeys/event log
- Inventory screen
- Inspection mode views
- World map view

**Implementation Approach:**
```
1. Use ratatui layout system for panels
2. Create HUD widget with stat display
3. Event log as scrollable list
4. ASCII art renderer for inspection mode
5. Modal system for inventory/menus
```

**Note:** UI work is mostly presentation. Core game logic should be complete first.

---

### 7. Inventory and Equipment (Importance: 4, Difficulty: 2)

**Current State:** Full equipment system implemented.

**Completed:**
- ✅ Equipment slots (weapon, armor, accessory)
- ✅ Equip/unequip actions
- ✅ Equipment stat bonuses (armor value)
- ✅ Item inspection with hidden properties
- ✅ Equippable light sources (hand torch in accessory slot)

**Missing:**
- Weight system
- Item condition/durability

**Implementation Approach:**
```
1. Add Equipment struct with slots
2. Equip/unequip actions
3. Apply equipment stats to player
4. Weight calculation affects MS
5. Durability decrements on use
```

**DES Testing:** Add equipment assertions, weight checks.

---

### 8. Character Progression (Importance: 3, Difficulty: 3)

**Current State:** XP and level-up system implemented with stat allocation.

**Completed:**
- ✅ XP gain from combat (enemy xp_value)
- ✅ Level thresholds (data-driven in progression.json)
- ✅ Stat points on level up (3 per level)
- ✅ Stat allocation (max_hp, max_ap, reflex)

**Missing:**
- Skill trees (if keeping)
- Backgrounds with starting bonuses

**Implementation Approach:**
```
1. Add xp, level fields to Player
2. XP rewards in combat/quest completion
3. Level-up triggers stat point allocation
4. Skill unlocks at level thresholds
5. Background selection at game start
```

**DES Testing:** Add XP/level assertions.

---

### 9. Exploration (Importance: 3, Difficulty: 2)

**Current State:** FOV reveals tiles, explored tiles persist in memory.

**Completed:**
- ✅ Explored tile memory (`revealed` HashSet)
- ✅ FOV union into explored each turn
- ✅ Explored-but-not-visible tiles render in gray (actual glyph, not placeholder)
- ✅ Dynamic objects (enemies, NPCs, items) hidden when out of FOV

**Missing:**
- Auto-explore pathfinding

**Implementation Approach:**
```
Auto-explore: BFS to nearest unexplored walkable tile
```

---

### 10. Quests and Storytelling (Importance: 3, Difficulty: 4)

**Current State:** Quest system implemented with data-driven objectives and rewards.

**Completed:**
- ✅ Quest data structure (QuestDef, Objective, QuestReward)
- ✅ QuestLog tracks active/completed quests
- ✅ Objective types: kill, collect, reach, talk_to
- ✅ Quest hooks in combat, item pickup, movement, NPC talk
- ✅ DES support (accept_quest, complete_quest actions + assertions)

**Missing:**
- Quest log UI
- NPC quest givers (NPCs that offer quests)
- Story flags/triggers

**Depends on:** NPC dialogue system (completed).

---

### 11. Crafting System (Importance: 2, Difficulty: 3)

**Current State:** Data-driven crafting system implemented.

**Completed:**
- ✅ Recipe definitions in data/recipes.json
- ✅ Material requirements checking
- ✅ GameState::craft() method
- ✅ DES support (craft action)

**Missing:**
- Crafting UI
- Crafting stations (location-based crafting)

---

### 12. Modularity/Modding (Importance: 2, Difficulty: 3)

**Current State:** Items and enemies are data-driven (JSON).

**Missing:**
- Mod loading system
- Override/extend base data
- Custom scripts (lua?)
- Documentation for modders

**Implementation Approach:**
```
1. Mod folder structure
2. Load order and merge logic
3. Validation for mod data
```

**Lower priority** - focus on base game first.

---

### 13. Audio and Sound (Importance: 1, Difficulty: 2)

**Current State:** Not implemented (TUI game).

**Notes:** Could add terminal bell or external audio library later. Not critical for TUI roguelike.

---

## Recommended Implementation Order

### Milestone 1: Core Loop (Vertical Slice)
1. ✅ Turn system basics
2. ✅ AP system
3. ✅ Combat math (hit/damage/armor)
4. ✅ Ranged attacks
5. ✅ Status effects (basic set)

### Milestone 2: World & Exploration
1. ✅ Explored tile memory
2. ✅ World map generation
3. ✅ Tile-from-world-seed generation
4. ✅ Dynamic lighting

### Milestone 3: Depth
1. ✅ Proper inventory system with inspect stats/properties
2. ✅ Equipment system (weapon/armor/accessory slots)
3. ✅ AI demeanor types (aggressive, defensive, neutral, pacifist)
4. ✅ NPC dialogue (adaptation + item conditions)
5. ✅ Character progression (XP, levels, stat point allocation)
6. ✅ Equippable light sources (hand torch)

### Milestone 4: Content
1. ✅ Quests (data-driven with objectives and rewards)
2. ✅ Crafting (data-driven recipes)
3. ✅ More enemy types (16 total)
4. ✅ More items/equipment (25 total)

### Milestone 5: Polish
1. ⬜ Full UI/HUD
2. ⬜ Inspection mode ASCII art
3. ⬜ Modding support
4. ⬜ Audio (optional)
