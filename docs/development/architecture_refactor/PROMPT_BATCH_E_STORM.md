# VERA Batch E Prompt: Storm System

**Execute VERA Batch E: Storm system migration. Work directly on main.**

Read `docs/development/architecture_refactor/VERA_FULL_MIGRATION.md` — this is Batch E.

**Goal:** Convert `StormSystem::apply_storm()` from direct map mutation to a rule that returns traced effects.

**Current implementation:**

`systems/storm.rs` has `StormSystem::apply_storm(state: &mut GameState)` which:
1. Reads storm state (intensity, edit types, countdown)
2. Selects tiles to edit based on intensity and RNG
3. Applies 7 edit types: Glass, Rotate, Swap, Mirror, Fracture, Crystallize, Vortex
4. Each edit directly mutates `state.world.map.tiles[idx]`
5. May spawn wraiths (enemies) at high intensity
6. Updates storm highlight state

The storm timer tick (`storm.tick()`) is separate — it decrements the countdown and returns a bool indicating whether the storm should fire.

**Migration approach:**

1. **Rule function:** `rule_storm_tick(ctx: &QueryContext, rng: &mut ChaCha8Rng) -> RuleOutput`

   The rule reads storm state from QueryContext, decides which tiles to edit and how, and returns effects. The map mutation happens in apply, not in the rule.

2. **New Effect variants:**
   ```rust
   pub enum StormEffect {
       EditTile { x: i32, y: i32, edit_type: StormEditType, new_tile: TileData },
       SpawnWraith { x: i32, y: i32, wraith_id: String },
       AdvanceTimer { new_countdown: i32 },
       SetIntensity { level: u8 },
       SetHighlight { tile_indices: Vec<usize> },
       ClearHighlight,
   }
   ```

   `StormEditType` is an enum matching the 7 edit types. `TileData` carries enough info for the apply arm to set the tile (or the apply arm can call the existing edit functions).

3. **Pragmatic option for complex edits:** The Rotate, Swap, Mirror edits operate on rectangular regions, not individual tiles. If returning per-tile effects is too verbose, use coarse effects:
   ```rust
   StormEffect::ApplyEdit { edit_type: StormEditType, region: Rect, seed: u64 }
   ```
   The apply arm calls the existing edit function with the region and seed. The trace records "a Rotate edit was applied to this region" — sufficient for verification.

4. **QueryContext expansion:** The rule needs to read storm state (intensity, countdown, edit types, active status). Add storm fields to QueryContext or pass storm state as a parameter.

**Key constraints:**
- RNG ordering: storm edits consume RNG for tile selection and edit parameters. Preserve the consumption order exactly.
- Storm edits are the game's signature mechanic — the trace should clearly show what changed. Coarse effects (per-edit, not per-tile) are the right granularity.
- Wraith spawning should produce effects that the apply arm handles (add enemy to world.enemies, update spatial index).
- The storm timer tick can stay as a simple effect: `StormEffect::AdvanceTimer`.

**Integration with end_turn:**
```rust
TurnPhase::TickStorm => {
    let ctx = QueryContext::from_fields(self);
    let output = rule_storm_tick(&ctx, &mut self.rng);
    self.apply_and_trace(output, "rule_storm_tick");
    // Derives: update_lighting (storm edits change map, affecting light)
}
```

**New file:** `src/game/rules/storm.rs`

**Tests:**
- Rule unit tests: storm at intensity 0 produces no edits, storm at intensity 5 produces edit effects, wraith spawn at high intensity
- Existing DES scenario `storm_intensity_scaling_test` must pass
- Consider adding a DES scenario that asserts on storm edit effects in the trace

**After completing:** Report how many StormEffect variants were needed, whether coarse or fine granularity was used, and whether the existing storm edit functions were reused or rewritten.
