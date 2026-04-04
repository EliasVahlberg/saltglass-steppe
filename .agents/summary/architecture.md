# architecture.md

## System Overview

```mermaid
graph TB
    subgraph Entry["Entry Points"]
        Main["main.rs — TUI Game Loop"]
        MapGen["mapgen_tool.rs — CLI"]
        DES["des/mod.rs — Headless Testing"]
    end

    subgraph UI["UI Layer — src/ui/"]
        Input["input.rs"]
        GameView["game_view.rs"]
        HUD["hud.rs"]
        Menus["Menus (~15 modules)"]
    end

    subgraph Renderer["Renderer — src/renderer/"]
        Tiles["tiles.rs"]
        Entities["entities.rs"]
        Lighting["lighting.rs"]
        Particles["particles.rs"]
    end

    subgraph Core["Game Simulation — src/game/"]
        State["state.rs — GameState"]
        Systems["systems/"]
        Content["combat, quest, item, npc,<br/>enemy, dialogue, skills,<br/>storm, light, adaptation, ..."]
    end

    subgraph Gen["Procedural Generation"]
        TFA["terrain_forge_adapter.rs"]
        Conn["connectivity.rs (GSB)"]
        Struct["structure_library.rs"]
        Micro["microstructures.rs"]
        EnvProps["environmental_props.rs"]
        Spawn["spawn.rs"]
    end

    subgraph Data["Data Layer"]
        JSON["data/*.json"]
        Schemas["schemas/*.json"]
        DataLoader["DataLoader&lt;T&gt;"]
    end

    Main --> UI --> State
    Main --> Renderer --> State
    DES --> State
    MapGen --> Gen
    State --> Systems
    State --> Gen
    DataLoader --> JSON
    DataLoader --> Schemas
```

## Central State Hub

`GameState` in `state.rs` is the single source of truth. Key sub-structs:

| Struct | Responsibility |
|--------|---------------|
| `PlayerState` | Position, HP, inventory, equipment, skills, status effects |
| `WorldState` | World map, current tile map, biome data, discovered locations |
| `NarrativeEngine` | Quest state, dialogue tracking, event history |

All systems read/write through `GameState`. When adding features, you almost certainly need to touch `state.rs`.

## ECS-Style Systems

Systems in `src/game/systems/` are decoupled processors that operate on `GameState`:

```mermaid
graph LR
    Event["GameEvent"] --> Systems
    subgraph Systems["src/game/systems/"]
        AI["ai.rs"]
        Combat["combat.rs"]
        Movement["movement.rs"]
        Storm["storm.rs"]
        Status["status.rs"]
        Loot["loot.rs"]
        Quest["quest.rs"]
    end
    Systems --> State["GameState mutation"]
```

Systems communicate via `GameEvent` dispatched through the shared `GameState`. No direct system-to-system calls.

## Generation Pipeline

Tile generation follows a fixed pipeline order:

```mermaid
graph LR
    A["terrain-forge<br/>(base terrain)"] --> B["connectivity.rs<br/>(Glass Seam Bridging)"]
    B --> C["structure_library.rs<br/>(prefab stamps)"]
    C --> D["microstructures.rs<br/>(small features)"]
    D --> E["environmental_props.rs<br/>(props & decor)"]
    E --> F["spawn.rs<br/>(entities)"]
```

- **terrain_forge_adapter.rs** bridges the `terrain-forge` crate to game tile types, driven by biome profiles in `data/biome_profiles.json`
- **connectivity.rs** implements Glass Seam Bridging (GSB) — a novel algorithm ensuring all walkable regions connect. Documented in `docs/papers/`
- **constraints.rs** and **quest_constraints.rs** validate post-generation requirements

## Data-Driven Architecture

```mermaid
graph LR
    Rust["Rust structs<br/>(serde + schemars)"] -->|schema_gen| Schema["schemas/*.json"]
    JSON["data/*.json"] -->|validated against| Schema
    JSON -->|loaded by| DL["DataLoader&lt;T&gt;"]
    DL -->|cached via| OC["once_cell::Lazy"]
    DL --> State["GameState"]
```

Cross-reference rules: items ↔ traders, loot_tables, recipes, quests. Enemies ↔ biome_spawn_tables, loot_tables. Adding data entries requires checking all referencing files.

## Deterministic RNG

All RNG uses `ChaCha8Rng` from `rand_chacha` with explicit seeds. `RngState` is serialized with saves. This guarantees: same seed → same world, same combat outcomes, same loot drops.

## Multi-Terminal IPC

```mermaid
graph LR
    Main["Main Game"] -->|Unix domain socket| Log["Log Terminal"]
    Main -->|Unix domain socket| Status["Status Terminal"]
    Main -->|Unix domain socket| Inv["Inventory Terminal"]
```

`src/ipc.rs` handles socket communication. `src/satellite.rs` runs satellite terminal processes. `src/terminal_spawn.rs` detects and launches terminal emulators.

## Key Architectural Decisions

1. **No ECS framework** — custom systems pattern keeps dependencies lean and code greppable
2. **terrain-forge for base terrain** — adapted via `terrain_forge_adapter.rs` with biome-specific profiles
3. **Glass Seam Bridging** for connectivity — novel algorithm, not a standard flood-fill approach
4. **Schema validation at load time** — all `data/*.json` validated against auto-generated schemas; run `schema_gen` after changing Rust data types
5. **DES for gameplay testing** — headless scenario execution replaces manual TUI testing in CI
