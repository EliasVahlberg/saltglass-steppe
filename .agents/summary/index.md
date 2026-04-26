# Saltglass Steppe — Documentation Index

<!-- Generated: 2026-04-06 | Primary context file for AI assistants -->

## How to Use This Index

This file is the entry point for understanding the saltglass-steppe codebase. It contains enough metadata about each documentation file that you can determine which file to consult for any given question without reading all of them.

**For AI assistants**: Load this file first. Use the summaries below to decide which detailed file to read. Most questions can be answered by consulting 1–2 files from this index.

**Authoritative source for system status**: Always check `docs/development/SYSTEM_STATUS.md` before assuming any system works. It overrides claims in these summary files.

## Quick Reference

| Question | Consult |
|----------|---------|
| How does state mutation work? | `architecture.md` — Verified State Store, two-tier mutations |
| What are the major subsystems? | `components.md` — all 13 systems, generation, UI, renderer |
| How do I add a new Command? | `interfaces.md` — Command interface, system handler signatures |
| What data structures exist? | `data_models.md` — GameState, entities, Effect/Mutation enums |
| How does combat/turn/travel work? | `workflows.md` — sequence diagrams for all major flows |
| What crates are used and why? | `dependencies.md` — dependency graph and integration points |
| Project metadata and build info? | `codebase_info.md` — versions, metrics, entry points |

## Documentation Files

### codebase_info.md
**Purpose**: Project metadata, build configuration, entry points.
**Contains**: Package name/version, Rust edition, dependency table, binary entry points (which are safe for agents to run), release profile settings, CI configuration summary.
**Consult when**: You need basic project facts, want to know how to build/run, or need to check which binaries exist.

### architecture.md
**Purpose**: Core architectural pattern and design decisions.
**Contains**: Verified State Store pattern explanation, two-tier mutation model (atomic vs bridge), state transition detection and reaction wiring, invariant verification rules, RNG clone-writeback pattern, turn processing phase sequence, layer separation diagram.
**Consult when**: You need to understand how state changes flow through the system, why mutations are structured the way they are, or how to add a new reaction to state changes.
**Key insight**: Systems return `Vec<Mutation>`, state applies them with invariant checks, `notify.rs` maps transitions to reactive mutations. Cascade depth-limited to 10.

### components.md
**Purpose**: Major subsystems and their responsibilities.
**Contains**: Component relationship diagram, descriptions of all 13 system handlers (combat, movement, AI, storm, turn, world, quest, explore, interact, loot, status, items, player), 7 rule modules, 8 generation components, UI components (~20 menus), renderer components (tiles, entities, lighting, particles), DES interpreter.
**Consult when**: You need to find which file handles a specific gameplay feature, understand what a subsystem does, or determine where to add new functionality.

### interfaces.md
**Purpose**: APIs, function signatures, and integration contracts.
**Contains**: Complete Command routing table (22 variants → system handlers), system handler signature patterns (command handlers vs notification handlers), QueryContext fields, TestContext builder API, DataLoader generic interface, IPC multi-terminal protocol, DES scenario JSON schema, renderer interface.
**Consult when**: You need to write a new system handler, understand what QueryContext provides, write a DES scenario, or integrate with the data loading system.

### data_models.md
**Purpose**: Data structures, enums, and JSON data organization.
**Contains**: GameState class diagram (PlayerState, WorldState, NarrativeEngine), entity models (Enemy, Npc, Item with their data definitions), map models (Map, Tile, WorldMap), all 7 Effect domain enums, Mutation categories (~70 variants), JSON data file organization diagram, cross-reference map for data modifications, save format.
**Consult when**: You need to understand what fields exist on GameState or its sub-structs, what Mutation variants are available, how data files reference each other, or how saves work.

### workflows.md
**Purpose**: Step-by-step processes and data flows.
**Contains**: Game loop sequence, command dispatch flow, combat flow with reaction cascade, turn processing (9 phases), tile generation pipeline, world travel with encounters, DES test execution, save/load flow, data loading flow, "adding a new gameplay system" checklist.
**Consult when**: You need to trace how a specific action flows through the system, understand the order of operations in turn processing, or follow the tile generation pipeline.

### dependencies.md
**Purpose**: External crate usage and integration architecture.
**Contains**: Dependency graph (Mermaid), detailed tables for all dependency categories (UI, generation, serialization, determinism, architecture, utilities), dev dependencies, key integration points explaining how terrain-forge, bracket-*, serde, and rand_chacha shape the codebase.
**Consult when**: You need to understand why a specific crate is used, how terrain-forge integrates with the generation pipeline, or what the bracket-* crates provide.

## Related Documentation (Outside This Directory)

| File | Purpose |
|------|---------|
| `AGENTS.md` (repo root) | Agent navigation guide — directory map, architecture overview, DES testing, repo patterns, custom instructions |
| `README.md` (repo root) | Project overview, setup, usage, debug commands |
| `docs/development/SYSTEM_STATUS.md` | **Source of truth** for system wiring status — overrides all other claims |
| `docs/development/ROADMAP.md` | Feature roadmap, technical debt backlog, development priorities |
| `docs/development/architecture_refactor/VERIFIED_STATE_STORE.md` | Canonical architecture specification |
| `docs/DOCUMENT_DATABASE.md` | Complete listing of all documentation files |
| `CONTRIBUTING.md` | Development setup, coding standards, contribution workflow |
