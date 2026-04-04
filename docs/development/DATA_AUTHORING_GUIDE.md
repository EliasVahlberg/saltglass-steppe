---
status: current
last_verified: 2026-04-04
commit: e0d1fe7
---

# Data Authoring Guide

How to add and modify game content in `data/` JSON files. Every data file is validated at load time against its schema in `schemas/`.

## ID Conventions

- **snake_case** for all IDs: `salt_crawler`, `brine_vial`, `merchant_keth`
- IDs must be unique within their file
- Match existing glyph patterns (single ASCII character for entities)
- Keep lore tone consistent: salt, glass, storms, mutations — no modern slang

## Adding an Item

**Primary file**: `data/items.json`

Required fields:
```json
{
  "id": "void_shard",
  "name": "Void Shard",
  "glyph": "◆",
  "description": "A fragment of collapsed reality",
  "value": 75,
  "weight": 1,
  "tier": 3,
  "usable": false
}
```

**Cross-references to update**:

| File | When | Field |
|------|------|-------|
| `data/traders.json` | Item is purchasable | `items[].item_id` |
| `data/loot_tables.json` | Item drops from chests | `entries[].item_id` |
| `data/biome_spawn_tables.json` | Item spawns on ground | `items[].id` |
| `data/recipes.json` | Item is craftable or a material | `output` or `materials` keys |

**Schema**: `schemas/items_v1.json`

## Adding an Enemy

**Primary file**: `data/enemies/{common,uncommon,rare,elite,boss}.json` — pick the rarity tier.

Required fields:
```json
{
  "id": "glass_lurker",
  "name": "Glass Lurker",
  "glyph": "L",
  "max_hp": 12,
  "damage_min": 2,
  "damage_max": 4,
  "sight_range": 5,
  "level": 3,
  "xp_value": 25,
  "description": "Ambush predator that hides in glass formations",
  "faction": "glass_spirits",
  "tags": ["glass", "ambush"],
  "loot_table": [
    { "item": "glass_shard", "weight": 4, "min_count": 1, "max_count": 2 }
  ]
}
```

**Cross-references to update**:

| File | When | Field |
|------|------|-------|
| `data/biome_spawn_tables.json` | Enemy spawns in the world | `enemies[].id` |
| `data/loot_tables.json` | Enemy-specific loot tables | `entries[].item_id` (items it drops must exist) |

Optional: `behavior_id` (defaults to `standard_melee`). Available behaviors: `standard_melee`, `ranged_only`, `healer`, `suicide_bomber`.

**Schema**: `schemas/enemies_v1.json`

## Adding an NPC

**Primary file**: `data/npcs.json`

**Cross-references to update**:

| File | When | Field |
|------|------|-------|
| `data/dialogues.json` | NPC has dialogue | `npc_id` must match |
| `data/traders.json` | NPC is a trader | `trader_id` must match |
| `data/quests.json` | NPC is a quest target | `npc_id` in objectives |

**Schema**: `schemas/npcs_v1.json`

## Adding a Quest

**Primary file**: `data/quests.json` (side quests) or `data/main_questline.json` (main story)

Required fields:
```json
{
  "id": "glass_harvest",
  "name": "Glass Harvest",
  "description": "Collect storm glass for the traders",
  "objectives": [
    { "id": "collect", "description": "Collect 5 storm glass", "type": "collect", "item_id": "storm_glass", "count": 5 }
  ],
  "reward": { "xp": 30, "items": [], "salt_scrip": 100 }
}
```

Objective types: `kill`, `collect`, `reach`, `talk_to`, `examine`, `interact`, `explore`.

**Cross-references to check**:
- `npc_id` in talk_to objectives → must exist in `data/npcs.json`
- `item_id` in collect objectives → must exist in `data/items.json`
- `requires_quests_completed` → referenced quest IDs must exist
- `reward.items` → item IDs must exist in `data/items.json`
- `reward.unlocks_quests` → quest IDs must exist

**Schema**: `schemas/quests_v1.json`

## Adding a Dialogue Tree

**Primary file**: `data/dialogues.json`

Structure: array of dialogue trees, each keyed by `npc_id`. Each tree has a `root_node` and `nodes` array. Nodes can have `options` with `leads_to` (another node ID) or `action` (trade, give_item, etc.).

**Cross-references**:
- `npc_id` → must match an NPC in `data/npcs.json`
- `action.parameters.trader_id` → must match a trader in `data/traders.json`
- Item IDs in give/require actions → must exist in `data/items.json`

**Schema**: `schemas/dialogues_v1.json`

## Adding a Crafting Recipe

**Primary file**: `data/recipes.json`

```json
{
  "id": "refined_lens",
  "name": "Refined Lens",
  "description": "A carefully ground glass lens",
  "materials": { "glass_shard": 2, "salt_crystal": 1 },
  "output": "angle_lens",
  "output_count": 1,
  "skill_required": 2,
  "station_required": null,
  "faction_required": null
}
```

**Cross-references**: All IDs in `materials` and `output` must exist in `data/items.json`.

**Schema**: `schemas/recipes_v1.json`

## Adding a Structure/Prefab

**Primary file**: `data/structures/structures.json`
**Pattern files**: `data/structures/patterns/{core,ruins,special}/`

Structures are tile patterns stamped onto generated maps. Each has metadata (faction, usage, tags) and a pattern (inline or file reference).

**Cross-references**: Legend entries reference tile types from `data/map_elements.json`.

## Cross-Reference Dependency Graph

```
items.json ──────┬──→ traders.json (item_id)
                 ├──→ loot_tables.json (item_id)
                 ├──→ biome_spawn_tables.json (id)
                 ├──→ recipes.json (materials, output)
                 └──→ quests.json (reward items, collect objectives)

enemies/*.json ──┬──→ biome_spawn_tables.json (id)
                 └──→ loot_tables.json (items they drop must exist)

npcs.json ───────┬──→ dialogues.json (npc_id)
                 ├──→ traders.json (trader_id)
                 └──→ quests.json (talk_to objectives)

quests.json ─────┬──→ npcs.json (npc_id in objectives)
                 ├──→ items.json (collect objectives, rewards)
                 └──→ quests.json (requires_quests_completed, unlocks_quests)

structures.json ─┬──→ map_elements.json (tile types in legend)
```

## Validation Checklist

Before committing data changes:

1. `jq . data/<file>.json` — validates JSON syntax
2. `cargo test` — DataLoader validates against schemas at load time
3. Verify all cross-referenced IDs exist in their target files
4. Run relevant DES scenarios: `cargo test --test des_scenarios`
5. If adding spawn table entries: `./test_all_configs.sh`

## Regenerating Schemas

After changing Rust data structs (the `#[derive(Serialize, Deserialize)]` types):

```bash
cargo run --bin schema_gen
```

This regenerates all `schemas/*_v1.json` files from the Rust type definitions.
