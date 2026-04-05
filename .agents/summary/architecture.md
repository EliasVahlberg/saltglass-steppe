# Architecture

## Overview

Saltglass Steppe is a monolithic Rust application organized in layers. The codebase is actively migrating from imperative state mutation to the VERA (Verified Effect-Rule Architecture) pattern. Both patterns coexist during migration.

## Layer Architecture

```mermaid
graph TB
    subgraph "Input Layer"
        TUI["TUI Input (main.rs)"]
        DES["DES Interpreter (des/mod.rs)"]
    end

    subgraph "Dispatch Layer"
        CMD["Command Enum"]
        DISP["dispatch() in state.rs"]
    end

    subgraph "Rule Layer (VERA)"
        RI["rules/item.rs"]
        RM["rules/movement.rs"]
        RC["rules/combat.rs"]
        RA["rules/actions.rs"]
        RT["rules/turn.rs"]
    end

    subgraph "Effect Layer (VERA)"
        EFF["Effect Enums (effects/mod.rs)"]
        APP["apply_effect (effects/apply.rs)"]
        TRC["Trace (effects/trace.rs)"]
    end

    subgraph "Legacy Systems"
        AI["systems/ai.rs"]
        STM["systems/storm.rs"]
        MOV["systems/movement.rs"]
        LOT["systems/loot.rs"]
        STS["systems/status.rs"]
    end

    subgraph "State Layer"
        GS["GameState (state.rs)"]
        PS["PlayerState"]
        WS["WorldState"]
    end

    subgraph "Rendering Layer (read-only)"
        REN["Renderer"]
        TILE["Tile Renderer"]
        ENT["Entity Renderer"]
        LIT["Lighting Renderer"]
        PART["Particle System"]
    end

    subgraph "Data Layer"
        JSON["data/*.json"]
        SCH["schemas/*.json"]
        DL["DataLoader"]
    end

    TUI --> CMD
    DES --> CMD
    CMD --> DISP
    DISP --> RI & RM & RC & RA & RT
    RI & RM & RC & RA & RT --> EFF
    EFF --> APP
    APP --> GS
    APP --> TRC
    DISP -.->|bridge effects| AI & STM & STS
    GS --> PS & WS
    GS --> REN
    DL --> JSON
    DL --> SCH
```

## VERA Pattern (Current State)

The VERA migration converts imperative `GameState` methods into a three-step pipeline:

```mermaid
sequenceDiagram
    participant Input
    participant Dispatch
    participant Rule
    participant Apply
    participant State
    participant Trace

    Input->>Dispatch: Command enum
    Dispatch->>Rule: rule_fn(args, &QueryContext, &mut rng)
    Rule-->>Dispatch: RuleOutput { effects, presentation }
    loop For each effect
        Dispatch->>Apply: apply_effect(&effect)
        Apply->>State: Mechanical field mutation
        Dispatch->>Trace: record(effect, source, turn)
    end
    Dispatch->>Dispatch: run_reactions(effects)
    Dispatch->>State: update_fov(), update_lighting()
```

### Migration Status

Systems are in three states:

1. **Fully migrated** (pure rules): item use, movement, combat (melee/ranged), player actions (wait, rest, equip, etc.), world travel
2. **Bridge effects** (traced but calling legacy code): AI system, storm system, status effects, subsystem ticks (psychic, skills, light, void, crystal)
3. **Legacy** (not yet traced): save/load, NPC dialogue, trading, crafting

See `docs/development/SYSTEM_STATUS.md` for the authoritative status of each system.

## Turn Processing

End-of-turn executes a fixed phase sequence:

```mermaid
graph LR
    A[ResetAp] --> B[TickStatusEffects]
    B --> C[TickSubsystems]
    C --> D[AdvanceTurn]
    D --> E[RunAI]
    E --> F[TickStorm]
    F --> G[AdvanceTime]
    G --> H[UpdateDerives]
    H --> I[CheckEncounters]
```

All phases except UpdateDerives (FOV/lighting recalc) produce traced effects.

## Procedural Generation Pipeline

```mermaid
graph TB
    WG["world_gen.rs<br/>World map generation"] --> TG["tile_generator.rs<br/>Tile map orchestrator"]
    TG --> TFA["terrain_forge_adapter.rs<br/>Base terrain via terrain-forge"]
    TFA --> CON["connectivity.rs<br/>Glass Seam Bridging"]
    CON --> SL["structure_library.rs<br/>Stamp prefab structures"]
    SL --> MS["microstructures.rs<br/>Small features"]
    MS --> EP["environmental_props.rs<br/>Decorations"]
    EP --> SP["spawn.rs<br/>Entity population"]
    SP --> FM["feature_materializer.rs<br/>Story hooks, NPCs, loot"]
    FM --> QC["quest_constraints.rs<br/>Validate quest requirements"]
```

## Data Flow

All game content is data-driven via JSON files validated against schemas at load time:

```mermaid
graph LR
    JSON["data/*.json"] --> DL["DataLoader<br/>(generic, cached)"]
    SCH["schemas/*.json"] --> DL
    DL --> GS["GameState"]
    DL --> GEN["Generation Pipeline"]
    DL --> DES["DES Interpreter"]
```

## Multi-Terminal Architecture

The game supports satellite terminal windows via IPC:

```mermaid
graph TB
    MAIN["Main Game Process"] -->|Unix Socket| LOG["Log UI Terminal"]
    MAIN -->|Unix Socket| STAT["Status UI Terminal"]
    MAIN -->|Unix Socket| INV["Inventory UI Terminal"]
```

## Key Design Decisions

- **God object pattern**: `GameState` in `state.rs` is the central hub. All systems access state through it. This is intentional — it's the coordination point.
- **Deterministic RNG**: All randomness uses `ChaCha8Rng` with explicit seeds. Same seed = same gameplay.
- **DES over manual testing**: The Debug Execution System enables headless, deterministic gameplay testing via JSON scenarios.
- **Bridge pattern for migration**: Deeply coupled systems (AI, storm) use bridge effects that call existing code while providing trace visibility.
- **Reactions replace events**: The old `GameEvent` system was replaced by VERA reactions (Batch F). Kill → loot drop → quest progress is now a reaction chain.
