# State Store Refactor — Working Document

> Status: Final cleanup remaining
> Reference: `VERIFIED_STATE_STORE.md` (standing architecture document)
> Baseline: 3,195 LOC → 1,387 LOC (Stages 1-5 + decomposition complete)

---

## Completed

- [x] Stage 1: Mutation enum, apply_one, apply_mutations
- [x] Stage 2: dispatch.rs, notify.rs, combat end-to-end
- [x] Stage 3: All 16 commands through dispatch.rs
- [x] Stage 4: Bridge subsystems (TickSubsystem)
- [x] Stage 5: Delete effects/apply.rs, old dispatch infrastructure
- [x] World travel → systems/world.rs (−822 LOC)
- [x] Constructors → state_init.rs (−390 LOC)
- [x] Auto-explore → systems/explore.rs (−238 LOC)
- [x] Turn system → systems/turn.rs (−128 LOC)
- [x] Movement → systems/movement.rs (−80 LOC)

## Final Cleanup: state.rs 1,387 → ~1,080 LOC

Everything below is a mechanical move — no architectural changes, no new patterns.

### Move to systems/player.rs (~83 LOC)

| Method | LOC | Why it doesn't belong |
|--------|-----|----------------------|
| `check_adaptation_threshold` | 22 | Game logic: checks refraction vs thresholds, grants adaptations |
| `apply_light_effects` | 43 | Game logic: checks light levels, applies status from items |
| `apply_status_effect` | 23 | Game logic: stack/duration merge rules |
| `recalc_equipment_stats` | 13 | Game logic: derives weapon/armor from equipment |

`apply_status_effect` is called from `apply_one` (AddStatusEffect arm). After the move, the arm calls `systems::player::apply_status_effect(self, id, duration)` — same pattern as other bridge arms.

`recalc_equipment_stats` is called from `apply_one` (SetEquipment arm). Same treatment.

### Move to systems/items.rs (~63 LOC)

| Method | LOC | Why it doesn't belong |
|--------|-----|----------------------|
| `can_open_chest` | 15 | Game logic: adjacency check |
| `open_chest` | 29 | Game logic: lock/key check, unlock, open |
| `transfer_to_chest` | 22 | Game logic: capacity check, inventory ↔ chest |
| `transfer_from_chest` | 19 | Game logic: inventory ↔ chest |

These are called from main.rs UI handlers. After the move, callers use `systems::items::open_chest(state, idx)` instead of `state.open_chest(idx)`.

### Move to systems/world.rs or generation (~53 LOC)

| Method | LOC | Why it doesn't belong |
|--------|-----|----------------------|
| `generate_crystal_formations` | 53 | Generation logic, not state management |

Called from state_init.rs and load_test_tile. After the move, callers use `systems::world::generate_crystal_formations(state, biome, rooms, rng)`.

### Move to test infrastructure (~35 LOC)

| Method | LOC | Why it doesn't belong |
|--------|-----|----------------------|
| `load_test_tile` | 35 | Test-only, not part of the game state API |

Move to `des_testing.rs` or a `#[cfg(test)]` block.

### Delete: delegation accessors (~80 LOC)

The entire second `impl GameState` block (L1245-1349):
```rust
pub fn player_x(&self) -> i32 { self.player.x }
pub fn map(&self) -> &Map { &self.world.map }
pub fn enemies(&self) -> &Vec<Enemy> { &self.world.enemies }
// ... 30+ more
```

All fields are `pub`. Callers can write `state.player.x` directly. These add zero value. Delete them and update callers.

### Delete: pickup_items wrapper (~3 LOC)

```rust
pub fn pickup_items(&mut self) { MovementSystem::pickup_items(self) }
```

One-line delegation. Callers can use `MovementSystem::pickup_items(state)` directly.

### Visual effect wrappers — decision needed (~60 LOC)

```rust
pub fn trigger_hit_flash(&mut self, x: i32, y: i32) {
    self.world.visual_effects.trigger_hit_flash(x, y);
}
// ... 10 more one-line delegations
```

These are used by the rendering layer. Two options:
- **Delete**: callers access `state.world.visual_effects` directly
- **Keep**: they're thin, they hide the field path, rendering code doesn't need to know about `world.visual_effects`

Recommendation: delete. The rendering layer already accesses `state.world` for other things.

### Summary

| Action | LOC removed | Destination |
|--------|-------------|-------------|
| Delete delegation accessors | ~80 | Callers use `state.player.x` etc. |
| Move player system methods | ~83 | `systems/player.rs` |
| Move chest operations | ~63 | `systems/items.rs` |
| Delete visual effect wrappers | ~60 | Callers use `state.world.visual_effects` |
| Move crystal generation | ~53 | `systems/world.rs` |
| Move load_test_tile | ~35 | `des_testing.rs` |
| Delete pickup_items wrapper | ~3 | Callers use `MovementSystem::pickup_items` |
| **Total** | **~377** | |

**Projected state.rs: 1,387 − 377 = ~1,010 LOC**

### What remains after cleanup (~1,010 LOC)

| Category | LOC | Content |
|----------|-----|---------|
| Data model | ~178 | Struct defs, rng_serde, MsgType, msg_type_from_str |
| Mutation engine | ~430 | apply_one (exhaustive match), apply_mutations |
| Command API | ~55 | dispatch, dispatch_craft/buy/sell |
| Derives | ~55 | update_fov, update_lighting |
| Spatial index | ~40 | ensure_spatial_index, rebuild_spatial_index_internal |
| Queries | ~50 | effective_ambient_light, get_light_level, enemy_at, npc_at, has_adaptation, has_status_effect, get_reputation, get_quest_ids_for_location, get_next_tutorial, dismiss_tutorial |
| Logging | ~28 | log, log_typed, log_quest_completions, apply_presentation |
| Misc | ~20 | save, load, trigger_effect, debug_command |

Everything in this list is either data definition, the mutation engine, derives, or read-only queries. Zero system logic.

## Future Work (not in this cleanup)

- **Mutation enum cleanup**: Remove duplicate variants (SpendAp vs SetPlayerAp, AddHp vs SetPlayerHp, etc.). Keep Set* variants, remove relative variants. Systems compute final values.
- **Bridge arm decomposition**: Replace WorldMove/MovePlayer/EndTurn/RestTick bridge arms with atomic mutations. Requires the systems to produce the full mutation sequence.
- **apply_one inline logic extraction**: Move QuestNotify (~30 LOC), UsePsychicAbility (~15 LOC), AttemptFlee (~20 LOC) logic to helper functions or system functions.
- **rules/ absorption**: Move remaining rule functions into their corresponding systems/ modules.

## Done When

- [x] state.rs ≤ 1,100 LOC
- [ ] Zero system logic in state.rs (adaptation, light effects, status merge, chest ops, crystal gen)
- [ ] Delegation accessors deleted
- [ ] Visual effect wrappers deleted
- [ ] load_test_tile moved to test infrastructure
- [ ] `cargo test` passes, all DES scenarios pass
