# Scalability Audit

**Date:** 2024-12-14  
**Current Scale:** 50x22 map, ~7 enemies, ~10 items, ~7 NPCs, 14 effects

---

## Executive Summary

The codebase has two categories of scaling concerns:

**Performance Scaling** - O(n²) patterns that degrade at 10-100x entity counts
**Content Scaling** - Hardcoded logic that requires code changes to extend

The data-driven JSON architecture is excellent for defining *types* (enemy definitions, item properties). However, *behaviors* and *spawning* remain hardcoded, limiting content creators.

### Key Findings

| Category | What Scales Well | What Doesn't Scale |
|----------|------------------|-------------------|
| Performance | Tile storage, static data loading | Entity lookup, rendering, pathfinding |
| Content | Entity definitions (JSON) | Behaviors, spawn rules, effect logic |

---

## Critical Issues (Breaks at 10x scale)

### 1. Rendering Loop - O(map_size × entities)

**Location:** `main.rs` render function

**Current behavior:**
```rust
for y in 0..map.height {
    for x in 0..map.width {
        // For EACH tile, search ALL enemies, npcs, items
        if let Some(e) = state.enemies.iter().find(|e| e.x == x && e.y == y) { ... }
        if let Some(n) = state.npcs.iter().find(|n| n.x == x && n.y == y) { ... }
        if let Some(i) = state.items.iter().find(|i| i.x == x && i.y == y) { ... }
    }
}
```

**Complexity:** O(1100 tiles × (7 enemies + 7 npcs + 10 items)) = O(26,400) per frame  
**At 10x:** O(1100 × 240) = O(264,000) per frame  
**At 100x:** O(1100 × 2400) = O(2,640,000) per frame

**Desired behavior:**
- Build position→entity lookup maps once per frame
- Render loop does O(1) lookup per tile
- Complexity: O(map_size + entities) = O(1,124) per frame

**Solution:**
```rust
// Pre-build spatial index before render
let enemy_positions: HashMap<(i32, i32), &Enemy> = 
    state.enemies.iter().filter(|e| e.hp > 0).map(|e| ((e.x, e.y), e)).collect();

// In render loop - O(1) lookup
if let Some(e) = enemy_positions.get(&(x, y)) { ... }
```

---

### 2. Enemy AI Pathfinding - O(enemies × map_size)

**Location:** `state.rs` update_enemies()

**Current behavior:**
```rust
for i in 0..self.enemies.len() {
    // A* pathfinding for EVERY enemy EVERY turn
    let path = a_star_search(enemy_idx, player_idx, &self.map);
}
```

**Complexity:** O(7 enemies × 1100 tiles) = O(7,700) per turn  
**At 10x:** O(70 × 1100) = O(77,000) per turn  
**At 100x:** O(700 × 1100) = O(770,000) per turn

**Desired behavior:**
- Only pathfind for enemies within sight range of player
- Cache paths and only recalculate when player moves significantly
- Use simpler movement for distant enemies (move toward player direction)

**Solution:**
```rust
// Only pathfind if enemy is visible or near player
if dist < sight && self.visible.contains(&enemy_idx) {
    let path = a_star_search(...);
} else if dist < sight * 2 {
    // Simple directional movement for nearby-but-not-visible
    let dx = (px - ex).signum();
    let dy = (py - ey).signum();
}
// Distant enemies don't move
```

---

### 3. Entity Position Lookup - O(n) per query

**Location:** `state.rs` enemy_at(), npc_at()

**Current behavior:**
```rust
pub fn enemy_at(&self, x: i32, y: i32) -> Option<usize> {
    self.enemies.iter().position(|e| e.x == x && e.y == y && e.hp > 0)
}
```

**Called from:** try_move (2x), update_enemies (per enemy), render (per tile)

**Complexity:** O(n) per call, called O(map_size + enemies) times per turn

**Desired behavior:**
- O(1) lookup via spatial HashMap
- Maintained incrementally when entities move

**Solution:**
```rust
// Add to GameState
pub enemy_positions: HashMap<(i32, i32), usize>,

// Update when enemy moves
self.enemy_positions.remove(&(old_x, old_y));
self.enemy_positions.insert((new_x, new_y), idx);

// O(1) lookup
pub fn enemy_at(&self, x: i32, y: i32) -> Option<usize> {
    self.enemy_positions.get(&(x, y)).copied()
}
```

---

## High Priority Issues (Degrades at 100x scale)

### 4. Effect String Parsing - O(effects) per frame

**Location:** `effect.rs` parse_effect(), get_enemy_effects(), get_active_effects()

**Current behavior:**
```rust
// Called every frame for every visible enemy
pub fn get_enemy_effects(enemy_id: &str) -> Vec<VisualEffect> {
    all_effects()
        .iter()
        .filter(|e| e.target == "enemy" && e.condition.enemy_type.as_deref() == Some(enemy_id))
        .filter_map(|e| parse_effect(&e.effect))  // String parsing!
        .collect()
}
```

**Complexity:** O(14 effects × string_parse) per enemy per frame  
**At 100x effects:** O(1400 × string_parse) per enemy per frame

**Desired behavior:**
- Parse effects once at load time
- Index effects by target and condition for O(1) lookup

**Solution:**
```rust
// Pre-parsed effect storage
static PARSED_EFFECTS: Lazy<HashMap<String, Vec<VisualEffect>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for effect in all_effects() {
        let parsed = parse_effect(&effect.effect);
        map.entry(effect.target.clone())
           .or_insert_with(Vec::new)
           .push((effect.condition.clone(), parsed));
    }
    map
});
```

---

### 5. Dead Enemy Iteration

**Location:** `state.rs` update_enemies()

**Current behavior:**
```rust
for i in 0..self.enemies.len() {
    if self.enemies[i].hp <= 0 { continue; }  // Still iterates dead enemies
    ...
}
```

**Desired behavior:**
- Remove dead enemies from the list (or use a separate dead list)
- Or use a "live enemies" index

---

### 6. Item Pickup Iteration

**Location:** `state.rs` pickup_items()

**Current behavior:**
```rust
for (i, item) in self.items.iter().enumerate() {
    if item.x == px && item.y == py { ... }  // Checks ALL items
}
```

**Desired behavior:**
- Use spatial index for O(1) lookup at player position

---

## Medium Priority (Architectural Debt)

### 7. No Spatial Partitioning

The game has no spatial data structure for entities. All queries are linear scans.

**Desired behavior:**
- Single `SpatialIndex` struct managing all entity positions
- Provides O(1) point queries and O(k) range queries
- Updated incrementally as entities move

```rust
pub struct SpatialIndex {
    enemies: HashMap<(i32, i32), usize>,
    npcs: HashMap<(i32, i32), usize>,
    items: HashMap<(i32, i32), Vec<usize>>,  // Multiple items can stack
}

impl SpatialIndex {
    pub fn enemy_at(&self, x: i32, y: i32) -> Option<usize>;
    pub fn entities_in_range(&self, x: i32, y: i32, range: i32) -> Vec<EntityRef>;
    pub fn move_enemy(&mut self, idx: usize, from: (i32, i32), to: (i32, i32));
}
```

---

### 8. Duplicated EntityEffect Struct

**Location:** `enemy.rs` and `item.rs` both define identical `EntityEffect`

**Desired behavior:**
- Single definition in a shared module (e.g., `effect.rs`)
- Import where needed

---

### 9. Hardcoded Wall Types

**Location:** `map.rs` random_wall_type()

```rust
fn random_wall_type(rng: &mut ChaCha8Rng) -> String {
    let types = ["sandstone", "shale", "salt_crystal"];  // Hardcoded!
    ...
}
```

**Desired behavior:**
- Read wall type IDs from WALL_DEFS keys
- Or add a "spawnable" flag to wall definitions

---

## Low Priority (Fine for Current Scale)

### FOV Computation
- O(FOV_RANGE²) = O(64) per move - bounded and acceptable

### Static Data Loading
- Lazy HashMap initialization - loads once, O(1) lookup thereafter

### Message Log
- Capped at 5 messages - no scaling issue

### Tile Storage
- Flat Vec with O(1) index access - optimal

---

## Scaling Projections

| Metric | Current | 10x | 100x | 1000x |
|--------|---------|-----|------|-------|
| Map tiles | 1,100 | 11,000 | 110,000 | 1,100,000 |
| Enemies | 7 | 70 | 700 | 7,000 |
| Items | 10 | 100 | 1,000 | 10,000 |
| NPCs | 7 | 70 | 700 | 7,000 |
| Effects | 14 | 140 | 1,400 | 14,000 |

### Render Performance (per frame)
| Scale | Current O(n²) | With Spatial Index O(n) |
|-------|---------------|-------------------------|
| 1x | 26K ops | 1.1K ops |
| 10x | 264K ops | 11K ops |
| 100x | 2.6M ops | 110K ops |

### Enemy AI Performance (per turn)
| Scale | Current (all pathfind) | With culling |
|-------|------------------------|--------------|
| 1x | 7.7K ops | ~1K ops |
| 10x | 77K ops | ~5K ops |
| 100x | 770K ops | ~10K ops |

---

## Recommended Implementation Order

1. **Spatial Index for Entities** (High impact, moderate effort)
   - Fixes rendering, enemy_at, npc_at, pickup_items
   - Single change improves multiple systems

2. **Pre-parse Effects** (Medium impact, low effort)
   - Parse once at load time
   - Index by target type

3. **Enemy AI Culling** (High impact, low effort)
   - Only pathfind visible/nearby enemies
   - Simple directional movement for others

4. **Dead Enemy Cleanup** (Low impact, low effort)
   - Remove or flag dead enemies
   - Reduces iteration overhead

5. **Deduplicate EntityEffect** (Low impact, trivial effort)
   - Move to shared module
   - Clean up imports

---

---

## Content Modularity Issues

These issues don't affect performance but require code changes to add new content types.

### M1. Hardcoded Spawn Configuration

**Location:** `state.rs` GameState::new()

```rust
// Hardcoded item spawn list
let spawn_items = ["storm_glass", "storm_glass", "brine_vial", ...];

// Hardcoded NPC placement
if rooms.len() > 3 {
    npcs.push(Npc::new(npc_room.0, npc_room.1, "mirror_monk"));
}
```

**Problem:** Adding new items/NPCs to spawn requires editing Rust code.

**Desired behavior:** Data-driven spawn tables

```json
// data/spawn_tables.json
{
  "default": {
    "items": [
      {"id": "storm_glass", "weight": 4},
      {"id": "brine_vial", "weight": 3},
      {"id": "angle_lens", "weight": 1, "room": "last"}
    ],
    "enemies": [
      {"id": "mirage_hound", "weight": 3},
      {"id": "glass_beetle", "weight": 2, "min_room": 3}
    ],
    "npcs": [
      {"id": "mirror_monk", "room": "late"},
      {"id": "salt_hermit", "room": "any", "chance": 0.3}
    ]
  }
}
```

---

### M2. Hardcoded Item Effect Logic

**Location:** `state.rs` use_item()

```rust
if def.heal > 0 { ... }
if def.reduces_refraction > 0 { ... }
if def.suppresses_adaptations { ... }
if def.reveals_map { ... }
```

**Problem:** Each effect type has bespoke code. Adding "grants_temporary_flight" requires new code.

**Desired behavior:** Effect handler registry

```rust
// Effect handlers registered by type
type EffectHandler = fn(&mut GameState, &ItemDef) -> bool;

static EFFECT_HANDLERS: Lazy<HashMap<&str, EffectHandler>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("heal", handle_heal as EffectHandler);
    m.insert("reduce_refraction", handle_reduce_refraction);
    m.insert("suppress_adaptations", handle_suppress);
    m
});

// In use_item - iterate registered handlers
for (effect_type, handler) in EFFECT_HANDLERS.iter() {
    if def.has_effect(effect_type) {
        handler(self, def);
    }
}
```

Or fully data-driven with a simple effect DSL:
```json
{
  "id": "brine_vial",
  "on_use": [
    {"effect": "heal", "value": 5},
    {"effect": "log", "message": "The brine soothes your wounds."}
  ]
}
```

---

### M3. Hardcoded Enemy Behaviors

**Location:** `state.rs` update_enemies()

```rust
// Each behavior is inline code
if def.requires_saint_key && has_saint_key { continue; }
if def.flees_adapted_players && adaptation_count >= 2 { /* flee logic */ }
if def.reflects_damage { /* reflection logic */ }
```

**Problem:** Adding "teleports_when_low_hp" requires new code in update_enemies().

**Desired behavior:** Behavior component system

```json
// In enemies.json
{
  "id": "refraction_wraith",
  "behaviors": [
    {"type": "flee_if", "condition": "player_adaptations >= 2"},
    {"type": "passive_if", "condition": "player_has_item:saint_key"},
    {"type": "on_hit", "action": "reflect_damage", "percent": 25}
  ]
}
```

```rust
// Behavior evaluator
pub fn evaluate_behavior(enemy: &Enemy, ctx: &BehaviorContext) -> EnemyAction {
    for behavior in &enemy.def().behaviors {
        if behavior.condition_met(ctx) {
            return behavior.action();
        }
    }
    EnemyAction::Default
}
```

---

### M4. Hardcoded Adaptation Effects

**Location:** Scattered across `state.rs`

```rust
// In try_move - glass damage
if self.has_adaptation(Adaptation::Saltblood) { /* immune */ }

// In combat
if self.has_adaptation(Adaptation::Sunveins) { dmg += 2; }
```

**Problem:** Adaptations are a Rust enum. Adding new adaptations requires:
1. Add variant to enum
2. Add to available list in check_adaptation_threshold()
3. Add effect logic wherever relevant

**Desired behavior:** Data-driven adaptations

```json
// data/adaptations.json
{
  "adaptations": [
    {
      "id": "saltblood",
      "name": "Saltblood",
      "description": "Immune to glass damage",
      "effects": [
        {"type": "immunity", "damage_source": "glass"}
      ]
    },
    {
      "id": "sunveins", 
      "name": "Sunveins",
      "effects": [
        {"type": "damage_bonus", "value": 2}
      ]
    }
  ]
}
```

---

### M5. No Event System

**Current:** Direct state mutation scattered throughout code

```rust
// In try_move
self.player_hp -= dmg;
self.log("...");
self.trigger_effect("...");
self.check_adaptation_threshold();
```

**Problem:** Adding reactions to events (achievements, sound triggers, UI updates) requires editing every event source.

**Desired behavior:** Event bus for decoupled communication

```rust
pub enum GameEvent {
    PlayerDamaged { amount: i32, source: DamageSource },
    EnemyKilled { enemy_id: String, position: (i32, i32) },
    ItemPickedUp { item_id: String },
    AdaptationGained { adaptation_id: String },
}

// Systems subscribe to events
impl GameState {
    pub fn emit(&mut self, event: GameEvent) {
        self.event_queue.push(event);
    }
    
    pub fn process_events(&mut self) {
        for event in self.event_queue.drain(..) {
            // Logging system
            self.log_system.handle(&event);
            // Achievement system
            self.achievements.handle(&event);
            // Visual effects system
            self.effects.handle(&event);
        }
    }
}
```

---

### M6. Tile Types Not Extensible

**Location:** `map.rs` Tile enum

```rust
pub enum Tile {
    Floor,
    Wall { id: String, hp: i32 },
    Glass,
}
```

**Problem:** Adding "Water" or "Lava" tiles requires enum changes and updating all match statements.

**Desired behavior:** Data-driven tiles

```json
// data/tiles.json
{
  "tiles": [
    {"id": "floor", "glyph": ".", "walkable": true, "transparent": true},
    {"id": "wall", "glyph": "#", "walkable": false, "transparent": false, "breakable": true},
    {"id": "glass", "glyph": "*", "walkable": true, "damage_on_enter": 1, "effects": ["increase_refraction"]},
    {"id": "water", "glyph": "~", "walkable": true, "movement_cost": 2},
    {"id": "lava", "glyph": "≈", "walkable": true, "damage_on_enter": 5, "color": "Red"}
  ]
}
```

---

## Modularity Improvement Priority

| Issue | Impact | Effort | Priority |
|-------|--------|--------|----------|
| M1. Spawn Tables | High - enables level design | Low | **1** |
| M3. Enemy Behaviors | High - enables unique enemies | Medium | **2** |
| M2. Item Effects | Medium - enables new items | Medium | **3** |
| M5. Event System | High - enables many features | High | **4** |
| M4. Adaptations | Medium - limited content type | Medium | **5** |
| M6. Tile Types | Low - rarely need new tiles | Medium | **6** |

---

## Combined Recommendations

### Phase 1: Quick Wins (1-2 days)
1. **Spatial Index** - Performance fix, enables larger maps
2. **Spawn Tables** - Content fix, enables level designers
3. **Pre-parse Effects** - Performance fix, simple change

### Phase 2: Behavior System (3-5 days)
4. **Enemy Behavior DSL** - Content fix, major extensibility gain
5. **Item Effect Handlers** - Content fix, pairs with behaviors
6. **Enemy AI Culling** - Performance fix

### Phase 3: Architecture (1-2 weeks)
7. **Event System** - Enables achievements, sound, decoupled systems
8. **Data-driven Adaptations** - Complete the data-driven vision
9. **Data-driven Tiles** - Full content extensibility

---

## Conclusion

The codebase has a solid foundation with data-driven entity *definitions*. The next evolution is data-driven *behaviors* and *spawning*.

**For performance:** Implement spatial indexing first - it's high impact and touches multiple systems.

**For content creation:** Implement spawn tables first - it's low effort and immediately useful for level design. Then tackle the behavior system to enable unique enemy designs without code changes.

The event system is the largest architectural change but provides the most long-term value for adding features like achievements, sound, tutorials, and analytics without touching core game logic.
