# Documentation Triage Summary — 2026-04-04

Commit: e0d1fe7

## docs/development/

| File | Status | Action | Reason |
|------|--------|--------|--------|
| README.md | stale | Front-matter added | Links to non-existent docs |
| DOCUMENTATION_GUIDELINES.md | current | Front-matter added | Guidelines still match |
| ROADMAP.md | stale | Front-matter added | LOC/scenario counts outdated after cleanup |
| GAME_STATE_GUIDE.md | stale | Front-matter added | LOC counts outdated; 27 dead methods removed |
| DATA_AUTHORING_GUIDE.md | current | Front-matter added | Cross-reference table matches data files |
| MULTI_TERMINAL_SYSTEM.md | stale | Front-matter added | References deleted terminal_spawn.rs |
| SYSTEM_STATUS.md | current | Front-matter added | Just created |
| DES_USAGE.md | current | Front-matter added | Most comprehensive DES doc; kept as canonical |
| DES_README.md | archive | Moved to docs/archive/ | Superseded by DES_USAGE.md |
| DEBUG_EXECUTION_SYSTEM.md | archive | Moved to docs/archive/ | Superseded by DES_USAGE.md |
| PROCEDURAL_GENERATION_COMPREHENSIVE_GUIDE.md | stale | Front-matter added | References non-existent directories |
| CONSTRAINT_SYSTEM_GUIDE.md | stale | Front-matter added | Example constraint types don't match JSON |
| AUTO_EXPLORE_SYSTEM.md | current | Front-matter added | Config fields match implementation |
| GLASS_SEAM_BRIDGING_ALGORITHM.md | current | Front-matter added | Algorithm matches connectivity.rs |
| NEW_SYSTEMS_DOCUMENTATION.md | archive | Moved to docs/archive/ | Describes removed ability systems |
| SETTLEMENT_GENERATION_RESEARCH.md | current | Front-matter added | Accurately documents 7-pass pipeline |
| SETTLEMENT_IMPLEMENTATION_PLAN.md | current | Front-matter added | Phase 1 complete, forward-looking reference |
| SETTLEMENT_GENERATION_PLAN.md | archive | Moved to docs/archive/ | Pre-implementation plan; references deleted prefabs |
| SETTLEMENT_FUTURE_WORK.md | current | Front-matter added | Future work items, explicitly deferred |
| UNIFIED_STRUCTURE_SYSTEM.md | archive | Moved to docs/archive/ | Completed 2026-03-01 |
| TILE_GENERATOR_REFACTOR_PLAN.md | archive | Moved to docs/archive/ | Completed; tile_generator.rs implemented |
| TERRAIN_FORGE_IMPROVEMENT_SUGGESTIONS.md | current | Front-matter added | Upstream suggestions still valid |
| SKILL_SYSTEM_ARCHITECTURE.md | current | Front-matter added | Matches actual skills.rs |
| SKILL_TREE_DESIGN.md | current | Front-matter added | Design reference, 7 categories match |
| SKILL_SYSTEM_IMPLEMENTATION_PLAN.md | archive | Moved to docs/archive/ | All 4 phases completed |
| SKILL_MENU_REWORK_PLAN.md | archive | Moved to docs/archive/ | Completed 2026-03-07 |
| SKILL_CATALOG_IMPLEMENTATION_PLAN.md | archive | Moved to docs/archive/ | Superseded by 7-category rework |
| JSON_SCHEMA_TODO.md | archive | Moved to docs/archive/ | Completed; all schemas generated |
| SCHEMA_REVIEW.md | archive | Moved to docs/archive/ | Consolidation executed 2026-02-22 |
| MAP_ELEMENTS_UNIFICATION_PLAN.md | archive | Moved to docs/archive/ | Completed; map_elements.json is single source |
| KEYBOARD_CONFIG_MIGRATION.md | stale | Front-matter added | Partial migration; 135 hardcoded matches remain |
| PREFAB_SYSTEM_DESIGN.md | archive | Moved to docs/archive/ | Replaced by StructureLibrary |
| DATA_FILE_AUDIT.md | current | Front-matter added | Audit findings match codebase |
| LORE_GRAPH_PROPOSAL.md | current | Front-matter added | Proposal for future work |

## docs/design/

| File | Status | Action | Reason |
|------|--------|--------|--------|
| README.md | current | — | Index page |
| CORE_MECHANICS.md | current | — | Creative vision |
| CREATIVE_DIRECTION_SUMMARY.md | current | — | Core creative pillars |
| GAME_CONCEPT.md | current | — | High-concept vision |
| IDEAS_AND_QUIRKY_FEATURES.md | current | — | Brainstorming |
| Initial_idea.md | current | — | Historical creative doc |
| inspirations.md | current | — | Reference material |
| MECHANICS_PRIORITY.md | stale | Front-matter added | Completion estimates outdated |
| SKILL_CATALOG_SUGGESTIONS.md | current | — | Aspirational catalog |
| SKILL_TREE_DESIGN.md | current | — | Design spec with status annotations |
| VISUAL_POLISH.md | current | — | Visual effects spec |
| core_design/00–08 | current | — | Creative vision and design specs |
| core_design/09_Audio_Direction.md | archive | Moved to docs/archive/ | TUI game with no audio |
| core_design/10_VerticalSlice_Plan.md | stale | Front-matter added | Milestone targets likely outdated |
| core_design/11–15 | current | — | Creative vision and world-building |
| core_design/16_Implementation_Specs.md | stale | Front-matter added | RefractionState struct doesn't exist |
| core_design/17_Material_Physics_and_Crafting.md | current | — | Design goals, partially realized |
| core_design/18_Implementation_Specs_Crafting_Spectrum.md | archive | Moved to docs/archive/ | MaterialType/Base+Lens never implemented |
| core_design/19_Quest_Structure_and_Endings.md | current | — | Creative vision |
| core_design/20_Implementation_Specs_Quests_Dialogue.md | stale | Front-matter added | QuestStatus/QuestStage don't exist |
| core_design/21_Implementation_Specs_Narrative_Engine.md | stale | Front-matter added | StoryState/JournalEntry don't exist |
| Main_Questline/* (4 files) | current | — | Creative vision |
| side_quests/* (3 files) | current | — | Creative vision |

## Totals

| Status | Development | Design | Total |
|--------|------------|--------|-------|
| Current | 14 | 34 | 48 |
| Stale | 7 | 5 | 12 |
| Archive | 13 | 2 | 15 |
| **Total** | **34** | **41** | **75** |
