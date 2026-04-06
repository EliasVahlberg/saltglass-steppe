# Orchestrator Reflection — Two Refactors of saltglass-steppe

> Written: 2026-04-06
> Author: Kiro (system-agent), acting as architecture advisor and review orchestrator
> Scope: VERA soft-migration (Phases 0-4, Batches A-F) + Verified State Store refactor (Stages 1-5 + decomposition)

---

## What happened

Two refactors, back to back, with different goals and different outcomes.

**Refactor 1 (VERA)** was a soft-migration: wrap every GameState method in Command→Rule→Effect→Apply. The goal was traceability — every state mutation should be recorded and testable. It succeeded at that goal. It failed at the deeper goal of making the codebase maintainable, because it left all the logic on GameState. state.rs grew from 3,185 to 3,680 LOC.

**Refactor 2 (Verified State Store)** was a response to the problems Refactor 1 created. The goal was separation of concerns: systems own logic, state owns data, notifications handle cross-system coordination. state.rs went from 3,195 to 940 LOC. The architecture is in place but incompletely realized — bridge mutations mean a significant portion of the game logic still runs through imperative code paths that bypass the verification layer.

The two refactors together took the codebase from "everything is a method on GameState" to "systems produce mutations, state applies them with verification." That's a real architectural improvement. But the path was longer and more expensive than it needed to be.

## Key takeaways

### 1. The first refactor solved the wrong problem

VERA was designed to answer "how do we trace state mutations?" The real problem was "why is all the logic on GameState?" Tracing was a symptom — the root cause was that GameState was simultaneously a data container, a command router, and every domain's implementation.

The soft-migration wrapped each method in Command/Effect/Rule without questioning why the method was on GameState in the first place. The result was the same monolith with more ceremony. Every new feature still required touching state.rs. The Effect enums grew to ~50 variants across 7 domains. The dispatch helpers mixed rule calls, post-processing, and legacy code.

The user identified this correctly when they said: "we conflated game state mutation with the logic involved in calculating and producing the change." I should have pushed harder on this during the VERA design phase instead of accepting the soft-migration approach.

### 2. The user's instinct was right from the start

When the user first described the state.rs problems, they sketched a "categorical resolver" pattern with a flat data interface and verification behavior tied to state variables. That sketch was essentially the Verified State Store — systems produce mutation requests, state verifies and applies them, subscribers react to transitions.

I initially proposed domain modules (Proposal 1) — self-contained files that own dispatch + rules + apply + tests per domain. The LeadDeveloper proposed dispatch extraction (Proposal 2) — move orchestration to free functions, keep apply centralized. Both proposals were incremental improvements to the existing structure. The user pushed back on both, asking about code reuse, extensibility, and the "write your code in 10 different places" friction.

That pushback led to the Verified State Store design, which is fundamentally different from both proposals. The key insight the user contributed — that mutations should be atomic and verification should be per-field — was not something either agent proposed. We were optimizing the arrangement of existing code; the user was questioning the abstraction.

### 3. Agent-driven development requires the blueprint to be right

The user made a critical observation about AI agents: they replicate whatever patterns they see, they don't experience friction, and they optimize for task completion rather than codebase health. This means the existing code is the de facto specification — agents will produce more of whatever is already there.

This changes the calculus on "when to refactor." The traditional answer is "when it causes problems." The correct answer for agent-driven development is "before the agents start replicating the wrong pattern." Every day the 3,680 LOC state.rs existed, it was teaching agents that dispatch helpers on GameState is the right pattern.

### 4. The honest assessment matters more than the optimistic one

During the VERA migration, I reported progress in terms of phases completed, tests passing, and LOC added. All true, all misleading. The codebase was getting larger, not better. state.rs grew. The dispatch helpers were a third category of code that VERA didn't account for. Bridge effects were pragmatic compromises that became permanent.

When the user did a focused review of state.rs and said "this alone should raise red flags," that was the turning point. The honest assessment — that the soft-migration had wrapped the problem rather than solving it — led directly to the second refactor.

I should have flagged the state.rs growth earlier. I knew it was getting larger. I rationalized it as "the migration adds infrastructure that will pay off later." It didn't pay off — it had to be replaced.

## What went well

**The staged approach worked.** Both refactors were done incrementally with tests passing after each stage. No big-bang rewrites. The VERA migration had 12 batches, each independently verifiable. The state store refactor had 5 stages plus a decomposition phase. At no point was the codebase broken for more than a few minutes.

**DES as a regression harness.** The 26 DES scenarios (even with 7 being thin) caught regressions throughout both refactors. Having a headless test framework that drives GameState without rendering was essential — it meant every structural change could be verified in ~1 second.

**The design conversation.** The back-and-forth between the user, me, and the LeadDeveloper produced a better design than any single participant would have reached alone. The user contributed the core insight (atomic mutations with verification). I contributed the notification/transition model. The LeadDeveloper contributed pragmatic implementation decisions (bridge mutations, clone-writeback for rng, mutation_log for DES compatibility).

**The standing architecture document.** Splitting VERIFIED_STATE_STORE.md (permanent reference) from STATE_STORE_REFACTOR.md (working document) was the right call. The standing document will outlive the refactor and serve as the specification for how state management works.

**vera-effects crate.** Publishing the generic types (Trace, TraceEntry, RuleOutput) as a separate crate was the right abstraction. The game crate uses type aliases. The crate is small, stable, and reusable.

## What didn't go well

**Refactor 1 was largely wasted effort.** The VERA soft-migration added ~9,500 lines across 73 files. Most of that infrastructure (Effect enums, apply arms, dispatch helpers, rule functions) was replaced or deleted in Refactor 2. The rule functions survived (they're still in rules/), but the Effect enums, apply.rs, and dispatch helpers are gone. The trace system was adapted but the TraceSource/TraceEntry types changed meaning.

I don't think Refactor 1 was avoidable — the user needed to see the soft-migration's limitations to understand what the real problem was. But if I had pushed harder on the "why is everything on GameState?" question during the VERA design phase, we might have gone directly to something closer to the state store architecture.

**The Mutation enum grew beyond its design.** The architecture document specifies "every variant changes exactly one state field." The actual enum has bridge mutations (WorldMove, MovePlayer, EndTurn), compound mutations (AllocateStat, SetEquipment, DamageWall), duplicate variants (SpendAp vs SetPlayerAp), and wrapper variants (Equip delegates to SetEquipment). This happened because each stage prioritized "make it work" over "make it right." The enum is now a mix of atomic mutations and imperative commands wearing mutation costumes.

**The notification system is mostly decorative.** notify.rs handles 2 of 7 transition types. The other 5 are detected but produce no reactions. The bridge mutations bypass the notification system entirely because they handle their own post-processing internally. The "spreadsheet + bulletin board" model is only actually working for combat.

**Three design documents were written and superseded.** DOMAIN_DECOMPOSITION_PLAN.md → superseded by DISPATCH_EXTRACTION_DESIGN.md → superseded by VERIFIED_STATE_STORE.md. Each represented a different understanding of the problem. The iteration was necessary but it means there are two dead documents in the architecture_refactor directory that could confuse future readers.

## Did we change the architectural pattern?

Yes, fundamentally.

**VERA** (Refactor 1) was: Command → Rule (pure function) → Vec\<Effect\> → apply_effect (mechanical) → trace. The pattern was about traceability — recording what happened so tests could assert on it. The unit of work was the Effect, which was a domain-specific description of a state change (CombatEffect::DealDamage, PlayerEffect::GainXp).

**Verified State Store** (Refactor 2) is: Command → System (free function) → Vec\<Mutation\> → apply_one (verified) → StateTransition → notify (reactive). The pattern is about separation of concerns — systems own logic, state owns data integrity, notifications handle coordination. The unit of work is the Mutation, which is an atomic field-level state change (SetEnemyHp, SetPlayerAp).

The key differences:
- Effects were domain-specific (CombatEffect, PlayerEffect). Mutations are field-specific (SetPlayerHp, SetEnemyHp).
- Effects were applied mechanically (no invariant checks). Mutations are applied with per-field verification (clamp, bounds check).
- Effects had no transition detection. Mutations detect state transitions (hp crossed zero, position changed) and report them.
- Reactions were hardcoded in collect_reactions on GameState. Reactions are external in notify.rs.
- Rules returned domain-specific RuleOutput. Systems return Vec\<Mutation\>.

The VERA name still applies to the trace/verification aspect, and the vera-effects crate is still used for the Trace type. But the architectural pattern changed from "trace everything" to "verify everything and react to transitions."

## Do we currently adhere to the pattern?

Partially. The architecture is in place structurally — dispatch.rs routes commands, systems produce mutations, apply_one verifies and detects transitions, notify.rs handles reactions. But the bridge mutations mean a significant portion of the game logic bypasses the pattern entirely.

Concretely:
- **Combat** fully adheres. handle_melee/handle_ranged return atomic mutations. EnemyHpChanged and EnemyHpReachedZero trigger reactions in notify.rs. Swarm aggro, reflect damage, loot drops, and split-on-death all happen through the cascade.
- **Movement, world travel, turn system, psychic, flee** do not adhere. They use bridge mutations that call imperative code inside apply_one. No transitions are emitted. No reactions fire.
- **Wait, rest, equip, unequip, allocate_stat, use_item** partially adhere. They go through dispatch.rs and produce mutations, but some mutations are compound (AllocateStat) or delegate (Equip → SetEquipment).

The honest answer: about 30% of the game's command surface fully adheres to the pattern. The rest uses the infrastructure (dispatch.rs, apply_mutations) but bypasses the verification and notification layers.

## Biggest weakness in saltglass-steppe right now

**The bridge mutations are a ticking clock.** Every bridge mutation is a place where the old imperative pattern is preserved inside the new architecture. They work, they pass tests, but they're invisible to the notification system and they bypass verification. As features are added, the temptation will be to add more bridge mutations because they're the path of least resistance. Each one makes the notification system less useful and the verification layer less meaningful.

The second biggest weakness is **test coverage**. The systems/ directory has zero unit tests. The 7 fake DES scenarios provide false confidence. The real DES coverage is thin for world travel, turn phases, auto-explore, and chest operations. A developer can break these systems and not know it.

## Where to go from here

1. **Stop refactoring. Build features.** The architecture is good enough. The bridge mutations are technical debt, not blockers. The notification system is underused, not broken. Ship gameplay. The refactor has been going on long enough.

2. **Convert bridge mutations opportunistically.** When a system needs modification for a feature, convert its bridge mutation to atomic mutations at that time. Don't do a dedicated "convert all bridges" pass — it's high effort for low immediate value.

3. **Write DES scenarios for every new feature.** The test coverage gap is the most dangerous weakness. Every new command should have a DES scenario before it's considered done. This is cheap (JSON files) and it's the only regression safety net.

4. **Delete the fake DES scenarios.** They're actively harmful. Replace them with real scenarios or remove them.

5. **Wire notify.rs as systems get touched.** When movement is modified, wire PlayerPositionChanged to trigger pickup/FOV/adaptation. When the turn system is modified, wire TurnAdvanced to trigger subsystem ticks. Don't do it preemptively — do it when the system is already open for changes.

6. **Clean up the Mutation enum.** Remove the duplicate variants (SpendAp, AddHp, etc.) in a single pass. This is a grep-and-replace task that reduces confusion about which variant to use.

## Things to watch out for

**Agent pattern replication.** The bridge mutations are now the pattern agents will copy. If a new system is added and the developer looks at how WorldMove works, they'll create another bridge mutation. The standing architecture document (VERIFIED_STATE_STORE.md) says to prefer atomic mutations, but the code says bridge mutations are fine. Code wins over documentation every time. The first few features built on this architecture will set the precedent — review them carefully.

**Mutation enum growth.** The enum currently has ~70 variants. Every new feature adds variants. Unlike the old Effect enums (which were domain-scoped), the Mutation enum is a single flat namespace. At 150+ variants it will become unwieldy. Consider domain-scoping (PlayerMutation, WorldMutation) if it gets there.

**The rng clone-writeback pattern.** It works but it's easy to forget. Every new command handler that uses rng needs the clone-writeback dance. If someone forgets the writeback, determinism silently breaks. Consider extracting this into a helper function that makes the pattern impossible to get wrong.

**notify.rs ordering.** Reactions fire in the order transitions are detected, which is the order mutations are applied. If two mutations in the same batch both produce transitions, the reactions for the first transition fire before the reactions for the second. This is usually fine but it means mutation ordering within a Vec\<Mutation\> is semantically meaningful. Systems that produce mutations need to be aware of this.

**The parallel trace systems.** Both `state.trace` (Effect-based) and `state.mutation_log` (Mutation-based) exist. DES assertions check both. This works but it means the same assertion can match different things depending on which path a command took. When the old Effect trace is eventually removed, some DES assertions may silently stop matching and the scenarios will fail. Plan for this.
