# Hotfixes

## 2026-03-08

- **World map Esc enters tile** — `Esc` was grouped with `Enter`/`m` and called `travel_to_tile`. Split: `Esc` now opens pause menu, `Enter`/`m`/`M` enter the tile.
- **Pause menu invisible on world map** — render loop returned early for world map before reaching pause menu render. Pause menu now rendered inside the world map branch.
- **Save list shows "day 20520"** — was displaying days since Unix epoch instead of a real date. Now shows `YYYY-MM-DD HH:MM` in local time.
- **Save list time off by 1 hour** — was using raw UTC seconds. Now applies local UTC offset via `/etc/timezone` or `TZ` env var.
- **Save list sort order** — was sorting by filename (hash), not by modification time. Now sorted newest-first by actual mtime.
- **Load from world map drops to tile** — loading a save made on the world map reset `world_map_view.open = false`, triggering tile generation. Added `saved_on_world_map` flag to `WorldState`; restored on load.
- **Build warnings** — suppressed `dead_code` warnings on serde deserialization fields (`params`, `main_questline`) and `schema_gen` structs.
