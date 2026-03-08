# QA Ways of Working

## Principles

- Every bug needs a reproduction path — seed + steps, a debug state, or a DES scenario
- DES is the source of truth for regression coverage; manual testing finds what DES misses
- A bug without a reproduction artifact is a note, not a ticket

---

## Tools

| Tool | How to open | Purpose |
|---|---|---|
| Debug console | `` ` `` | Run commands, capture state |
| Debug menu | `F12` | Live game state, performance, saved states |
| Issue reporter | `report_issue` in console | Structured bug report with guided form |
| mapgen-tool | `cargo run --bin mapgen-tool` | Reproduce generation offline without full game |
| DES scenarios | `cargo test --test des_scenarios` | Automated regression suite |

---

## When You Find a Bug

### 1. Capture the state immediately
Open the console (`` ` ``) and run:
```
save_debug <short_description>
```
This saves a `.ron` snapshot to `debug_states/` with the full game state including seed.

### 2. File the report
Run `report_issue` in the console. Fill in:
- **Description** — one sentence summary
- **Steps** — exact sequence to reproduce (add each step, blank Enter to advance)
- **Expected** — what should happen
- **Actual** — what happened
- **Severity** — see guide below
- **Category** — closest match

### 3. Note the seed
Run `debug_info` in the console. The output includes `Seed` and `Tile Seed`. Add both to the report steps or description if the bug is generation-related.

### 4. Attach the debug state
Reference the `.ron` filename from step 1 in the report. A dev can load it with `load_debug <name>` to reproduce instantly.

---

## Seed-Specific Bugs

Generation bugs (wrong layout, missing buildings, bad spawns) only occur on specific seeds. Workflow:

1. `save_debug <description>` — captures seed in the file
2. Note the seed from `debug_info` output
3. Include seed in the issue report description: `"Occurs on world seed 84729, tile seed 31045"`
4. If reproducible offline: `cargo run --bin mapgen-tool settlement <seed> <tier>` or `cargo run --bin mapgen-tool tile <seed>` — attach the ASCII output

If the bug is consistent enough to warrant a regression test, a dev will convert it to a DES scenario.

---

## Severity Guide

| Severity | Definition | Examples |
|---|---|---|
| Critical | Crash, data loss, softlock | Game panics, save corrupted, can't progress |
| High | Major feature broken, no workaround | Combat doesn't resolve, quest can't complete |
| Medium | Incorrect behavior, workaround exists | Wrong damage numbers, NPC in wrong building |
| Low | Visual glitch, minor annoyance | Wrong color, typo, misaligned UI |

---

## Regression Testing

Before marking any feature complete or after any fix:

```bash
cargo test --test des_scenarios
```

Expected: 25 pass, 1 fail (`interaction_system_test` — known pre-existing), 1 ignored.

Any new failure is a regression — do not ship.

---

## DES vs Manual Testing

| Use DES for | Use manual testing for |
|---|---|
| Verifying a fix didn't break anything | Finding bugs in the first place |
| Deterministic, repeatable scenarios | Visual/UX issues |
| CI regression coverage | Seed-specific generation bugs |
| Edge cases that are hard to reach manually | Feel and pacing |

When manual testing finds a reproducible bug, the goal is to eventually express it as a DES scenario so it never regresses silently.

---

## Checklist

The in-game QA checklist lives at `docs/testing/QA_CHECKLIST.md`. Work through it top to bottom for a full feature regression pass. Check off items as you go — unchecked items at the end of a session are candidates for bug reports.
