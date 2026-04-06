# Priority 2: Agent Workflow Document

> Effort: 2 hours
> Impact: Makes the VERA pattern self-reinforcing for AI agents
> Files: New `docs/development/VERA_WORKFLOW.md`, update agent configs

## Problem

The VERA pattern is documented across FINAL_ARCHITECTURE.md, AGENTS.md commit policy, steering files, and agent prompts. But there's no single "here's how you add a feature" checklist that an agent reads before starting work. An agent has to piece together the workflow from multiple sources.

## What to build

Create `docs/development/VERA_WORKFLOW.md` — a concise, step-by-step workflow document. This is the document an agent reads before starting any gameplay feature work.

### Contents

```markdown
# VERA Development Workflow

## Before you start

1. Read `docs/development/SYSTEM_STATUS.md` — know what's wired and what isn't
2. Identify which systems your feature touches
3. Check if those systems are pure rules or bridge effects

## Adding a new player action

1. Add a `Command` variant to `src/game/effects/mod.rs`
2. Add `Effect` variants to the appropriate domain enum (PlayerEffect, CombatEffect, etc.)
3. Write a rule function in `src/game/rules/` — pure, takes `&QueryContext`, returns `RuleOutput`
4. Implement apply arms in `src/game/effects/apply.rs` — mechanical field assignments only
5. Wire the command in `GameState::dispatch()` in `src/game/state.rs`
6. Write rule unit tests using `TestContext` (minimum: happy path + one validation failure)
7. Write or update a DES scenario with state assertions AND effect assertions
8. Run `cargo test`, `cargo clippy -- -D warnings`
9. Update `docs/development/SYSTEM_STATUS.md`

## Modifying an existing system

- If the system has a rule file in `src/game/rules/` → modify the rule, update tests
- If the system is a bridge effect (AI, storm, status ticks) → follow the existing pattern, don't convert to pure rules unless explicitly asked
- If adding a new Effect variant → the compiler will force you to add an apply arm

## What NOT to do

- Don't mutate GameState directly outside dispatch/apply
- Don't put logic in apply arms — logic belongs in rules
- Don't write DES scenarios that only assert `player_alive`
- Don't skip rule unit tests
- Don't skip SYSTEM_STATUS.md updates
```

### Agent config updates

Add `file://docs/development/VERA_WORKFLOW.md` to the `resources` array in:
- `LeadDeveloper.json`
- `systems-engineer.json`
- `combat-engineer.json`
- `implementation-planner.json`

## Verify

All agent JSON files valid. The workflow document is consistent with AGENTS.md commit policy and steering files.
