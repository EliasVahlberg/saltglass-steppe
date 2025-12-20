# Core Mechanics Priority Analysis

Ranked by gameplay importance and implementation difficulty.
Scale: 1-5 (1=low, 5=high)

## Executive Summary: Path to Minimal Working State

**Current State:** The game has a solid foundation with ~60% of core systems implemented. The turn-based combat loop, world generation, equipment, questing, and crafting are functional.

**Minimal Working State Definition:** A playable vertical slice that demonstrates the core creative vision:

- Storm-driven map editing that creates tactical challenge
- Adaptation system with social/mechanical consequences
- Light-based tactical combat
- Faction-differentiated playstyles
- 30-60 minute gameplay loop with procedural replayability

**Critical Path (4-6 weeks estimated):**

1. **Glass Storm System** (2 weeks) - THE defining mechanic

   - Storm generation with tile rewriting
   - Storm Glass currency drops
   - Safe/danger zone indicators
   - Storm forecast system

2. **Adaptation System** (1 week) - Core identity mechanic

   - Refraction stat tracking
   - 5-8 basic adaptations (crystalline vision, glass skin, light sensitivity)
   - NPC reaction system based on refraction level
   - Visual indicators in UI

3. **NPC Faction System** (1 week) - Social consequence layer

   - 3 core factions (Monks, Engineers, Glassborn)
   - Reputation tracking
   - Faction-specific dialogues with adaptation checks
   - Trade/quest access based on faction standing

4. **World Tile Transition** (1 week) - Exploration loop

   - World map UI with biome/POI display
   - Lazy tile generation on entry
   - Save/load tile state
   - Storm effects on world map

5. **Content Polish** (1 week) - Make it feel complete
   - 3-5 starter quests with faction branching
   - 10-15 enemy types distributed by biome
   - Inspection mode ASCII art for 3-5 signature items/NPCs
   - Basic balance pass

**Post-Minimal (Future):**

- Psychic abilities system
- Subterranean layers
- Body part targeting
- Liquid/hazard simulation
- Advanced AI (patrol routes, cover seeking)
- Background/specialization system
- Modding support

---

## Priority Matrix

| Rank | Mechanic                      | Importance | Difficulty | Current State | Dependencies             |
| ---- | ----------------------------- | ---------- | ---------- | ------------- | ------------------------ |
| 1    | **Glass Storm System**        | 5          | 4          | 0%            | Map, Turn system         |
| 2    | **Adaptation System**         | 5          | 3          | 0%            | Combat, NPC              |
| 3    | Turn-Based System             | 5          | 2          | 80%           | None                     |
| 4    | Combat System                 | 5          | 4          | 70%           | Turn system              |
| 5    | **NPC Faction System**        | 5          | 3          | 20%           | NPC dialogue, Adaptation |
| 6    | Player Movement & Interaction | 5          | 2          | 60%           | Map system               |
| 7    | Map and Environment           | 5          | 4          | 65%           | None                     |
| 8    | Enemy AI and NPCs             | 4          | 4          | 50%           | Combat, Map              |
| 9    | UI and HUD                    | 4          | 3          | 70%           | All systems              |
| 10   | Inventory and Equipment       | 4          | 2          | 90%           | Items data               |
| 11   | Character Progression         | 3          | 3          | 60%           | Combat                   |
| 12   | Exploration                   | 3          | 2          | 70%           | Map, FOV                 |
| 13   | Quests and Storytelling       | 3          | 4          | 60%           | NPCs, Map                |
| 14   | Crafting System               | 2          | 3          | 60%           | Inventory                |
| 15   | Psychic Abilities             | 3          | 4          | 0%            | Combat, PSY stat         |
| 16   | Modularity/Modding            | 2          | 3          | 40%           | All systems              |
| 17   | Audio and Sound               | 1          | 2          | 0%            | None                     |

**Bold = Critical for minimal working state**

---

---

## Detailed Analysis

### **NEW 1. Glass Storm System (Importance: 5, Difficulty: 4) [CRITICAL FOR MINIMAL STATE]**

**Current State:** Not implemented. This is THE defining mechanic of Saltglass Steppe.

**Required for Minimal State:**

- ✅ World map exists (can track storm locations)
- ⬜ Storm generation algorithm (procedural storm paths)
- ⬜ Tile editing during storms (walls convert to glass, corridors seal/open)
- ⬜ Storm Glass `◆` drops from storm-affected tiles
- ⬜ Storm forecast UI showing next storm timing/location
- ⬜ Safe zone indicators (mirrors, shelters)
- ⬜ Storm damage system (beam hazards, glass dust)

**Design Notes:**

- Storms are not random weather—they are targeted map edits that:
  - Convert walls to glass (changes tactical cover)
  - Seal/open corridors (forces route changes)
  - Drop Storm Glass currency
  - Trigger adaptation exposure rolls
- Storms follow semi-predictable patterns (via `storm_forecast` tool)
- Creates the core tension: rush before storm or prepare to survive it

**Implementation Approach:**

```
Phase 1 - Basic Storm:
  - Storm timer (turns until next storm)
  - Storm "hits" current tile
  - Random wall tiles → glass conversion
  - Drop Storm Glass items

Phase 2 - Tile Editing:
  - Storm affects specific tile regions (beams from above)
  - Doors seal/unseal
  - New corridors open in glass walls
  - Player can take cover to reduce exposure

Phase 3 - Forecast System:
  - Storm Compass `⊕` item predicts timing
  - Archive data shows historical storm patterns
  - Engineers can build storm shelters
```

**DES Testing:**

```json
{
  "actions": [{ "turn": 5, "action": { "type": "trigger_storm" } }],
  "assertions": [
    {
      "at_turn": 6,
      "check": {
        "type": "map_tile_at",
        "x": 10,
        "y": 5,
        "expected": "glass_wall"
      }
    },
    {
      "at_turn": 6,
      "check": {
        "type": "player_has_item",
        "item_id": "storm_glass",
        "min_count": 1
      }
    }
  ]
}
```

**Why Critical:** Without storms, the game is just another roguelike. Storms create:

- Unique tactical challenge (map changes mid-run)
- Core currency loop (Storm Glass economy)
- Adaptation pressure (storm exposure triggers mutations)
- Faction interaction points (who helps you survive)

---

### **NEW 2. Adaptation System (Importance: 5, Difficulty: 3) [CRITICAL FOR MINIMAL STATE]**

**Current State:** Not implemented. Core identity/consequence system.

**Required for Minimal State:**

- ⬜ Refraction stat (0-100 scale, visible in HUD)
- ⬜ 5-8 basic adaptations with thresholds
  - Crystalline Vision (20+ refraction): see through glass walls
  - Glass Skin (40+ refraction): +armor, -speed
  - Light Sensitivity (60+ refraction): light sources hurt
  - Prism Reflex (80+ refraction): beam attacks have splash
- ⬜ Adaptation triggers (storm exposure, consuming Storm Glass, psychic use)
- ⬜ NPC reaction modifiers based on refraction
- ⬜ Visual indicators (character glyph changes, side panel ASCII art updates)

**Design Notes:**

- Adaptations are permanent unless using rare Saint's Tear `○` items
- Low refraction = trusted by Engineers, distrusted by Glassborn
- High refraction = trusted by Monks/Glassborn, flagged by Archive drones
- Each adaptation has pros AND cons (never pure upgrade)

**Implementation Approach:**

```
Phase 1 - Stat Tracking:
  - Add refraction field to Player (0-100)
  - Increment on storm exposure, Storm Glass consumption
  - Display in top HUD

Phase 2 - Adaptation Unlocks:
  - Define adaptations in data/adaptations.json
  - Check thresholds each turn
  - Apply stat modifiers and special abilities
  - Log "You feel your skin hardening..." messages

Phase 3 - Social Integration:
  - NPC dialogue checks refraction level
  - Faction reputation modifiers
  - Special dialogue branches unlock/lock
  - Shopkeeper pricing affected
```

**DES Testing:**

```json
{
  "player": { "refraction": 45 },
  "actions": [
    { "turn": 0, "action": { "type": "talk_to_npc", "npc_id": "monk_halix" } }
  ],
  "assertions": [
    {
      "at_end": true,
      "check": {
        "type": "dialogue_option_available",
        "option": "discuss_glass_nature"
      }
    },
    {
      "at_end": true,
      "check": { "type": "player_has_adaptation", "adaptation": "glass_skin" }
    }
  ]
}
```

**Why Critical:** Adaptations are the game's unique identity hook:

- Risk/reward decision making (power vs social consequences)
- Emergent playstyle differentiation
- Ties into narrative (what does it mean to become glass?)
- Creates faction-based gameplay variance

---

### **NEW 3. NPC Faction System (Importance: 5, Difficulty: 3) [CRITICAL FOR MINIMAL STATE]**

**Current State:** Basic NPC dialogue exists, no faction reputation system.

**Required for Minimal State:**

- ⬜ Faction enum (Monks, Engineers, Glassborn, Hermits, Archives)
- ⬜ Reputation tracking (-100 to +100 per faction)
- ⬜ Faction-specific dialogue trees with adaptation/reputation checks
- ⬜ Trade access gated by faction standing
- ⬜ Quest availability filtered by faction
- ⬜ 3-5 signature NPCs per faction with distinct personalities

**Design Notes:**

- Factions have conflicting goals (can't max all reputations)
- Monk reputation ↑ when accepting adaptations, ↓ when using tech
- Engineer reputation ↑ when using tech, ↓ when mysticism
- Glassborn reputation ↑ with high refraction, ↓ with low
- Archive reputation based on saint-key possession and protocol compliance

**Implementation Approach:**

```
Phase 1 - Reputation System:
  - Add faction_reputation HashMap to GameState
  - Track rep changes from dialogue choices, quest completions
  - Display faction standings in UI

Phase 2 - Gated Content:
  - NPC dialogue checks reputation thresholds
  - Quest givers only offer quests if rep > threshold
  - Shopkeepers have faction-specific inventory

Phase 3 - Dynamic Reactions:
  - NPCs greet player differently based on rep
  - Hostile factions attack on sight at very low rep
  - Allied factions offer assistance in combat
```

**DES Testing:**

```json
{
  "player": { "faction_reputation": { "monks": 50, "engineers": -20 } },
  "actions": [
    { "turn": 0, "action": { "type": "talk_to_npc", "npc_id": "monk_halix" } }
  ],
  "assertions": [
    {
      "at_end": true,
      "check": { "type": "dialogue_greeting", "expected": "friendly" }
    },
    {
      "at_end": true,
      "check": { "type": "quest_available", "quest_id": "storm_walk_trial" }
    }
  ]
}
```

**Why Critical:** Factions create:

- Replayability (different faction allegiances = different runs)
- Consequence depth (choices matter beyond single encounters)
- Narrative investment (NPCs remember you)
- Strategic decision-making (who to trust)

---

### 4. Turn-Based System (Importance: 5, Difficulty: 2)

**Current State:** 80% - AP system implemented. Actions consume AP, auto-end turn when depleted.

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

### 5. Combat System (Importance: 5, Difficulty: 4)

**Current State:** 70% - Combat math, ranged attacks, and status effects implemented.

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

### 6. Player Movement & Interaction (Importance: 5, Difficulty: 2)

**Current State:** 60% - 8-directional movement, item pickup, basic FOV, NPC dialogue implemented.

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

### 7. Map and Environment (Importance: 5, Difficulty: 4)

**Current State:** 65% - Single-tile procgen with rooms, corridors, FOV, storm glass conversion. World map generation implemented with biomes, terrain, elevation, and POI placement.

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

### 8. Enemy AI and NPCs (Importance: 4, Difficulty: 4)

**Current State:** 50% - Demeanor system implemented with multiple behavior types (aggressive, defensive, neutral, pacifist).

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

### 9. UI and HUD (Importance: 4, Difficulty: 3)

**Current State:** 70% - Full HUD implemented with side panel, bottom panel, inventory, quest log, and crafting menus.

**Completed:**

- ✅ Side panel with player stats, equipment, active quests
- ✅ Bottom panel with message log and hotkey reference
- ✅ Inventory screen (i key)
- ✅ Quest log screen (Q key)
- ✅ Crafting menu (c key)

**Missing:**

- Inspection mode ASCII art
- World map view

**Note:** UI is presentation-only, not tested via DES.

---

### 10. Inventory and Equipment (Importance: 4, Difficulty: 2)

**Current State:** 90% - Full equipment system implemented with slots, stat bonuses, item inspection, and equippable light sources.

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

### 11. Character Progression (Importance: 3, Difficulty: 3)

**Current State:** 60% - XP and level-up system implemented with stat allocation.

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

### 12. Exploration (Importance: 3, Difficulty: 2)

**Current State:** 70% - FOV reveals tiles, explored tiles persist in memory with proper rendering.

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

### 13. Quests and Storytelling (Importance: 3, Difficulty: 4)

**Current State:** 60% - Quest system implemented with data-driven objectives and rewards, DES support.

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

### 14. Crafting System (Importance: 2, Difficulty: 3)

**Current State:** 60% - Data-driven crafting system implemented with recipes and DES support.

**Completed:**

- ✅ Recipe definitions in data/recipes.json
- ✅ Material requirements checking
- ✅ GameState::craft() method
- ✅ DES support (craft action)

**Missing:**

- Crafting UI
- Crafting stations (location-based crafting)

---

### 15. Psychic Abilities (Importance: 3, Difficulty: 4)

**Current State:** 0% - PSY stat exists in HUD but no abilities implemented.

**Required for Full Implementation:**

- ⬜ PSY resource management (recharge per turn, max PSY stat)
- ⬜ Psychic ability types:
  - Mind Control: take control of enemy for 1 turn (high PSY cost)
  - Telekinesis: move objects/enemies remotely (medium PSY cost)
  - Solar Ray: beam attack using light refraction (low PSY cost, requires line to light source)
  - Psychic Blast: area damage with pushback (high PSY cost)
  - Psychic Shield: temporary damage reduction (medium PSY cost)
- ⬜ Ability cooldowns to prevent spam
- ⬜ PSY regeneration mechanics (meditation, consumables, light exposure)
- ⬜ Psychic overload consequences (high refraction gain, temporary debuffs)

**Design Notes:**

- Psychic use increases refraction (ties into adaptation system)
- Some abilities only available at certain refraction thresholds
- Light sources enhance psychic power (proximity to torch/bright areas)
- Psycic Savant builds rely heavily on this system

**Implementation Approach:**

```
Phase 1 - Basic Abilities:
  - Define abilities in data/abilities.json
  - PSY cost checking and deduction
  - Simple targeting (single enemy, area)

Phase 2 - Advanced Mechanics:
  - Cooldown tracking per ability
  - Light source proximity bonuses
  - Refraction synergies

Phase 3 - Consequences:
  - Psychic overload status effect
  - NPC reactions to psychic use
  - Archive drone responses
```

**DES Testing:**

```json
{
  "player": { "max_psy": 20, "psy": 20 },
  "actions": [
    {
      "turn": 0,
      "action": {
        "type": "use_ability",
        "ability": "mind_control",
        "target_x": 12,
        "target_y": 10
      }
    }
  ],
  "assertions": [
    { "at_turn": 0, "check": { "type": "player_psy", "expected": 12 } },
    { "at_turn": 0, "check": { "type": "enemy_controlled", "enemy_id": 0 } }
  ]
}
```

**Priority Note:** Not critical for minimal state, but important for build diversity and unique mechanics.

---

### 16. Modularity/Modding (Importance: 2, Difficulty: 3)

**Current State:** 40% - Items, enemies, NPCs, quests, recipes, and weapons are data-driven (JSON).

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

### 17. Audio and Sound (Importance: 1, Difficulty: 2)

**Current State:** 0% - Not implemented (TUI game).

**Notes:** Could add terminal bell or external audio library later. Not critical for TUI roguelike.

---

## Recommended Implementation Order

### **CRITICAL PATH TO MINIMAL WORKING STATE (4-6 weeks)**

**Milestone 0: Storm & Adaptation Foundation (2 weeks)**

1. ⬜ Glass Storm System (core mechanic)
   - Storm timer and triggers
   - Tile editing (wall → glass conversion)
   - Storm Glass drops
   - Storm forecast UI
2. ⬜ Adaptation System (identity mechanic)
   - Refraction stat tracking
   - 5-8 basic adaptations with thresholds
   - Visual indicators in UI
   - Adaptation unlock messages

**Milestone 1: Social Consequences (1 week)**

1. ⬜ NPC Faction System
   - Faction reputation tracking (Monks, Engineers, Glassborn)
   - Faction-specific dialogue with adaptation checks
   - Trade/quest access gating
   - 3-5 signature NPCs per faction

**Milestone 2: World Integration (1 week)**

1. ⬜ World Tile Transition
   - World map UI rendering
   - Lazy tile generation on entry
   - Save/load tile state
   - Storm effects on world map display

**Milestone 3: Content & Polish (1 week)**

1. ⬜ Starter Quest Content
   - 3-5 faction-branching quests
   - Tutorial quest introducing storms/adaptations
   - Faction introduction quests
2. ⬜ Enemy Distribution
   - 10-15 enemy types assigned to biomes
   - Balance pass on combat difficulty
3. ⬜ Inspection Mode
   - ASCII art for 3-5 signature items
   - ASCII art for 3-5 key NPCs
   - Landmark inspection support

**At this point: PLAYABLE VERTICAL SLICE**

- 30-60 minute gameplay loop
- Core creative vision demonstrated
- Storm-driven tactical challenge
- Adaptation with consequences
- Faction-differentiated gameplay

---

### ALREADY COMPLETED SYSTEMS (Strong Foundation)

**Milestone ✅: Core Loop (Completed)**

1. ✅ Turn system basics
2. ✅ AP system with action costs
3. ✅ Combat math (hit/damage/armor)
4. ✅ Ranged attacks with LOS and ammo
5. ✅ Status effects (Poison, Burn, Stun, Bleed, Slow)
6. ✅ Critical hits

**Milestone ✅: World & Exploration (Completed)**

1. ✅ Explored tile memory
2. ✅ World map generation (64x64 grid)
3. ✅ Biome/terrain/elevation layers
4. ✅ POI placement with distance penalty
5. ✅ Tile-from-world-seed generation
6. ✅ Dynamic lighting system

**Milestone ✅: Depth Systems (Completed)**

1. ✅ Inventory system with item inspection
2. ✅ Equipment system (weapon/armor/accessory slots)
3. ✅ AI demeanor types (aggressive, defensive, neutral, pacifist)
4. ✅ NPC dialogue system with conditions
5. ✅ Character progression (XP, levels, stat points)
6. ✅ Equippable light sources

**Milestone ✅: Content Infrastructure (Completed)**

1. ✅ Quest system (data-driven with objectives)
2. ✅ Crafting system (data-driven recipes)
3. ✅ 16 enemy types defined
4. ✅ 25+ items defined
5. ✅ 6 weapons defined

**Milestone ✅: UI/UX Polish (Mostly Complete)**

1. ✅ Full HUD (side panel, bottom panel)
2. ✅ Inventory screen
3. ✅ Quest log screen
4. ✅ Crafting menu
5. ⬜ Inspection mode ASCII art (pending)
6. ⬜ World map view (pending)

---

### POST-MINIMAL STATE (Future Enhancements)

**Milestone 4: Advanced Combat (Future)**

1. ⬜ Psychic abilities system
2. ⬜ Body part targeting
3. ⬜ Cover mechanics (crouch/take cover)
4. ⬜ Dodge/evasion rolls
5. ⬜ Interruptions/reactions

**Milestone 5: World Complexity (Future)**

1. ⬜ Subterranean layers
2. ⬜ Time of day system
3. ⬜ Liquid/hazard simulation
4. ⬜ Environmental interactions (levers, traps)
5. ⬜ Auto-explore pathfinding

**Milestone 6: AI Depth (Future)**

1. ⬜ Patrol routes
2. ⬜ Aggro system with target switching
3. ⬜ Cover-seeking behavior
4. ⬜ Enemy item usage

**Milestone 7: Progression Depth (Future)**

1. ⬜ Skill trees (if keeping)
2. ⬜ Background system with starting bonuses
3. ⬜ Specializations
4. ⬜ Companion progression

**Milestone 8: Modding Support (Future)**

1. ⬜ Mod loading system
2. ⬜ Override/extend base data
3. ⬜ Script hooks (Lua?)
4. ⬜ Modding documentation

**Milestone 9: Audio (Optional)**

1. ⬜ Ambient soundscapes
2. ⬜ UI feedback sounds
3. ⬜ Combat sounds
4. ⬜ Musical themes

---

## Success Metrics for Minimal State

**Technical:**

- [ ] Game runs for 30-60 minutes without crashes
- [ ] Storms occur predictably (every 50-100 turns)
- [ ] Tile generation completes in < 1 second
- [ ] Save/load preserves all state correctly

**Creative Vision:**

- [ ] Players experience at least 2 storms in a run
- [ ] Players gain at least 1-2 adaptations
- [ ] Players interact with all 3 core factions
- [ ] Map changes from storms create meaningful tactical choices

**Gameplay:**

- [ ] Combat feels fair and tactical (hit chances 5-95%)
- [ ] Resource management matters (AP, PSY, items, Storm Glass)
- [ ] Faction choices create dilemmas (can't please everyone)
- [ ] Runs feel different due to adaptations + faction paths

**Content:**

- [ ] 3-5 starter quests fully implemented
- [ ] 10+ enemy types distributed across biomes
- [ ] 5+ adaptation types with distinct effects
- [ ] 20+ items with meaningful uses
