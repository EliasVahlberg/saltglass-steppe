---
status: current
last_verified: 2026-04-04
commit: e0d1fe7
---

# Settlement Generation — Future Work

Items deferred from the settlement generation sprint. Not critical path.

---

## [FUTURE] Furniture & Decoration Micro-Prefab System

Small inline patterns (1–4 tiles) placed procedurally inside buildings and around entrances.

**Indoor furniture:**
- `data/structures/furniture.json` — micro-prefabs with `placement` tag (`wall`, `floor`, `corner`) and `building_tags`
- Auto-rotate based on adjacent wall orientation
- Examples: `stove_and_sink` (1×4, wall), `bed` (1×3, wall), `table_and_chairs` (2×2, floor)

**Outdoor props:**
- Scatter 1–3 single-tile props (`barrel`, `crate`, `lumber_pile`) in `ground`-type cells near building entrances
- Driven by `outdoor_props` list in structure metadata

**Prerequisites:**
- `ground` and `path` legend types ✅ (implemented 2026-03-01)
- Structures need `building_tags` added to metadata for furniture eligibility

---

## [FUTURE] Multi-Z-Level Settlements (Basements, Upper Floors)

Absolute x,y positions that translate directly across z-levels.

**Design:**
- `z=0` surface, `z=-1` basement, `z=1` upper floor
- Same tile coordinate maps to the cell directly above/below
- Staircase transitions using existing `StairsDown`/`StairsUp` tiles
- Per-level map storage in `WorldState`

**Prerequisites:**
- Z-level tracking in `WorldState` (currently only `layer: i32` for subterranean)
- Per-level map storage (currently single `world.map`)
- Builds on existing `enter_subterranean` / `exit_subterranean` pattern

**Not needed until:** multi-story buildings or underground vaults beneath settlements are in scope.

---

## [FUTURE] Settlement Plan Doc Update

`docs/development/SETTLEMENT_GENERATION_PLAN.md` is stale — written before implementation and diverges significantly from what was built. Either:
- Update to reflect actual implementation (StructureLibrary, grid-with-jitter, no separate building_types.json, etc.)
- Or archive it and mark superseded by `docs/features/SETTLEMENT_GENERATION_SUMMARY.md`
