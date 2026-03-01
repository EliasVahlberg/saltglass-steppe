# Settlement Generation — Implementation Record

**Completed**: 2026-03-01  
**Commits**: `3605b07`, `99ae1e8`, `02f20ae`, `f516428`, `200609e`, `76040bb`

> This document reflects the **actual implementation**. The original planning doc was superseded during development. For future work see `SETTLEMENT_FUTURE_WORK.md`. For user-facing docs see `docs/features/SETTLEMENT_GENERATION.md`. For technical reference see `docs/features/SETTLEMENT_GENERATION_SUMMARY.md`.

---

## What Was Built

### Module Structure

```
src/game/generation/settlement/
├── mod.rs           — SettlementConfig, Settlement, Building, generate_settlement(), stamp_settlement(), place_decorations()
├── layout.rs        — calculate_dimensions(), generate_layout() [grid-with-jitter]
├── buildings.rs     — place_buildings() [StructureLibrary weighted random by faction]
├── faction_theme.rs — get_dominant_faction(), get_significant_factions()
└── population.rs    — calculate_population()
```

### Data Files

| File | Purpose |
|------|---------|
| `data/structures/structures.json` | All building definitions — pattern, legend, metadata (npc_types, entrance_side, npc_count, faction, tags, weight) |
| `data/structures/patterns/` | Pattern files referenced by structures.json |
| `data/npcs.json` | 19 generic settlement NPC roles added (merchant, innkeeper, guard, etc.) |
| `data/map_elements.json` | `dirt_path` floor tile added |
| `data/settlement_config.json` | Tier parameters (sizes, building counts, ratios) |

### Settlement Tiers (actual sizes)

| Tier | Map Size | Max Buildings |
|------|----------|---------------|
| Village | 80×60 | 6 |
| Town | 120×90 | 12 |
| City | 180×120 | 20 |

---

## Key Design Decisions & Divergences from Original Plan

### StructureLibrary replaced prefab system
The original plan used `data/prefabs/` with a `PrefabLibrary`. This was replaced by a unified `StructureLibrary` backed by `data/structures/structures.json`. Building metadata (`npc_types`, `entrance_side`, `npc_count`) lives directly in structure metadata — no separate `building_types.json` needed.

### Grid-with-jitter layout (terrain-forge abandoned)
Terrain-forge BSP/Voronoi was implemented but abandoned: all algorithms produce one connected region, not isolated building plots. Grid-with-jitter was used instead — deterministic, simple, produces well-spaced buildings. See `TERRAIN_FORGE_IMPROVEMENT_SUGGESTIONS.md` for the gap logged.

### Faction ID normalization
`factions.json` uses PascalCase IDs (`MirrorMonks`). `structures.json` uses snake_case (`mirror_monks`). A `to_snake_case()` helper in `buildings.rs` normalizes before `by_faction()` lookup.

### Save/load — no changes needed
`WorldState` (including `world.npcs`) is already fully serialized. Settlements regenerate deterministically from `tile_seed`, so no caching required and no `SAVE_VERSION` bump was needed.

### Ground and Path legend types
Two new `LegendEntry` variants added to `structure_library.rs`:
- `Ground` — skips stamping, leaves terrain tile unchanged (outdoor areas within bounding box)
- `Path` — stamps `dirt_path` tile (placeholder for future road generation)

---

## What Was Deferred

See `SETTLEMENT_FUTURE_WORK.md`:
- Furniture & decoration micro-prefab system
- Multi-z-level settlements (basements, upper floors)
- Building interiors (enterable buildings with separate map layer — decided against; inline patterns are sufficient for now)
