# Data-Driven Items and Enemies

**Date:** 2025-12-14

## Summary
Converted hardcoded item and enemy definitions to JSON data files, enabling easy content addition without code changes.

## New Files
```
data/
├── items.json      # Item definitions
└── enemies.json    # Enemy definitions
```

## Item Schema
```json
{
  "id": "string",           // Unique identifier
  "name": "string",         // Display name
  "glyph": "string",        // Single character for map
  "description": "string",  // Flavor text
  "value": 0,               // Trade value (optional)
  "weight": 0,              // Inventory weight (optional)
  "usable": false,          // Can be used from inventory
  "heal": 0,                // HP restored on use (optional)
  "reveals_map": false      // Reveals entire map on use (optional)
}
```

## Enemy Schema
```json
{
  "id": "string",           // Unique identifier
  "name": "string",         // Display name
  "glyph": "string",        // Single character for map
  "max_hp": 10,             // Starting/max health
  "damage_min": 1,          // Minimum attack damage
  "damage_max": 3,          // Maximum attack damage
  "sight_range": 6,         // Detection range (optional)
  "description": "string"   // Flavor text (optional)
}
```

## Adding New Content

### New Item
Add to `data/items.json`:
```json
{
  "id": "salt_crystal",
  "name": "Salt Crystal",
  "glyph": "◇",
  "description": "Pure crystallized salt",
  "value": 5,
  "weight": 1,
  "usable": false
}
```

### New Enemy
Add to `data/enemies.json`:
```json
{
  "id": "prism_wraith",
  "name": "Prism Wraith",
  "glyph": "W",
  "max_hp": 20,
  "damage_min": 3,
  "damage_max": 6,
  "sight_range": 8,
  "description": "Spectral entity that bends light"
}
```

## Technical Details
- JSON embedded at compile time via `include_str!`
- Definitions loaded into static `HashMap` using `once_cell::Lazy`
- Requires rebuild after JSON changes

## Dependencies Added
- `serde_json = "1"`
- `once_cell = "1"`

## Files Modified
- `Cargo.toml`
- `src/game/item.rs` (rewritten)
- `src/game/enemy.rs` (rewritten)
- `src/game/state.rs` (updated for string IDs)
- `src/game/mod.rs` (updated exports)
- `src/main.rs` (updated for new API)
- `src/lib.rs` (updated tests)
