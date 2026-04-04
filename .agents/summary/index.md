# Saltglass Steppe — Documentation Index

> **For AI Assistants**: This file is the primary entry point for understanding the codebase. Read this first, then consult specific files as needed.

## How to Use This Documentation

1. **Start here** — this index has enough metadata to determine which file has the answer
2. **Consult specific files** — each file below has a summary of its contents
3. **Cross-reference** — files link to each other where topics overlap
4. **Data files** — game content lives in `data/*.json`, validated by `schemas/*_v1.json`
5. **Existing docs** — extensive docs in `docs/` (architecture, design, development, features, narrative, testing)

## Quick Reference

- **Central game state**: `src/game/state.rs`
- **All data definitions**: `data/*.json`
- **All schemas**: `schemas/*_v1.json`
- **DES scenarios**: `tests/scenarios/`
- **Main entry point**: `src/main.rs`
- **Cannot run default binary** in agent context (locks terminal for TUI) — use DES or `mapgen-tool` instead

## Documentation Files

### [codebase_info.md](codebase_info.md)
**What**: Project metadata, entry points, directory map, tech stack, CI pipeline.
**When to consult**: Understanding project basics, finding where code lives, checking dependencies.

### [architecture.md](architecture.md)
**What**: System overview diagrams, design patterns (central state hub, ECS-style systems, data-driven, deterministic RNG, DES, multi-terminal IPC), key architectural decisions.
**When to consult**: Understanding how systems connect, why architectural choices were made, adding new subsystems.

### [components.md](components.md)
**What**: Detailed descriptions of every major component — game systems, procedural generation, UI, renderer, testing infrastructure. Reflects post-cleanup state (dead ability methods removed from light/crystal/void, custom algorithms deleted).
**When to consult**: Understanding what a specific module does, finding the right file to modify.

### [interfaces.md](interfaces.md)
**What**: API contracts between systems — DataLoader, game systems interface, generation pipeline, DES scenario format, IPC protocol, save/load, rendering pipeline, data file cross-references.
**When to consult**: Integrating systems, understanding data flow, writing DES scenarios, adding new data files.

### [data_models.md](data_models.md)
**What**: All data structures — GameState, entities, world map, quests, combat, progression, generation models. Complete data file reference table.
**When to consult**: Understanding data shapes, adding fields, creating new data files, working with save format.

### [workflows.md](workflows.md)
**What**: Key process flows — game turn loop (15 live phases in end_turn), map generation pipeline, DES testing, save/load, world travel, CI pipeline, content addition workflow.
**When to consult**: Understanding execution order, debugging flow issues, adding new pipeline stages.

### [dependencies.md](dependencies.md)
**What**: All Cargo dependencies with versions and usage, build configuration, external tool requirements.
**When to consult**: Adding dependencies, understanding what libraries are available, build/release configuration.

## Existing Project Documentation

| Directory | Contents |
|-----------|----------|
| `docs/architecture/` | System design, tech stack |
| `docs/design/` | Game concept, core mechanics, creative direction, skill trees, quest design |
| `docs/development/` | DES usage, procedural generation guide, codebase health audit, architecture proposals, roadmap |
| `docs/features/` | Feature specs: enemies, factions, skills, settlements, renderer, effects, connectivity, oku integration |
| `docs/narrative/` | World lore, entity lore |
| `docs/testing/` | QA checklist, debug reference |
| `docs/papers/` | Glass Seam Bridging algorithm paper |

## Topic Quick-Find

| Topic | Primary File | Also See |
|-------|-------------|----------|
| Adding a new enemy | data_models.md | `data/enemies/`, `schemas/enemies_v1.json` |
| Adding a new quest | data_models.md, workflows.md | `data/quests.json`, `docs/development/DES_USAGE.md` |
| Map generation | components.md, workflows.md | `docs/development/PROCEDURAL_GENERATION_COMPREHENSIVE_GUIDE.md` |
| Writing DES tests | interfaces.md, workflows.md | `docs/development/DES_USAGE.md`, `tests/scenarios/` |
| Combat mechanics | components.md | `src/game/combat.rs`, `src/game/systems/combat.rs` |
| UI/menu changes | components.md | `src/ui/`, `src/renderer/` |
| Save system | interfaces.md, data_models.md | `src/game/save.rs` |
| Storm system | components.md | `src/game/storm.rs`, `src/game/systems/storm.rs` |
| Skill system | components.md, data_models.md | `docs/development/SKILL_TREE_DESIGN.md` |
| Architecture proposals | — | `docs/development/architecture_refactor/` |
| Codebase health | — | `docs/development/CODEBASE_HEALTH_AUDIT.md` |
