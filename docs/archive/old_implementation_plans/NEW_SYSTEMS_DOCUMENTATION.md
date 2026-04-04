# New Gameplay Systems Documentation

This document describes the three new gameplay systems implemented for Saltglass Steppe: Light Manipulation, Void Energy, and Crystal Resonance.

## Light Manipulation System

### Overview
The Light Manipulation System enables players to create, control, and interact with light beams and refraction surfaces. This system is particularly relevant in Refraction Fields and Glass Gardens biomes.

### Core Components

#### Light Beams
- **Direction**: 8-directional movement (N, S, E, W, NE, NW, SE, SW)
- **Intensity**: 1-10, affects damage and range
- **Color**: Different colors have different properties:
  - White: Standard light (1.0x damage)
  - Red: Heat/damage (1.5x damage)
  - Blue: Cold/slowing (0.8x damage)
  - Green: Healing properties
  - Yellow: Revealing/detection
  - Violet: Psychic enhancement
- **Range**: How far the beam travels before dissipating

#### Light Sources
- **Position**: Fixed location light emitters
- **Intensity**: Brightness level (1-10)
- **Color**: Determines light properties
- **Active State**: Can be turned on/off

#### Refraction Surfaces
- **Position**: Location of the refracting surface
- **Angle**: Direction of refraction (0-7, representing 8 directions)
- **Efficiency**: How much light is refracted vs absorbed (0.0-1.0)

### Player Abilities

#### Focus Beam (Cost: 10 Light Energy)
Creates a directed light beam from the player's position.
```
Debug Command: focus_beam <direction>
Example: focus_beam e (creates eastward beam)
```

#### Create Prism (Cost: 20 Light Energy)
Places a refraction surface that bends light beams.
```
Debug Command: create_prism <x> <y>
Example: create_prism 12 10
```

#### Absorb Light
Gains light energy from nearby sources and beams.
```
Debug Command: absorb_light
```

### Mechanics

#### Beam Tracing
Light beams follow these rules:
1. Travel in straight lines until hitting an obstacle
2. Stop when hitting walls
3. Refract when hitting refraction surfaces
4. Have limited range based on intensity

#### Light Energy
- Players accumulate light energy by absorbing from sources
- Energy is spent on light manipulation abilities
- Energy regenerates slowly over time in bright areas

### Integration Points
- **Combat**: Light beams can damage enemies
- **Puzzles**: Refraction surfaces can redirect beams to activate mechanisms
- **Exploration**: Light sources reveal hidden areas
- **Biome Effects**: Enhanced in Refraction Fields and Glass Gardens

---

## Void Energy System

### Overview
The Void Energy System tracks the player's exposure to void corruption and unlocks reality-bending abilities. This system is central to Shattered Citadel content and void-touched areas.

### Core Components

#### Void Exposure
Exposure levels determine available abilities and side effects:
- **None** (0-10): No effects
- **Minimal** (11-25): 5% reality distortion chance
- **Moderate** (26-50): 15% reality distortion chance
- **High** (51-75): 30% reality distortion chance
- **Extreme** (76-100): 50% reality distortion chance

#### Void Abilities
Unlocked progressively based on exposure level:

1. **Void Step** (Req: 20 exposure, Cost: 15 energy)
   - Short-range teleportation (max 5 tiles)
   - Creates spatial distortion at departure point

2. **Void Drain** (Req: 15 exposure, Cost: 10 energy)
   - Drains energy from enemies
   - Creates psychic distortion at target

3. **Void Shield** (Req: 30 exposure, Cost: 20 energy)
   - Absorbs incoming damage using void energy

4. **Reality Rend** (Req: 40 exposure, Cost: 25 energy)
   - Armor-ignoring damage attack
   - Creates material distortion at target

5. **Phase Walk** (Req: 60 exposure, Cost: 30 energy)
   - Walk through walls for 3 turns
   - Highest-tier ability

#### Reality Distortions
Spontaneous effects that occur based on exposure level:
- **Temporal**: Time flows differently
- **Spatial**: Space is warped
- **Material**: Matter becomes unstable
- **Psychic**: Mental effects

### Player Abilities

#### Void Step
```
Debug Command: void_step <x> <y>
Example: void_step 15 12
```

#### Reality Rend
```
Debug Command: reality_rend <x> <y>
Example: reality_rend 11 10
```

### Mechanics

#### Exposure Accumulation
- Gained by spending time in void-corrupted areas
- Increases maximum void energy capacity
- Unlocks new abilities at thresholds
- Cannot be reduced (permanent corruption)

#### Void Energy
- Resource used for void abilities
- Maximum increases with exposure (50 + exposure/2)
- Regenerates slowly at high exposure levels
- Gained by draining enemies or void sources

#### Reality Distortions
- Random effects triggered by high exposure
- Create temporary hazardous areas
- Can affect movement, damage, or perception
- Duration and intensity based on exposure level

### Integration Points
- **Combat**: Void abilities provide unique tactical options
- **Exploration**: Phase walk enables access to new areas
- **Story**: Exposure level affects NPC reactions
- **Risk/Reward**: Power comes at cost of increasing instability

---

## Crystal Resonance System

### Overview
The Crystal Resonance System allows players to interact with crystalline formations, attune to different frequencies, and create harmonic effects. This system is most prominent in Glass Gardens biomes.

### Core Components

#### Crystal Frequencies
Five distinct frequency types with different properties:

1. **Alpha** (1-3 Hz): Deep resonance, structural effects (Base Power: 5)
2. **Beta** (4-7 Hz): Mental clarity, psychic enhancement (Base Power: 3)
3. **Gamma** (8-12 Hz): Energy flow, healing (Base Power: 4)
4. **Delta** (13-30 Hz): High energy, combat enhancement (Base Power: 6)
5. **Epsilon** (31+ Hz): Chaotic, unpredictable effects (Base Power: 8)

#### Crystal Formations
- **Position**: Fixed location in the world
- **Frequency**: One of the five types
- **Size**: 1-10, affects resonance range and power
- **Stability**: 1-100, determines lifespan
- **Growth Stage**: 0-5, crystals can grow over time

#### Harmonic Effects
Created when multiple crystals resonate together:
- **Healing**: Gamma + Beta frequencies
- **Enhancement**: Delta + Alpha frequencies
- **Psychic**: Beta + Epsilon frequencies
- **Structural**: Alpha + Gamma frequencies
- **Chaotic**: Epsilon + any other frequency

### Player Abilities

#### Create Crystal Seed (Cost: 20 Resonance Energy)
Plants a new crystal formation.
```
Debug Command: create_crystal <x> <y> <frequency>
Example: create_crystal 12 10 gamma
```

#### Resonate
Attunes to nearby crystals and gains energy.
```
Debug Command: resonate
```

#### Harmonize (Cost: 40 Resonance Energy)
Creates harmonic effects between nearby crystals.
```
Debug Command: harmonize
```

#### Shatter Crystals (Cost: Variable)
Destroys crystals in an area for tactical purposes.

### Mechanics

#### Frequency Attunement
- Players gain attunement (0-100) to each frequency
- Higher attunement increases energy gain from that frequency
- Attunement gained by resonating with crystals
- Affects effectiveness of crystal-based abilities

#### Crystal Growth
- Crystals can grow over time (5% chance per turn)
- Growth increases power and range
- Reduces stability (growing crystals are more fragile)
- Can be triggered by player abilities

#### Resonance Energy
- Resource used for crystal abilities
- Gained by resonating with crystals
- Maximum capacity: 100 (can be increased)
- Regenerates slowly over time

#### Harmonic Resonance
- Created when multiple crystals interact
- Effects depend on frequency combinations
- Duration based on total power involved
- Can provide beneficial or chaotic effects

### Integration Points
- **Combat**: Crystal effects can enhance or hinder combat
- **Healing**: Gamma crystals provide healing opportunities
- **Exploration**: Crystal formations mark important locations
- **Puzzles**: Harmonic effects can activate mechanisms
- **Character Development**: Frequency attunement provides specialization

---

## System Integration

### Cross-System Interactions

#### Light + Crystal
- Light beams can charge crystal formations
- Crystals can refract light beams
- Prismatic crystals create rainbow light effects

#### Void + Crystal
- Void energy can corrupt crystal frequencies
- Epsilon crystals are naturally void-touched
- Reality distortions can affect crystal stability

#### Light + Void
- Light can partially counteract void corruption
- Void abilities can manipulate light sources
- Phase walking affects light interaction

### Biome Synergies

#### Refraction Fields
- Enhanced light manipulation effects
- Natural refraction surfaces
- Light-based hazards and opportunities

#### Glass Gardens
- Abundant crystal formations
- Natural harmonic resonance
- Crystal growth accelerated

#### Shattered Citadel
- High void corruption
- Reality distortions common
- Void abilities more powerful

### Debug Commands Summary

#### Light System
- `focus_beam <direction>` - Create light beam
- `create_prism <x> <y>` - Create refraction surface
- `add_light_energy [amount]` - Add light energy
- `absorb_light` - Absorb nearby light

#### Void System
- `add_void_exposure [amount]` - Increase void exposure
- `add_void_energy [amount]` - Add void energy
- `void_step <x> <y>` - Teleport through void
- `reality_rend <x> <y>` - Void damage attack

#### Crystal System
- `create_crystal <x> <y> <frequency>` - Create crystal
- `resonate` - Resonate with nearby crystals
- `add_resonance_energy [amount]` - Add resonance energy
- `harmonize` - Create harmonic effect

### Testing

All three systems include comprehensive DES (Debug Execution System) tests:
- `light_manipulation_basic.json`
- `void_energy_basic.json`
- `crystal_resonance_basic.json`

Tests can be run with:
```bash
cargo test --test des_scenarios <test_name>
```

### Future Enhancements

#### Planned Features
1. **Light Puzzles**: Complex refraction-based challenges
2. **Void Corruption Spread**: Environmental void effects
3. **Crystal Networks**: Connected crystal systems
4. **Cross-System Abilities**: Abilities that use multiple systems
5. **Advanced Harmonics**: More complex frequency interactions

#### Balance Considerations
- Energy costs may need adjustment based on gameplay testing
- Ability unlock thresholds should match progression curve
- Cross-system interactions need careful balance to avoid exploitation
- Biome-specific bonuses should feel meaningful but not mandatory

---

This documentation provides a comprehensive overview of the three new systems and their integration into Saltglass Steppe's gameplay mechanics.
