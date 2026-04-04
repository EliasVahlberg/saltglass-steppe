---
name: codebase-systems
description: All game systems (combat, AI, movement, loot, quest, status, storm) — their responsibilities, event handling, and extension points. Use when adding new gameplay mechanics, modifying combat, AI behavior, or quest progression.
---

# Codebase: Game Systems

**Location**: `src/game/systems/` (7 modules)

All systems implement the `System` trait:
```rust
pub trait System {
    fn update(&self, state: &mut GameState);
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
```

Systems are stateless — all data lives in `GameState`.

---

## CombatSystem (`systems/combat.rs`)

Called from `MovementSystem` when player bumps an enemy, or directly for ranged attacks.

**Melee flow**:
```
try_move(dx, dy) → handle_enemy_combat()
  → CombatSystem::attack_melee(state, enemy_idx)
    → roll_attack() — hit/miss/crit (uses mock_combat_hit if set)
    → calc_damage() — base + weapon + adaptation modifiers (uses mock_combat_damage if set)
    → apply damage to enemy.hp
    → if hp <= 0: process_enemy_death()
         → award XP (gain_xp)
         → handle split_on_death behavior
         → trigger on_death visual effects
         → emit EnemyKilled { enemy_id, x, y }
```

**Key functions** (in `src/game/combat.rs` — pure functions, no GameState):
- `roll_attack(attacker_stats, defender_stats, rng) -> AttackResult`
- `calc_damage(weapon, stats, rng) -> i32`
- `default_weapon() -> WeaponDef`

**Adding combat mechanics**: modify `roll_attack()` or `calc_damage()` in `src/game/combat.rs`.

---

## AiSystem (`systems/ai.rs`)

Called from `state.update_enemies()` in `end_turn()`.

**Strategy pattern via behavior registry**:
```rust
pub trait AiBehavior: Send + Sync {
    fn execute(&self, entity_idx: usize, state: &mut GameState) -> bool;
}

static BEHAVIOR_REGISTRY: Lazy<HashMap<&str, Box<dyn AiBehavior>>> = ...;
```

**Built-in behaviors**:
| `behavior_id` | Behavior |
|---|---|
| `standard_melee` | Chase player, melee attack when adjacent |
| `ranged_only` | Keep distance, ranged attack |
| `suicide_bomber` | Explode on contact with player |
| `healer` | Heal nearby allies |

**Adding a behavior**:
1. Create struct implementing `AiBehavior`
2. Register in `BEHAVIOR_REGISTRY`
3. Set `behavior_id` in `data/enemies.json`

Uses `bracket_pathfinding` for A* pathfinding.

---

## MovementSystem (`systems/movement.rs`)

Called via `state.try_move(dx, dy)`.

**Priority order** (bump resolution):
1. NPC at target → `handle_npc_interaction()` → dialogue, trade, quest events
2. Enemy at target → `handle_enemy_combat()` → `CombatSystem::attack_melee()`
3. Wall at target → wall break check (if player has `glass_pick`)
4. Open tile → `handle_movement()` → move player, pickup items, tile effects

**Tile effects on movement**:
- `Glass` → refraction gain
- `Glare` → light damage (currently disabled pending balance)
- `WorldExit` → prompt world map travel

**Item pickup**: `pickup_items()` — picks up all items at player position, emits `ItemPickedUp`.

---

## LootSystem (`systems/loot.rs`)

**Reactive only** — no `update()` logic. Responds to `EnemyKilled`:

```
EnemyKilled { enemy_id, x, y }
  → get_enemy_def(enemy_id).loot_table
  → weighted roll on loot table
  → spawn Item at (x, y)
  → log "The enemy drops X."
```

---

## QuestSystem (`systems/quest.rs`)

**Reactive only** — no `update()` logic. Responds to game events and updates `player.quest_log`:

| Event | Quest action |
|-------|-------------|
| `EnemyKilled { enemy_id }` | `on_enemy_killed(enemy_id)` |
| `ItemPickedUp { item_id }` | `on_item_collected(item_id)` |
| `PlayerMoved { to_x, to_y }` | `on_position_changed(x, y)` |
| `NpcTalkedTo { npc_id }` | `on_npc_talked(npc_id)` |
| `InteractableUsed { id }` | `on_interact(id)` |
| `InteractableExamined { id }` | `on_examine(id)` |
| `AriaInterfaced { item_id }` | `on_aria_interfaced(item_id)` |
| `TurnEnded` | `on_turn_passed()` |

After each event, calls `check_auto_complete()` → emits `QuestCompleted { quest_id }` for any newly completed quests.

---

## StatusEffectSystem (`systems/status.rs`)

Called in `end_turn()` before AI runs.

- Ticks duration on all `player.status_effects`
- Applies per-turn effects (poison damage, burn, etc.)
- Removes expired effects
- Emits `StatusEffectExpired` when effect ends

---

## StormSystem (`systems/storm.rs`)

Called in `end_turn()` when `state.world.storm.tick()` returns `true`.

**Storm application**:
1. Log storm arrival
2. Apply refraction gain to player (`intensity * refraction_multiplier()`)
3. Check adaptation threshold (may unlock new adaptation)
4. Apply map transformations based on `StormEditType`:
   - `Glass` — convert floor tiles to glass
   - `Rotate` — rotate map section
   - `Swap` — swap two areas
   - `Mirror` — mirror a section
   - `Fracture` — create cracks
   - `Crystallize` — create crystal formations
   - `Vortex` — spiral pattern
5. Spawn glass wraiths (count based on intensity + refraction)
6. Emit `StormArrived { intensity }`

**Storm timing**: `Storm::forecast(rng)` generates next storm countdown. `storm.tick()` decrements and returns `true` when it fires.

---

## Key Events Reference

```rust
pub enum GameEvent {
    PlayerDamaged { amount: i32, source: String },
    PlayerHealed { amount: i32 },
    EnemyKilled { enemy_id: String, x: i32, y: i32 },
    ItemPickedUp { item_id: String },
    ItemUsed { item_id: String },
    AdaptationGained { name: String },
    StormArrived { intensity: u8 },
    LevelUp { level: u32 },
    QuestCompleted { quest_id: String },
    NpcTalkedTo { npc_id: String },
    DialogueStarted { npc_id: String },
    InteractableUsed { interactable_id: String },
    InteractableExamined { interactable_id: String },
    AriaInterfaced { item_id: String },
    TradeCompleted { npc_id: String },
    FactionReputationChanged { faction_id: String, delta: i32 },
    StatusEffectApplied { effect_id: String, duration: i32 },
    VoidExposureChanged { level: VoidExposureLevel },
    CrystalResonanceChanged { frequency: String },
    TurnEnded { turn: u32 },
    PlayerMoved { from_x: i32, from_y: i32, to_x: i32, to_y: i32 },
}
```

---

## Adding a New System

1. Create `src/game/systems/my_system.rs`
2. Implement `System` trait
3. Add `pub mod my_system;` to `systems/mod.rs`
4. Call `MySystem.update(self)` in `state.end_turn()` at appropriate point
5. Add any new events to `GameEvent` enum in `src/game/event.rs`
6. Write DES scenario to test
