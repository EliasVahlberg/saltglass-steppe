# Visual Animation Effects System

## Overview

The visual animation effects system provides dynamic visual feedback through blink effects, glow animations, and screen shake. This system is designed to be data-driven, performant, and decoupled from the core game logic.

## Features

### Animation Types

1. **Blink Effects**
   - Alternates between two colors at configurable intervals
   - Useful for highlighting important elements or indicating status changes
   - Duration-based with automatic cleanup

2. **Glow Effects**
   - Smooth color transitions that cycle continuously
   - Creates pulsing/breathing effects for atmospheric elements
   - Configurable intensity steps and cycle duration

3. **Screen Shake**
   - Temporary viewport offset for impact feedback
   - Configurable intensity, frequency, and duration
   - Sine-wave based motion for natural feel

### Technical Architecture

#### Core Components

- **AnimationSystem**: Main coordinator that manages all active animations
- **Animation**: Individual animation instances with timing and state
- **VisualAnimationConfig**: Data-driven configuration loaded from JSON

#### Integration Points

- **Renderer Integration**: Animations are applied during the final rendering pass
- **Style Application**: Animation effects modify ratatui styles for visual elements
- **Screen Shake**: Offsets camera position for viewport-wide effects

## Configuration

Animation settings are configured in `data/render_config.json`:

```json
{
  "visual_animations": {
    "blink": {
      "duration_ms": 500,
      "on_color": "White",
      "off_color": "DarkGray"
    },
    "glow": {
      "cycle_duration_ms": 1000,
      "base_color": "Yellow",
      "glow_color": "LightYellow",
      "intensity_steps": 5
    },
    "screen_shake": {
      "duration_ms": 200,
      "intensity": 2,
      "frequency_hz": 20.0
    }
  }
}
```

### Configuration Parameters

#### Blink Animation
- `duration_ms`: Total duration of the blink effect
- `on_color`: Color when blink is "on"
- `off_color`: Color when blink is "off"

#### Glow Animation
- `cycle_duration_ms`: Duration of one complete glow cycle
- `base_color`: Base color when not glowing
- `glow_color`: Peak glow color
- `intensity_steps`: Number of steps in the glow transition

#### Screen Shake
- `duration_ms`: How long the shake effect lasts
- `intensity`: Maximum offset in pixels
- `frequency_hz`: Shake frequency in hertz

## Usage

### Triggering Animations

```rust
// Add blink effect
renderer.add_blink_effect();

// Add glow effect
renderer.add_glow_effect();

// Add screen shake
renderer.add_screen_shake();
```

### Animation Lifecycle

1. **Creation**: Animations are created with configuration parameters
2. **Update**: Each frame, animations update their internal state
3. **Application**: Animation effects are applied to rendered elements
4. **Cleanup**: Completed animations are automatically removed

## Performance Considerations

### Optimizations

- **Efficient Updates**: Animations only update timing state, not heavy computations
- **Automatic Cleanup**: Completed animations are removed to prevent memory leaks
- **Minimal Allocation**: Animation state is pre-allocated and reused
- **Viewport Integration**: Screen shake works with existing viewport culling

### Performance Impact

- **CPU**: Minimal overhead - simple mathematical operations per frame
- **Memory**: Low memory footprint with automatic cleanup
- **Rendering**: Integrates with existing rendering pipeline without duplication

## Implementation Details

### Animation State Management

```rust
pub struct Animation {
    pub animation_type: AnimationType,
    pub start_time: Instant,
    pub duration: Duration,
    pub active: bool,
}
```

### Style Application

Animations modify ratatui styles during rendering:

```rust
// Apply animation effects to all spans
for row in &mut final_spans {
    for span in row {
        span.style = self.animation_system.get_combined_style(span.style);
    }
}
```

### Screen Shake Implementation

Screen shake works by offsetting the camera position:

```rust
let (shake_x, shake_y) = self.animation_system.get_screen_offset();
let adjusted_cam_x = cam_x + shake_x as i32;
let adjusted_cam_y = cam_y + shake_y as i32;
```

## Testing

The animation system includes comprehensive testing:

- **DES Integration**: Animation test scenario validates basic functionality
- **Unit Tests**: Individual animation components are tested
- **Integration Tests**: Full rendering pipeline with animations

### Test Scenario

The `animation_effects_test.json` scenario validates that animations don't interfere with core gameplay mechanics.

## Future Enhancements

### Potential Additions

1. **Animation Chaining**: Sequence multiple animations
2. **Easing Functions**: More sophisticated timing curves
3. **Color Interpolation**: Smooth color transitions
4. **Entity-Specific Animations**: Target specific game elements
5. **Animation Events**: Trigger game events when animations complete

### Extension Points

The system is designed for easy extension:

- Add new `AnimationType` variants
- Implement custom timing functions
- Create specialized animation behaviors
- Integrate with game event system

## Compatibility

- **Ratatui Integration**: Works seamlessly with ratatui's styling system
- **Performance**: Maintains 60 FPS target with multiple active animations
- **Configuration**: Hot-reloadable through JSON configuration
- **Modular Design**: Can be disabled without affecting other systems

## Troubleshooting

### Common Issues

1. **Animations Not Visible**: Check that visual_animations config is properly loaded
2. **Performance Issues**: Reduce number of simultaneous animations
3. **Color Issues**: Verify color names match ratatui's color constants

### Debug Information

Enable debug logging to see animation state:

```rust
// Animation system provides debug information
println!("Active animations: {}", animation_system.animations.len());
```
