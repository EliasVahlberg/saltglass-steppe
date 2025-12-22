# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased] - 2025-12-22

### Enhanced Visual Effects System

#### Added
- **New Effect Types**: Expanded from 2 to 8 visual effect types
  - `Pulse`: Rhythmic heartbeat-like effect for critical states
  - `Wave`: Spatial wave propagation across the map
  - `Shimmer`: Multi-color cycling based on position
  - `Rainbow`: Time-based color cycling
  - `Fade`: Slow fade in/out with longer cycles
  - `Drift`: Particle-like drifting effect

- **Enhanced Conditions**: Added 7 new effect trigger conditions
  - `adaptation_count_gte`: Trigger on adaptation count thresholds
  - `in_storm_eye`: Effects during active storms
  - `on_fragile_glass`: Position + health-based triggers
  - `psychic_active`: Status effect-based triggers
  - `high_salt_exposure`: Multi-adaptation exposure effects
  - `void_exposure`: Void-touched status effects

- **New Target Types**: Extended targeting system
  - `environment`: Map-wide atmospheric effects
  - `tile`: Specific tile-based warnings

- **22 New Visual Effects**: Comprehensive effect library
  - Critical health pulse effect
  - Storm particle drift atmosphere
  - Glass resonance for adapted players
  - Adaptation surge for highly mutated characters
  - Environmental storm warnings
  - Psychic resonance effects
  - Salt crystallization visuals
  - Void-touched corruption effects

#### Enhanced
- **Multi-Color Support**: Shimmer and Rainbow effects support multiple colors
- **Spatial Calculations**: Wave, Shimmer, and Drift use position-based algorithms
- **Performance Optimized**: Frame-based calculations with configurable speeds
- **Data-Driven**: All effects configurable via JSON without code changes

#### Technical Improvements
- **Expanded EffectContext**: Added 7 new context fields for complex conditions
- **Enhanced Parser**: Robust parsing for multi-parameter effect strings
- **Improved Rendering**: Position-aware effect calculations in game view
- **Better Documentation**: Comprehensive effect creation guide

#### Visual Enhancements
- **Atmospheric Effects**: Storm particles and environmental ambiance
- **Character Status**: More nuanced health and adaptation indicators
- **Enemy Identity**: Enhanced creature-specific visual effects
- **Interactive Feedback**: Glass resonance and tile-based effects

### Documentation
- **Updated Content Creation Guide**: Complete visual effects documentation
- **Effect Design Guidelines**: Best practices for thematic consistency
- **Testing Procedures**: Step-by-step effect testing workflow
- **Performance Notes**: Guidelines for optimal effect performance

### Files Modified
- `data/effects.json`: Expanded from 14 to 22 effects
- `src/game/effect.rs`: Enhanced parsing and condition system
- `src/ui/game_view.rs`: Improved rendering with spatial calculations
- `src/main.rs`: Extended effect context with new conditions
- `design_docs/CONTENT_CREATION_GUIDE.md`: Comprehensive effects documentation

### Breaking Changes
None - all changes are backward compatible with existing effects.

### Migration Notes
Existing effects continue to work unchanged. New effect types and conditions are opt-in.

---

## Previous Versions

*This changelog was started with the visual effects enhancement. Previous changes were not documented.*
