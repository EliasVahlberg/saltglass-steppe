# Implementation Tasks for Minimal Working State

**Assigned To:** FeatureDeveloper  
**Priority:** CRITICAL PATH - Complete in order listed  
**Timeline:** 5-7 weeks estimated (updated after gameplay analysis)

**⚠️ IMPORTANT:** Read `GAMEPLAY_SIMULATION_ANALYSIS.md` first to understand why these features are critical for a fun experience.

---

## Work Process for Each Task

1. **Implement** - Write the feature code
2. **DES test** - Create automated test scenarios
3. **[If fail]** Troubleshoot/fix/test
4. **[If working]** Document - Update relevant docs
5. **Commit** - Git commit with clear message

### Key Reminders

- **Decouple systems as much as possible** - Keep game logic separate from UI
- **Favor data-driven implementations** - Use JSON files to make it easier to add more content and reuse logic
- **Test with DES** - Every feature should have automated test coverage
- **Follow existing patterns** - Look at how items, enemies, quests are implemented

---

## TASK 0: Critical Quality-of-Life Improvements [NEW - 1 week]

**Priority:** HIGHEST - Must be done BEFORE storm system to prevent frustration

**Rationale:** Gameplay simulation revealed that without these features, players will bounce off the game immediately. These are not "nice to have" - they are essential for basic playability.

### Subtask 0.1: Tutorial & Onboarding (2 days)

**Implementation:**

- Create tutorial message sequence that triggers on game start
- Display control hints in HUD footer
- First NPC ("Hermit Guide") provides tutorial dialogue
- Tutorial quest teaches movement, combat, item use, rest

**Data-Driven Approach:**

```json
// data/tutorial.json (new file)
{
  "messages": [
    {
      "trigger": "game_start",
      "text": "Welcome to the Saltglass Steppe. Use arrow keys to move.",
      "highlight": "movement",
      "dismiss_key": "space"
    },
    {
      "trigger": "first_enemy_visible",
      "text": "Press Space to attack adjacent enemies. Combat consumes AP.",
      "highlight": "combat"
    },
    {
      "trigger": "first_item_visible",
      "text": "Walk over items to pick them up. Press 'i' to view inventory.",
      "highlight": "inventory"
    },
    {
      "trigger": "low_hp",
      "text": "Your HP is low! Use healing items with 'u' or rest with 'r'.",
      "highlight": "healing"
    }
  ]
}
```

**Files to Create:**

- `src/game/tutorial.rs` (new) - Tutorial system
- `data/tutorial.json` (new) - Tutorial messages

**Files to Modify:**

- `src/game/state.rs` - Track tutorial progress
- `src/ui/hud.rs` - Display control hints

---

### Subtask 0.2: Rest Mechanic (1 day)

**Implementation:**

- Add `rest()` function to GameState
- Check: no enemies within sight range
- Effect: Heal 50% max HP, advance 10 turns
- Display message: "You rest and recover..."

**DES Test Scenario:**

```json
{
  "name": "rest_mechanic",
  "seed": 42,
  "player": { "x": 10, "y": 10, "hp": 10, "max_hp": 20 },
  "actions": [{ "turn": 0, "action": { "type": "rest" } }],
  "assertions": [
    { "at_turn": 10, "check": { "type": "player_hp", "expected": 20 } },
    { "at_turn": 10, "check": { "type": "message_logged", "contains": "rest" } }
  ]
}
```

**Files to Modify:**

- `src/game/state.rs` - Add rest() function
- `src/main.rs` - Handle 'r' key for rest

---

### Subtask 0.3: Enemy Loot Tables (2 days)

**Implementation:**

- Add `loot_table` field to enemy definitions
- On enemy death, roll loot table and spawn items
- Items appear at enemy death location

**Data-Driven Approach:**

```json
// In data/enemies.json - add loot_table to all enemies
{
  "id": "glass_beetle",
  "loot_table": [
    { "item": "glass_shard", "chance": 0.4 },
    { "item": "beetle_carapace", "chance": 0.2 },
    { "item": "brine_vial", "chance": 0.15 },
    { "currency": "salt_scrip", "min": 5, "max": 15, "chance": 0.3 }
  ]
}
```

**DES Test Scenario:**

```json
{
  "name": "enemy_loot_drops",
  "seed": 42,
  "player": { "x": 10, "y": 10 },
  "entities": [
    { "entity_type": "enemy", "id": "glass_beetle", "x": 12, "y": 10 }
  ],
  "actions": [
    {
      "turn": 0,
      "action": { "type": "attack", "target_x": 12, "target_y": 10 }
    },
    {
      "turn": 1,
      "action": { "type": "attack", "target_x": 12, "target_y": 10 }
    },
    {
      "turn": 2,
      "action": { "type": "attack", "target_x": 12, "target_y": 10 }
    }
  ],
  "assertions": [
    {
      "at_end": true,
      "check": { "type": "item_at_position", "x": 12, "y": 10 }
    }
  ]
}
```

**Files to Modify:**

- `src/game/enemy.rs` - Add loot_table field to EnemyDef
- `src/game/combat.rs` - Drop loot on enemy death
- `data/enemies.json` - Add loot tables to all 16 enemies

---

### Subtask 0.4: Quest Chain System (2 days)

**Implementation:**

- Add `unlocks_quest` field to quest rewards
- Add `requires_quest_completed` field to quest definitions
- When quest completes, unlock next quest in chain
- Tutorial quest → Faction intro quests → Main questline

**Data-Driven Approach:**

```json
// In data/quests.json - chain quests together
{
  "id": "tutorial_basics",
  "name": "Survival Basics",
  "description": "Learn to survive in the Steppe.",
  "objectives": [
    {"type": "kill", "enemy_id": "mirage_hound", "count": 1},
    {"type": "collect", "item_id": "brine_vial", "count": 1},
    {"type": "rest", "count": 1}
  ],
  "reward": {
    "xp": 25,
    "items": ["rusty_blade"],
    "unlocks_quest": "choose_faction"
  }
},
{
  "id": "choose_faction",
  "name": "A Path Forward",
  "requires_quest_completed": "tutorial_basics",
  "description": "Speak to faction representatives.",
  "objectives": [
    {"type": "talk_to_npc", "npc_id": "hermit_guide"}
  ],
  "reward": {
    "xp": 50,
    "unlocks_quest": ["monk_intro", "engineer_intro", "glassborn_intro"]
  }
}
```

**Files to Modify:**

- `src/game/quest.rs` - Add quest unlocking logic
- `data/quests.json` - Add 5-7 chained quests

---

### Subtask 0.5: Currency & Basic Economy (1 day)

**Implementation:**

- Add `salt_scrip: u32` to Player (currency)
- Display in HUD: "$: 127"
- Items have `value` field (already exists)
- NPCs can buy items at 50% value, sell at 100% value

**Data-Driven Approach:**

```json
// In data/npcs.json - add shop inventory
{
  "id": "hermit_merchant",
  "name": "Hermit Merchant",
  "is_merchant": true,
  "shop_inventory": [
    { "item_id": "brine_vial", "stock": 5, "price": 10 },
    { "item_id": "health_potion", "stock": 3, "price": 25 },
    { "item_id": "rusty_blade", "stock": 1, "price": 50 }
  ],
  "buys_items": true,
  "buy_price_multiplier": 0.5
}
```

**Files to Modify:**

- `src/game/state.rs` - Add salt_scrip field
- `src/game/npc.rs` - Add shop functionality
- `src/ui/shop.rs` (new) - Shop UI

---

## TASK 1: Glass Storm System [CRITICAL - 2 weeks]

**Priority:** HIGHEST - This is THE defining mechanic of Saltglass Steppe

### Subtask 1.1: Storm Timer and Tracking (2 days)

**Implementation:**

- Add `storm_timer: u32` to `GameState` (turns until next storm)
- Add `storm_history: Vec<StormEvent>` to track past storms
- Create `StormEvent` struct with: `turn, world_x, world_y, intensity, tiles_affected`
- Add storm timer display to top HUD (e.g., "Storm: 26" turns remaining)
- Decrement timer each turn, trigger storm when reaches 0
- Reset timer to random range (50-100 turns) after storm

**Data-Driven Approach:**

```json
// data/storm_config.json (new file)
{
  "min_interval": 50,
  "max_interval": 100,
  "base_intensity": 10,
  "intensity_variance": 5,
  "glass_conversion_chance": 0.3,
  "storm_glass_drop_chance": 0.15
}
```

**DES Test Scenario:**

```json
{
  "name": "storm_timer_basic",
  "seed": 42,
  "player": { "x": 10, "y": 10 },
  "storm_timer": 5,
  "actions": [
    { "turn": 0, "action": { "type": "wait" } },
    { "turn": 1, "action": { "type": "wait" } },
    { "turn": 4, "action": { "type": "wait" } }
  ],
  "assertions": [
    { "at_turn": 0, "check": { "type": "storm_timer", "expected": 5 } },
    { "at_turn": 4, "check": { "type": "storm_timer", "expected": 1 } },
    { "at_turn": 5, "check": { "type": "storm_occurred" } }
  ]
}
```

**Files to Modify:**

- `src/game/state.rs` - Add storm timer fields
- `src/game/storm.rs` (new) - Storm system logic
- `src/ui/hud.rs` - Display storm timer
- `data/storm_config.json` (new) - Storm parameters

---

### Subtask 1.2: Tile Editing - Wall to Glass Conversion (3 days)

**Implementation:**

- Create `apply_storm_to_tile(map: &mut Map, rng: &mut ChaCha8Rng, intensity: u32)` function
- Select random wall tiles based on intensity
- Convert `Tile::Wall { wall_type }` → `Tile::GlassWall { original_type }`
- Update FOV/pathfinding after conversion (glass walls should be transparent but not walkable)
- Add visual effect markers during storm (flashing tiles, color changes)

**Data-Driven Approach:**

```json
// In data/tiles.json - add glass wall variants
{
  "glass_wall_sandstone": {
    "glyph": "*",
    "color": "Cyan",
    "transparent": true,
    "walkable": false,
    "description": "A wall of vitrified sandstone, glowing with inner light"
  }
}
```

**DES Test Scenario:**

```json
{
  "name": "storm_glass_conversion",
  "seed": 42,
  "player": { "x": 10, "y": 10 },
  "map_preset": "room_with_walls",
  "actions": [
    { "turn": 0, "action": { "type": "trigger_storm", "intensity": 20 } }
  ],
  "assertions": [
    {
      "at_turn": 1,
      "check": {
        "type": "tile_type_count",
        "tile_type": "glass_wall",
        "min": 5
      }
    },
    {
      "at_turn": 1,
      "check": {
        "type": "tile_transparent",
        "x": 12,
        "y": 10,
        "expected": true
      }
    }
  ]
}
```

**Files to Modify:**

- `src/game/storm.rs` - Add tile conversion logic
- `src/game/map.rs` - Update FOV for glass walls
- `data/tiles.json` - Add glass wall tile definitions
- `src/game/tile.rs` - Add `GlassWall` variant if needed

---

### Subtask 1.3: Storm Glass Currency Drops (2 days)

**Implementation:**

- When storm converts tiles, chance to spawn `storm_glass` item at location
- Add `storm_glass` to `data/items.json` if not exists
- Storm Glass should be high-value currency item (tradeable, craftable)
- Drop rate based on storm intensity (higher intensity = more drops)

**Data-Driven Approach:**

```json
// In data/items.json
{
  "id": "storm_glass",
  "name": "Storm Glass",
  "glyph": "◆",
  "color": "Cyan",
  "description": "Crystallized storm energy. Proof of survival. Currency of the desperate.",
  "value": 50,
  "weight": 0.1,
  "usable": false,
  "stackable": true,
  "effects": ["B(@3 &Cyan)", "G(@2)"]
}
```

**DES Test Scenario:**

```json
{
  "name": "storm_glass_drops",
  "seed": 42,
  "player": { "x": 10, "y": 10 },
  "actions": [
    { "turn": 0, "action": { "type": "trigger_storm", "intensity": 30 } }
  ],
  "assertions": [
    {
      "at_turn": 1,
      "check": {
        "type": "item_exists_on_map",
        "item_id": "storm_glass",
        "min_count": 3
      }
    }
  ]
}
```

**Files to Modify:**

- `src/game/storm.rs` - Add item spawning logic
- `data/items.json` - Add/verify storm_glass definition

---

### Subtask 1.4: Storm Forecast System (2 days)

**Implementation:**

- Add `get_storm_forecast(&self) -> Option<u32>` to GameState
- Base forecast on: storm_timer, player items (Storm Compass), NPC interactions
- Create `storm_compass` item that reveals exact storm timer
- Without compass, show approximate warning ("Storm approaching", "Storm distant")

**Data-Driven Approach:**

```json
// In data/items.json
{
  "id": "storm_compass",
  "name": "Storm Compass",
  "glyph": "⊕",
  "color": "Yellow",
  "description": "An Archive-era device that tracks atmospheric refraction patterns.",
  "value": 200,
  "weight": 0.5,
  "usable": false,
  "equippable": true,
  "slot": "accessory",
  "effects": ["reveals_storm_timer"]
}
```

**DES Test Scenario:**

```json
{
  "name": "storm_forecast_with_compass",
  "seed": 42,
  "player": {
    "x": 10,
    "y": 10,
    "inventory": ["storm_compass"],
    "equipped": { "accessory": "storm_compass" }
  },
  "storm_timer": 25,
  "assertions": [
    {
      "at_turn": 0,
      "check": { "type": "storm_forecast_available", "expected": true }
    },
    {
      "at_turn": 0,
      "check": { "type": "storm_forecast_value", "expected": 25 }
    }
  ]
}
```

**Files to Modify:**

- `src/game/storm.rs` - Add forecast logic
- `src/ui/hud.rs` - Display forecast in HUD
- `data/items.json` - Add storm_compass item

---

### Subtask 1.5: Safe Zone Indicators (1 day)

**Implementation:**

- Mark certain tiles as "storm_safe" (e.g., underground areas, archive wings)
- Visual indicator in UI (different background color, special glyph overlay)
- Player takes no refraction damage if in safe zone during storm

**DES Test Scenario:**

```json
{
  "name": "storm_safe_zones",
  "seed": 42,
  "player": { "x": 5, "y": 5 },
  "map_metadata": { "safe_zones": [{ "x": 5, "y": 5, "radius": 2 }] },
  "actions": [
    { "turn": 0, "action": { "type": "trigger_storm", "intensity": 30 } }
  ],
  "assertions": [
    { "at_turn": 1, "check": { "type": "player_refraction", "expected": 0 } },
    { "at_turn": 1, "check": { "type": "player_alive" } }
  ]
}
```

**Files to Modify:**

- `src/game/map.rs` - Add safe zone metadata
- `src/game/storm.rs` - Check safe zones before applying damage
- `src/ui/render.rs` - Visual indicators for safe zones

---

## TASK 2: Adaptation System [CRITICAL - 1 week]

**Priority:** HIGH - Core identity mechanic

### Subtask 2.1: Refraction Stat Tracking (1 day)

**Implementation:**

- Add `refraction: u32` to Player struct (0-100 scale)
- Display in top HUD (e.g., "REF: 35")
- Add functions: `gain_refraction(amount)`, `lose_refraction(amount)`, `get_refraction()`
- Triggers: storm exposure, Storm Glass consumption, psychic ability use

**DES Test Scenario:**

```json
{
  "name": "refraction_tracking",
  "seed": 42,
  "player": { "x": 10, "y": 10, "refraction": 0 },
  "actions": [
    { "turn": 0, "action": { "type": "trigger_storm", "intensity": 20 } },
    { "turn": 1, "action": { "type": "use_item", "item_id": "storm_glass" } }
  ],
  "assertions": [
    { "at_turn": 1, "check": { "type": "player_refraction", "min": 5 } },
    { "at_turn": 2, "check": { "type": "player_refraction", "min": 10 } }
  ]
}
```

**Files to Modify:**

- `src/game/player.rs` - Add refraction field
- `src/game/storm.rs` - Grant refraction on storm exposure
- `src/ui/hud.rs` - Display refraction in HUD

---

### Subtask 2.2: Adaptation Definitions (Data-Driven) (2 days)

**Implementation:**

- Create `data/adaptations.json` with 5-8 adaptations
- Each adaptation has: id, name, description, threshold, stat_modifiers, special_abilities
- Load adaptations via `once_cell::Lazy` like items/enemies

**Data-Driven Approach:**

```json
// data/adaptations.json (new file)
[
  {
    "id": "crystalline_vision",
    "name": "Crystalline Vision",
    "threshold": 20,
    "description": "Your eyes refract light like living prisms. You can see through glass walls.",
    "stat_modifiers": {},
    "special_abilities": ["see_through_glass"],
    "visual_changes": { "eye_glyph": "◊" }
  },
  {
    "id": "glass_skin",
    "name": "Glass Skin",
    "threshold": 40,
    "description": "Your skin hardens into translucent crystal. Armor bonus, but movement penalty.",
    "stat_modifiers": { "armor": 3, "movement_speed": -2 },
    "special_abilities": [],
    "visual_changes": { "skin_tone": "Cyan" }
  },
  {
    "id": "light_sensitivity",
    "name": "Light Sensitivity",
    "threshold": 60,
    "description": "Intense light sources cause pain. Bright areas deal damage over time.",
    "stat_modifiers": {},
    "special_abilities": ["light_damage"],
    "visual_changes": { "glow_effect": "G(@2)" }
  },
  {
    "id": "prism_reflex",
    "name": "Prism Reflex",
    "threshold": 80,
    "description": "Your body bends light reflexively. Beam attacks split to nearby targets.",
    "stat_modifiers": { "reflex": 2 },
    "special_abilities": ["beam_splash"],
    "visual_changes": { "aura": "B(@3 &Cyan)" }
  }
]
```

**Files to Create:**

- `src/game/adaptation.rs` (new) - Adaptation struct and loading
- `data/adaptations.json` (new) - Adaptation definitions

---

### Subtask 2.3: Adaptation Unlock Logic (1 day)

**Implementation:**

- Check refraction thresholds each turn in `GameState::end_turn()`
- When threshold reached, unlock adaptation and log message
- Add `adaptations: Vec<String>` to Player (list of unlocked adaptation IDs)
- Apply stat modifiers from adaptations to player stats

**DES Test Scenario:**

```json
{
  "name": "adaptation_unlock",
  "seed": 42,
  "player": { "x": 10, "y": 10, "refraction": 19 },
  "actions": [
    { "turn": 0, "action": { "type": "gain_refraction", "amount": 5 } }
  ],
  "assertions": [
    {
      "at_turn": 1,
      "check": {
        "type": "player_has_adaptation",
        "adaptation": "crystalline_vision"
      }
    },
    {
      "at_turn": 1,
      "check": { "type": "message_logged", "contains": "Crystalline Vision" }
    }
  ]
}
```

**Files to Modify:**

- `src/game/adaptation.rs` - Unlock checking logic
- `src/game/player.rs` - Add adaptations field
- `src/game/state.rs` - Check thresholds in end_turn()

---

### Subtask 2.4: Visual Indicators (1 day)

**Implementation:**

- Update player ASCII art in side panel based on adaptations
- Add glyph effects to player tile (color changes, effects like G/B)
- Show adaptation list in character screen or side panel

**Files to Modify:**

- `src/ui/side_panel.rs` - Update player visual representation
- `src/ui/render.rs` - Apply visual effects to player tile
- `src/ui/character_screen.rs` - List adaptations

---

### Subtask 2.5: Special Ability Implementation (2 days)

**Implementation:**

- Implement special abilities: `see_through_glass`, `light_damage`, `beam_splash`
- `see_through_glass`: Modify FOV calculation to ignore glass walls
- `light_damage`: Apply damage each turn if player in high-light tile
- `beam_splash`: When hit by beam, deal damage to adjacent tiles

**DES Test Scenario:**

```json
{
  "name": "crystalline_vision_ability",
  "seed": 42,
  "player": { "x": 10, "y": 10, "adaptations": ["crystalline_vision"] },
  "map_preset": "glass_wall_corridor",
  "assertions": [
    {
      "at_turn": 0,
      "check": { "type": "can_see_tile", "x": 15, "y": 10, "expected": true }
    }
  ]
}
```

**Files to Modify:**

- `src/game/fov.rs` - Modify for see_through_glass
- `src/game/state.rs` - Apply light_damage in turn logic
- `src/game/combat.rs` - Add beam_splash logic

---

## TASK 3: NPC Faction System [CRITICAL - 1 week]

**Priority:** HIGH - Social consequence layer

### Subtask 3.1: Faction Reputation Tracking (1 day)

**Implementation:**

- Add `faction_reputation: HashMap<String, i32>` to GameState (-100 to +100)
- Create faction enum or use string IDs: "monks", "engineers", "glassborn", "hermits", "archives"
- Add functions: `modify_reputation(faction, delta)`, `get_reputation(faction)`

**Data-Driven Approach:**

```json
// data/factions.json (new file)
[
  {
    "id": "monks",
    "name": "Mirror Monks",
    "description": "Interpreters of storm scripture. Reverence for adaptation.",
    "base_reputation": 0,
    "rep_thresholds": {
      "hostile": -50,
      "unfriendly": -20,
      "neutral": 0,
      "friendly": 20,
      "allied": 50
    }
  },
  {
    "id": "engineers",
    "name": "Sand-Engineers",
    "description": "Pragmatic builders. Distrust mysticism.",
    "base_reputation": 0
  },
  {
    "id": "glassborn",
    "name": "Glassborn",
    "description": "The transformed. Kinship through refraction.",
    "base_reputation": 0
  }
]
```

**DES Test Scenario:**

```json
{
  "name": "faction_reputation_tracking",
  "seed": 42,
  "player": { "x": 10, "y": 10 },
  "faction_reputation": { "monks": 0, "engineers": 0 },
  "actions": [
    {
      "turn": 0,
      "action": { "type": "modify_reputation", "faction": "monks", "delta": 10 }
    },
    {
      "turn": 1,
      "action": {
        "type": "modify_reputation",
        "faction": "engineers",
        "delta": -5
      }
    }
  ],
  "assertions": [
    {
      "at_turn": 1,
      "check": {
        "type": "faction_reputation",
        "faction": "monks",
        "expected": 10
      }
    },
    {
      "at_turn": 2,
      "check": {
        "type": "faction_reputation",
        "faction": "engineers",
        "expected": -5
      }
    }
  ]
}
```

**Files to Create:**

- `src/game/faction.rs` (new) - Faction system
- `data/factions.json` (new) - Faction definitions

**Files to Modify:**

- `src/game/state.rs` - Add faction_reputation field

---

### Subtask 3.2: Faction-Specific Dialogue (2 days)

**Implementation:**

- Extend NPC dialogue system to check faction reputation
- Add `faction_requirement` field to dialogue options
- Add `refraction_requirement` field to dialogue options
- NPCs should greet player differently based on faction standing

**Data-Driven Approach:**

```json
// In data/npcs.json - extend dialogue structure
{
  "id": "monk_halix",
  "faction": "monks",
  "dialogues": [
    {
      "id": "greeting_allied",
      "faction_requirement": { "monks": 50 },
      "text": "Brother Halix bows deeply. 'The storms have blessed you, marked one.'",
      "options": [
        { "text": "Discuss storm prophecy", "next": "prophecy_branch" }
      ]
    },
    {
      "id": "greeting_hostile",
      "faction_requirement": { "monks": -50 },
      "text": "Brother Halix turns away. 'The unmarked have no place in our choir.'",
      "options": []
    },
    {
      "id": "greeting_adapted",
      "refraction_requirement": 40,
      "text": "Brother Halix studies your crystalline features. 'You bear the shimmer...'",
      "options": [
        { "text": "Ask about adaptations", "next": "adaptation_lore" }
      ]
    }
  ]
}
```

**DES Test Scenario:**

```json
{
  "name": "faction_dialogue_gating",
  "seed": 42,
  "player": { "x": 10, "y": 10, "refraction": 45 },
  "faction_reputation": { "monks": 55 },
  "entities": [{ "entity_type": "npc", "id": "monk_halix", "x": 11, "y": 10 }],
  "actions": [
    { "turn": 0, "action": { "type": "talk_to_npc", "npc_id": "monk_halix" } }
  ],
  "assertions": [
    {
      "at_end": true,
      "check": {
        "type": "dialogue_option_available",
        "option": "Discuss storm prophecy"
      }
    },
    {
      "at_end": true,
      "check": { "type": "dialogue_greeting", "expected": "greeting_allied" }
    }
  ]
}
```

**Files to Modify:**

- `src/game/npc.rs` - Add faction/refraction checks to dialogue
- `src/game/dialogue.rs` - Extend dialogue filtering
- `data/npcs.json` - Add faction-specific dialogues

---

### Subtask 3.3: Reputation Modifiers from Actions (1 day)

**Implementation:**

- Actions trigger reputation changes:
  - Completing faction quests: +10 to +30 rep
  - Betraying faction: -20 to -50 rep
  - Using tech around Monks: -5 rep
  - Using mysticism around Engineers: -5 rep
  - High refraction around Glassborn: +2 rep per interaction
  - Low refraction around Glassborn: -2 rep per interaction

**DES Test Scenario:**

```json
{
  "name": "reputation_from_quest",
  "seed": 42,
  "player": { "x": 10, "y": 10 },
  "faction_reputation": { "monks": 0 },
  "quests_active": ["monk_storm_walk"],
  "actions": [
    {
      "turn": 0,
      "action": { "type": "complete_quest", "quest_id": "monk_storm_walk" }
    }
  ],
  "assertions": [
    {
      "at_turn": 1,
      "check": { "type": "faction_reputation", "faction": "monks", "min": 15 }
    }
  ]
}
```

**Files to Modify:**

- `src/game/quest.rs` - Add reputation rewards
- `src/game/state.rs` - Track action-based reputation changes

---

### Subtask 3.4: Faction-Gated Content (2 days)

**Implementation:**

- Quest availability filtered by faction reputation
- Shop inventory/pricing based on faction
- Faction-specific NPCs refuse to talk if hostile
- Some areas require faction alliance (guards block entrance)

**Data-Driven Approach:**

```json
// In data/quests.json - add faction requirements
{
  "id": "monk_storm_walk",
  "name": "The Storm Walk",
  "giver_npc": "monk_halix",
  "requirements": {
    "faction_reputation": { "monks": 10 },
    "min_refraction": 20
  },
  "objectives": [
    {
      "type": "reach",
      "x": 25,
      "y": 25,
      "description": "Enter the storm chapel"
    }
  ],
  "rewards": {
    "xp": 100,
    "items": ["scripture_shard"],
    "faction_reputation": { "monks": 15 }
  }
}
```

**DES Test Scenario:**

```json
{
  "name": "faction_quest_gating",
  "seed": 42,
  "player": { "x": 10, "y": 10, "refraction": 25 },
  "faction_reputation": { "monks": 5 },
  "entities": [{ "entity_type": "npc", "id": "monk_halix", "x": 11, "y": 10 }],
  "assertions": [
    {
      "at_end": true,
      "check": {
        "type": "quest_available",
        "quest_id": "monk_storm_walk",
        "expected": false
      }
    }
  ]
}
```

**Files to Modify:**

- `src/game/quest.rs` - Add faction requirement checking
- `src/game/shop.rs` - Faction-based pricing (if shops exist)
- `src/game/npc.rs` - Faction-based interaction blocking

---

## TASK 4: World Tile Transition [CRITICAL - 1 week]

**Priority:** MEDIUM-HIGH - Exploration loop completion

### Subtask 4.1: World Map UI (2 days)

**Implementation:**

- Create world map view mode (press 'M' to toggle)
- Render 64x64 world grid with biome colors/glyphs
- Show player position on world map
- Show POI markers (towns, dungeons, landmarks)
- Cursor navigation on world map

**Files to Create:**

- `src/ui/world_map.rs` (new) - World map rendering

**Files to Modify:**

- `src/ui/mod.rs` - Add world map mode
- `src/main.rs` - Handle 'M' key for world map toggle

---

### Subtask 4.2: Lazy Tile Generation (2 days)

**Implementation:**

- When player enters new world tile, generate local map
- Use `Map::generate_from_world(world, tile_x, tile_y, seed)`
- Cache generated tiles in HashMap (world_x, world_y) -> Map
- First tile player starts on should be more hospitable

**DES Test Scenario:**

```json
{
  "name": "lazy_tile_generation",
  "seed": 42,
  "player": { "world_x": 10, "world_y": 10 },
  "actions": [
    {
      "turn": 0,
      "action": { "type": "enter_world_tile", "world_x": 11, "world_y": 10 }
    }
  ],
  "assertions": [
    {
      "at_turn": 1,
      "check": {
        "type": "current_tile_generated",
        "world_x": 11,
        "world_y": 10
      }
    },
    {
      "at_turn": 1,
      "check": { "type": "player_position", "world_x": 11, "world_y": 10 }
    }
  ]
}
```

**Files to Modify:**

- `src/game/state.rs` - Add tile cache, lazy generation logic
- `src/game/map.rs` - Ensure generate_from_world is robust

---

### Subtask 4.3: Save/Load Tile State (2 days)

**Implementation:**

- Save all generated tiles to save file
- Include: map layout, items, enemies, NPCs, storm damage
- Load tiles from save file when entering previously visited tiles
- Tiles should remember storm modifications

**Files to Modify:**

- `src/game/state.rs` - Serialize/deserialize tile cache
- Save/load system (if exists, or create simple JSON save)

---

### Subtask 4.4: Storm Effects on World Map (1 day)

**Implementation:**

- Show storm location on world map (pulsing indicator)
- Storm affects current world tile only (for now)
- Display "Storm active on this tile" warning when entering stormy tile

**Files to Modify:**

- `src/ui/world_map.rs` - Render storm indicator
- `src/game/storm.rs` - Track storm world location

---

## TASK 5: Content Polish [CRITICAL - 1 week]

**Priority:** MEDIUM - Makes it feel complete

### Subtask 5.1: Starter Quests (3 days)

**Implementation:**

- Create 3-5 starter quests with faction branching
- Tutorial quest introducing storms/adaptations
- One quest per faction (Monks, Engineers, Glassborn)
- Quests should demonstrate unique mechanics

**Data-Driven Approach:**

```json
// In data/quests.json - add starter quests
{
  "id": "tutorial_first_storm",
  "name": "Baptism by Light",
  "giver_npc": "hermit_guide",
  "description": "Survive your first storm and collect Storm Glass.",
  "objectives": [
    { "type": "survive_storm", "count": 1 },
    { "type": "collect", "item_id": "storm_glass", "count": 3 }
  ],
  "rewards": {
    "xp": 50,
    "items": ["brine_vial"],
    "message": "The hermit nods. 'You've felt the light. Now you understand.'"
  }
}
```

**Files to Modify:**

- `data/quests.json` - Add 3-5 quests
- `data/npcs.json` - Create quest giver NPCs
- Test each quest with DES scenarios

---

### Subtask 5.2: Enemy Distribution (2 days)

**Implementation:**

- Assign 10-15 enemies to biomes (Saltflat, Oasis, Ruins, etc.)
- Update spawn tables to match biome
- Balance enemy stats for early/mid/late game

**Data-Driven Approach:**

```json
// In data/spawn_tables.json
{
  "saltflat": {
    "enemies": [
      { "id": "salt_crawler", "weight": 30 },
      { "id": "glass_spider", "weight": 20 },
      { "id": "shard_wraith", "weight": 10 }
    ]
  },
  "oasis": {
    "enemies": [
      { "id": "brine_lurker", "weight": 25 },
      { "id": "mirage_phantom", "weight": 15 }
    ]
  }
}
```

**Files to Modify:**

- `data/spawn_tables.json` - Update enemy distributions
- Test spawn rates with multiple runs

---

### Subtask 5.3: Inspection Mode ASCII Art (2 days)

**Implementation:**

- Create ASCII art for 3-5 signature items (storm_glass, saint_key, scripture_shard)
- Create ASCII art for 3-5 key NPCs (monk_halix, engineer_ressa, glassborn_sable)
- Inspection mode displays art + detailed lore text

**Example ASCII Art:**

```
    Storm Glass

       ◆
      ◇◆◇
     ◇◆◆◆◇
      ◇◆◇
       ◆

   Crystallized storm
   energy. Shimmers
   with inner light.
```

**Files to Create:**

- `data/ascii_art/items/` (new directory)
- `data/ascii_art/npcs/` (new directory)

**Files to Modify:**

- `src/ui/inspection.rs` - Display ASCII art
- Add art files for each item/NPC

---

### Subtask 5.4: Balance Pass (1 day)

**Implementation:**

- Playtest 3-5 runs to completion
- Adjust combat difficulty (enemy HP, damage)
- Adjust resource scarcity (item spawn rates)
- Ensure 30-60 minute play time feels achievable

**Files to Modify:**

- `data/enemies.json` - Balance stats
- `data/items.json` - Balance spawn rates
- `data/storm_config.json` - Balance storm frequency

---

## COMPLETION CHECKLIST

### Glass Storm System

- [ ] Storm timer tracking and display
- [ ] Wall to glass conversion
- [ ] Storm Glass currency drops
- [ ] Storm forecast system
- [ ] Safe zone indicators
- [ ] DES tests for all features
- [ ] Documentation updated

### Adaptation System

- [ ] Refraction stat tracking and display
- [ ] 5-8 adaptations defined in data
- [ ] Adaptation unlock logic
- [ ] Visual indicators in UI
- [ ] Special abilities implemented
- [ ] DES tests for all features
- [ ] Documentation updated

### NPC Faction System

- [ ] Faction reputation tracking
- [ ] Faction-specific dialogue
- [ ] Reputation modifiers from actions
- [ ] Faction-gated content (quests/shops)
- [ ] 3 core factions fully implemented
- [ ] DES tests for all features
- [ ] Documentation updated

### World Tile Transition

- [ ] World map UI rendering
- [ ] Lazy tile generation
- [ ] Save/load tile state
- [ ] Storm effects on world map
- [ ] DES tests for all features
- [ ] Documentation updated

### Content Polish

- [ ] 3-5 starter quests created
- [ ] 10-15 enemies distributed by biome
- [ ] ASCII art for 3-5 items
- [ ] ASCII art for 3-5 NPCs
- [ ] Balance pass completed
- [ ] Full playthrough tested

---

## TESTING STRATEGY

### For Each Feature:

1. Write DES scenario BEFORE implementing
2. Run scenario (should fail initially)
3. Implement feature
4. Run scenario until it passes
5. Add edge case tests
6. Run full test suite to ensure no regressions

### Example Test Flow:

```bash
# Before implementation
cargo test storm_timer_basic  # Should fail

# Implement feature
# ...

# After implementation
cargo test storm_timer_basic  # Should pass
cargo test --test des_scenarios  # All tests should pass

# Commit
git add .
git commit -m "feat: implement storm timer tracking

- Add storm_timer to GameState
- Display timer in HUD
- Decrement each turn
- Tests: storm_timer_basic scenario passes"
```

---

## GIT COMMIT GUIDELINES

### Commit Message Format:

```
<type>: <short description>

<detailed description>
<what was changed>
<why it was changed>

Tests: <which tests now pass>
```

### Commit Types:

- `feat`: New feature
- `fix`: Bug fix
- `test`: Adding/updating tests
- `docs`: Documentation only
- `refactor`: Code restructuring
- `data`: JSON data changes

### Example Commits:

```
feat: implement storm timer tracking

- Add storm_timer field to GameState
- Display timer in top HUD
- Decrement timer each turn, trigger storm at 0
- Load config from data/storm_config.json

Tests: storm_timer_basic passes

---

feat: implement wall to glass conversion

- Create apply_storm_to_tile() in storm.rs
- Convert Tile::Wall to GlassWall on storm
- Update FOV to handle transparent glass walls
- Add glass_wall definitions to data/tiles.json

Tests: storm_glass_conversion passes

---

data: add 5 core adaptations

- Define crystalline_vision, glass_skin, light_sensitivity,
  prism_reflex, mirror_skin in data/adaptations.json
- Include thresholds, stat modifiers, special abilities

Tests: adaptation definitions load correctly
```

---

## NOTES

- **Prioritize data-driven**: Every new content type should be JSON-defined
- **Test early, test often**: DES scenarios catch bugs before manual testing
- **Decouple ruthlessly**: Game logic should not import ratatui/crossterm
- **Document as you go**: Update design docs when behavior changes
- **Ask if blocked**: If a task is unclear or seems wrong, ask for clarification

**Good luck, FeatureDeveloper! This will be an amazing vertical slice.**
