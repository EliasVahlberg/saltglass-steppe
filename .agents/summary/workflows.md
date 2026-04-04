# Workflows

## Game Turn Loop

Input → Action → state mutation → `end_turn()` → Render.

```mermaid
sequenceDiagram
    participant Player
    participant Input as Input Handler
    participant State as GameState
    participant Systems as end_turn()
    participant Renderer

    Player->>Input: Keypress
    Input->>State: Action (move/attack/use/wait)
    State->>State: Deduct AP, apply action
    State->>Systems: end_turn()
    Note over Systems: Reset AP
    Note over Systems: StatusEffectSystem.update()
    Note over Systems: psychic/skills/light/void/crystal tick
    Note over Systems: tick_turn + update_enemies (AI)
    Note over Systems: Storm tick → StormSystem::apply_storm
    Note over Systems: tick_time → update_lighting → update_fov
    Note over Systems: check_encounter_completion
    Note over Systems: emit TurnEnded → process_events
    Note over Systems: Event loop: LootSystem + QuestSystem
    Systems->>Renderer: Render frame
    Renderer->>Player: TUI display
```

## Map Generation Pipeline

`travel_to_tile()` → `generate_tile()` → terrain-forge exclusively (custom algorithms deleted).

```mermaid
graph TD
    A["travel_to_tile(wx, wy)"] --> B["TileParams::from_world_state()"]
    B --> C["generate_tile()"]
    C --> D["TerrainForgeGenerator::generate_tile_with_seed()"]
    D --> D1{"POI type?"}
    D1 -->|"Dungeon/Landmark/Shrine"| D2["DungeonGenerator (fallback to terrain-forge)"]
    D1 -->|"Town/None"| D3["terrain-forge ops::generate with biome profile"]
    D2 --> E["Glass Seam Bridging connectivity"]
    D3 --> E
    E --> F["place_environmental_props()"]
    F --> G["Spawn enemies from biome_spawn_tables"]
    G --> H["Spawn items + loot"]
    H --> I["Spawn NPCs + chests + interactables"]
    I --> J["Return GeneratedTile"]
    J --> K["Post-load in state.rs"]
    K --> K1["feature_materializer"]
    K1 --> K2["spawn_crafting_stations (towns)"]
    K2 --> K3["spawn_quest_required_npcs"]
    K3 --> K4["update_fov + rebuild_spatial_index + update_lighting"]
```

## DES Testing Workflow

Scenarios in `tests/scenarios/` — JSON or `.des` format. Run without TUI.

```mermaid
graph TD
    A["Write scenario JSON"] --> B["Define map_setup, player, entities"]
    B --> C["Define actions sequence"]
    C --> D["Define assertions (inline + at_end)"]
    D --> E["cargo test --test des_scenarios"]
    E --> F["DES executor loads scenario"]
    F --> F1{"inherits?"}
    F1 -->|Yes| F2["Merge with BASE_* scenario"]
    F1 -->|No| G["Apply map setup + spawn entities"]
    F2 --> G
    G --> H["Apply mocks (combat_always_hit, etc.)"]
    H --> I["Execute actions sequentially"]
    I --> J["Check inline assertions"]
    J --> K["Check at_end assertions"]
    K --> L{Pass?}
    L -->|Yes| M["Test passes"]
    L -->|No| N["Assertion failure with context"]
```

## Save/Load Flow

RON format, MD5-named files, versioned with migration.

```mermaid
graph LR
    A["GameState"] -->|"save_game()"| B["Wrap in SaveFile{version, state}"]
    B --> C["Serialize to RON"]
    C --> D["Compute MD5 of content"]
    D --> E["Write saves/<md5>.ron"]
    E --> F["Update saves/meta.json"]
```

```mermaid
graph LR
    G["Load request"] --> H["Read saves/<hash>.ron"]
    H --> I["Compute MD5, compare to filename"]
    I -->|Mismatch| I2["Mark HashMismatch in meta"]
    I -->|Match| J["Deserialize RON → SaveFileOwned"]
    I2 --> J
    J -->|"Parse error"| J2["Mark Corrupt, abort"]
    J -->|"version < SAVE_VERSION"| K["migrate_save()"]
    J -->|"version == SAVE_VERSION"| L["Use state directly"]
    J -->|"version > SAVE_VERSION"| J3["Reject: future version"]
    K --> L
    L --> M["rebuild_spatial_index + update_lighting"]
    M --> N["Mark Ok in meta, return GameState"]
```

## World Travel Flow

```mermaid
graph TD
    A["Player moves on world map"] --> B["Check adjacency"]
    B --> C["travel_to_tile(new_wx, new_wy)"]
    C --> D["Build TileParams from WorldState"]
    D --> E["generate_tile() — full pipeline"]
    E --> F["Set world position, map, entities"]
    F --> G["feature_materializer"]
    G --> H{"POI == Town?"}
    H -->|Yes| I["spawn_crafting_stations"]
    H -->|No| J["spawn_quest_required_npcs"]
    I --> J
    J --> K["generate_crystal_formations"]
    K --> L["update_fov + rebuild_spatial_index + update_lighting"]
    L --> M["Log area entry"]
```

## CI Pipeline

Two jobs: `test` (build + test + lint + fmt) then `des-scenarios`.

```mermaid
graph LR
    A["Push/PR to main"] --> B["cargo build"]
    B --> C["cargo test"]
    C --> D["cargo clippy -- -D warnings"]
    D --> E["cargo fmt -- --check"]
    E --> F["des-scenarios job"]
    F --> G["cargo test --test des_scenarios"]
```

## Content Addition Workflow

1. Add data to appropriate JSON file in `data/`
2. Cross-reference IDs (items in traders/loot_tables, enemies in biome_spawn_tables, NPCs in dialogues/quests)
3. If Rust types changed: `cargo run --bin schema_gen`
4. Write DES scenario exercising the new content
5. Run `cargo test` + relevant `./test_all_*` script
6. Update `docs/development/SYSTEM_STATUS.md` if adding/modifying a system
