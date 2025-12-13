# Tech Stack Recommendation

## Language: Rust

Why Rust fits this project:
- Determinism is a core requirement (seeded RNG, reproducible storms/procgen) — Rust's strict ownership model prevents hidden state bugs
- Performance-critical systems: FOV, pathfinding, storm map-edits, AI scheduling
- Strong serialization ecosystem (serde) for save/load with versioning
- No GC pauses during gameplay
- Excellent TUI libraries

## TUI Framework: Ratatui

- Active community, well-documented
- Flexible layout system for multi-panel UI (map + log + forecast + inventory)
- Supports color/styling for shimmer overlays, glare tiles, targeting lines
- Pairs with `crossterm` for cross-platform terminal input

## Data Format: RON

- Factions, items, enemies, mutations, encounter tables as data files
- RON (Rusty Object Notation) integrates seamlessly with serde
- Alternative: TOML for more human-readable content

## Key Libraries

| Purpose | Library |
|---------|---------|
| TUI rendering | `ratatui` + `crossterm` |
| Serialization | `serde` + `ron` |
| RNG (seedable) | `rand` + `rand_chacha` |
| Pathfinding | `pathfinding` |
| FOV | `bracket-lib` (FOV module) or custom |
| ECS (optional) | `hecs` or `specs` |
| Noise/procgen | `noise` or `fastnoise-lite` |

## Architecture

```
┌─────────────────────────────────────────┐
│              UI Layer (ratatui)         │
│  (reads state, sends input events)      │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│           Game Simulation               │
│  (turn loop, ECS/component store,       │
│   deterministic RNG, event queue)       │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│         Data Layer (RON/TOML)           │
│  (factions, items, mutations, maps)     │
└─────────────────────────────────────────┘
```

## Alternatives Considered

| Alternative | Tradeoff |
|-------------|----------|
| Python + blessed/curses | Too slow for large maps, harder determinism |
| C++ | Slower iteration, manual memory management |
| Go | Weaker type system, less mature TUI ecosystem |
| bracket-lib (full) | Opinionated; better to use pieces selectively |

## Minimal Cargo.toml

```toml
[dependencies]
ratatui = "0.28"
crossterm = "0.28"
serde = { version = "1", features = ["derive"] }
ron = "0.8"
rand = "0.8"
rand_chacha = "0.3"
```
