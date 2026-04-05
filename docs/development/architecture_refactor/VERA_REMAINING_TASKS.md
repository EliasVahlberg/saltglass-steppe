# VERA Soft-Migration: Remaining Tasks

> Date: 2026-04-05
> Scope: Complete the current soft-migration. Close verification gaps. Clean up.
> NOT in scope: Domain decomposition, state.rs redesign, architectural changes.
> After this: Design and plan the resolution to the state.rs structural problems.

---

## What "done" means

The soft-migration is complete when:
1. Every gameplay action goes through `dispatch()` (no legacy bypass paths)
2. DES can assert on effects (not just state)
3. Agents have a single workflow document to follow
4. No hardcoded game content in code
5. Data files are internally consistent
6. SYSTEM_STATUS.md reflects reality

---

## Tasks

### 1. DES Effect Assertions

**Why:** The core verification promise. Without this, the trace is just a debug dump.

**Scope:** Add 3 assertion types to `src/des/mod.rs`:
- `effect_occurred { effect_pattern }` — substring match on effect Debug repr
- `effect_not_occurred { effect_pattern }`
- `effect_count { effect_pattern, op, value }`

Add 3 unit tests. Update 2-3 existing DES scenarios to use effect assertions alongside state assertions.

**Effort:** Half day. **Branch:** main.

**Detail:** `PRIORITY_1_DES_EFFECT_ASSERTIONS.md`

---

### 2. Agent Workflow Document

**Why:** Agents currently piece together the VERA workflow from 5+ sources. One checklist.

**Scope:** Create `docs/development/VERA_WORKFLOW.md`. Add as resource to LeadDeveloper, systems-engineer, combat-engineer, implementation-planner agent configs.

**Effort:** 2 hours. **Branch:** main.

**Detail:** `PRIORITY_2_AGENT_WORKFLOW.md`

---

### 3. Hardcoded Constructor Cleanup

**Why:** `GameState::new()` has hardcoded NPCs and items. Content belongs in data files.

**Scope:**
- Remove hardcoded `dying_pilgrim`, `hand_torch`, `glass_pick` from constructor
- Add these to spawn table data so the generation pipeline handles them
- Delete the 6 lore generation stubs that return `None`
- Move `new()` / `new_with_class()` to `state_init.rs` (structural separation only, no redesign)

**Effort:** Half day. **Branch:** main.

---

### 4. Legacy Method Deletion

**Why:** Old methods still exist alongside VERA equivalents. External code can bypass dispatch.

**Scope:** For each legacy method: find callers → update to use dispatch → delete method.

| Delete | VERA equivalent |
|--------|----------------|
| `gain_xp()` | `PlayerEffect::GainXp` apply arm |
| `allocate_stat()` | `Command::AllocateStat` |
| `wait_turn()` | `Command::Wait` |
| `rest()` | `Command::Rest` |
| `apply_status()` | `PlayerEffect::ApplyStatusEffect` |
| `modify_reputation()` | `PlayerEffect::ModifyReputation` |
| `apply_status_effect()` | `PlayerEffect::ApplyStatusEffect` |
| `use_psychic_ability()` | `Command::UsePsychic` |

Convert 4 dispatch passthroughs to proper dispatch helpers with effects:
- `Interact` → currently calls `self.interact_at()` directly
- `Examine` → currently calls `self.examine_at()` directly
- `EnterSubterranean` → currently calls `self.enter_subterranean()` directly
- `ExitSubterranean` → currently calls `self.exit_subterranean()` directly

**Effort:** 1 day. **Branch:** main. **Depends on:** Task 1 (effect assertions for verification).

**Detail:** `PRIORITY_4_LEGACY_CLEANUP.md`

---

### 5. Selective Bridge Conversion

**Why:** Status, loot, and quest bridges call legacy code from apply arms. These systems are likely to be modified during feature work and need rule unit tests.

**Convert to pure rules/reactions:**
- `PlayerEffect::TickStatusEffects` → `rule_tick_status(ctx) → Vec<Effect>`
- `EventEffect::LootDrop` → `reaction_loot_drop(ctx, rng) → Vec<Effect>`
- `EventEffect::QuestNotify` → reaction functions per event type

**Leave as bridges (stable, rarely modified):**
- `PlayerEffect::RunAI`
- `MapEffect::TickStorm`

**Effort:** 1-2 days. **Branch:** branch. **Depends on:** Task 1.

**Detail:** `PRIORITY_3_BRIDGE_CONVERSION.md`

---

### 6. Data Integrity Fixes

**Why:** 18 dangling cross-references from the original audit. Items referenced in spawn/loot tables that don't exist.

**Scope:**
- Fix 16 item IDs in `data/biome_spawn_tables.json`
- Fix 2 item IDs in `data/loot_tables.json`
- Write a validation script to prevent future dangling references

**Effort:** Half day. **Branch:** main.

---

### 7. Update SYSTEM_STATUS.md

**Why:** The registry doesn't reflect the VERA migration.

**Scope:** Update every system entry with:
- VERA status: pure rule / bridge effect / legacy passthrough
- Rule unit test count
- DES scenario references

**Effort:** 1 hour. **Branch:** main. **Do last** — after all other tasks.

---

## Execution Order

```
Can start immediately (independent):
  Task 1: DES effect assertions
  Task 2: Agent workflow document
  Task 3: Hardcoded constructor cleanup
  Task 6: Data integrity fixes

After Task 1:
  Task 4: Legacy method deletion
  Task 5: Selective bridge conversion

After everything:
  Task 7: Update SYSTEM_STATUS.md
```

**Total effort:** ~5-6 days.

---

## After This

Once these tasks are complete, the soft-migration is finished. The codebase will be:
- Consistently using VERA for all gameplay actions
- Verifiable via DES effect assertions + rule unit tests
- Free of legacy bypass paths
- Free of hardcoded content
- Data-consistent

Then we design the resolution to the state.rs structural problems (verified state store architecture) with a clear picture of what we're working with.

Design document: `VERIFIED_STATE_STORE.md`
