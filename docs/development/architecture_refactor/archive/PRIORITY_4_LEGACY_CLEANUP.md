# Priority 4: Legacy Method Cleanup

> Effort: 1 day
> Impact: state.rs shrinks, single dispatch entry point enforced
> Files: `src/game/state.rs`, callers in `src/des/mod.rs`, `src/ui/`, `src/main.rs`

## Problem

Several legacy methods still exist on GameState alongside their VERA equivalents. External code can bypass dispatch by calling the legacy method directly. state.rs is 3,680 LOC — larger than pre-refactor (3,185) — partly because old methods weren't deleted.

## Legacy methods to delete

These have VERA equivalents via dispatch but the old methods still exist:

| Method | VERA equivalent | Callers to update |
|--------|----------------|-------------------|
| `gain_xp()` | `PlayerEffect::GainXp` apply arm | Check DES, combat post-processing |
| `allocate_stat()` | `Command::AllocateStat` | Check UI input handler |
| `wait_turn()` | `Command::Wait` | Check UI input handler, DES |
| `rest()` | `Command::Rest` | Check UI input handler, DES |
| `apply_status()` | `PlayerEffect::ApplyStatusEffect` apply arm | Check combat, status system |

## Dispatch passthroughs to convert

These go through dispatch but immediately call legacy methods without producing effects:

| Command | Currently calls | Action |
|---------|----------------|--------|
| `Interact { x, y }` | `self.interact_at(x, y)` | Convert to rule or dispatch helper with effects |
| `Examine { x, y }` | `self.examine_at(x, y)` | Convert to rule or dispatch helper with effects |
| `EnterSubterranean` | `self.enter_subterranean()` | Convert to dispatch helper with effects |
| `ExitSubterranean` | `self.exit_subterranean()` | Convert to dispatch helper with effects |

For each passthrough:
1. Identify what state the method mutates
2. Express those mutations as effects
3. Move the decision logic to a rule or dispatch helper
4. Delete the legacy method

## Process for each deletion

1. `grep -rn "method_name" src/` — find all callers
2. Update each caller to use `dispatch(Command::...)` instead
3. Delete the method from state.rs
4. `cargo build` — compiler catches any missed callers
5. `cargo test` — verify no regressions

## Expected outcome

- state.rs drops by ~200-300 LOC
- No public method on GameState mutates state outside dispatch/apply (except derives: update_fov, update_lighting, rebuild_spatial_index)
- The rule "if it changes GameState, it goes through dispatch" is enforced by the API surface

## Verify

`cargo test`, `cargo clippy -- -D warnings`, all DES scenarios pass. Grep for deleted method names confirms zero remaining callers.
