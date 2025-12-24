# Material Physics & Crafting Mechanics

**Purpose:** To translate the lore of "Glass Types" and "Data Light" into concrete game mechanics for crafting and combat.

---

## 1. The Material Tag System

Every item in the game is composed of materials. These materials determine how the item interacts with Light and Storms.

### Glass Types (The "Lithopedia" Mechanics)

| Material      | Tag             | Effect on Item                                    | Crafting Use              |
| :------------ | :-------------- | :------------------------------------------------ | :------------------------ |
| **Fulgurite** | `mat_fulgurite` | **Shock:** Melee hits deal +2 Electric dmg.       | Energy Weapons, Batteries |
| **Obsidian**  | `mat_obsidian`  | **Stealth:** Reduces enemy detection range by 1.  | Stealth Armor, Cloaks     |
| **Prism**     | `mat_prism`     | **Focus:** Increases range of beam attacks by +2. | Laser Rifles, Scopes      |
| **Verdant**   | `mat_verdant`   | **Regen:** Restores 1 HP per 10 turns.            | Medkits, Living Armor     |
| **Sanguine**  | `mat_sanguine`  | **Bond:** +5 Max HP, but -5 Sanity.               | Blood-Magic Items         |

### Crafting Logic

Crafting is not just "A + B = C". It is **"Base + Lens = Result"**.

- **Base:** The frame (Metal, Wood, Bone).
- **Lens:** The Glass Type (Fulgurite, Obsidian, etc.).
- **Result:** A weapon/armor that inherits the _Base_ stats but the _Lens_ special effect.
  - _Example:_ `Scrap Sword` (Base) + `Fulgurite Shard` (Lens) = `Shock Sword`.
  - _Example:_ `Scrap Sword` (Base) + `Verdant Shard` (Lens) = `Living Blade` (Heals on kill).

---

## 2. The Light Spectrum Mechanics

Light sources in the game are not just binary (Lit/Unlit). They have a **Color/Frequency**.

### The Spectrum

| Color      | Frequency     | Gameplay Effect                                       | Lore Explanation             |
| :--------- | :------------ | :---------------------------------------------------- | :--------------------------- |
| **White**  | Full Spectrum | Standard visibility. No buffs/debuffs.                | "Raw Data"                   |
| **Red**    | Low Freq      | **Damage:** Deals 1 HP/turn to entities.              | "Delete/Burn Command"        |
| **Blue**   | High Freq     | **Energy:** Recharges batteries/abilities.            | "Write/Power Command"        |
| **Green**  | Bio Freq      | **Growth:** Spawns cover/plants. Heals Verdant items. | "Execute Biological Routine" |
| **Purple** | Null Freq     | **Void:** Silences abilities. No Refraction possible. | "System Error / Null"        |

### Light Source Interaction

- **Lanterns:** Players can equip lanterns with different glass lenses to change their light color.
  - _Red Lantern:_ Use as a weapon to burn enemies in a radius.
  - _Blue Lantern:_ Use to power up ancient machinery.
- **Environmental Light:**
  - _Red Sun (Sunset):_ The whole map deals minor damage (The "Purge").
  - _Blue Moon:_ Abilities recharge faster.

---

## 3. The "Hard Light" Physics

**Concept:** High-intensity light becomes solid.

- **Hard Light Bridges:** Beams of light that act as walkable tiles.
  - _Mechanic:_ If a beam source is blocked, the bridge disappears, dropping anyone on it.
- **Hard Light Shields:** Temporary walls.
  - _Mechanic:_ Can be shattered by "Sonic" damage (Resonators) or "Obsidian" weapons (which absorb the light).

---

## 4. The "Void" Mechanics (Deletion)

**Concept:** Darkness deletes.

- **The Decay Timer:** Items dropped in unlit tiles (Darkness) gain a "Decay" counter.
  - After 100 turns in darkness, the item is deleted (`File Not Found`).
- **Shadow Zones:** Specific tiles that are "Null Zones".
  - Entering them drains "Refraction" (Sanity/Mana).
  - If Refraction hits 0, the player takes HP damage (Body deletion).
