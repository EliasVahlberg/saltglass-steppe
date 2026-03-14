# Settlement Generation — Implementation Plan

*Derived from `SETTLEMENT_GENERATION_RESEARCH.md` (CDDA / Caves of Qud comparison). Each phase is incremental — the game stays working and all 26 tests pass after every merge.*

---

## terrain-forge Gap Mapping

Before diving into phases, here's how each identified gap maps to terrain-forge 0.7.0:

| Gap | terrain-forge Module | Status | Integration Notes |
|-----|---------------------|--------|-------------------|
| Roads are afterthought | `effects::connectivity` — `connect_markers()`, `connect_regions_spanning()` | Small Addition | Both carve `Tile::Floor` — use returned path coords game-side to paint typed road tiles instead. Zero TF changes needed for basic integration. |
| No terrain-aware routing | `spatial::pathfinding` — `shortest_path()`, `dijkstra_map()` | Small Addition | `PathfindingConstraints` has per-direction cost but no per-cell cost. Add `cell_costs: Option<Grid<f32>>` (~15 lines in `pathfinding.rs`). |
| No multi-tile structures | `algorithms::prefab` — `PrefabPlacer`, rotation, tags, placement modes | New Feature | No join-edge system for composable chunks. Needs `JoinEdge` concept + growth loop. Largest effort. |
| No faction spatial arrangement | `algorithms::voronoi`, `analysis::graph` — MST/Delaunay, `semantic` — Region tags | Small Addition | Voronoi partitions space; Graph computes MST over district centroids. Need region-ID output from Voronoi and `Region::center()`. |
| No WFC interiors | `algorithms::wfc` — `Wfc`, `WfcPatternExtractor`, backtracking | Ready | Fully functional. Binary `Tile` (Floor/Wall) limits multi-type output — post-process with prefab system for furniture. Work is authoring patterns + adapter. |
| No typed roads | `spatial::pathfinding` — `shortest_path()` returns `Vec<(usize, usize)>` | Game-Side | `shortest_path` already returns coordinates. Game paints typed tiles (`cobblestone`, `salt_brick`) using those coords. Zero TF changes. |

---

## Phase 1 — Foundation

**Objective**: Extend terrain-forge with `Region::center()` and per-cell cost support; refactor `paint_roads` to use terrain-forge A\* pathfinding instead of L-shaped walks.

### terrain-forge PRs

| Task | Description | Hours |
|------|-------------|-------|
| `Region::center()` | Add `pub fn center(&self) -> (f32, f32)` — centroid of `self.cells`. Return `(0.0, 0.0)` for empty regions. | 1 |
| Per-cell cost | Add `cell_costs: Option<Grid<f32>>` to `PathfindingConstraints`. Multiply direction cost by destination cell cost during Dijkstra expansion. | 4 |
| Tests | `Region::center()` on empty/single/multi-cell. `shortest_path` with cell costs (wall=∞, floor=1.0, glass=3.0). | 2 |

### saltglass-steppe changes

| Task | File | Description | Hours | Depends On |
|------|------|-------------|-------|------------|
| Cost map builder | `terrain_forge_adapter.rs` | `build_cost_grid(map: &Map) -> Grid<f32>` — `Wall`→∞, `dry_soil`→1.0, glass→3.0. Costs from `terrain_config.json`. | 3 | TF per-cell cost |
| Refactor `paint_roads` | `settlement/mod.rs` | Replace `paint_path` (L-shaped) with `shortest_path` using cost grid. Convert returned `Vec<(usize, usize)>` to `dirt_path` tiles. Keep MST edge selection unchanged. | 4 | Cost map builder |
| Fallback path | `settlement/mod.rs` | If `shortest_path` returns `None`, fall back to L-shaped walk. | 1 | Refactor paint_roads |
| Test updates | tests | Verify roads follow non-L-shaped paths. Deterministic seed comparison. | 2 | — |

### Data changes

| File | Change | Hours |
|------|--------|-------|
| `terrain_config.json` | Add `"tile_movement_costs": { "dry_soil": 1.0, "salt_crust": 1.5, "soft_sand": 2.0, "glass_shard": 5.0 }` | 1 |

### Acceptance criteria

- `Region::center()` returns centroid; empty → `(0.0, 0.0)`
- `shortest_path` with cell costs produces shorter weighted paths
- `paint_roads` produces organic, terrain-aware paths (no L-shapes)
- All 26 tests pass; deterministic output for seed 12345
- `cargo test` + `./test_all_algorithms.sh` green

### Risks

- terrain-forge changes require local path dependency during dev, then version bump + publish
- Per-cell cost may need Dijkstra internals refactor — inspect before committing

**Phase total: ~18 hours**

---

## Phase 2 — Typed Roads

**Objective**: Replace the single `dirt_path` with typed roads (`dirt_path`, `cobblestone`, `salt_brick`) selected by faction, tier, and proximity to settlement center.

### terrain-forge PRs

None — Phase 1 pathfinding is sufficient.

### saltglass-steppe changes

| Task | File | Description | Hours | Depends On |
|------|------|-------------|-------|------------|
| Road type enum | `settlement/mod.rs` | `RoadType` enum → floor ID mapping. | 1 | — |
| Road selection | `settlement/mod.rs` | Per-edge type: faction preference, tier (Village=dirt only, City=mixed), distance from center (inner→upgraded). | 3 | Road type enum |
| Road width | `settlement/mod.rs` | MST edges width 2, loop edges width 1. Paint adjacent parallel tiles. | 2 | Phase 1 A\* paths |
| Paint with type | `settlement/mod.rs` | Use selected floor ID instead of hardcoded `"dirt_path"`. | 1 | Road selection |
| Renderer | `map_elements.json` | Ensure new floor types render with distinct glyphs/colors. | 1 | — |

### Data changes

| File | Change | Hours |
|------|--------|-------|
| `map_elements.json` | Add `cobblestone` (glyph `═`, Gray) and `salt_brick` (glyph `▬`, Cyan) floor defs. | 1 |
| `data/road_types.json` (new) | Road type configs with cost, width, faction preferences. | 2 |
| `terrain_config.json` | Movement costs for new floor types. | 0.5 |

### Acceptance criteria

- Mirror Monks settlements produce `salt_brick` roads near center
- Villages produce only `dirt_path`; Cities produce mixed types
- Road types render with distinct glyphs/colors in TUI
- Width-2 main roads don't collide with building footprints
- `road_types.json` validates with `jq`

### Risks

- Width-2 roads may overlap buildings — check bounds before widening
- Too many floor types may confuse players — keep glyph differences subtle, color-distinct

**Phase total: ~11.5 hours**

---

## Phase 3 — Faction-Driven Layout

**Objective**: Use Voronoi tessellation to divide settlements into faction-controlled districts with faction-specific buildings, decorations, and road types.

### terrain-forge PRs

| Task | Description | Hours |
|------|-------------|-------|
| Voronoi region tagging | Ensure Voronoi output can be consumed as tagged regions via `SemanticExtractor`. Use `flood_regions()` post-hoc to assign site IDs. | 3 |
| Region adjacency | `pub fn adjacent_regions(regions: &[Region], grid: &Grid) -> Vec<(u32, u32)>` — pairs of region IDs sharing a border. | 3 |

### saltglass-steppe changes

| Task | File | Description | Hours | Depends On |
|------|------|-------------|-------|------------|
| District zoning | `settlement/districts.rs` (new) | Generate Voronoi sites (one per significant faction + neutral). Assign cells to districts. Map districts → factions via `faction_control`. | 5 | TF Voronoi tagging |
| Faction building filter | `settlement/buildings.rs` | Filter structure candidates by district faction. Mirror Monks district → `mirror_monks` tagged structures. | 3 | District zoning |
| District boundaries | `settlement/districts.rs` | Paint border cells with faction-specific floor tiles using adjacency query. | 2 | TF region adjacency |
| District decorations | `settlement/mod.rs` | `place_decorations` uses district faction instead of global dominant faction. | 2 | District zoning |
| District-aware roads | `settlement/mod.rs` | Roads within a district use that faction's preferred road type. Transition tile at boundaries. | 2 | Phase 2 typed roads |

### Data changes

| File | Change | Hours |
|------|--------|-------|
| `data/faction_layouts.json` (new) | Per-faction: `building_density`, `preferred_road`, `border_floor`, `decoration_palette`, `layout_style` (radial/clustered/linear). | 3 |
| `data/structures/structures.json` | Tag existing structures with faction affinity. Add 2-3 faction-specific structures per major faction. | 4 |

### Acceptance criteria

- Settlements with 2+ factions produce visually distinct districts
- Building selection respects district faction
- District borders visible in TUI (distinct floor tiles)
- Single-faction settlements produce uniform layout (no artificial districts)
- Deterministic: same seed + same factions = same district layout
- `faction_layouts.json` validates with `jq`

### Risks

- Voronoi with few sites (2-3) may produce unbalanced districts — mitigate with Lloyd relaxation (2-3 iterations)
- District boundaries may cut through buildings — run placement after district assignment
- Performance: Voronoi on 180×120 (City) is ~21K cells — should be fast, but profile

**Phase total: ~27 hours**

---

## Phase 4 — Interior Generation & Multi-Tile Structures

**Objective**: Use WFC to generate building interiors. Support multi-tile landmark structures that span multiple building footprints.

### terrain-forge PRs

| Task | Description | Hours |
|------|-------------|-------|
| WFC pattern library | Extend `Wfc` to accept pre-authored pattern sets from JSON. Add `Wfc::generate_from_pattern_set()`. | 4 |
| Sub-grid generation | `pub fn generate_subgrid(algo, width, height, seed, params) -> Grid<Tile>` — small grid for building-scale generation. | 2 |

### saltglass-steppe changes

| Task | File | Description | Hours | Depends On |
|------|------|-------------|-------|------------|
| Interior generator | `settlement/interiors.rs` (new) | Per-building WFC interior generation. Input: footprint + faction + building type. Output: tile grid stamped inside walls. | 6 | TF WFC pattern library |
| Pattern authoring | `data/interior_patterns/` | 3-4 WFC pattern sets: `shop_interior`, `residence_interior`, `temple_interior`, `guild_hall_interior`. | 4 | — |
| Multi-tile landmarks | `settlement/landmarks.rs` (new) | Landmarks span 2-4 merged building footprints. Select adjacent positions, merge, generate interior at merged scale. | 5 | Interior generator |
| Landmark selection | `settlement/landmarks.rs` | Per-faction landmark types. 1 per Town, 2-3 per City, 0 for Villages. | 2 | Phase 3 districts |
| Interior entity spawning | `settlement/interiors.rs` | Place NPCs/interactables inside interiors by building type. Merchant → counter + NPC. Temple → altar + priest. | 3 | Interior generator |
| Structure library integration | `structure_library.rs` | Register multi-tile landmarks so quests/narrative can reference them. | 2 | Landmarks |

### Data changes

| File | Change | Hours |
|------|--------|-------|
| `data/interior_patterns/` (new dir) | WFC pattern files per building type. JSON with tile_size, patterns, constraints. 4 files minimum. | 5 |
| `data/landmarks.json` (new) | Landmark configs: id, faction, min_footprint, interior_pattern, required_features, npc_slots. | 3 |
| `data/structures/structures.json` | Large multi-tile landmark patterns. | 3 |

### Acceptance criteria

- Building interiors are generated (not empty rectangles)
- Interiors contain appropriate furniture/interactables for building type
- Towns have 1 landmark; Cities have 2-3; Villages have 0
- WFC interiors are deterministic (same seed = same interior)
- Interior generation <100ms per building
- `interior_patterns/*.json` and `landmarks.json` validate with `jq`

### Risks

- WFC may fail to converge for small footprints (6×6) — fallback to template stamping below 8×8
- Multi-tile merging may produce irregular shapes — constrain to rectangular unions
- Pattern authoring is labor-intensive — start with 1 set per type, iterate. Consider extracting from existing hand-authored structures via `WfcPatternExtractor`
- Interior generation for 20+ buildings adds time — cap at 50ms total or generate lazily on first visit

**Phase total: ~39 hours**

---

## Summary

| Phase | Focus | TF Changes | SS Changes | Data | Hours |
|-------|-------|-----------|-----------|------|-------|
| 1 | Foundation | `Region::center()`, per-cell cost | Cost map, A\* road refactor | `tile_movement_costs` | ~18 |
| 2 | Typed Roads | — | Road types, width, faction selection | `road_types.json`, floor defs | ~11.5 |
| 3 | Faction Layout | Region adjacency, Voronoi tagging | District zoning, faction buildings | `faction_layouts.json` | ~27 |
| 4 | Interiors | WFC pattern library, sub-grid gen | Interior gen, landmarks, entity spawn | `interior_patterns/`, `landmarks.json` | ~39 |
| **Total** | | | | | **~95.5** |

### Dependency Chain

```
Phase 1 ──→ Phase 2 (roads need A* from Phase 1)
Phase 1 ──→ Phase 3 (districts need cost grid for boundary-aware roads)
Phase 2 ──→ Phase 3 (district roads need typed roads from Phase 2)
Phase 3 ──→ Phase 4 (interiors need district/faction context from Phase 3)
```

### Creative Pillar Alignment

- **Storms Rewrite Maps**: District boundaries and road types give storms more meaningful targets — vitrifying cobblestone into glass creates a different tactical situation than hitting dirt.
- **Mutation with Social Consequences**: Faction districts create spatial consequences for reputation — high Refraction players may be unwelcome in certain districts.
- **TUI as Aesthetic Strength**: Distinct road glyphs (`╌` dirt, `═` cobblestone, `▬` salt brick) and district border tiles make settlements visually readable in ASCII.
- **Authored Weirdness**: Faction-specific landmarks (refraction cathedrals, storm shrines) add authored strangeness to procedural settlements.

---

## Related Documents

- `SETTLEMENT_GENERATION_RESEARCH.md` — research comparison (CDDA, CoQ) that motivated this plan
- `SETTLEMENT_FUTURE_WORK.md` — deferred features (z-levels, micro-prefabs)
- `TERRAIN_FORGE_IMPROVEMENT_SUGGESTIONS.md` — terrain-forge API requests from integration work
- `SETTLEMENT_GENERATION_PLAN.md` — original (stale) implementation plan
