# Data File Audit

> Last updated: 2026-03-07

## Summary

41 JSON files in `data/`. 38 are loaded by game code. 3 are not yet wired.

---

## Load Method by File

### Loaded via `include_str!` (compile-time, embedded in binary)

| File | Loaded by |
|------|-----------|
| `abilities.json` | `src/game/skills.rs` (skills + abilities), `src/game/psychic.rs` |
| `actions.json` | `src/game/state.rs` |
| `adaptations.json` | `src/game/adaptation.rs` |
| `auto_explore_config.json` | `src/game/state.rs` |
| `biome_profiles.json` | `src/game/generation/biomes.rs` |
| `biome_spawn_tables.json` | `src/game/generation/spawn.rs` |
| `chests.json` | `src/game/state.rs` |
| `classes.json` | `src/game/state.rs` |
| `constraint_rules.json` | `src/game/generation/constraints.rs` |
| `dialogues.json` | `src/game/dialogue.rs` |
| `dynamic_events.json` | `src/game/generation/events.rs` |
| `effects.json` | `src/game/status.rs`, `src/game/state.rs` |
| `encounter_config.json` | `src/game/encounter.rs` |
| `enemies/common.json` | `src/game/enemy.rs` |
| `enemies/uncommon.json` | `src/game/enemy.rs` |
| `enemies/rare.json` | `src/game/enemy.rs` |
| `enemies/elite.json` | `src/game/enemy.rs` |
| `enemies/boss.json` | `src/game/enemy.rs` |
| `factions.json` | `src/game/faction.rs` |
| `interactables.json` | `src/game/interactable.rs` |
| `items.json` | `src/game/item.rs` |
| `keyboard_config.json` | `src/ui/input.rs` |
| `loot_tables.json` | `src/game/generation/loot.rs` |
| `main_questline.json` | `src/game/quest.rs` |
| `map_features.json` | `src/game/generation/feature_registry.rs` |
| `microstructures.json` | `src/game/generation/microstructures.rs` |
| `narrative_integration.json` | `src/game/generation/narrative.rs` |
| `npcs.json` | `src/game/npc.rs` |
| `progression.json` | `src/game/progression.rs` |
| `quests.json` | `src/game/quest.rs` |
| `recipes.json` | `src/game/crafting.rs` |
| `skill_trees.json` | `src/game/skills.rs` |
| `storm_config.json` | `src/game/systems/storm.rs` |
| `structure_generation.json` | `src/bin/tilegen-tool.rs` only (CLI tool, not main game) |
| `terrain_config.json` | `src/game/generation/terrain_forge_adapter.rs` |
| `traders.json` | `src/game/trading.rs` |
| `travel_config.json` | `src/game/travel.rs` |
| `tutorial.json` | `src/game/state.rs` |
| `weapons.json` | `src/game/item.rs` |

### Loaded via `fs::read_to_string` (runtime, not embedded)

| File | Loaded by | Notes |
|------|-----------|-------|
| `render_config.json` | `src/renderer/mod.rs` | Reloaded on theme change |
| `themes.json` | `src/renderer/mod.rs` | Reloaded on theme change |
| `narrative_templates.json` | `src/game/generation/narrative_templates.rs` | Grammar templates |
| `books.json` | `src/game/book.rs` | Loaded on first book read |
| `map_elements.json` | `src/game/map_elements.rs` | Loaded with schema validation |

### Not yet loaded (planned)

| File | Status | Notes |
|------|--------|-------|
| `settlement_config.json` | Planned — not yet wired | Settlement generation task 13 (building interiors). Defines tier sizes, building ratios, faction building mappings. |
| `skill_trees.schema.json` | Schema only — intentionally not loaded | JSON Schema for validating skill_trees.json. Not game data. |

---

## Findings

### `structure_generation.json` — CLI tool only
Only loaded by `src/bin/tilegen-tool.rs`. Not used by the main game. This is intentional — it's a generation testing config. No action needed, but it should not be treated as game data.

### `biome_profiles.json` vs `terrain_config.json` — no overlap
- `biome_profiles.json`: environmental flavor (feature descriptions, mechanical effects per biome feature type)
- `terrain_config.json`: generation algorithm parameters (noise settings, tile weights, POI configs)

These serve different purposes. Consolidation is not warranted.

### `settlement_config.json` — planned, not dead
Referenced in `SETTLEMENT_GENERATION_PLAN.md` and `ROADMAP.md`. Will be loaded when settlement building interior generation is implemented (settlement task 13).

---

## Recommendations

1. No files to delete — all files are either active, planned, or intentional tool configs.
2. Consider migrating `books.json`, `map_elements.json` to `include_str!` for consistency and to avoid runtime path dependency. Low priority.
3. `structure_generation.json` could be moved to `src/bin/data/` to make its tool-only status explicit. Low priority.
