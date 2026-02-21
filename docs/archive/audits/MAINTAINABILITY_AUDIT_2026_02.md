# Maintainability Overhaul: Findings & Action Plan

> Generated: 2026-02-14 | Status: Cleanup + Phase 1-2 decomposition COMPLETE

## What Was Done

### Cleanup Completed
- ✅ Deleted 3 deprecated steering files
- ✅ Fixed all compilation warnings (0 warnings now)
- ✅ Removed stub modules: `ritual.rs`, `sanity.rs` (fields, DES support, HUD display all cleaned)
- ✅ Removed unused generation abstractions: `registry.rs` (439 lines), `pipeline.rs` (156 lines), `layered_generation.rs` (392 lines)
- ✅ Deleted 9 orphaned data files (8 orphaned + `generation_config.json`)
- ✅ Kept `progression.rs` — NOT a stub, actively used by `gain_xp()`

### State.rs Decomposition Completed
- ✅ **Phase 1: Extracted VisualEffects** → `src/game/visual_effects.rs` (229 lines)
  - Moved: DamageNumber, ProjectileTrail, LightBeam, BeamType structs
  - Moved: 6 fields, 12 methods, line_path helper
  - GameState keeps delegation methods for backward compatibility
- ✅ **Phase 2: Extracted DebugSystem** → `src/game/debug_commands.rs` (172 lines)
  - Moved: ~850-line debug_command() method + helper
  - GameState keeps 3-line delegation method
- **Result: state.rs reduced from 4,553 → 3,536 lines (22% reduction)**
- **133 tests pass, 0 failures, 0 warnings**

---

## Architecture Issues (by severity)

### 🔴 CRITICAL: state.rs God Object
- **3,536 lines** (down from 4,553), ~85 methods remaining
- VisualEffects and DebugSystem extracted; still holds player state, world state, narrative generation, spatial indexing
- **Next extractions** (in priority order):
  1. NarrativeEngine (story model, template library, grammar, narrative integration) — ~300 lines
  2. PlayerState (HP, AP, XP, inventory, equipment, adaptations) — high-impact, many call sites
  3. WorldState (map, enemies, NPCs, items, chests) — high-impact, many call sites

### 🟠 HIGH: Event Bus Inadequacy
- **25 event types** (up from 9) — Phase 3 complete
- All major game actions now emit events: combat, movement, interaction, status effects, trading, faction reputation, crystal/void systems, quest completion
- QuestSystem handles all quest-related events via event bus (no more direct `quest_log.on_*()` calls from game logic)
- `process_events()` supports cascading events (QuestCompleted emitted during processing)
- **Remaining**: `TileChanged` variant exists but not emitted (storms change many tiles per edit, needs batching or consumer first). `PlayerDamaged`, `PlayerHealed`, `ItemUsed`, `StoryHook` variants exist but are not emitted yet.
- **Future**: Wire consumers for new events (e.g., crystal system reacting to `TileChanged`, faction UI reacting to `FactionReputationChanged`)

### 🟡 MEDIUM: Generation Over-Engineering
- `AlgorithmRegistry` (439 lines) — only referenced in tests, provides plugin system for 1 algorithm
- `GenerationPipeline` (156 lines) — created in state.rs but never executed
- `LayeredGenerator` (392 lines) — no usage found outside its own module
- **Impact**: ~1,000 lines of abstraction scaffolding that adds complexity without benefit

### 🟡 MEDIUM: FOV Duplication
- `src/game/fov.rs` and `src/game/map.rs::compute_fov()` both implement FOV
- Both use bracket-lib but with different interfaces
- **Impact**: Bug fixes need to be applied in two places

### 🔵 LOW: DES Monolith
- `des/mod.rs` at 2,489 lines handles parsing, execution, assertions, rendering, and test framework
- Works well as-is; splitting is a nice-to-have

---

## Feature Status

### Wired Into Game Loop ✅
| System | Integration Point | UI |
|--------|------------------|-----|
| Combat | CombatSystem in update() | Target HUD, damage numbers |
| AI | AiSystem in end_turn() | Entity rendering |
| Movement | MovementSystem in update() | Input handling |
| Quests | QuestSystem, quest_log.on_turn_passed() | Quest log menu |
| Skills | skills.tick() in end_turn() | Skills menu |
| Status Effects | StatusEffectSystem in end_turn() | HUD display |
| Storm | StormSystem in end_turn() | Storm forecast |
| Psychic | psychic.tick() in end_turn() | Psychic menu |
| Void Energy | void_system.update() in end_turn() | ❌ No dedicated UI |
| Crystal Resonance | crystal_system.update() in end_turn() | ❌ No dedicated UI |
| Light System | light_system.update() in end_turn() | Lighting effects |
| Crafting | craft action in update() | Crafting menu |
| Trading | trade actions in update() | Trade menu |
| Dialogue | dialogue system in movement | Dialogue UI |
| Adaptation | check_adaptation_threshold() | Character display |

### NOT Wired In (Stubs) ❌
| System | What Exists | What's Missing |
|--------|------------|----------------|
| **Ritual** | `ritual.rs` (192 lines), `completed_rituals` field in state | No `tick()`/`update()` call, no triggers, no UI |
| **Sanity** | `sanity.rs` (252 lines), `sanity` field in state | No `tick()`/`update()` call, no triggers, no UI |
| **Progression** | `progression.rs` (30 lines), data loading | Only used in test code, no runtime integration |
| **Faction Data** | `data/factions.json` | File never loaded; faction reputation is a bare HashMap in state |

### Missing Supporting Systems for Existing Features
| Feature | Missing Support |
|---------|----------------|
| Void Energy | No UI to trigger abilities, no discovery mechanic |
| Crystal Resonance | No UI to trigger abilities, no discovery mechanic |
| Faction Reputation | No UI display, no loaded data definitions, limited player feedback |
| Skill Points | No UI for allocating points |
| Tutorial | Data exists (`tutorial.json`), not shown to player |
| Book Reading | Book data exists, reader UI exists, connection unclear |
| Crafting Stations | Referenced in recipes but not placed in world |

---

## Orphaned Data Files

These JSON files exist in `data/` but are **never loaded by any Rust code**:

| File | Size | Likely Purpose |
|------|------|---------------|
| `tiles.json` | 554B | Tile type definitions (tiles are hardcoded in map.rs) |
| `factions.json` | 7.3KB | Faction definitions (reputation is bare HashMap) |
| `expanded_quests.json` | 7.6KB | Extended quest definitions (duplicate of quests.json?) |
| `expanded_spawn_tables.json` | 5.7KB | Extended spawn tables (biome_spawn_tables.json is used instead) |
| `lore_based_quests.json` | 10KB | Lore-driven quests |
| `lore_database.json` | 10KB | Comprehensive lore database |
| `spawn_tables.json` | 1.1KB | Basic spawn tables (biome_spawn_tables.json is used instead) |
| `world_generation_integration.json` | 8.2KB | World gen integration settings |

**Decision needed**: Wire these into code, or delete them to reduce confusion.

---

## Minimum Viable Gameplay Assessment

### Currently Working (30-min session possible) ✅
- Character creation with class selection
- Movement, FOV, map exploration
- Turn-based combat with AI enemies
- Inventory, equipment, item pickup
- Quest acceptance and basic objectives
- World map travel with tile transitions
- Crafting and trading
- Storm events with map transformations
- Auto-explore

### Gaps for "Enjoyable" Experience ❌
1. **No tutorial flow** — new players have no guidance
2. **Faction reputation invisible** — system works but player can't see it
3. **Advanced systems hidden** — void/crystal/psychic work but have no discovery path
4. **Ritual & sanity disconnected** — exist as code but do nothing
5. **No skill point spending UI** — points earned but can't be allocated

---

## Prioritized Action Plan

### Phase 1: Reduce Confusion ✅ DONE
- [x] Removed orphaned data files (9 files deleted)
- [x] Removed stub modules (ritual.rs, sanity.rs — fields, DES, HUD all cleaned)
- [x] Removed unused generation abstractions (registry.rs, pipeline.rs, layered_generation.rs)
- [x] Kept progression.rs (actively used by gain_xp)

### Phase 2: Architecture Quick Wins ✅ DONE
- [x] Extracted `VisualEffects` from state.rs → visual_effects.rs (229 lines)
- [x] Extracted `DebugSystem` from state.rs → debug_commands.rs (172 lines)
- [ ] Consolidate FOV into single implementation (deferred — lower priority)

### Phase 3: Event System Expansion (1 week)
- [x] Add 7 new event types: `PlayerMoved`, `NpcTalkedTo`, `InteractableUsed`, `InteractableExamined`, `AriaInterfaced`, `QuestCompleted`, `TurnEnded`
- [x] Fix double-dispatch bug: `on_item_collected` was called twice per item pickup
- [x] Refactor movement system to emit `PlayerMoved` instead of direct `quest_log.on_position_changed()`
- [x] Refactor NPC interaction to emit `NpcTalkedTo` instead of direct `quest_log.on_npc_talked()`
- [x] Refactor interact/examine/aria to emit events instead of direct quest_log calls
- [x] Route `quest_log.on_turn_passed()` through `TurnEnded` event
- [x] QuestSystem emits `QuestCompleted` events; `handle_event()` logs completion feedback
- [x] `process_events()` loops to handle cascading events (safety limit: 10 iterations)
- [x] Add Batch 3 cross-system events: `StatusEffectApplied/Expired`, `TradeCompleted`, `FactionReputationChanged`, `CrystalResonanceChanged`, `VoidExposureChanged`, `EnemyDamaged`, `DialogueStarted`, `TileChanged` (variant only — no emission yet, storms too noisy without consumers)

### Phase 4: Feature Integration (2 weeks) ✅ COMPLETE
- [x] Wire tutorial system into game flow (overlay + dismissal + game loop check)
- [x] Add faction reputation UI display (FactionMenu + factions.json + 'F' key)
- [x] Add void energy ability UI (VoidMenu + 'v' key + ability activation)
- [x] Add crystal resonance ability UI (CrystalMenu + 'V' key)
- [x] Skill point allocation UI (already existed — SkillsMenu fully functional)
- [x] Place crafting stations in world (crafting_table + glass_forge interactables, station proximity check in craft(), town spawning)
- [ ] Either integrate ritual/sanity into end_turn() or remove them (deferred to Phase 5)

### Phase 5: State.rs Deep Refactor (2-3 weeks) ✅ PLANNED
- [x] **Planning Complete** — See `docs/development/PHASE_5_IMPLEMENTATION_PLAN.md`
- [x] Audit findings: Ritual/sanity systems should be REMOVED (only placeholders)
- [x] Feature accessibility audit: Light manipulation inaccessible, crystal/void lack world integration
- [x] Quest system audit: ARIA system missing, several NPCs/items missing, boss mechanics incomplete
- [ ] Extract `PlayerState` struct (2-3 days)
- [ ] Extract `WorldState` struct (2-3 days)
- [ ] Extract `NarrativeEngine` struct (2 days)
- [ ] Remove ritual/sanity placeholders (1 hour)
- [ ] Implement Light Manipulation UI (2-3 days)
- [ ] Integrate Crystal/Void with world generation (2-3 days)
- [ ] Implement ARIA Interface System (2-3 days)
- [ ] Add missing NPCs (the_architect, high_prism, custodian_iri_7, sable_of_the_seam) (2-3 days)
- [ ] Add missing quest items and boss mechanics (2 days)

**Total Estimated Effort**: 18-22 developer days  
**Status**: Ready for implementation — detailed plan with task breakdown, dependencies, and testing strategy complete

---

## Code Quality Debt

| Issue | Count | Priority |
|-------|-------|----------|
| `unwrap()` in non-test code | 97+ | HIGH (crash risk) |
| Functions >100 lines | ~15 in state.rs | MEDIUM |
| Missing unit tests (non-DES) | Most game systems | MEDIUM |
| Disabled DES scenarios | 12 | LOW |
