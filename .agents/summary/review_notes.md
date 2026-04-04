# Review Notes

## Post-Cleanup Regeneration (2026-04-04)

This documentation was regenerated after a major dead code cleanup that removed ~3,600 LOC of dead/half-wired code. The previous summary files contained several false claims that are now corrected.

### Corrections from Previous Version

| Previous Claim | Reality | Status |
|---|---|---|
| `ritual.rs` exists | Never existed | Corrected — not mentioned |
| Algorithm registry in `registry.rs` | File never existed | Corrected — not mentioned |
| 7 structure algorithms selectable | None used in game; all deleted | Corrected — terrain-forge only |
| All special systems functional | Light/crystal/void abilities unreachable | Corrected — abilities removed, resource accumulation kept |
| terrain-forge 0.3.1 | Actually 0.7.0 | Corrected |

### Known Data Integrity Issues

18 dangling cross-references exist in data files (not a documentation issue, but noted):
- 16 item IDs in `biome_spawn_tables.json` reference items not defined in `items.json`
- 2 item IDs in `loot_tables.json` reference items not defined in `items.json` (angle_split_lens, prism_shard)

No runtime validation exists for these cross-references. DataLoader validates schema structure but not referential integrity.

### Documentation Gaps

1. **Renderer pipeline**: No dedicated architecture doc. `docs/features/RENDERER_ENHANCEMENT_OVERVIEW.md` focuses on enhancements, not the base pipeline.
2. **Save migration strategy**: No doc explaining versioning or how to add migrations.
3. **Enemy AI behaviors**: No design doc for when each behavior is used or how to add new ones.
4. **Effect system**: `src/game/effect.rs` has parsed effects and conditions with no dedicated documentation.

### Consistency Check

- Entry points match Cargo.toml `[[bin]]` declarations (mapgen-tool only; schema_gen auto-discovered)
- Dependency versions match Cargo.toml
- Component descriptions match post-cleanup source code
- DES scenario format matches `src/des/mod.rs` implementation
- Data file references match actual `data/` contents
