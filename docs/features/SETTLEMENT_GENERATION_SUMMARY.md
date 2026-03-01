# Settlement Generation Summary

Technical reference for the settlement generation system implemented in `src/game/generation/settlement/`.

## Architecture

Settlement generation follows a three-stage pipeline:

1. **SettlementConfig** defines tier, seed, and faction control percentages
2. **generate_settlement()** produces a Settlement with positioned buildings
3. **stamp_settlement()** applies buildings to the game map and spawns NPCs

## Layout Algorithm

Uses **grid-with-jitter** placement, not terrain-forge BSP. Buildings are positioned on a regular grid with random jitter to avoid perfect alignment:

- Village: 18x14 spacing, 8-tile margin
- Town: 16x12 spacing, 6-tile margin  
- City: 14x10 spacing, 5-tile margin

Jitter range is ±25% of spacing to maintain distribution while adding organic variation.

## Structure Selection

**StructureLibrary** loads from `data/structures/structures.json` and filters buildings by faction. Faction ID normalization converts PascalCase (factions.json) to snake_case (structures.json) via `to_snake_case()`.

Building selection uses weighted random sampling from faction-appropriate structures marked with `usage: "connectable"`. Falls back to generic connectable structures if no faction-specific buildings exist.

Building limits by tier: Village (6), Town (12), City (20).

## NPC Spawning

Each structure's `metadata.npc_types` array specifies which NPC roles to spawn. During `stamp_settlement()`, the system:

1. Reads `npc_types` from StructureMetadata
2. Creates Npc instances using `Npc::new()` with appropriate role IDs
3. Places NPCs at the closest walkable position to the building

NPC roles are defined in `data/npcs.json` with 19 settlement-specific types: merchant, innkeeper, guard, blacksmith, etc.

## State Integration

Called from `state.rs` in `travel_to_tile()` when `poi == POI::Town`. Uses the tile's faction control data and tile seed for deterministic generation. Settlement buildings are stamped before structure-based NPCs to ensure proper layering.

## Known Limitations

Terrain-forge isolated room extraction is not used for settlement layout. The system generates building positions independently rather than extracting rooms from BSP-generated structures. See `TERRAIN_FORGE_IMPROVEMENT_SUGGESTIONS.md` for potential integration approaches.