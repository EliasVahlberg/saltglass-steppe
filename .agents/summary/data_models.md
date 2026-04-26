# Data Models

<!-- Generated: 2026-04-06 | tags: data-models, structs, enums, state -->

## GameState (Central Hub)

```mermaid
classDiagram
    class GameState {
        +PlayerState player
        +WorldState world
        +NarrativeEngine narrative
        +SpatialIndex spatial
        +DebugState debug
        +ChaCha8Rng rng
        +u32 turn
        +u32 wait_counter
        +HashSet~usize~ visible
        +HashSet~usize~ revealed
        +Vec~GameMessage~ messages
        +Vec~TriggeredEffect~ active_effects
        +Vec~Decoy~ decoys
        +MapFeatures map_features
        +PendingUi pending_ui
        +dispatch(Command)
        +apply_mutations(Vec~Mutation~) Vec~StateTransition~
    }

    class PlayerState {
        +i32 x, y
        +i32 hp, max_hp
        +i32 ap, max_ap
        +u32 xp, level
        +u32 salt_scrip
        +u32 refraction
        +Vec~String~ inventory
        +Equipment equipment
        +Vec~Adaptation~ adaptations
        +Vec~StatusEffect~ status_effects
        +SkillsState skills
        +PsychicState psychic
    }

    class WorldState {
        +Map map
        +WorldMap world_map
        +Vec~Enemy~ enemies
        +Vec~Npc~ npcs
        +Vec~Item~ items
        +Vec~Chest~ chests
        +Vec~Interactable~ interactables
        +u8 time_of_day
        +Weather weather
        +Storm storm
        +Option~EncounterState~ encounter_state
        +usize world_x, world_y
        +i32 layer
    }

    class NarrativeEngine {
        +QuestLog quest_log
        +StoryModel story_model
        +WorldHistory world_history
    }

    GameState --> PlayerState
    GameState --> WorldState
    GameState --> NarrativeEngine
```

## Entity Models

### Enemy

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | References `data/enemies/*.json` |
| `x`, `y` | `i32` | Map position |
| `hp`, `max_hp` | `i32` | Health |
| `status_effects` | `Vec<StatusEffect>` | Active effects |
| `provoked` | `bool` | Aggro state |

Defined by `EnemyDef` in data: `glyph`, `behavior` (StandardMelee/RangedOnly/Healer/SuicideBomber), `demeanor` (Aggressive/Defensive/Neutral), `faction`, `loot`, `xp_value`, `sight_range`. Split across `data/enemies/{common,uncommon,rare,elite,boss}.json`.

### Npc

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | References `data/npcs.json` |
| `x`, `y` | `i32` | Map position |
| `hp`, `max_hp` | `i32` | Health |

Defined by `NpcDef`: `glyph`, `dialogue`, `backstory`, `available_actions` (trade, quest, dialogue, craft).

### Item

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | References `data/items.json` |
| `x`, `y` | `i32` | Map position (when on ground) |

Defined by `ItemDef`: `glyph`, `tier`, `consumable`, `pickup`, `effects` (heal, damage, reveal, status), `light_source`, `equip_slot`.

## Map Models

### Map

| Field | Type | Description |
|-------|------|-------------|
| `width`, `height` | `usize` | Dimensions |
| `tiles` | `Vec<Tile>` | Flat grid (row-major) |
| `inscriptions` | `Vec<MapInscription>` | Discoverable text |
| `lights` | `Vec<MapLight>` | Static light sources |

### Tile

| Field | Type | Description |
|-------|------|-------------|
| `glyph` | `char` | Display character |
| `walkable` | `bool` | Movement allowed |
| `transparent` | `bool` | FOV passes through |
| `name` | `String` | Tile type name |
| `wall_hp` | `Option<i32>` | Breakable walls |

### WorldMap

Grid of world tiles with `Biome`, `Terrain`, `POI`, `Resources`, faction territories, and road connections.

## Effect & Mutation Enums

### Effect Domains (7)

| Domain | Key Variants |
|--------|-------------|
| `PlayerEffect` | Heal, TakeDamage, SpendAp, SetPosition, GainXp, RunAI, TickSubsystems |
| `CombatEffect` | DealDamage, Miss, Kill, Provoke, StunEnemy |
| `ItemEffect` | Consume, Equip, Unequip, AddToInventory, SpawnOnMap |
| `MapEffect` | RevealAll, DamageWall, TickStorm, AdvanceTime, SetWeather |
| `ResourceEffect` | GainLightEnergy, GainVoidEnergy, GainResonanceEnergy |
| `EventEffect` | OpenBook, LootDrop, QuestNotify |
| `QuestEffect` | Accept, Complete, SetFactionAlignment |

### Mutation Categories (~70 variants)

| Category | Examples |
|----------|---------|
| Player vitals | `SetPlayerHp`, `SetPlayerAp`, `SetPlayerPosition` |
| Player progression | `SetPlayerXp`, `SetPlayerLevel`, `SetPlayerSaltScrip` |
| Player state | `AddAdaptation`, `AddStatusEffect`, `Equip`, `Unequip` |
| Inventory | `AddToInventory`, `RemoveFromInventory`, `SpawnItemOnMap` |
| Enemies | `SetEnemyHp`, `RemoveEnemy`, `SpawnEnemy`, `StunEnemy` |
| World | `SetTimeOfDay`, `SetWeather`, `AdvanceTurn`, `SetWorldPosition` |
| Map | `SetTile`, `RevealTile`, `RevealAll`, `DamageWall` |
| Encounter | `SetEncounterState`, `SetLastFleeAttempt` |
| Faction/Quest | `SetReputation`, `AcceptQuest`, `CompleteQuest`, `QuestNotify` |
| Resources | `SetLightEnergy`, `AddVoidEnergy`, `SetResonanceEnergy` |
| Presentation | `LogMessage`, `HitFlash`, `DamageNumber`, `SpawnProjectile` |
| Bridge | `MovePlayer`, `EndTurn`, `WorldMove`, `RestTick`, `TickSubsystem` |

## Data Files (JSON)

```mermaid
graph TB
    subgraph Content["Game Content"]
        ITEMS[items.json]
        ENEMIES[enemies/*.json]
        NPCS[npcs.json]
        QUESTS[quests.json]
        MAINQ[main_questline.json]
        DIALOGUES[dialogues.json]
        BOOKS[books.json]
    end

    subgraph Config["Configuration"]
        TERRAIN[terrain_config.json]
        BIOME_SPAWN[biome_spawn_tables.json]
        LOOT[loot_tables.json]
        FACTIONS[factions.json]
        STORM[storm_config.json]
        TRAVEL[travel_config.json]
    end

    subgraph Systems["System Data"]
        SKILLS[skill_trees.json]
        ABILITIES[abilities.json]
        ADAPTATIONS[adaptations.json]
        RECIPES[recipes.json]
        WEAPONS[weapons.json]
        EFFECTS_D[effects.json]
    end

    subgraph Visual["Visual/UI"]
        RENDER[render_config.json]
        THEMES[themes.json]
        KEYBOARD[keyboard_config.json]
        TUTORIAL[tutorial.json]
    end
```

### Cross-Reference Map

| When modifying... | Also check... |
|-------------------|---------------|
| `items.json` | `traders.json`, `loot_tables.json`, `recipes.json` |
| `enemies/*.json` | `biome_spawn_tables.json`, `loot_tables.json` |
| `npcs.json` | `dialogues.json`, `quests.json` |
| `structures/` | `map_elements.json` |
| Rust data types | Run `cargo run --bin schema_gen` to regenerate schemas |

## Save Format

`SaveFile` envelope with version number + serialized `GameState` in RON format. Current version: v1 (with v2 migration for faction reputation). Stored in `saves/` directory with MD5 checksum for integrity.
