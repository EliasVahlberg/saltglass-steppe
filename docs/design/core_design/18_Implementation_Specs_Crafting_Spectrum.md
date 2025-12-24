# Implementation Specifications: Crafting & Spectrum

**Purpose:** Technical specs for implementing the systems defined in `17_Material_Physics_and_Crafting.md`.

---

## 1. Material & Crafting System

### Data Structures (`src/game/crafting.rs`)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MaterialType {
    Iron,
    Wood,
    Bone,
    Fulgurite,
    Obsidian,
    Prism,
    Verdant,
    Sanguine,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CraftingRecipe {
    pub id: String,
    pub base_item_id: String, // e.g., "sword_frame"
    pub lens_material: MaterialType,
    pub result_item_id: String, // e.g., "shock_sword"
}

// In ItemDef
pub struct ItemDef {
    // ...
    pub material_tags: Vec<MaterialType>,
    pub crafting_base: bool, // Can be used as a base?
    pub crafting_lens: bool, // Can be used as a lens?
}
```

### Logic
1.  **Recipe Lookup:** When player combines two items, check `recipes.json` (new data file).
2.  **Dynamic Generation (Optional):** If no explicit recipe exists, generate a new item:
    *   Name: `{Material} {BaseName}` (e.g., "Fulgurite Sword")
    *   Stats: Base Stats + Material Modifier.

---

## 2. Light Spectrum System

### Data Structures (`src/game/light.rs`)

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LightColor {
    White,
    Red,
    Blue,
    Green,
    Purple,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LightSource {
    pub radius: u32,
    pub color: LightColor,
    pub intensity: u32,
}

// In Tile/Entity
pub struct LitState {
    pub is_lit: bool,
    pub color: LightColor, // Dominant color
}
```

### The Lighting Algorithm (Update to `src/game/map.rs`)
Current algorithm likely just checks FOV/Distance. Needs to propagate Color.
*   **Mixing:** If Red and Blue overlap -> Purple? Or dominant wins?
    *   *Simplification:* Dominant wins (highest intensity). If equal, White.

### Effect Application (`src/game/systems.rs`)
In `end_turn()`:
```rust
fn apply_light_effects(state: &mut GameState) {
    for entity in state.entities.iter_mut() {
        let tile_light = state.map.get_light_at(entity.pos);
        match tile_light.color {
            LightColor::Red => entity.take_damage(1),
            LightColor::Blue => entity.recharge_energy(1),
            LightColor::Green => {
                if entity.has_tag("verdant") {
                    entity.heal(1);
                }
            },
            _ => {}
        }
    }
}
```

---

## 3. Hard Light & Void Mechanics

### Hard Light Tiles
*   **TileType:** New variant `TileType::HardLightBridge`.
*   **Logic:**
    *   Is Walkable = `true`.
    *   Is Transparent = `true`.
    *   **Dependency:** Needs a `source_entity_id`. If source is destroyed/off, tile reverts to `TileType::Chasm`.

### Void Decay
*   **Component:** `DecayTimer(u32)` on Item Entities.
*   **System:**
    ```rust
    fn process_decay(state: &mut GameState) {
        for item in state.items.iter_mut() {
            if !state.map.is_lit(item.pos) {
                item.decay_timer += 1;
                if item.decay_timer > 100 {
                    item.mark_for_deletion = true;
                }
            } else {
                item.decay_timer = 0; // Reset if lit
            }
        }
    }
    ```
