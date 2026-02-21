# Terrain Forge Feature Request: Spawn Markers & Semantic Layers

> Status update: terrain-forge v0.3.x now includes semantic layers (`SemanticExtractor`, `Region`, `Marker`, `Masks`, `ConnectivityGraph`). This request remains as a reference for desired behaviors and future refinements (e.g., richer masks/metadata and configurable marker presets).

## Summary
Add semantic outputs to terrain-forge so downstream games can materialize entities (items/enemies/NPCs/lights/interactables) deterministically without baking game logic into terrain generation. The goal is to emit spawn-ready annotations (regions, markers, masks) alongside tiles.

## Motivation
- Current terrain-forge output is just tiles; game code must rediscover rooms/clearings/connectors to place spawns.
- Homogeneous, data-driven generation is blocked by the lack of spawn slots and region metadata.
- Providing semantic layers keeps terrain-forge focused on geometry while enabling richer, deterministic spawning in consuming games.

## Goals
- Emit reusable, game-agnostic annotations: regions, markers (slots), masks.
- Support constrained sampling (distance, density, reachability) via utilities.
- Keep outputs deterministic (seeded RNG) and lightweight to serialize.

## Non-Goals
- No game-specific enemy/item selection inside terrain-forge.
- No rendering or UI concerns.
- No change to existing tile outputs beyond adding optional semantic layers.

## Proposed Additions (v0.2.x)

### Data Structures
- `Region { id, kind: String, bbox: Rect, cells: Vec<(u32,u32)>, tags: Vec<String> }`
- `Marker { x: u32, y: u32, tag: String, weight: f32, region_id: Option<u32>, tags: Vec<String> }`
- `Masks { walkable: BitGrid, no_spawn: BitGrid, poi_core: BitGrid }`
- `ConnectivityGraph { regions: Vec<Region>, edges: Vec<(u32,u32)> }`

### Generation Result
Extend generation output to include:
- `tiles: Grid<Tile>`
- `regions: Vec<Region>` (rooms, corridors, clearings, perimeter)
- `markers: Vec<Marker>` (spawn slots tagged by type)
- `masks: Masks` (walkable + optional exclusions)
- `connectivity: ConnectivityGraph`

### Marker/Slot Emission
- Per-algorithm hooks to emit markers:
  - Rooms: `loot_slot`, `npc_slot`, `light_anchor`, `boss_room_core`
  - Corridors: `patrol_path`, `trap_slot`
  - Clearings: `camp_slot`, `poi_central`
- Optional quotas per region (e.g., `3 loot_slot per room`).
- Weighted tags for variety.

### Sampling Utilities
- Poisson/farthest-point sampler constrained to a mask.
- Region distribution helper: distribute N markers across regions proportional to area or weight.
- Constraint filters: min distance to start, avoid chokepoints, enforce reachability.

### Tag Propagation
- Accept caller-provided tags (biome/poi/terrain) and attach to regions/markers.
- Allow algorithms to add tags (e.g., `chokepoint`, `junction`, `dead_end`).

## Example Flow
1) Caller requests generation with biome/poi tags.
2) Terrain-forge returns tiles + regions + markers (`light_anchor`, `treasure_cache`, `altar`).
3) Game uses markers to materialize entities via its own spawn tables.

## API Sketch
```rust
pub struct GenerationResult {
    pub tiles: Grid<Tile>,
    pub regions: Vec<Region>,
    pub markers: Vec<Marker>,
    pub masks: Masks,
    pub connectivity: ConnectivityGraph,
}

pub fn generate_with_markers(config: &Config, seed: u64) -> GenerationResult;
pub fn sample_markers(mask: &BitGrid, count: usize, method: Sampling) -> Vec<(u32,u32)>;
pub fn distribute_markers(regions: &[Region], tag: &str, total: usize) -> Vec<Marker>;
```

## Acceptance Criteria
- Marker/region layers compile under `no_std` optional feature set (if supported today).
- Deterministic outputs given seed + config.
- Backward compatible: existing APIs stay intact; new fields are additive.
- Benchmarks show minimal overhead when semantic layers are requested.

## Open Questions
- Do we expose region decomposition in all algorithms or only in BSP/rooms/cellular?
- Should masks be compressed (bit-level) or leave as bool grid?
- Do we support user-defined marker injectors (callback) to let callers add custom tags at generation time?
