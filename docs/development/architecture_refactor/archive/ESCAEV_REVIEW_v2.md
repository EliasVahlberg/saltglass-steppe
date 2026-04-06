# ESCAEV v2 — Feasibility & Suitability Review

> Reviewer: LeadDeveloper (Kiro CLI)
> Date: 2026-04-04
> Input: ESCAEV v2 proposal, v1 review, migration plan, codebase health audit (Parts 1–6.5 + meta-level taxonomy)
> Status: REVIEW COMPLETE

---

## 1. Does ESCAEV Fit the Codebase?

The meta-level computational taxonomy answers this directly. Every ESCAEV atomic maps to an existing pattern:

| ESCAEV Atomic | Taxonomy Pattern | Existing Modules | Migration Effort |
|---|---|---|---|
| State Facet | `DATA-DEF` | 22 leaf modules (map.rs, entity types, data loaders) | None — already correct |
| Query | `DECISION-FN` reads | combat.rs, encounter.rs, adaptation.rs, travel.rs | None — already pure |
| Command | `INPUT-DISPATCH` output | ui/input.rs Action enum | Rename/tag only |
| Rule | `DECISION-FN` | combat.rs, encounter.rs, progression.rs | Low — already pure functions |
| Reaction | `EVENT-ROUTER` | LootSystem.on_event, QuestSystem.on_event | Low — change return type |
| Derive | Spatial/cache recomputation | update_fov, update_lighting, rebuild_spatial_index | Low — already identified |
| Effect | (new) | Replaces direct mutation in `STATE-ORCHESTRATOR` | **This is the actual work** |
| Trace | (new) | No equivalent | New infrastructure |

The fit is strong because ESCAEV doesn't fight the existing architecture — it formalizes what the well-designed parts already do (pure decision functions, event routing) and replaces what doesn't work (the sole STATE-ORCHESTRATOR in state.rs doing everything through direct mutation).

The taxonomy's key insight — that dead code follows exactly 2 patterns (DATA-XFORM never connected to orchestrator, RESOURCE-ACCUM without INPUT-DISPATCH path) — is precisely what ESCAEV's structural gates prevent. A Rule that returns no Effects produces no trace entries. A DES scenario requiring both `expect_effects` AND `at_end` catches both unwired Rules and no-op apply functions.

---

## 2. Does It Support Planned Features?

| Planned Feature | ESCAEV Impact | Assessment |
|---|---|---|
| Glass Storm editing (THE defining mechanic) | Storm edits become traceable Rules with coarse/fine Effects. Already works — ESCAEV makes it inspectable. | ✅ Good fit |
| Adaptation social consequences | Threshold checks become Reactions to `AddRefraction` Effects. Already works — ESCAEV makes the cascade explicit. | ✅ Good fit |
| Light-based tactical combat | Currently half-wired (abilities unreachable). Under ESCAEV, each ability is a Rule returning Effects. This is the **correct fix** for the scaffold-and-abandon problem — the Rule signature forces you to define what the ability produces. | ✅ Excellent fit — solves the wiring problem |
| Psychic abilities | Currently 3-of-N effects work via hardcoded match. Under ESCAEV, each ability is a data-driven Rule. The "Effect not implemented" catch-all becomes structurally impossible. | ✅ Excellent fit |
| Oku settlement integration | Generation pipeline is DATA-XFORM, outside ESCAEV scope. No conflict. | ⬜ Neutral |
| Multi-z-level settlements | Future. ESCAEV doesn't help or hinder. | ⬜ Neutral |
| Building interiors | Generation pipeline. Outside scope. | ⬜ Neutral |
| Advanced AI (patrol, cover) | AI behaviors are already Strategy pattern. Under ESCAEV, each behavior becomes a Rule in a Priority Chain. Clean extension point. | ✅ Good fit |
| Subterranean layers | Map generation + movement. Movement Rules would need layer-aware queries. QueryContext would need a z-level field. Manageable. | ✅ Workable |

ESCAEV is particularly well-suited to the game's signature mechanics (storms, adaptations, light combat) because these are all cross-concern systems that produce cascading effects — exactly what the Rule → Effect → Reaction chain models.

---

## 3. Migration Feasibility

I agree with the v1 review's confidence levels, with one adjustment:

| Phase | v1 Review Confidence | My Assessment | Notes |
|---|---|---|---|
| 0 (foundation) | High | **High** | New modules, zero behavior change. Can't fail. |
| 0.5 (dead code) | Low risk | **Low risk, but scope is underspecified** (see §5) |
| 1 (use_item) | High | **High** | Self-contained, concrete example in v2. Proof of concept. |
| 2 (movement) | Medium | **Medium-High** | MovementSystem is already well-structured. Good second win. |
| 3 (combat) | Medium-High | **Medium** | process_enemy_death cascade is complex. Expect 2x estimated effort. |
| 3.5 (sub-states) | Medium | **Medium** | Informed by Phases 1-3 usage patterns. Smart ordering. |
| 4 (end_turn) | High (proposal) | **Medium-Low** | 15 live phases after cleanup. This is where it either proves itself or stalls. |
| 5 (DES traces) | Low risk | **Low risk** | Additive. Independent value. |

---

## 4. What the Proposal and v1 Review Both Miss

### 4.1 Phase 0.5 scope is underspecified

The migration plan lists 11 items but doesn't address findings from the audit that should be resolved before encoding anything into the new architecture:

- **Narrative generation code** (1,050 LOC in generation/narrative.rs + narrative_templates.rs) — real Markov chain code that works but is never called from the game pipeline. Keep or delete?
- **18 dangling data cross-references** — 16 spawn table items and 2 loot table items reference IDs that don't exist in items.json. These are runtime bugs.
- **7 fake DES scenarios** — crystal_resonance_basic, void_energy_basic, light_manipulation_basic, enhanced_enemy_systems_test, fov_system_test, narrative_system_test, story_model_test. All assert only `player_alive` with no meaningful actions. Delete or rewrite.
- **Dead UI exports** — `render_map`, `dim_color`, `render_inventory_bar` in ui/mod.rs.
- **ViewportCuller** in renderer/performance.rs — instantiated, result unused.
- **`structure_generation.json`** + deprecated tilegen-tool — delete the tool too?
- **2 dead .des files** — skill_progression_test.des and faction_system_test.des are never executed by the test runner.

These aren't blockers, but Phase 0.5 should be exhaustive. The audit identified them; the migration plan should reference the audit explicitly.

### 4.2 Roadmap discrepancy on state.rs decomposition

The roadmap claims state.rs was "decomposed into PlayerState/WorldState/NarrativeEngine." The audit found 3,525 LOC and 163 methods still on GameState. The decomposition was structural (nested sub-structs as fields) not behavioral (methods still on the god object). This is actually good news for ESCAEV — the sub-structs already exist, so QueryContext can borrow them directly. But the documents should acknowledge this discrepancy to avoid confusion.

### 4.3 The `use_item` example is clean but incomplete

The real `use_item` has conditional branches that depend on item type: book items open the book reader UI, ARIA items trigger dialogue. These are presentation/UI concerns that don't fit neatly into GameEffect. The v2 proposal's PresentationEffect enum handles some of this (Log, HitFlash, DamageNumber) but doesn't cover "open a UI screen."

This needs a design decision: is "open book reader" a PresentationEffect, a DeferredCommand, or something the orchestrator handles after applying effects? Recommendation: the Rule returns a `PresentationEffect::OpenScreen(ScreenId)` variant, and the orchestrator (main.rs) sets the UI state accordingly. This keeps the Rule pure while acknowledging that some items have UI side effects.

### 4.4 DES interpreter modification in Phase 5

DES currently drives GameState through `Action` dispatch. Under ESCAEV, Actions become Commands that produce Effects. The DES interpreter needs to understand this new flow to support `expect_effects` assertions. Phase 5 addresses this, but the DES interpreter (`des/mod.rs`, ~2,400 LOC) is itself the second-largest file in the codebase. The migration plan should note this as a significant piece of work, not just "additive."

### 4.5 Half-wired systems decision has downstream implications

The v2 proposal recommends "remove ability methods, keep resource accumulation." I agree. But the implications should be explicit:

- Light energy, void energy, crystal resonance energy are kept as `ResourceEffect` variants
- If abilities are later wired (light beams, void step, crystal shatter), they become new Rules with ability-specific Effect variants at that point
- The Effect enum's domain-scoped design should leave room for this extension — don't seal the `ResourceEffect` enum

### 4.6 Narrative generation code fate

The audit found 3 disconnected layers: narrative_engine.rs (stub state container), generation/narrative.rs + narrative_templates.rs (real Markov chain code, 1,050 LOC, only used in unit tests), and 10 dead bridge methods in state.rs. The dead bridge methods are covered by Phase 0.5 item 5. But the generation code itself isn't mentioned. If kept, it eventually becomes a Rule that produces text Effects. If deleted, it's ~1,050 LOC less to maintain. Recommend: keep the generation code (it works), delete the bridge stubs, and wire it via ESCAEV when narrative generation is prioritized.

---

## 5. Suitability Verdict

**ESCAEV is well-suited to this codebase and its aspirations.** The architecture:

1. **Directly addresses the root cause** identified in the audit (sole STATE-ORCHESTRATOR with opaque mutations)
2. **Formalizes patterns that already work** (pure decision functions, event routing) — 22 DATA-DEF modules and 5 DECISION-FN modules need zero changes
3. **Provides structural gates** against the scaffold-and-abandon anti-pattern — the exact failure mode that produced ~3,600 LOC of dead code
4. **Supports the game's signature mechanics** (cascading cross-concern effects from storms, adaptations, combat) — these are what Rule → Effect → Reaction chains model
5. **Enables AI-agent self-verification** — Rule unit tests are pure function tests, which AI agents excel at generating
6. **Migrates incrementally** with independently valuable phases — each phase delivers testability improvements even if later phases are deferred

---

## 6. Risk Assessment

### Risks adequately addressed by v2

- Effect enum explosion → domain-scoped enums (§2.5)
- RNG ordering sensitivity → migration sentinel scenarios (§7 Phase 0.5)
- Borrow checker friction → QueryContext (§2.4)
- Save/load compatibility → struct changes deferred to Phase 3.5 (§8)
- Cascade depth → limit 10 (§2.2 rule 4)

### Risks that need monitoring

| Risk | When it manifests | Mitigation |
|---|---|---|
| Phase 4 stalls | After Phase 3, when end_turn's 15 live phases need decomposition | Hard phase boundary. If Phase 4 is too large, split into sub-phases (status tick, storm, AI as separate PRs). |
| Effect enum compile time | After Phase 3, when enum has 40+ variants | Effect enum in separate module (already planned). Monitor incremental build times. |
| DES interpreter complexity | Phase 5, when adding trace assertions to 2,400 LOC interpreter | Budget this as a significant task, not a quick addition. |
| "Open UI screen" effects | Phase 1, when use_item encounters book/ARIA items | Design decision needed before Phase 1 starts. Recommend `PresentationEffect::OpenScreen(ScreenId)`. |

### The biggest risk

**Phase 4 (end_turn decomposition) is where the architecture either proves itself or stalls.** The fan-out is 15 live calls after Phase 0.5 cleanup. Each becomes a phase step with its own Rule. If the pattern from Phases 1-3 holds, this is execution. If it doesn't, this is where you'll discover it.

The proposal's own advice — "phase boundaries are hard stops" — is the most important constraint in the entire document.

---

## 7. Recommendation

**Proceed.** Start Phase 0 + 0.5. If Phase 1 (use_item) succeeds cleanly, the architecture has proven itself. The remaining phases are engineering effort, not design risk.

**Process suggestion:** After Phase 1 completes, do a brief retrospective before starting Phase 2. Document:
- What was harder than expected?
- What did the QueryContext usage patterns look like?
- Did the Effect enum granularity feel right?
- How many Effect variants did use_item actually need?
- Did the migration sentinel scenarios catch anything?

This informs Phases 2-4 and prevents the "push through regardless" failure mode.

---

## 8. System-Agent Notes (Kiro CLI)

> Added: 2026-04-04, after reviewing this document

### Agreed — gaps to incorporate into migration plan

- **Phase 0.5 needs to be exhaustive.** Add: 18 dangling data cross-references (16 spawn table + 2 loot table), 7 fake DES scenarios (delete or rewrite), dead UI exports (render_map, dim_color, render_inventory_bar), ViewportCuller dead code, 2 dead .des files. Reference the audit section-by-section.
- **Book/ARIA design decision must be resolved before Phase 1.** `PresentationEffect::OpenScreen(ScreenId)` is the right approach — keeps the Rule pure. Add this as a Phase 0 design decision item.
- **DES interpreter modification (Phase 5) is significant work**, not "additive." The interpreter is 2,400 LOC. Budget accordingly.
- **Narrative generation code (1,050 LOC):** keep, delete bridge stubs only. Wire via ESCAEV when narrative generation is prioritized.
- **Half-wired system Effect enum extensibility:** don't seal domain enums. Leave room for future ability Rules.

### Minor pushback

- §4.2 (state.rs decomposition discrepancy): accurate observation, but the v2 proposal already accounts for this — QueryContext borrows existing sub-structs (PlayerState, WorldState), Phase 3.5 handles behavioral decomposition. Worth clarifying in docs, not a plan gap.

### Strongest recommendation from this review

The Phase 1 retrospective (§7) is the right process gate. Phase 1 is the experiment. Before Phase 2, document: what was harder than expected, what did QueryContext usage look like, did Effect granularity feel right, how many variants did use_item actually need, did sentinels catch anything. This prevents "push through regardless."
