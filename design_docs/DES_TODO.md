# Debug Execution System (DES) - Implementation TODO

## Status Legend
- ✅ Implemented
- 🔨 In Progress
- ⏳ Pending
- 🚫 BLOCKED (requires core functionality not yet implemented)

## Core Features

| Feature | Status | Notes |
|---------|--------|-------|
| Game State Management | ✅ | GameState with RON serialization exists |
| RNG Seeding | ✅ | ChaCha8Rng with deterministic seeding |
| Entity Management | ✅ | Enemies, NPCs, Items exist |
| Basic Logging | ✅ | Messages vec in GameState |
| DES Module Structure | ✅ | src/des/mod.rs |
| DES Types | ✅ | Scenario, Action, EntitySpawn |
| DES JSON Parser | ✅ | Parse scenario files |
| DES Executor Core | ✅ | Execute scenarios headlessly |

## Blocked Features

| Feature | Status | Blocking Reason |
|---------|--------|-----------------|
| Action Queue System | 🚫 | Needs core action abstraction layer |
| Testing Framework Integration | 🚫 | Needs DES executor + assertions |
| Base DES File Inheritance | 🚫 | Needs parser + variable system |
| Action/State Indexing | 🚫 | Needs action queue implementation |
| Parallel Test Execution | 🚫 | Needs thread-safe DES executor |
| System Mocking | 🚫 | Needs dependency injection in GameState |
| CI Integration | 🚫 | Needs working test suite |
| Rendered Slow Execution | 🚫 | Needs UI decoupling + frame control |

## Implementation Plan

### Phase 1: Core DES (Current)
1. ✅ Create TODO documentation
2. 🔨 Define DES types (Scenario, Action, etc.)
3. 🔨 Implement JSON parser
4. 🔨 Implement basic executor
5. 🔨 Add dummy stubs for blocked features

### Phase 2: Action System (Future)
- Abstract action layer for player/entity actions
- Action queue with turn ordering
- Action indexing for replay/debug

### Phase 3: Advanced Features (Future)
- Base scenario inheritance
- Variable overrides
- Parallel execution
- System mocking
- CI pipeline integration

## Dummy Implementations

The following features have dummy implementations that will panic if called:
- `DES::run_parallel()` - Parallel test execution
- `DES::with_mocks()` - System mocking
- `DES::run_rendered()` - Slow rendered execution
- `Scenario::inherit_from()` - Base file inheritance
