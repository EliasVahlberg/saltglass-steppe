# Environmental Props System

## Problem

Every visible element on a map is currently either terrain (floor/wall tiles), an interactable item, an enemy, or an NPC. This makes the world feel gamey — the player learns that anything they see is something they can interact with. Real landscapes have visual noise: dead vegetation, rock formations, animal traces, abandoned campsites. These non-interactable props make the world feel lived-in and natural without adding gameplay clutter.

Settlement decorations (`place_decorations`) already solve this for towns by swapping `dry_soil` tiles to faction-themed floor IDs at 8% density. This document proposes generalizing that pattern to all map types.

## Current Systems Analysis

### Tile System

`Tile::Floor { id: String }` is fully data-driven. Floor definitions live in `data/map_elements.json` with glyph, color, name, and description. The renderer looks up the floor ID and draws the corresponding glyph/color. Adding a new visual prop is as simple as adding a new floor entry — no code changes to the renderer or tile enum.

Current floor types (19):
- **Terrain**: `dry_soil`, `soft_sand`, `salt_crust`, `glass_sand`, `brine_mud`, `crystal_moss`, `void_stone`, `storm_glass_shards`, `light_pool`, `salt_gravel`
- **Constructed**: `wood_floor`, `stone_floor`, `brick_floor`, `tile_floor`, `dirt_path`
- **Decorative** (settlement only): `ancient_tile`, `crushed_saltglass`, `prismatic_tiles`
- **Utility**: `smooth_granite_glass`

### Settlement Decorations

`place_decorations()` in `settlement/mod.rs` iterates over the settlement area, and for each `dry_soil` tile rolls an 8% chance to replace it with a faction-themed floor ID. This is the exact pattern we want to generalize.

### Microstructures

`data/microstructures.json` defines 6 multi-tile structures (scavenger_camp, abandoned_outpost, etc.) that spawn NPCs, items, and chests. These are interactable content, not visual props. The environmental props system is complementary — it fills the space between microstructures with visual detail.

### Terrain Generation Pipeline

For non-settlement maps, the pipeline is:
1. `TerrainForgeGenerator` → base terrain (biome-specific floors/walls/glass)
2. `place_microstructures` → interactable structures with NPCs/items
3. `ensure_connectivity` → carve tunnels between isolated regions

Environmental props would slot in after step 1, before microstructures.

## Proposed Design

### Core Concept

Environmental props are floor tile replacements — same as settlement decorations. A new function `place_environmental_props(map, biome, terrain, rng)` runs after terrain generation on all maps. It reads prop definitions from `data/environmental_props.json`, filters by biome/terrain, and replaces eligible floor tiles at configured densities.

### Why Floor Tiles (Not Entities)

- Zero runtime cost — no entity tracking, no collision checks, no save/load overhead
- Already rendered by the existing tile renderer with no changes
- Player can `look` at them and see name/description via existing `Tile::name()`/`Tile::description()`
- Deterministic — same seed produces same prop placement
- Settlement decorations already prove this pattern works

### Data Schema: `data/environmental_props.json`

```json
{
  "schema": "environmental_props_v1",
  "props": [
    {
      "id": "dead_scrub",
      "biomes": ["desert", "saltflat", "scrubland"],
      "terrains": null,
      "place_on": ["dry_soil", "soft_sand"],
      "density": 0.04,
      "cluster_chance": 0.3,
      "cluster_size": [2, 5],
      "min_distance_from_walls": 1,
      "tags": ["foliage"]
    },
    {
      "id": "cairn",
      "biomes": null,
      "terrains": ["hills", "mesa", "flat"],
      "place_on": ["dry_soil", "soft_sand", "salt_crust"],
      "density": 0.002,
      "cluster_chance": 0.0,
      "tags": ["human_trace"]
    },
    {
      "id": "campfire_remains",
      "biomes": null,
      "terrains": null,
      "place_on": ["dry_soil", "soft_sand"],
      "density": 0.001,
      "cluster_chance": 0.0,
      "tags": ["human_trace"]
    }
  ]
}
```

Field definitions:
- `id` — matches a floor definition in `map_elements.json`
- `biomes` — list of biome IDs where this prop appears, or `null` for all biomes
- `terrains` — list of terrain types (canyon, mesa, hills, dunes, flat), or `null` for all
- `place_on` — which existing floor IDs this prop can replace
- `density` — probability per eligible tile (0.0–1.0)
- `cluster_chance` — probability that placing one prop triggers a cluster of adjacent props
- `cluster_size` — `[min, max]` range for cluster size (optional, defaults to `[1, 1]`)
- `min_distance_from_walls` — minimum Manhattan distance from wall tiles (optional, default 0)
- `tags` — categorical tags for filtering/debugging

### New Floor Definitions in `map_elements.json`

Props need corresponding floor entries. Proposed initial set:

**Foliage**
| ID | Glyph | Color | Description |
|---|---|---|---|
| `dead_scrub` | `τ` | DarkYellow | Dried-out scrub brush, brittle and sun-bleached |
| `salt_lichen` | `~` | White | Pale lichen clinging to the salt crust |
| `crystal_sprout` | `↑` | LightCyan | A tiny crystal formation pushing through the ground |
| `dried_moss` | `"` | DarkGreen | Desiccated moss, long dead |
| `thorn_bush` | `♣` | DarkYellow | A thorny bush, too dry to be useful |

**Geological**
| ID | Glyph | Color | Description |
|---|---|---|---|
| `cairn` | `▲` | LightGray | A deliberate stack of stones, marking something |
| `loose_rocks` | `∴` | Gray | Scattered loose stones |
| `salt_deposit` | `�ite` | White | A small mound of crystallized salt |
| `sand_ripple` | `≈` | Yellow | Wind-sculpted ripples in the sand |
| `glass_fragment` | `·` | Cyan | A single shard of storm glass, too small to collect |

**Human/Animal Traces**
| ID | Glyph | Color | Description |
|---|---|---|---|
| `campfire_remains` | `○` | DarkRed | Cold ashes and charred stones — someone camped here |
| `bone_scatter` | `~` | White | Bleached bones, picked clean by wind and sun |
| `old_tracks` | `≡` | DarkYellow | Faded footprints in the hardened ground |
| `torn_cloth` | `≋` | DarkGray | A scrap of fabric, sun-faded and fraying |
| `carved_mark` | `×` | LightGray | A symbol scratched into the ground — a warning? |

**Biome-Specific**
| ID | Glyph | Color | Description |
|---|---|---|---|
| `brine_puddle` | `○` | DarkCyan | A shallow pool of concentrated brine |
| `glass_bloom` | `✦` | LightMagenta | A cluster of tiny glass crystals, almost beautiful |
| `storm_scar` | `╳` | DarkGray | A gouge in the earth left by a glass storm |
| `void_stain` | `▪` | DarkMagenta | A dark discoloration — something unnatural happened here |
| `mirror_shard` | `◇` | LightCyan | A flat piece of natural mirror, reflecting the sky |

## Implementation Plan

### Step 1: Add floor definitions to `map_elements.json`

Add the new floor entries. No code changes needed — the data loader picks them up automatically.

### Step 2: Create `data/environmental_props.json`

Define which props appear where, at what density. Start with ~15-20 props covering the main biomes.

### Step 3: Implement `place_environmental_props()`

New function in `src/game/generation/` (either a new `environmental_props.rs` module or added to an existing generation module). Logic:

```
fn place_environmental_props(map, biome, terrain, rng):
    load props from environmental_props.json
    filter props by biome and terrain
    for each tile in map:
        if tile is Floor { id } and id is in any prop's place_on list:
            for each eligible prop (shuffled):
                if rng.gen_bool(prop.density):
                    if wall distance check passes:
                        replace tile with prop floor ID
                        if rng.gen_bool(prop.cluster_chance):
                            place cluster of same prop on adjacent eligible tiles
                        break  // only one prop per tile
```

### Step 4: Wire into tile generation pipeline

In `tile_generator.rs`, call `place_environmental_props()` after terrain generation but before microstructures:

```
1. TerrainForgeGenerator → base terrain
2. place_environmental_props() ← NEW
3. place_microstructures → interactable structures
4. [settlement pipeline if POI::Town]
5. ensure_connectivity
```

For settlement maps, environmental props run on the base terrain before the settlement is stamped. The settlement clearing/stamping will overwrite props inside the town area, which is correct — towns have their own decoration system.

### Step 5: Skip props near player spawn

Add a minimum distance from spawn point (e.g., 5 tiles) so the player's immediate area isn't cluttered on first load.

## Design Considerations

### Density Tuning

Start conservative. The settlement decoration system uses 8% density which works because it's a small area. For full 250×110 maps, even 1-2% density means hundreds of props. Recommended starting densities:
- Foliage: 2-4% (common, fills space)
- Geological: 0.5-1% (noticeable but not overwhelming)
- Human traces: 0.1-0.2% (rare, makes discovery feel meaningful)

### Clustering

Some props look better in groups (dead scrub, loose rocks, salt lichen). The `cluster_chance` + `cluster_size` fields handle this — when a prop is placed, there's a chance it spawns 2-5 more of the same type on adjacent eligible tiles. This creates natural-looking patches.

### Biome Identity

Each biome should have a distinct visual signature from its props:
- **Saltflat**: salt_lichen, salt_deposit, brine_puddle, glass_fragment
- **Desert**: dead_scrub, sand_ripple, bone_scatter, old_tracks
- **Oasis**: dried_moss, crystal_sprout, thorn_bush
- **Ruins**: carved_mark, torn_cloth, campfire_remains, loose_rocks
- **Glass Gardens**: glass_bloom, crystal_sprout, mirror_shard
- **Storm Scars**: storm_scar, glass_fragment, void_stain

Universal props (cairn, campfire_remains, old_tracks) appear in all biomes at low density to suggest human presence.

### Interaction with Other Systems

- **FOV**: Props are floor tiles, so they're transparent and walkable — no impact on FOV or pathfinding
- **Combat**: No impact — props don't block movement or line of sight
- **Save/Load**: No impact — props are part of the map tile array, already serialized
- **Storms**: Glass storms can overwrite prop tiles (converting them to Glass), which is thematically correct
- **Connectivity**: `ensure_connectivity` carves `dry_soil` tunnels, which may overwrite some props — acceptable

### Not In Scope

- Props that block movement (use walls or microstructures for that)
- Props that the player can interact with (use items, chests, or interactables)
- Animated props (would require renderer changes — future work)
- Props that affect gameplay (damage, status effects — use existing tile types like Glass/Glare)

## File Changes Summary

| File | Change |
|---|---|
| `data/map_elements.json` | Add ~20 new floor definitions |
| `data/environmental_props.json` | New file — prop placement rules |
| `src/game/generation/environmental_props.rs` | New module — `place_environmental_props()` |
| `src/game/generation/mod.rs` | Export new module |
| `src/game/generation/tile_generator.rs` | Call `place_environmental_props()` in pipeline |

## Testing

- Visual: Use tile test presets (`data/tile_tests/*.json`) across all biomes to verify prop density and distribution
- Deterministic: Same seed must produce identical prop placement
- Unit test: Verify prop placement respects biome/terrain filters and density bounds
- Regression: Ensure existing microstructures and settlement decorations still work correctly
