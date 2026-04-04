---
status: current
last_verified: 2026-04-04
commit: e0d1fe7
---

# Settlement Generation Research & Comparison

*Written: 2026-03-14. Based on research into CDDA and Caves of Qud, compared against the current Saltglass Steppe implementation.*

---

## How Our Pipeline Works

The current settlement generation runs in seven sequential passes, each operating only on what the previous pass left open.

### 1. Dimensions (`layout.rs → calculate_dimensions`)
Tier determines the map canvas: Village = 80×60, Town = 120×90, City = 180×120.

### 2. Building positions (`layout.rs → generate_layout`)
A regular grid of candidate positions is laid out with tier-appropriate spacing, then each position receives a small random jitter to break the grid feel.

### 3. Building selection + rotation (`buildings.rs → place_buildings`)
For each grid position, a structure is picked by weighted random from the structure library, filtered to the dominant faction's buildings if one exists. The building is then rotated (0/90/180/270°) so its `entrance_side` faces the settlement centre `(width/2, height/2)`.

Rotation helpers:
- `rotation_to_face(entrance, target)` — CW rotation needed to align entrance with target direction
- `toward(from, to)` — dominant cardinal direction between two points

### 4. Footprint clearing (`mod.rs → clear_settlement_footprint`)
Before stamping, every natural wall within 8 tiles of any building footprint is replaced with `dry_soil`. Distance is measured per-cell to the nearest building rectangle (Euclidean, squared integer comparison), giving an organic rounded polygon rather than a bounding box.

### 5. Building stamping (`mod.rs → stamp_settlement`)
Each building's ASCII pattern is written onto the map tile-by-tile with rotation applied via `rotate_coords(px, py, w, h, rotation)`. Walls → `Tile::Wall`, floors → their specific floor id, doors → `wood_floor`. Effective dimensions swap w/h for 90°/270° rotations.

### 6. Road painting (`mod.rs → paint_roads`)
Entrance points are computed using the rotated entrance side and effective dimensions. Kruskal's MST connects all entrances with minimum total road length; ~n/5 short non-MST edges are added back for loops. Paths are painted by L-shaped walks, overwriting only `dry_soil` tiles — indoor floors are never touched.

### 7. Decorations (`mod.rs → place_decorations`)
Remaining `dry_soil` tiles have an 8% chance of being replaced with a faction-themed decoration (e.g. `prismatic_tiles` for Mirror Monks, `salt_crust` for Salt Traders).

---

## How CDDA Does It

### Two-scale architecture

CDDA separates generation into two distinct scales that Saltglass Steppe currently conflates:

**Overmap level (macro, ~24×24 tile chunks):**
Cities are placed first as a single intersection point with a size value. Roads radiate outward from that centre. `city_building` entries (equivalent to our structures) are placed *along those roads* by frequency weights defined in `region_settings`. The city radius determines which buildings are eligible. **Roads come first; buildings fill in around them.** This is the inverse of our approach.

**Local map level (micro, per-tile):**
Each overmap tile is generated independently on demand using JSON `mapgen` definitions — ASCII patterns with legends, exactly like our prefab system. Buildings support rotation via auto-generated `_north/_east/_south/_west` variants loaded at startup.

### Typed connections (`overmap_connection`)

Roads in CDDA are a first-class typed entity, not painted tiles. An `overmap_connection` defines:
- Which terrain types it can cross
- The cost to cross each type (fields cheap, forests moderate, swamps expensive, rivers → bridge)
- The resulting terrain placed when crossing

This lets the game pathfind roads between points with terrain-aware costs, automatically placing bridges over rivers and preferring open ground. Our roads have no concept of what they're crossing.

### Mutable specials (join system)

For organic multi-tile structures (ant colonies, microlabs, underground complexes), CDDA uses a **mutable special** system:
- Each building chunk declares compatible join edges (north/south/east/west/above/below)
- A root chunk is placed first
- Subsequent phases grow the structure by satisfying open joins, weighted by rules
- Phases run in order; the final phase must cap all remaining open joins

This produces structures that feel organically grown rather than stamped from a fixed template. It's complex to author but very powerful for non-rectangular layouts.

### Key CDDA reference
- Overmap documentation: https://docs.cataclysmdda.org/JSON/OVERMAP.html
- Generation sequence in `overmap.cpp`: `place_cities()` → `place_roads()` → `place_specials()`

---

## How Caves of Qud Does It

### Village generation

CoQ's villages are primarily **narrative-first, layout-second**. Procedurally generated villages:
- Generate a history (two random events, each with a 1-in-20 chance of making the village abandoned)
- Assign fixed NPC roles: mayor, tinker, apothecary, warden, merchant, one quest-giver
- The physical layout is relatively simple — the named static villages (Joppa, Kyakukya, etc.) are hand-crafted; unnamed procedural villages use simpler placement

### Ruins generation (GDC 2019 — multi-pass WFC)

For ruins specifically, CoQ uses a sophisticated multi-pass approach driven by procedurally generated cultures:

1. **Culture pass** — a culture is generated with attributes: architectural style, preferred materials, building philosophy, symbolic motifs
2. **Coarse structure pass** — large-scale room/zone layout derived from cultural attributes (e.g. a militaristic culture produces fortified perimeters; a scholarly culture produces radial library layouts)
3. **WFC (Wave Function Collapse) pass** — fills in tile-level detail by learning adjacency rules from small example bitmaps representing that culture's style. WFC ensures local tile patterns are consistent with the cultural aesthetic without hand-authoring every variant
4. **Connectivity pass** — ensures the dungeon is navigable (all rooms reachable)
5. **Population pass** — places NPCs and items appropriate to the culture and its history

The key insight: **culture drives architecture drives tile detail**, top-down. The physical layout is a consequence of who lived there, not an independent aesthetic choice.

### Key CoQ references
- GDC 2019 talk: "Tile-Based Map Generation using Wave Function Collapse in Caves of Qud" — https://gdcvault.com/play/1026263
- Village wiki: https://wiki.cavesofqud.com/wiki/Village
- World generation wiki: https://wiki.cavesofqud.com/wiki/World_generation

---

## Comparison Table

| Aspect | CDDA | Caves of Qud | Saltglass Steppe |
|---|---|---|---|
| Generation order | Roads first, buildings fill in | Culture → structure → WFC detail | Buildings first, roads after |
| Building rotation | Auto 4-way variants at load time | Culture-driven orientation | Entrance faces settlement centroid |
| Building placement | Frequency-weighted along roads | Static (named) + simple procedural (unnamed) | Jittered grid |
| Roads as typed entity | Yes (`overmap_connection` with terrain costs) | No | No (painted `dry_soil` tiles) |
| Multi-tile structures | Mutable specials (join system) | WFC + culture pass | Single prefabs only |
| Faction/culture influence | Region settings frequency weights | Culture → architectural style → tile patterns | Faction filters building pool |
| Interior generation | Per-tile mapgen on demand | WFC detail pass | Not implemented |
| Connectivity guarantee | Road pathfinding + connection system | Connectivity pass | MST (guarantees spanning tree) |
| Organic layout | Mutable specials grow organically | WFC produces organic tile patterns | Distance-field footprint clearing |

---

## Gaps and Future Directions

### Structural gaps vs both references

**Roads are an afterthought.** Both CDDA and CoQ treat connectivity as a first-class concern that shapes layout. Our roads are generated after buildings as a post-process. This means roads can only connect what already exists rather than influencing where things are placed.

**No terrain awareness in roads.** Our `paint_path` walks an L-shaped route and paints `dry_soil`. It has no concept of obstacles, terrain cost, or preferred routing. CDDA's `overmap_connection` system handles this elegantly.

**No multi-tile structures.** All our buildings are single fixed-size prefabs. CDDA's mutable specials and CoQ's WFC pass both produce structures that feel grown rather than stamped.

### Directions worth exploring

**Roads-first layout (from CDDA):**
Define one or two "main street" axes first, then place buildings along them. More realistic urban feel, and roads naturally connect buildings rather than needing MST post-processing. Particularly relevant for Town and City tiers.

**Typed road connections (from CDDA):**
Promote roads from painted tiles to a typed entity with terrain cost. This would let roads adapt to terrain variation when the settlement sits on non-flat ground, and would make the road system extensible (dirt track vs cobblestone vs salt-brick road).

**Faction-driven spatial arrangement (from CoQ culture pass):**
Faction could drive not just which buildings appear but their spatial arrangement. Examples:
- Storm Cult settlements cluster ritual spaces centrally with residential on the periphery
- Salt Traders line buildings along a single main trade road
- Mirror Monks arrange buildings in a radial pattern around a central light pool

This is a natural extension of the existing faction system and would make settlements feel culturally distinct beyond just building type.

**WFC for interior generation (from CoQ):**
Wave Function Collapse is the natural next step for building interiors once hand-authoring every variant becomes impractical. A small example bitmap per building type (inn, forge, temple) would let WFC generate varied but stylistically consistent interiors. This is tracked in `SETTLEMENT_FUTURE_WORK.md`.

**Mutable specials for landmark structures (from CDDA):**
The join system is overkill for ordinary buildings but well-suited to landmark structures — a sprawling archive complex, a multi-wing temple, a fortified trading post. Defining join edges on prefab chunks would let these grow organically to fill available space.

---

## Related Documents

- `docs/development/SETTLEMENT_GENERATION_PLAN.md` — original implementation plan
- `docs/development/SETTLEMENT_FUTURE_WORK.md` — deferred features (z-levels, interiors)
- `docs/features/SETTLEMENT_GENERATION.md` — user-facing feature description
- `docs/features/SETTLEMENT_GENERATION_SUMMARY.md` — technical summary
- `docs/development/PROCEDURAL_GENERATION_COMPREHENSIVE_GUIDE.md` — broader procgen guide
