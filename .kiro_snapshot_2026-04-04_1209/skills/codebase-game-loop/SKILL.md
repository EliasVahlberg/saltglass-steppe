---
name: codebase-game-loop
description: The main game loop, turn processing, and system call order. Use when adding new per-turn logic, understanding execution order, or debugging turn-based behavior.
---

# Codebase: Game Loop

## Entry Point (`src/main.rs`)

```
run_main_game()
  └─ 'main loop
       ├─ Main menu loop → get (class_id, seed)
       └─ Game loop
            ├─ Tick animations / visual effects
            ├─ Handle pending UI actions (dialogue, trade, book, tutorial)
            ├─ terminal.draw(render)
            ├─ handle_input() → Action enum
            └─ update(state, action, ui) → Option<bool>
                 ├─ Some(true)  → continue
                 ├─ Some(false) → quit
                 └─ None        → return to main menu
```

## `update()` — Action Dispatch

`fn update(state: &mut GameState, action: Action, ui: &mut UiState) -> Option<bool>`

Key action handlers:
| Action | Handler |
|--------|---------|
| `Move(dx, dy)` | `state.try_move(dx, dy)` → `MovementSystem` |
| `EndTurn` | `state.end_turn()` |
| `Wait` | `state.wait_turn()` → `end_turn()` |
| `AutoExplore` | `state.auto_explore()` → BFS + `try_move` |
| `RangedAttack(x, y)` | `state.try_ranged_attack(x, y)` |
| `WorldMapTravel(wx, wy)` | `state.travel_to_tile_safe(wx, wy)` |
| `Save` / `Load` | `save::save_game` / `save::load_game` |
| `UseItem(idx)` | `state.use_item(idx)` |
| `Craft` | `state.craft(recipe_id)` |

## `state.end_turn()` — Turn Processing Order

```rust
pub fn end_turn(&mut self) {
    self.ensure_spatial_index();
    self.player.ap = self.player.max_ap;          // 1. Reset AP
    StatusEffectSystem.update(self);               // 2. Tick status effects
    self.player.psychic.tick();                    // 3. Tick psychic cooldowns
    self.player.skills.tick();                     // 4. Tick skill cooldowns
    self.player.light_system.update(&mut self.rng); // 5. Tick light system
    self.player.void_system.update(&mut self.rng);  // 6. Tick void system
    self.player.crystal_system.update(&mut self.rng); // 7. Tick crystal system
    self.tick_turn();                              // 8. Increment turn, tick effects/decoys
    self.update_enemies();                         // 9. Run AI (AiSystem)
    if self.world.storm.tick() {
        StormSystem::apply_storm(self);            // 10. Storm transformation (if triggered)
    }
    self.tick_time();                              // 11. Advance time of day / weather
    self.update_lighting();                        // 12. Recompute lighting
    self.update_fov();                             // 13. Recompute FOV
    self.check_dynamic_events();                   // 14. Dynamic events (stub — not yet re-implemented)
    self.emit(GameEvent::TurnEnded { turn: self.turn }); // 15. Emit TurnEnded
    self.process_events();                         // 16. Process event queue
}
```

## `process_events()` — Event Bus

```rust
fn process_events(&mut self) {
    // Loop up to 10 iterations to handle cascading events
    loop {
        let events = self.drain_events();
        if events.is_empty() { break; }
        for event in events {
            LootSystem.on_event(self, &event);   // drops loot on EnemyKilled
            QuestSystem.on_event(self, &event);  // updates quest progress
            self.handle_event(&event);           // logging + state updates
        }
    }
}
```

## `travel_to_tile_safe()` — World Map Travel

```rust
pub fn travel_to_tile_safe(&mut self, new_wx: usize, new_wy: usize) {
    // 1. Reject non-adjacent travel
    if !travel::is_adjacent(from, to) { return; }
    // 2. Apply travel cost (turns advance)
    let cost = travel::travel_cost(terrain, biome);
    self.turn += cost;
    // 3. Generate new tile (lazy)
    self.travel_to_tile(new_wx, new_wy);
    // 4. Find safe spawn position
    // 5. Update FOV + lighting
}
```

## `travel_to_tile()` — Lazy Tile Generation

Called on world map movement. Generates the tile if not yet visited:
1. Get world context (biome, terrain, elevation, POI, level) from `WorldMap`
2. Generate tile via `TerrainForgeGenerator::generate_tile_with_seed()`
3. Spawn enemies (biome spawn table, spatial distribution)
4. Spawn NPCs (biome table + quest-required NPCs)
5. Spawn items + chests
6. Place microstructures
7. Materialize terrain-forge feature markers
8. Spawn crafting stations (towns only)
9. Update player position, FOV, spatial index, lighting
10. Generate narrative fragments + biome content + template content (stubs — not yet re-implemented)

## `tick_turn()` — Per-Turn Bookkeeping

```rust
fn tick_turn(&mut self) {
    self.turn += 1;
    // Tick adaptations_hidden_turns
    // Tick triggered_effects (retain non-zero)
    // Tick decoys (retain non-zero)
    self.apply_light_effects(); // glare damage, item light interactions
}
```

## `tick_time()` — Time of Day

- Every 10 turns = 1 hour
- At dawn (hour 6) and dusk (hour 18): random weather change (Clear/Dusty/Sandstorm)
- Weather affects `effective_ambient_light()` which feeds into lighting

## Auto-Explore

BFS from player position to nearest:
1. Item matching pickup filter
2. Untalked NPC (not yet interacted via quest)
3. Unexplored tile

Stops if: enemy within detection range, dangerous tile (glass/glare) in path.

## IPC (Satellite Terminals)

After each action, `main.rs` sends `IpcMessage` to satellite terminal processes:
- `GameState` — hp, turn, storm, adaptations
- `InventoryUpdate` — items, equipped
- `LogEntry` — new messages since last update
- `DebugInfo` — position, enemy count, seed, world pos
