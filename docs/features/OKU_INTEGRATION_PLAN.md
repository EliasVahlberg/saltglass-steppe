# Oku/Ogun Integration Plan for Larger Settlements

## Status: PROPOSAL

## Problem

Current settlement generation uses a simple grid layout with jitter + overlap rejection.
Max buildings: Village=6, Town=12, City=20. No spatial optimization, no category-aware
placement, no intelligent road routing. Buildings are placed on a shuffled grid and
connected via MST + A* pathfinding after the fact.

## Goal

Use oku (city generation facade over ogun) to generate spatially-optimized settlements
with category-aware building placement, negotiated road routing, and optional erosion
for ruins. Scale up to 30-60+ buildings for Cities while maintaining organic layouts.

---

## Architecture Overview

```
Current:
  SettlementConfig → grid layout → random building pick → overlap reject → MST roads → stamp

Proposed:
  SettlementConfig → build AgentCatalog from StructureLibrary
                   → build CitySpec from SettlementConfig
                   → oku::generate() → CityLayout
                   → translate CityLayout → Settlement
                   → stamp (reuse existing stamp_settlement / paint_roads)
```

The key insight: oku handles the hard part (where buildings go, how roads connect),
then we translate back to the existing Settlement struct and reuse all the stamping,
decoration, and NPC placement code unchanged.

---

## Dependency Compatibility

| Crate | saltglass-steppe | ogun/oku |
|-------|-----------------|----------|
| rand | 0.8 | 0.9 |
| rand_chacha | 0.3 | 0.9 |
| serde | 1.0 | 1.0 ✓ |

**RNG mismatch is not a blocker.** Oku manages its own RNG internally from a `u64` seed.
We pass `SettlementConfig.seed` → `CitySpec.seed` and never share RNG instances across
the boundary. Both versions of rand can coexist in the dependency tree (Cargo handles
this), though it adds ~2s compile time.

**Option:** Upgrade saltglass-steppe to rand 0.9 in a separate PR. This is a larger
change (trait API differences) but would unify the dependency tree. Not required for
integration.

---

## Phase 1: Add oku dependency + translation layer

### 1a. Add oku as a path dependency

```toml
# Cargo.toml
oku = { path = "../oku" }
```

Use path dependency during development, switch to crates.io version for release.

### 1b. Create `src/game/generation/settlement/oku_bridge.rs`

Translation layer with two functions:

```rust
/// Build oku inputs from saltglass-steppe's settlement config + structure library
fn build_oku_inputs(config: &SettlementConfig) -> (CitySpec, AgentCatalog)

/// Translate oku output back to saltglass-steppe's Settlement struct
fn translate_city_layout(layout: &CityLayout, config: &SettlementConfig) -> Settlement
```

**build_oku_inputs:**
- Map `SettlementTier` → `CitySpec` dimensions (keep existing: Town 120×90, City 180×120)
- Map tier → `CityType` (Village→FrontierOutpost, Town→TradeHub, City→PlannedCapital)
- Map tier → `Era` (default Growth; Ruins POI → PostCollapse with erosion)
- Load `StructureLibrary`, filter `Connectable` structures
- Map each structure → `BuildingTemplate`:
  - `name` = structure.id
  - `category` = derived from tags/faction (see category mapping below)
  - `radius` = max(structure.width, structure.height) / 2
  - `priority` = structure.metadata.weight (normalized)
  - `connections` = based on category defaults
  - `material` = derived from faction (see material mapping below)
- Build faction-specific `InteractionMatrix` (or use oku defaults)
- Set `beta` based on faction: MirrorMonks=3.0 (ordered), StormCults=0.5 (chaotic)
- Pass terrain walls as `Space.obstacles` so buildings avoid existing terrain

**translate_city_layout:**
- For each `PlacedBuilding` in CityLayout:
  - Look up original structure by template_index → structure.id
  - Compute rotation: entrance faces nearest road or centroid (reuse existing logic)
  - Create `Building { prefab_name, x, y, faction, rotation }`
- Roads from CityLayout are already routed — convert to dirt_path tiles during stamping
  (replaces current MST + A* road painting)

### 1c. Category mapping

Add optional `category` field to `StructureMetadata` in structure_library.rs:

```rust
#[serde(default)]
pub category: Option<String>,  // "residential", "commercial", "sacred", "military", "infrastructure"
```

Fallback heuristic when not specified:
- Tags contain "temple"/"shrine"/"monastery" → Sacred
- Tags contain "barracks"/"watchtower"/"gate" → Military
- Tags contain "market"/"shop"/"tavern" → Commercial
- Tags contain "well"/"bridge"/"warehouse" → Infrastructure
- Default → Residential

### 1d. Material mapping (for erosion)

| Faction | Material | Rationale |
|---------|----------|-----------|
| MirrorMonks | Glass | Crystal/light theme |
| StormCults | Glass | Storm glass |
| SaltTradingCompany | Stone | Practical, durable |
| ArchiveDrones | Metal | Mechanical |
| Default | Stone | Generic |

---

## Phase 2: Wire into tile_generator.rs

### 2a. Conditional dispatch

In `tile_generator.rs`, the existing `if params.poi == POI::Town` block:

```rust
if params.poi == POI::Town {
    let settlement = if should_use_oku(&config) {
        oku_bridge::generate_oku_settlement(config, &map)
    } else {
        generate_settlement(config, &mut settlement_rng)
    };
    // ... rest of stamping unchanged
}
```

**`should_use_oku`**: Use oku for Town and City tiers. Keep current system for Village
(too few buildings to benefit, and the simple grid works fine at that scale).

### 2b. Pass terrain as obstacles

Before calling oku, scan the terrain-forge-generated map for wall tiles and pass them
as `Space.obstacles` (or use `routing_costs` with high cost for walls). This makes
ogun place buildings in open areas and route roads around terrain features.

```rust
let obstacles: Vec<ogun::Rect> = find_wall_clusters(&map);
// or
let routing_costs: Grid<f32> = map.tiles.iter().map(|t| if t.walkable() { 1.0 } else { 100.0 });
```

The `routing_costs` approach is better — it lets roads path through walls if needed
(the stamping code already clears walls around roads) while strongly preferring open ground.

### 2c. Road stamping from oku paths

Replace the current `paint_roads()` call with direct stamping from CityLayout.roads:

```rust
for road in &city_layout.roads {
    for &(rx, ry) in &road.path {
        map.set_tile(rx, ry, Tile::Floor { id: "dirt_path" });
    }
}
```

This replaces the MST + A* road generation entirely — oku's roads are already
optimally routed via ogun's congestion-aware Dijkstra with rip-up-and-reroute.

---

## Phase 3: Scale up building counts

With oku handling placement, we can safely increase building counts:

| Tier | Current | Proposed | Rationale |
|------|---------|----------|-----------|
| Village | 6 | 6 (unchanged) | Keep simple system |
| Town | 12 | 20-30 | Oku handles density well |
| City | 20 | 40-60 | Full oku with districts |

Performance at these scales (from ogun benchmarks):
- 30 nodes on 120×90 grid: ~15ms (well under budget)
- 60 nodes on 180×120 grid: ~50ms (comfortable)

### 3a. New structure templates

To fill larger settlements, we'll need more building variety:
- Small houses (radius 1-2) for residential infill
- Market stalls (radius 1) for commercial areas
- Guard posts (radius 1) for military
- Wells, fountains (radius 1) for infrastructure
- Faction-specific variants of each

These can be added to `data/structure_templates.json` incrementally.

---

## Phase 4: Ruins via erosion (optional, high-value)

For the Ruins biome, generate a full city then erode it:

```rust
let spec = CitySpec {
    city_type: CityType::Ruin,
    era: Era::PostCollapse,
    erosion: Some(ErosionSpec { severity: 0.6, seed }),
    ..
};
```

This produces partially-destroyed settlements with:
- Weakest buildings (Wood, Glass) removed first
- Disconnected buildings decay faster (connectivity cascade)
- Roads to removed buildings become dead ends
- Remaining structures feel like a real ruin, not random placement

This would replace the current Ruins POI generation with something much more
narratively coherent — you'd find actual ruined cities, not just scattered structures.

---

## Suggestions for ogun/oku improvements

### For ogun

1. **Rectangular footprints**: Currently nodes have circular radius. Buildings are
   rectangular. Adding `width`/`height` to `Node` (instead of just `radius`) would
   give tighter packing and more realistic overlap rejection. The potential function
   and `mark_footprint` already use square approximation, so this is mostly about
   the overlap check in `utility()`.

2. **Directional placement bias**: A `facing` field on Node that biases placement
   toward positions where the node can face a specific direction (toward roads,
   toward center). Currently we handle rotation post-hoc in the interpret layer.

3. **Incremental generation**: An API to place some nodes, then add more later.
   Useful for settlement growth over game time (start with founding buildings,
   add more as the player progresses). Currently `generate()` is all-or-nothing.

### For oku

1. **Hierarchy module**: The TODO stub for district→block→building generation would
   be very valuable for Cities. Districts could map to faction zones in saltglass-steppe
   (MirrorMonks quarter, Salt Trading district, etc.).

2. **Custom CityType variants**: Allow user-defined city types beyond the 4 built-in
   ones. Saltglass-steppe has faction-specific settlement styles that don't map cleanly
   to PlannedCapital/FrontierOutpost/TradeHub/Ruin.

3. **Terrain-aware generation**: Pass terrain heightmap or walkability as `Space.obstacles`
   or `routing_costs`. Currently Space is a blank rectangle. For saltglass-steppe,
   settlements are stamped onto terrain-forge output that already has walls and features.

4. **Building rotation in output**: CityLayout.PlacedBuilding has position but no
   rotation. Adding a `facing` direction (toward nearest road or centroid) would let
   the interpret layer handle rotation instead of the consumer.

5. **Road width**: Currently roads are 1-cell wide paths. A `road_width` parameter
   (or per-edge width based on traffic/importance) would produce more realistic
   main streets vs alleys.

6. **Serde for InteractionMatrix**: Allow loading custom interaction weights from JSON
   at runtime (currently hardcoded via `default_urban()`). This would let saltglass-steppe
   define faction-specific interaction rules.

   *Note: InteractionMatrix already supports serde and loads from embedded JSON
   (`data/default_weights.json`). The suggestion is to expose a constructor that
   takes a path or string, so consumers can provide their own weights without
   forking the crate.*

---

## Implementation Order

1. **Phase 1a-1b** (1-2 hours): Add dependency, create bridge module with translation functions
2. **Phase 1c-1d** (30 min): Add category field to structures, map factions to materials
3. **Phase 2a-2c** (1-2 hours): Wire into tile_generator, pass terrain as obstacles, stamp roads
4. **Testing** (1 hour): Visual testing with mapgen-tool, verify all tiers
5. **Phase 3** (ongoing): Add more structure templates as needed
6. **Phase 4** (1-2 hours): Ruins erosion integration

Total estimated effort: ~5-7 hours for core integration, then incremental content.

---

## Risks

- **Performance**: ogun at 60 nodes on 180×120 should be ~50ms. If it's slower,
  we can reduce node count or use lower beta (less optimization = faster).
- **Visual quality**: oku's placement may look too regular at high beta or too
  chaotic at low beta. Tuning beta per faction/tier will be important.
- **Structure variety**: With only ~10 connectable structures currently, larger
  settlements will have repetition. Need more templates before scaling to 40-60.
- **rand version coexistence**: Two versions of rand in the tree. Works but adds
  compile time. Consider upgrading saltglass-steppe to rand 0.9 separately.
