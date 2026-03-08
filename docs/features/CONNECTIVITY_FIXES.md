# Connectivity Fixes

## Problems Found

1. **`ensure_connectivity()` never called** — `connectivity.rs` (Glass Seam Bridging) exists but is never invoked. Only terrain-forge's own `glass_seam` op runs, which is insufficient.
2. **Buildings surrounded by walls** — `stamp_settlement()` places prefabs directly onto terrain without clearing a floor footprint around each building first.
3. **No roads between buildings** — No path carving between building entrances or across the settlement.
4. **No entrance to the settlement** — No path carved from the map edge (player spawn area) into the settlement.
5. **Stale `walkable_positions`** — Collected before `stamp_settlement()` runs, so town NPCs are placed using pre-stamp walkable data.

## Fix Order (most logical)

1. **Clear terrain footprint before stamping buildings** (`settlement/mod.rs` `stamp_settlement`)  
   Foundation — buildings must sit on floor, not wall. Everything else depends on this.

2. **Carve roads between buildings** (`settlement/mod.rs` — new `carve_roads()`)  
   Once buildings have clear footprints, connect them with dirt paths.

3. **Carve entrance path from spawn to settlement** (`tile_generator.rs` — after `stamp_settlement`)  
   Connect the player's spawn point to the nearest settlement road/floor tile.

4. **Refresh `walkable_positions` after stamping** (`tile_generator.rs`)  
   NPC placement must use post-stamp walkable data.

5. **Call `ensure_connectivity()` after all generation** (`tile_generator.rs` — final step)  
   GSB pass to catch any remaining disconnected regions across the whole map.
