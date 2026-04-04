# Components

> Updated 2026-04-04 after cleanup: ability methods removed from light/crystal/void, 7 custom generation algorithms deleted (terrain-forge handles all generation), 27 dead methods removed from state.rs, fake DES scenarios deleted.

## Core Game Components

### GameState (`src/game/state.rs`)
Central hub — all systems read/write through it. Contains three sub-structs:
- **PlayerState** (`player_state.rs`): Position, HP/AP, inventory, equipment, refraction, adaptations, status effects, faction reputation, quest log, skill state, and all resource systems (psychic, light, void, crystal).
- **WorldState** (`world_state.rs`): Current tile map, entity lists (enemies, NPCs, items, chests, interactables), storm state, weather, lighting, encounter state, world map navigation. Maintains spatial indexes (`enemy_positions`, `npc_positions`, etc.) rebuilt on load.
- **NarrativeEngine** (`narrative_engine.rs`): Quest log (active/completed), story model (chapter + flags), world history timeline, triggered effects with timers, tutorial progress.

Turn processing, event dispatch, save/load serialization, and all accessor methods live in `state.rs`.

### Map System (`src/game/map.rs`, `world_map.rs`)
- **TileMap** (`Map`): 250×110 grid. Tile types include Floor, Wall, Glass, StairsDown, StairsUp, WorldExit, and data-driven variants via string IDs. FOV calculation, A* pathfinding, tile metadata storage.
- **WorldMap**: 192×64 overworld. Each cell stores biome, terrain, elevation, POI, resources, connectivity, and level. Faction territories generated per-seed. Deterministic tile seeds via `tile_seed(wx, wy)`.

### Combat (`src/game/combat.rs`, `combat_actions.rs`, `systems/combat.rs`)
Turn-based with AP costs. Hit chance and damage use deterministic formulas factoring reflex, armor, weapon stats, and status effects. Melee and ranged attacks. Mock system (`combat_always_hit`, `combat_fixed_damage`) enables deterministic DES testing.

### Quest System (`src/game/quest.rs`, `narrative_engine.rs`)
Multi-objective quests with types: Kill, Collect, Reach, Talk, Examine, Interact, Explore. Act progression and faction alignment. Event hooks: `on_enemy_killed`, `on_item_collected`, `on_npc_talked`, `on_position_changed`, `on_interact`, `on_examine`, `on_turn_passed`. Data in `data/quests.json` and `data/main_questline.json`.

### Enemy AI (`src/game/systems/ai.rs`)
Four behavior strategies: StandardMelee, RangedOnly, Healer, SuicideBomber. Demeanor system (hostile, neutral, defensive) with flee thresholds. AI selects actions based on distance to player, HP percentage, and behavior type.

### Storm System (`src/game/storm.rs`, `systems/storm.rs`)
Glass storms with 7 edit types: Glass, Rotate, Swap, Mirror, Fracture, Crystallize, Vortex. All fully implemented. Intensity scaling, countdown timers, and storm forecasting.

### Skill System (`src/game/skills.rs`)
7-category tree with 35 skills. Canvas-based UI for navigation. Prerequisites, passive effects, active abilities. Data in `data/skill_trees.json`.

### Adaptation System (`src/game/adaptation.rs`)
Mutations triggered by refraction exposure. Stat modifiers, immunities, abilities. Fully wired into combat (damage modifiers) and movement (phase-through). Social consequences via faction reputation changes.

### Special Systems
- **Light** (`light.rs`): Resource accumulation only — ability methods removed. `LightSystem` tracks light level on `PlayerState`.
- **Crystal Resonance** (`crystal_resonance.rs`): Resource tracking + crystal placement on map — ability methods removed. `CrystalSystem` on `PlayerState`.
- **Void Energy** (`void_energy.rs`): Resource accumulation only — ability methods removed. `VoidSystem` on `PlayerState`.
- **Psychic** (`psychic.rs`): 3 working effects: `stun_aoe`, `guaranteed_hit`, `phasing`. Cooldown-based. `PsychicState` on `PlayerState`.

## Procedural Generation

### Tile Generator (`generation/tile_generator.rs`)
Orchestrates the full pipeline: `TileParams` → terrain-forge adapter → environmental props → enemy/item/NPC spawning → microstructures → settlement stamping (for towns) → GSB connectivity pass → `GeneratedTile`.

### Terrain Forge Adapter (`generation/terrain_forge_adapter.rs`)
Bridges the `terrain-forge` crate. `TerrainForgeGenerator::generate_tile_with_seed()` produces base terrain using biome profiles, algorithm layers, and POI layouts. All terrain generation goes through this — custom algorithms were removed.

### Connectivity (`generation/connectivity.rs`)
Glass Seam Bridging (GSB): flood fill to find regions, Delaunay pruning, frustum ray refinement, gradient descent for natural-looking tunnels. `ensure_connectivity()` called as final pass.

### Constraint System (`generation/constraints.rs`)
Post-generation validation. `validate_constraints()` runs all rules. `are_critical_constraints_satisfied()` for hard requirements. `calculate_satisfaction_score()` for soft quality.

### Settlement Generation (`generation/settlement/`)
Town/Village/City tiers. Building placement, A* road pathfinding, decorations, faction theming, population scaling. Structures stamped from `StructureLibrary` prefabs.

### World Generator (`generation/world_gen.rs`)
Overworld creation: biomes, terrain types, POI placement, faction territories, roads, quest location assignment.

## UI Components

### Input Handler (`src/ui/input.rs`)
Central input dispatcher. Routes keypresses to the active screen/menu. Manages UI state transitions, debug console, dialog boxes.

### Game View (`src/ui/game_view.rs`)
Main viewport rendering: tile map, entities, damage numbers, debug console overlay, death screen.

### HUD (`src/ui/hud.rs`)
Side panel (stats, equipment, status effects) and bottom panel (game log).

### Menus (~20 modules)
Main, inventory, skills, trade, crafting, quest log, wiki, book reader, world map, debug, faction, psychic, void, crystal, light, chest, storm forecast, issue reporter, ARIA interface. All wired and reachable from gameplay.

## Renderer

### Pipeline (`src/renderer/mod.rs`)
`Renderer::render_game()` orchestrates sub-renderers in order:
1. **TileRenderer** (`tiles.rs`) — theme-aware tile rendering with light dimming
2. **LightingRenderer** (`lighting.rs`) — dynamic multi-source lighting, viewport culling, dirty flags
3. **EntityRenderer** (`entities.rs`) — player, enemies, NPCs, items with lighting applied
4. **EffectsRenderer** (`effects.rs`) — visual effect compositing
5. **ParticleSystem** (`particles.rs`) — 6 types: Sparkle, Glow, Float, Drift, Pulse, Shimmer
6. **AnimationSystem** (`animations.rs`) — screen shake, glow, blink
7. **ProceduralEffects** (`procedural.rs`) — weather particles, ambient effects

Camera with smooth scrolling. Theme manager for color schemes. Frame limiter for performance. Effects quality/performance modes.

## Testing

### DES (`src/des/mod.rs`)
Scenario interpreter for headless gameplay testing. Loads JSON scenarios from `tests/scenarios/`. Sets up game state, executes action sequences, checks assertions. Supports scenario inheritance (`BASE_*` files), mocks, snapshots. Run via `cargo test --test des_scenarios`.
