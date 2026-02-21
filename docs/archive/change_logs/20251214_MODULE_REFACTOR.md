# Module Structure Refactor

**Date:** 2025-12-14

## Summary
Refactored monolithic `lib.rs` into modular `src/game/` directory structure following single-responsibility principle.

## Before
```
src/
├── lib.rs    # ~450 lines, all game logic
└── main.rs
```

## After
```
src/
├── lib.rs              # Re-exports + tests
├── main.rs             # TUI rendering
└── game/
    ├── mod.rs          # Module declarations + public API
    ├── constants.rs    # MAP_WIDTH, MAP_HEIGHT, FOV_RANGE
    ├── map.rs          # Tile, Map, compute_fov, pathfinding traits
    ├── enemy.rs        # EnemyKind, Enemy
    ├── storm.rs        # Storm
    ├── adaptation.rs   # Adaptation
    ├── item.rs         # ItemKind, Item
    └── state.rs        # GameState (game logic, save/load)
```

## Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `constants` | Configuration values |
| `map` | Terrain, FOV, pathfinding |
| `enemy` | Enemy types and data |
| `storm` | Storm mechanics |
| `adaptation` | Player mutations |
| `item` | Item types and data |
| `state` | Game state and logic |

## Public API
Unchanged - all types re-exported through `lib.rs`:
- `tui_rpg::GameState`
- `tui_rpg::Tile`
- `tui_rpg::Enemy`
- `tui_rpg::Item`
- etc.

## Files Modified
- `src/lib.rs` (reduced to re-exports + tests)
- `src/game/mod.rs` (new)
- `src/game/constants.rs` (new)
- `src/game/map.rs` (new)
- `src/game/enemy.rs` (new)
- `src/game/storm.rs` (new)
- `src/game/adaptation.rs` (new)
- `src/game/item.rs` (new)
- `src/game/state.rs` (new)
