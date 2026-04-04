# Task: Technical Documentation Triage and Archive

> Priority: Do this BEFORE any architecture refactoring work.
> Reason: Stale docs mislead agents during refactoring. Clean docs first, refactor second.

## Context

`docs/development/` and `docs/design/` contain documents from the earliest development phases. Many describe plans that were executed, partially executed, or abandoned. Agents reading these docs can't distinguish current truth from stale plans, which leads to implementations that anticipate systems that no longer exist or were never built.

The `CODEBASE_HEALTH_AUDIT.md` (in `docs/development/architecture_refactor/`) and `SYSTEM_STATUS.md` (in `docs/development/`) are the current sources of truth for what actually exists in the codebase. Use these as references when triaging.

## Instructions

### Step 1: Triage every file

Go through every file in `docs/development/` and `docs/design/` (not `docs/narrative/` — that's a separate effort). For each file, determine its status:

| Status | Meaning | Action |
|--------|---------|--------|
| `current` | Accurately describes the codebase as it is today | Add front-matter, keep in place |
| `stale` | Describes something that was partially implemented or has drifted from reality | Add front-matter, keep in place with `status: stale` warning |
| `archive` | Describes a plan that was completed, abandoned, or superseded | Move to `docs/archive/` |

To determine status, spot-check claims against the actual code. You don't need to verify every line — check 2-3 key claims per document. If the document references files, modules, or systems that don't exist, it's likely `archive` or `stale`.

### Step 2: Add front-matter to current and stale docs

Add YAML front-matter to the top of every document that stays in `docs/development/` or `docs/design/`:

```markdown
---
status: current
last_verified: 2026-04-04
commit: <current HEAD short hash>
---
```

For stale docs, add a visible warning below the front-matter:

```markdown
---
status: stale
last_verified: 2026-04-04
commit: <current HEAD short hash>
stale_reason: "Settlement generation was implemented but plan details diverge from actual code"
---

> ⚠️ **STALE DOCUMENT** — This document may not accurately reflect the current codebase. 
> Reason: Settlement generation was implemented but plan details diverge from actual code.
> Last verified: 2026-04-04
```

### Step 3: Move archived docs

Move archived files to `docs/archive/`. The archive directory already exists and has subdirectories. Use your judgment on subdirectory organization — by topic or by date, whichever makes more sense given the volume.

For each moved file, leave a one-line tombstone at the original path:

```markdown
Archived to `docs/archive/<filename>`. Reason: <brief reason>.
```

This prevents broken links from silently failing — anyone following an old reference sees where the doc went.

### Step 4: Produce a summary

After completing the triage, produce a summary table in `docs/development/DOC_TRIAGE_SUMMARY.md`:

```markdown
# Documentation Triage Summary — 2026-04-04

| File | Previous Location | Status | Action | Reason |
|------|------------------|--------|--------|--------|
| ROADMAP.md | docs/development/ | archive | Moved to docs/archive/ | Superseded by current development priorities |
| GAME_STATE_GUIDE.md | docs/development/ | current | Front-matter added | Accurately describes state.rs |
| ... | ... | ... | ... | ... |
```

## Files to triage

### docs/development/ (excluding architecture_refactor/ subdirectory)

These are the files to evaluate. The `architecture_refactor/` subdirectory is current — skip it.

- `README.md`
- `DOCUMENTATION_GUIDELINES.md`
- `ROADMAP.md`
- `GAME_STATE_GUIDE.md`
- `DATA_AUTHORING_GUIDE.md`
- `MULTI_TERMINAL_SYSTEM.md`
- `SYSTEM_STATUS.md` (just created — current by definition)
- `DES_USAGE.md`
- `DES_README.md`
- `DEBUG_EXECUTION_SYSTEM.md`
- `PROCEDURAL_GENERATION_COMPREHENSIVE_GUIDE.md`
- `CONSTRAINT_SYSTEM_GUIDE.md`
- `AUTO_EXPLORE_SYSTEM.md`
- `GLASS_SEAM_BRIDGING_ALGORITHM.md`
- `NEW_SYSTEMS_DOCUMENTATION.md`
- `SETTLEMENT_GENERATION_RESEARCH.md`
- `SETTLEMENT_IMPLEMENTATION_PLAN.md`
- `SETTLEMENT_GENERATION_PLAN.md`
- `SETTLEMENT_FUTURE_WORK.md`
- `UNIFIED_STRUCTURE_SYSTEM.md`
- `TILE_GENERATOR_REFACTOR_PLAN.md`
- `TERRAIN_FORGE_IMPROVEMENT_SUGGESTIONS.md`
- `SKILL_SYSTEM_ARCHITECTURE.md`
- `SKILL_TREE_DESIGN.md`
- `SKILL_SYSTEM_IMPLEMENTATION_PLAN.md`
- `SKILL_MENU_REWORK_PLAN.md`
- `SKILL_CATALOG_IMPLEMENTATION_PLAN.md`
- `JSON_SCHEMA_TODO.md`
- `SCHEMA_REVIEW.md`
- `MAP_ELEMENTS_UNIFICATION_PLAN.md`
- `KEYBOARD_CONFIG_MIGRATION.md`
- `PREFAB_SYSTEM_DESIGN.md`
- `DATA_FILE_AUDIT.md`
- `LORE_GRAPH_PROPOSAL.md` (just created — current by definition)

### docs/design/

- All files in `docs/design/` root
- All files in `docs/design/core_design/`
- All files in `docs/design/Main_Questline/`
- All files in `docs/design/side_quests/`

## Guidelines

- **When in doubt, archive.** A missing doc can be retrieved from git. A stale doc that misleads an agent causes real damage.
- **Don't rewrite stale docs.** Just mark them. Rewriting is a separate task.
- **The DES docs (3 files) are likely all describing the same system.** Consider consolidating into one current doc and archiving the others.
- **Implementation plans for completed work should be archived.** The code is the source of truth, not the plan.
- **Design docs that describe the game's creative vision are likely still current** even if old — creative direction doesn't change as fast as technical plans.
