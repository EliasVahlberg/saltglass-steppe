---
status: stale
last_verified: 2026-04-04
commit: e0d1fe7
stale_reason: "LOC counts outdated; 27 dead methods removed; line ranges shifted"
---

> ⚠️ **STALE DOCUMENT** — This document may not accurately reflect the current codebase.
> Reason: LOC counts outdated; 27 dead methods removed; line ranges shifted
> Last verified: 2026-04-04

# GameState Guide

`src/game/state.rs` is the central hub of the game. All gameplay state lives here and every system reads/writes through it. This guide explains its organization so you can navigate and extend it safely.

## Struct Layout

`GameState` fields fall into four categories:

### Persisted State (serialized to save files)
- `player: PlayerState` — position, HP, AP, inventory, equipment, skills, adaptations, currency
- `world: WorldState` — map, world_map, enemies, NPCs, items, chests, interactables, storm, weather, time
- `visible` / `revealed` — FOV sets (tile indices)
- `messages` — game log
- `turn` — current turn counter
- `rng` — ChaCha8 RNG (serialized via custom `rng_serde` module)
- `narrative` — NarrativeEngine (quest log, story model, tutorial)
- `map_features` — hidden locations, safe routes, waypoints
- `seed` — original world seed

### Spatial Indices (`#[serde(skip)]`, rebuilt on load)
- `enemy_positions`, `npc_positions`, `item_positions`, `chest_positions`, `interactable_positions` — HashMap lookups by (x,y)
- `spatial_dirty` — dirty flag; call `mark_spatial_dirty()` after moving entities
- Rebuilt via `rebuild_spatial_index()` on load or when dirty

### Transient UI State (`#[serde(skip)]`)
- `pending_trade`, `pending_dialogue`, `pending_aria_dialogue`, `pending_book_open` — UI triggers set by game logic, consumed by UI layer
- `event_queue` — `GameEvent` queue processed each turn

### Debug/Test State (`#[serde(skip)]`)
- `debug_god_view`, `debug_phase`, `debug_disable_glare` — debug toggles
- `mock_combat_hit`, `mock_combat_damage` — DES testing mocks
- `test_mode` — suppresses side effects during tests

## Method Organization (by line range)

The file's ~3500 lines are grouped into logical sections:

| Lines | Section | Key Methods |
|-------|---------|-------------|
| 101–172 | Struct definition | Field declarations |
| 173–585 | `new()` constructor | World generation, initial spawn, starting inventory |
| 586–634 | Spatial indexing | `mark_spatial_dirty()`, `rebuild_spatial_index()` |
| 636–1388 | World travel & map transitions | `travel_to_tile()`, `move_on_world_map()`, `enter_subterranean()`, `exit_subterranean()`, encounters |
| 1390–1447 | Lighting & FOV | `update_lighting()`, `update_fov()` |
| 1449–1605 | Narrative generation | `generate_*()` text methods, `get_*()` lore methods |
| 1607–1676 | Time, light, events | `tick_time()`, `get_light_level()`, `emit()`, `drain_events()` |
| 1678–1767 | Progression | `gain_xp()`, `allocate_stat()`, `end_turn()` |
| 1769–1860 | Event processing | `process_events()`, `handle_event()` |
| 1862–1975 | Turn actions | `apply_status()`, `wait_turn()`, `rest()`, `tick_turn()` |
| 1977–2112 | Interaction | `log()`, `interact_at()`, `examine_at()`, `debug_command()` |
| 2114–2280 | Visual effects & adaptations | `trigger_hit_flash()`, `spawn_damage_number()`, `get_adaptation_visual_effects()`, `check_adaptation_threshold()` |
| 2282–2542 | Entity queries & movement | `enemy_at()`, `npc_at()`, `auto_explore()`, `try_move()`, `pickup_items()` |
| 2544–2855 | Inventory & items | `open_chest()`, `use_item()`, `use_item_on_tile()`, `use_psychic_ability()` |
| 2895–3008 | Equipment & quests | `equip_item()`, `accept_quest()`, `complete_quest()` |
| 3010–3199 | Economy | `craft()`, `buy_item()`, `sell_item()`, `calculate_price()` |
| 3201–3335 | Status effects & AI | `apply_status_effect()`, `process_enemy_behavior()`, `apply_light_effects()` |
| 3337–3524 | Save/load & accessors | `save()`, `load()`, all `player_*()` / `map()` / `enemies()` accessors, `load_test_tile()` |

## How to Extend GameState

### Adding a new field

1. Add the field to the `GameState` struct with `#[serde(default)]` if it needs to be backward-compatible with existing saves
2. Initialize it in `new()`
3. If it's transient, add `#[serde(skip)]`
4. If it needs spatial lookup, add a position HashMap and update `rebuild_spatial_index_internal()`
5. If save version changes are needed, bump `SAVE_VERSION` in `save.rs` and add a migration case

### Adding a new action

1. Add the method to `GameState` in the appropriate section (see table above)
2. If it costs AP, call `action_cost()` and deduct from `player.ap`
3. If it modifies entity positions, call `mark_spatial_dirty()`
4. If it should trigger system reactions, call `emit(GameEvent::...)`
5. Wire it up in `src/ui/input.rs` for keyboard handling

### Adding a new system

Systems live in `src/game/systems/`. Each implements the `System` trait:
```rust
pub trait System {
    fn update(&self, state: &mut GameState);
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
```
Register the new system in `src/game/systems/mod.rs` and call it from the turn processing in `end_turn()` or `process_events()`.

## Common Patterns

### Spatial index usage
```rust
// Always mark dirty after moving entities
state.mark_spatial_dirty();
// Index is lazily rebuilt on next access via ensure_spatial_index()
```

### Event flow
```rust
state.emit(GameEvent::EnemyKilled { index });
// Events are drained and processed in process_events()
// Systems react via on_event()
```

### Mock combat (DES testing)
```rust
// Set by DES executor before running scenarios
state.mock_combat_hit = Some(true);   // Force all attacks to hit
state.mock_combat_damage = Some(10);  // Fixed damage amount
```

## Gotchas

- **Don't hold references across mutations**: Entity vectors can be reindexed. Use indices, not references.
- **Spatial index staleness**: Always call `mark_spatial_dirty()` after moving/removing entities. The index rebuilds lazily.
- **Save compatibility**: New fields need `#[serde(default)]` or a save migration. Test with `save_load_roundtrip_preserves_state`.
- **RNG determinism**: All randomness must go through `self.rng`. Never use `thread_rng()` or other unseeded sources.
