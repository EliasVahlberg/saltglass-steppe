# Subagent Orchestration Guide

## Available Subagents

### systems-engineer
**Purpose**: Core gameplay systems, procedural generation, performance  
**Access**: Read, write, execute  
**Use when**: Implementing mechanics, optimizing algorithms, integrating terrain-forge

### ui-developer
**Purpose**: TUI rendering, input handling, visual effects  
**Access**: Read, write, execute  
**Use when**: Building screens/menus, optimizing rendering, creating ASCII effects

### qa-tester
**Purpose**: DES scenarios, integration tests, bug reproduction  
**Access**: Read, write, execute  
**Use when**: Writing tests, finding edge cases, validating determinism

### content-writer
**Purpose**: Lore, dialogue, quests, flavor text  
**Access**: Read, write, web search  
**Use when**: Writing narrative content, item descriptions, quest design

### gameplay-balancer
**Purpose**: Difficulty tuning, progression, mechanics balance  
**Access**: Read, write, execute  
**Use when**: Balancing combat, tuning progression, adjusting configs

### proc-gen-developer
**Purpose**: Procedural generation, terrain-forge integration, biome profiles, deterministic world building  
**Access**: Read, write, execute  
**Use when**: Implementing generation algorithms, adding biomes, tuning terrain-forge parameters, fixing generation bugs

### save-load-engineer
**Purpose**: Save system, RON serialization, versioning, migration functions  
**Access**: Read, write, execute  
**Use when**: Adding save fields, bumping SAVE_VERSION, writing migration functions, debugging save/load issues

### quest-writer
**Purpose**: Quest design, DES scenario scripting, dialogue trees, narrative hooks  
**Access**: Read, write, execute  
**Use when**: Designing quests, writing dialogue, implementing quest mechanics in JSON, testing quest flows with DES

### combat-engineer
**Purpose**: Combat mechanics, enemy AI, mob stats, status effects, adaptation balance  
**Access**: Read, write, execute  
**Use when**: Implementing combat features, designing enemies, tuning mob stats, balancing adaptations

## Important: File Creation for Subagents

**⚠️ CRITICAL**: Subagents may not have permission to create new files. Before delegating tasks that require file output:

1. **Create empty placeholder files** in the target locations
2. **Verify file permissions** are writable
3. **Then delegate** to subagent to populate the files

Example:
```bash
# Before delegating enemy creation:
mkdir -p data/enemies
touch data/enemies/{common,uncommon,rare,elite,boss}.json
# Now delegate to content-writer
```

## When to Use Subagents

### LeadDeveloper Workflows

**New Gameplay Feature**:
```
1. systems-engineer: Implement core mechanics
2. ui-developer: Create TUI interface
3. qa-tester: Write DES scenarios
4. content-writer: Add flavor text and lore
5. gameplay-balancer: Tune difficulty and costs
```

**Bug Fix**:
```
1. qa-tester: Reproduce with DES scenario
2. systems-engineer: Fix the issue
3. qa-tester: Verify fix with regression test
```

**Performance Optimization**:
```
1. qa-tester: Benchmark current performance
2. systems-engineer: Optimize hot paths
3. qa-tester: Verify improvement
```

**UI Enhancement**:
```
1. ui-developer: Implement new screen/menu
2. qa-tester: Test input handling and edge cases
3. content-writer: Add help text and descriptions
```

### CreativeDirector Workflows

**New Quest Chain**:
```
1. quest-writer: Design quest, write dialogue, implement JSON data
2. quest-writer: Write DES scenarios for all quest branches
3. gameplay-balancer: Tune rewards and difficulty
```

**World-Building**:
```
1. content-writer: Expand lore and faction backstories
2. systems-engineer: Implement faction reputation system
3. gameplay-balancer: Balance faction rewards/penalties
```

**Content Expansion**:
```
1. content-writer: Design new items, enemies, locations
2. systems-engineer: Implement data structures
3. gameplay-balancer: Balance stats and costs
4. qa-tester: Validate spawning and interactions
```

## Orchestration Patterns

### Sequential Pipeline
When tasks depend on each other:
```
> systems-engineer: Implement storm rotation mechanic
  (wait for completion)
> ui-developer: Add storm forecast UI
  (wait for completion)
> qa-tester: Write DES scenarios for storm system
```

### Parallel Execution
When tasks are independent (up to 4 simultaneous):
```
> After implementing the adaptation system:
  - content-writer: Write adaptation descriptions and lore
  - gameplay-balancer: Tune adaptation costs and effects
  - qa-tester: Write DES scenarios for each adaptation
  - ui-developer: Create adaptation selection menu
```

### Iterative Refinement
For complex features requiring feedback loops:
```
> systems-engineer: Implement draft of light tactics system
> qa-tester: Test edge cases and find issues
> systems-engineer: Refine based on test results
> gameplay-balancer: Tune damage and range values
> qa-tester: Validate final implementation
```

## Best Practices

### Be Explicit
Specify which subagent and what task:
```
✓ "systems-engineer: Implement FOV calculation using shadowcasting"
✓ "ui-developer: Create inventory menu with grid layout"
✓ "qa-tester: Write DES scenario for combat edge cases"
✗ "Fix the rendering" (unclear who and what)
```

### Batch Related Work
Group similar tasks for efficiency:
```
✓ "content-writer: Write descriptions for all 15 adaptation types"
✓ "qa-tester: Create DES scenarios for all quest branches"
✗ Requesting one item at a time (inefficient)
```

### Leverage Parallelism
Independent tasks run simultaneously:
```
✓ "systems-engineer implement, ui-developer create UI, qa-tester write tests"
✗ Sequential when parallel would work (slower)
```

### Trust the Specialists
They have focused expertise:
```
✓ "ui-developer: Optimize rendering to hit 60fps target"
✓ "content-writer: Ensure tone matches mythic-reverent style"
✗ Micromanaging implementation details
```

## Common Workflows

### New Gameplay System
1. **systems-engineer**: Implement core mechanics in `src/game/`
2. **ui-developer**: Create TUI interface in `src/ui/`
3. **qa-tester**: Write DES scenarios in `tests/scenarios/`
4. **content-writer**: Add flavor text and lore
5. **gameplay-balancer**: Tune values in `data/` configs

### New Combat Feature
1. **combat-engineer**: Implement mechanics, design enemy stats
2. **qa-tester**: Write DES scenarios with mock combat settings
3. **gameplay-balancer**: Tune balance across difficulty tiers

### New Biome / Generation Feature
1. **proc-gen-developer**: Implement biome profile, terrain-forge params
2. **qa-tester**: Test generation determinism and edge cases
3. **content-writer**: Write biome descriptions and lore

### Save System Change
1. **save-load-engineer**: Add fields, bump SAVE_VERSION, write migration
2. **qa-tester**: Validate save/load round-trip with DES

### New Quest
1. **quest-writer**: Design quest, write dialogue, implement JSON, write DES tests
2. **gameplay-balancer**: Tune rewards and faction consequences
3. **content-writer**: Polish tone and lore consistency

### Storm System Feature
1. **systems-engineer**: Implement map editing algorithms
2. **ui-developer**: Add storm forecast UI and visual effects
3. **qa-tester**: Test determinism and edge cases
4. **content-writer**: Write storm descriptions and log messages
5. **gameplay-balancer**: Tune storm frequency and intensity

### Refraction Adaptation
1. **content-writer**: Design adaptation trees and lore
2. **systems-engineer**: Implement mutation system
3. **gameplay-balancer**: Balance power vs. social cost
4. **ui-developer**: Create adaptation selection menu
5. **qa-tester**: Validate all adaptation paths

### Bug Investigation
1. **qa-tester**: Reproduce bug with DES scenario
2. **systems-engineer** or **ui-developer**: Fix the issue
3. **qa-tester**: Verify fix and add regression test

### Performance Issue
1. **qa-tester**: Benchmark and identify bottleneck
2. **systems-engineer** or **ui-developer**: Optimize hot path
3. **qa-tester**: Verify performance improvement

### Content Update
1. **content-writer**: Write new quests, items, or lore
2. **systems-engineer**: Implement data structures if needed
3. **gameplay-balancer**: Tune stats and rewards
4. **qa-tester**: Validate content integration

## Subagent Limitations

- **No inter-subagent communication** - They work independently
- **No persistent state** - Each invocation is fresh context
- **Max 4 parallel** - Additional requests queue
- **Core tools only** - No MCP servers in subagents
- **No TUI testing** - Can't run game interactively (use DES instead)

## Example Commands

### LeadDeveloper Examples
```
> systems-engineer: Implement glass storm rotation algorithm in src/game/storm.rs

> ui-developer: Create storm forecast UI showing intensity and edit type

> qa-tester: Write DES scenarios for storm system edge cases

> Parallel: systems-engineer implement FOV, ui-developer optimize rendering,
  qa-tester benchmark performance

> content-writer: Write descriptions for all biome types following tone guide
```

### CreativeDirector Examples
```
> content-writer: Design main quest chain following creative pillars

> gameplay-balancer: Tune refraction adaptation costs and faction penalties

> Parallel: content-writer write item lore, gameplay-balancer balance stats

> content-writer: Expand Mirror Monk faction lore and dialogue trees
```

## Quality Gates

### Before Merging Features
1. **systems-engineer**: Implementation complete and tested
2. **ui-developer**: UI is clear and accessible
3. **qa-tester**: DES scenarios pass, edge cases covered
4. **content-writer**: Tone is consistent, lore is integrated
5. **gameplay-balancer**: Balance is tuned, multiple builds viable

### Code Review Checklist
- [ ] Follows creative pillars
- [ ] Deterministic (seeded RNG)
- [ ] Data-driven (configs, not hardcoded)
- [ ] TUI-friendly (clear ASCII representation)
- [ ] Tested with DES scenarios
- [ ] Documented (rustdoc + design docs)
- [ ] Performance acceptable (<16ms frame time)

## Escalation

### When Subagents Need Help
If a subagent encounters issues beyond their scope:
- **systems-engineer** → LeadDeveloper for architecture decisions
- **ui-developer** → LeadDeveloper for rendering architecture
- **proc-gen-developer** → LeadDeveloper for generation architecture decisions
- **save-load-engineer** → LeadDeveloper for state structure changes
- **combat-engineer** → LeadDeveloper for systems architecture, CreativeDirector for design philosophy
- **quest-writer** → CreativeDirector for tone/lore conflicts, LeadDeveloper for DES/technical issues
- **content-writer** → CreativeDirector for tone/lore conflicts
- **gameplay-balancer** → CreativeDirector for design philosophy

### Cross-Discipline Issues
When features span multiple domains:
- LeadDeveloper orchestrates technical subagents
- CreativeDirector orchestrates content subagents
- Both collaborate on features requiring technical + creative alignment
