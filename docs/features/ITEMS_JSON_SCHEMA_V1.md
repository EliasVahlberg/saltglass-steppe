# Items JSON Schema (v1)

Schema file: `schemas/items_v1.json`  
Data file: `data/items.json`

## Overview
This schema validates item definitions used throughout Saltglass Steppe. Each item entry is an object inside the top-level `items` array and must include a schema version string.

## Top-Level Structure
- `schema`: Must be `"items_v1"`.
- `items`: Array of item definition objects.

## Required Item Fields
- `id`: Snake_case identifier.
- `name`: Display name.
- `glyph`: Single character for rendering.
- `description`: Lore description.

## Common Optional Fields
- `value`: Trade value (integer).
- `weight`: Carry weight (integer).
- `usable`: Whether the item can be used.
- `consumable`: Whether the item is consumed on use.
- `tier`: Power/rarity tier.
- `equip_slot`: Equipment slot name.
- `armor_value`: Armor value when equipped.
- `effects`: Visual/audio effects list.
- `light_source`: Light radius + intensity.
- `hidden_properties`: Fields hidden from the UI.

## Ability/Feature Flags
Many item behaviors are toggled with boolean flags. Examples include:
- `grants_invisibility`
- `reveals_map`
- `reveals_locations`
- `breaks_walls`
- `grants_phasing`
- `channels_storms`

Refer to `schemas/items_v1.json` for the full set of optional fields.

## Example
```json
{
  "schema": "items_v1",
  "items": [
    {
      "id": "brine_vial",
      "name": "Brine Vial",
      "glyph": "!",
      "description": "Concentrated salt water that heals wounds",
      "value": 10,
      "weight": 1,
      "tier": 1,
      "usable": true,
      "heal": 5,
      "effects": [
        { "condition": "on_pickup", "effect": "B(@3 &Blue)" },
        { "condition": "on_use", "effect": "P(@3 &LightBlue)" }
      ]
    }
  ]
}
```

## Common Mistakes
- Missing the top-level `schema` field.
- Using non-snake_case IDs.
- Multi-character `glyph` values.
