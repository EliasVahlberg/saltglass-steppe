# QA Testing Guide

This document describes the comprehensive QA testing tools available in the TUI RPG for debugging, issue reporting, and deterministic testing.

## Overview

The QA testing system provides:
- **Debug Console**: Enhanced command-line interface for debugging
- **Debug Menu**: Visual debug information and controls
- **Issue Reporter**: Guided issue reporting with automatic state capture
- **Debug State Management**: Save/load game states for reproduction
- **DES Testing**: Deterministic Event Sequence testing for automated validation

## Quick Access Reference

| Feature | Access Key | Description |
|---------|------------|-------------|
| Debug Console | `` ` `` (backtick) | Command-line debug interface |
| Debug Menu | `F12` | Visual debug information panel |
| Issue Reporter | `` ` `` → `report_issue` | Guided bug reporting tool |

## Debug Console

### Access
Press `` ` `` (backtick) during gameplay to open the debug console.

### Available Commands

#### Basic Debug Commands
- `show tile` - Enable god view (see entire map)
- `hide tile` - Disable god view
- `sturdy` - Set player HP to 9999 (god mode)
- `phase` - Toggle wall phasing (walk through walls)
- `help` - Show all available commands

#### State Management
- `save_debug [name]` - Save current game state
  - Example: `save_debug boss_fight`
  - Auto-names with timestamp if no name provided
- `load_debug <name>` - Load a saved debug state
  - Example: `load_debug boss_fight.ron`
- `list_debug` - List all saved debug states

#### Information Commands
- `debug_info` - Show detailed game state information
  - Player position, HP, turn number, seed
  - Enemy/item counts, storm status
  - Memory usage

#### Issue Reporting
- `report_issue` - Open the guided issue reporter

#### DES Testing
- `run_des <file>` - Execute a DES test file (looks in tests/ directory)
- `list_des` - List available DES test files
- `create_sample_des` - Create a sample DES test file

## Debug Menu (F12)

### Access
Press `F12` during gameplay to open the visual debug menu.

### Navigation
- `Tab` - Switch between tabs
- `Shift+Tab` - Previous tab
- `F12` or `Esc` - Close menu

### Tabs

#### Info Tab
- Current game state (turn, position, HP)
- Player statistics
- System information
- Storm status

#### Performance Tab
- FPS metrics (simulated)
- Memory usage gauge
- Performance timing data

#### States Tab
- List of saved debug states
- Quick access to state files

#### Commands Tab
- Reference list of all debug commands
- Quick command lookup

## Issue Reporter

### Access
1. Open debug console with `` ` ``
2. Type `report_issue` and press Enter
3. Console closes automatically and issue reporter opens

### Workflow (6 Steps)

#### Step 1: Description
- Provide a clear, concise description of the issue
- Press `Enter` to proceed to next step

#### Step 2: Reproduction Steps
- Add steps one by one to reproduce the issue
- Type step description and press `Enter` to add
- Press `Enter` on empty field to proceed to next step
- Press `Backspace` to remove last step

#### Step 3: Expected Behavior
- Describe what you expected to happen
- Press `Enter` to proceed

#### Step 4: Actual Behavior
- Describe what actually happened
- Press `Enter` to proceed

#### Step 5: Severity
- Press `Space` to cycle through severity levels:
  - Low, Medium, High, Critical
- Press `Enter` to proceed

#### Step 6: Category
- Press `Space` to cycle through categories:
  - Gameplay, UI, Performance, Save, Combat, AI, Map, Other
- Press `Enter` to proceed

#### Step 7: Review
- Review all information
- Press `Enter` to submit
- Press `Backspace` to go back and edit

### Output
- Issue report saved to `issue_reports/issue_TIMESTAMP.json`
- Game state automatically saved to `debug_states/issue_TIMESTAMP.ron`
- Confirmation message in game log

## Debug State Management

### File Locations
- **Debug States**: `debug_states/` directory
- **Issue Reports**: `issue_reports/` directory

### File Formats
- **Debug States**: RON format (`.ron` files)
- **Issue Reports**: JSON format (`.json` files)

### Usage Patterns

#### Reproducing Issues
1. Save state before attempting reproduction: `save_debug before_bug`
2. Reproduce the issue
3. Save state after issue occurs: `save_debug after_bug`
4. Load the "before" state to test fixes: `load_debug before_bug.ron`

#### Testing Scenarios
1. Save at key game moments (boss fights, story events)
2. Use for regression testing after code changes
3. Share states with other developers for collaboration

## DES Testing (Deterministic Event Sequence)

### Purpose
Automated testing system for verifying game behavior remains consistent across code changes.

### Creating Tests

#### Sample Test Creation
```
` → create_sample_des
```
This creates `sample_test.des` with basic movement test.

#### Test File Structure (JSON)
```json
{
  "name": "test_name",
  "description": "Test description",
  "initial_state_file": "debug_state.ron",
  "actions": [
    {
      "action_type": "move",
      "parameters": ["1", "0"],
      "description": "Move right"
    }
  ],
  "expected_outcomes": [
    {
      "check_type": "player_position",
      "expected_value": "10,5",
      "description": "Player should be at (10,5)"
    }
  ]
}
```

#### Available Actions
- `move` - Parameters: [dx, dy]
- `use_item` - Parameters: [item_index]
- `wait` - Parameters: [turns] (optional, defaults to 1)
- `debug_command` - Parameters: [command, args...]

#### Available Expectations
- `player_hp` - Expected HP value
- `player_position` - Expected position "x,y"
- `inventory_contains` - Item ID that should be in inventory
- `inventory_count` - Expected inventory size
- `turn_number` - Expected turn number
- `enemy_count` - Expected number of enemies

### Running Tests
```
` → run_des sample_test.des
```

### Test Results
- Pass/fail status
- Detailed execution log
- Failed expectations list
- Final state saved for analysis

## GitHub Issue Integration

### Automatic Issue Creation
When using the issue reporter, the generated files can be easily attached to GitHub issues:

1. **Issue Report JSON**: Contains structured issue data
2. **Debug State RON**: Contains exact game state for reproduction

### GitHub Issue Template
```markdown
## Bug Report

**Description:** [Copy from issue report JSON]

**Steps to Reproduce:**
[Copy reproduction steps from issue report]

**Expected Behavior:** [Copy from issue report]

**Actual Behavior:** [Copy from issue report]

**Environment:**
- OS: [From system_info in JSON]
- Game Version: [From system_info in JSON]
- Seed: [From system_info in JSON]
- Turn: [From system_info in JSON]

**Debug Files:**
- Issue Report: `issue_reports/[filename].json`
- Game State: `debug_states/[filename].ron`

**Severity:** [From issue report]
**Category:** [From issue report]
```

### Attaching Files to GitHub Issues
1. Locate files in `issue_reports/` and `debug_states/`
2. Drag and drop JSON and RON files to GitHub issue
3. Or use GitHub CLI: `gh issue create --title "Bug Title" --body-file issue_report.json`

## Best Practices

### For Testers
1. **Always save state before testing** - Use `save_debug` before attempting to reproduce issues
2. **Use descriptive names** - `save_debug boss_fight_crash` vs `save_debug test1`
3. **Include reproduction steps** - Be specific in the issue reporter
4. **Test with different seeds** - Some bugs may be seed-dependent

### For Developers
1. **Load reported states** - Use `load_debug` to jump directly to reported issues
2. **Create DES tests** - Convert bug reports into automated tests
3. **Verify fixes** - Load "before" state, apply fix, verify "after" state
4. **Share states** - Debug states can be shared between team members

### For QA Workflow
1. **Daily testing routine**:
   - Load key saved states
   - Run DES test suite: `list_des` then `run_des` for each
   - Report any failures through issue reporter

2. **Release testing**:
   - Create comprehensive DES tests for major features
   - Save states at critical game moments
   - Verify all existing DES tests pass

## Troubleshooting

### Common Issues

#### "No such file or directory" when saving
- Directories are created automatically
- Check file permissions in project directory

#### Debug console not responding
- Press `Esc` to ensure no other menus are open
- Try pressing `` ` `` again to toggle

#### Issue reporter stuck on step
- Press `Esc` to cancel and restart
- Ensure you're not in debug console mode

#### DES test failures
- Verify initial state file exists
- Check that expected values match actual game behavior
- Use `debug_info` to see current state values

### File Management
- Debug states and issue reports are automatically ignored by git
- Clean up old files periodically to save disk space
- Archive important test states for long-term use

## Advanced Usage

### Custom DES Tests
Create comprehensive test suites for specific features:

```json
{
  "name": "combat_system_test",
  "description": "Verify combat calculations remain consistent",
  "initial_state_file": "combat_setup.ron",
  "actions": [
    {"action_type": "debug_command", "parameters": ["sturdy"], "description": "Enable god mode"},
    {"action_type": "move", "parameters": ["5", "0"], "description": "Move to enemy"},
    {"action_type": "wait", "parameters": ["1"], "description": "End turn"}
  ],
  "expected_outcomes": [
    {"check_type": "player_hp", "expected_value": "9999", "description": "God mode should preserve HP"},
    {"check_type": "enemy_count", "expected_value": "0", "description": "Enemy should be defeated"}
  ]
}
```

### Batch Testing
Run multiple DES tests in sequence by creating a script that calls `run_des` for each test file.

This comprehensive QA system ensures reliable bug reporting, efficient debugging, and automated regression testing for the TUI RPG project.
