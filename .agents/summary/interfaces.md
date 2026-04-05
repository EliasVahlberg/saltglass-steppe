# Interfaces

## VERA Dispatch Interface

The primary interface for all gameplay actions:

```rust
// Command → dispatch → rule → effects → apply → trace
impl GameState {
    pub fn dispatch(&mut self, command: Command);
}
```

### Command Enum (22 variants)

| Command | Rule Function | Module |
|---------|--------------|--------|
| `UseItem { index }` | `rule_use_item` | `rules/item.rs` |
| `UseItemOnTile { index, x, y }` | `rule_use_item_on_tile` | `rules/item.rs` |
| `Move { dx, dy }` | `rule_move` | `rules/movement.rs` |
| `Attack { target_x, target_y }` | `rule_melee_attack` | `rules/combat.rs` |
| `RangedAttack { target_x, target_y }` | `rule_ranged_attack` | `rules/combat.rs` |
| `Wait` | `rule_wait` | `rules/actions.rs` |
| `Rest` | `rule_rest` | `rules/actions.rs` |
| `Equip { inv_idx, slot }` | `rule_equip` | `rules/actions.rs` |
| `Unequip { slot }` | `rule_unequip` | `rules/actions.rs` |
| `AllocateStat { stat }` | `rule_allocate_stat` | `rules/actions.rs` |
| `AcceptQuest { quest_id }` | dispatch helper | `state.rs` |
| `CompleteQuest { quest_id }` | dispatch helper | `state.rs` |
| `Interact { x, y }` | dispatch helper | `state.rs` |
| `Examine { x, y }` | dispatch helper | `state.rs` |
| `UsePsychic { ability_id }` | `rule_use_psychic` | `rules/actions.rs` |
| `FleeEncounter` | dispatch helper | `state.rs` |
| `WorldMove { new_wx, new_wy }` | dispatch helper | `state.rs` |
| `WorldMoveSafe { new_wx, new_wy }` | dispatch helper | `state.rs` |
| `EnterSubterranean` | dispatch helper | `state.rs` |
| `ExitSubterranean` | dispatch helper | `state.rs` |
| `FollowWorldPath` | dispatch helper | `state.rs` |
| `CalculateWorldPath { target_wx, target_wy }` | dispatch helper | `state.rs` |

### Effect Enum (7 domains)

| Domain | Variants | Purpose |
|--------|----------|---------|
| `PlayerEffect` | 34 | HP, AP, position, stats, status, bridge ticks |
| `CombatEffect` | 5 | Damage, miss, kill, provoke, stun |
| `ItemEffect` | 6 | Consume, equip, unequip, inventory |
| `MapEffect` | 9 | Reveal, wall damage, time, weather, storm bridge |
| `ResourceEffect` | 5 | Light/void/resonance energy, crystal placement |
| `EventEffect` | 3 | Book open, loot drop reaction, quest notify reaction |
| `QuestEffect` | 3 | Accept, complete, faction alignment |

## QueryContext Interface

Read-only view of game state for rule functions:

```rust
pub struct QueryContext<'a> {
    pub player: &'a PlayerState,
    pub world: &'a WorldState,
    pub turn: u32,
    pub visible: &'a HashSet<usize>,
    pub enemy_positions: &'a HashMap<(i32, i32), usize>,
    pub npc_positions: &'a HashMap<(i32, i32), usize>,
    pub mock_combat_hit: Option<bool>,
    pub mock_combat_damage: Option<i32>,
    pub time_of_day: u8,
    pub encounter_state: Option<&'a EncounterState>,
    pub player_adaptations: &'a [Adaptation],
    pub player_refraction: u32,
}
```

Key convenience methods:
- `from_state(state: &GameState) -> Self`
- `item_def(id: &str) -> Option<&ItemDef>`
- `enemy_idx_at(x, y) -> Option<usize>`
- `has_npc_at(x, y) -> bool`
- `has_enemy_at(x, y) -> bool`
- `has_adaptation(id: &str) -> bool`

## TestContext Interface

Builder for unit testing rules without GameState:

```rust
TestContext::new()
    .with_player_hp(100)
    .with_player_ap(10)
    .with_inventory(vec!["healing_salve".into()])
    .with_enemy_at("salt_crawler", 7, 5)
    .with_floor_at(6, 5)
    .with_mock_combat_hit(true)
    .build()  // → QueryContext
```

## System Trait

Legacy system interface (being replaced by VERA):

```rust
pub trait System {
    fn update(&self, state: &mut GameState);
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
```

## AI Behavior Trait

```rust
pub trait AiBehavior: Send + Sync {
    fn execute(&self, entity_idx: usize, state: &mut GameState) -> bool;
}
```

4 implementations: `StandardMeleeBehavior`, `RangedOnlyBehavior`, `HealerBehavior`, `SuicideBomberBehavior`. Registered in `BEHAVIOR_REGISTRY` (lazy static HashMap).

## DataLoader Interface

Generic data loading with schema validation:

```rust
pub struct DataLoader<T: DeserializeOwned> { ... }

impl<T> DataLoader<T> {
    pub fn load_single(path: &str) -> Result<T>;
    pub fn load_multiple(paths: &[&str]) -> Result<Vec<T>>;
    pub fn get(id: &str) -> Option<&T>;
    pub fn all() -> &[T];
    pub fn ids() -> Vec<String>;
}
```

## DES Scenario Interface

JSON-based test scenarios:

```rust
pub struct Scenario {
    pub name: String,
    pub inherits: Option<String>,
    pub seed: u64,
    pub map_setup: Option<MapSetup>,
    pub player: Option<PlayerSetup>,
    pub entities: Vec<EntitySpawn>,
    pub mocks: Option<MockSettings>,
    pub actions: Vec<ScheduledAction>,
    pub assertions: Vec<Assertion>,
}
```

Key DES methods:
- `DesExecutor::from_json(json) -> Self`
- `run() -> ExecutionResult`
- `execute_player_action(action)` — dispatches via VERA `Command` for migrated actions
- `check_assertion(assertion) -> AssertionResult`

## IPC Interface

Multi-terminal communication:

```rust
pub enum IpcMessage {
    // Game state updates sent to satellite terminals
    // Log messages, status updates, inventory changes
}

pub struct IpcServer { ... }  // Main game process
pub struct IpcClient { ... }  // Satellite terminal
```

## Save/Load Interface

```rust
impl GameState {
    pub fn save(&self, slot: &str) -> Result<()>;
    pub fn load(slot: &str) -> Result<GameState>;
}
```

Uses `serde_json` serialization with MD5 checksums and version-based migration.
