# Bug Report — 2026-02-15

## BUG-001: Auto-explore keeps targeting Dying Pilgrim after conversation

**Severity**: Medium  
**Status**: Open (partially addressed)  
**Component**: Auto-explore (`state.rs::auto_explore`), Movement (`systems/movement.rs`)

### Description

After talking to the Dying Pilgrim (completing objective 1 of "pilgrims_last_angle"), auto-explore continues to path toward and bump into the NPC, re-triggering dialogue.

### Root Cause

Two interacting issues:

1. **BFS pathing doesn't avoid the pilgrim's tile.** The `has_talked_npc_at_idx()` check only skips tiles where `npc.talked == true`. But `talked` stays `false` for the pilgrim because `has_pending_quest_objectives()` returns true (objectives 2 and 3 are still pending). So the BFS treats the pilgrim's tile as passable and routes through it.

2. **NPC targeting was fixed but pathing wasn't.** The `has_interacted_with_npc()` fix (commit 093cb04) correctly prevents the pilgrim from being selected as a *target*, but auto-explore still paths *through* the pilgrim's tile en route to other targets, triggering bump-to-talk on each pass.

### Relevant Code

```
src/game/state.rs:2388-2393  — NPC targeting (fixed, no longer targets pilgrim)
src/game/state.rs:2410-2412  — BFS neighbor filter uses has_talked_npc_at_idx()
src/game/state.rs:2484-2492  — has_talked_npc_at_idx() checks npc.talked (false for pilgrim)
src/game/systems/movement.rs:104-106 — talked only set true when no pending objectives
```

### Proposed Fix

In the BFS neighbor expansion, also skip tiles with NPCs we've already interacted with (not just `npc.talked`):

```rust
// In auto_explore BFS neighbor loop:
if self.has_talked_npc_at_idx(next_idx) || self.has_interacted_npc_at_idx(next_idx) {
    continue;
}
```

Where `has_interacted_npc_at_idx` checks `has_interacted_with_npc` for any NPC at that position.

---

## BUG-002: Quest objective 3 completable without objective 2

**Severity**: High  
**Status**: Open  
**Component**: Quest system (`quest.rs::ActiveQuest::on_npc_talked`)

### Description

The "pilgrims_last_angle" quest has 3 sequential objectives:
1. Talk to dying pilgrim
2. Recover pilgrim's cache (collect scripture_shard)
3. Return to dying pilgrim

Talking to the pilgrim a second time completes objective 3 even if objective 2 (collecting the scripture shard) hasn't been done. This breaks the intended quest flow.

### Root Cause

`ActiveQuest::on_npc_talked()` (quest.rs:277) finds the first uncompleted `TalkTo` objective matching the NPC and marks it complete. It does **not** check whether prior objectives in the sequence are completed:

```rust
// quest.rs:280-289
pub fn on_npc_talked(&mut self, npc_id: &str) {
    if let Some(def) = self.def() {
        for (i, obj) in def.objectives.iter().enumerate() {
            if let ObjectiveType::TalkTo { npc_id: target } = &obj.objective_type {
                if target == npc_id && !self.objectives[i].completed {
                    self.objectives[i].completed = true;  // No check on prior objectives
                    break;
                }
            }
        }
    }
}
```

After objective 1 is completed, the next uncompleted TalkTo for "dying_pilgrim" is objective 3. Since there's no sequential gate, it completes immediately on the next conversation.

### Proposed Fix

Add a sequential check — only allow completing objective `i` if all objectives `0..i` are already completed:

```rust
if target == npc_id && !self.objectives[i].completed {
    // Enforce sequential completion
    let prior_complete = self.objectives[..i].iter().all(|o| o.completed);
    if prior_complete {
        self.objectives[i].completed = true;
        break;
    }
}
```

This same check should be applied to all objective completion methods (`on_enemy_killed`, `on_item_collected`, `on_position_reached`, `on_aria_interfaced`) for consistency.

---

## BUG-003: Player spawn 5x5 clearing fails at map edges

**Severity**: Low  
**Status**: Partial fix applied  
**Component**: Map generation (`state.rs::GameState::new`)

### Description

The 5x5 floor clearing around player spawn (commit 093cb04) correctly avoids out-of-bounds access with boundary checks (`cx >= 1 && cy >= 1 && cx < width-1 && cy < height-1`), but if the spawn point `rooms[0]` is within 2 tiles of a map edge, the clearing is asymmetric — tiles beyond the boundary aren't cleared, potentially leaving walls adjacent to the player.

### Current Behavior

Map is 250×110. The clearing loop skips tiles at the map border. If `rooms[0]` is at e.g. `(1, 5)`, the clearing only extends 0 tiles to the left instead of 2.

### Proposed Fix

Clamp `rooms[0]` to at least 3 tiles from any map edge before using it as spawn point:

```rust
let (mut px, mut py) = rooms[0];
px = px.max(3).min(map.width as i32 - 4);
py = py.max(3).min(map.height as i32 - 4);
```

This ensures the full 5x5 clearing always fits. The room center is already a floor tile, so shifting by 1-2 tiles is safe.

---

## BUG-004: FOV is distance-based only — player can see through walls

**Severity**: High  
**Status**: Open  
**Component**: Renderer (`renderer/tiles.rs`), Lighting (`renderer/lighting.rs`)

### Description

The tile renderer determines visibility using `light_level > 80` (distance-based lighting), not the bracket-lib FOV result (`state.visible`). This means any tile within light radius is rendered as visible regardless of wall occlusion. The player can see rooms, enemies, and items through walls.

Entities (enemies, NPCs, items) correctly use `state.visible` for their visibility check, creating an inconsistency: tiles show through walls but entities on those tiles don't.

### Root Cause

```rust
// renderer/tiles.rs:53-54 — uses light level, not FOV
let visible = light_level > 80 || state.debug_god_view;
let revealed = state.revealed.contains(&idx) || state.debug_god_view;
```

Should be:
```rust
let visible = state.visible.contains(&idx) || state.debug_god_view;
```

The bracket-lib FOV (`state.visible`) is computed correctly via `compute_fov()` which calls `field_of_view()` with `is_opaque()` checking wall transparency. It's just not used by the tile renderer.

### Relevant Code

```
src/renderer/tiles.rs:53       — tile visibility: light_level > 80 (WRONG)
src/renderer/entities.rs:135   — entity visibility: state.visible (CORRECT)
src/game/map.rs:665-667        — is_opaque() correctly blocks walls
src/game/map.rs:689-693        — compute_fov() uses bracket-lib field_of_view
src/game/state.rs:1218-1234    — update_fov() computes state.visible correctly
```

### Proposed Fix

Replace light-level visibility with FOV check in tile renderer. Light level can still modulate brightness/color of visible tiles:

```rust
let in_fov = state.visible.contains(&idx) || state.debug_god_view;
let revealed = state.revealed.contains(&idx) || state.debug_god_view;

let span = if in_fov {
    self.render_visible_tile(state, x, y, idx, light_map, frame_count)
} else if revealed {
    self.render_revealed_tile(state, idx)
} else {
    Span::raw(" ")
};
```

---

## BUG-005: Look mode only shows one entity per tile

**Severity**: Low  
**Status**: Open  
**Component**: Inspect system (`game/inspect.rs::describe_at`)

### Description

When multiple entities occupy the same tile (e.g., an item on the ground next to an NPC, or two items stacked), the look cursor (`x` key) only describes the first one found. The priority chain is: player → enemy → NPC → item → light → inscription → tile. Anything after the first match is hidden.

### Root Cause

`describe_at()` (inspect.rs:97) uses early `return` after each entity type check:

```rust
if let Some(ei) = self.enemy_at(x, y) { return format!(...); }
if let Some(ni) = self.npc_at(x, y) { return format!(...); }
if let Some(item) = self.items().iter().find(...) { return format!(...); }
```

### Proposed Fix

Collect all descriptions into a `Vec<String>` and join them:

```rust
let mut descriptions = Vec::new();
if let Some(ei) = self.enemy_at(x, y) { descriptions.push(format!(...)); }
if let Some(ni) = self.npc_at(x, y) { descriptions.push(format!(...)); }
for item in self.items().iter().filter(|i| i.x == x && i.y == y) { descriptions.push(...); }
// ... etc
descriptions.join(" | ")
```

---

## BUG-006: Tutorial UI does not appear after spawn

**Severity**: Medium  
**Status**: Open  
**Component**: Tutorial system (`game/tutorial.rs`, `game/narrative_engine.rs`)

### Description

The tutorial overlay never appears. New players get no guidance on controls, mechanics, or objectives.

### Root Cause

Two disconnected systems:

1. **Real tutorial module** (`game/tutorial.rs`, 145 lines) loads messages from `data/tutorial.json` with trigger conditions, but is never called from the game loop.

2. **NarrativeEngine stub** (`game/narrative_engine.rs:202-205`) — `state.get_next_tutorial_message()` delegates to `TutorialProgress::get_next_message()` which is a placeholder that always returns `None`:

```rust
pub fn get_next_message(&self, _state: &GameState) -> Option<String> {
    None  // Placeholder implementation
}
```

3. **No initial trigger** — even if the tutorial system worked, `main.rs` only checks for tutorial messages after a player action (`update` returns `Some(true)`). There's no check on first render/spawn.

### Additional Issue

The tutorial content in `data/tutorial.json` may not be helpful enough for new players. Needs review of message content, trigger conditions, and sequencing.

### Proposed Fix

Wire `game/tutorial.rs` into `get_next_tutorial_message()` instead of the NarrativeEngine stub. Add an initial tutorial check before the first frame render in `main.rs`.

---

## BUG-007: Cannot progress past 2nd quest — lacking implementation

**Severity**: High  
**Status**: Open (by design — not yet implemented)  
**Component**: Quest system, World generation

### Description

After completing "pilgrims_last_angle" (quest 1), "the_broken_key" (quest 2) activates but cannot be completed. The quest requires:
1. Reach vitrified library ruins at (50, 50)
2. Collect `broken_saint_key`

Neither objective is achievable:

- **No location data**: The coordinates (50, 50) are arbitrary tile-map positions with no special generation. There's no "vitrified library ruins" structure spawned at any location.
- **No item spawn**: `broken_saint_key` exists in `items.json` but is not in any spawn table or loot table. It cannot be found in the world.
- **No quest-driven generation**: The `quest_constraints.json` defines that `pilgrims_last_angle` requires `vitrified_library_ruins` microstructure, but the constraint system doesn't drive actual world generation.

### Scope

This is a known gap from the Phase 5 audit (see `PHASE_5_IMPLEMENTATION_PLAN.md`). Remaining unimplemented quest infrastructure:
- Quest-driven location/item spawning
- ARIA interface system (needed for Act III quests)
- Missing NPCs: `the_architect`, `high_prism`, `custodian_iri_7`, `sable_of_the_seam`
- Boss encounter mechanics (Act IV)
- Prime Lens assembly system (Act IV)
