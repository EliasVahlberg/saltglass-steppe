# Particle Effects System

## Overview
Implemented a comprehensive, data-driven particle effects system for enhanced visual appeal in the TUI RPG.

## Features

### Particle Types
- **Sparkles**: Twinkling effects with configurable colors and characters
- **Glow**: Pulsing brightness effects for ambient lighting
- **Float**: Upward-floating particles affected by gravity
- **Drift**: Wind-affected particles with fade effects
- **Pulse**: Size-changing particles with wave patterns
- **Shimmer**: Color-cycling particles for magical effects

### Data-Driven Configuration
- JSON-based particle configuration in `render_config.json`
- Configurable spawn rates, lifetimes, colors, and behaviors
- Per-effect-type customization (velocity, gravity, pulse speed, etc.)

### Performance Features
- Maximum particle limits to prevent performance issues
- Efficient particle lifecycle management
- Viewport culling for off-screen particles
- Optimized update and rendering loops

## Implementation Details

### Files Added
- `src/renderer/particles.rs`: Complete particle system implementation
- Particle configuration in `data/render_config.json`

### Files Modified
- `src/renderer/mod.rs`: Integrated particle system into main renderer
- `src/renderer/config.rs`: Added ParticleConfig support
- Test files: Updated to include particle configuration

### Key Components
- `Particle` struct: Individual particle with position, velocity, color, lifetime
- `ParticleSystem`: Manager for all particles with update and rendering
- `ParticleConfig`: Data-driven configuration for all effect types
- Rendering integration: Particles rendered on top of game world

## Usage
```rust
// Add particle effects programmatically
renderer.add_particle_effect(x, y, ParticleType::Sparkle);
renderer.add_particle_effect(x, y, ParticleType::Glow);

// Clear all particles
renderer.clear_particles();
```

## Configuration Example
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
  }
}
```

## Testing
- Added DES test scenario `particle_effects_test.json`
- Comprehensive unit tests for particle creation, updates, and lifecycle
- All existing scenarios continue to pass
- Performance tested with multiple particle types
