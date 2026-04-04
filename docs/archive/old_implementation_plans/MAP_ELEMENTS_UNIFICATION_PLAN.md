# Unified Map Elements Plan

## Summary
Migration complete: `data/map_elements.json` is now the single source of truth and legacy split files have been removed.

## Current State
- `data/map_elements.json` -> `schemas/map_elements_v1.json`, loaded in `src/game/map_elements.rs`
  - includes tiles and lights in a single file

## Goal
Provide a single source of truth for map-related definitions with clear sections:
- `tiles.walls`
- `tiles.floors`
- `lights.defs`
- `lights.spawn_rules`

This enables:
- Shared ID conventions and validation for tiles
- Centralized schema validation
- Fewer loaders and consistent error handling

## Proposed Schema Shape (Draft)
```json
{
  "schema": "map_elements_v1",
  "tiles": {
    "walls": [ { "id": "...", "name": "...", "glyph": "...", "color": "...", "hp": 10, "description": "..." } ],
    "floors": [ { "id": "...", "name": "...", "glyph": "...", "color": "...", "description": "..." } ]
  },
  "lights": {
    "defs": [ { "id": "...", "name": "...", "glyph": "...", "radius": 6, "intensity": 120, "color": "orange" } ],
    "spawn_rules": { "...": { "lights_per_room": [0, 2], "weights": { "torch": 5 } } }
  }
}
```

## Migration Plan
1. **Schema Introduced**
   - `schemas/map_elements_v1.json` added.
   - Loader reads `data/map_elements.json` only.

2. **Single-Write**
   - Authoring moved to `data/map_elements.json`.

3. **Deprecate Legacy**
   - `walls_v1.json`, `floors_v1.json`, `lights_v1.json` marked deprecated.
   - Legacy split data files removed.

## Loader Considerations
- The unified loader should preserve existing APIs:
  - `get_wall_def(id)`
  - `get_floor_def(id)`
  - `get_light_def(id)`
  - `get_spawn_rule(biome)`
- During migration, implement adapter maps from unified data to the existing in-memory structures.

## Open Questions
- Should `glyph` stay as a `string` or become a stricter `char` pattern?
- Should tile `color` move to a palette enum shared with themes?
- Is `lights.spawn_rules` better housed in a future `generation_config`?

## Non-Goals
- No gameplay changes.
- No rendering changes.
- No changes to map generation algorithms.
