# Visual Variety and Vibrance Systems - Creative Director Guide

## Overview

The game now has enhanced systems to create visual variety and environmental vibrance. These systems are designed to make the world feel more alive and varied without adding mechanical complexity.

## Available Systems for Creating Variation

### 1. Terrain Variety System

**Purpose**: Create visually distinct environments through varied wall and floor types.

**How to Use**:
- **Wall Types**: Edit `data/walls.json` to add new wall materials
- **Floor Types**: Edit `data/floors.json` to add new floor surfaces
- **Biome Integration**: Configure `data/terrain_config.json` to control which terrain types appear in which biomes

**Creative Opportunities**:
- Desert biomes can use soft sand floors with sandstone walls
- Saltflat biomes can use salt crust floors with crystalline walls
- Ruins can use ancient tile floors with reinforced concrete walls
- Each biome can have a distinct visual identity

**Example Wall Types Available**:
- Sandstone (`#`, DarkYellow) - weathered desert stone
- Shale (`▓`, DarkGray) - layered sedimentary rock
- Salt Crystal (`◆`, White) - brittle crystalline formations
- Saltglass (`░`, Cyan) - fused glass and salt
- Reinforced Concrete (`█`, Gray) - pre-storm construction
- Rusted Steel (`¤`, Red) - corroded metal framework

**Example Floor Types Available**:
- Dry Soil (`.`, DarkYellow) - parched earth
- Crushed Saltglass (`·`, LightCyan) - glittering fragments
- Smooth Granite Glass (`▫`, LightGray) - polished stone-glass fusion
- Soft Sand (`:`, Yellow) - fine desert sand
- Salt Crust (`░`, White) - crystalline salt deposits
- Ancient Tile (`▪`, DarkGray) - pre-storm flooring

### 2. Biome-Specific Terrain Generation

**Purpose**: Make biomes feel distinctly different through terrain density and type.

**How to Use**:
- Configure `floor_threshold_bonus` to make biomes more/less open
- Set `floor_type_override` and `wall_type_override` for biome-specific materials
- Adjust `glass_density_multiplier` for hazard variation

**Creative Opportunities**:
- Desert biomes: Almost entirely soft sand floors (very open)
- Saltflats: Dense salt crystal walls with salt crust floors
- Oases: More open with dry soil floors
- Ruins: Ancient tiles and reinforced concrete structures

### 3. Structure Template System

**Purpose**: Place handcrafted structures for narrative and visual interest.

**How to Use**:
- Create templates in `data/structure_templates.json`
- Define symbol dictionaries mapping characters to terrain/NPCs
- Specify clearing areas and lore information

**Creative Opportunities**:
- Mesa villages with carved stone architecture
- Salt shrines with crystalline walls
- Ruined archives with pre-storm materials
- Each structure can tell a story through its materials and layout

**Current Templates Available**:
- `mesa_village` - Settlement with reinforced concrete and steel
- `salt_shrine` - Crystalline shrine with salt keeper NPC
- `ruined_archive` - Pre-storm data facility with maintenance drone

### 4. Color and Symbol Variety

**Purpose**: Create immediate visual distinction between terrain types.

**Available Colors**: Red, Green, Yellow, Blue, Magenta, Cyan, Gray, DarkGray, LightRed, LightGreen, LightYellow, LightBlue, LightMagenta, LightCyan, White, DarkYellow

**Symbol Guidelines**:
- `#` - Basic walls
- `▓` - Dense/layered materials (shale)
- `◆` - Crystalline materials (salt crystal)
- `░` - Translucent/refractive materials (saltglass, salt crust)
- `█` - Solid/industrial materials (concrete)
- `¤` - Metal materials (rusted steel)
- `.` - Basic floors
- `·` - Granular floors (crushed glass)
- `▫` - Smooth floors (granite glass)
- `:` - Loose floors (sand)
- `▪` - Constructed floors (ancient tile)

## Implementation Guidelines

### Adding New Wall Types

1. Add entry to `data/walls.json`
2. Choose appropriate glyph and color
3. Set HP based on material strength
4. Write evocative description

### Adding New Floor Types

1. Add entry to `data/floors.json`
2. Choose glyph that suggests texture
3. Select color that fits biome palette
4. Write atmospheric description

### Creating New Structure Templates

1. Design layout using ASCII art
2. Create symbol dictionary mapping characters to terrain/NPCs
3. Define clearing area if needed
4. Write discovery message and atmosphere text
5. Add to `data/structure_templates.json`

### Biome Configuration

1. Edit `data/terrain_config.json`
2. Set `floor_type` and `wall_type` for base terrain
3. Add biome modifiers for overrides
4. Adjust `floor_threshold_bonus` for openness
5. Set `glass_density_multiplier` for hazard levels

## Visual Design Principles

1. **Biome Identity**: Each biome should have a distinct color palette and material set
2. **Narrative Through Materials**: Use terrain types to tell environmental stories
3. **Gameplay Clarity**: Ensure visual variety doesn't compromise readability
4. **Atmospheric Consistency**: Materials should fit the post-apocalyptic glass storm setting
5. **Progressive Revelation**: Different areas should feel like discoveries

## Current Biome Themes

- **Desert**: Yellow/brown palette, sand and sandstone
- **Saltflat**: White/cyan palette, salt and crystal materials
- **Oasis**: Green/brown palette, natural materials with life
- **Ruins**: Gray palette, pre-storm industrial materials
- **Scrubland**: Mixed palette, weathered natural materials

## Next Steps for Enhanced Variety

1. Add more wall and floor types for specific narrative contexts
2. Create biome-specific structure templates
3. Develop material interaction systems (e.g., glass walls refracting light)
4. Add seasonal or storm-based material transformations
5. Create rare/special materials for unique locations

This system provides the foundation for rich environmental storytelling through visual variety while maintaining the game's core aesthetic and readability.
