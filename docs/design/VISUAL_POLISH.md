# Visual Polish Feature Document

Goal: Add visual flair to enhance game feel without impacting core gameplay systems.

## Architecture Principles

- **Decoupled**: Visual effects are separate from game logic
- **Data-driven**: Effect definitions in data, not hardcoded
- **Frame-based**: Animation system ticks independently of game turns

## Core Animation System

All visual effects share a common animation infrastructure:

```rust
// Animation state stored separately from game state
struct VisualEffects {
    animations: Vec<Animation>,
    frame_counter: u64,
}

struct Animation {
    kind: AnimationKind,
    start_frame: u64,
    duration_frames: u16,
    position: (i32, i32),
    data: AnimationData,
}
```

## Feature List

### 1. Combat Hit Flash Effects (Priority: 8, Complexity: 4)
Brief color flash when entities take damage.
- Flash tile red/white on hit
- Duration: 2-3 frames
- Tags: animation, combat, feedback

### 2. Status Effect Visual Indicators (Priority: 7, Complexity: 3)
Color-code entities by active status effects.
- Burning: orange/red flicker
- Poisoned: green tint
- Frozen: cyan tint
- Bleeding: dark red pulse
- Tags: status-effects, color, feedback

### 3. Health Bar Styling (Priority: 7, Complexity: 2)
Gradient health bars with low-health warning.
- Green → Yellow → Red gradient
- Unicode block chars (█▓▒░)
- Pulse effect when < 25% HP
- Tags: ui, hud, color

### 4. Floating Damage Numbers (Priority: 7, Complexity: 5)
Numbers rise and fade when damage dealt.
- Physical: red
- Psy: blue
- Healing: green
- Rise 1 cell over ~10 frames, then fade
- Tags: animation, combat, feedback

### 5. Message Log Enhancements (Priority: 6, Complexity: 2)
Color-coded message types.
- Combat: red
- Loot/items: gold
- System: gray
- Dialogue: white
- Older messages fade to dim
- Tags: ui, messages, color

### 6. Projectile Trail Animation (Priority: 6, Complexity: 5)
Animate ranged attacks from source to target.
- Arrow: `-` or `→`
- Bolt: `*`
- Spell: `~` or `○`
- Trail fades behind projectile
- Tags: animation, combat, ranged

### 7. Color Theme System (Priority: 6, Complexity: 3)
Centralized color palette for theming.
- Theme struct with named colors
- Presets: classic, dungeon, ice, fire
- UI modules reference theme, not hardcoded colors
- Tags: color, theming, config

### 8. Environmental Ambient Animations (Priority: 5, Complexity: 4)
Subtle tile animations for atmosphere.
- Water: shimmer/ripple (cycle chars or colors)
- Torches: flicker orange/yellow
- Grass: occasional sway
- Tags: animation, environment, atmosphere

### 9. Smooth Camera/Viewport Transitions (Priority: 5, Complexity: 4)
Animate viewport shifts instead of snapping.
- Lerp camera position over 3-5 frames
- Triggered when player nears viewport edge
- Tags: animation, camera, polish

### 10. Death/Game Over Screen Polish (Priority: 4, Complexity: 3)
Dramatic death presentation.
- Screen tint to red
- ASCII skull/tombstone art
- Stats summary
- Text reveal animation
- Tags: ui, death, polish

## Data-Driven Definitions

Effects defined in `data/visual_effects.json`:

```json
{
  "hit_flash": {
    "duration_frames": 3,
    "colors": ["#FF0000", "#FFFFFF", "#FF0000"]
  },
  "status_colors": {
    "burning": { "base": "#FF6600", "alt": "#FF3300" },
    "poisoned": { "base": "#00FF00", "alt": "#008800" },
    "frozen": { "base": "#00FFFF", "alt": "#0088FF" }
  }
}
```

## Implementation Order

Recommended sequence based on dependencies and impact:

1. **Health Bar Styling** - Quick win, no dependencies
2. **Message Log Enhancements** - Quick win, improves feedback
3. **Color Theme System** - Foundation for other features
4. **Status Effect Visual Indicators** - Uses theme system
5. **Combat Hit Flash Effects** - Requires animation system base
6. **Floating Damage Numbers** - Builds on animation system
7. **Projectile Trail Animation** - Complex animation
8. **Environmental Ambient Animations** - Ambient, lower priority
9. **Smooth Camera Transitions** - Polish
10. **Death Screen Polish** - Final polish

## Testing Strategy

Each feature needs DES tests where applicable:
- Status indicators: spawn entity with status, assert render output
- Damage numbers: deal damage, assert animation spawned
- Message colors: trigger events, check message log state

For pure visual features, manual verification may be needed, but state changes (animation queue, message log) can be tested.
