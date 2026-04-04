# Skill System & Menu Rework Plan

**Status**: Complete (2026-03-07)  
**Scope**: `src/game/skills.rs`, `src/ui/skills_menu.rs`, `src/ui/input.rs`  
**Depends on**: `SKILL_SYSTEM_ARCHITECTURE.md` (typed accessors, SkillCategory update)

---

## Overview

Two parallel changes:

1. **State layer** (`skills.rs`) — update data model to support the 7-category tree design
2. **UI layer** (`skills_menu.rs`) — replace flat list with a pannable 2D skill tree graph

These are independent. The state changes can land first; the UI rework can be done against the new state.

---

## Part 1 — State Layer (`skills.rs`)

### 1.1 Update `SkillCategory`

```rust
// Before (5 variants)
pub enum SkillCategory { Combat, Athletics, Survival, Crafting, Social }

// After (7 variants, matching SKILL_TREE_DESIGN.md)
pub enum SkillCategory {
    SaltAlchemy, Crafting, Social, Survival, Medical, MeleeCombat, RangedCombat,
}
```

Update `next_category` / `prev_category` cycle in `skills_menu.rs` to match.

### 1.2 Add fields to `SkillDef`

```rust
pub struct SkillDef {
    // existing fields unchanged...
    #[serde(default)]
    pub tree_parent: Option<String>,  // parent skill ID; None = root node
    #[serde(default)]
    pub blocked: bool,                // true = not yet implementable (storm, etc.)
    #[serde(default)]
    pub active: bool,                 // true = has an active use handler
}
```

No migration needed — `serde(default)` handles missing fields in existing JSON.

### 1.3 Add typed accessors to `SkillsState`

Replace all raw `passive_bonuses.get("key")` calls outside `skills.rs` with methods:

```rust
impl SkillsState {
    pub fn melee_accuracy_bonus(&self) -> f32 {
        self.passive_bonuses.get("melee_accuracy_bonus").copied().unwrap_or(0.0)
    }
    pub fn melee_damage_bonus(&self) -> f32 {
        self.passive_bonuses.get("melee_damage_bonus").copied().unwrap_or(0.0)
    }
    pub fn ranged_accuracy_bonus(&self) -> f32 {
        self.passive_bonuses.get("ranged_accuracy_bonus").copied().unwrap_or(0.0)
    }
    pub fn ranged_damage_bonus(&self) -> f32 {
        self.passive_bonuses.get("ranged_damage_bonus").copied().unwrap_or(0.0)
    }
    // Add one method per passive key as skills are implemented
}
```

Update the 4 raw lookups in `systems/combat.rs` to use these.

### 1.4 Add tree query helpers

Used by the UI to build the layout:

```rust
/// All skills that are direct children of `parent_id`
pub fn get_skill_children(parent_id: &str) -> Vec<&'static SkillDef> {
    SKILLS.values()
        .filter(|def| def.tree_parent.as_deref() == Some(parent_id))
        .collect()
}

/// Root skills for a category (no tree_parent)
pub fn get_category_roots(category: &SkillCategory) -> Vec<&'static SkillDef> {
    SKILLS.values()
        .filter(|def| &def.category == category && def.tree_parent.is_none())
        .collect()
}
```

---

## Part 2 — UI Layer (`skills_menu.rs`)

### 2.1 New `SkillsMenu` state

Replace the current list-based state with a graph-navigation state:

```rust
pub struct SkillsMenu {
    pub active: bool,
    pub category_idx: usize,          // index into CATEGORIES array
    pub cursor_id: String,            // currently focused skill ID
    pub pan_x: i32,                   // viewport offset in canvas chars
    pub pan_y: i32,
    pub detail_scroll: usize,         // scroll offset for detail panel
}
```

`CATEGORIES` is a fixed-order array of the 7 `SkillCategory` variants. `category_idx` replaces the old `selected_category` enum field.

Remove: `mode` (Skills/Abilities split), `selected_index`, `list_state`. The tree view shows all skills; abilities are shown in the detail panel for the selected skill.

### 2.2 Layout

```
┌──────────────────────────────────────────────────┬─────────────────────┐
│  Salt Alchemy                      [3 SP]  ←→ 2/7│  Crucible Tech      │
│                                                   │  ─────────────────  │
│  [Crucible Tech]──[Adaptation Tinctures]          │  Lv. 0/3            │
│       │                                           │                     │
│       └──────────[Poison Creation]                │  Desc text here     │
│                        │                          │                     │
│                  [Contact Poison] ⚠               │  Cost: 2 SP         │
│                                                   │  Prereqs: ✓         │
│  [Brine Distillation]──[Salt Preservation]        │                     │
│       │                                           │  [Enter] Upgrade    │
│       └──────────[Void Reagents]                  │                     │
│                        │                          │  Abilities:         │
│                  [Void Glass Synth]               │  (none)             │
│                                                   │                     │
├──────────────────────────────────────────────────┴─────────────────────┤
│  ↑↓←→: Move  HJKL: Pan  Tab: Category  Enter: Upgrade  Esc: Close      │
└─────────────────────────────────────────────────────────────────────────┘
```

- Left panel (70%): pannable skill tree canvas
- Right panel (30%): detail for focused skill
- Header: category name, SP count, category position indicator
- Footer: controls

### 2.3 Node layout algorithm

Computed once per category change, stored as `HashMap<String, (i32, i32)>` (canvas coordinates in character cells).

```
NODE_W = 22   // chars per node cell (includes box + padding)
NODE_H = 3    // rows per node cell
COL_GAP = 4   // extra chars between columns
ROW_GAP = 1   // extra rows between rows
```

**Algorithm** (depth-first, assigns row slots):

```
1. Get root nodes for category (sorted by name for stability)
2. For each root, recursively assign positions:
   - column = depth from root
   - row = next available row slot at this depth
3. x = column * (NODE_W + COL_GAP)
4. y = row * (NODE_H + ROW_GAP)
```

This is a simple left-to-right tree layout. No overlap detection needed because the tree structure is a DAG with at most one parent per node (enforced by `tree_parent: Option<String>`).

### 2.4 Node rendering

Each node is rendered as a box at its canvas position:

```
┌────────────────────┐
│  Crucible Tech     │   ← selected: reversed style
│  Lv.0/3  [2 SP]   │
└────────────────────┘
```

Node color:
- `Color::Yellow` — maxed out
- `Color::Green` — can upgrade (prereqs met, have SP)
- `Color::White` — known but locked (prereqs not met)
- `Color::DarkGray` — not yet reached (no prereq chain)
- `Color::Red` — blocked (`def.blocked == true`)

Node suffix glyphs:
- `⚠` — partial blocker
- `✗` — fully blocked
- `★` — active skill (has use handler)

### 2.5 Connection rendering

For each node with children, draw connections from the node's right edge to each child's left edge.

```
Single child:
  [Parent]────────────[Child]

Multiple children:
  [Parent]────────────[Child A]
      │
      └──────────────[Child B]
      │
      └──────────────[Child C]
```

Connection characters: `─`, `│`, `├`, `└`, `┬`

**Rendering approach**: render into a `Vec<Vec<char>>` canvas buffer sized to the full tree extent, then blit the visible viewport (offset by `pan_x`, `pan_y`) into the ratatui frame.

### 2.6 Viewport and panning

```rust
// Canvas → screen coordinate
fn canvas_to_screen(cx: i32, cy: i32, pan_x: i32, pan_y: i32) -> (i32, i32) {
    (cx - pan_x, cy - pan_y)
}
```

A node is visible if its canvas position falls within `[pan_x, pan_x + viewport_w)` × `[pan_y, pan_y + viewport_h)`.

**Auto-pan on cursor move**: when the cursor moves to a new node, check if the node's canvas position is within the viewport. If not, shift `pan_x`/`pan_y` to bring it into view (with a small margin).

**Manual pan**: HJKL keys shift `pan_x`/`pan_y` by `NODE_W` / `NODE_H` per press.

### 2.7 Cursor navigation

```
←  (Left)  → move to tree_parent of cursor node
→  (Right) → move to first child of cursor node
↑  (Up)    → move to previous sibling (same parent, same depth)
↓  (Down)  → move to next sibling (same parent, same depth)
```

"Sibling" = another node with the same `tree_parent`. Sorted by canvas Y position.

If no parent (at root), Left does nothing. If no children, Right does nothing.

### 2.8 Detail panel

Shows for the focused skill:
- Name, level, max level
- Description
- Upgrade cost and available SP
- Prerequisite status (each prereq listed with ✓/✗)
- Blocker reason if `blocked == true`
- Abilities unlocked by this skill (from `ABILITIES` where `required_skill == cursor_id`)

### 2.9 Input changes (`src/ui/input.rs`)

Add new key bindings for the skills menu:

| Key | Action |
|-----|--------|
| `h` / `H` | Pan left |
| `j` / `J` | Pan down |
| `k` / `K` | Pan up |
| `l` / `L` | Pan right |
| `←` `→` `↑` `↓` | Move cursor |
| `Tab` | Next category |
| `Shift+Tab` | Prev category |
| `Enter` | Upgrade focused skill |
| `Esc` | Close menu |

These bindings are only active when `skills_menu.active == true`.

---

## Part 3 — Implementation Phases

### Phase A — State changes ✓ DONE
1. Update `SkillCategory` enum (7 variants)
2. Add `tree_parent`, `blocked`, `active` to `SkillDef`
3. Add `get_skill_children()` and `get_category_roots()` helpers
4. Add typed accessor methods to `SkillsState`
5. Update `combat.rs` to use typed accessors
6. Created `data/skill_trees.json` with 35 skills across all 7 categories

### Phase B — UI rework ✓ DONE
1. Replace `SkillsMenu` struct fields
2. Implement node layout algorithm
3. Implement canvas buffer renderer (nodes + connections)
4. Implement viewport blit into ratatui frame
5. Implement cursor navigation
6. Implement auto-pan on cursor move
7. Implement detail panel
8. Update input handling

### Phase C — Polish
1. Smooth pan clamping (don't pan past tree bounds)
2. Category transition animation (optional)
3. Blocked skill tooltip on hover
4. Ability list in detail panel with use button

---

## What Does NOT Change

- `recalculate_passive_bonuses()` — untouched
- `upgrade_skill()` / `can_upgrade_skill()` — untouched
- `crafting.rs`, `status.rs`, `trading.rs` — untouched
- `abilities.json` structure — additive fields only
- Save format — `SkillsState` serialization unchanged (`tree_parent` etc. are on `SkillDef`, not state)

---

## Open Questions

1. **Multi-parent skills** (cross-tree prerequisites like Medicine Synthesis requiring Medicine Understanding): `tree_parent` is single-parent for layout purposes. Cross-tree prereqs are still in `prerequisites: Vec<SkillPrerequisite>`. The UI should show cross-tree prereqs in the detail panel with a different indicator (e.g., `⟵ requires: Medicine Understanding`), not as a drawn edge.

2. **Category root ordering**: roots within a category should be sorted consistently. Alphabetical by ID is simplest and stable.

3. **Canvas buffer vs. direct ratatui rendering**: the canvas buffer approach (render to `Vec<Vec<char>>` then blit) is simpler to implement than computing ratatui widget positions for each node. It also makes panning trivial. Downside: no ratatui styling on individual characters — use a `Vec<Vec<(char, Style)>>` buffer instead.
