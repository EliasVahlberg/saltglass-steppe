# Copilot Instructions for saltglass-steppe

## Project Overview
Rust-based turn-based TUI RPG ("Saltglass Steppe") using `ratatui`/`crossterm`. Deterministic seeded RNG (`rand_chacha`) enables reproducible gameplay and automated testing.

## Architecture

### Core Separation
- **`src/game/`**: Pure game logic (no UI deps). `state.rs` contains `GameState` (~1200 lines) - the heart of game simulation.
- **`src/ui/`**: Rendering and input handling. `UiState` is separate from `GameState` to enable headless testing.
- **`src/des/`**: Debug Execution System - headless JSON-driven test framework for automated testing.
- **`src/main.rs`**: Glue layer - game loop, terminal setup, connects `GameState` ↔ `UiState`.

### Data-Driven Content
All game content lives in `data/*.json` - items, enemies, NPCs, weapons, quests, effects. Adding content usually requires no code changes:
```
data/items.json    → ItemDef loaded via get_item_def()
data/enemies.json  → EnemyDef loaded via get_enemy_def()
data/npcs.json     → NpcDef loaded via get_npc_def()
data/effects.json  → Visual effects (B=Blink, G=Glow syntax)
```
Static data uses `once_cell::Lazy` with `include_str!` for compile-time embedding.

### Key Patterns
- **Module re-exports**: `src/game/mod.rs` and `src/ui/mod.rs` consolidate public APIs
- **Effect DSL**: `"B(@3 &Cyan)"` = Blink every 3 frames, Cyan color (see `src/game/effect.rs`)
- **Tile types**: `Tile::Wall { id, hp }` - walls are destructible with typed variants

## Commands

```bash
cargo run                        # Run game
cargo test                       # All tests (unit + DES scenarios)
cargo test --test des_scenarios  # DES integration tests only
cargo test basic_movement_scenario  # Single scenario
./build-windows.sh              # Cross-compile for Windows (creates .zip with data/)
```

## Debug Execution System (DES)

DES enables automated testing without rendering. Scenarios in `tests/scenarios/*.json`:

```json
{
  "name": "test_name",
  "seed": 42,
  "player": { "x": 10, "y": 10, "inventory": ["brine_vial"] },
  "entities": [{ "entity_type": "enemy", "id": "salt_crawler", "x": 12, "y": 10 }],
  "actions": [
    {"turn": 0, "action": {"type": "move", "dx": 1, "dy": 0}},
    {"turn": 1, "action": {"type": "attack", "target_x": 12, "target_y": 10}}
  ],
  "assertions": [{ "at_end": true, "check": {"type": "player_alive"} }]
}
```

Scenarios can inherit from `BASE_*.json` files using `"base": "BASE_combat.json"`.

Use `"mocks": { "combat_always_hit": true }` to force deterministic combat outcomes.

## Adding Content

### New Item
1. Add entry to `data/items.json` with required fields: `id`, `name`, `glyph`, `description`, `value`, `weight`, `usable`
2. Optional: `heal`, `reveals_map`, `effects` array
3. Test with look mode (`x` key in-game)

### New Enemy
1. Add to `data/enemies.json`: `id`, `name`, `glyph`, `max_hp`, `damage_min/max`, `sight_range`, `xp_value`
2. Optional: `behaviors` array, `spawns_during_storm`, `effects`

### New Test Scenario
Create `tests/scenarios/your_test.json` - automatically picked up by `run_all_scenarios`.

## Code Conventions
- Game logic in `src/game/` must not import `ratatui` or `crossterm`
- All randomness via `ChaCha8Rng` from `GameState.rng` (serializable)
- Effect durations in **turns** not frames (decoupled from render loop)
- Use `state.log("message")` for game log, `state.log_typed(msg, MsgType::Combat)` for colored logs
