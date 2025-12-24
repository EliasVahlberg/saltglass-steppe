# Functionality Requests — New Content Mechanics

**Date:** 2025-12-14  
**Author:** CreativeDirector  
**Purpose:** Document mechanics needed to support newly added content entities.

---

## Priority: High

### 1. Adaptation Suppression System
**Required for:** `veil_tincture`

- Temporary flag `adaptations_hidden: bool` on player state
- Duration: N turns (suggest 10-15)
- While active: NPCs treat player as `adaptation_count: 0`
- Log line: "Your glow dims. The tincture masks your changes."

### 2. Wall Breaking Mechanic
**Required for:** `glass_pick`

- Player action: target adjacent wall tile
- Wall HP system (already in `walls.json`)
- Reduce wall HP on use; destroy when HP reaches 0
- Glass pick has durability (suggest 3 uses)
- Log line: "You strike the wall. Cracks spread through the glass."

### 3. Damage Reflection
**Required for:** `refraction_wraith`

- On hit: reflect percentage of damage back to attacker (suggest 25-50%)
- Visual feedback in log: "Light bends—your attack refracts back!"
- Consider: reflection only works if player has no adaptations (thematic)

### 4. Saint-Key Credential Check
**Required for:** `archive_drone`, `archive_custodian`

- Inventory check for `saint_key` item
- If present: drone is passive, custodian allows queries
- If absent: drone is hostile, custodian refuses interaction
- Log line (hostile): "CREDENTIAL CHECK. STATUS: UNAUTHORIZED."

---

## Priority: Medium

### 5. Storm-Spawned Enemies
**Required for:** `refraction_wraith`

- Flag `spawns_during_storm: true` in enemy data
- During storm event: spawn 1-2 wraiths at random glass tiles
- Despawn after storm ends (or persist—design decision)
- Log line: "A wraith coalesces from the storm's edge."

### 6. Swarm Behavior
**Required for:** `shard_spider`

- Multiple spiders act as group
- When one attacks, others in range also attack
- Spawn in clusters (3-5)
- Low individual HP, dangerous in numbers

### 7. Refraction Increase on Hit
**Required for:** `shard_spider`

- Field `increases_refraction: N` on enemy
- On successful hit: player gains N refraction
- Log line: "Glass shards pierce you. (+2 Refraction)"

### 8. Flee Behavior for Adapted Players
**Required for:** `dust_wraith`

- Check player `adaptation_count`
- If >= 2: enemy flees instead of attacking
- Log line: "The dust wraith recoils from your glow."

### 9. Adaptation-Based Pricing
**Required for:** `glassborn_merchant`

- Price modifier based on `adaptation_count`
- 0 adaptations: 2x prices
- 1 adaptation: 1.5x prices
- 2+ adaptations: 1x prices (normal)
- Dialogue reflects this

---

## Priority: Low

### 10. Storm Path Revelation
**Required for:** `storm_compass`

- On use: highlight tiles that will be affected by next storm
- Duration: until storm hits
- Visual: overlay `≈` or color shift on affected tiles
- Log line: "The needle trembles. The storm will strike to the west."

### 11. Refraction Reduction
**Required for:** `saints_tear`

- Field `reduces_refraction: N` on item
- On use: reduce player refraction by N
- Cannot go below 0
- Log line: "The tear dissolves. Your glow fades slightly."

### 12. Item Exchange Actions
**Required for:** `dying_pilgrim`

- NPC action that consumes one item, gives another
- `effect: {"gives_item": "X", "consumes": "Y"}`
- Check player has Y before allowing action
- Log line: "The pilgrim presses something into your hand."

### 13. Lore Revelation System
**Required for:** `archive_custodian`

- Action `reveals_lore: true`
- Display lore text (from separate lore data file?)
- Could reveal map locations, enemy weaknesses, or story fragments
- Log line: "DATA RETRIEVED. DISPLAYING RECORD..."

---

## Schema Extensions Needed

### items.json
```json
{
  "suppresses_adaptations": true,
  "breaks_walls": true,
  "reveals_storm_path": true,
  "reduces_refraction": 10
}
```

### enemies.json
```json
{
  "spawns_during_storm": true,
  "reflects_damage": true,
  "swarm": true,
  "increases_refraction": 2,
  "flees_adapted_players": true,
  "requires_saint_key": true,
  "hostile_without_key": true
}
```

### npcs.json (action effects)
```json
{
  "effect": {
    "gives_item": "item_id",
    "consumes": "item_id",
    "reveals_lore": true,
    "price_modifier_by_adaptations": true
  }
}
```

---

## Implementation Notes

- All new fields are optional; existing content unaffected
- Prioritize #1-4 (High) for vertical slice
- #5-9 (Medium) enhance enemy variety significantly
- #10-13 (Low) are polish/depth features

---

*This document should be updated as mechanics are implemented.*
