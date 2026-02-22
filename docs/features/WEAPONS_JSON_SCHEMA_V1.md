# Weapons JSON Schema (v1)

Schema file: `schemas/weapons_v1.json`  
Data file: `data/weapons.json`

## Overview
This schema validates weapon definitions used by combat and equipment systems. Each weapon entry is an object inside the top-level `weapons` array and must include a schema version string.

## Top-Level Structure
- `schema`: Must be `"weapons_v1"`.
- `weapons`: Array of weapon definition objects.

## Required Weapon Fields
- `id`: Snake_case identifier.
- `name`: Display name.
- `glyph`: Single character for rendering.
- `damage_min`: Minimum damage.
- `damage_max`: Maximum damage.
- `accuracy`: Base hit chance (0-100).
- `range`: Attack range in tiles.

## Optional Weapon Fields
- `ap_cost`: Action point cost (defaults to 2).
- `ammo_type`: Ammo item ID.
- `description`: Lore description.

## Example
```json
{
  "schema": "weapons_v1",
  "weapons": [
    {
      "id": "salt_knife",
      "name": "Salt Knife",
      "glyph": "/",
      "damage_min": 2,
      "damage_max": 5,
      "accuracy": 85,
      "range": 1,
      "ap_cost": 2,
      "description": "Crystallized blade that never dulls"
    }
  ]
}
```

## Common Mistakes
- Missing the top-level `schema` field.
- Using non-snake_case IDs.
- Multi-character `glyph` values.
