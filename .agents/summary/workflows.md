# Workflows

## Player Action Flow

```mermaid
sequenceDiagram
    participant UI as UI Input
    participant Main as main.rs
    participant Disp as dispatch()
    participant Rule as Rule Function
    participant Apply as apply_effect()
    participant React as Reactions
    participant Derive as Derives

    UI->>Main: Keypress
    Main->>Disp: Command enum
    Disp->>Rule: rule_fn(args, &QueryContext, &mut rng)
    Rule-->>Disp: RuleOutput { effects, presentation }
    loop Each effect
        Disp->>Apply: apply_effect(&effect)
        Disp->>Disp: trace.record(effect)
    end
    Disp->>React: run_reactions(effects, depth=0)
    React-->>Apply: reaction effects applied
    Disp->>Derive: update_fov(), update_lighting()
    Main->>Main: end_turn() if AP depleted
```

## End-of-Turn Flow

```mermaid
sequenceDiagram
    participant ET as end_turn()
    participant Phase as execute_phase()
    participant Bridge as Bridge Effects
    participant Legacy as Legacy Systems

    ET->>Phase: ResetAp
    Phase->>Phase: PlayerEffect::ResetAp → trace

    ET->>Phase: TickStatusEffects
    Phase->>Bridge: PlayerEffect::TickStatusEffects
    Bridge->>Legacy: StatusEffectSystem::tick_player_effects()

    ET->>Phase: TickSubsystems
    Phase->>Bridge: TickPsychic, TickSkills, TickLight, TickVoid, TickCrystal
    Bridge->>Legacy: Each .tick()/.update() method

    ET->>Phase: AdvanceTurn
    Phase->>Phase: PlayerEffect::AdvanceTurn + TickHousekeeping + adaptation check

    ET->>Phase: RunAI
    Phase->>Bridge: PlayerEffect::RunAI
    Bridge->>Legacy: AiSystem::update_enemies()

    ET->>Phase: TickStorm
    Phase->>Bridge: MapEffect::TickStorm
    Bridge->>Legacy: storm.tick() + StormSystem::apply_storm()

    ET->>Phase: AdvanceTime
    Phase->>Phase: Inline time/weather logic → traced

    ET->>Phase: UpdateDerives
    Phase->>Phase: update_fov(), update_lighting() (not traced)

    ET->>Phase: CheckEncounters
    Phase->>Phase: rule_check_encounters → traced
```

## Combat Flow

```mermaid
sequenceDiagram
    participant P as Player
    participant D as dispatch()
    participant R as rule_melee_attack
    participant A as apply_effect
    participant React as Reactions

    P->>D: Command::Attack { target_x, target_y }
    D->>R: rule_melee_attack(x, y, &ctx, &mut rng)
    alt Hit
        R-->>D: [SpendAp, DealDamage, RecordDamageDealt]
        D->>A: Apply each effect
    else Miss
        R-->>D: [SpendAp, Miss]
    else Kill
        R-->>D: [SpendAp, DealDamage, Kill, GainXp]
        D->>A: Apply effects
        D->>React: run_reactions([Kill])
        React-->>A: LootDrop, QuestNotify(Kill)
    end
```

## Reaction Chain (Post-Batch F)

```mermaid
graph TB
    KILL["CombatEffect::Kill"] --> LOOT["EventEffect::LootDrop"]
    KILL --> QUEST["EventEffect::QuestNotify(Kill)"]
    LOOT --> APPLY_LOOT["LootSystem::drop_loot()"]
    QUEST --> APPLY_QUEST["quest_log.on_enemy_killed()"]
    APPLY_QUEST --> AUTO["check_auto_complete()"]
```

Reactions replace the old `GameEvent` system. Max depth: 10.

## World Travel Flow

```mermaid
sequenceDiagram
    participant P as Player
    participant D as dispatch()
    participant S as state.rs helpers

    P->>D: Command::WorldMove { new_wx, new_wy }
    D->>S: dispatch_world_move()
    S->>S: Validate adjacency
    S->>S: move_on_world_map() — regenerate tile map
    S->>S: spawn_quest_required_npcs()
    S->>S: spawn_crafting_stations()
    S->>S: Trace effects (SetWorldPosition, IncrementTilesTraveled)
```

## Map Generation Flow

```mermaid
graph TB
    START["travel_to_tile()"] --> PARAMS["TileParams from WorldState"]
    PARAMS --> TFA["terrain_forge_adapter<br/>Base terrain generation"]
    TFA --> CONN["connectivity.rs<br/>Glass Seam Bridging"]
    CONN --> STRUCT["structure_library.rs<br/>Stamp prefabs"]
    STRUCT --> MICRO["microstructures.rs<br/>Small features"]
    MICRO --> PROPS["environmental_props.rs<br/>Decorations"]
    PROPS --> SPAWN["spawn.rs<br/>Enemies, items"]
    SPAWN --> FEAT["feature_materializer.rs<br/>NPCs, story hooks"]
    FEAT --> QUEST["quest_constraints.rs<br/>Validate requirements"]
    QUEST --> DONE["GeneratedTile"]
```

## DES Test Execution Flow

```mermaid
sequenceDiagram
    participant Runner as des_scenarios.rs
    participant DES as DesExecutor
    participant GS as GameState

    Runner->>DES: from_json(scenario)
    DES->>DES: inherit_from(base) if inherits
    DES->>GS: new_with_class() + apply_map_setup()
    DES->>GS: Spawn entities, apply mocks
    loop Each action
        DES->>GS: execute_player_action(action)
        Note over GS: dispatch(Command) for VERA actions
        DES->>DES: Check mid-action assertions
    end
    DES->>DES: Check at_end assertions
    DES-->>Runner: ExecutionResult { passed, failures }
```

## Save/Load Flow

```mermaid
sequenceDiagram
    participant P as Player
    participant S as save.rs
    participant FS as Filesystem

    P->>S: save_game(slot)
    S->>S: Serialize GameState to JSON
    S->>S: Compute MD5 checksum
    S->>FS: Write saves/{slot}.json
    S->>FS: Update saves/meta.json

    P->>S: load_game(slot)
    S->>FS: Read saves/{slot}.json
    S->>S: Verify checksum
    S->>S: Check version, migrate if needed
    S-->>P: GameState
```

## CI Pipeline

```mermaid
graph LR
    PUSH["git push"] --> BUILD["cargo build"]
    BUILD --> TEST["cargo test"]
    TEST --> CLIP["cargo clippy -- -D warnings"]
    CLIP --> FMT["cargo fmt -- --check"]
    FMT --> DES["cargo test --test des_scenarios"]
```
