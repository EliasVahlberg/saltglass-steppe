# terrain-forge Improvement Suggestions

Collected from saltglass-steppe integration work. Candidates for a future major version bump.

---

## API Ergonomics

### `Region::center() -> Option<(u32, u32)>`
**Context**: Settlement layout generation needs region centroids as building placement positions.  
**Current workaround**: Compute manually — `(sum_x / count, sum_y / count)` over `region.cells`.  
**Suggested addition**: Public method on `Region` in `src/semantic.rs`.

```rust
pub fn center(&self) -> Option<(u32, u32)> {
    if self.cells.is_empty() { return None; }
    let sum_x: u32 = self.cells.iter().map(|(x, _)| x).sum();
    let sum_y: u32 = self.cells.iter().map(|(_, y)| y).sum();
    let count = self.cells.len() as u32;
    Some((sum_x / count, sum_y / count))
}
```

---

## Semantic System

### Named zone markers for settlement generation
**Context**: Settlement layout needs semantic zones like `town_square`, `residential_block`, `faction_district` — not just generic `Spawn`/`Exit`/`Custom`.  
**Current workaround**: Use `MarkerType::Custom(String)` with agreed-upon string conventions.  
**Suggested addition**: A `Settlement` variant family in `MarkerType`, or a dedicated `SemanticConfig::for_settlements()` preset with appropriate size thresholds and marker types.

### `SemanticConfig` presets for non-dungeon maps
**Context**: `for_rooms()`, `for_caves()`, `for_mazes()` are dungeon-oriented. Settlement and overworld maps need different size thresholds and marker densities.  
**Suggested addition**: `SemanticConfig::for_settlements()` and `SemanticConfig::for_overworld()` presets.

### Isolated room generation (no corridors)
**Context**: Settlement building placement needs isolated floor regions — one region per building plot. All current algorithms (`bsp`, `rooms`, `voronoi`) produce connected floor areas, so `SemanticExtractor` flood-fills them into a single region.  
**Current workaround**: Grid-based position generation with jitter (no terrain-forge semantic extraction used).  
**Suggested addition**: A `PlotLayout` algorithm or a `SimpleRooms` option (`connect_rooms: false`) that places rooms without carving corridors between them. Each room would be its own flood-fill region, enabling semantic extraction to return one region per building plot.

---

## Generation

### Road/path generation between semantic markers
**Context**: Settlements need streets connecting buildings. Currently there's no explicit path generation — only the corridor system built into BSP.  
**Suggested addition**: A post-processor or algorithm that takes a set of `(x, y)` points and carves floor paths between them (e.g. A* or straight-line with jitter).

### Hierarchical generation (district → block → building)
**Context**: City-scale settlements need nested structure: districts carved by Voronoi, blocks by BSP within each district, individual building plots within blocks.  
**Current workaround**: Single-pass BSP or Voronoi — no nesting.  
**Suggested addition**: A `Pipeline` stage that applies an algorithm within each region of a prior stage's output.

---

## Notes

- Items are ordered roughly by implementation effort (smallest first).
- `Region::center()` is the most immediately useful — low effort, high value.
- Road generation and hierarchical generation are larger features that would justify a major version bump on their own.
