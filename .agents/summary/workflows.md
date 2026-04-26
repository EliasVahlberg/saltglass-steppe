# Workflows

<!-- Generated: 2026-04-06 | tags: workflows, processes, pipelines -->

## Game Loop

```mermaid
sequenceDiagram
    participant M as main.rs
    participant S as session.rs
    participant UI as ui/input.rs
    participant GS as GameState
    participant R as Renderer

    M->>S: run_game_session()
    loop Every frame
        S->>UI: handle_input(event)
        UI->>GS: dispatch(Command)
        Note over GS: apply_with_cascade
        S->>R: render_game(state)
        R-->>S: Frame buffer
    end
```

The main loop in `main.rs` runs at configurable FPS (default 60). Each frame: poll input → dispatch commands → render. The game is turn-based — rendering happens every frame but state only changes on player actions.

## Command Dispatch Flow

```mermaid
flowchart TD
    INPUT[Key Press] --> HANDLER[input.rs handler]
    HANDLER --> CMD[Command variant]
    CMD --> DISPATCH[dispatch.rs::route_command]
    DISPATCH --> SYSTEM[System handler]
    SYSTEM --> MUTS[Vec of Mutation]
    MUTS --> APPLY[state.apply_mutations]
    APPLY --> TRANS[Vec of StateTransition]
    TRANS --> NOTIFY[notify::on_transitions]
    NOTIFY --> REACT[Reactive mutations]
    REACT -->|depth < 10| APPLY
    APPLY --> DERIVES[update_fov + update_lighting]
```

## Combat Flow

```mermaid
sequenceDiagram
    participant P as Player Input
    participant D as dispatch.rs
    participant C as combat system
    participant S as state
    participant N as notify.rs
    participant L as loot system

    P->>D: Command::Attack
    D->>C: handle_melee(query, rng)
    C-->>D: [SetEnemyHp, SpendAp, HitFlash, DamageNumber]
    D->>S: apply_mutations
    S-->>D: [EnemyHpChanged] or [EnemyHpReachedZero]
    D->>N: on_transitions
    alt Enemy HP changed
        N->>C: on_enemy_hit → swarm aggro, reflect
    end
    alt Enemy killed
        N->>C: on_enemy_killed → split-on-death
        N->>L: reaction_loot_drop → SpawnItemOnMap
    end
    N-->>D: reactive mutations
    D->>S: apply_mutations (cascade)
```

## Turn Processing Flow

```mermaid
flowchart LR
    ET[EndTurn mutation] --> P1[ResetAp]
    P1 --> P2[TickStatusEffects]
    P2 --> P3["TickSubsystems (psychic, skills, light, void, crystal)"]
    P3 --> P4[AdvanceTurn]
    P4 --> P5["RunAI (all enemies act)"]
    P5 --> P6["TickStorm (map edits, wraith spawns)"]
    P6 --> P7["AdvanceTime (time_of_day, weather)"]
    P7 --> P8["UpdateDerives (FOV, lighting)"]
    P8 --> P9["CheckEncounters (overworld only)"]
```

Each phase produces mutations that are applied immediately. AI phase runs all enemies sequentially with spatial index rebuilds between actions.

## Tile Generation Pipeline

```mermaid
flowchart TD
    TRAVEL[travel_to_tile] --> PARAMS[TileParams from world state]
    PARAMS --> TILEGEN[tile_generator::generate_tile]
    TILEGEN --> TERRAIN[terrain_forge_adapter — algorithm selection + layering]
    TERRAIN --> CONNECT[connectivity.rs — Glass Seam Bridging]
    CONNECT --> STRUCT[structure_library — prefab placement]
    STRUCT --> MICRO[microstructures — small structures]
    MICRO --> PROPS[environmental_props — decorations]
    PROPS --> SPAWN[spawn.rs — enemies, items, NPCs]
    SPAWN --> FEAT[feature_materializer — story hooks, interactables]
    FEAT --> QUEST[quest_constraints — validation]
    QUEST --> MAP[Generated Map]

    SETTLE{POI == Town?}
    TILEGEN --> SETTLE
    SETTLE -->|Yes| TOWN[settlement/ — buildings, roads, NPCs]
    TOWN --> MAP
```

## World Travel Flow

```mermaid
sequenceDiagram
    participant P as Player
    participant W as world system
    participant TG as tile_generator
    participant E as encounter system

    P->>W: Command::WorldMove
    W->>W: is_adjacent? travel_cost?
    W->>E: should_trigger_encounter?
    alt Encounter triggered
        E-->>W: EncounterState (hostile/neutral/beneficial)
        W->>W: spawn_encounter_entities
    else No encounter
        W->>TG: generate_tile(params)
        TG-->>W: GeneratedTile
        W->>W: Replace map, spawn entities
    end
```

## DES Test Execution Flow

```mermaid
flowchart TD
    JSON[Scenario JSON] --> PARSE[from_json / from_file]
    PARSE --> INHERIT{inherits?}
    INHERIT -->|Yes| MERGE[Merge parent fields]
    INHERIT -->|No| SETUP
    MERGE --> SETUP[apply_map_setup + player setup]
    SETUP --> ENTITIES[Spawn entities]
    ENTITIES --> MOCKS[Apply mock settings]
    MOCKS --> ACTIONS[Execute actions sequentially]
    ACTIONS --> ASSERT[Evaluate assertions]
    ASSERT --> RESULT{All pass?}
    RESULT -->|Yes| PASS[ExecutionResult::success]
    RESULT -->|No| FAIL[ExecutionResult with failures]
```

## Save/Load Flow

```mermaid
sequenceDiagram
    participant UI as Save Menu
    participant SV as save.rs
    participant FS as Filesystem

    UI->>SV: save_game(state, slot_name)
    SV->>SV: Serialize GameState to RON
    SV->>SV: Compute MD5 checksum
    SV->>SV: Wrap in SaveFile envelope (version + data)
    SV->>FS: Write to saves/{hash}.ron
    SV->>SV: Update saves/meta.json

    UI->>SV: load_game(slot_name)
    SV->>FS: Read saves/{hash}.ron
    SV->>SV: Verify checksum
    SV->>SV: Check SAVE_VERSION
    SV->>SV: migrate_save if needed (v1→v2)
    SV->>SV: Deserialize GameState
    SV-->>UI: GameState
```

## Data Loading Flow

```mermaid
flowchart LR
    STARTUP[Game startup] --> LOAD["DataLoader::load_single/load_multiple"]
    LOAD --> PARSE[serde_json::from_str]
    PARSE --> VALIDATE["jsonschema::validate against schemas/*_v1.json"]
    VALIDATE -->|Pass| CACHE[Cached in once_cell lazy statics]
    VALIDATE -->|Fail| PANIC[Panic with validation error]
    CACHE --> QUERY["get(id) / all() / ids()"]
```

Data files are loaded once at startup via `include_str!` (compile-time embedding) or `fs::read_to_string` (runtime). Schema validation catches structural errors immediately.

## Adding a New Gameplay System

```mermaid
flowchart TD
    RULE["1. Write system function in systems/"] --> MUT["2. Add Mutation variants if needed"]
    MUT --> APPLY["3. Add apply_one arm in state.rs"]
    APPLY --> WIRE["4. Wire in dispatch.rs::route_command"]
    WIRE --> NOTIFY_Q{Needs reactions?}
    NOTIFY_Q -->|Yes| NOTIFY_W["5. Add handler + wire in notify.rs"]
    NOTIFY_Q -->|No| DES
    NOTIFY_W --> DES["6. Write DES scenario"]
    DES --> STATUS["7. Update SYSTEM_STATUS.md"]
```
