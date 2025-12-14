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

## Wall Types (`data/walls.json`)

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
| `target` | Yes | What to apply effect to: `player`, `enemy`, `ui` |
| `effect` | Yes | Effect string (see syntax below) |

### Effect Syntax

Effects are encoded as strings with the format `TYPE(parameters)`.

#### Blink: `B(@speed &color)`

Alternates visibility/color at specified frame intervals.

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

Multiple conditions can be combined—all must be true for effect to trigger.

### Example: Adding a New Effect

```json
{
  "id": "critical_hp",
  "condition": {"low_hp": 3},
  "target": "player",
  "effect": "B(@2 &LightRed)"
}
```

This creates a fast red blink when player HP drops to 3 or below.

### Example: Enemy-Specific Effect

```json
{
  "id": "dust_wraith_shimmer",
  "condition": {"enemy_type": "dust_wraith"},
  "target": "enemy",
  "effect": "B(@5 &Yellow)"
}
```

### Current Effects

| ID | Condition | Visual |
|----|-----------|--------|
| `low_hp_warning` | HP ≤ 5 | Red blink |
| `storm_imminent` | Storm ≤ 3 turns | Yellow blink |
| `adaptation_glow` | Has adaptation | Magenta glow |
| `glass_shimmer` | On glass tile | Cyan blink |
| `suppression_active` | Tincture active | Dark gray glow |

---

## Checklist for New Content

- [ ] Add JSON entry with all required fields
- [ ] Write description that fits the setting (salt, glass, storms, mutation)
- [ ] For enemies: add color mapping in `main.rs` if desired
- [ ] For NPCs: write dialogue that reacts to adaptations (Pillar 1)
- [ ] For items: add to spawn list if it should appear naturally
- [ ] Test with `cargo run` and use look mode (`x`) to verify description

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

---

*Last updated: 2025-12-14*
