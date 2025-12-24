# Effects Configuration System

## Overview

The effects configuration system provides a unified, JSON-based configuration management for all visual effects in the TUI RPG. It consolidates particles, animations, themes, and procedural effects into a single, cohesive configuration system with performance and quality presets.

## Features

### Unified Configuration

- **Single Configuration File**: All visual effects managed through `data/effects_config.json`
- **Centralized Management**: One system to control particles, animations, themes, and procedural effects
- **Performance Presets**: Low, Balanced, and High performance modes
- **Quality Levels**: Minimal, Standard, and Enhanced quality settings

### Configuration Categories

1. **Particles Configuration**
   - Sparkles, glow, float, drift, pulse, and shimmer effects
   - Individual enable/disable controls
   - Intensity and performance parameters

2. **Animations Configuration**
   - Blink, glow, and screen shake animations
   - Timing and visual parameters
   - Color and intensity settings

3. **Themes Configuration**
   - Multiple color schemes and active theme selection
   - Runtime theme switching capabilities
   - Comprehensive color category coverage

4. **Procedural Effects Configuration**
   - Weather particles (rain, snow, dust)
   - Atmospheric effects (heat shimmer, dust motes)
   - Ambient lighting variations

5. **Global Effects Configuration**
   - Master enable/disable switch
   - Performance mode selection
   - Quality level settings
   - Maximum effects limits

## Configuration Structure

```json
{
  "particles": {
    "sparkles": {
      "enabled": true,
      "spawn_rate": 2.0,
      "lifetime": 1.0,
      "colors": ["White", "Yellow", "Cyan"],
      "characters": ["*", "✦", "✧", "◆"],
      "intensity": 1.0
    }
  },
  "animations": {
    "blink": {
      "duration_ms": 500,
      "on_color": "White",
      "off_color": "DarkGray"
    }
  },
  "themes": {
    "active_theme": "classic"
  },
  "procedural": {
    "weather": {
      "dust": {
        "enabled": true,
        "intensity": 0.1
      }
    }
  },
  "global": {
    "enabled": true,
    "performance_mode": "Balanced",
    "quality_level": "Standard",
    "max_effects": 100
  }
}
```

## Performance Modes

### Low Performance Mode
- **Target**: Older hardware or performance-critical scenarios
- **Particle Intensity**: Reduced to 50% of standard
- **Procedural Effects**: Minimal dust particles
- **Max Effects**: Limited to 50 concurrent effects

### Balanced Performance Mode (Default)
- **Target**: Standard hardware and typical usage
- **Particle Intensity**: Standard levels
- **Procedural Effects**: Normal dust and atmospheric effects
- **Max Effects**: Up to 100 concurrent effects

### High Performance Mode
- **Target**: Modern hardware with performance headroom
- **Particle Intensity**: Enhanced to 150% of standard
- **Procedural Effects**: Increased particle density
- **Max Effects**: Up to 200 concurrent effects

## Quality Levels

### Minimal Quality
- **Target**: Maximum performance, minimal visual effects
- **Particles**: Sparkles disabled
- **Atmospheric**: Heat shimmer disabled
- **Focus**: Core gameplay with minimal visual overhead

### Standard Quality (Default)
- **Target**: Balanced visual quality and performance
- **Particles**: All standard effects enabled
- **Atmospheric**: Heat shimmer and basic atmospheric effects
- **Focus**: Good visual experience with reasonable performance

### Enhanced Quality
- **Target**: Maximum visual fidelity
- **Particles**: All effects enabled with enhanced settings
- **Atmospheric**: All atmospheric effects including dust motes
- **Focus**: Best visual experience for capable hardware

## Implementation Details

### EffectsManager Class

```rust
pub struct EffectsManager {
    config: EffectsConfig,
}

impl EffectsManager {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>>;
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;
    pub fn set_performance_mode(&mut self, mode: PerformanceMode);
    pub fn set_quality_level(&mut self, level: QualityLevel);
}
```

### Automatic Configuration Application

When performance mode or quality level changes, the system automatically adjusts:

```rust
fn apply_performance_settings(&mut self) {
    match self.config.global.performance_mode {
        PerformanceMode::Low => {
            self.config.particles.sparkles.intensity = 0.5;
            self.config.global.max_effects = 50;
        }
        // ... other modes
    }
}
```

### Integration with Renderer

The effects manager integrates seamlessly with the existing renderer:

```rust
// Load effects manager
let effects_manager = EffectsManager::load_from_file("data/effects_config.json")?;

// Use configuration for subsystems
let procedural_effects = ProceduralEffects::new(effects_manager.get_config().procedural.clone());
```

## Usage Examples

### Runtime Configuration Changes

```rust
// Change performance mode
renderer.set_effects_performance_mode(PerformanceMode::High);

// Change quality level
renderer.set_effects_quality_level(QualityLevel::Enhanced);

// Save configuration
renderer.save_effects_config()?;
```

### Loading Custom Configuration

```rust
// Load from custom file
let effects_manager = EffectsManager::load_from_file("custom_effects.json")?;

// Apply to renderer
let renderer = Renderer::with_effects_manager(effects_manager)?;
```

## Configuration Management

### File Structure

```
data/
├── effects_config.json    # Unified effects configuration
├── themes.json           # Legacy theme configuration (still supported)
└── render_config.json    # Base rendering configuration
```

### Backward Compatibility

The system maintains backward compatibility with existing configuration files:
- Individual effect configurations are still supported
- Legacy theme files continue to work
- Gradual migration path for existing configurations

### Hot Reloading

Configuration changes can be applied at runtime:
- Performance mode changes take effect immediately
- Quality level adjustments update active effects
- No application restart required

## Performance Considerations

### Configuration Impact

- **Load Time**: Minimal overhead loading unified configuration
- **Memory Usage**: Consolidated configuration reduces memory fragmentation
- **Runtime Changes**: Efficient application of performance/quality changes
- **File I/O**: Single file reduces disk access overhead

### Optimization Features

1. **Lazy Loading**: Effects are only initialized when enabled
2. **Dynamic Adjustment**: Performance mode changes adjust active effects
3. **Memory Management**: Quality levels control memory allocation
4. **Batch Updates**: Configuration changes applied in batches

## Testing

The effects configuration system includes comprehensive testing:

- **DES Integration**: Effects config test scenario validates functionality
- **Unit Tests**: Individual configuration components tested
- **Integration Tests**: Full system integration with all effect types

### Test Coverage

- Configuration loading and saving
- Performance mode transitions
- Quality level adjustments
- Backward compatibility validation

## Future Enhancements

### Planned Features

1. **User Profiles**: Per-user effect preferences
2. **Dynamic Adjustment**: Automatic performance scaling based on system load
3. **Effect Presets**: Predefined effect combinations for different scenarios
4. **Configuration Validation**: Schema validation for configuration files
5. **Migration Tools**: Automated migration from legacy configurations

### Extension Points

The system is designed for easy extension:
- Add new effect categories
- Implement custom performance modes
- Create specialized quality presets
- Integrate with external configuration systems

## Troubleshooting

### Common Issues

1. **Configuration Not Loading**: Check file path and JSON syntax
2. **Performance Issues**: Try lower performance mode or quality level
3. **Effects Not Applying**: Verify global effects are enabled

### Debug Information

```rust
// Check configuration status
println!("Effects enabled: {}", effects_manager.get_config().global.enabled);
println!("Performance mode: {:?}", effects_manager.get_config().global.performance_mode);
println!("Quality level: {:?}", effects_manager.get_config().global.quality_level);
```

### Configuration Validation

The system provides helpful error messages for invalid configurations:
- Missing required fields
- Invalid enum values
- Out-of-range numeric parameters

## Best Practices

### Configuration Management

1. **Version Control**: Include effects configuration in version control
2. **Environment-Specific**: Use different configurations for development/production
3. **Documentation**: Document custom configuration changes
4. **Testing**: Test configuration changes across different scenarios

### Performance Optimization

1. **Profile First**: Use performance mode appropriate for target hardware
2. **Quality Balance**: Choose quality level that balances visuals and performance
3. **Monitor Impact**: Watch for performance degradation with enhanced settings
4. **User Choice**: Allow users to adjust settings based on their preferences

The effects configuration system provides a powerful, flexible foundation for managing all visual effects while maintaining excellent performance and user experience.
