# Implementation Specifications: World Mechanics

**Purpose:** Concrete technical specifications for implementing the lore concepts defined in `15_World_Mechanics_Deep_Dive.md`.

---

## 1. Refraction System Implementation

**Concept:** Refraction is a stat that unlocks abilities but increases vulnerability.

### Data Structure (`src/game/state.rs`)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefractionState {
    pub current_value: u32,      // 0-100
    pub max_capacity: u32,       // Based on level/items
    pub adaptations: Vec<String>, // IDs of active adaptations
}

// In GameState
pub struct GameState {
    // ...
    pub refraction: RefractionState,
}
```

### Mechanics

1.  **Gain:**
    - Consuming `Storm Glass` (+5 Refraction).
    - Getting hit by `Light` damage (+1 Refraction).
    - Standing in a `Storm` (+1 per turn).
2.  **Thresholds (defined in `data/adaptations.json`):**
    - 25: Unlock "Low Light Vision".
    - 50: Unlock "Glass Skin" (Natural Armor +1).
    - 75: Unlock "Phase Walk" (Active Ability).
    - 100: **Critical Mass** (Game Over or forced transformation event).
3.  **Vulnerability:**
    - `Incoming Light Damage = Base Damage * (1.0 + (Refraction / 100.0))`

---

## 2. Storm Prediction System

**Concept:** Players can see where the storm will hit _before_ it happens.

### The Storm Compass Item

- **Effect:** When equipped, overlays a "Warning" glyph on tiles targeted for the next turn's edit.

### Rendering Logic (`src/ui/game_view.rs`)

```rust
fn render_map(state: &GameState, frame: &mut Frame, area: Rect) {
    // ... existing map render ...

    if state.player.has_equipped("storm_compass") {
        let next_storm_center = state.storm_system.predict_next_strike(state.turn + 1);
        // Render a warning overlay (e.g., flashing red background)
        // on tiles within radius of next_storm_center
    }
}
```

### Deterministic RNG

- Use `state.rng` to seed the storm.
- `predict_next_strike(turn)` must clone the RNG state, step it forward `turn` times, and return the result without mutating the actual game state RNG.

---

## 3. Archive Hacking Minigame

**Concept:** Interacting with "Active Glass" or Terminals requires a TUI minigame based on "tuning" frequencies.

### UI State (`src/ui/hacking_minigame.rs`)

- **Visual:** A waveform display (ASCII chart).
- **Goal:** Match the "Target Wave" with the "Player Wave".
- **Controls:**
  - `Left/Right`: Adjust Frequency (Width of wave).
  - `Up/Down`: Adjust Amplitude (Height of wave).
- **Success:** When waves overlap > 90% for 3 seconds.

### Integration

```rust
// In GameState
pub enum InteractionType {
    None,
    Dialogue(NpcId),
    Hacking(HackingState), // New state
}

// In Input Handler
match state.interaction {
    InteractionType::Hacking(ref mut hack_state) => {
        // Route input to hacking minigame logic
        // If success -> Unlock door / Download data / Disable drone
    }
}
```

---

## 4. Dynamic Economy Logic

**Concept:** Prices change based on who you are trading with.

### Data (`data/factions.json` extension)

```json
{
  "id": "monks",
  "trade_modifiers": {
    "tag_scripture": 2.0,
    "tag_tech": 0.5,
    "tag_glass": 1.5
  }
}
```

### Logic (`src/game/trading.rs`)

```rust
fn calculate_price(item: &ItemDef, merchant_faction: &FactionId) -> u32 {
    let base_price = item.value;
    let modifier = get_faction_modifier(merchant_faction, &item.tags);
    (base_price as f32 * modifier) as u32
}
```

- **Tags:** Items in `items.json` need a `tags: ["tech", "glass", ...]` field.
