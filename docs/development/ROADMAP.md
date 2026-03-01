# Development Roadmap

> Last updated: 2026-03-01

## Current State

The codebase has been through a major maintainability overhaul (Phases 1–5). The monolithic `state.rs` has been decomposed into `PlayerState`, `WorldState`, and `NarrativeEngine`. Dead code, stub systems, and duplicate implementations have been removed. The game compiles with zero warnings and 161 tests pass.

A 30-minute play session is possible: character creation, movement, combat, quests, inventory, crafting, trading, storms, and auto-explore all work. The first quest ("The Pilgrim's Last Angle") is completable. Quest progression beyond that is blocked by missing infrastructure.

---

## Recently Completed

### Tile Generator Refactor + Tile-Gen Tester (2026-03-01) ⏳ Awaiting Approval
- **Refactor**: Extracted `generate_tile()` from `travel_to_tile()` into `src/game/generation/tile_generator.rs`. `travel_to_tile` is now a thin wrapper. Generation is now independently callable without a full `GameState`.
- **TileTestConfig**: JSON-driven test configs in `data/tile_tests/` (21 configs covering all biomes, POIs, factions)
- **In-game tester**: "Test Tile Generation" option in main menu → config sub-menu → loads tile directly into game session with `test_mode=true` (exits blocked, save disabled)
- **Documentation**: `docs/development/TILE_GENERATOR_REFACTOR_PLAN.md`

### Settlement Generation — Complete (2026-02-22 to 2026-03-01)
- **Unified StructureLibrary**: Replaced prefab system with single JSON+txt hybrid loading
- **Grid-with-jitter layout**: Replaced terrain-forge BSP/Voronoi (all algorithms produce one connected region, not isolated plots)
- **Building placement**: Weighted random by faction, `to_snake_case()` normalization for faction ID lookup
- **Faction integration**: Dominant faction determines building selection and settlement aesthetic
- **NPC spawning**: Per-building `npc_types` from `structures.json` metadata
- **Decorations**: Faction-themed decoration placement
- **World map integration**: `Map::generate_from_world()` detects `POI::Town`, calls `generate_settlement()`
- **mapgen-tool**: `settlement <seed> <tier>` renders full ASCII map + building list
- **Documentation**: `docs/features/SETTLEMENT_GENERATION.md`, `docs/features/SETTLEMENT_GENERATION_SUMMARY.md`
- **Future work logged**: `docs/development/SETTLEMENT_FUTURE_WORK.md`

### Settlement Generation Foundation (2026-02-22 to 2026-03-01)
- **Prefab Library System**: Implemented JSON-based building template system with rotation, mirroring, and validation
- **Building Content**: Created 35 building prefabs (14 core + 21 faction-specific)
- **Settlement Tiles**: Added 5 wall types and 4 floor types for settlement construction
- **Furniture & Decorations**: Added 12 furniture types as interactables
- **Settlement Module**: Implemented core data structures (SettlementConfig, Settlement, Building) with tier system (Village/Town/City)
- **Configuration**: Created settlement_config.json with tier parameters and faction building mappings
- **Unified Structure System**: Designed consolidation of structure_templates and prefabs into single system with hybrid loading (external .txt files + inline JSON)
- **Documentation**: `docs/development/SETTLEMENT_GENERATION_PLAN.md`, `docs/development/PREFAB_SYSTEM_DESIGN.md`, `docs/development/UNIFIED_STRUCTURE_SYSTEM.md`

### Cleanup (2026-02-15)
- Removed ritual/sanity placeholder systems and orphaned DES test files
- Removed 7 dead generation stub structs from NarrativeEngine
- Removed ~200 lines of commented-out generation code from state.rs
- Eliminated 4 compiler warnings (now zero)
- Consolidated FOV into single bracket-lib shadowcasting implementation (deleted duplicate `fov.rs`)

### Bug Fixes (2026-02-15)
- BUG-004: Tile renderer now uses FOV (`state.visible`) instead of light distance — player can no longer see through walls
- BUG-002: Quest objectives enforce sequential completion — can't skip objectives
- BUG-001: Auto-explore avoids NPCs with quest interaction history
- BUG-006: Tutorial system wired into game loop with real trigger conditions
- BUG-003: Player spawn point clamped away from map edges
- BUG-005: Look mode shows all entities on a tile, not just the first
- Tutorial dismiss fix: `has_shown` now keyed by message id, not trigger string

### Phase 5: State.rs Decomposition (2026-02-14 to 2026-02-15)
- Extracted `PlayerState` (25 fields), `WorldState` (20 fields), `NarrativeEngine` (5 fields)
- 997 compilation errors fixed via systematic bulk sed scripts (~11 minutes)
- Phase 4 feature integration: tutorial overlay, faction menu, void/crystal menus, crafting stations

---

## Remaining Technical Debt

| Issue | Severity | Estimate |
|-------|----------|----------|
| 97+ `unwrap()` in non-test code | High | 2–3 days |
| ~15 functions >100 lines in state.rs | Medium | Ongoing |
| 12 disabled DES scenarios | Low | 1 day |
| NarrativeEngine QuestLog is a stub duplicate of real quest_log | Medium | 1 day |
| Stale docs and lore files | Low | 1 day |

### Data File Audit ✅ **COMPLETED** (2026-02-21)

**Findings**:
- Audited 54 JSON files in `data/` directory
- Removed 6 dead files (11% reduction): `npc_spawn_config.json`, `quest_constraints.json`, `structure_spawn_config.json`, `grammars/*.json`, `templates/content_templates.json`
- Identified 49 active files (91%)
- `biome_profiles.json` unused but kept (valid future feature for environmental storytelling)
- `structure_generation.json` confirmed test-only (tilegen-tool.rs)
- No consolidation needed - `biome_profiles.json` and `terrain_config.json` serve different purposes

**Documentation**: `docs/development/DATA_FILE_AUDIT.md`

---

## Feature Roadmap

Features are grouped into tiers. Each tier builds on the previous one. Within a tier, items can be worked in parallel.

### Tier 1 — Core Systems Rework

These are foundational systems that most other features depend on. They should be tackled first.

#### 1. Save/Load Game Library with Versioning ✅
- `src/game/save.rs`: `SaveFile` envelope with `SAVE_VERSION` (currently v1) + `GameState`
- `save_game()` / `load_game()` replace raw `state.save()` / `GameState::load()`
- Version mismatch returns descriptive error; corrupt saves caught at deserialization
- Wired into `main.rs` action handlers
- Future: migration functions between versions, integrity checks, auto-save, save slots

#### 2. Proper Overworld Travel ✅

**Step 1 ✅ — Adjacent movement with terrain travel costs**
- `src/game/travel.rs`: data-driven travel cost from `data/travel_config.json`
- `is_adjacent()` restricts world map movement to cardinal neighbors
- `travel_cost(terrain, biome)` = base terrain cost + biome modifier (min 1)
- `travel_to_tile_safe` rejects non-adjacent, advances turns, logs travel

**Step 2 ✅ — Random encounters during travel**
- `src/game/encounter.rs`: deterministic encounter generation using world seed
- Three encounter types: Hostile (50%), Neutral (30%), Beneficial (20%)
- Threat/boon point budget system for enemy/item spawning
- Flee mechanic with cooldown and distance requirements
- Encounter completion checking and XP rewards
- Fast worldmap movement with deferred tile generation
- Direct arrow key movement and inspect mode with pathfinding
- Keyboard configuration system for worldmap controls
- 25% base encounter rate, 50-turn cooldown per tile

**Remaining steps:**
- Resource consumption (water, food) during travel
- Camp/rest mechanic during long journeys
- **Depends on**: Save system ✅

#### 3. Actual Skill Catalog
- Design and implement full skill tree with meaningful choices
- Skill categories: Combat, Survival, Social, Psychic, Crafting
- Active skills (usable abilities) and passive skills (stat modifiers)
- Skill synergies and prerequisites
- Skill point allocation UI already exists — needs real skill definitions
- **Depends on**: Nothing (standalone system)

#### 4. Proper Faction System ✅ **COMPLETED** (2026-02-21)
- ✅ Load and use `factions.json` definitions (7 factions)
- ✅ Faction reputation system (-100 to +100 scale)
- ✅ Starting reputation by character class
- ✅ Faction territories on world map (Voronoi division, neutral center)
- ✅ Faction overlay on world map (F key toggle)
- ✅ Quest reputation rewards
- ✅ Faction menu UI with exact numbers and color-coded standings
- ✅ Save system migration (v1 → v2)
- ✅ Reputation affects: NPC dialogue, quest availability, shop prices (existing integrations)
- 🔲 Enemy faction tags (deferred - see TODO list)
- 🔲 Faction-specific quests and storylines (future content)
- 🔲 Reputation decay/growth over time (future enhancement)
- **Documentation**: `docs/features/FACTION_SYSTEM.md`

### Tier 2 — Content & Generation

These features fill the world with meaningful content. They depend on Tier 1 foundations.

#### 5. Procedural Village/Town/City Generation ✅ **COMPLETE** (2026-03-01)

- ✅ Unified StructureLibrary with hybrid loading (.txt pattern files + inline JSON)
- ✅ 35 building prefabs (14 core + 21 faction-specific)
- ✅ Grid-with-jitter layout algorithm (terrain-forge BSP/Voronoi abandoned — produces one connected region, not isolated plots)
- ✅ Building placement with faction-weighted selection and `to_snake_case()` normalization
- ✅ Faction integration: dominant faction determines building selection and aesthetic
- ✅ NPC spawning from per-building `npc_types` in `structures.json` metadata
- ✅ Decoration placement with faction theming
- ✅ World map integration: `POI::Town` triggers `generate_settlement()`
- ✅ mapgen-tool `settlement` command for iteration
- ✅ Save/load: deterministic from `tile_seed`, no special handling needed
- 🔲 Building interiors (deferred — z-level approach, logged in `SETTLEMENT_FUTURE_WORK.md`)
- 🔲 Furniture micro-prefab system (deferred, low priority)
- **Documentation**: `docs/features/SETTLEMENT_GENERATION.md`, `docs/features/SETTLEMENT_GENERATION_SUMMARY.md`

#### 5.5. Biome-Driven Tile Generation Profiles ✅

> ⚠️ **READ TERRAIN-FORGE DOCUMENTATION TO SEE WHAT IS ALREADY IMPLEMENTED** before starting work on any procedural generation task. Many algorithms and features may already exist in the library.
- **Status**: Implemented (2026-02-15)
- Replaced random algorithm switching with data-driven biome+terrain profiles in `terrain_config.json`
- Each biome defines weighted algorithm preferences (cellular, bsp, rooms) with per-terrain overrides
- POI types (town, dungeon, shrine, landmark) have separate algorithm overrides that take priority
- Natural biomes (desert, saltflat) favor cellular automata; structured biomes (ruins) favor BSP/rooms
- `generate_with_dungeon_generator` now uses biome-aware selection instead of hardcoded BSP
- All weights tunable in `data/terrain_config.json` without recompilation

#### 5.6. Algorithm Layering & Composition
- Allow biome profiles to specify multiple algorithms as ordered layers (e.g. Perlin noise base → cellular detail pass → GSB connectivity)
- Data-driven layer definitions in `terrain_config.json`: each layer specifies algorithm, params, and blend mode
- Desert example: Perlin noise with large scale + low threshold as base, glass seam bridging as connectivity pass
- Enable per-biome algorithm parameter overrides (noise scale, floor threshold, room sizes) in the profile
- Testable via `tilegen-tool` with `--biome` and `--terrain` flags for visual comparison
- **Depends on**: Biome-driven profiles (5.5), data file audit (tech debt)

#### 6. Mob and Item Spawn Table Update
- Biome-specific enemy rosters with level scaling
- Elite and rare enemy variants with unique loot
- Environmental spawns (glass crawlers near glass, salt wraiths in flats)
- Dynamic spawn density based on player level and area danger
- Item rarity tiers affecting spawn weights
- **Depends on**: Tiered mobs, tiered loot (co-developed)

#### 7. Tiered Mob Overhaul
- Enemy tier system: Common → Uncommon → Rare → Elite → Boss
- Tier affects: stats, AI complexity, loot quality, XP reward
- Visual indicators for tier (glyph color/style)
- Tier-appropriate AI behaviors (elites use abilities, bosses have phases)
- Named enemies with unique mechanics
- **Depends on**: Nothing (standalone, but informs spawn tables)

#### 8. Tiered Loot Overhaul
- Item rarity: Common → Uncommon → Rare → Epic → Legendary
- Rarity affects: stat ranges, special properties, visual indicators
- Procedural affixes: "Burning Glass Sword of the Storm"
- Set items with bonuses for wearing multiple pieces
- Unique/legendary items with lore and special abilities
- Loot quality scaling with area danger and enemy tier
- **Depends on**: Tiered mobs (loot drops from tiered enemies)

#### 9. Adaptations Rework
- Redesign mutation system with clearer progression paths
- Adaptation trees: Glass, Salt, Storm, Void, Light
- Each tree has 5–8 nodes with prerequisites
- Social consequences: visible mutations affect NPC reactions and faction standing
- Adaptation conflicts: some mutations are mutually exclusive
- Environmental interactions: adaptations change how biomes affect you
- **Depends on**: Faction system (social consequences), skill catalog (integration)

#### 10. Storm System Rework
- Predictable storm cycles with forecast system
- Storm types: Glass Storm, Salt Squall, Light Flare, Void Rift
- Each type has unique map edits and gameplay effects
- Storm intensity scaling with world progression
- Storm shelters and preparation mechanics
- Storm-created opportunities (new paths, revealed secrets, rare spawns)
- **Depends on**: Tiered mobs (storm spawns), adaptations (storm resistance)

### Tier 3 — Combat & Economy

#### 11. Ranged/Throw Weapon Overhaul
- Proper projectile system with trajectory and obstacles
- Weapon types: bows, crossbows, thrown weapons, slings
- Ammunition system with crafting
- Range penalties and bonuses
- Line-of-sight integration with FOV
- Special ammo types (glass-tipped, salt-coated, light-infused)
- **Depends on**: Tiered loot (weapon tiers), skill catalog (ranged skills)

#### 12. Trader Overhaul
- Dynamic pricing based on supply, demand, and faction reputation
- Trader inventories that refresh and respond to world state
- Specialized traders: weaponsmith, alchemist, scribe, glass-worker
- Haggling mechanic using social skills
- Black market traders for illegal/rare goods
- Caravan system: traders travel between settlements
- **Depends on**: Faction system (prices), settlement generation (trader placement), tiered loot

#### 12.5. Improved Biome Variance
- Distinct visual identity per biome (unique glyphs, color palettes, tile types)
- Biome-specific hazards, resources, and encounters
- Transition zones between biomes with blended features
- Weather patterns per biome affecting gameplay
- Biome-specific structures and points of interest
- Seasonal/cyclical biome changes
- **Depends on**: Storm rework (weather), settlement generation (biome-appropriate buildings)

### Tier 4 — Narrative & Progression

#### 13. 50–60% Complete Main Questline
- Implement Acts I–II fully (quests 1–7 of 13)
- Quest-driven location spawning: required structures appear in world
- ARIA interface system for archive interactions
- Missing NPCs: the_architect, high_prism, custodian_iri_7, sable_of_the_seam
- Key items: broken_saint_key, aria_core_fragment, lens_of_the_first_saint
- Act II boss encounter with phase mechanics
- Multiple dialogue paths based on faction alignment
- **Depends on**: Settlement generation (quest locations), faction system (dialogue), tiered mobs (bosses)

#### 14. Procedurally Generated Quests
- Quest template system: fetch, kill, escort, explore, defend
- Dynamic objectives based on world state and player location
- Procedural quest givers with generated names and motivations
- Reward scaling based on quest difficulty and player level
- Quest chains: completing one procedural quest can spawn follow-ups
- Integration with faction reputation (faction-specific procedural quests)
- **Depends on**: Main questline (quest infrastructure), faction system, settlement generation

#### 15. Procedurally Generated Lore
- Lore fragment system: books, inscriptions, NPC dialogue, item descriptions
- Grammar-based text generation for environmental descriptions
- Consistent world history generated per seed
- Discoverable lore that reveals game mechanics and world secrets
- Lore connections: fragments reference each other across locations
- Integration with quest system (lore discoveries can trigger quests)
- **Depends on**: Main questline (lore framework), biome variance (location-specific lore)

---

## Development Principles

- **Determinism first**: All new systems must use seeded RNG from GameState
- **Data-driven**: Content in JSON, mechanics in Rust — tune without recompilation
- **Test with DES**: Every new feature gets at least one DES scenario
- **Versioned saves**: Every GameState change must include a migration path
- **Zero warnings**: `cargo clippy` clean before every commit
