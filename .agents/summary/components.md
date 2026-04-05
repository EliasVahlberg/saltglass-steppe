# Components

## Core Game Components

### GameState (`src/game/state.rs`)
Central hub for all gameplay state. Contains `PlayerState`, `WorldState`, `NarrativeEngine`, spatial indices, turn processing, and the VERA dispatch system. All systems access state through this struct.

Key responsibilities:
- `dispatch(Command)` — VERA command dispatch to rule functions
- `apply_effect(Effect)` — mechanical state mutation
- `end_turn()` — executes `TurnPhase::sequence()` in order
- `run_reactions(effects)` — deferred reaction chain (max depth 10)
- World travel, save/load, FOV/lighting updates

### Effects System (`src/game/effects/`)

| File | Purpose |
|------|---------|
| `mod.rs` | Effect, Command, TurnPhase, Presentation, RuleOutput enums |
| `apply.rs` | `apply_effect()` — mechanical match arms for all effect variants |
| `context.rs` | `QueryContext` (read-only state view for rules), `TestContext` (test builder) |
| `trace.rs` | `Trace`, `TraceEntry`, `TraceSource` — records effects during DES runs |

### Rules (`src/game/rules/`)

Pure functions: `(args, &QueryContext, &mut ChaCha8Rng) → RuleOutput`

| File | Functions | Tests |
|------|-----------|-------|
| `item.rs` | `rule_use_item`, `rule_use_item_on_tile` | 7 |
| `movement.rs` | `rule_move` → `MoveOutput` | 7 |
| `combat.rs` | `rule_melee_attack`, `rule_ranged_attack` | 7 |
| `actions.rs` | `rule_wait`, `rule_rest`, `rule_equip`, `rule_unequip`, `rule_allocate_stat`, `rule_use_psychic` | 7 |
| `turn.rs` | `rule_tick_time`, `rule_check_encounters`, `rule_check_adaptation` | 4 |

### Systems (`src/game/systems/`)

ECS-style systems implementing the `System` trait (`update`, `on_event`). Currently called via bridge effects from VERA dispatch.

| System | File | Status |
|--------|------|--------|
| AI | `ai.rs` | Bridge effect (`PlayerEffect::RunAI`). 4 behaviors: StandardMelee, RangedOnly, Healer, SuicideBomber |
| Storm | `storm.rs` | Bridge effect (`MapEffect::TickStorm`). 7 edit types: Glass, Rotate, Swap, Mirror, Fracture, Crystallize, Vortex |
| Combat | `combat.rs` | Post-processing: swarm aggro, enemy death handling |
| Movement | `movement.rs` | NPC interaction, world transitions, item pickup |
| Status | `status.rs` | Bridge effect (`PlayerEffect::TickStatusEffects`). Ticks durations, applies damage |
| Loot | `loot.rs` | Called via `EventEffect::LootDrop` reaction |
| Quest | `quest.rs` | Stub — quest logic lives in `game/quest.rs` |

## Procedural Generation (`src/game/generation/`)

### Core Pipeline
- `tile_generator.rs` — Orchestrates tile map generation
- `terrain_forge_adapter.rs` — Bridges the `terrain-forge` crate for base terrain
- `connectivity.rs` — Glass Seam Bridging algorithm for region connectivity
- `constraints.rs` — Post-generation constraint validation
- `quest_constraints.rs` — Validates map meets quest requirements

### Content Generation
- `spawn.rs` — Entity population from biome spawn tables
- `microstructures.rs` — Small structural features
- `structure_library.rs` — Prefab structure loading and stamping
- `environmental_props.rs` — Decorative props
- `feature_materializer.rs` — Story hooks, NPCs, loot placement

### World Generation
- `world_gen.rs` — Overworld map with biomes, POIs, factions, roads
- `settlement/` — Town and village generation (layout, buildings, roads, population)

### Narrative Generation
- `narrative.rs` — Story fragments, faction influence, narrative seeds
- `narrative_templates.rs` — Markov chains, template filling, contextual text
- `story.rs` — Story model, characters, events, faction lore
- `grammar.rs` — Grammar-based text expansion
- `templates.rs` — Content template system with inheritance

### Supporting
- `biomes.rs` — Biome profiles, hazards, environmental features
- `events.rs` — Dynamic event system with triggers and consequences
- `loot.rs` — Loot table generation
- `spatial.rs` — Poisson disk sampling
- `algorithm.rs` — Generation algorithm framework
- `config.rs` — Generation configuration loader

## UI Components (`src/ui/`)

### Core
- `input.rs` — Input handler, routes keypresses to appropriate handler
- `game_view.rs` — Main game viewport rendering
- `hud.rs` — HUD panels (health bar, inventory bar, side panel)
- `menu.rs` — Main menu, class select, seed input

### Game Menus
`inventory_menu.rs`, `skills_menu.rs`, `crafting_menu.rs`, `trade_menu.rs`, `quest_log.rs`, `wiki.rs`, `book_reader.rs`, `faction_menu.rs`, `psychic_menu.rs`, `void_menu.rs`, `crystal_menu.rs`, `light_menu.rs`, `chest_ui.rs`

### Debug/Dev
- `debug_menu.rs` — Debug overlay with tabs (info, states, commands, performance)
- `issue_reporter.rs` — In-game issue reporting
- `storm_forecast.rs` — Storm forecast display

### Special
- `world_map.rs` — Overworld map view with biome/faction overlays
- `aria_interface.rs` — Aria NPC terminal interface
- `theme.rs` — UI color themes

## Renderer (`src/renderer/`)

Read-only rendering layer. Never mutates GameState.

- `mod.rs` — Renderer orchestration, theme management
- `tiles.rs` — Tile rendering with lighting
- `entities.rs` — Player, enemy, NPC, item rendering
- `lighting.rs` — Dynamic lighting calculation
- `particles.rs` — Particle system (sparkle, glow, drift, shimmer)
- `animations.rs` — Animation system (blink, glow, screen shake)
- `effects.rs` — Visual effects rendering
- `procedural.rs` — Weather particles, ambient lighting, heat shimmer
- `themes.rs` — Color theme management
- `camera.rs` — Smooth camera following
- `config.rs` — Render configuration

## DES (`src/des/mod.rs`)

Debug Execution System — headless, deterministic gameplay testing. Parses JSON scenarios, executes actions, checks assertions. Supports scenario inheritance, mocks, map setup, entity spawning, and ~50 assertion types.

## Domain Modules (`src/game/`)

| Module | Purpose |
|--------|---------|
| `map.rs` | Tile map, FOV computation, pathfinding |
| `world_map.rs` | Overworld biomes, terrain, POIs |
| `enemy.rs` | Enemy definitions, behavior context, spawning |
| `npc.rs` | NPC definitions, dialogue, actions |
| `item.rs` | Item definitions, data loading |
| `quest.rs` | Quest system, objectives, progression |
| `combat.rs` | Combat formulas (hit chance, damage) |
| `skills.rs` | Skill trees, passive bonuses, abilities |
| `adaptation.rs` | Refraction adaptations (mutations) |
| `storm.rs` | Storm forecasting, edit type generation |
| `encounter.rs` | Encounter generation, flee mechanics |
| `dialogue.rs` | Dialogue trees, conditions, actions |
| `equipment.rs` | Equipment slots, stat recalculation |
| `trading.rs` | Trade interface, buy/sell |
| `crafting.rs` | Recipe system |
| `status.rs` | Status effect definitions, ticking |
| `psychic.rs` | Psychic abilities (3 working: stun_aoe, guaranteed_hit, phasing) |
| `void_energy.rs` | Void energy/exposure tracking |
| `crystal_resonance.rs` | Crystal formation system |
| `light.rs` | Light energy tracking |
| `save.rs` | Save/load with versioning and checksums |
| `faction.rs` | Faction definitions, reputation |
| `event.rs` | GameEvent enum (legacy, being replaced by reactions) |
| `visual_effects.rs` | Damage numbers, hit flash, beams, projectiles |
