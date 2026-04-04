---
name: codebase-save-load
description: Save/load system architecture, versioning, and migration. Use when modifying GameState fields that affect serialization, implementing save migrations, or debugging save/load issues.
---

# Codebase: Save/Load System

**Location**: `src/game/save.rs`

## Architecture

Saves use a versioned envelope pattern. `GameState` is never serialized directly — it's wrapped in `SaveFile` which carries a schema version.

```rust
// Serialization envelope (borrows state)
#[derive(Serialize)]
struct SaveFile<'a> {
    version: u32,
    state: &'a GameState,
}

// Deserialization envelope (owns state)
#[derive(Deserialize)]
struct SaveFileOwned {
    version: u32,
    state: GameState,
}

pub const SAVE_VERSION: u32 = 1;
```

## API

```rust
// Save
pub fn save_game(state: &GameState, path: impl AsRef<Path>) -> Result<(), String>

// Load
pub fn load_game(path: impl AsRef<Path>) -> Result<GameState, String>
```

## Save Flow

```
save_game(state, "savegame.ron")
  → ron::to_string(SaveFile { version: SAVE_VERSION, state })
  → fs::write(path, data)
```

## Load Flow

```
load_game("savegame.ron")
  → fs::read_to_string(path)
  → ron::from_str::<SaveFileOwned>(data)
  → version check: file.version != SAVE_VERSION → Err("Save version mismatch: ...")
  → state.rebuild_spatial_index()
  → state.update_lighting()
  → Ok(state)
```

## Version Mismatch

Returns a descriptive error: `"Save version mismatch: file is v{X}, game expects v{Y}. This save is incompatible with the current version."`

No automatic migration yet — planned for future (see ROADMAP.md Tier 1).

## Serialization Format

RON (Rusty Object Notation) via the `ron` crate. Human-readable, supports Rust enums natively.

## What Is and Isn't Saved

**Saved** (standard serde fields):
- `player: PlayerState` — all player data
- `world: WorldState` — map, enemies, npcs, items, chests, storm, etc.
- `narrative: NarrativeEngine` — tutorial progress, story flags
- `visible`, `revealed` — FOV state
- `messages`, `turn`, `rng`, `triggered_effects`, `decoys`
- `wait_counter`, `map_features`, `seed`

**Not saved** (`#[serde(skip)]`):
- `light_map` — recomputed via `update_lighting()` on load
- Spatial indices (`enemy_positions`, etc.) — rebuilt via `rebuild_spatial_index()` on load
- `event_queue` — cleared between turns
- `mock_combat_hit/damage` — DES-only
- `pending_trade/dialogue/aria_dialogue/book_open` — transient UI state
- `meta: MetaProgress` — loaded separately from its own file
- Debug flags

## Wiring in `main.rs`

```rust
const SAVE_FILE: &str = "savegame.ron";

Action::Save => match save::save_game(state, SAVE_FILE) {
    Ok(_) => state.log("Game saved."),
    Err(e) => state.log(format!("Save failed: {}", e)),
},
Action::Load => match save::load_game(SAVE_FILE) {
    Ok(loaded) => { *state = loaded; state.log("Game loaded."); }
    Err(e) => state.log(format!("Load failed: {}", e)),
},
```

## Legacy API

`GameState` also has direct `save(path)` / `load(path)` methods (no version envelope). These are kept for backward compatibility but `save::save_game` / `save::load_game` should be used instead.

## When to Bump SAVE_VERSION

Bump `SAVE_VERSION` in `save.rs` whenever:
- Adding a field without `#[serde(default)]`
- Removing or renaming a field
- Changing a field's type

If adding a field with `#[serde(default)]`, no bump needed — old saves will deserialize with the default value.

## Future: Migration Functions

Planned pattern (not yet implemented):
```rust
fn migrate_v1_to_v2(state: GameState) -> GameState { ... }

// In load_game:
match file.version {
    1 => Ok(migrate_v1_to_v2(file.state)),
    SAVE_VERSION => Ok(file.state),
    v => Err(format!("Unknown version: {v}")),
}
```

## Tests

`save.rs` includes two tests:
- `save_load_roundtrip_preserves_state` — verifies position and turn survive round-trip
- `load_rejects_wrong_version` — verifies version mismatch returns error
