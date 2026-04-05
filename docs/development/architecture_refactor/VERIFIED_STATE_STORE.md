# Verified State Store

> Canonical architecture reference for state management in saltglass-steppe.

---

## Overview

GameState is a verified data store. It holds all game data, applies mutations with per-field invariant checks, and reports state transitions. It contains zero system logic and zero notification logic.

Three roles, strictly separated:

- **Systems** decide what should change. They receive read-only state and return atomic mutation requests.
- **State** applies mutations, enforces invariants, and reports what changed.
- **Notifications** react to state transitions. They map transitions to reactive mutations, enabling cross-system coordination without coupling.

```
Input (Command)
  │
  ▼
dispatch.rs ─── routes to system
  │
  ▼
System::handle(query, rng) → Vec<Mutation>         ← system owns all logic
  │
  ▼
state.apply_mutations(mutations) → Vec<Transition>  ← state owns data + verification
  │
  ▼
notify::on_transitions(transitions, state, rng)     ← external, expandable
  → Vec<Mutation>
  │
  ▼
state.apply_mutations(reactions) → Vec<Transition>   ← cascade (depth-limited to 10)
  │
  ▼
Derives: FOV, lighting, spatial index                ← run once after all cascades settle
```

## File Structure

```
src/game/
├── state.rs           — GameState struct, apply_mutations → Vec<Transition>, apply_one, derives, accessors
├── state_init.rs      — new(), new_with_class()
├── mutations.rs       — Mutation enum, StateTransition enum, SubsystemId
├── notify.rs          — on_transitions(): transition → reactive mutations
├── dispatch.rs        — Command → system routing, cascade loop
├── systems/
│   ├── mod.rs
│   ├── combat.rs      — handle_melee, handle_ranged, on_enemy_hit, on_enemy_killed
│   ├── movement.rs    — handle_move
│   ├── items.rs       — handle_use_item, handle_equip, handle_craft, handle_trade, on_player_moved
│   ├── world.rs       — handle_world_move, handle_subterranean, on_player_moved, on_entered_world_tile
│   ├── player.rs      — handle_wait, handle_rest, handle_psychic, handle_stats, handle_flee
│   ├── quest.rs       — handle_accept, handle_complete, on_enemy_killed, on_item_collected
│   ├── turn.rs        — handle_end_turn, on_ap_exhausted, on_turn_advanced
│   ├── loot.rs        — on_enemy_killed → spawn items
│   ├── ai.rs          — update_enemies (bridge)
│   ├── storm.rs       — tick (bridge)
│   ├── status.rs      — tick effects (bridge)
│   ├── explore.rs     — auto_explore
│   └── interact.rs    — handle_interact, handle_examine
├── effects/
│   ├── mod.rs         — Command enum
│   ├── context.rs     — QueryContext (read-only state view)
│   └── trace.rs       — trace recording
```

## Mutations

Every mutation changes exactly one state field. Systems produce these. GameState applies them.

```rust
pub enum Mutation {
    // Player vitals
    SetPlayerHp(i32),
    SetPlayerMaxHp(i32),
    SetPlayerAp(i32),
    SetPlayerMaxAp(i32),
    SetPlayerPosition { x: i32, y: i32 },
    SetPlayerReflex(i32),
    SetPlayerArmor(i32),

    // Player progression
    SetPlayerXp(u32),
    SetPlayerLevel(u32),
    SetPlayerStatPoints(i32),
    SetPlayerSkillPoints(u32),
    SetPlayerSaltScrip(u32),

    // Player state
    SetPlayerRefraction(u32),
    SetWaitCounter(u32),
    SetAdaptationsHidden(u32),
    AddAdaptation(String),
    AddStatusEffect { id: String, duration: i32 },
    SetLastDamageDealt(u32),

    // Inventory & equipment
    AddToInventory(String),
    RemoveFromInventory(usize),
    SetEquipment { slot: String, item_id: Option<String> },
    SpawnItemOnMap { item_id: String, x: i32, y: i32 },

    // Enemies
    SetEnemyHp { idx: usize, hp: i32 },
    SetEnemyProvoked { idx: usize, provoked: bool },
    AddEnemyStatus { idx: usize, id: String, duration: i32 },
    RemoveEnemy { idx: usize, x: i32, y: i32 },
    SpawnEnemy { id: String, x: i32, y: i32 },

    // World state
    SetWorldPosition { wx: usize, wy: usize },
    SetLayer(i32),
    SetTimeOfDay(u8),
    SetWeather(Weather),
    IncrementTilesTraveled,
    AdvanceTurn,

    // Map
    SetTile { idx: usize, tile: Tile },
    RevealTile(usize),
    RevealAll,
    ClearStormHighlight(usize),
    SetWorldPath { path: Vec<(usize, usize)>, target: Option<(usize, usize)> },
    ClearWorldPath,

    // Encounter
    SetEncounterState(Option<Box<EncounterState>>),
    IncrementEncounterTimer,
    SetLastFleeAttempt(u32),

    // Faction & quest
    SetReputation { faction: String, value: i32 },
    AcceptQuest(String),
    CompleteQuest(String),
    SetFactionAlignment(String),

    // Resources
    SetLightEnergy(u32),
    SetVoidEnergy(u32),
    SetVoidExposure(u32),
    SetResonanceEnergy(u32),
    PlaceCrystal { x: i32, y: i32, frequency: String },

    // Presentation (no verification, no transitions)
    LogMessage { text: String, msg_type: MsgType },
    OpenBook(String),
    PlaceDecoy { x: i32, y: i32 },
    HitFlash { x: i32, y: i32 },
    DamageNumber { x: i32, y: i32, value: i32, is_heal: bool },
    SpawnProjectile { from: (i32, i32), to: (i32, i32), ch: char },

    // Subsystem ticks (bridge — subsystem handles internally)
    TickSubsystem(SubsystemId),
}

pub enum SubsystemId {
    Psychic, Skills, Light, Void, Crystal, Status, AI, Storm, Housekeeping,
}
```

## State Transitions

Detected inside `apply_one()` by comparing pre/post values. State reports these but takes no action.

```rust
pub enum StateTransition {
    PlayerPositionChanged { old_x: i32, old_y: i32, new_x: i32, new_y: i32 },
    PlayerApReachedZero,
    PlayerDied,
    EnemyHpChanged { idx: usize, old_hp: i32, new_hp: i32 },
    EnemyHpReachedZero { idx: usize, enemy_id: String, x: i32, y: i32 },
    TurnAdvanced { old_turn: u32, new_turn: u32 },
    ItemAddedToInventory { item_id: String },
    PlayerEnteredWorldTile { wx: usize, wy: usize },
}
```

## Verification Rules

Each mutable field has an invariant enforced inside `apply_one()`. Systems never bypass it.

| Field | Invariant | Enforcement |
|-------|-----------|-------------|
| `player.hp` | 0 ≤ hp ≤ max_hp | Clamp |
| `player.ap` | 0 ≤ ap ≤ max_ap | Clamp |
| `player.max_hp` | ≥ 1 | Clamp(1, ..) |
| `player.max_ap` | ≥ 1 | Clamp(1, ..) |
| `player.xp` | Monotonically increasing | max(old, new) |
| `player.level` | 1 ≤ level ≤ max_level | Clamp |
| `player.salt_scrip` | ≥ 0 | u32 (natural) |
| `faction_reputation` | -100 ≤ rep ≤ 100 | Clamp |
| `enemy[idx].*` | idx in bounds | No-op if out of bounds |
| `world.time_of_day` | 0..24 | Modulo 24 |
| `resonance_energy` | ≤ max_resonance_energy | Clamp |
| Presentation mutations | None | Always applied |

## Notification Module

`notify.rs` is external to state. It maps transitions to reactive mutations. This is the single place for all cross-system coordination.

- Adding a new reaction: add a line to the relevant transition match arm in `notify.rs`
- Adding a new transition type: add a variant to `StateTransition`, add detection in `apply_one()`, add a match arm in `notify.rs`

Subscribers are static (compile-time match arms, not dynamic registration). The compiler enforces exhaustive handling.

## Dispatch Loop

`dispatch.rs` owns the cascade:

1. Route command to system → `Vec<Mutation>`
2. `state.apply_mutations(mutations)` → `Vec<StateTransition>`
3. `notify::on_transitions(transitions, state, rng)` → `Vec<Mutation>`
4. Repeat from step 2 with reactive mutations (depth-limited to 10)
5. Run derives once (FOV, lighting, spatial index)

## System Interface

Systems are free functions. Two signatures:

**Command handlers** — called from dispatch, take `&QueryContext` (pure read-only snapshot):
```rust
pub fn handle_melee(query: &QueryContext, x: i32, y: i32, rng: &mut ChaCha8Rng) -> Vec<Mutation>
```

**Notification handlers** — called from notify.rs, take `&GameState` (read-only, richer access):
```rust
pub fn on_enemy_hit(state: &GameState, idx: usize, damage: i32) -> Vec<Mutation>
```

Both return `Vec<Mutation>`. Neither mutates state.

## What state.rs Contains

| Concern | Role |
|---------|------|
| GameState struct definition | Central data |
| `apply_mutations(Vec<Mutation>) → Vec<StateTransition>` | Loop + collect transitions |
| `apply_one(&Mutation) → Option<StateTransition>` | Verify + apply + detect transition |
| Derives: `update_fov`, `update_lighting`, `rebuild_spatial_index` | Cross-cutting, after cascades settle |
| Logging: `log`, `log_typed` | Utility |
| Accessors | Used by UI/rendering |
| Save/load | Serialization |

state.rs has zero system logic, zero notification logic, zero dispatch logic.

## Borrow Checker

`QueryContext::from_fields()` borrows individual state fields excluding `rng`. Systems receive `(&QueryContext, &mut ChaCha8Rng)` as separate parameters. The borrow checker sees non-overlapping borrows.

For notification handlers that take `&GameState`: rng is passed as a separate parameter where needed.
