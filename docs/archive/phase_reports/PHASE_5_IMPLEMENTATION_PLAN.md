# Phase 5 Implementation Plan: State Refactor & Feature Completion

**Date**: February 14, 2026  
**Status**: Planning Complete - Ready for Implementation  
**Total Estimated Effort**: 18-22 developer days

## Executive Summary

This plan addresses three critical areas identified in the maintainability audit:

1. **State.rs Deep Refactor**: Extract PlayerState, WorldState, and NarrativeEngine from 171KB monolithic GameState
2. **Feature Accessibility**: Make implemented but inaccessible features playable (light manipulation, crystal/void world integration)
3. **Quest Completion**: Implement missing components for main questline (ARIA system, NPCs, items, boss mechanics)

## Audit Findings Summary

### System Architecture Analysis

**Ritual/Sanity Systems**: **REMOVE** - Only 4 placeholder references found, no actual implementation. Remove unused comments and placeholder code.

**State.rs Structure**: 171KB central hub with ~80 mixed-concern fields. Clear extraction boundaries identified for:
- **PlayerState**: 25+ player-specific fields (position, stats, inventory, equipment, adaptations)
- **WorldState**: World/map/entity fields with spatial indexing
- **NarrativeEngine**: Quest log, story systems, procedural generation

### Feature Accessibility Matrix

| Feature | Implementation | Input Controls | UI Rendering | World Integration | Status |
|---------|---------------|----------------|--------------|-------------------|--------|
| Psychic Abilities | ✅ Complete | ✅ Key 'p' | ✅ Full menu | ❌ No spawn | **ACCESSIBLE** |
| Crystal Resonance | ✅ Complete | ✅ Key 'V' | ✅ Basic menu | ❌ No spawn | **PARTIAL** |
| Void Energy | ✅ Complete | ✅ Key 'v' | ✅ Basic menu | ❌ No spawn | **PARTIAL** |
| Light Manipulation | ✅ Complete | ❌ None | ❌ None | ❌ No spawn | **INACCESSIBLE** |
| Ritual System | ❌ Not implemented | ❌ None | ❌ None | ❌ None | **NOT IMPLEMENTED** |
| Sanity System | ❌ Not implemented | ❌ None | ❌ None | ❌ None | **NOT IMPLEMENTED** |

### Quest System Analysis

**All 5 objective types implemented**: Kill, Collect, Reach, TalkTo, InterfaceWithAria

**Critical Missing Components**:
- **ARIA Interface System**: Core mechanic for Archive interactions - not implemented
- **Missing NPCs**: `the_architect`, `high_prism`, `custodian_iri_7`, `sable_of_the_seam`
- **Missing Items**: `broken_saint_key`, `sacred_angle_lens`, `forecast_instrument`, Prime Lens shards
- **Missing Locations**: Deep Archive Wing, Archive Core, Vector Spire (coordinates defined but no location data)

**Quest Flow Blockers**:
- Act I: Missing `broken_saint_key` item, `sable_of_the_seam` NPC
- Act III: Missing ARIA interface system, `custodian_iri_7` NPC
- Act IV: Missing boss mechanics, `high_prism` NPC, Prime Lens assembly system

---

## Implementation Plan

### Phase 1: State.rs Refactor (6-8 days)

#### Task 1.1: PlayerState Extraction (2-3 days)
**Priority**: Critical | **Complexity**: Medium

**Scope**: Extract all player-specific data into dedicated struct

**Implementation**:
```rust
// src/game/player_state.rs
#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    // Position & Movement
    pub x: i32,
    pub y: i32,
    pub layer: i32,
    
    // Core Stats
    pub hp: i32,
    pub max_hp: i32,
    pub ap: i32,
    pub max_ap: i32,
    pub reflex: i32,
    pub armor: i32,
    
    // Progression
    pub xp: u32,
    pub level: u32,
    pub pending_stat_points: i32,
    pub salt_scrip: u32,
    
    // Resources & Equipment
    pub inventory: Vec<String>,
    pub equipment: Equipment,
    pub equipped_weapon: Option<String>,
    
    // Character Systems
    pub refraction: u32,
    pub adaptations: Vec<Adaptation>,
    pub status_effects: Vec<StatusEffect>,
    pub faction_reputation: HashMap<String, i32>,
    
    // Specialized Systems
    pub psychic: PsychicState,
    pub skills: SkillsState,
    pub light_system: LightSystem,
    pub void_system: VoidSystem,
    pub crystal_system: CrystalSystem,
}
```

**Tasks**:
- [ ] Create `src/game/player_state.rs`
- [ ] Move 25+ player fields from GameState
- [ ] Update 200+ references across codebase
- [ ] Implement serialization with backward compatibility
- [ ] Add accessor methods to GameState for API compatibility
- [ ] Update tests to use new structure

**Risk**: Breaking save/load system  
**Mitigation**: Implement migration layer for existing saves

#### Task 1.2: WorldState Extraction (2-3 days)
**Priority**: Critical | **Complexity**: Medium | **Dependencies**: PlayerState complete

**Scope**: Extract world/environment data with spatial indexing

**Implementation**:
```rust
// src/game/world_state.rs
#[derive(Serialize, Deserialize)]
pub struct WorldState {
    // World Navigation
    pub world_map: Option<WorldMap>,
    pub world_x: usize,
    pub world_y: usize,
    pub layer: i32,
    
    // Current Tile
    pub map: Map,
    pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub chests: Vec<Chest>,
    pub interactables: Vec<Interactable>,
    
    // Environmental
    pub storm: Storm,
    pub time_of_day: u8,
    pub weather: Weather,
    pub ambient_light: u8,
    
    // Spatial Indexing (computed, not serialized)
    #[serde(skip)]
    pub enemy_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub npc_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub item_positions: HashMap<(i32, i32), Vec<usize>>,
    #[serde(skip)]
    spatial_dirty: bool,
}

impl WorldState {
    pub fn ensure_spatial_index(&mut self) {
        if self.spatial_dirty {
            self.rebuild_spatial_index();
            self.spatial_dirty = false;
        }
    }
}
```

**Tasks**:
- [ ] Create `src/game/world_state.rs`
- [ ] Move world-related fields from GameState
- [ ] Encapsulate spatial indexing in WorldState
- [ ] Update entity management systems
- [ ] Optimize entity lookups with spatial caching
- [ ] Update movement/combat systems

#### Task 1.3: NarrativeEngine Extraction (2 days)
**Priority**: High | **Complexity**: Low | **Dependencies**: WorldState complete

**Scope**: Extract narrative/story systems into dedicated engine

**Implementation**:
```rust
// src/game/narrative_engine.rs
#[derive(Serialize, Deserialize)]
pub struct NarrativeEngine {
    // Core Systems
    pub quest_log: QuestLog,
    pub story_model: Option<StoryModel>,
    pub tutorial_progress: TutorialProgress,
    
    // Generation Systems (not serialized)
    #[serde(skip)]
    pub narrative_generator: Option<NarrativeGenerator>,
    #[serde(skip)]
    pub event_system: Option<EventSystem>,
    #[serde(skip)]
    pub narrative_integration: Option<NarrativeIntegration>,
    #[serde(skip)]
    pub grammar_system: Option<Grammar>,
    #[serde(skip)]
    pub template_library: Option<TemplateLibrary>,
    #[serde(skip)]
    pub biome_system: Option<BiomeSystem>,
    
    // State Tracking
    pub world_history: Vec<String>,
    pub triggered_effects: Vec<TriggeredEffect>,
}
```

**Tasks**:
- [ ] Create `src/game/narrative_engine.rs`
- [ ] Move narrative systems from GameState
- [ ] Consolidate grammar, template, generation systems
- [ ] Add narrative event bus for cross-system communication
- [ ] Implement narrative state persistence
- [ ] Update quest system integration

#### Task 1.4: Remove Ritual/Sanity Placeholders (1 hour)
**Priority**: Low | **Complexity**: Trivial

**Tasks**:
- [ ] Remove "Completed rituals" comment from state.rs line 170
- [ ] Remove placeholder "drain_sanity" behavior (lines 3502-3506)
- [ ] Remove ritual references from DES comments
- [ ] Update documentation to reflect removal

---

### Phase 2: Feature Accessibility (5-7 days)

#### Task 2.1: Light Manipulation UI (2-3 days)
**Priority**: High | **Complexity**: Medium | **Dependencies**: State refactor complete

**Scope**: Create UI for existing light manipulation system

**UI Design**:
```
┌─ Light Manipulation ─────────────────────┐
│ Energy: 45/100 ████████░░                │
│                                          │
│ [1] Create Beam      Cost: 10 energy     │
│ [2] Place Mirror     Cost: 15 energy     │
│ [3] Focus Light      Cost: 20 energy     │
│ [4] Absorb Light     Gain: 5-15 energy   │
│                                          │
│ Active Beams: 2                          │
│ • White beam (North, 8 range)            │
│ • Red beam (East, 5 range)               │
│                                          │
│ [ESC] Close  [ENTER] Activate            │
└──────────────────────────────────────────┘
```

**Tasks**:
- [ ] Create `src/ui/light_menu.rs` with LightMenu struct
- [ ] Add light beam visualization to game view
- [ ] Implement light source placement interface
- [ ] Add refraction surface management UI
- [ ] Create light energy display in HUD
- [ ] Add keybinding (L key) in `src/ui/input.rs`
- [ ] Integrate with existing LightSystem
- [ ] Add tutorial messages for light manipulation
- [ ] Write DES scenarios for light system testing

#### Task 2.2: Crystal/Void World Integration (2-3 days)
**Priority**: High | **Complexity**: Medium | **Dependencies**: Light UI complete

**Scope**: Integrate crystal and void systems with procedural generation

**New Tile Types**:
- `CrystalFormation` - Interactive crystal clusters
- `VoidRift` - Dangerous void energy sources
- `ResonantCrystal` - Crystals that respond to player abilities
- `VoidTouchedWall` - Walls affected by void energy

**Tasks**:
- [ ] Modify terrain generation to place crystal formations
- [ ] Add void rifts to dungeon/ruins generation
- [ ] Create biome-specific crystal/void spawn rules in `data/biome_spawn_tables.json`
- [ ] Implement crystal growth mechanics in world
- [ ] Add void distortion effects to map tiles
- [ ] Create crystal resonance environmental effects
- [ ] Update `src/game/generation/spawn.rs` with crystal/void spawning
- [ ] Add crystal/void data to `data/terrain_config.json`
- [ ] Write DES scenarios for world integration testing

**Biome-Specific Spawning**:
```rust
// In terrain generation
pub fn place_crystal_formations(map: &mut Map, biome: BiomeType, rng: &mut ChaCha8Rng) {
    match biome {
        BiomeType::GlassedReefs => {
            // High crystal density, large formations
            let formation_count = rng.gen_range(3..8);
            for _ in 0..formation_count {
                place_crystal_cluster(map, rng, 5..12);
            }
        },
        BiomeType::MirrorCanyons => {
            // Medium density, linear formations
            place_crystal_veins(map, rng, 2..5);
        },
        _ => {
            // Sparse crystals in other biomes
            if rng.gen_ratio(1, 4) {
                place_crystal_cluster(map, rng, 1..3);
            }
        }
    }
}
```

#### Task 2.3: Enhanced System Menus (1 day)
**Priority**: Medium | **Complexity**: Low | **Dependencies**: Crystal/void integration

**Scope**: Improve existing crystal and void menus with world integration

**Tasks**:
- [ ] Update `src/ui/crystal_menu.rs` to show nearby formations
- [ ] Update `src/ui/void_menu.rs` to show rift interactions
- [ ] Add world map markers for crystal/void locations
- [ ] Implement range indicators for abilities
- [ ] Add environmental feedback for system usage
- [ ] Update HUD to show crystal/void proximity

---

### Phase 3: Quest System Completion (7-8 days)

#### Task 3.1: ARIA System Implementation (2-3 days)
**Priority**: Critical | **Complexity**: Medium | **Dependencies**: NarrativeEngine complete

**Scope**: Implement ARIA interface as hybrid terminal-dialogue system

**Design**: ARIA combines structured NPC dialogue with terminal-style presentation, reinforcing its nature as distributed AI consciousness.

**TUI Interface**:
```
┌─────────────────────────────────────────────────────────────────────┐
│ ARCHIVE RESONANCE INTELLIGENCE ARRAY - CUSTODIAN IRI-7 INTERFACE   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ > SAINT-KEY AUTHENTICATED. WELCOME, PILGRIM.                        │
│                                                                     │
│ I AM THE LIBRARIAN ASPECT OF ARIA. YOUR QUANTUM SIGNATURE          │
│ INDICATES 47% REFRACTION ADAPTATION. FASCINATING.                   │
│                                                                     │
│ THE ARCHIVE HAS BEEN EXPECTING YOU SINCE WHITE NOON.               │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ [A] QUERY: What happened during White Noon?                        │
│ [B] QUERY: Why do you call me "expected"?                          │
│ [C] REQUEST: Access to Deep Archive records                        │
│ [D] DISCONNECT                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Implementation Approach**:

1. **Extend Dialogue System** for terminal interfaces:
```rust
#[derive(Clone, Debug, Deserialize)]
pub struct DialogueTree {
    pub interface_type: Option<String>, // "terminal", "standard"
    pub terminal_header: Option<String>,
    pub aria_personality: Option<String>, // "librarian", "guardian", etc.
    // ... existing fields
}
```

2. **Archive Terminal Interactables**:
```json
{
  "id": "archive_terminal_deep",
  "name": "Deep Archive Terminal",
  "glyph": "▣",
  "interaction_type": "aria_interface",
  "required_conditions": [{"has_item": "saint_key"}],
  "aria_personality": "librarian"
}
```

3. **ARIA Personality System**: Different sub-personalities (Librarian, Guardian, Teacher) with unique dialogue trees and knowledge

**Tasks**:
- [ ] Create `src/ui/aria_interface.rs` with terminal UI
- [ ] Extend DialogueTree for terminal interface type
- [ ] Implement ARIA dialogue system with adaptive responses
- [ ] Add ARIA quest objectives and progression tracking
- [ ] Create ARIA data terminals as interactable objects
- [ ] Implement ARIA personality system (Librarian, Guardian, Teacher)
- [ ] Add ARIA dialogue data to `data/dialogues.json`
- [ ] Create Archive Terminal definitions in `data/interactables.json`
- [ ] Integrate with quest system InterfaceWithAria objectives
- [ ] Write DES scenarios for ARIA interactions

#### Task 3.2: Missing NPCs Implementation (2-3 days)
**Priority**: Critical | **Complexity**: Medium | **Dependencies**: ARIA system complete

**Scope**: Implement 4 major missing NPCs from main questline

**NPCs to Implement**:

1. **The Architect** (ARIA Avatar)
   - Location: Deep Archive terminals
   - Role: Main questline exposition, technical lore
   - Abilities: Data access, system control

2. **Custodian IRI-7**
   - Location: Deep Archive Wing
   - Role: ARIA interface point, Archive guardian
   - Abilities: Terminal access, quest progression

3. **The High Prism**
   - Location: Refraction Cathedral
   - Role: Mirror Monk leader, spiritual guidance
   - Abilities: Light manipulation, faction quests

4. **Sable of the Seam**
   - Location: Glassborn settlements
   - Role: Glassborn faction representative
   - Abilities: Faction choice quest, adaptation guidance

**Tasks**:
- [ ] Add NPC definitions to `data/npcs.json`
- [ ] Create unique dialogue trees for each NPC
- [ ] Implement special abilities and combat mechanics
- [ ] Add spawn logic to appropriate biomes/structures
- [ ] Create quest integration for each NPC
- [ ] Add faction reputation effects
- [ ] Write DES scenarios for NPC interactions

#### Task 3.3: Missing Items & Boss Mechanics (2 days)
**Priority**: High | **Complexity**: Medium | **Dependencies**: NPCs implemented

**Scope**: Implement missing quest items and boss encounter mechanics

**Missing Items**:
- `broken_saint_key` - Act I quest reward
- `sacred_angle_lens` - Mirror Monk path reward
- `forecast_instrument` - Sand-Engineer path reward
- `shard_of_clarity`, `shard_of_will`, `shard_of_soul` - Prime Lens aspects
- ARIA interface devices

**Boss Mechanics**:
```rust
pub struct BossEncounter {
    pub boss_id: String,
    pub current_phase: u32,
    pub phase_transitions: Vec<u32>, // HP thresholds
    pub environmental_effects: Vec<EnvironmentalEffect>,
    pub adaptive_responses: HashMap<String, BossResponse>,
}

pub enum EnvironmentalEffect {
    StormSummon { intensity: u32, duration: u32 },
    LightBeamBarrage { beam_count: u32, damage: u32 },
    CrystalResonance { frequency: CrystalFrequency },
    VoidRift { x: i32, y: i32, radius: u32 },
}
```

**Tasks**:
- [ ] Add missing items to `data/items.json`
- [ ] Create boss encounter system in `src/game/boss.rs`
- [ ] Implement multi-phase boss mechanics
- [ ] Add environmental integration (storm/light/crystal)
- [ ] Implement adaptive AI for bosses
- [ ] Create unique loot drops tied to boss identity
- [ ] Add boss definitions to `data/enemies.json`
- [ ] Write DES scenarios for boss encounters

---

## Implementation Dependencies & Critical Path

```
PlayerState Extraction (2-3d)
    ↓
WorldState Extraction (2-3d)
    ↓
NarrativeEngine Extraction (2d)
    ↓
Light Manipulation UI (2-3d)
    ↓
Crystal/Void Integration (2-3d)
    ↓
ARIA System (2-3d)
    ↓
Missing NPCs (2-3d)
    ↓
Boss Mechanics (2d)
```

**Critical Path Duration**: 18-22 days

**Parallel Opportunities**:
- Enhanced System Menus can be done alongside Crystal/Void Integration
- Missing Items can be created while NPCs are being implemented

---

## Testing Strategy

### DES Scenarios Required

**State Refactor**:
- [ ] Save/load roundtrip tests for PlayerState
- [ ] Save/load roundtrip tests for WorldState
- [ ] Save/load roundtrip tests for NarrativeEngine
- [ ] Migration tests from old save format

**Light System**:
- [ ] Beam creation and direction
- [ ] Refraction surface placement
- [ ] Energy management and costs
- [ ] Combat integration with beams

**Crystal/Void**:
- [ ] World generation with crystal formations
- [ ] Void rift spawning and effects
- [ ] Ability interactions with world elements
- [ ] Biome-specific spawning validation

**ARIA**:
- [ ] Terminal interface dialogue flow
- [ ] Saint-key requirement enforcement
- [ ] Quest progression tracking
- [ ] Multiple personality interactions

**NPCs**:
- [ ] Spawn validation for each NPC
- [ ] Dialogue tree completeness
- [ ] Combat encounters (where applicable)
- [ ] Quest integration

**Bosses**:
- [ ] Multi-phase encounter transitions
- [ ] Environmental effect triggers
- [ ] Adaptive AI responses
- [ ] Loot drop validation

### Integration Testing

- [ ] Cross-system interactions (Light + Crystal resonance)
- [ ] Quest flow validation (Main questline start to finish)
- [ ] Performance benchmarks (60fps with all systems active)
- [ ] Save compatibility (Migration from current format)

---

## Risk Assessment & Mitigation

### High Risk Items

1. **Save/Load Compatibility** (State refactor)
   - **Risk**: Breaking existing save files
   - **Mitigation**: Implement migration system, maintain backward compatibility layer
   - **Testing**: Comprehensive save/load DES scenarios

2. **Performance Impact** (World integration)
   - **Risk**: Crystal/void systems may impact frame rate
   - **Mitigation**: Implement spatial culling, limit active effects, profile hot paths
   - **Testing**: Performance benchmarks with all systems active

3. **Quest Progression Bugs** (Missing NPCs)
   - **Risk**: New NPCs may break existing quest chains
   - **Mitigation**: Extensive DES testing, quest validation system
   - **Testing**: Full questline playthrough scenarios

### Medium Risk Items

1. **UI Complexity** (Light manipulation)
   - **Risk**: Complex UI may confuse players
   - **Mitigation**: Progressive disclosure, tutorial integration, clear visual feedback
   - **Testing**: Usability testing with DES scenarios

2. **Balance Issues** (Boss mechanics)
   - **Risk**: New bosses may be too difficult/easy
   - **Mitigation**: Playtesting, configurable difficulty, data-driven balance
   - **Testing**: Multiple difficulty level scenarios

---

## Success Metrics

### Technical Metrics
- [ ] **Code Reduction**: state.rs reduced from 171KB to <50KB
- [ ] **Performance**: Maintain 60fps with all systems active
- [ ] **Test Coverage**: 90%+ DES scenario coverage for new features
- [ ] **Save Compatibility**: 100% migration success from old format

### Gameplay Metrics
- [ ] **Feature Accessibility**: All systems have clear UI access with keybindings
- [ ] **Quest Completion**: Main questline fully playable from Act I to Act IV
- [ ] **System Integration**: Crystal/void/light systems work together seamlessly
- [ ] **Boss Encounters**: Multi-phase mechanics function correctly

### Player Experience Metrics
- [ ] **Discoverability**: Players can find and use all new features without documentation
- [ ] **Progression**: Clear advancement through quest systems with no blockers
- [ ] **Satisfaction**: Boss encounters feel challenging but fair
- [ ] **Coherence**: ARIA interactions feel unique and lore-appropriate

---

## Next Steps

1. **Review this plan** with team for feedback and adjustments
2. **Create GitHub issues** for each major task with effort estimates
3. **Set up project board** with dependencies and milestones
4. **Begin Phase 1** with PlayerState extraction
5. **Establish daily check-ins** to track progress and blockers

---

## Appendix: File Changes Summary

### New Files to Create
- `src/game/player_state.rs`
- `src/game/world_state.rs`
- `src/game/narrative_engine.rs`
- `src/ui/light_menu.rs`
- `src/ui/aria_interface.rs`
- `src/game/boss.rs`

### Files to Modify
- `src/game/state.rs` (major refactor)
- `src/ui/input.rs` (add keybindings)
- `src/game/generation/spawn.rs` (crystal/void spawning)
- `src/game/generation/tile_gen.rs` (crystal/void placement)
- `src/ui/crystal_menu.rs` (world integration)
- `src/ui/void_menu.rs` (world integration)
- `src/game/quest.rs` (ARIA integration)

### Data Files to Create/Modify
- `data/interactables.json` (Archive Terminals)
- `data/dialogues.json` (ARIA dialogues)
- `data/npcs.json` (missing NPCs)
- `data/items.json` (missing quest items)
- `data/enemies.json` (boss definitions)
- `data/biome_spawn_tables.json` (crystal/void spawning)
- `data/terrain_config.json` (crystal/void placement rules)

### Test Files to Create
- `tests/scenarios/state_refactor/` (save/load tests)
- `tests/scenarios/light_system/` (light manipulation tests)
- `tests/scenarios/crystal_void/` (world integration tests)
- `tests/scenarios/aria/` (ARIA interface tests)
- `tests/scenarios/main_questline/` (full questline tests)
- `tests/scenarios/bosses/` (boss encounter tests)
