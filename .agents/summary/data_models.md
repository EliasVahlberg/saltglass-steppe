# Data Models

## Core State

- **GameState** — Central hub. Fields: `player: PlayerState`, `world: WorldState`, `narrative: NarrativeEngine`, `rng: ChaCha8Rng`, `event_queue: Vec<GameEvent>`, `messages: Vec<GameMessage>`, `visible: HashSet<usize>`, `revealed: HashSet<usize>`, `turn`, `light_map`, `triggered_effects`, `decoys`, `wait_counter`, `map_features`, `meta: MetaProgress`. Spatial indices (`enemy_positions`, `npc_positions`, `item_positions`, `chest_positions`, `interactable_positions`) are `#[serde(skip)]` and rebuilt on load. Mock fields for DES testing: `mock_combat_hit`, `mock_combat_damage`.

- **PlayerState** — `x/y/layer` position, `hp/max_hp`, `ap/max_ap`, `reflex`, `armor`, `level`, `xp`, `pending_stat_points`, `salt_scrip` (currency), `inventory: Vec<String>`, `equipped_weapon`, `equipment: Equipment`, `refraction`, `adaptations: Vec<Adaptation>`, `status_effects`, `faction_reputation: HashMap<String, i32>`, `quest_log: QuestLog`, `skills: SkillsState`, `psychic: PsychicState`, `light_system: LightSystem`, `void_system: VoidSystem`, `crystal_system: CrystalSystem`, `last_damage_dealt`.

- **WorldState** — `map: Map`, `world_map: Option<WorldMap>`, `world_x/world_y/layer`, `enemies: Vec<Enemy>`, `npcs: Vec<Npc>`, `items: Vec<Item>`, `chests: Vec<Chest>`, `interactables: Vec<Interactable>`, `microstructures`, `storm: Storm`, `time_of_day`, `weather: Weather`, `ambient_light`, `visual_effects`, `encounter_state`, `encounter_history: HashMap<(usize,usize), u32>`, `total_tiles_traveled`, world map pathfinding state. Spatial indices are `#[serde(skip)]`, rebuilt via `ensure_spatial_index()`.

- **NarrativeEngine** — `quest_log` (active/completed quest IDs), `story_model` (current chapter, story flags), `tutorial_progress`, `world_history` (events, timeline), `triggered_effects` (active effects, timers).

## Entity Models

- **Enemy** — `id`, `x/y` position, `hp`, `status_effects`. References `EnemyDef` (name, glyph, stats, behavior, demeanor, loot entries, spawn conditions).
- **Npc** — `id`, `x/y` position, `hp`. References `NpcDef` with dialogue entries and actions.
- **Item** — `id`, `x/y` position. References `ItemDef` with effects and light sources.
- **Chest** — `id`, items, lock state. References `ChestDef`.
- **Interactable** — `id`, `x/y` position. References `InteractableDef` with examine/interact messages.

## World Map

- **WorldMap** — Grid of `WorldTile`.
- **WorldTile** — `biome`, `terrain`, optional `poi`, `resources`, `connections`, `level`.
- **Biome** enum: `Saltflat`, `Desert`, `Oasis`, `Ruins`, `Scrubland`.
- **Terrain** enum: `Canyon`, `Mesa`, `Hills`, `Dunes`, `Flat`.
- **POI** enum: `Town`, `Shrine`, `Landmark`, `Dungeon`, `None`.

## Quest Models

- **QuestDef** — `id`, `name`, `description`, `objectives`, `rewards`, `criteria`.
- **ObjectiveType** — `Kill`, `Collect`, `Reach`, `TalkTo { npc_id }`, `Examine`, `Interact`, `Explore`.
- **ActiveQuest** — def reference, progress per objective, current act.
- **QuestLog** — `active: Vec<ActiveQuest>`, `completed: Vec<String>`, story choices, faction alignment.

## Combat Models

- **CombatResult** — hit/miss, damage dealt, status applied.
- **WeaponDef** — `id`, damage, accuracy, AP cost, range.
- **StatusEffect** — type, duration, stacks, tick behavior.

## Progression Models

- **SkillsState** — skill levels, passive bonuses, unlocked abilities. Ticks per turn.
- **Adaptation** — `id`, references `AdaptationDef` with stat modifiers, effects, immunities.
- **PsychicState**, **LightSystem**, **VoidSystem**, **CrystalSystem** — Specialized subsystems, each with per-turn `update()`/`tick()`.

## Generation Models

- **TileParams** — `seed`, `biome`, `terrain`, `elevation`, `poi`, `level`, `quest_ids`. Built from `WorldState` via `from_world_state()`.
- **GeneratedTile** — `map: Map`, `enemies`, `items`, `npcs`, `chests`, `spawn_pos`, `walkable_positions`.
- **BiomeAlgorithmProfile** — algorithm layers, modifiers per biome. Loaded from `biome_profiles.json`.
- **Structure** — pattern, metadata (faction, usage, tags), placement rules.

## Data Files

| File | Model | Purpose |
|------|-------|---------|
| `items.json` | ItemDef | All items |
| `enemies/{common,uncommon,rare,elite,boss}.json` | EnemyDef | Enemies by rarity tier |
| `npcs.json` | NpcDef | NPCs with dialogue refs |
| `quests.json` | QuestDef | Side quests |
| `main_questline.json` | MainQuestlineFile | Main story |
| `dialogues.json` | DialoguesFile | Dialogue trees |
| `factions.json` | Faction | Factions and standings |
| `skill_trees.json` | SkillTreesFile | Skill tree structure |
| `abilities.json` | AbilitiesFile | Active abilities |
| `adaptations.json` | AdaptationsFile | Mutations |
| `terrain_config.json` | TerrainConfig | Biome terrain params |
| `biome_profiles.json` | BiomeProfilesFile | Generation algorithm profiles |
| `biome_spawn_tables.json` | SpawnTables | Per-biome enemy/item spawns |
| `constraint_rules.json` | ConstraintRulesFile | Map validation rules |
| `weapons.json` | WeaponDef | Weapon stats |
| `effects.json` | EffectsFile | Visual/gameplay effects |
| `recipes.json` | RecipesFile | Crafting recipes |
| `traders.json` | TraderTable | Shop inventories |
| `loot_tables.json` | LootTable | Drop tables |
| `books.json` | BookData | In-game books |
| `structures/structures.json` | StructureLibrary | Prefab structure definitions |
| `microstructures.json` | MicroStructures | Small feature definitions |
| `environmental_props.json` | EnvironmentalProps | Visual decorations |
| `map_elements.json` | MapElements | Map element definitions |
| `map_features.json` | MapFeatures | Hidden locations, safe routes |
| `dynamic_events.json` | DynamicEvents | Random event definitions |
| `narrative_templates.json` | NarrativeTemplates | Procedural text generation |
| `narrative_integration.json` | NarrativeIntegration | Story integration config |
| `encounter_config.json` | EncounterConfig | Encounter trigger rules |
| `travel_config.json` | TravelConfig | World travel costs |
| `storm_config.json` | StormConfig | Storm behavior params |
| `render_config.json` | RenderConfig | Rendering settings |
| `themes.json` | ThemeConfig | Color themes |
| `keyboard_config.json` | KeyboardConfig | Key bindings |
| `tutorial.json` | TutorialConfig | Tutorial triggers |
| `classes.json` | ClassDef | Character classes |
| `progression.json` | ProgressionConfig | XP/level curve |
| `actions.json` | ActionDef | Action definitions |
| `auto_explore_config.json` | AutoExploreConfig | Auto-explore settings |
| `character_names.json` | CharacterNames | Name generation pools |
| `interactables.json` | InteractableDef | Interactable objects |
| `chests.json` | ChestDef | Chest definitions |

Structure patterns stored as text files in `data/structures/patterns/{core,ruins,special}/`.
Tile test configs in `data/tile_tests/*.json`.

## Save Format

- **Format**: RON (Rusty Object Notation), not JSON.
- **File**: `saves/<md5_hash>.ron` — filename is the MD5 of file content (tamper detection).
- **Structure**: `SaveFile { version: u32, state: GameState }`.
- **Version**: Currently `SAVE_VERSION = 3`. Older versions migrated via `migrate_save()`. Future versions rejected.
- **Meta**: `saves/meta.json` tracks per-save status (`Ok`/`HashMismatch`/`Corrupt`), timestamp, character name, save version.
- **Load flow**: Read file → compute MD5 → compare to filename → deserialize RON → migrate if needed → `rebuild_spatial_index()` → `update_lighting()`.
