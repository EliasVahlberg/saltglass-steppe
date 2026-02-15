# Bug Report — 2026-02-15

## BUG-001: Auto-explore keeps targeting Dying Pilgrim after conversation

**Severity**: Medium  
**Status**: Open  
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
