# Customizable Color Schemes (Theme System)

## Overview

The theme system provides customizable color schemes that allow users to switch between different visual styles for the TUI RPG. The system supports predefined themes and runtime theme switching without requiring application restart.

## Features

### Predefined Themes

1. **Classic Theme**
   - Traditional roguelike colors
   - Yellow player, red enemies, green NPCs
   - Gray walls and floors with cyan glass

2. **Dark Theme**
   - Dark mode with muted colors
   - Black backgrounds with light foreground colors
   - Reduced eye strain for extended play sessions

3. **High Contrast Theme**
   - Accessibility-focused high contrast colors
   - Black and white primary colors
   - Enhanced visibility for users with visual impairments

4. **Neon Theme**
   - Bright cyberpunk-inspired colors
   - Cyan, magenta, and bright accent colors
   - Futuristic aesthetic with black backgrounds

### Technical Architecture

#### Core Components

- **ThemeManager**: Manages theme loading, switching, and persistence
- **ThemeConfig**: Configuration structure for all available themes
- **Theme**: Individual theme definition with colors and metadata
- **ThemeColors**: Structured color definitions for different game elements

#### Integration Points

- **Renderer Integration**: Themes modify the render config at runtime
- **Configuration System**: Themes are loaded from JSON configuration files
- **Runtime Switching**: Theme changes are applied immediately without restart

## Configuration

Themes are configured in `data/themes.json`:

```json
{
  "themes": {
    "classic": {
      "name": "Classic",
      "description": "Traditional roguelike colors",
      "colors": {
        "entities": {
          "player": "Yellow",
          "enemies": "Red",
          "npcs": "Green",
          "items": "LightMagenta"
        },
        "tiles": {
          "floor": "DarkGray",
          "wall": "Gray",
          "glass": "Cyan",
          "stairs": "Yellow",
          "world_exit": "Green"
        },
        "lighting": {
          "torch": "Yellow",
          "ambient": "DarkGray"
        },
        "ui": {
          "revealed_tile": "DarkGray",
          "look_cursor_bg": "White",
          "look_cursor_fg": "Black",
          "hit_flash_bg": "Red",
          "hit_flash_fg": "White"
        }
      }
    }
  },
  "active_theme": "classic"
}
```

### Color Categories

#### Entity Colors
- `player`: Player character color
- `enemies`: Default enemy color
- `npcs`: Non-player character color
- `items`: Item pickup color

#### Tile Colors
- `floor`: Floor tile color
- `wall`: Wall tile color
- `glass`: Glass shard color
- `stairs`: Staircase color
- `world_exit`: World exit portal color

#### Lighting Colors
- `torch`: Torch light source color
- `ambient`: Ambient lighting color

#### UI Colors
- `revealed_tile`: Previously explored tile color
- `look_cursor_bg`: Look cursor background color
- `look_cursor_fg`: Look cursor foreground color
- `hit_flash_bg`: Hit flash background color
- `hit_flash_fg`: Hit flash foreground color

## Usage

### Runtime Theme Switching

```rust
// Set active theme
let success = renderer.set_theme("dark");

// Get list of available themes
let themes = renderer.list_themes();

// Get current active theme
let current = renderer.get_active_theme();
```

### Theme Management

```rust
// Load themes from file
let theme_manager = ThemeManager::load_from_file("data/themes.json")?;

// Save themes to file
theme_manager.save_to_file("data/themes.json")?;

// Get specific theme
let theme = theme_manager.get_theme("neon");
```

## Implementation Details

### Theme Application Process

1. **Theme Selection**: User selects a theme by name
2. **Validation**: System validates theme exists
3. **Color Extraction**: Theme colors are extracted and cloned
4. **Config Update**: Render config is updated with new colors
5. **Immediate Effect**: Changes are applied on next render frame

### Color Mapping

The theme system maps simplified color names to the full render configuration:

```rust
fn apply_theme_to_config(&mut self, theme_colors: &ThemeColors) {
    // Entity colors
    self.config.colors.entities.player.base = theme_colors.entities.player.clone();
    
    // Tile colors
    self.config.colors.tiles.floor = theme_colors.tiles.floor.clone();
    self.config.colors.tiles.wall = theme_colors.tiles.wall.clone();
    
    // UI colors
    self.config.colors.ui.look_cursor.bg = theme_colors.ui.look_cursor_bg.clone();
}
```

### File Structure

```
data/
├── themes.json          # Theme definitions and active theme
└── render_config.json   # Base rendering configuration
```

## Performance Considerations

### Optimizations

- **Lazy Loading**: Themes are loaded once at startup
- **Clone Optimization**: Only necessary color data is cloned during switching
- **Immediate Application**: No restart required for theme changes
- **Memory Efficient**: Themes share common structure definitions

### Performance Impact

- **Startup**: Minimal overhead loading theme definitions
- **Runtime**: Theme switching is O(1) operation
- **Memory**: Low memory footprint with shared theme data
- **Rendering**: No performance impact on rendering pipeline

## Extensibility

### Adding New Themes

1. **Define Theme**: Add new theme to `themes.json`
2. **Color Specification**: Define all required color categories
3. **Metadata**: Include name and description
4. **Testing**: Verify theme works across all game elements

### Custom Color Categories

The system can be extended to support additional color categories:

```rust
// Add new color category to ThemeColors
pub struct ThemeColors {
    pub entities: ThemeEntityColors,
    pub tiles: ThemeTileColors,
    pub lighting: ThemeLightingColors,
    pub ui: ThemeUiColors,
    pub effects: ThemeEffectColors,  // New category
}
```

## Accessibility Features

### High Contrast Theme

- **Black and White**: Primary colors for maximum contrast
- **Yellow Accents**: High visibility accent colors
- **Clear Distinctions**: Distinct colors for different element types

### Color Blind Support

Future enhancements could include:
- **Deuteranopia Theme**: Red-green color blind friendly
- **Protanopia Theme**: Alternative red-green color blind support
- **Tritanopia Theme**: Blue-yellow color blind friendly

## Testing

The theme system includes comprehensive testing:

- **DES Integration**: Theme test scenario validates functionality
- **Unit Tests**: Individual theme components are tested
- **Integration Tests**: Full rendering pipeline with themes

### Test Scenario

The `theme_system_test.json` scenario validates that theme switching doesn't interfere with core gameplay mechanics.

## Future Enhancements

### Potential Additions

1. **User-Defined Themes**: Allow users to create custom themes
2. **Theme Editor**: In-game theme customization interface
3. **Dynamic Themes**: Time-based or context-sensitive themes
4. **Theme Inheritance**: Base themes with variations
5. **Export/Import**: Share themes between users

### Integration Opportunities

- **Settings Menu**: In-game theme selection interface
- **Keybind Support**: Quick theme switching hotkeys
- **Profile Integration**: Per-user theme preferences
- **Mod Support**: Community-created theme packs

## Compatibility

- **Ratatui Integration**: Works with all ratatui color constants
- **Configuration System**: Integrates with existing JSON config system
- **Modular Design**: Can be disabled without affecting other systems
- **Backward Compatibility**: Graceful fallback to default colors

## Troubleshooting

### Common Issues

1. **Theme Not Loading**: Check `themes.json` file exists and is valid JSON
2. **Colors Not Changing**: Verify theme name matches exactly
3. **Missing Colors**: Ensure all required color categories are defined

### Debug Information

```rust
// Check available themes
println!("Available themes: {:?}", renderer.list_themes());

// Check current theme
println!("Current theme: {}", renderer.get_active_theme());
```

### File Validation

The theme system validates JSON structure on load and provides helpful error messages for malformed theme files.
