# NPCs JSON Schema (v1)

Schema file: `schemas/npcs_v1.json`  
Data files: `data/npcs.json`, `data/traders.json`

## Overview
This schema validates NPC definitions and trader tables. Each file must include the `schema` field with the value `npcs_v1`.

## Top-Level Structure
- `schema`: Must be `"npcs_v1"`.
- `npcs`: Array of NPC definitions (used in `data/npcs.json`).
- `traders`: Array of trader definitions (used in `data/traders.json`).

## Required NPC Fields
- `id`: Snake_case identifier.
- `name`: Display name.
- `glyph`: Single character for rendering.
- `faction`: Faction identifier.
- `dialogue`: Array of dialogue entries.

## NPC Dialogue
- `dialogue` entries include `text` and optional `conditions`.
- `conditions` support keys like `has_adaptation`, `adaptation_count_gte`, `has_item`, `min_salt_scrip`, `min_reputation`.
- Additional condition keys are allowed for custom checks.

## Trader Fields
- `trader_id`: Snake_case identifier.
- `name`: Display name.
- `faction`: Faction identifier.
- `base_tier`: Base tier for inventory.
- `items`: Array of trade items.
- `reputation_modifiers`: Map of reputation threshold to modifiers.

## Example (NPC)
```json
{
  "schema": "npcs_v1",
  "npcs": [
    {
      "id": "mirror_monk",
      "name": "Mirror Monk",
      "glyph": "M",
      "faction": "MirrorMonks",
      "description": "Robed figure whose skin shimmers with embedded glass",
      "dialogue": [
        { "conditions": [], "text": "You walk unmarked through the Saltglass Steppe." }
      ],
      "actions": []
    }
  ]
}
```

## Example (Trader)
```json
{
  "schema": "npcs_v1",
  "traders": [
    {
      "trader_id": "merchant_keth",
      "name": "Merchant Keth",
      "faction": "saltborn_collective",
      "base_tier": 1,
      "items": [
        { "item_id": "hand_torch", "base_price": 15, "stock": 5 }
      ],
      "reputation_modifiers": {
        "50": { "price_multiplier": 0.8, "stock_bonus": 3, "exclusive_items": [] }
      }
    }
  ]
}
```

## Common Mistakes
- Missing the top-level `schema` field.
- Using non-snake_case IDs.
- Multi-character `glyph` values.
