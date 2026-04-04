# Debug Execution System (DES) Standards

## What is DES?

A custom scripting language for reproducible gameplay scenarios. Enables automated testing without manual play.

**Purpose**:
- Regression testing for gameplay features
- Bug reproduction with exact steps
- Performance benchmarking
- Save/load validation

## DES Scenario Structure

### File Format

**Location**: `tests/scenarios/*.des`

**Basic Structure**:
```des
# Comment: Test storm rotation mechanic
seed 12345
spawn player 10 10
spawn enemy mirage_hound 15 10
move player east
move player east
attack player east
assert player.hp > 0
assert enemy.hp < 100
wait 5
```

### Commands

**Setup**:
- `seed <number>` - Set RNG seed for determinism
- `spawn <type> <x> <y>` - Spawn entity at position
- `load <save_file>` - Load game state from file

**Actions**:
- `move <entity> <direction>` - Move entity (north/south/east/west)
- `attack <entity> <direction>` - Attack in direction
- `use <entity> <item>` - Use item from inventory
- `interact <entity> <x> <y>` - Interact with tile
- `wait <turns>` - Advance game turns

**Assertions**:
- `assert <condition>` - Verify game state
- `assert_tile <x> <y> <type>` - Check tile type
- `assert_entity <x> <y> <type>` - Check entity at position
- `assert_log <pattern>` - Verify log message

**Control Flow**:
- `repeat <n> { ... }` - Repeat commands
- `if <condition> { ... }` - Conditional execution

## Writing DES Scenarios

### Naming Convention

**Format**: `<feature>_<test_case>.des`

Examples:
- `storm_rotation_test.des`
- `combat_basic_attack.des`
- `quest_completion_flow.des`
- `adaptation_mutation_trigger.des`

### Scenario Template

```des
# Purpose: Test [feature] under [conditions]
# Expected: [outcome]

# Setup
seed 42
spawn player 10 10
spawn enemy test_enemy 15 10

# Action
move player east
move player east
attack player east

# Verification
assert player.hp > 0
assert enemy.hp < enemy.max_hp
assert_log "You attack"

# Cleanup (if needed)
wait 1
```

### Best Practices

**Determinism**:
- Always set seed at start
- Use same seed for reproducibility
- Document seed choice (e.g., 42 = simple test, 12345 = complex)

**Clarity**:
- Add comments explaining test purpose
- Group related commands
- Use descriptive entity names

**Scope**:
- One scenario per feature/bug
- Keep scenarios short (<50 commands)
- Split complex tests into multiple scenarios

**Assertions**:
- Verify expected outcomes
- Check edge cases (HP = 0, inventory full)
- Assert log messages for player feedback

## DES Integration

### Running Scenarios

**CLI**:
```bash
cargo test des_scenarios
cargo test -- --test-threads=1  # Sequential execution
```

**In Code**:
```rust
use crate::des::DESInterpreter;

#[test]
fn test_combat_scenario() {
    let scenario = include_str!("scenarios/combat_basic_attack.des");
    let mut interpreter = DESInterpreter::new();
    interpreter.run(scenario).expect("Scenario failed");
}
```

### Scenario Organization

**Directory Structure**:
```
tests/
├── scenarios/
│   ├── combat/
│   │   ├── basic_attack.des
│   │   ├── ranged_combat.des
│   │   └── aoe_damage.des
│   ├── quests/
│   │   ├── main_quest_01.des
│   │   └── side_quest_shrine.des
│   ├── storms/
│   │   ├── rotation_test.des
│   │   └── swap_test.des
│   └── regression/
│       ├── bug_123_fix.des
│       └── bug_456_fix.des
└── des_test_suite.rs
```

## Testing Strategies

### Feature Testing

**New Feature Checklist**:
1. Happy path scenario (feature works as intended)
2. Edge case scenarios (boundary conditions)
3. Failure scenarios (invalid input, error handling)
4. Integration scenarios (feature + other systems)

**Example: Storm Rotation**:
```des
# Happy path: Room rotates 90 degrees
seed 100
spawn player 10 10
assert_tile 15 10 wall
trigger_storm rotation
wait 10
assert_tile 10 15 wall  # Wall moved

# Edge case: Rotation at map boundary
seed 101
spawn player 1 1
trigger_storm rotation
wait 10
assert player.hp > 0  # No crash

# Failure: Storm during combat
seed 102
spawn player 10 10
spawn enemy 11 10
trigger_storm rotation
attack player east
assert enemy.hp < 100  # Combat still works
```

### Regression Testing

**Bug Fix Workflow**:
1. Reproduce bug with DES scenario
2. Verify scenario fails before fix
3. Apply fix
4. Verify scenario passes after fix
5. Keep scenario in `regression/` folder

**Example: Bug #123 - Player falls through floor**:
```des
# Bug #123: Player falls through floor after storm
# Reproduction steps from issue report
seed 12345
spawn player 10 10
trigger_storm fuse_walls
move player south
assert_tile 10 11 floor  # Should be floor, not void
assert player.y == 11    # Player should move
```

### Performance Testing

**Benchmark Scenarios**:
```des
# Benchmark: FOV calculation with 100 entities
seed 999
spawn player 50 50
repeat 100 {
    spawn enemy random_pos
}
benchmark_start fov_calc
move player north
benchmark_end fov_calc
assert benchmark.fov_calc < 5ms
```

## DES Interpreter Implementation

### Command Parsing

**Tokenization**:
```rust
enum DESToken {
    Command(String),
    Argument(String),
    Number(i32),
    Direction(Direction),
    BlockStart,
    BlockEnd,
}
```

**Execution**:
```rust
impl DESInterpreter {
    pub fn run(&mut self, scenario: &str) -> Result<(), DESError> {
        let tokens = self.tokenize(scenario)?;
        for token in tokens {
            self.execute(token)?;
        }
        Ok(())
    }
}
```

### State Management

**Scenario Context**:
```rust
struct DESContext {
    game_state: GameState,
    entities: HashMap<String, EntityId>,
    variables: HashMap<String, Value>,
    assertions: Vec<AssertionResult>,
}
```

### Error Handling

**Clear Error Messages**:
```
DES Error at line 15: Entity 'player' not found
  spawn player 10 10
  ^^^^^
Did you forget to spawn the player?
```

## Advanced DES Features

### Variables

```des
set player_hp = player.hp
move player north
assert player.hp == $player_hp  # HP unchanged
```

### Loops

```des
repeat 10 {
    move player east
    wait 1
}
```

### Conditionals

```des
if player.hp < 50 {
    use player healing_potion
}
```

### Macros

```des
macro setup_combat {
    seed 42
    spawn player 10 10
    spawn enemy 15 10
}

setup_combat
attack player east
```

## Documentation

### Scenario Comments

**Required**:
- Purpose (what is being tested)
- Expected outcome
- Seed rationale (if non-standard)

**Optional**:
- Related issue/PR numbers
- Known limitations
- Future improvements

### DES Language Docs

**Maintain**:
- `docs/development/DES_USAGE.md` - User guide
- `docs/development/DES_README.md` - Implementation details
- Inline rustdoc for interpreter code

## CI/CD Integration

### Automated Testing

**GitHub Actions**:
```yaml
- name: Run DES scenarios
  run: cargo test des_scenarios --release
```

**Pre-commit Hook**:
```bash
#!/bin/bash
cargo test des_scenarios || exit 1
```

### Scenario Validation

**Lint Scenarios**:
- Check for missing seeds
- Verify entity references
- Validate assertion syntax

**Coverage Tracking**:
- Track which features have DES coverage
- Report uncovered gameplay paths
