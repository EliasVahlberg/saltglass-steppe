---
status: stale
last_verified: 2026-04-04
commit: e0d1fe7
stale_reason: "Partial migration status still accurate but 135 hardcoded matches remain"
---

> ⚠️ **STALE DOCUMENT** — This document may not accurately reflect the current codebase.
> Reason: Partial migration status still accurate but 135 hardcoded matches remain
> Last verified: 2026-04-04

# Keyboard Config Migration TODO

This document tracks hardcoded `KeyCode::` matches that need to be migrated to use the keyboard config system (`data/keyboard_config.json` and `src/game/keyboard_config.rs`).

## Status

**Created:** 2026-02-21  
**Config System:** ✅ Implemented  
**Migration:** ⚠️ Partial (worldmap only)

## Files with Hardcoded KeyCode Matches

### src/ui/input.rs (128 occurrences)

**Worldmap Input** (✅ Migrated):
- Lines 817-870: Uses `CONFIG.matches_worldmap()` for inspect_toggle, set_target, auto_move

**Still Hardcoded:**

1. **Pause Menu** (lines ~482-530):
   - `KeyCode::Esc` - Return to main menu
   - `KeyCode::Up/Down` - Navigate
   - `KeyCode::Enter` - Select
   - `KeyCode::Char(' ')` - Confirm dialog

2. **Book Reader** (lines ~626-630):
   - `KeyCode::Esc | KeyCode::Char('q')` - Close
   - `KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ')` - Next page
   - `KeyCode::Left | KeyCode::Char('h')` - Previous page

3. **Wiki Menu** (lines ~647-651):
   - `KeyCode::Esc | KeyCode::Char('w')` - Close
   - `KeyCode::Tab | KeyCode::Char('l')` - Next tab
   - `KeyCode::BackTab | KeyCode::Char('h')` - Previous tab
   - `KeyCode::Char('j') | KeyCode::Down` - Navigate down
   - `KeyCode::Char('k') | KeyCode::Up` - Navigate up

4. **Quest Log** (lines ~660-662):
   - `KeyCode::Esc | KeyCode::Char('q')` - Close
   - `KeyCode::Char('j') | KeyCode::Down` - Navigate down
   - `KeyCode::Char('k') | KeyCode::Up` - Navigate up

5. **Crafting Menu** (lines ~670-673):
   - `KeyCode::Esc | KeyCode::Char('c')` - Close
   - `KeyCode::Char('j') | KeyCode::Down` - Navigate down
   - `KeyCode::Char('k') | KeyCode::Up` - Navigate up
   - `KeyCode::Enter` - Craft

6. **Inventory Menu** (lines ~681-702):
   - `KeyCode::Esc | KeyCode::Char('i')` - Close
   - `KeyCode::Char('j') | KeyCode::Down` - Navigate down
   - `KeyCode::Char('k') | KeyCode::Up` - Navigate up
   - `KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left | KeyCode::Right` - Switch tabs
   - `KeyCode::Char('x')` - Examine item
   - `KeyCode::Char('u')` - Use item
   - `KeyCode::Enter` - Equip/unequip

7. **Look Mode** (lines ~719+):
   - `KeyCode::Esc | KeyCode::Enter` - Exit look mode
   - Arrow keys and hjkl for cursor movement

8. **Chest UI** (lines ~730+):
   - Similar to inventory (navigate, transfer, close)

9. **Trade Menu** (lines ~760+):
   - Navigate, buy, sell, close

10. **Psychic Menu** (lines ~790+):
    - Navigate, select ability, close

11. **Faction Menu** (lines ~1110+):
    - Navigate, close

12. **Void Menu, Crystal Menu, Light Menu, Skills Menu** (various lines):
    - All use hardcoded navigation and close keys

13. **Debug Console** (lines ~1000+):
    - `KeyCode::Esc` - Close
    - `KeyCode::Enter` - Execute command
    - `KeyCode::Backspace` - Delete character
    - Character input

14. **Main Game Input** (lines ~860+):
    - `KeyCode::Char('`')` - Debug console
    - `KeyCode::F(12)` - Debug menu
    - `KeyCode::Char('S')` - Save
    - `KeyCode::Char('L')` - Load
    - `KeyCode::Char('x')` - Look mode
    - `KeyCode::Char('X')` - Examine
    - `KeyCode::Char('e')` - Wait
    - `KeyCode::Char('E')` - Interact
    - `KeyCode::Char('o')` - Auto-explore
    - `KeyCode::Char('i')` - Inventory
    - `KeyCode::Char('q')` - Quest log
    - `KeyCode::Char('c')` - Crafting
    - `KeyCode::Char('C')` - Open chest
    - `KeyCode::Char('w')` - Wiki
    - `KeyCode::Char('m')` - World map
    - `KeyCode::Char('p')` - Psychic menu
    - `KeyCode::Char('f')` - Faction menu
    - `KeyCode::Char('v')` - Void menu
    - `KeyCode::Char('r')` - Crystal menu
    - `KeyCode::Char('t')` - Light menu
    - `KeyCode::Char('k')` - Skills menu
    - `KeyCode::Char('>')` - Use stairs
    - Arrow keys and hjkl/yubn for movement
    - `KeyCode::Char('1-9')` - Use items

### src/ui/menu.rs (12 occurrences)

**Main Menu**:
- `KeyCode::Esc` - Back/quit
- `KeyCode::Up | KeyCode::Char('k')` - Navigate up
- `KeyCode::Down | KeyCode::Char('j')` - Navigate down
- `KeyCode::Enter` - Select
- `KeyCode::Backspace` - Delete character (seed input)
- `KeyCode::Char(c) if c.is_ascii_digit()` - Seed input

**Pause Menu**:
- Same navigation keys

### src/satellite.rs (13 occurrences)

**Satellite UI** (separate terminal windows):
- Various navigation and control keys
- Lower priority for migration

## Migration Strategy

### Phase 1: Core Gameplay (High Priority)
- [ ] Main game input (movement, actions, menus)
- [ ] Look mode
- [ ] Debug console

### Phase 2: Menus (Medium Priority)
- [ ] Inventory menu
- [ ] Crafting menu
- [ ] Quest log
- [ ] Wiki menu
- [ ] Book reader

### Phase 3: Special Menus (Medium Priority)
- [ ] Psychic menu
- [ ] Faction menu
- [ ] Void menu
- [ ] Crystal menu
- [ ] Light menu
- [ ] Skills menu

### Phase 4: UI Chrome (Low Priority)
- [ ] Chest UI
- [ ] Trade menu
- [ ] Pause menu
- [ ] Main menu
- [ ] Satellite UI

## Implementation Notes

### Current Config Structure

```json
{
  "gameplay": { ... },
  "worldmap": { ... },
  "menus": {
    "navigate_up": "Up",
    "navigate_down": "Down",
    "navigate_left": "Left",
    "navigate_right": "Right",
    "select": "Enter",
    "back": "Esc",
    "next_tab": "Tab",
    "prev_tab": "BackTab"
  },
  "debug": { ... }
}
```

### Needed Additions

Add to `keyboard_config.json`:
- `menus.close` (for menu-specific close keys like 'i' for inventory)
- `menus.examine` (for 'x' in inventory)
- `menus.use_item` (for 'u' in inventory)
- `menus.transfer` (for chest UI)
- `menus.buy` (for trade menu)
- `menus.sell` (for trade menu)

### Migration Pattern

**Before:**
```rust
KeyCode::Char('i') => ui.inventory_menu.close(),
```

**After:**
```rust
code if CONFIG.matches_menu(code, "close_inventory") => ui.inventory_menu.close(),
```

Or use a generic close key:
```rust
code if CONFIG.matches_menu(code, "back") => ui.inventory_menu.close(),
```

## Benefits of Full Migration

1. **User Customization**: Players can rebind keys via JSON
2. **Consistency**: All keys defined in one place
3. **Documentation**: Config file serves as key reference
4. **Accessibility**: Easier to support alternative layouts (Dvorak, Colemak, etc.)
5. **Internationalization**: Support for non-QWERTY keyboards

## Testing Checklist

After migration:
- [ ] All menus open/close correctly
- [ ] Navigation works in all contexts
- [ ] No key conflicts between contexts
- [ ] Config file validates on load
- [ ] Invalid keys show clear error messages
- [ ] Default config covers all actions

## Related Files

- `data/keyboard_config.json` - Key definitions
- `src/game/keyboard_config.rs` - Config loading and matching
- `src/ui/input.rs` - Main input handler
- `src/ui/menu.rs` - Menu input handlers
- `docs/CONTROLS.md` - User-facing control documentation (needs update after migration)
