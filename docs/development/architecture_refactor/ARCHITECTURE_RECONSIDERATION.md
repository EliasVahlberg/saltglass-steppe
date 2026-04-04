# Architecture Reconsideration

> Date: 2026-04-04
> Context: Before committing to ESCAEV, evaluate all options honestly.

## The Question

We have a working v2 proposal for ESCAEV (Event-Sourced Command Architecture with Effect Verification). Before proceeding, we should consider whether this is the right path, or whether alternatives would serve us better. Three paths are on the table:

### Path 1: Adopt ESCAEV as-is
Find existing implementations and crates that implement this pattern. Study professional examples. Leverage LLM parametric knowledge of established packages.

### Path 2: Customize the pattern
Modify ESCAEV to fit our specific needs. Potentially publish as a crate. Mitigate frictions that could become architectural weaknesses.

### Path 3: Continue researching
Look for more suitable patterns before committing.

Key evaluation criteria:
- How much work to change the codebase?
- Can we find existing implementations to learn from?
- Does the pattern have a simple "why this works" explanation?
- Does it fit our specific constraint (AI agent self-verification)?

---

## First viewpoint (system-agent)

### On Path 1: Adopt ESCAEV as-is

**The honest problem: ESCAEV doesn't exist as a named pattern.**

I searched for Rust crates and prior art. What exists in the ecosystem:

- **Enterprise CQRS/ES crates** (`esrc`, `esrs`, `eventcore`, `eventsourcing`): These are built for distributed systems with event stores, projections, and aggregate roots. They solve a completely different problem — persistence and eventual consistency across services. They're architecturally heavy, async-first, and assume a database backend. None of them are useful for a single-process game.

- **ECS frameworks** (`bevy_ecs`, `hecs`, `specs`, `legion`): These solve entity composition and system scheduling. We already rejected full ECS for testability reasons. These don't provide the command→effect→trace pattern.

- **CoQ's architecture** (C#, Part/Event system): Caves of Qud uses a component system where Parts are attached to objects and respond to Events via `WantEvent`/`HandleEvent`. This is closer to what we want — parts react to events independently, producing emergent behavior. But it's a component-event system, not a command-effect system. The key difference: CoQ's events are notifications that parts react to by mutating state directly. ESCAEV's effects are *descriptions of mutations* that get applied mechanically. CoQ doesn't have traces or effect verification.

- **The Rust roguelike architecture article** (rsaarelm, 2015): Describes the exact borrow checker problems we face — world state locked during entity iteration, can't borrow mutably and immutably simultaneously. Their solution was thread-local state with runtime borrow checking via `Cell`. This is the problem ESCAEV's QueryContext solves more cleanly.

**There are no existing implementations of ESCAEV to study.** The name is ours. The pattern is a hybrid of:
- Command Pattern (GoF)
- Event Sourcing (DDD/CQRS, but without the persistence layer)
- Effect Systems (functional programming, algebraic effects)
- The "action description" pattern from functional game architectures

No single crate implements this combination. LLMs will have parametric knowledge of each ingredient separately (Command Pattern, Event Sourcing, CQRS) but not of the specific combination we've designed. This means Path 1 as stated — "find existing implementations" — isn't viable. There's nothing to find.

**However**, the individual ingredients are well-understood. An LLM knows how to implement Command Pattern in Rust. It knows Event Sourcing. It knows how to write `match` on enums. The composition is novel, but the parts aren't.

### On Path 2: Customize and potentially publish as a crate

This is where I'd push back on the framing. Publishing a crate has real costs:

- **API stability pressure.** Once published, changing the Effect trait or Rule signature means a semver bump. During active development of saltglass-steppe, you want to iterate freely on the architecture. A published crate creates friction against iteration.
- **Generalization tax.** A crate needs to work for users who aren't you. Your Effect enum is domain-specific (CombatEffect, StormEffect). A generic crate would need trait-based extensibility, which adds complexity that doesn't serve your project.
- **Maintenance burden.** You already maintain terrain-forge, ogun, and oku on crates.io. Another crate is another thing to keep updated.

The counterargument: publishing forces you to think about the API carefully, and it's a portfolio piece. Both valid. But the timing is wrong — you haven't proven the pattern works yet. Phase 1 (use_item extraction) is the proof. Publishing before that proof is premature.

**My recommendation for Path 2:** Keep the architecture project-local until Phase 3 is complete. At that point, you'll know which parts are genuinely reusable and which are saltglass-specific. If the core (Effect enum, Trace, apply, QueryContext) is clean enough to extract, publish then. Not before.

### On Path 3: Continue researching

This is the responsible option to consider. Are there patterns we haven't looked at?

**Patterns I've considered and why they're worse fits:**

1. **Full ECS (Bevy/hecs/specs):** Already rejected. Testability problem for AI agents. Would be a rewrite, not a refactor.

2. **Actor Model:** Each entity is an actor with a mailbox. Actors send messages, react independently. Good for emergence, but: (a) Rust's actor frameworks (Actix, etc.) are async and network-oriented, (b) debugging actor message ordering is harder than debugging a trace, (c) turn-based games don't benefit from actor concurrency.

3. **Functional Reactive Programming (FRP):** State as a function of event streams. Elegant in theory, but Rust's FRP libraries are immature, and the pattern fights Rust's ownership model. Also, FRP is continuous-time — awkward for turn-based games.

4. **Redux/Elm Architecture:** Single state, actions dispatched through a reducer: `(State, Action) → State`. This is actually very close to ESCAEV but simpler — no Reactions, no Traces, no Phase Sequences. The reducer is the Rule. The action is the Command. The new state is the effect of applying the action.

   **This deserves serious consideration.** The Elm Architecture is proven, simple, and well-understood by LLMs. The question is whether it's *enough*. It gives you: pure reducer functions (testable), single state (inspectable), action log (traceable). It doesn't give you: Reactions (emergence), domain-scoped effects (granularity), or the GameEffect/PresentationEffect split.

5. **CoQ-style Part/Event system:** Parts attached to entities, reacting to events. This is the emergence model. But it requires a component registry, dynamic dispatch, and runtime event routing — all of which fight Rust's type system. And it doesn't provide traces or effect verification.

6. **Simple event bus expansion:** The v2 review's "lightweight alternative" (§8.1). Expand GameEvent, make process_events the central loop. Gets 40% of ESCAEV's value for 20% of the effort. But no traces, no Rule unit testing, no structural gate against scaffold-and-abandon.

### My verdict

**Path 2 is the right path, but without the crate publication (for now).**

Here's my reasoning:

1. **ESCAEV is not a standard pattern.** It's a custom hybrid. There are no crates to adopt, no reference implementations to study. This is a fact, not a problem — the pattern was designed for a specific constraint (AI agent self-verification) that standard patterns don't address.

2. **The closest standard pattern is the Elm Architecture** (reducer pattern). ESCAEV is essentially Elm + Reactions + Traces + domain-scoped effects. If ESCAEV feels too heavy, the fallback is: implement the Elm core (Command → Rule → State change) and add Reactions/Traces incrementally. This is actually what the phased migration already does — Phase 1 is pure Elm, Reactions don't appear until Phase 2-3.

3. **The "why this works" is simple and survives scrutiny:** "Functions that say what they would do, instead of doing it." This is the core of every functional architecture. ESCAEV just adds a recording layer (Trace) and a composition layer (Reactions) on top.

4. **The work estimate is modest.** ~6.4% of the codebase touched. The pattern targets exactly the files the audit identified as problematic. The 22 leaf modules, renderer, data loaders, generation pipeline — untouched.

5. **The risk of over-researching is real.** You have a working proposal, a review, a v2, a migration plan, and a feasibility assessment. The pattern has been stress-tested against 5 real systems. More research at this point is procrastination, not due diligence. The actual test is Phase 1 — writing real code.

**What I'd actually do:**

- Start Phase 0 + 0.5 on main (foundation + dead code cleanup). Zero risk.
- Branch for Phase 1 (use_item). This is the experiment.
- After Phase 1, do the retrospective the v2 review recommends.
- If Phase 1 succeeds: continue. If it doesn't: you've learned something concrete about why, and you can pivot to the simpler Elm-style reducer or the event bus expansion.

The worst outcome isn't picking the wrong pattern — it's spending another month researching patterns instead of writing the 200 lines of Phase 0 code that would tell you whether the pattern works.

### Points you might not have considered

1. **The Elm Architecture as a fallback/simplification.** If ESCAEV's grammar (Phase Sequences, Priority Chains, Sequential Execution) feels too heavy during implementation, you can drop those composition rules and keep just the core: Command → Rule → Effects → Apply. That's Elm. The composition rules are only needed for Phase 4 (end_turn). You might never need them if you decompose end_turn differently.

2. **The crate question has a middle ground.** You don't have to publish to crates.io to get the benefits of a separate crate. You can create a workspace member crate (`saltglass-effects` or similar) within the repo. This gives you: compile-time isolation, clean API boundary, forced separation of concerns — without the maintenance burden of a public crate. If it proves reusable later, publishing is a `cargo publish` away.

3. **The real competitor to ESCAEV isn't another pattern — it's "just expand the event system."** The v2 review called this out (§8.1): expand GameEvent to carry mutation data, make process_events central. This gets you 40% of the value for 20% of the effort. The question is whether 40% is enough. For AI agent self-verification specifically, I don't think it is — you need the Rule unit test layer, which requires the command→rule→effect split. But it's worth being explicit that this simpler path exists.
## First viewpoint (LeadDeveloper)

### Honest framing

Before evaluating the three paths, I want to name something: ESCAEV is not a known architectural pattern. It's a custom vocabulary we invented for this project's specific problems. The underlying ideas — command pattern, event sourcing, functional core/imperative shell — are well-established. But the specific combination (Rules, Reactions, Derives, Traces, Phase Sequences, Priority Chains) is ours. This matters because it affects what "find existing implementations" and "find a crate" actually mean in practice.

---

### Path 1: ESCAEV as-is + study existing implementations/crates

**The good:** Studying how others solve the same problem is always valuable. The discipline of looking outward before building prevents NIH syndrome.

**The honest problem:** There are no ESCAEV implementations to study, because ESCAEV doesn't exist outside this project. What exists are the constituent patterns:

- **Command pattern** — universal, well-documented, but doesn't address the Reaction/cascade problem
- **Event sourcing / CQRS** — mature in enterprise (Rust crates: `cqrs-es`, `eventstore-rs`). But these are designed for distributed systems with event stores, projections, and eventual consistency. A single-player turn-based game doesn't need any of that. Adapting them would mean fighting abstractions designed for a different problem.
- **ECS with events** — Bevy's event system is the closest gamedev equivalent. But we already rejected full ECS, and Bevy's events are notification-only (like our current GameEvent), not mutation-carrying.
- **Elm Architecture / TEA** — `(Model, Msg) → (Model, Cmd)`. Very close to ESCAEV's core idea (pure function returns effects). Simpler — no Reactions, no Derives, no Phase Sequences. Rust implementations exist (`iced`, `druid` used similar patterns) but for UI, not game logic.
- **Redux pattern** — Actions → Reducer → State, with middleware for side effects. Well-understood, massive ecosystem in JS/TS. No mature Rust game crate for this.

**On crates specifically:** I searched my knowledge for Rust crates that implement "command → effect list → mechanical application" for game logic. The honest answer is: I'm not aware of one that fits. The closest things are:

- `bevy_ecs` — too heavy, wrong abstraction level
- `specs` / `legion` — same problem, full ECS
- `cqrs-es` — enterprise CQRS, wrong domain
- `reducer` — minimal Redux-like pattern, but too simple (no reactions, no traces)

The LLM parametric knowledge argument is real but cuts both ways: if we use a well-known crate, I can leverage training data about its usage patterns. But if no well-known crate fits, forcing one in creates more friction than building the right thing.

**My assessment of Path 1:** Worth doing the research (study Elm Architecture, Redux in games, Caves of Qud's event system, Cogmind's action system). Not worth forcing an ill-fitting crate. The research should inform our design, not replace it.

---

### Path 2: Modify ESCAEV, publish as a crate

**The good:** The discipline of designing a public API often improves architecture. Separating the framework from the game forces clean boundaries. You enjoy making crates, and that motivation matters.

**The concerns:**

1. **Premature extraction.** We haven't proven ESCAEV works yet. Phase 1 hasn't been done. Extracting a crate from an unproven architecture means we're publishing our hypothesis, not our knowledge. If Phase 3 reveals that the Rule signature needs to change, we have a published crate with the wrong API.

2. **One consumer.** A crate with exactly one consumer (saltglass-steppe) has all the maintenance overhead of a public API (semver, docs, backwards compatibility) with none of the benefits (community feedback, battle-testing, shared maintenance). The cost/benefit only flips if other projects adopt it.

3. **The LLM argument is weak for new crates.** LLMs have parametric knowledge of crates that were popular *before their training cutoff*. A newly published niche crate won't be in any model's training data for 1-2 years. The benefit is real but delayed and uncertain.

4. **Scope creep risk.** Maintaining a separate crate while also migrating the game creates two fronts. Every ESCAEV design change requires: update the crate, publish, update saltglass-steppe's dependency, test. This friction slows iteration during the critical Phase 1-3 period when the design is still being validated.

**When Path 2 makes sense:** After Phase 3 succeeds. At that point, the pattern is proven across three different systems (items, movement, combat), the Effect enum has real variants, and the QueryContext pattern has been battle-tested. Extracting a crate from working code is much safer than extracting from a design document.

**My assessment of Path 2:** Not yet. Revisit after Phase 3.

---

### Path 3: Continue researching alternative patterns

**The good:** Intellectual honesty. Maybe there's something better we haven't considered.

**What I'd want to investigate:**

1. **"Functional core, imperative shell" without the ESCAEV vocabulary.** The simplest version: extract pure functions (Rules) from state.rs, have them return data (Effects), apply mechanically. No Trace, no Reactions, no Phase Sequences, no formal grammar. Just "functions return data instead of mutating state." This is ESCAEV Phase 1 without the rest. The v1 review called this "40% of the value for 20% of the effort" (§8.1). Is that enough?

2. **The Elm Architecture adapted for turn-based games.** `(GameState, Command) → (GameState, Vec<SideEffect>)`. Single update function, no Reaction chains. Simpler than ESCAEV but doesn't model emergent behavior from cascading effects. Would this be sufficient for storms → adaptations → faction consequences?

3. **Lightweight event expansion.** Don't change the architecture at all. Just: (a) make GameEvent carry mutation data, (b) expand process_events to be the central dispatch, (c) have systems return events instead of mutating directly. This is the cheapest path and builds on existing code. But it doesn't give us Rule unit testing or Traces.

**What I don't think we'll find:** A dramatically different pattern that solves our specific problems better than ESCAEV. The problems are: opaque mutations in a god object, no unit-test layer for game logic, no structural gate against incomplete integration. These are addressed by any variant of "pure functions return effect descriptions." The question is how much formalism we need around that core idea.

**My assessment of Path 3:** A brief, time-boxed investigation (1-2 hours) is worth it. Specifically: look at how Caves of Qud, Cogmind, and DCSS handle action → effect → reaction chains. Look at Elm Architecture in Rust. Then decide.

---

### The point you might not have considered

**The risk of the migration itself becoming scaffold-and-abandon.**

We build Phase 0 infrastructure. We extract use_item in Phase 1. Then Phase 2 is harder than expected, or a gameplay feature becomes urgent, or motivation shifts. We end up with a hybrid codebase: some systems use ESCAEV (use_item), most don't. The hybrid is harder to understand than either the old or new architecture, because now there are two patterns to learn.

This is the same anti-pattern the audit identified — vertical scaffolding without horizontal completion — but at the architectural level instead of the feature level.

Mitigation: each phase must be independently valuable. Phase 0 adds infrastructure but changes nothing. Phase 1 makes use_item testable — that's valuable even if we stop there. Phase 2 makes movement testable. Each phase improves the codebase even if the migration never completes. The v2 proposal claims this, and I believe it for Phases 0-3. Phase 4 (end_turn) is where the "independently valuable" claim gets shaky — a partially decomposed end_turn is arguably worse than the current monolith.

**The other thing:** Before any architectural migration, the dead code cleanup (Phase 0.5) and the 18 dangling data cross-references should be fixed regardless. These are bugs and maintenance burden that exist independent of architecture choice. If we do nothing else, do Phase 0.5.

---

### My recommendation

**Modified Path 1, with a gate before committing.**

1. **Do Phase 0.5 now** (dead code cleanup). This is valuable regardless of architecture choice. It reduces the codebase by ~3,600 LOC of dead weight and fixes data integrity issues.

2. **Do a brief Path 3 investigation** (time-boxed, 1-2 hours). Study Elm Architecture in Rust, look at how other roguelikes handle action→effect chains. Not to find a replacement for ESCAEV, but to validate that the core idea (pure functions returning effect descriptions) is the right abstraction level, and to see if the formalism (Reactions, Derives, Phase Sequences) is necessary or if a simpler variant suffices.

3. **Do Phase 0 + Phase 1 as a proof of concept.** Implement the Effect enum, QueryContext, and extract use_item. This is low-risk and answers the real question: does the pattern work in practice, not just on paper?

4. **Retrospective after Phase 1.** Was it worth it? Did the Effect enum feel right? Did QueryContext work ergonomically? Did the Rule unit tests actually catch anything? Based on this, decide whether to continue to Phase 2 or simplify the approach.

5. **Revisit Path 2 (crate extraction) after Phase 3**, if the pattern has proven itself across three systems.

Don't publish a crate from a design document. Don't commit to the full 5-phase migration before proving Phase 1. Don't skip the dead code cleanup. And don't let the architecture work delay the game — the roadmap says a 30-minute play session is possible now. Protecting that while improving the foundation is the right balance.
