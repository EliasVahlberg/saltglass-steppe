# Game Systems Overview

> **Purpose**: A comprehensive guide for developers working on Saltglass Steppe. Start here to understand how systems interact and where to add new content.

## Quick Reference

| I Want To...                     | Look At                                                    |
| -------------------------------- | ---------------------------------------------------------- |
| Add a new item                   | `data/items.json`, see [Data Files](#data-files)           |
| Add a new enemy                  | `data/enemies.json`, see [Enemy System](#enemy-system)     |
| Add a new quest                  | `data/quests.json`, see [Quest System](#quest-system)      |
| Add a dynamic event              | `data/dynamic_events.json`, see [Event System](#event-system) |
| Add a story fragment             | `data/narrative_integration.json`, see [Narrative System](#narrative-system) |
| Create a test scenario           | `tests/scenarios/`, see [DES Testing](#des-testing-system) |
| Add a new AI behavior            | `src/game/systems/ai.rs`, see [AI System](#ai-system)      |
| Add a new game mechanic          | `src/game/systems/`, see [Systems Layer](#systems-layer)   |
| Understand the game loop         | `src/main.rs`, see [Game Loop](#game-loop)                 |
| Add visual effects               | `data/effects_config.json`, see [Effects DSL](#visual-effects-dsl) |

---

## Architecture Philosophy

### Core Principles

1. **Determinism First**: All game logic uses seeded RNG (`ChaCha8Rng`) for 100% reproducibility. This enables automated testing via the Debug Execution System (DES).

2. **Data-Driven Design**: Game content lives in JSON files under `data/`. Adding items, enemies, quests, etc. requires *no code changes* unless you're adding new mechanics.

3. **Strict Layer Separation**:
   - `src/game/` — Pure game logic. **Never** imports rendering libraries.
   - `src/renderer/` — All rendering via `ratatui`. Reads `GameState`, never modifies game logic.
   - `src/des/` — Headless test framework. Drives `GameState` without rendering.

4. **Event-Driven Communication**: Systems communicate through a `GameEvent` bus rather than direct method calls, enabling loose coupling.

### Directory Structure

```
src/
├── main.rs              # Entry point, game loop, input handling
├── lib.rs               # Library exports
├── game/                # Pure game logic (THE CORE)
│   ├── mod.rs           # Module re-exports
│   ├── state.rs         # GameState - the single source of truth
│   ├── systems/         # Extracted logic systems
│   │   ├── ai.rs        # Enemy AI and behavior registry
│   │   ├── combat.rs    # Attack resolution, damage, death
│   │   ├── movement.rs  # Player movement, tile effects
│   │   ├── loot.rs      # Event-driven loot drops
│   │   ├── quest.rs     # Event-driven quest progress
│   │   ├── status.rs    # Status effect ticking
│   │   └── storm.rs     # Glass storm map transformations
│   ├── enemy.rs         # Enemy definitions & data loading
│   ├── item.rs          # Item definitions & data loading
│   ├── npc.rs           # NPC definitions & data loading
│   ├── quest.rs         # Quest definitions & data loading
│   └── ...              # Other game modules
├── renderer/            # TUI rendering (ratatui)
├── des/                 # Debug Execution System
└── ui/                  # UI state and input handling
data/                    # JSON content definitions
tests/
└── scenarios/           # DES test scenarios (JSON)
```

---

## The GameState Model

`GameState` (`src/game/state.rs`) is the **single source of truth** for all game data. It is:

- **Serializable**: Supports save/load via `serde`
- **Deterministic**: Uses `ChaCha8Rng` for all randomness
- **Central Hub**: All systems read from and write to `GameState`

### Decomposed Structure

GameState has been decomposed into three major sub-structs for better organization:

```rust
pub struct GameState {
    // Decomposed state
    pub player: PlayerState,
    pub world: WorldState,
    pub narrative: NarrativeEngine,
    
    // Core game state
    pub visible: HashSet<usize>,
    pub revealed: HashSet<usize>,
    pub light_map: LightMap,  // Skipped in serialization
    pub messages: Vec<GameMessage>,
    pub turn: u32,
    
    // Seeded RNG (critical for determinism)
    #[serde(with = "rng_serde")]
    pub rng: ChaCha8Rng,
    
    // Effects & Events
    pub triggered_effects: Vec<TriggeredEffect>,
    pub decoys: Vec<Decoy>,
    pub event_queue: Vec<GameEvent>,  // Skipped in serialization
    
    // Meta
    pub seed: u64,
    pub meta: MetaProgress,
}
```

### PlayerState Fields

```rust
pub struct PlayerState {
    // Position
    pub x: i32, pub y: i32, pub layer: u8,
    
    // Core Stats
    pub hp: i32, pub max_hp: i32,
    pub ap: i32, pub max_ap: i32,
    pub reflex: i32, pub armor: i32,
    
    // Progression
    pub xp: u32, pub level: u32,
    pub pending_stat_points: u32,
    pub salt_scrip: i32,
    
    // Inventory & Equipment
    pub inventory: Vec<String>,
    pub equipped_weapon: Option<String>,
    pub equipment: Equipment,
    
    // Mutations & Adaptations
    pub refraction: i32,
    pub adaptations: Vec<Adaptation>,
    pub adaptations_hidden_turns: u32,
    
    // Status & Social
    pub status_effects: Vec<StatusEffect>,
    pub faction_reputation: HashMap<String, i32>,
    
    // Quests
    pub quest_log: QuestLog,
    
    // Specialized Systems
    pub psychic: PsychicState,
    pub skills: SkillState,
    pub light_system: LightSystem,
    pub void_system: VoidSystem,
    pub crystal_system: CrystalSystem,
    
    // Combat
    pub last_damage_dealt: i32,
}
```

### WorldState Fields

```rust
pub struct WorldState {
    // Navigation
    pub world_map: WorldMap,
    pub world_x: usize, pub world_y: usize,
    pub layer: u8,
    
    // Current Tile
    pub map: Map,
    pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub chests: Vec<Chest>,
    pub interactables: Vec<Interactable>,
    pub microstructures: Vec<Microstructure>,
    
    // Environment
    pub storm: Storm,
    pub time_of_day: TimeOfDay,
    pub weather: Weather,
    pub ambient_light: u8,
    
    // Visual
    pub visual_effects: Vec<VisualEffect>,
    pub light_map: LightMap,
    
    // Encounters
    pub encounter_state: EncounterState,
    pub encounter_history: Vec<EncounterRecord>,
    pub total_tiles_traveled: u32,
    
    // Pathfinding
    pub world_map_target: Option<(usize, usize)>,
    pub world_map_path: Vec<(usize, usize)>,
    
    // Spatial Index (computed on load)
    pub enemy_positions: HashMap<(i32, i32), usize>,
    pub npc_positions: HashMap<(i32, i32), usize>,
    pub item_positions: HashMap<(i32, i32), Vec<usize>>,
}
```

### NarrativeEngine Fields

```rust
pub struct NarrativeEngine {
    // Quests
    pub quest_log: QuestLog,
    
    // Story
    pub story_model: StoryModel,
    
    // Tutorial
    pub tutorial_progress: TutorialProgress,
    
    // History
    pub world_history: WorldHistory,
    
    // Effects
    pub triggered_effects: TriggeredEffects,
}
```

### Pattern: Accessing GameState

Systems receive `&mut GameState` and operate on it:

```rust
// In src/game/systems/combat.rs
impl CombatSystem {
    pub fn attack_melee(state: &mut GameState, target_idx: usize) {
        // Read from state
        let damage = state.calc_player_damage();
        // Modify state
        state.enemies[target_idx].hp -= damage;
        // Emit event
        state.events.push(GameEvent::EnemyKilled { ... });
    }
}
```

---

## Systems Layer

Systems are stateless modules that operate on `GameState`. They follow the **Data-Oriented Design** pattern: data lives in `GameState`, logic lives in systems.

### System Trait

```rust
// src/game/systems/mod.rs
pub trait System {
    fn update(&self, state: &mut GameState);
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
```

### Active Systems

| System               | File                          | Responsibility                              |
| -------------------- | ----------------------------- | ------------------------------------------- |
| `CombatSystem`       | `systems/combat.rs`           | Attack resolution, damage, death processing |
| `AiSystem`           | `systems/ai.rs`               | Enemy decision-making, behavior dispatch    |
| `MovementSystem`     | `systems/movement.rs`         | Player movement, tile effects, NPC triggers |
| `LootSystem`         | `systems/loot.rs`             | Listens to `EnemyKilled`, drops loot        |
| `QuestSystem`        | `systems/quest.rs`            | Listens to events, updates quest progress   |
| `StatusEffectSystem` | `systems/status.rs`           | Ticks status effects each turn              |
| `StormSystem`        | `systems/storm.rs`            | Storm progression, map transformations      |

### Generation Systems (Unified Module)

| System               | File                          | Responsibility                              |
| -------------------- | ----------------------------- | ------------------------------------------- |
| `GenerationPipeline` | `generation/pipeline.rs`      | Coordinates all generation systems          |
| `WorldGenerator`     | `generation/world_gen.rs`     | Enhanced world map generation with POI preferences |
| `TerrainForgeGenerator` | `generation/terrain_forge_adapter.rs` | Tile generation via terrain-forge (external crate) |
| `SpawnSystem`        | `generation/spawn.rs`         | Weighted entity spawning by biome/level    |
| `LootGeneration`     | `generation/loot.rs`          | Procedural loot generation with weighted tables |
| `SpatialSystem`      | `generation/spatial.rs`       | Poisson disk sampling and spatial distribution |
| `MicroStructures`    | `generation/microstructures.rs` | Mini-structure placement system          |
| `BiomeSystem`        | `generation/biomes.rs`        | Biome-specific environmental content generation |
| `Grammar`            | `generation/grammar.rs`       | Dynamic text generation with rule expansion |
| `TemplateLibrary`    | `generation/templates.rs`     | Template-based procedural content creation |
| `WeightedTable`      | `generation/weighted_table.rs`| Enhanced weighted selection for spawns/loot |
| `EventSystem`        | `generation/events.rs`        | Dynamic events based on player/world state |
| `NarrativeIntegration` | `generation/narrative.rs`   | Story fragment placement and faction influence |
| `NarrativeTemplates` | `generation/narrative_templates.rs` | Template-based narrative generation |
| `StorySystem`        | `generation/story.rs`         | Procedural story and character generation |
| `ConstraintSystem`   | `generation/constraints.rs`   | Constraint-based generation validation |
| `ConnectivitySystem` | `generation/connectivity.rs`  | Glass Seam Bridging Algorithm for map connectivity |
| `QuestConstraints`   | `generation/quest_constraints.rs` | Quest-driven generation constraints |
| `AlgorithmRegistry`  | `generation/registry.rs`      | Plugin system for generation algorithms |
| `GenerationAlgorithm`| `generation/algorithm.rs`     | Core algorithm trait and framework |
| `StructureGenerators`| `generation/structures/`      | BSP, Cellular Automata, and dungeon generators |
| `BracketAdapter`     | `generation/adapters/`        | bracket-lib integration layer |
| `MapFeature pipeline`| *(planned in generation/pipeline.rs)* | Materializes `Map.features` (from terrain-forge `SemanticExtractor`) into interactables, props, and spawns |

### Adding a New System

1. Create `src/game/systems/my_system.rs`
2. Implement the `System` trait
3. Add `pub mod my_system;` to `src/game/systems/mod.rs`
4. Call from `GameState::end_turn()` or relevant trigger point

### Map Features (new)

- `Map.features` (see `src/game/map.rs`) carries data-driven feature placements produced during tile generation (`TerrainForgeGenerator` consumes `data/terrain_config.json` POI layouts and biome feature weights).
- Materialization is intentionally separate: a lightweight feature materializer should run after map generation to translate feature ids into interactables (`data/interactables.json`), props/loot (via `generation/spawn.rs`), or narrative hooks—keeping generation deterministic but decoupled from runtime systems.

---

## Event Bus

Systems communicate through events to avoid tight coupling.

### GameEvent Enum

```rust
// src/game/event.rs
pub enum GameEvent {
    PlayerDamaged { amount: i32, source: String },
    PlayerHealed { amount: i32 },
    EnemyKilled { enemy_id: String, x: i32, y: i32 },
    ItemPickedUp { item_id: String },
    ItemUsed { item_id: String },
    AdaptationGained { name: String },
    StormArrived { intensity: u8 },
    LevelUp { level: u32 },
}
```

### Event Flow

```
1. System emits event → state.events.push(GameEvent::EnemyKilled {...})
2. end_turn() calls → process_events()
3. Each system's on_event() is called → LootSystem drops loot, QuestSystem updates progress
```

### Pattern: Event-Driven Side Effects

Instead of `CombatSystem` directly calling loot logic:

```rust
// BAD: Tight coupling
fn kill_enemy(state: &mut GameState, idx: usize) {
    drop_loot(state, idx);       // Direct call
    update_quests(state, idx);   // Direct call
}

// GOOD: Event-driven
fn kill_enemy(state: &mut GameState, idx: usize) {
    state.events.push(GameEvent::EnemyKilled { ... });
    // LootSystem and QuestSystem will react in on_event()
}
```

---

## Data Loading System

All game content is defined in JSON and loaded via the unified `DataLoader<T>` system.

### Unified Data Loader

**Location**: `src/game/data_loader.rs`

Generic loader with JSON schema validation:

```rust
use crate::game::data_loader::DataLoader;

// Single file loading
static ITEMS: Lazy<DataLoader<ItemDef>> = Lazy::new(|| {
    DataLoader::load_single(
        include_str!("../../data/items.json"),
        "items_v1",
        "items.json"
    ).expect("Failed to load items")
});

// Multiple file loading (e.g., enemies)
static ENEMIES: Lazy<DataLoader<EnemyDef>> = Lazy::new(|| {
    DataLoader::load_multiple(&[
        ("common", include_str!("../../data/enemies/common.json")),
        ("uncommon", include_str!("../../data/enemies/uncommon.json")),
        ("rare", include_str!("../../data/enemies/rare.json")),
        ("elite", include_str!("../../data/enemies/elite.json")),
        ("boss", include_str!("../../data/enemies/boss.json")),
    ], "enemies_v1", "enemies").expect("Failed to load enemies")
});

// Access data
pub fn get_item_def(id: &str) -> Option<&'static ItemDef> {
    ITEMS.get(id)
}

pub fn all_item_ids() -> Vec<&'static str> {
    ITEMS.all_ids()
}
```

**Features**:
- ✅ Automatic schema version validation
- ✅ Consistent error messages with file context
- ✅ Duplicate ID detection
- ✅ Type-safe generic implementation
- ✅ Single and multiple file loading

### Data Files (45 Total)

**Consolidated from 52 files** (7 files merged):
- `dialogues.json` ← merged `aria_dialogues.json`
- `effects.json` ← merged `status_effects.json`, `effects_config.json`
- `abilities.json` ← merged `skills.json`, `psychic_abilities.json`
- `map_elements.json` ← merged `floors.json`, `walls.json`, `lights.json`

**Core Game Content (7 files)**:
- `items.json` - Items, equipment, consumables
- `weapons.json` - Weapon definitions and stats
- `abilities.json` - Player abilities, skills, psychic powers (consolidated)
- `classes.json` - Character class definitions
- `progression.json` - Character progression settings
- `adaptations.json` - Player mutations/upgrades
- `effects.json` - Effects, status effects, config (consolidated)

**Enemies & Combat (8 files)**:
- `enemies/common.json` - Common enemy types
- `enemies/uncommon.json` - Uncommon enemy types
- `enemies/rare.json` - Rare enemy types
- `enemies/elite.json` - Elite enemy types
- `enemies/boss.json` - Boss enemy types
- `factions.json` - Faction definitions, reputation systems
- `loot_tables.json` - Weighted loot distributions
- `biome_spawn_tables.json` - Per-biome entity spawns

**World Generation (6 files)**:
- `terrain_config.json` - Terrain generation parameters (active)
- `biome_profiles.json` - Biome-specific content and features
- `structure_templates.json` - Procedural structure definitions
- `structure_generation.json` - Structure generation config
- `microstructures.json` - Mini-structure definitions
- `map_features.json` - Map feature definitions
- `map_elements.json` - Walls, floors, lights (consolidated)

**Environmental Systems (5 files)**:
- `storm_config.json` - Storm timing and effects
- `encounter_config.json` - Encounter system configuration
- `dynamic_events.json` - Dynamic events and triggers
- `travel_config.json` - Travel system configuration
- `auto_explore_config.json` - Auto-exploration system settings

**NPCs & Narrative (7 files)**:
- `npcs.json` - NPCs, merchants, dialogue refs, faction leaders
- `dialogues.json` - Conversation trees
- `aria_dialogues.json` - ARIA-specific dialogues
- `traders.json` - Trader NPCs and their inventories
**NPCs & Narrative (5 files)**:
- `npcs.json` - NPCs, merchants, dialogue refs, faction leaders
- `dialogues.json` - Conversation trees, ARIA personalities (consolidated)
- `traders.json` - Trader NPCs and their inventories
- `books.json` - Readable books and lore texts
- `narrative_templates.json` - Narrative generation templates
- `narrative_integration.json` - Story seeds, fragments, factions

**Quests & Progression (4 files)**:
- `quests.json` - Quest definitions, objectives
- `main_questline.json` - Main story quest definitions
- `tutorial.json` - Tutorial system configuration
- `recipes.json` - Crafting recipes

**World Objects (4 files)**:
- `chests.json` - Chest and container definitions
- `interactables.json` - Interactive objects and quest triggers
- `actions.json` - Action definitions and costs
- `constraint_rules.json` - Constraint validation rules

**UI & Rendering (3 files)**:
- `render_config.json` - Rendering system configuration
- `themes.json` - UI theme and color definitions
- `keyboard_config.json` - Keyboard configuration

### JSON Schema Validation

**All 45 data files** have corresponding JSON schemas in `schemas/`:
- Schema version validation on load
- Automatic duplicate ID detection
- Type-safe data structures
- IDE autocomplete support (with `.vscode/settings.json` configuration)

**Schema documentation**: `docs/features/*_JSON_SCHEMA_V1.md` for each schema

**Note**: Schemas are manually maintained. The `schema-gen` tool was used for initial generation only.

---

## Key Systems Detail

### Auto-Explore System

**Location**: `src/game/auto_explore.rs`

**Integration**: Called from main game loop when auto-explore is active

**Flow**:
```
Main Loop → AutoExplore::update()
  → Find nearest unexplored tile using pathfinding
  → Check for dangers and items along path
  → Move player toward target or stop for interaction
  → Handle item pickup and danger avoidance
```

**Features**:
- **Smart Pathfinding**: Uses A* to find optimal routes to unexplored areas
- **Item Detection**: Automatically picks up items when configured
- **Danger Avoidance**: Stops exploration when enemies or hazards detected
- **Configurable**: Behavior controlled via `auto_explore_config.json`
- **Visual Feedback**: Shows exploration target and path in UI

### Interaction System

**Location**: `src/game/interactable.rs`

**Integration**: Called from movement system when player interacts with objects

**Flow**:
```
Player Input → MovementSystem::try_interact()
  → InteractionSystem::handle_interaction()
  → Execute interaction based on object type
  → Update quest progress and game state
```

**Features**:
- **Data-Driven Objects**: Toggles, buttons, and quest objectives defined in JSON
- **Quest Integration**: Interactions can trigger quest progress
- **State Persistence**: Object states are saved with game state
- **Visual Feedback**: Objects show interaction prompts and state changes

### Light Manipulation System

**Location**: `src/game/light.rs`

**Integration**: Called from debug commands and item usage

**Flow**:
```
Player Action → Light System
  → Create light beams with direction and intensity
  → Trace beam paths with refraction calculations
  → Apply light effects (damage, illumination)
  → Update light sources and refraction surfaces
```

**Features**:
- **Beam Tracing**: 8-directional light beams with range and intensity
- **Refraction**: Light bends when hitting refraction surfaces
- **Light Sources**: Fixed position emitters with configurable properties
- **Player Abilities**: Focus Beam, Create Prism, Absorb Light
- **Energy System**: Light energy resource for abilities

### Void Energy System

**Location**: `src/game/void_energy.rs`

**Integration**: Called from item usage and game loop updates

**Flow**:
```
Void Exposure → Progressive Corruption
  → Unlock void abilities based on exposure level
  → Reality distortions spawn randomly
  → Void energy regeneration at high exposure
  → Cross-system interactions with light/crystal
```

**Features**:
- **Exposure Tracking**: 5 levels from None to Extreme
- **Progressive Abilities**: 5 void abilities unlock with exposure
- **Reality Distortions**: Temporal, Spatial, Material, Psychic effects
- **Void Energy**: Resource system for ability usage
- **Risk/Reward**: Power increases with corruption

### Crystal Resonance System

**Location**: `src/game/crystal_resonance.rs`

**Integration**: Called from biome generation and item usage

**Flow**:
```
Crystal Formation Generation → Player Interaction
  → Resonate with crystals to gain energy
  → Attune to different frequencies
  → Create harmonic effects between crystals
  → Plant crystal seeds to expand network
```

**Features**:
- **Five Frequencies**: Alpha, Beta, Gamma, Delta, Epsilon with unique properties
- **Crystal Growth**: Formations grow over time and can be cultivated
- **Harmonic Effects**: Multiple crystals create combined effects
- **Frequency Attunement**: Player specialization in crystal types
- **Biome Integration**: Natural crystal spawning in appropriate areas

### Event System

**Location**: `src/game/generation/events.rs`

**Integration**: Called during `end_turn()` via `check_dynamic_events()`

**Flow**:
```
end_turn() → check_dynamic_events()
  → EventSystem::check_triggers() (evaluate player state)
  → EventSystem::apply_consequences() (modify game state)
  → Track narrative momentum
  → Log event messages
```

**Event Types**:
- `player_hp_below` — Trigger when HP drops below threshold
- `biome_match` — Trigger in specific biomes
- `storm_intensity` — Trigger during intense storms
- `turn_multiple` — Trigger on specific turn intervals
- `refraction_level` — Trigger at high refraction levels

**Consequences**:
- `damage_player` — Apply damage to player
- `heal_player` — Restore player health
- `add_refraction` — Increase refraction level
- `environmental_story` — Display atmospheric messages

### Narrative System

**Location**: `src/game/generation/narrative.rs`

**Integration**: Called during `travel_to_tile()` via `generate_narrative_fragments()`

**Flow**:
```
travel_to_tile() → generate_narrative_fragments()
  → NarrativeIntegration::generate_fragments() (create story content)
  → Place fragments based on biome rules
  → Track narrative momentum
  → Log fragment discovery
```

**Components**:
- **Narrative Seeds**: 5 thematic seeds (ancient mysteries, faction conflict, etc.)
- **Story Fragments**: 8 placeable story elements with biome rules
- **Faction Influence**: 5 faction systems affecting narrative content
- **Emergent Tracking**: Momentum system driving story thread activation

### ConnectivitySystem (Glass Seam Bridging Algorithm)

**Location**: `src/game/generation/connectivity.rs`

**Integration**: Called during tile generation to ensure map connectivity

**Flow**:
```
TerrainForgeGenerator::generate_tile_with_seed() → ConnectivitySystem::ensure_connectivity()
  → Identify disconnected floor regions
  → Build connectivity graph with tunnel costs
  → Find optimal tunnel set using modified Dijkstra's algorithm
  → Create tunnels to connect regions
  → Validate coverage threshold is met
```

**Algorithm Features**:
- **Region Detection**: Flood-fill algorithm to identify disconnected areas
- **Cost Calculation**: Manhattan distance with wall-breaking penalties
- **Optimal Tunneling**: Finds minimum-cost tunnel set meeting coverage requirements
- **Validation**: Ensures specified percentage of floor tiles are reachable
- **Deterministic**: Uses seeded RNG for consistent results

**Configuration**: Coverage threshold and tunnel costs configurable via `constraint_rules.json`

### Biome System

**Location**: `src/game/generation/biomes.rs`

**Integration**: Called during `travel_to_tile()` via `generate_biome_content()`

**Flow**:
```
travel_to_tile() → generate_biome_content()
  → BiomeSystem::generate_environment_description() (create atmospheric descriptions)
  → BiomeSystem::generate_environmental_features() (1-3 features per tile)
  → BiomeSystem::check_hazards() (biome-specific dangers)
  → Log environmental content to player
```

**Components**:
- **Environmental Features**: Biome-specific terrain elements with mechanical effects
- **Atmospheric Elements**: Mood and ambiance descriptors with intensity levels
- **Hazards**: Biome-specific dangers with severity ratings
- **Resource Modifiers**: Biome effects on material availability

### Grammar System

**Location**: `src/game/generation/grammar.rs`

**Integration**: Used by `generate_biome_content()` for dynamic text generation

**Flow**:
```
generate_biome_content() → Grammar::generate()
  → Rule expansion with weighted selection
  → Variable substitution from context
  → Fallback to BiomeSystem descriptions if generation fails
```

**Components**:
- **Grammar Rules**: Hierarchical text generation rules with expansions
- **Weighted Selection**: Probability-based rule choice for variety
- **Variable Substitution**: Context-aware text replacement
- **Recursion Control**: Depth limiting to prevent infinite expansion

### Content Template System

**Location**: `src/game/generation/templates.rs`

**Integration**: Called during `travel_to_tile()` via `generate_template_content()`

**Flow**:
```
travel_to_tile() → generate_template_content()
  → TemplateLibrary::instantiate() (select template by category)
  → Apply context variables (biome, level, storm_intensity)
  → Select variant based on conditions
  → Apply inheritance and overrides
  → Log generated content to player
```

**Components**:
- **Content Templates**: Parameterized content definitions with categories
- **Template Variants**: Conditional variations with weight-based selection
- **Inheritance System**: Parent-child template relationships for reuse
- **Context Variables**: Dynamic parameter substitution from game state

### WeightedTable System

**Location**: `src/game/generation/weighted_table.rs`

**Integration**: Enhanced spawn and loot generation throughout the game

**Usage**:
- `weighted_pick_enhanced()` - Improved spawn selection
- `generate_loot_enhanced()` - Enhanced loot generation
- Used by BiomeSystem for feature selection
- Provides consistent weighted selection across all systems

**Components**:
- **Weighted Entries**: Items with associated probability weights
- **Selection Algorithm**: Deterministic weighted random selection
- **Generic Implementation**: Works with any cloneable type
- **Empty Table Handling**: Graceful failure for invalid configurations

### Combat System

**Location**: `src/game/systems/combat.rs`, `src/game/combat.rs`

**Flow**:
```
Player Input (Attack) → CombatSystem::attack_melee/ranged()
  → roll_attack() (hit/miss/crit)
  → calc_damage() (base + modifiers)
  → Apply damage to enemy
  → If HP <= 0: process_enemy_death()
    → Emit EnemyKilled event
    → Award XP
    → Trigger on_death effects
```

**Adding Combat Mechanics**: Modify `roll_attack()` or `calc_damage()` in `src/game/combat.rs`. These are pure functions that don't depend on `GameState`.

### AI System

**Location**: `src/game/systems/ai.rs`

**Pattern**: Strategy Pattern via Behavior Registry

```rust
pub trait AiBehavior: Send + Sync {
    fn execute(&self, entity_idx: usize, state: &mut GameState) -> bool;
}

static BEHAVIOR_REGISTRY: Lazy<HashMap<&str, Box<dyn AiBehavior>>> = ...;
```

**Built-in Behaviors**:
- `StandardMeleeBehavior` — Chase and melee attack
- `RangedOnlyBehavior` — Keep distance, ranged attacks
- `SuicideBomberBehavior` — Explode on contact
- `HealerBehavior` — Heal nearby allies

**Adding a Behavior**:
1. Create struct implementing `AiBehavior`
2. Register in `BEHAVIOR_REGISTRY`
3. Reference by `behavior_id` in `data/enemies.json`

### Enemy System

**Location**: `src/game/enemy.rs`

**Data Structure** (in `data/enemies.json`):
```json
{
  "id": "glass_crawler",
  "name": "Glass Crawler",
  "glyph": "c",
  "max_hp": 15,
  "damage_min": 2, "damage_max": 5,
  "sight_range": 8,
  "behaviors": [
    {"type": "split_on_death", "condition": "glass_shard", "value": 2}
  ],
  "behavior_id": "standard_melee",
  "xp_value": 25
}
```

### Quest System

**Location**: `src/game/quest.rs`, `src/game/systems/quest.rs`

**Enhanced Main Questline**: Complete 4-act structure with 13 interconnected quests covering the full narrative arc from initial discovery to cosmic endgame choice.

**Objective Types**:
- `Kill { enemy_id, count }` — Defeat specific enemies
- `Collect { item_id, count }` — Gather quest items
- `Reach { x, y }` — Travel to locations
- `TalkTo { npc_id }` — Interact with NPCs
- `InterfaceWithAria { item_required }` — Archive system interactions

**Advanced Features**:
- **Conditional Unlocking**: Quests unlock based on completed prerequisites, faction reputation, and player state
- **Adaptive Objectives**: Quest content responds to player choices and faction alignment
- **Faction Integration**: Reputation thresholds affect quest availability and dialogue options
- **Multiple Endings**: Vector Choice quest provides 4 distinct ending paths based on player decisions

**Quest Categories**:
- `main` — Core narrative questline (Acts I-IV)
- `side` — Optional content and exploration
- `faction` — Faction-specific storylines

**Key Quest Files**:
- `data/main_questline.json` — 13-quest main story arc
- `data/quests.json` — Side quests and optional content
- `data/lore_based_quests.json` — Lore-driven exploration quests

**Quest Chains**: Complex unlocking system supports branching narratives, faction choices, and prerequisite validation through `QuestCriteria` system.

### Storm System

**Location**: `src/game/storm.rs`, `src/game/systems/storm.rs`

**Edit Types** (map transformations):
- `Glass` — Convert tiles to glass
- `Rotate` — Rotate map section
- `Swap` — Swap two areas
- `Mirror` — Mirror a section
- `Fracture` — Create cracks
- `Crystallize` — Create crystal formations
- `Vortex` — Spiral pattern transformation

---

## Visual Effects DSL

Effects use a string-based DSL defined in `data/effects_config.json`.

**Syntax**: `"EFFECT(@SPEED &COLOR)"`

**Examples**:
```
"B(@3 &Cyan)"      — Blink at speed 3, cyan color
"G(&Yellow)"       — Glow yellow
"P(@2 &Red)"       — Pulse at speed 2, red
"S(@1 &White,Blue)"— Shimmer between white and blue
```

**Usage**: Effects are attached to entities and tiles via the `effects` field in data files.

---

## Entity Trait

**Location**: `src/game/entity.rs`

Unified interface for enemies and NPCs:

```rust
pub trait Entity {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn hp(&self) -> Option<i32>;
    fn status_effects(&self) -> &[StatusEffect];
    fn name(&self) -> &str;
    fn glyph(&self) -> char;
}
```

**Purpose**: Enables generic systems that operate on any entity type without type-specific code.

---

## DES Testing System

**Location**: `src/des/mod.rs`, `tests/scenarios/`

The Debug Execution System enables headless, deterministic testing.

### Scenario Structure

```json
{
  "name": "combat_basic",
  "seed": 12345,
  "map_setup": {
    "clear_radius": 5,
    "ensure_paths": [{"from_x": 10, "from_y": 10, "to_x": 11, "to_y": 10}]
  },
  "player": { "x": 10, "y": 10, "inventory": ["sword"] },
  "entities": [
    {"entity_type": "enemy", "id": "glass_crawler", "x": 11, "y": 10}
  ],
  "mocks": {
    "combat_always_hit": true,
    "combat_fixed_damage": 100
  },
  "actions": [
    {"turn": 0, "action": {"type": "attack", "target_x": 11, "target_y": 10}}
  ],
  "assertions": [
    {"at_end": true, "check": {"type": "enemy_at", "x": 11, "y": 10, "alive": false}}
  ]
}
```

### Running Tests

```bash
# Run all DES scenarios
cargo test --test des_scenarios

# Run specific scenario
cargo test --test des_scenarios combat_basic

# Run with output
cargo test --test des_scenarios -- --nocapture
```

### Writing Test Scenarios

1. Create `tests/scenarios/my_test.json`
2. Define initial state: `player`, `entities`, `map_setup`
3. Define `actions` to execute each turn
4. Define `assertions` to verify outcomes
5. Use `mocks` to control randomness if needed

### Assertion Types

- `player_alive` / `player_dead`
- `player_hp { op, value }` — Compare player HP
- `player_position { x, y }`
- `has_item { item_id }`
- `enemy_at { x, y, alive }`
- `enemy_count { op, value }`
- `quest_complete { quest_id }`
- `message_contains { text }`

---

## Game Loop

**Location**: `src/main.rs`

```
┌─────────────────────────────────────────────┐
│                  Main Loop                   │
├─────────────────────────────────────────────┤
│  1. Handle Input → Action enum               │
│  2. update(state, action)                    │
│     ├─ Movement → MovementSystem            │
│     ├─ Combat → CombatSystem                │
│     └─ etc.                                 │
│  3. state.end_turn()                        │
│     ├─ AI runs → AiSystem                   │
│     ├─ Status effects tick                  │
│     ├─ Storm progresses                     │
│     ├─ Dynamic events → EventSystem         │
│     └─ Events processed                     │
│  4. travel_to_tile()                        │
│     ├─ Biome content → BiomeSystem          │
│     ├─ Grammar generation → Grammar         │
│     ├─ Template content → TemplateLibrary   │
│     └─ Narrative fragments → NarrativeIntegration │
│  5. Render → Renderer reads GameState       │
└─────────────────────────────────────────────┘
```

---

## Decoupled Design Example: Crafting

`src/game/crafting.rs` demonstrates ideal decoupling:

```rust
// Pure function — doesn't need GameState
pub fn can_craft(recipe: &Recipe, inventory: &[String]) -> bool {
    for (item_id, &required) in &recipe.materials {
        let count = inventory.iter().filter(|id| *id == item_id).count();
        if count < required as usize { return false; }
    }
    true
}
```

**Why It's Good**:
- Takes specific data, not entire `GameState`
- Easy to test in isolation
- No side effects
- Can be called from UI without game logic coupling

---

## Adding New Content

### New Item

1. Add to `data/items.json`:
```json
{
  "id": "prism_shard",
  "name": "Prism Shard",
  "glyph": "*",
  "description": "A crystalline fragment that refracts light.",
  "value": 50,
  "usable": true,
  "heal": 10
}
```

2. That's it! Item is now spawnable and usable.

### New Enemy

1. Add to `data/enemies.json`:
```json
{
  "id": "salt_wraith",
  "name": "Salt Wraith",
  "glyph": "W",
  "max_hp": 30,
  "damage_min": 5, "damage_max": 10,
  "behavior_id": "standard_melee",
  "xp_value": 50
}
```

2. Add to spawn tables in `data/biome_spawn_tables.json` if needed.

### New Dynamic Event

1. Add to `data/dynamic_events.json`:
```json
{
  "id": "glass_resonance",
  "name": "Glass Resonance",
  "description": "High refraction causes glass to resonate",
  "triggers": [
    {
      "trigger_type": "refraction_level",
      "conditions": {"min_level": 75},
      "probability": 0.4
    }
  ],
  "consequences": [
    {
      "consequence_type": "environmental_story",
      "parameters": {
        "message": "Your crystalline skin hums with resonant energy."
      }
    }
  ],
  "weight": 1.0,
  "cooldown_turns": 15
}
```

2. That's it! Event is now active in the game loop.

### New Story Fragment

1. Add to `data/narrative_integration.json`:
```json
{
  "fragment_id": "crystal_garden",
  "narrative_seed": "adaptation_journey",
  "fragment_type": "discovery",
  "content": "Crystalline formations grow in impossible spirals, each one unique yet harmonious.",
  "placement_rules": {
    "biomes": ["saltflat", "ruins"],
    "min_distance_from_player": 8,
    "max_distance_from_player": 20,
    "requires_poi": null,
    "exclusion_zones": ["desert"]
  },
  "faction_influence": {
    "glassborn": 0.4
  },
  "prerequisites": []
}
```

2. Fragment will be placed during tile travel based on rules.

### New Grammar Rule

1. Add to `data/grammars/descriptions.json`:
```json
{
  "rules": {
    "new_rule": {
      "expansions": [
        "A <material> structure <condition>",
        "The <atmosphere> chamber <detail>"
      ],
      "weights": [60.0, 40.0]
    }
  }
}
```

2. Rule can be used in Grammar::generate("new_rule", context, rng).

### New Content Template

1. Add to `data/templates/content_templates.json`:
```json
{
  "id": "mystical_encounter",
  "category": "encounter",
  "parameters": {
    "enemy_type": "crystal_guardian",
    "description": "A ${enemy_type} emerges from the ${biome} terrain"
  },
  "variants": [
    {
      "id": "weak_guardian",
      "weight": 70.0,
      "conditions": ["level=low"],
      "overrides": {
        "enemy_count": 1
      }
    }
  ],
  "inheritance": "encounter_basic"
}
```

2. Template will be instantiated during procedural content generation.

### New Mechanic

1. Consider if it should be a new System or extend existing
2. Create/modify system in `src/game/systems/`
3. Add any new events to `GameEvent` enum
4. Write DES test scenarios
5. Update `GameState` if new data fields needed

---

## Common Patterns

### Safe Definition Lookup

```rust
// BAD: Panics if enemy def missing
let damage = enemy.def().unwrap().damage_max;

// GOOD: Handle missing gracefully
let Some(def) = enemy.def() else {
    log::warn!("Missing enemy def: {}", enemy.id);
    return;
};
let damage = def.damage_max;
```

### Deterministic Randomness

```rust
// BAD: Non-deterministic
use rand::thread_rng;
let roll = thread_rng().gen_range(1..=20);

// GOOD: Uses seeded RNG from state
let roll = state.rng.gen_range(1..=20);
```

### Spatial Index Usage

```rust
// Ensure index is fresh before queries
state.ensure_spatial_index();
if let Some(&enemy_idx) = state.enemy_positions.get(&(x, y)) {
    // Enemy found at position
}
```

---

## Advanced Gameplay Systems (NEW)

### Light Manipulation System

**Location**: `src/game/light.rs`

**Integration**: Called from debug commands and item usage

**Flow**:
```
Player Action → Light System
  → Create light beams with direction and intensity
  → Trace beam paths with refraction calculations
  → Apply light effects (damage, illumination)
  → Update light sources and refraction surfaces
```

**Features**:
- **Beam Tracing**: 8-directional light beams with range and intensity
- **Refraction**: Light bends when hitting refraction surfaces
- **Light Sources**: Fixed position emitters with configurable properties
- **Player Abilities**: Focus Beam, Create Prism, Absorb Light
- **Energy System**: Light energy resource for abilities

### Void Energy System

**Location**: `src/game/void_energy.rs`

**Integration**: Called from item usage and game loop updates

**Flow**:
```
Void Exposure → Progressive Corruption
  → Unlock void abilities based on exposure level
  → Reality distortions spawn randomly
  → Void energy regeneration at high exposure
  → Cross-system interactions with light/crystal
```

**Features**:
- **Exposure Tracking**: 5 levels from None to Extreme
- **Progressive Abilities**: 5 void abilities unlock with exposure
- **Reality Distortions**: Temporal, Spatial, Material, Psychic effects
- **Void Energy**: Resource system for ability usage
- **Risk/Reward**: Power increases with corruption

### Crystal Resonance System

**Location**: `src/game/crystal_resonance.rs`

**Integration**: Called from biome generation and item usage

**Flow**:
```
Crystal Formation Generation → Player Interaction
  → Resonate with crystals to gain energy
  → Attune to different frequencies
  → Create harmonic effects between crystals
  → Plant crystal seeds to expand network
```

**Features**:
- **Five Frequencies**: Alpha, Beta, Gamma, Delta, Epsilon with unique properties
- **Crystal Growth**: Formations grow over time and can be cultivated
- **Harmonic Effects**: Multiple crystals create combined effects
- **Frequency Attunement**: Player specialization in crystal types
- **Biome Integration**: Natural crystal spawning in appropriate areas

### System Integration Points

#### Cross-System Interactions
- **Light + Crystal**: Light beams can charge crystals, crystals refract light
- **Void + Crystal**: Void corruption affects crystal stability
- **Light + Void**: Light can counteract void effects partially

#### Discovery Mechanics
- **Items**: Light Crystal, Void Shard, Resonance Tuner teach systems
- **Quests**: Tutorial quests guide players through each system
- **World Generation**: Crystal formations spawn naturally in biomes
- **Progressive Unlocking**: Systems unlock through exposure and usage

---

## Bracket-lib Integration

**Status**: Phase 3 of Map Generation Overhaul (In Progress)

The game is transitioning from the `noise` crate to `bracket-lib` for procedural generation, providing access to advanced algorithms and better performance.

### Integration Architecture

**Location**: `src/game/generation/adapters/`

**Components**:
- `bracket_adapter.rs` — Core adapter interface
- `bracket_integration.rs` — Integration layer for bracket-lib algorithms

### Current Status

✅ **Completed**:
- Replaced Perlin noise with bracket-noise FastNoise
- Integrated bracket-lib noise functions into tile generation
- Removed old noise crate dependency
- Fixed noise range compatibility issues

🚧 **In Progress**:
- BSP room generation implementation
- Cellular Automata cave generation
- Algorithm validation and testing

### Algorithm Support

| Algorithm | Status | File Location |
|-----------|--------|---------------|
| FastNoise (Perlin) | ✅ Complete | `generation/algorithms/perlin_noise.rs` |
| BSP Rooms | 🚧 In Progress | `generation/structures/algorithms/bsp.rs` |
| Cellular Automata | 🚧 In Progress | `generation/structures/algorithms/cellular_automata.rs` |
| Dungeon Generator | ✅ Complete | `generation/structures/dungeon_generator.rs` |

### Testing Framework

**Enhanced Tile Generation Test Suite**: Comprehensive testing framework with:
- Algorithm-specific test configurations
- PNG visualization output
- Quality metrics and evaluation
- Custom report generation per algorithm

**Usage**:
```bash
# Test specific algorithms
cargo test bsp_algorithm_test_suite
cargo test cellular_automata_test_suite

# Generate test reports
cargo run --bin tilegen-test-tool -- --config enhanced-tile-test-suite/configs/bsp_basic.json
```

### Testing Tools

**Tile Generation Testing**: `tilegen-test-tool` and `tilegen-tool` provide comprehensive testing capabilities:

```bash
# Test terrain generation with various biomes and POIs
cargo run --bin tilegen-tool tile 12345 town desert
cargo run --bin tilegen-tool tile 12345 shrine saltflat

# Run enhanced evaluation system
cargo run --bin tilegen-test-tool -- --config test_config.json

# Generate comprehensive test suite with PNG output
cargo test enhanced_tile_generation_test_suite
```

**Features**:
- **Visual Output**: PNG generation for terrain visualization
- **Quality Metrics**: Connectivity, variety, and constraint validation
- **Algorithm Testing**: Support for BSP, Cellular Automata, and custom algorithms
- **Deterministic Testing**: Seeded generation for reproducible results

---

## Related Documentation

- [ARCHITECTURE_AUDIT.md](./ARCHITECTURE_AUDIT.md) — Technical audit and recommendations
- [systems_analysis.md](./systems_analysis.md) — Detailed refactoring history and anti-patterns
- [SCALABILITY_AUDIT.md](./SCALABILITY_AUDIT.md) — Performance considerations
- [TECH_STACK.md](./TECH_STACK.md) — Technology choices
- [NEW_SYSTEMS_DOCUMENTATION.md](../development/NEW_SYSTEMS_DOCUMENTATION.md) — Complete documentation for Light, Void, and Crystal systems
- [PROCEDURAL_GENERATION_COMPREHENSIVE_GUIDE.md](../development/PROCEDURAL_GENERATION_COMPREHENSIVE_GUIDE.md) — Complete procedural generation guide
- [GLASS_SEAM_BRIDGING_ALGORITHM.md](../development/GLASS_SEAM_BRIDGING_ALGORITHM.md) — Glass Seam Bridging Algorithm documentation
- [CONSTRAINT_SYSTEM_GUIDE.md](../development/CONSTRAINT_SYSTEM_GUIDE.md) — Constraint validation system guide
- [AUTO_EXPLORE_SYSTEM.md](../development/AUTO_EXPLORE_SYSTEM.md) — Auto-exploration system documentation

---

## Glossary

| Term       | Definition                                                              |
| ---------- | ----------------------------------------------------------------------- |
| DES        | Debug Execution System — headless testing framework                     |
| GameState  | Central data struct holding all game state                              |
| System     | Stateless module that operates on GameState                             |
| Adaptation | Player mutation/upgrade that grants abilities                           |
| Storm      | Glass storm event that transforms the map                               |
| Entity     | Trait unifying Enemy/NPC with common interface                          |
| Behavior   | AI behavior strategy for enemies                                        |
| DSL        | Domain-Specific Language (used for visual effects)                      |
| Light Energy | Resource used for light manipulation abilities                         |
| Void Exposure | Corruption level that unlocks void abilities                          |
| Crystal Frequency | One of five resonance types (Alpha, Beta, Gamma, Delta, Epsilon)    |
| Refraction Surface | Light-bending surface created by player abilities                    |
| Reality Distortion | Void-induced environmental effect                                    |
| BSP        | Binary Space Partitioning — room-based dungeon generation algorithm     |
| Cellular Automata | Cave generation algorithm using iterative rules                      |
| Glass Seam Bridging | Algorithm ensuring map connectivity through optimal tunneling        |
| Bracket-lib | Rust library providing advanced procedural generation algorithms       |
| Algorithm Registry | Plugin system for swappable generation algorithms                    |
| Constraint System | Validation system ensuring generated content meets requirements       |
| Quest Constraints | Generation rules driven by active quest requirements                 |
| Microstructures | Small procedural structures placed within larger generated areas      |
| Interactable | Data-driven interactive objects (buttons, toggles, quest triggers)    |
