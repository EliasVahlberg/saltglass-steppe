# Procedural Visual Effects System

## Overview

The procedural visual effects system generates dynamic environmental effects using algorithmic techniques like noise functions and particle systems. This creates immersive atmospheric effects such as weather particles, ambient lighting variations, and other environmental phenomena that enhance the game's visual appeal.

## Features

### Weather Effects

1. **Rain Particles**
   - Diagonal falling particles with variable speed
   - Multiple character representations (|, \, /)
   - Blue color variations for realistic appearance
   - Configurable intensity and speed

2. **Snow Particles**
   - Gentle floating particles with drift
   - Soft character representations (*, ·, °)
   - White and light gray colors
   - Slower movement with natural variation

3. **Dust Particles**
   - Ambient floating particles using noise-based movement
   - Subtle character representations (·, ˚, °)
   - Gray color variations
   - Perlin noise for natural, organic movement patterns

### Atmospheric Effects

1. **Ambient Lighting Variations**
   - Subtle lighting fluctuations using noise functions
   - Creates dynamic atmosphere without being distracting
   - Configurable variation speed and intensity

2. **Heat Shimmer**
   - Subtle position offsets for heat distortion effects
   - Uses Perlin noise for natural shimmer patterns
   - Minimal performance impact with realistic appearance

3. **Dust Motes**
   - Floating particles in lit areas
   - Adds life to static environments
   - Performance-optimized particle management

### Technical Architecture

#### Core Components

- **ProceduralEffects**: Main system coordinator managing all procedural effects
- **WeatherParticle**: Individual weather particle with physics and rendering data
- **ProceduralConfig**: Data-driven configuration for all procedural effects
- **Noise Functions**: Perlin noise for natural, organic movement patterns

#### Algorithm Integration

- **Perlin Noise**: Used for natural movement patterns and ambient variations
- **Particle Systems**: Manages weather particle lifecycle and physics
- **Procedural Generation**: Algorithmic creation of dynamic visual effects

## Configuration

Procedural effects are configured through the ProceduralConfig structure:

```rust
ProceduralConfig {
    weather: WeatherConfig {
        rain: WeatherEffectConfig {
            enabled: false,
            intensity: 0.3,
            speed: 2.0,
            characters: vec!["|", "\\", "/"],
            colors: vec!["Blue", "LightBlue"],
        },
        snow: WeatherEffectConfig {
            enabled: false,
            intensity: 0.2,
            speed: 1.0,
            characters: vec!["*", "·", "°"],
            colors: vec!["White", "LightGray"],
        },
        dust: WeatherEffectConfig {
            enabled: true,
            intensity: 0.1,
            speed: 0.5,
            characters: vec!["·", "˚", "°"],
            colors: vec!["DarkGray", "Gray"],
        },
    },
    ambient_lighting: AmbientLightingConfig {
        enabled: true,
        variation_speed: 0.02,
        variation_intensity: 0.15,
        base_level: 20,
    },
    atmospheric: AtmosphericConfig {
        heat_shimmer: true,
        dust_motes: true,
        light_rays: false,
    },
}
```

### Configuration Parameters

#### Weather Effects
- `enabled`: Whether the effect is active
- `intensity`: Spawn rate multiplier (0.0 - 1.0)
- `speed`: Movement speed multiplier
- `characters`: Array of character representations
- `colors`: Array of color names for variation

#### Ambient Lighting
- `variation_speed`: How fast lighting changes occur
- `variation_intensity`: Magnitude of lighting variations
- `base_level`: Base ambient light level

#### Atmospheric Effects
- `heat_shimmer`: Enable heat distortion effects
- `dust_motes`: Enable floating dust particles
- `light_rays`: Enable light ray effects (future feature)

## Implementation Details

### Particle Physics

Weather particles use simple physics simulation:

```rust
pub struct WeatherParticle {
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub character: char,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
}
```

### Noise-Based Movement

Dust particles use Perlin noise for natural movement:

```rust
let noise_x = self.perlin.get([x as f64 * 0.01, time as f64 * 0.1]) as f32;
let noise_y = self.perlin.get([y as f64 * 0.01, time as f64 * 0.1 + 100.0]) as f32;

particle.velocity_x = noise_x * config.speed * 0.5;
particle.velocity_y = noise_y * config.speed * 0.3;
```

### Rendering Integration

Procedural effects are rendered after particles but before UI elements:

```rust
// Render procedural effects
self.render_procedural_effects(&mut final_spans, cam_x, cam_y, view_width, view_height);
```

### Performance Optimizations

1. **Particle Culling**: Particles outside viewport are removed
2. **Lifetime Management**: Expired particles are automatically cleaned up
3. **Spawn Rate Control**: Configurable spawn intervals prevent performance issues
4. **Efficient Updates**: Only active effects are processed each frame

## Performance Considerations

### Computational Complexity

- **Weather Particles**: O(n) where n is number of active particles
- **Noise Calculations**: O(1) per particle using optimized Perlin noise
- **Rendering**: O(n) where n is number of visible particles
- **Memory Usage**: Minimal with automatic cleanup

### Performance Impact

- **CPU Usage**: Low impact with typical particle counts (10-50 particles)
- **Memory**: Efficient particle management with automatic cleanup
- **Rendering**: Integrated with existing rendering pipeline
- **Frame Rate**: Maintains 60 FPS target with default settings

### Optimization Strategies

1. **Viewport Culling**: Only process particles within view
2. **Batch Processing**: Update multiple particles efficiently
3. **Configurable Intensity**: Adjust particle count based on performance needs
4. **Efficient Noise**: Use optimized Perlin noise implementation

## Usage Examples

### Basic Weather Effects

```rust
// Enable rain
let mut config = ProceduralConfig::default();
config.weather.rain.enabled = true;
config.weather.rain.intensity = 0.5;

let mut effects = ProceduralEffects::new(config);
```

### Ambient Lighting Variations

```rust
// Get current lighting variation
let variation = effects.get_ambient_light_variation();
let adjusted_ambient = base_ambient + (variation * 10.0) as u8;
```

### Heat Shimmer Effects

```rust
// Get shimmer offset for position
let (offset_x, offset_y) = effects.get_heat_shimmer_offset(x, y);
let adjusted_x = x + offset_x as i32;
let adjusted_y = y + offset_y as i32;
```

## Integration with Game Systems

### Weather System Integration

The procedural effects can be controlled by game weather systems:

```rust
match current_weather {
    Weather::Rain => {
        effects.config.weather.rain.enabled = true;
        effects.config.weather.dust.enabled = false;
    },
    Weather::Snow => {
        effects.config.weather.snow.enabled = true;
        effects.config.weather.rain.enabled = false;
    },
    Weather::Clear => {
        effects.config.weather.dust.enabled = true;
        effects.config.weather.rain.enabled = false;
        effects.config.weather.snow.enabled = false;
    },
}
```

### Lighting System Integration

Ambient lighting variations can enhance the lighting system:

```rust
let base_ambient = lighting_config.ambient_level;
let variation = procedural_effects.get_ambient_light_variation();
let dynamic_ambient = (base_ambient as f32 + variation * 20.0).clamp(0.0, 255.0) as u8;
```

## Testing

The procedural effects system includes comprehensive testing:

- **DES Integration**: Procedural effects test scenario validates functionality
- **Unit Tests**: Individual effect components are tested
- **Performance Tests**: Particle count and performance impact validation

### Test Scenario

The `procedural_effects_test.json` scenario validates that procedural effects don't interfere with core gameplay mechanics.

## Future Enhancements

### Potential Additions

1. **Light Rays**: Volumetric lighting effects through dust/fog
2. **Wind Effects**: Directional particle movement based on wind
3. **Seasonal Variations**: Different effects based on game season
4. **Interactive Effects**: Player actions affecting environmental effects
5. **Sound Integration**: Audio cues synchronized with visual effects

### Advanced Features

1. **Fluid Simulation**: More realistic water and air movement
2. **Cellular Automata**: Fire, smoke, and other complex effects
3. **Fractal Patterns**: Complex atmospheric phenomena
4. **Dynamic Weather**: Procedural weather pattern generation

## Compatibility

- **Ratatui Integration**: Works seamlessly with terminal rendering
- **Performance**: Maintains target frame rate with default settings
- **Configuration**: Hot-reloadable through configuration updates
- **Modular Design**: Can be disabled without affecting other systems

## Troubleshooting

### Common Issues

1. **Performance Impact**: Reduce particle intensity or disable effects
2. **Visual Clutter**: Adjust particle characters and colors
3. **Inconsistent Effects**: Check noise seed consistency

### Debug Information

```rust
// Check active particle count
println!("Active particles: {}", effects.get_weather_particles().len());

// Monitor performance impact
let start = Instant::now();
effects.update(delta_time, width, height);
let duration = start.elapsed();
println!("Update time: {:?}", duration);
```

The procedural visual effects system adds dynamic, atmospheric elements to the TUI RPG while maintaining excellent performance and providing extensive customization options.
