# Hotfixes

## 2026-03-08 (session 3 — cleanup + settlement visuals)

- **Dead code removed** — `src/bin/bsp-test.rs`, `cellular-automata-test.rs`, `dungeon-test-tool.rs` deleted (not called by any script or test). `tests/scenarios/broken/`, `state_refactor/`, 5 `.disabled` scenarios, `bug_fixes_test.yaml`, `data/settlement_config.json`, `data/skill_trees.schema.json` all removed.
- **`interaction_system_test` failing** — 12 interactable entries (`table`, `chair`, `bed`, `counter`, `shelf`, `workbench`, `altar`, `throne`, `fountain`, `statue`, `lamp`, `banner`) were missing the required `interact` field in their `messages` object. Added short interact messages to each.
- **Clippy errors blocking `cargo test`** — `src/game/generation/algorithm.rs` lines 522/531 used `3.14` as a test float value; clippy flagged it as an approximate PI constant (error, not warning). Changed test value to `1.5`.
- **Natural terrain bleeding into settlements** — tile generation stamped buildings directly onto noise-generated terrain, leaving natural walls inside town boundaries. Added `clear_settlement_footprint()` called before `stamp_settlement()`: computes the union bounding box of all buildings, expands by 6 tiles, floors all natural walls inside.
- **Settlement clearing was a rectangle** — `clear_settlement_footprint` used a simple bounding-box rectangle. Replaced with a distance-field approach: each map cell is cleared if its Euclidean distance to the nearest building footprint is ≤ 8 tiles. Result is the union of rounded rectangles — an organic polygon that follows the actual building layout.
- **Man-made tiles visually identical to natural terrain** — all man-made walls used `#` (same as `sandstone`) and all man-made floors used `.` (same as `dry_soil`). Updated `data/map_elements.json` with structured geometric glyphs: walls → `╪` `▩` `▬` `▐` `╫`; floors → `─` `▦` `▤` `+` `╌`. Colors nudged brighter on wood/stone.

- **World map Esc enters tile** — `Esc` was grouped with `Enter`/`m` and called `travel_to_tile`. Split: `Esc` now opens pause menu, `Enter`/`m`/`M` enter the tile.
- **Pause menu invisible on world map** — render loop returned early for world map before reaching pause menu render. Pause menu now rendered inside the world map branch.
- **Save list shows "day 20520"** — was displaying days since Unix epoch instead of a real date. Now shows `YYYY-MM-DD HH:MM` in local time.
- **Save list time off by 1 hour** — was using raw UTC seconds. Now applies local UTC offset via `/etc/timezone` or `TZ` env var.
- **Save list sort order** — was sorting by filename (hash), not by modification time. Now sorted newest-first by actual mtime.
- **Load from world map drops to tile** — loading a save made on the world map reset `world_map_view.open = false`, triggering tile generation. Added `saved_on_world_map` flag to `WorldState`; restored on load.
- **Build warnings** — suppressed `dead_code` warnings on serde deserialization fields (`params`, `main_questline`) and `schema_gen` structs.
- **Corrupt save crash** — `rebuild_spatial_index`/`update_lighting` could panic on malformed state. Wrapped in `catch_unwind`; error shown in load menu with red border instead of crashing.
- **Multiple failed saves show wrong status** — `failed_save_index: Option<usize>` in `MainMenuState` could only track one failure at a time. Replaced with `saves/meta.json` — persistent per-save status (`ok`/`hash_mismatch`/`corrupt`) written by `save_game`/`load_game`, read by `list_saves`. `SaveInfo.valid: bool` replaced with `SaveInfo.status: SaveStatus`. `failed_save_index` removed.
- **Corrupt/tampered status lost on menu navigation** — status was in-memory only; navigating away cleared it. Now persisted in `saves/meta.json` and survives restarts.
- **Load menu slow with many saves** — `list_saves()` was reading and MD5-hashing every `.ron` file on every menu open. Now trusts `Ok` entries in meta and skips the file read entirely; only re-reads files with no meta entry or `hash_mismatch` status.
- **Loaded/tile-test sessions missing visual effects and tutorial** — `MenuAction::LoadGame` and `MenuAction::TileTest` ran a simplified game loop without visual effects ticking or tutorial checks. Extracted unified `run_game_session()` in `src/session.rs`; all three session paths (new game, load, tile test) now use it.
