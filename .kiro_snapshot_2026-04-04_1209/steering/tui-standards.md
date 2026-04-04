# TUI Development Standards

## Ratatui Best Practices

### Layout Management

**Use Constraints, Not Hardcoded Sizes**:
```rust
✓ Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),      // HUD
        Constraint::Min(0),         // Map viewport
        Constraint::Length(10),     // Log
    ])

✗ Hardcoded pixel/char positions
```

### Rendering Performance

**Minimize Allocations**:
- Reuse `Vec<Span>` buffers
- Use `StatefulWidget` for complex components
- Cache computed layouts when possible

**Avoid Redundant Draws**:
- Only redraw on state changes
- Use dirty flags for partial updates
- Target 60fps minimum

### Color & Styling

**Use Theme System**:
```rust
// Define in renderer/themes.rs
pub struct Theme {
    pub floor: Style,
    pub wall: Style,
    pub danger: Style,
    // ...
}
```

**Accessibility**:
- Support multiple color schemes (light/dark/high-contrast)
- Don't rely solely on color (use glyphs + color)
- Test with colorblind-friendly palettes

### Text Wrapping

**Use textwrap for Long Text**:
```rust
use textwrap::wrap;

let wrapped = wrap(&description, width);
for line in wrapped {
    // Render line
}
```

## TUI-Specific Patterns

### Modal Overlays

**Stack-Based UI State**:
```rust
enum UIMode {
    Game,
    Inventory,
    Menu,
    Debug,
}

// Push/pop modes for overlays
```

### Input Handling

**Event-Driven, Non-Blocking**:
```rust
if crossterm::event::poll(Duration::from_millis(16))? {
    match crossterm::event::read()? {
        Event::Key(key) => handle_key(key),
        Event::Resize(w, h) => handle_resize(w, h),
        _ => {}
    }
}
```

**Key Bindings**:
- Vim keys (hjkl) + arrow keys for movement
- Single-key commands (i = inventory, q = quests)
- Escape to close menus
- `/` or `` ` `` for debug console

### Viewport & Camera

**Center on Player**:
```rust
let camera_x = player.x - (viewport_width / 2);
let camera_y = player.y - (viewport_height / 2);

// Clamp to map bounds
let camera_x = camera_x.max(0).min(map_width - viewport_width);
```

**Smooth Scrolling** (optional):
- Interpolate camera position over frames
- Only for non-gameplay-critical movement

## ASCII Art Guidelines

### Glyph Selection

**Distinct & Readable**:
```
Floor:  . (period)
Wall:   # (hash)
Glass:  ~ (tilde) or ≈ (almost equal)
Water:  ≈ (almost equal) or ∼ (tilde operator)
Player: @ (at sign)
Enemy:  Letters (g = goblin, D = dragon)
Item:   ! (exclamation) or % (percent)
Light:  * (asterisk) or ◦ (white bullet)
```

**Avoid Ambiguity**:
- Don't use similar glyphs for different things
- Test with different fonts (monospace required)

### Visual Effects

**Overlays**:
- Storm shimmer: Tint with `~` overlay
- Glare: `!` or `*` on affected tiles
- FOV: Dim unseen tiles (darker color)

**Animations**:
- Flicker: Alternate between 2 glyphs (1-2 frames)
- Pulse: Cycle through 3-4 brightness levels
- Particle: Move glyph across tiles over frames

## Log System

### Message Formatting

**Priority Levels**:
```rust
enum LogLevel {
    Debug,   // Gray, verbose
    Info,    // White, normal
    Warning, // Yellow, important
    Error,   // Red, critical
    Story,   // Cyan, narrative
}
```

**Message Structure**:
```
[Turn 42] You attack the mirage hound. (12 damage)
⚡ GLASS STORM! Intensity 3
The west wing *refracts*... corridors realign.
```

### Log Display

**Scrollable History**:
- Keep last 1000 messages
- Scroll with PgUp/PgDown
- Highlight recent messages (fade over time)

**Filtering** (optional):
- Toggle debug messages
- Show only combat/story/system

## Multi-Terminal System

### IPC Communication

**Unix Domain Sockets**:
- Main game creates socket
- Satellite terminals connect
- JSON-serialized messages

**Non-Blocking Updates**:
- Don't wait for satellite ACK
- Drop messages if buffer full
- Game continues if satellite disconnects

### Satellite Terminals

**Supported Types**:
- Log UI: Real-time game log
- Status UI: Player stats, buffs, debuffs
- Inventory UI: Live inventory display

**Auto-Spawn**:
- Detect terminal emulator (gnome-terminal, konsole, alacritty, kitty)
- Spawn with `--log-ui` / `--status-ui` flags
- Handle spawn failures gracefully

## Debug Tools

### Debug Console

**Commands**:
```
show tile    - God view (reveal all)
hide tile    - Normal FOV
sturdy       - Set HP to 9999
phase        - Toggle noclip
spawn <type> - Spawn entity
help         - List commands
```

**Access**:
- Press `/` or `` ` `` to open
- Type command + Enter
- Escape to close

### Mapgen Tool

**CLI for Testing**:
```bash
cargo run --bin mapgen-tool world [seed]
cargo run --bin mapgen-tool tile [seed] [poi]
```

**Output**:
- ASCII representation of generated map
- Stats (room count, connectivity, etc.)
- Determinism verification (same seed = same output)

## Performance Optimization

### Profiling

**Measure Before Optimizing**:
- Use `cargo flamegraph` for CPU profiling
- Track frame times (target <16ms for 60fps)
- Identify hot paths (FOV, rendering, pathfinding)

### Common Bottlenecks

**FOV Calculation**:
- Cache results when player doesn't move
- Use efficient algorithm (shadowcasting)
- Limit range (20-30 tiles)

**Rendering**:
- Only render visible tiles
- Batch draw calls
- Avoid string allocations in hot path

**Pathfinding**:
- Cache paths for NPCs
- Use A* with early exit
- Limit search depth

## Testing TUI Features

### Manual Testing

**Checklist**:
- [ ] Resize terminal (small, large, extreme)
- [ ] Test all menus (open, navigate, close)
- [ ] Verify colors in different themes
- [ ] Check text wrapping at various widths
- [ ] Test input responsiveness (no lag)

### Automated Testing

**DES Scenarios**:
- Test UI state transitions
- Verify log messages appear
- Check menu navigation

**Unit Tests**:
- Layout calculations
- Input parsing
- Color theme application

## Accessibility

### Terminal Compatibility

**Support Common Terminals**:
- gnome-terminal, konsole (Linux)
- Terminal.app, iTerm2 (macOS)
- Windows Terminal, ConEmu (Windows)
- alacritty, kitty (cross-platform)

**Fallbacks**:
- Detect terminal capabilities
- Degrade gracefully (no color → monochrome)
- Provide ASCII-only mode if Unicode fails

### Readability

**Font Requirements**:
- Monospace font required
- Recommend: Fira Code, JetBrains Mono, Cascadia Code
- Test with default terminal fonts

**Contrast**:
- Minimum 4.5:1 contrast ratio (WCAG AA)
- Avoid low-contrast color pairs
- Provide high-contrast theme

## Documentation

### UI Feature Docs

**Required Sections**:
- Purpose (why this UI exists)
- Layout (ASCII diagram of screen)
- Controls (key bindings)
- Implementation (code structure)

### Code Comments

**Annotate Complex Layouts**:
```rust
// Layout: [HUD (3 lines)] [Map (flexible)] [Log (10 lines)]
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),   // HUD
        Constraint::Min(0),      // Map
        Constraint::Length(10),  // Log
    ])
    .split(area);
```
