# Development Roadmap

> Last updated: 2026-02-15

## Current State

The codebase has been through a major maintainability overhaul (Phases 1–5). The monolithic `state.rs` has been decomposed into `PlayerState`, `WorldState`, and `NarrativeEngine`. Dead code, stub systems, and duplicate implementations have been removed. The game compiles with zero warnings and 133 tests pass.

A 30-minute play session is possible: character creation, movement, combat, quests, inventory, crafting, trading, storms, and auto-explore all work. The first quest ("The Pilgrim's Last Angle") is completable. Quest progression beyond that is blocked by missing infrastructure.

---

## Recently Completed

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

---

## Feature Roadmap

Features are grouped into tiers. Each tier builds on the previous one. Within a tier, items can be worked in parallel.

### Tier 1 — Core Systems Rework

These are foundational systems that most other features depend on. They should be tackled first.

#### 1. Save/Load Game Library with Versioning
- Replace current raw serde roundtrip with a versioned save format
- Schema version header in save files; migration functions between versions
- Backward compatibility: load saves from older versions, reject incompatible futures
- Save file integrity check (checksum or hash)
- Auto-save on world map travel and configurable save slots
- **Why first**: Every subsequent feature changes GameState — versioned saves prevent breaking player progress

#### 2. Proper Overworld Travel
- Replace current instant tile-transition with actual overworld movement
- Travel time based on distance, terrain, and encumbrance
- Random encounters during travel (bandits, storms, wildlife)
- Resource consumption (water, food) during travel
- Visible travel path on world map with waypoints
- Camp/rest mechanic during long journeys
- **Depends on**: Save system (travel state must persist)

#### 3. Actual Skill Catalog
- Design and implement full skill tree with meaningful choices
- Skill categories: Combat, Survival, Social, Psychic, Crafting
- Active skills (usable abilities) and passive skills (stat modifiers)
- Skill synergies and prerequisites
- Skill point allocation UI already exists — needs real skill definitions
- **Depends on**: Nothing (standalone system)

#### 4. Proper Faction System
- Load and use `factions.json` definitions (currently orphaned data)
- Faction reputation affects: NPC dialogue, quest availability, shop prices, enemy aggression
- Faction-specific quests and storylines
- Faction alignment choices with mutually exclusive paths
- Faction territory on world map influencing encounters and generation
- Reputation decay/growth over time based on actions
- **Depends on**: Overworld travel (faction territories), skill catalog (social skills)

### Tier 2 — Content & Generation

These features fill the world with meaningful content. They depend on Tier 1 foundations.

#### 5. Procedural Village/Town/City Generation
- Settlement generation algorithm: layout, buildings, NPCs, services
- Settlement tiers: camp (2–3 NPCs) → village (5–10) → town (15–25) → city (30+)
- Building types: tavern, smithy, temple, archive, market, barracks
- NPC placement with roles (merchant, quest-giver, guard, civilian)
- Settlement economy: supply/demand affecting prices
- Walls, gates, and districts for larger settlements
- Integration with world map POI system
- **Depends on**: Faction system (settlements belong to factions), overworld travel

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
