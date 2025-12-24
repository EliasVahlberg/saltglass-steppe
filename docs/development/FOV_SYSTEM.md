# Field of View (FOV) System Implementation

## Overview

The Field of View system implements shadow casting algorithm based on Ruggrogue's approach, providing tactical visibility mechanics that integrate with the existing lighting system.

## Key Features

### Shadow Casting Algorithm
- **Diamond-shaped walls**: Better visibility around corners compared to square walls
- **Symmetric/asymmetric visibility**: Center-to-center for floors, center-to-mid-line for walls
- **8-octant processing**: Efficient calculation covering all directions
- **Integer-only arithmetic**: Performance optimized without floating point operations

### Integration Points
- **Lighting system**: FOV works alongside existing light mechanics
- **Storm system**: Map changes trigger FOV recalculation
- **Movement**: FOV updates automatically on player movement
- **Debug system**: Compatible with god view and phase mode

## Technical Implementation

### Core Components

#### FieldOfView Struct
```rust
pub struct FieldOfView {
    pub visible_tiles: HashSet<(i32, i32)>,
    pub range: i32,
    pub dirty: bool,
}
```

#### Shadow Casting Process
1. **Octant division**: Split 360° view into 8 octants for uniform processing
2. **Column processing**: Work outward from player position column by column
3. **Sight tracking**: Maintain visible areas using slope-based sights
4. **Wall detection**: Handle runs of walls vs floors to create shadows

### Algorithm Details

#### Slope Representation
- Uses integer pairs (rise, run) to avoid floating point
- Enables precise comparison and calculation
- Supports exact visibility determination

#### Sight Management
- Even/odd sight lists for efficient memory usage
- Working slope tracks current floor runs
- Sight boundaries define visible areas per column

## Performance Characteristics

- **Time Complexity**: O(range²) for full FOV calculation
- **Memory Usage**: Minimal allocation, reuses sight lists
- **Integration Cost**: Single method call per turn/movement

## Usage Examples

### Basic FOV Calculation
```rust
let mut fov = FieldOfView::new(8);
fov.calculate(&map, (player_x, player_y));
if fov.is_visible((target_x, target_y)) {
    // Target is visible
}
```

### Integration with Game State
```rust
// Automatic updates
self.update_fov(); // Called on movement, turn end, map changes

// Check visibility
if self.player_fov.is_visible((x, y)) {
    // Tile is visible to player
}
```

## Creative Direction Alignment

### Pillar Support
- **Light is dangerous**: FOV integrates with lighting for tactical gameplay
- **Readable tactics**: Clear visibility rules for combat planning
- **TUI aesthetic**: Efficient ASCII-based visibility representation

### Storm Integration
- Map changes from storms trigger FOV recalculation
- Maintains tactical relevance as environment shifts
- Supports dynamic gameplay around changing visibility

## Testing and Validation

### DES Test Coverage
- Movement-based FOV updates
- Debug command compatibility
- Performance validation
- Range and visibility accuracy

### Quality Assurance
- Deterministic results for same inputs
- Proper integration with existing systems
- No performance regression from legacy FOV

## Future Enhancements

### Potential Additions
- **Enemy FOV**: Extend system to calculate monster vision
- **Light-based FOV**: Integrate more closely with lighting intensity
- **Adaptation effects**: FOV modifications from player adaptations
- **Environmental FOV**: Weather and storm effects on visibility

### Performance Optimizations
- **Caching**: Store FOV results for static positions
- **Incremental updates**: Only recalculate changed areas
- **Range optimization**: Dynamic range based on lighting conditions

## Implementation Status

✅ **Core Algorithm**: Shadow casting with diamond walls implemented  
✅ **Game Integration**: Seamless replacement of legacy FOV system  
✅ **Performance**: Optimized integer-only calculations  
✅ **Testing**: DES test coverage and validation  
✅ **Documentation**: Complete implementation guide  

The FOV system successfully enhances tactical gameplay while maintaining the game's performance and creative vision. The shadow casting algorithm provides precise, fair visibility mechanics that support the "light is dangerous" pillar through clear, readable tactical information.
