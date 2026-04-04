---
name: codebase-renderer
description: Rendering pipeline, subsystems, and extension points. Use when working on visual output, adding new rendering effects, modifying tile/entity appearance, or debugging rendering issues.
---

# Codebase: Renderer

**Location**: `src/renderer/` (13 modules)

## Architecture

`Renderer` is a coordinator struct that owns all rendering subsystems. It reads `GameState` but never modifies it.

```rust
pub struct Renderer {
    config: RenderConfig,           // data/render_config.json
    lighting: LightingRenderer,
    effects: EffectsRenderer,
    entities: EntityRenderer,
    tiles: TileRenderer,
    camera: Camera,
    frame_limiter: FrameLimiter,
    viewport_culler: ViewportCuller,
    particle_system: ParticleSystem,
    animation_system: AnimationSystem,
    theme_manager: ThemeManager,    // data/themes.json
    procedural_effects: ProceduralEffects,
    effects_manager: EffectsManager, // data/effects_config.json
}
```

## Modules

| Module | File | Responsibility |
|--------|------|----------------|
| `TileRenderer` | `tiles.rs` | Render map tiles with FOV/lighting |
| `EntityRenderer` | `entities.rs` | Render enemies, NPCs, items, player |
| `LightingRenderer` | `lighting.rs` | Apply light map to rendered cells |
| `EffectsRenderer` | `effects.rs` | Visual effects (hit flash, beams, projectiles) |
| `ParticleSystem` | `particles.rs` | Particle effects |
| `AnimationSystem` | `animations.rs` | Sprite/glyph animations |
| `Camera` | `camera.rs` | Viewport scrolling, smooth follow |
| `ThemeManager` | `themes.rs` | Color themes from `data/themes.json` |
| `EffectsManager` | `effects_config.rs` | Effects DSL from `data/effects_config.json` |
| `ProceduralEffects` | `procedural.rs` | Procedurally generated visual effects |
| `ViewportCuller` | `performance.rs` | Skip rendering off-screen tiles |
| `FrameLimiter` | `performance.rs` | Target FPS control |
| `RenderConfig` | `config.rs` | Loaded from `data/render_config.json` |

## Main Render Call

```rust
// Called from main.rs render()
renderer.render_game(
    frame,
    area,
    state,
    frame_count,
    look_cursor,    // Option<(i32, i32)> for look mode
    debug_console,  // bool
);
```

## Render Pipeline (inside `render_game`)

1. Camera update — smooth follow player position
2. Viewport culling — determine visible tile range
3. Tile pass — render each visible tile with FOV/lighting/theme
4. Entity pass — render items, enemies, NPCs, player on top of tiles
5. Effects pass — hit flash, projectile trails, light beams
6. Particle pass — particle effects
7. Animation pass — animated glyphs

## Tile Rendering

Tiles are rendered based on:
- `state.visible` — currently in FOV → full color
- `state.revealed` — seen before but not in FOV → desaturated
- Neither → not rendered (black)
- `state.light_map` — modifies brightness

## Entity Rendering

Entities are rendered with:
- Glyph from entity definition
- Color from theme + adaptation visual effects
- Effects DSL applied (blink, glow, pulse, shimmer)
- Hit flash overlay from `state.world.visual_effects`

## Visual Effects DSL

Defined in `data/effects_config.json`. Applied to entities via `effects` field in data files.

```
"B(@3 &Cyan)"       — Blink at speed 3, cyan
"G(&Yellow)"        — Glow yellow
"P(@2 &Red)"        — Pulse at speed 2, red
"S(@1 &White,Blue)" — Shimmer between white and blue
```

## Adaptation Visual Effects

`state.get_adaptation_visual_effects()` returns `Vec<VisualEffect>` based on player adaptations:
- `Prismhide` → Shimmer (cyan/white)
- `Sunveins` → Pulse (yellow)
- `Mirage Step` → Fade (light blue)
- `Saltblood` → Glow (white)
- `Phase Walking` → Drift (magenta)
- `Storm Affinity` → Wave (cyan)

## Configuration

`data/render_config.json` controls:
- Performance settings (target FPS, viewport size)
- Particle system settings
- Animation settings
- Default colors and styles

`data/themes.json` — color themes for tiles, entities, UI elements.

## Adding New Visual Effects

1. Add effect definition to `data/effects_config.json`
2. Reference by string in entity/tile data files via `effects` field
3. `EffectsManager` parses and applies at render time

## Adding New Particle Types

1. Add variant to `ParticleType` enum in `particles.rs`
2. Implement spawn/update/render logic
3. Trigger via `state.world.visual_effects` methods

## Satellite Terminals

`src/satellite.rs` — separate TUI processes that connect to the main game via IPC socket (`/tmp/saltglass-steppe.sock`). Each satellite renders a subset of game state:
- `log-ui` — message log
- `game-log-ui` — game + log combined
- `status-ui` — player status
- `inventory-ui` — inventory
- `debug-ui` — debug info

Launch with: `saltglass-steppe --log-ui` etc.
