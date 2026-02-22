# Quests JSON Schema (v1)

Schema file: `schemas/quests_v1.json`  
Data files: `data/quests.json`, `data/main_questline.json`

## Overview
This schema validates quest definitions and the main questline. Each file must include the `schema` field with the value `quests_v1`.

## Top-Level Structure
- `schema`: Must be `"quests_v1"`.
- `quests`: Array of quest definitions (used in `data/quests.json`).
- `main_questline`: Array of quest definitions (used in `data/main_questline.json`).
- `quest_content_injections`: Optional array of content injection rules (main questline file only).

## Required Quest Fields
- `id`: Snake_case identifier.
- `name`: Display name.
- `description`: Lore description.
- `objectives`: Array of objectives.

## Objective Types
- `kill`: Requires `enemy_id`, `count`.
- `collect`: Requires `item_id`, `count`.
- `reach`: Requires `x`, `y`.
- `talk_to`: Requires `npc_id`.
- `interface_with_aria`: Requires `item_required`.
- `interact`: Requires `target`.
- `collect_data`: Requires `data_points`.
- `wait`: Requires `duration`.
- `examine`: Requires `target`.

## Optional Fields
- `reward`: XP, items, salt scrip, reputation rewards, unlocked quests.
- `criteria`: Availability constraints (reputation, items, adaptations, refraction, etc.).
- `category`: Quest category string.
- `act`: Main questline act number.
- `requires_quests_completed`: Legacy shortcut for prerequisites in `data/quests.json`.
- `comedic_moments`, `comedic_resolution`: Flavor entries used in quests.

## Example
```json
{
  "schema": "quests_v1",
  "quests": [
    {
      "id": "first_steps",
      "name": "First Steps",
      "description": "Learn the basics of survival in the Saltglass Steppe.",
      "objectives": [
        {
          "id": "explore",
          "description": "Move 10 steps in any direction",
          "type": "reach",
          "x": 15,
          "y": 15
        }
      ],
      "reward": {
        "xp": 10,
        "salt_scrip": 25,
        "unlocks_quests": ["choose_your_path"]
      }
    }
  ]
}
```

## Common Mistakes
- Missing the top-level `schema` field.
- Omitting required objective fields (e.g., `count` on `collect`).
- Using non-snake_case IDs.
