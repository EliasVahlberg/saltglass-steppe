---
status: current
last_verified: 2026-04-26
commit: 3fdbafb
---

# Development Roadmap

> Last updated: 2026-04-26

---

## Technical Debt Backlog

> Added 2026-04-06 after VERA refactor + state.rs decomposition. See `STATE_STORE_REFLECTION.md` for full analysis.

### Immediate bugs ✅ ALL FIXED (2026-04-06)

| Item | Status |
|------|--------|
| `teleport` DES action doesn't call `check_auto_complete` | ✅ Fixed — `src/des/mod.rs` |
| `dying_pilgrim` bump-to-talk not setting `talked` flag | ✅ Fixed — `src/game/systems/movement.rs` |
| Dialogue condition check not logging to message log | ✅ Fixed — `src/game/npc.rs` (unknown condition fields now fail-closed) |

### DES scenario fixes ✅ ALL FIXED (2026-04-06)

All 7 wrong-test-data scenarios fixed. All 7 broken-JSON-format scenarios fixed or deleted. All 3 missing-data scenarios deleted. Dungeon connectivity bug fixed (GSB weight calculation + DES `poi_type` support).

**DES test status as of 2026-04-26**: 142 passing, 2 ignored (`run_all_scenarios` — meta-test; `storm_glass_drops` — probabilistic drop, flaky under parallel execution).

### False-positive DES scenarios (low priority)

20 scenarios assert only `player_alive`. They catch crashes but not logic errors. Each needs either real assertions added or deletion:

`animation_effects_test`, `basic_movement`, `biome_system_basic`, `constraint_system_basic`, `effects_config_test`, `event_system_basic`, `generation_pipeline_basic`, `grammar_generation_basic`, `microstructures_test`, `narrative_integration_basic`, `procedural_effects_test`, `spawn_distribution_test`, `storm_timer_countdown`, `system_integration_test`, `template_system_basic`, `test_renderer_frame`, `theme_system_test`, `tutorial_messages_display`, `world_tile_transition` (delete — no actions, no assertions), `base_empty_room` (delete — no assertions).

### Architecture debt (deferred from VERA refactor)

See `STATE_STORE_REFLECTION.md` §3 for the exhaustive list. Summary:

- **Bridge mutations** (`MovePlayer`, `EndTurn`, `WorldMove`, `RestTick`, etc.) bypass the invariant layer and cannot trigger reactions. Decompose into atomic mutations when those systems are next touched for feature work.
- **Duplicate mutation variants** (`SpendAp`/`SetPlayerAp`, `AddHp`/`SetPlayerHp`, `Equip`/`SetEquipment`, etc.) — remove delta variants, keep `Set*` only.
- **`apply_one` inline logic** — `QuestNotify` (~35 LOC), `UsePsychicAbility` (~14 LOC), `DamageWall` (~16 LOC) should be extracted to system functions.
- **`notify.rs` underuse** — 5 of 7 `StateTransition` variants produce no reactions. `PlayerPositionChanged` is detected on every move but nothing listens to it.
- **No unit tests for `systems/`** — all 14 system files have zero unit tests. Only tested via DES scenarios.
- **Parallel trace systems migration risk** — `state.trace` (Effect-based) and `state.mutation_log` (Mutation-based) both exist. Before removing the Effect trace, audit which DES assertions rely on it exclusively.
- **RNG clone-writeback is convention, not structure** — consider a `dispatch_with_rng` helper in `dispatch.rs` that makes the pattern impossible to get wrong.

### Remaining technical debt

| Issue | Severity | Estimate |
|-------|----------|----------|
| 97+ `unwrap()` in non-test code | High | 2–3 days |
| ~15 functions >100 lines in state.rs | Medium | Ongoing |
| NarrativeEngine QuestLog is a stub duplicate of real quest_log | Medium | 1 day |
| Stale docs and lore files | Low | 1 day |

---

## Current State

A 30-minute play session is possible: character creation, movement, combat, quests, inventory, crafting, trading, storms, and auto-explore all work. The first quest ("The Pilgrim's Last Angle") is completable. Quest progression beyond that is blocked by missing infrastructure.

**Test status**: 142 DES scenarios pass, 2 ignored (probabilistic/meta). All immediate bugs resolved.

**Gameplay gap (identified 2026-04-12)**: Adaptations are the game's signature system but are currently inert. Most adaptation effects exist only in data — only `saltblood` (glass immunity) and basic `armor`/`damage_bonus` stat modifiers are read in code. Adaptation gain is also passive and unearned (automatic on refraction threshold). Design direction needed before implementation — see §9 below.

---

## Recently Completed

### DES Infrastructure & Bug Fixes (2026-04-06 to 2026-04-26)
- Fixed all 3 immediate gameplay bugs (teleport quest completion, NPC talked flag, dialogue condition logging)
- Fixed DES `poi_type` support — scenarios can now generate dungeon/shrine/landmark maps via full tile pipeline
- Fixed GSB connectivity weight calculation — dungeon connectivity scenarios now pass
- Fixed crafting bug — `rule_craft` index collision when consuming multiple items of same type
- Fixed DES inventory setup — scenario inventory now replaces (not appends to) class starting items
- Fixed `GainSaltScrip` effect missing from `effect_to_mutation` — shop trading now deducts correctly
- Fixed `AddSaltScrip` overflow in debug mode — changed to `wrapping_add`
- Recorded microstructures as `MapFeature` entries in `tile_generator.rs`
- Fixed `NpcTalked` DES assertion to use `any()` instead of `find()` — handles duplicate NPC ids
- Fixed `DialogueCondition` unknown field handling — joke conditions no longer silently pass
- 26 previously-ignored DES scenarios now pass; 7 dead scenarios deleted

### Skill System Rework (2026-03-07)
- **7-category skill tree**: `SaltAlchemy`, `Crafting`, `Social`, `Survival`, `Medical`, `MeleeCombat`, `RangedCombat`
- **35 skills in `data/skill_trees.json`** with `tree_parent`, `blocked`, `active` flags and `passive_effects`
- **Canvas-based tree UI** in `skills_menu.rs`: pannable 2D graph, box-drawing nodes, connection lines, detail panel
- **Typed accessors** on `SkillsState`: `melee_accuracy_bonus()`, `ranged_accuracy_bonus()`, etc.
- **DES coverage**: `skill_tree_upgrade_test.json`
- **Design doc**: `docs/development/SKILL_TREE_DESIGN.md` — ~90 planned skills

### Faction Enemy Aggression (2026-03-07)
- `is_hostile()` checks player rep ≥ 25 with enemy faction before attacking

### Algorithm Layering & Composition (2026-03-07)
- Three blend modes: `replace`, `overlay`, `mask` in `terrain_forge_adapter.rs`
- `terrain_config.json` extended with `algorithm_layers` for desert, saltflat, ruins

### Settlement Generation — Complete (2026-02-22 to 2026-03-01)
- Unified StructureLibrary, grid-with-jitter layout, faction-weighted buildings, NPC spawning
- 35 building prefabs, world map integration, mapgen-tool support

### Faction System — Complete (2026-02-21)
- 7 factions, reputation system, world map territories, faction menu UI, save migration

---

## Feature Roadmap

Features are grouped into tiers. Each tier builds on the previous one. Within a tier, items can be worked in parallel.

### Tier 1 — Core Systems Rework ✅ COMPLETE

#### 1. Save/Load Game Library with Versioning ✅
#### 2. Proper Overworld Travel ✅ (resource consumption and camp/rest deferred)
#### 3. Actual Skill Catalog ✅ (35/~90 skills implemented; ~55 remaining)
#### 4. Proper Faction System ✅

### Tier 2 — Content & Generation

#### 5. Procedural Village/Town/City Generation ✅ **COMPLETE** (2026-03-01)
#### 5.5. Biome-Driven Tile Generation Profiles ✅
#### 5.6. Algorithm Layering & Composition ✅ **COMPLETED** (2026-03-07)

#### 6. Mob and Item Spawn Table Update
- Biome-specific enemy rosters with level scaling
- Elite and rare enemy variants with unique loot
- Environmental spawns (glass crawlers near glass, salt wraiths in flats)
- Dynamic spawn density based on player level and area danger
- Item rarity tiers affecting spawn weights
- **Depends on**: Tiered mobs (#7), tiered loot (#8) — co-developed

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
- **Depends on**: Tiered mobs (#7)

#### 9. Adaptations Rework ✏️ DESIGN COMPLETE — READY TO IMPLEMENT
- **Design doc**: `docs/features/ADAPTATION_SYSTEM.md`
- **Current state**: 10 placeholder adaptations in data; only `saltblood` and basic stat modifiers read in code. Gain is passive and automatic — to be replaced entirely.
- **Mechanic**: Choice at refraction threshold (tier 1: 150, tier 2: 400, tier 3: 800). 3 options drawn from pool weighted by activity counters. Locked adaptations require specific unlock conditions.
- **4 categories**: Survival, Predator, Precision, Artificer — each crystallizes a distinct playstyle.
- **Expected per run**: 3 adaptations (semi-long), 4–5 (late-game).
- **Implementation phases**:
  1. Activity counters + weighted selection logic + choice UI + threshold redesign
  2. New `adaptations.json` (24 entries) + wire each effect in code + faction reputation modifiers
  3. NPC dialogue references + visual indicators + DES scenarios
- **Save version bump required** (existing saves incompatible)
- **Depends on**: Faction system ✅, skill catalog ✅

#### 10. Storm System Rework
- Predictable storm cycles with forecast system
- Storm types: Glass Storm, Salt Squall, Light Flare, Void Rift
- Each type has unique map edits and gameplay effects
- Storm intensity scaling with world progression
- Storm shelters and preparation mechanics
- Storm-created opportunities (new paths, revealed secrets, rare spawns)
- **Depends on**: Tiered mobs (#7) (storm spawns), adaptations (#9) (storm resistance)

### Tier 3 — Combat & Economy

#### 11. Ranged/Throw Weapon Overhaul
- Proper projectile system with trajectory and obstacles
- Ammunition system with crafting
- Line-of-sight integration with FOV
- Special ammo types (glass-tipped, salt-coated, light-infused)
- **Depends on**: Tiered loot (#8), skill catalog ✅

#### 12. Trader Overhaul
- Dynamic pricing based on supply, demand, and faction reputation
- Trader inventories that refresh and respond to world state
- Specialized traders: weaponsmith, alchemist, scribe, glass-worker
- Haggling mechanic using social skills
- Caravan system: traders travel between settlements
- **Depends on**: Faction system ✅, settlement generation ✅, tiered loot (#8)

#### 12.5. Improved Biome Variance
- Distinct visual identity per biome (unique glyphs, color palettes, tile types)
- Biome-specific hazards, resources, and encounters
- Transition zones between biomes with blended features
- Weather patterns per biome affecting gameplay
- **Depends on**: Storm rework (#10), settlement generation ✅

### Tier 4 — Narrative & Progression

#### 13. 50–60% Complete Main Questline
- Implement Acts I–II fully (quests 1–7 of 13)
- Quest-driven location spawning: required structures appear in world
- ARIA interface system for archive interactions
- Missing NPCs: the_architect, high_prism, custodian_iri_7, sable_of_the_seam
- Key items: broken_saint_key, aria_core_fragment, lens_of_the_first_saint
- Act II boss encounter with phase mechanics
- Multiple dialogue paths based on faction alignment
- **Depends on**: Settlement generation ✅, faction system ✅, tiered mobs (#7)

#### 14. Procedurally Generated Quests
- Quest template system: fetch, kill, escort, explore, defend
- Dynamic objectives based on world state and player location
- Reward scaling based on quest difficulty and player level
- **Depends on**: Main questline (#13), faction system ✅, settlement generation ✅

#### 15. Procedurally Generated Lore
- Lore fragment system: books, inscriptions, NPC dialogue, item descriptions
- Grammar-based text generation for environmental descriptions
- Consistent world history generated per seed
- **Depends on**: Main questline (#13), biome variance (#12.5)

---

## Development Principles

- **Determinism first**: All new systems must use seeded RNG from GameState
- **Data-driven**: Content in JSON, mechanics in Rust — tune without recompilation
- **Test with DES**: Every new feature gets at least one DES scenario
- **Versioned saves**: Every GameState change must include a migration path
- **Zero warnings**: `cargo clippy` clean before every commit
- **Priority order**: fixable bugs → features blocking fixes → new features
