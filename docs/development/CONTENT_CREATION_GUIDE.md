# Content Creation Guide

This guide explains how to add new items, enemies, and NPCs to Saltglass Steppe. All content is data-driven via JSON files in the `data/` directory—no code changes required for basic additions.

---

## Items (`data/items.json`)

### Schema

```json
{
  "id": "unique_snake_case_id",
  "name": "Display Name",
  "glyph": "!",
  "description": "Short flavor text shown in look mode",
  "value": 10,
  "weight": 1,
  "usable": true,
  "heal": 5,
  "reveals_map": false
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier (snake_case) |
| `name` | Yes | Display name |
| `glyph` | Yes | Single character shown on map |
| `description` | Yes | Flavor text for look mode |
| `value` | Yes | Trade value |
| `weight` | Yes | Inventory weight |
| `usable` | Yes | Can player use this item? |
| `heal` | No | HP restored when used |
| `reveals_map` | No | Reveals entire map when used |

### Example: Adding a New Healing Item

```json
{
  "id": "salt_poultice",
  "name": "Salt Poultice",
  "glyph": "+",
  "description": "Crystallized salve that seals wounds",
  "value": 15,
  "weight": 1,
  "usable": true,
  "heal": 8
}
```

### Spawning Items

Edit `src/game/state.rs` in `GameState::new()`:
```rust
let spawn_items = ["storm_glass", "brine_vial", "salt_poultice", ...];
```

---

## Enemies (`data/enemies.json`)

### Schema

```json
{
  "id": "unique_snake_case_id",
  "name": "Display Name",
  "glyph": "e",
  "max_hp": 10,
  "damage_min": 1,
  "damage_max": 3,
  "sight_range": 6,
  "description": "Flavor text for look mode"
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier |
| `name` | Yes | Display name |
| `glyph` | Yes | Single lowercase character |
| `max_hp` | Yes | Starting health |
| `damage_min` | Yes | Minimum attack damage |
| `damage_max` | Yes | Maximum attack damage |
| `sight_range` | Yes | Tiles before enemy notices player |
| `description` | Yes | Flavor text |

### Example: Adding a New Enemy

```json
{
  "id": "dust_wraith",
  "name": "Dust Wraith",
  "glyph": "w",
  "max_hp": 6,
  "damage_min": 2,
  "damage_max": 5,
  "sight_range": 8,
  "description": "Swirling sand given malevolent form"
}
```

### Enemy Colors (requires code)

Edit `src/main.rs` in the render function:
```rust
let color = match e.id.as_str() {
    "dust_wraith" => Color::LightYellow,
    // ...
};
```

---

## NPCs (`data/npcs.json`)

### Schema

```json
{
  "id": "unique_snake_case_id",
  "name": "Display Name",
  "glyph": "N",
  "faction": "FactionName",
  "description": "Flavor text for look mode",
  "dialogue": [
    {
      "conditions": [{"has_adaptation": "Prismhide"}],
      "text": "Conditional dialogue line"
    },
    {
      "conditions": [],
      "text": "Default fallback dialogue"
    }
  ],
  "actions": [
    {
      "id": "action_id",
      "name": "Action Name",
      "conditions": [],
      "effect": {"heal": 5}
    }
  ]
}
```

### Dialogue Conditions

| Condition | Example | Description |
|-----------|---------|-------------|
| `has_adaptation` | `{"has_adaptation": "Prismhide"}` | Player has specific adaptation |
| `adaptation_count_gte` | `{"adaptation_count_gte": 2}` | Player has N+ adaptations |

Dialogue is evaluated **in order**—first matching entry wins. Always end with an empty `conditions: []` fallback.

### Action Effects

| Effect | Example | Description |
|--------|---------|-------------|
| `heal` | `{"heal": 10}` | Restore HP |
| `trade` | `{"trade": true}` | Open trade (placeholder) |

### Example: Adding a New NPC

```json
{
  "id": "salt_hermit",
  "name": "Salt Hermit",
  "glyph": "H",
  "faction": "Unaffiliated",
  "description": "Weathered figure caked in crystalline deposits",
  "dialogue": [
    {
      "conditions": [{"adaptation_count_gte": 3}],
      "text": "You're more salt than flesh now. Good. The steppe accepts you."
    },
    {
      "conditions": [{"has_adaptation": "Saltblood"}],
      "text": "Saltblood... you'll survive. Maybe."
    },
    {
      "conditions": [],
      "text": "Another soft one. The storms will harden you or bury you."
    }
  ],
  "actions": []
}
```

### Spawning NPCs (requires code)

Edit `src/game/state.rs` in `GameState::new()`:
```rust
if rooms.len() > 4 {
    let room = rooms[rooms.len() - 3];
    npcs.push(Npc::new(room.0, room.1, "salt_hermit"));
}
```

---

## Trading System (`data/traders.json`)

The trading system provides faction-based, tier-dependent commerce with reputation modifiers.

### Trader Schema

```json
{
  "trader_id": "unique_trader_id",
  "name": "Display Name",
  "faction": "faction_name",
  "base_tier": 1,
  "items": [
    {
      "item_id": "item_from_items_json",
      "base_price": 50,
      "stock": 5,
      "min_tier": 1,
      "max_tier": 3,
      "required_reputation": 0,
      "faction_exclusive": "faction_name"
    }
  ],
  "reputation_modifiers": {
    "50": {
      "price_multiplier": 0.8,
      "stock_bonus": 3,
      "exclusive_items": ["rare_item_id"]
    }
  }
}
```

### Trading Fields

| Field | Required | Description |
|-------|----------|-------------|
| `trader_id` | Yes | Unique identifier |
| `name` | Yes | Display name |
| `faction` | Yes | Trader's faction |
| `base_tier` | Yes | Minimum area tier for trader |
| `items` | Yes | Array of trade items |
| `reputation_modifiers` | No | Reputation-based bonuses |

### Trade Item Fields

| Field | Required | Description |
|-------|----------|-------------|
| `item_id` | Yes | Must exist in items.json |
| `base_price` | Yes | Base cost in salt scrip |
| `stock` | Yes | Available quantity (-1 = infinite) |
| `min_tier` | No | Minimum area tier (default: 1) |
| `max_tier` | No | Maximum area tier |
| `required_reputation` | No | Minimum faction reputation (default: 0) |
| `faction_exclusive` | No | Only available to faction members |

### Area Tiers

Tiers are calculated from enemy HP averages in the current area:
- **Tier 1**: 0-20 HP enemies (starting areas)
- **Tier 2**: 21-40 HP enemies 
- **Tier 3**: 41-60 HP enemies
- **Tier 4**: 61-80 HP enemies
- **Tier 5**: 81+ HP enemies (endgame areas)

### Reputation Effects

Reputation modifiers apply at specific thresholds:

| Reputation | Price Modifier | Sell Price | Access |
|------------|----------------|------------|---------|
| -50 to -26 | 1.5x | 30% | Hostile |
| -25 to -1 | 1.2x | 50% | Unfriendly |
| 0 to 24 | 1.0x | 70% | Neutral |
| 25 to 49 | 0.9x | 80% | Friendly |
| 50 to 74 | 0.8x | 90% | Allied |
| 75+ | 0.7x | 100% | Revered |

### Example: Adding a New Trader

```json
{
  "trader_id": "storm_scavenger",
  "name": "Scavenger Nix",
  "faction": "independent",
  "base_tier": 2,
  "items": [
    {
      "item_id": "storm_glass",
      "base_price": 80,
      "stock": 2,
      "min_tier": 2,
      "required_reputation": 10
    },
    {
      "item_id": "scrap_metal",
      "base_price": 15,
      "stock": -1,
      "min_tier": 1
    }
  ],
  "reputation_modifiers": {
    "25": {
      "price_multiplier": 0.9,
      "stock_bonus": 1,
      "exclusive_items": ["rare_storm_relic"]
    }
  }
}
```

---

## Dialogue System (`data/dialogues.json`)

The dialogue system provides branching conversations with conditions and actions.

### Dialogue Tree Schema

```json
{
  "npc_id": "npc_from_npcs_json",
  "name": "NPC Display Name",
  "faction": "faction_name",
  "root_node": "greeting",
  "nodes": [
    {
      "id": "greeting",
      "speaker": "NPC Name",
      "text": "Hello, traveler!",
      "options": [
        {
          "text": "Player response option",
          "condition": {
            "has_currency": 50,
            "faction_reputation": {"faction_name": 25}
          },
          "response": "NPC response text",
          "action": {
            "type": "trade",
            "parameters": {"trader_id": "trader_id"}
          },
          "leads_to": "next_node_id",
          "ends_conversation": false
        }
      ]
    }
  ]
}
```

### Dialogue Conditions

| Condition | Example | Description |
|-----------|---------|-------------|
| `has_currency` | `{"has_currency": 100}` | Player has minimum salt scrip |
| `faction_reputation` | `{"faction_reputation": {"guild": 25}}` | Minimum faction reputation |
| `has_item` | `{"has_item": "storm_glass"}` | Player has specific item |
| `player_level` | `{"player_level": 3}` | Minimum player level |
| `completed_quest` | `{"completed_quest": "quest_id"}` | Quest is completed |
| `has_adaptation` | `{"has_adaptation": "glass_sight"}` | Player has adaptation |
| `area_tier` | `{"area_tier": 3}` | Current area tier |

### Dialogue Actions

| Action Type | Parameters | Description |
|-------------|------------|-------------|
| `trade` | `{"trader_id": "merchant_id"}` | Open trade interface |
| `reputation_change` | `{"faction": "guild", "change": 10}` | Modify reputation |
| `give_item` | `{"item_id": "storm_glass"}` | Give item to player |
| `take_item` | `{"item_id": "salt_crystal"}` | Take item from player |
| `give_currency` | `{"amount": 50}` | Give salt scrip |
| `take_currency` | `{"amount": 25}` | Take salt scrip |

### Example: Adding New Dialogue

```json
{
  "npc_id": "guild_apprentice",
  "name": "Apprentice Kira",
  "faction": "glasswright_guild",
  "root_node": "greeting",
  "nodes": [
    {
      "id": "greeting",
      "speaker": "Apprentice Kira",
      "text": "The guild masters are always watching. What do you need?",
      "options": [
        {
          "text": "I'd like to trade.",
          "response": "I have some basic supplies available.",
          "action": {
            "type": "trade",
            "parameters": {"trader_id": "guild_apprentice"}
          },
          "ends_conversation": true
        },
        {
          "text": "Tell me about the guild.",
          "condition": {
            "faction_reputation": {"glasswright_guild": 0}
          },
          "leads_to": "guild_info"
        }
      ]
    },
    {
      "id": "guild_info",
      "speaker": "Apprentice Kira",
      "text": "We shape glass into tools and art. The masters say glass holds memory.",
      "options": [
        {
          "text": "Interesting. Thank you.",
          "leads_to": "greeting"
        }
      ]
    }
  ]
}
```

### Schema

```json
{
  "id": "wall_type_id",
  "name": "Display Name",
  "glyph": "#",
  "hp": 10,
  "description": "Flavor text"
}
```

Wall type is chosen randomly per map. HP is stored for future breakable wall mechanics.

---

## Visual Effects (`data/effects.json`)

Visual effects are condition-based animations applied to entities during rendering. Effects are defined in JSON and parsed at runtime—no code changes required.

### Schema

```json
{
  "id": "unique_effect_id",
  "condition": {"low_hp": 5},
  "target": "player",
  "effect": "B(@4 &Red)"
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier |
| `condition` | Yes | Object with condition checks (see below) |
| `target` | Yes | What to apply effect to: `player`, `enemy`, `ui`, `environment`, `tile` |
| `effect` | Yes | Effect string (see syntax below) |

### Effect Syntax

Effects are encoded as strings with the format `TYPE(parameters)`.

#### Blink: `B(@speed &color)`

Simple on/off alternation at specified frame intervals.

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| speed | `@` | Frames per toggle (lower = faster) |
| color | `&` | Color name |

Example: `B(@4 &Red)` — Blink red every 4 frames

#### Glow: `G(&color)`

Constant color overlay.

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| color | `&` | Color name |

Example: `G(&Magenta)` — Constant magenta glow

#### Pulse: `P(@speed &color)`

Rhythmic on/off pattern with longer "on" phases, like a heartbeat.

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| speed | `@` | Frames per pulse cycle |
| color | `&` | Color name |

Example: `P(@8 &LightRed)` — Pulse red every 8 frames

#### Wave: `W(@speed &color)`

Spatial wave effect that propagates across the map based on position.

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| speed | `@` | Frames per wave cycle |
| color | `&` | Color name |

Example: `W(@6 &Yellow)` — Wave effect with yellow color

#### Shimmer: `S(@speed &color1 &color2 ...)`

Cycles through multiple colors based on position, creating a shimmering effect.

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| speed | `@` | Frames per color change |
| colors | `&` | Multiple color names |

Example: `S(@4 &Cyan &LightCyan &White)` — Shimmer between cyan shades

#### Rainbow: `R(@speed &color1 &color2 ...)`

Cycles through colors over time (not position-based).

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| speed | `@` | Frames per color change |
| colors | `&` | Multiple color names |

Example: `R(@8 &Magenta &LightMagenta)` — Rainbow between magenta shades

#### Fade: `F(@speed &color)`

Slow fade in/out effect with longer cycles.

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| speed | `@` | Frames per fade cycle |
| color | `&` | Color name |

Example: `F(@16 &DarkGray)` — Slow fade with dark gray

#### Drift: `D(@speed &color)`

Particle-like drifting effect based on position and time.

| Parameter | Prefix | Description |
|-----------|--------|-------------|
| speed | `@` | Frames per drift cycle |
| color | `&` | Color name |

Example: `D(@12 &DarkGray)` — Drifting particles effect

### Available Colors

`Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`, `DarkGray`, `LightRed`, `LightGreen`, `LightYellow`, `LightBlue`, `LightMagenta`, `LightCyan`

### Condition Types

| Condition | Example | Description |
|-----------|---------|-------------|
| `low_hp` | `{"low_hp": 5}` | Player HP at or below value |
| `storm_near` | `{"storm_near": 3}` | Storm arriving in N or fewer turns |
| `has_adaptation` | `{"has_adaptation": true}` | Player has any adaptation |
| `on_tile` | `{"on_tile": "Glass"}` | Player standing on tile type |
| `adaptations_hidden` | `{"adaptations_hidden": true}` | Veil Tincture active |
| `enemy_type` | `{"enemy_type": "refraction_wraith"}` | For enemy-targeted effects |
| `adaptation_count_gte` | `{"adaptation_count_gte": 3}` | Player has N or more adaptations |
| `in_storm_eye` | `{"in_storm_eye": true}` | Storm is currently happening |
| `on_fragile_glass` | `{"on_fragile_glass": true}` | On glass with low HP |
| `psychic_active` | `{"psychic_active": true}` | Psychic boost status active |
| `high_salt_exposure` | `{"high_salt_exposure": true}` | 2+ adaptations (salt exposure) |
| `void_exposure` | `{"void_exposure": true}` | Void-touched status active |

Multiple conditions can be combined—all must be true for effect to trigger.

### Target Types

| Target | Description | Usage |
|--------|-------------|-------|
| `player` | Applied to player character | Character status effects |
| `enemy` | Applied to specific enemy types | Enemy visual identity |
| `ui` | Applied to UI elements | Interface warnings |
| `environment` | Applied to map background | Atmospheric effects |
| `tile` | Applied to specific tiles | Tile-based warnings |

### Example: Adding a New Effect

```json
{
  "id": "critical_hp",
  "condition": {"low_hp": 3},
  "target": "player",
  "effect": "P(@2 &LightRed)"
}
```

This creates a fast red pulse when player HP drops to 3 or below.

### Example: Multi-Color Shimmer Effect

```json
{
  "id": "glass_resonance",
  "condition": {"on_tile": "Glass", "has_adaptation": true},
  "target": "environment",
  "effect": "S(@4 &Cyan &LightCyan &White)"
}
```

Creates a shimmering effect when adapted players stand on glass.

### Example: Environmental Particle Effect

```json
{
  "id": "storm_particle_drift",
  "condition": {"storm_near": 8},
  "target": "environment",
  "effect": "D(@15 &DarkGray)"
}
```

Shows drifting particles when a storm approaches.

### Current Effects

| ID | Condition | Target | Visual | Description |
|----|-----------|--------|--------|-------------|
| `low_hp_warning` | HP ≤ 5 | player | Red blink | Health warning |
| `critical_hp` | HP ≤ 3 | player | Red pulse | Critical health |
| `storm_imminent` | Storm ≤ 3 turns | ui | Red wave | Storm warning |
| `adaptation_glow` | Has adaptation | player | Magenta rainbow | Mutation indicator |
| `glass_shimmer` | On glass tile | player | Cyan shimmer | Glass interaction |
| `suppression_active` | Tincture active | player | Gray fade | Suppression effect |
| `storm_particle_drift` | Storm ≤ 8 turns | environment | Gray drift | Atmospheric particles |
| `glass_resonance` | On glass + adapted | environment | Cyan shimmer | Glass resonance |
| `adaptation_surge` | 3+ adaptations | player | Magenta pulse | High mutation |

### Effect Design Guidelines

**Performance**: Effects are calculated per frame. Use reasonable speeds (4-20 frames) to avoid performance issues.

**Readability**: Ensure effects enhance rather than obscure gameplay information. Avoid overly bright or distracting colors.

**Thematic Consistency**: 
- Glass effects: Cyan/LightCyan/White shimmer or wave patterns
- Storm effects: Yellow/Red wave or drift patterns  
- Adaptation effects: Magenta/LightMagenta rainbow or pulse
- Danger effects: Red blink or pulse
- Suppression effects: DarkGray fade

**Layering**: Multiple effects can apply simultaneously. Design effects that work well together.

**Spatial Effects**: Wave, Shimmer, and Drift effects use position-based calculations for more dynamic visuals.

### Testing Effects

1. Add effect to `effects.json`
2. Run game with `cargo run`
3. Trigger conditions (take damage, stand on glass, etc.)
4. Observe visual effect in game
5. Adjust speed/colors as needed

### Common Effect Patterns

**Status Warnings**: Use Pulse or Blink with red colors
```json
{"effect": "P(@4 &Red)"}
```

**Environmental Atmosphere**: Use Wave or Drift with muted colors
```json
{"effect": "W(@12 &DarkGray)"}
```

**Magical/Mystical**: Use Rainbow or Shimmer with bright colors
```json
{"effect": "R(@8 &Magenta &LightMagenta)"}
```

**Subtle Indicators**: Use Fade or Glow with appropriate colors
```json
{"effect": "F(@20 &LightBlue)"}
```

---

## Entity Effects (in items.json, enemies.json)

Entities can have their own triggered effects that fire on specific events. These are defined directly in the entity's JSON, not in `effects.json`.

### Schema

```json
{
  "id": "entity_id",
  "name": "Entity Name",
  "effects": [
    {"condition": "on_hit", "effect": "B(@2 &Red)"},
    {"condition": "on_pickup", "effect": "G(&Green)"}
  ]
}
```

### Trigger Conditions

| Condition | Applies To | When Triggered |
|-----------|------------|----------------|
| `on_hit` | Enemies | Combat occurs (either direction) |
| `on_pickup` | Items | Player picks up item |
| `on_use` | Items | Player uses item |
| `on_death` | Enemies | Enemy is killed |

### Example: Enemy with Combat Effect

```json
{
  "id": "glass_beetle",
  "name": "Glass Beetle",
  "effects": [
    {"condition": "on_hit", "effect": "B(@3 &Cyan)"}
  ]
}
```

### Example: Item with Pickup and Use Effects

```json
{
  "id": "brine_vial",
  "name": "Brine Vial",
  "effects": [
    {"condition": "on_pickup", "effect": "B(@4 &Green)"},
    {"condition": "on_use", "effect": "G(&LightGreen)"}
  ]
}
```

### Current Enemy Effects

| Enemy | Trigger | Effect |
|-------|---------|--------|
| Mirage Hound | on_hit | Yellow blink @2 |
| Mirage Hound | on_death | Yellow blink @1 |
| Glass Beetle | on_hit | Cyan blink @3 |
| Glass Beetle | on_death | Light cyan blink @1 |
| Salt Mummy | on_hit | White glow |
| Salt Mummy | on_death | Dark gray blink @2 |
| Refraction Wraith | on_hit | White blink @1 |
| Refraction Wraith | on_death | Light cyan blink @1 |
| Shard Spider | on_hit | Light cyan blink @2 |
| Shard Spider | on_death | Cyan blink @1 |
| Dust Wraith | on_hit | Yellow blink @2 |
| Dust Wraith | on_death | Dark gray blink @1 |
| Archive Drone | on_hit | Light blue blink @1 |
| Archive Drone | on_death | Blue blink @1 |

### Current Item Effects

| Item | on_pickup | on_use |
|------|-----------|--------|
| Storm Glass | Cyan blink @3 | — |
| Scripture Shard | Yellow glow | — |
| Brine Vial | Green blink @4 | Green glow |
| Saint-Key | Light blue blink @2 | — |
| Angle-Split Lens | Magenta blink @3 | White blink @1 |
| Salt Poultice | Green blink @4 | Light green glow |
| Veil Tincture | Dark gray blink @4 | Dark gray glow |
| Glass Pick | Cyan blink @4 | — |
| Storm Compass | Yellow blink @3 | Light yellow blink @2 |
| Saint's Tear | White blink @2 | White glow |

---

## Checklist for New Content

### Items
- [ ] Add JSON entry with all required fields
- [ ] Write description that fits the setting (salt, glass, storms, mutation)
- [ ] Add to spawn list if it should appear naturally
- [ ] Consider adding to trader inventories

### Enemies  
- [ ] Add JSON entry with balanced stats for intended tier
- [ ] Add color mapping in `main.rs` if desired
- [ ] Consider HP range for area tier calculation

### NPCs
- [ ] Add JSON entry with faction-appropriate dialogue
- [ ] Write dialogue that reacts to adaptations (Pillar 1)
- [ ] Create corresponding dialogue tree in `dialogues.json`
- [ ] Link to trader if NPC should sell items

### Traders
- [ ] Add trader entry in `traders.json`
- [ ] Set appropriate tier and faction requirements
- [ ] Balance prices and stock levels
- [ ] Add reputation modifiers for faction members

### Dialogues
- [ ] Create branching conversation tree
- [ ] Use conditions to make dialogue reactive
- [ ] Include faction-appropriate voice and concerns
- [ ] Link trade actions to corresponding traders

### Testing
- [ ] Test with `cargo run` and use look mode (`x`) to verify descriptions
- [ ] Test trading interface and dialogue trees
- [ ] Verify reputation and tier requirements work correctly

---

## Creative Director Notes

*Space for Creative Director feedback and suggestions on new content.*

### Tone Reminders
- **Dread/Numinous axis**: Content should evoke awe mixed with unease
- **Transformation is identity**: Adaptations change how NPCs perceive you
- **The steppe is indifferent**: Not malicious, just harsh

### Suggested Content Gaps

<!-- Creative Director: Add suggestions here -->

**Enemies:**
- [ ] _Suggestion: Enemy that reacts differently to adapted players_
- [ ] _Suggestion: Glass-based creature that reflects damage_
- [ ] _Suggestion: Storm-spawned temporary enemy_

**NPCs:**
- [ ] _Suggestion: NPC who fears/shuns heavily adapted players_
- [ ] _Suggestion: Glassborn merchant with unique trade goods_
- [ ] _Suggestion: Dying pilgrim with cryptic lore_

**Items:**
- [ ] _Suggestion: Item that temporarily suppresses adaptations_
- [ ] _Suggestion: Tool for breaking walls_
- [ ] _Suggestion: Storm prediction device_

**Traders:**
- [ ] _Suggestion: Black market trader with illegal adaptations_
- [ ] _Suggestion: Faction-neutral trader in dangerous areas_
- [ ] _Suggestion: Traveling merchant with rotating stock_

**Dialogues:**
- [ ] _Suggestion: Multi-stage quest dialogue trees_
- [ ] _Suggestion: Faction conflict resolution conversations_
- [ ] _Suggestion: Adaptation-specific dialogue branches_

### Faction Voice Guidelines

| Faction | Voice | Avoid |
|---------|-------|-------|
| Mirror Monks | Reverent, cryptic, speaks of "angles" and "refraction" | Generic mysticism |
| Sand-Engineers | Pragmatic, terse, values utility | Flowery language |
| Glassborn | Alien, transformed perspective, speaks of "becoming" | Generic tough-guy |

### Dialogue Quality Checklist

- [ ] Does the NPC react meaningfully to at least one adaptation?
- [ ] Does the default dialogue establish faction identity?
- [ ] Is the voice distinct from other factions?
- [ ] Does it reinforce Pillar 1 (Mutation with Social Consequences)?
- [ ] Are dialogue conditions properly balanced?
- [ ] Do trade actions link to appropriate traders?
- [ ] Are reputation changes reasonable for the interaction?

### Trading Balance Checklist

- [ ] Are prices balanced for the item's utility and rarity?
- [ ] Does the tier requirement match item power level?
- [ ] Are reputation requirements appropriate for faction relations?
- [ ] Do exclusive items provide meaningful faction benefits?
- [ ] Are stock levels reasonable (finite for rare items)?

---

*Last updated: 2025-12-21*
