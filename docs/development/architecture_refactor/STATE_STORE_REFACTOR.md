# State Store Refactor — Working Document

> Status: Not started
> Target: Migrate from current VERA soft-migration to the verified state store architecture
> Reference: `VERIFIED_STATE_STORE.md` (standing architecture document)
> Baseline: Soft-migration complete (commit c6751ba), state.rs 3,195 LOC

---

## What Changes

| Current | Target |
|---------|--------|
| 15 `dispatch_*` methods on GameState (788 LOC) | Free functions in `systems/*.rs` |
| `effects/apply.rs` — domain-specific apply arms (419 LOC) | `state.apply_one()` — exhaustive match on `Mutation` |
| 7 Effect enums (PlayerEffect, CombatEffect, etc.) | Single `Mutation` enum in `mutations.rs` |
| `collect_reactions` hardcoded in state.rs | `notify.rs` — external, expandable |
| Compound effects (GainXp has level-up loop) | Atomic mutations (system calculates final values) |
| Post-processing hardcoded in dispatch helpers | Notification subscribers in `notify.rs` |
| state.rs: 3,195 LOC | state.rs: ~535 LOC |

## What Stays

- `Command` enum (unchanged)
- `QueryContext` / `TestContext` (unchanged, may need field expansion)
- DES scenarios (updated to assert on Mutations instead of Effects)
- Trace system (records Mutations instead of Effects)
- Bridge subsystems: AI, storm, status stay as `TickSubsystem` bridges initially

## Stage 1 Review Notes

These are known deviations in `apply_one` that are correct for Stage 1 but need attention when systems produce `Mutation` directly in Stage 3:

- **`SetEquipment`** — does `equipment.set()` + pushes old item to inventory + calls `recalc_equipment_stats()`. Three mutations in one. When systems emit `SetEquipment` directly, they must also emit `AddToInventory` (for the displaced item) and a `RecalcStats` mutation separately.
- **`SpawnItemOnMap`** — calls `rebuild_spatial_index()` inside `apply_one`. In the final design, spatial rebuild is a derive that runs after all cascades settle, not inside apply_one. Acceptable for Stage 1.
- **`RemoveEnemy`** — removes from spatial index and records in meta, but does not remove the enemy object from `world.enemies`. The enemy stays with `hp ≤ 0`. This matches pre-existing behavior (the old `Kill` apply arm did the same). Not a bug.

## Stage 3 Review Notes

- **`UsePsychicAbility` / `AttemptFlee`** — these are bridge mutations that call imperative logic inside `apply_one` rather than returning mutations directly. This is intentional for Stage 3: `use_ability` mutates psychic cooldown state and `attempt_flee` needs `&mut enemies`. Both should be converted to pure system functions returning `Vec<Mutation>` in Stage 4 when `PsychicState` and `EncounterState` are decomposed.
- **rng writeback in `route_command`** — all command handlers that take `&mut ChaCha8Rng` must clone-call-writeback to advance `state.rng`. The pattern is: `let mut rng = state.rng.clone(); let m = handler(..., &mut rng); state.rng = rng;`. Any handler that skips the writeback silently breaks determinism. Fixed in Stage 3 post-review (Attack arm was missing writeback).

## Migration Stages

### Stage 1: Introduce mutations.rs alongside existing effects

Add `Mutation` enum, `StateTransition` enum, `SubsystemId` to `mutations.rs`. Add `apply_one()` to GameState. Have existing apply arms in `effects/apply.rs` delegate to `apply_one()` internally where possible. This proves the verification works without changing any system behavior.

Files created: `mutations.rs`
Files modified: `state.rs` (add apply_one), `effects/apply.rs` (delegate to apply_one)
Files deleted: none

**Verify:** `cargo test`, all DES pass. Zero behavior change.

### Stage 2: Add dispatch.rs and notify.rs, convert combat end-to-end

Create `dispatch.rs` with `dispatch()`, `route_command()`, `apply_with_cascade()`, `apply_recursive()`. Create `notify.rs` with `on_transitions()` and handlers for `EnemyHpChanged` and `EnemyHpReachedZero`.

Convert combat: create `systems/combat.rs` with `handle_melee()` and `handle_ranged()` returning `Vec<Mutation>`. Add `on_enemy_hit()` and `on_enemy_killed()` as notification handlers. Route `Command::Attack` and `Command::RangedAttack` through the new path. Other commands still use the old `dispatch()` on GameState.

Files created: `dispatch.rs`, `notify.rs`, `systems/combat.rs` (new version)
Files modified: `state.rs` (dispatch delegates to dispatch.rs for combat commands)
Files deleted: none yet (old dispatch_melee_attack still exists for other callers)

**Verify:** `cargo test`, combat DES scenarios pass. New combat unit tests for `handle_melee → Vec<Mutation>`.

### Stage 3: Convert remaining systems (simplest first)

Each conversion follows the same pattern:
1. Create system function in `systems/X.rs` returning `Vec<Mutation>`
2. Add command routing in `dispatch.rs::route_command()`
3. Add transition handlers to `notify.rs` if needed
4. Delete old `dispatch_*` method from state.rs
5. `cargo test`

Order:
1. **player** (wait, rest, allocate_stat, psychic, flee) — no notifications
2. **items** (equip, unequip, use_item, craft, buy, sell) — `ItemAddedToInventory` notification
3. **quest** (accept, complete) — no notifications
4. **interact** (interact, examine) — no notifications
5. **movement** — `PlayerPositionChanged` notification (triggers FOV, lighting, pickup, world transition, adaptation)
6. **world** (world_move, world_move_safe, subterranean, pathing) — `PlayerEnteredWorldTile` notification
7. **turn** (end_turn, phase execution) — `TurnAdvanced` notification

### Stage 4: Convert bridge subsystems

AI, storm, status tick logic stays as `TickSubsystem(SubsystemId)` mutations initially. The `apply_one` arm for `TickSubsystem` calls the legacy system code. These can be decomposed into atomic mutations later when/if those systems need modification.

### Stage 5: Delete old infrastructure

- Delete `effects/apply.rs`
- Delete old Effect enums (PlayerEffect, CombatEffect, ItemEffect, MapEffect, ResourceEffect, EventEffect, QuestEffect)
- Delete old `dispatch()` method body from state.rs (replaced by dispatch.rs)
- Delete `collect_reactions`, `run_reactions`, `apply_and_trace` from state.rs
- Delete `rules/` directory (logic absorbed into `systems/`)
- Update DES to assert on Mutations instead of Effects
- Update trace to record Mutations

**Verify:** `cargo test`, all DES pass, `cargo clippy` clean.

## Current Post-Processing → Notification Mapping

These implicit behaviors in dispatch helpers become explicit notification handlers:

| Current (in dispatch helper) | Transition trigger | Notification handler |
|-----|-----|-----|
| `update_fov()`, `update_lighting()` | Any mutation batch | Derives — run after cascade settles, not a notification |
| `MovementSystem::pickup_items()` | `PlayerPositionChanged` | `items::on_player_moved()` |
| `MovementSystem::handle_world_transition()` | `PlayerPositionChanged` | `world::on_player_moved()` |
| `check_adaptation_threshold()` | `PlayerPositionChanged` | `player::on_position_changed()` |
| `check_auto_end_turn()` | `PlayerApReachedZero` | `turn::on_ap_exhausted()` |
| `trigger_hit_flash()`, `spawn_damage_number()` | `EnemyHpChanged` | Presentation mutations in notify.rs |
| `CombatSystem::trigger_swarm_aggro()` | `EnemyHpChanged` | `combat::on_enemy_hit()` |
| Reflect damage, on-hit effects | `EnemyHpChanged` | `combat::on_enemy_hit()` |
| `CombatSystem::process_enemy_death_post()` | `EnemyHpReachedZero` | `combat::on_enemy_killed()` |
| `LootSystem` loot drop | `EnemyHpReachedZero` | `loot::on_enemy_killed()` |
| Quest kill tracking | `EnemyHpReachedZero` | `quest::on_enemy_killed()` |
| Quest collect tracking | `ItemAddedToInventory` | `quest::on_item_collected()` |
| Subsystem ticks | `TurnAdvanced` | `turn::on_turn_advanced()` |
| Tile generation, encounter check | `PlayerEnteredWorldTile` | `world::on_entered_world_tile()` |

## Risks

| Risk | Mitigation |
|------|-----------|
| Large migration surface — every system changes | One system per stage, verify after each |
| DES scenarios break when Effects → Mutations | Update DES in Stage 5, keep old path working until then |
| Notification cascades cause unexpected behavior | Depth limit 10, same as current reactions |
| QueryContext needs expansion for new systems | Add fields as needed during each system conversion |
| Bridge subsystems (AI, storm) are complex | Stay as TickSubsystem bridges, don't force conversion |

## Done When

- [ ] state.rs ≤ 600 LOC
- [ ] Zero system logic in state.rs
- [ ] Zero notification logic in state.rs
- [ ] Every mutation goes through `apply_one()` with verification
- [ ] `notify.rs` is the single place for all cross-system reactions
- [ ] `cargo test` passes, all DES scenarios pass
- [ ] Old Effect enums deleted
- [ ] Old `effects/apply.rs` deleted
