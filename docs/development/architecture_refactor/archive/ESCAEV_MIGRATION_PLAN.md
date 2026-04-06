# ESCAEV Migration Plan — Rough Draft

> Status: ROUGH — for LeadDeveloper to refine into actionable tasks
> Date: 2026-04-03
> Reference: [ESCAEV v2 Proposal](ARCHITECTURE_PROPOSAL_ESCAEV_v2.md) · [Audit](CODEBASE_HEALTH_AUDIT.md)

## Branch Strategy

- Phase 0 + 0.5: directly on `main` (zero-risk additions + dead code deletion)
- Phase 1+: one branch per phase, merged when all DES passes

## Phase 0: Foundation (main)

Add new modules, zero changes to existing code.

1. Create `src/game/effects/mod.rs` — `GameEffect` enum (domain-scoped), `PresentationEffect` enum
2. Create `src/game/effects/apply.rs` — `apply_game_effect()`, `apply_presentation_effect()`
3. Create `src/game/effects/trace.rs` — `Trace` struct, recording, opt-in flag
4. Create `src/game/query_context.rs` — `QueryContext` struct, `From<&GameState>` impl
5. Start with `Resource` and `Item` effect domains only (enough for Phase 1)
6. Wire into `lib.rs` / `mod.rs` — compile, run `cargo test`, verify zero behavior change

## Phase 0.5: Dead Code Triage (main)

Deletions and decisions. Reference: CODEBASE_HEALTH_AUDIT.md Parts 1-4.

1. Delete confirmed dead files: `terminal_spawn.rs`, 4 dead algorithms (bsp, maze, voronoi, wfc)
2. Delete `data/structures/patterns/special/` (duplicates of `patterns/ruins/`)
3. Delete 7 deprecated schemas (aria_dialogues, floors, walls, lights, effects_config, skills, psychic_abilities, status_effects, structures_unified)
4. Delete 4 dead stubs in `end_turn` (generate_narrative_fragments, generate_biome_content, generate_template_content, check_dynamic_events)
5. Delete 16 dead pub methods in state.rs (10 narrative + 6 others per audit §1.2)
6. Decide: light/crystal/void ability methods → remove. Resource accumulation (ticks, item energy grants) → keep.
7. Delete dead ability methods from light.rs, crystal_resonance.rs, void_energy.rs. Keep `update()` and energy fields.
8. Delete 3 test-only algorithms (cellular_automata, drunkard_walk, simple_rooms) + their smoke tests
9. Delete `structure_generation.json` (only used by deprecated tilegen-tool)
10. Write migration sentinel DES scenarios: 3-5 scenarios with fixed seeds asserting exact numeric outcomes
11. Run full `cargo test`, verify everything passes

## Phase 1: use_item Extraction (branch: `escaev/phase-1-use-item`)

1. Write `use_item_rule()` in new module `src/game/rules/item.rs`
2. Modify `state.rs::use_item()` to: construct QueryContext → call rule → apply effects → record trace
3. Write Rule unit tests per item type (healing, void, crystal, map reveal, book, consumable)
4. Write DES trace tests with `expect_effects` + `at_end`
5. Run migration sentinels — verify RNG ordering preserved
6. Run full test suite
7. Merge to main

## Phase 2+: LeadDeveloper to refine

Phases 2-5 follow the same pattern. LeadDeveloper should break each into concrete tasks based on lessons from Phase 1. Key ordering:

- Phase 2: Movement (easier, second validation of pattern)
- Phase 3: Combat (hardest single-system extraction — start with process_enemy_death as first Reaction)
- Phase 3.5: Sub-state extraction (informed by QueryContext usage patterns from Phases 1-3)
- Phase 4: end_turn decomposition (one sub-phase at a time: status tick → storm → AI)
- Phase 5: DES trace assertions (additive, no behavior change)
