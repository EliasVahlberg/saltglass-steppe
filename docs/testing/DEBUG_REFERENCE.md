# Debug Reference

## Opening the Debug Console

Press `` ` `` (backtick) during gameplay to open/close the console.

Console controls:
- `Up / Down` — navigate command history (or cycle autocomplete suggestions)
- `Left / Right` — cycle autocomplete suggestions
- `Tab` — accept current suggestion
- `Enter` — execute command
- `Esc` or `` ` `` — close console

---

## Debug Commands

| Command | Effect |
|---|---|
| `show tile` | Enable god view — reveals entire map and all entities |
| `hide tile` | Disable god view |
| `sturdy` | Set player HP to 9999/9999 |
| `phase` | Toggle wall phasing (move through walls) |
| `flee` | Force-attempt to flee the current encounter |
| `complete_quest <quest_id>` | Complete all objectives for a quest (e.g. `complete_quest pilgrims_last_angle`) |
| `debug_info` | Print turn, position, HP, enemy count, storm state, seed, and memory usage to the game log |
| `save_debug [name]` | Save current game state to `debug_states/<name>.ron` (auto-names with timestamp if omitted) |
| `load_debug <name>` | Load a previously saved debug state |
| `list_debug` | List all saved debug states |
| `run_des <file>` | Run a DES scenario file and print pass/fail + log to the game log |
| `report_issue` | Open the in-game issue reporter (see below) |
| `help` | Print all commands to the game log |

---

## Debug Menu (F12)

Press `F12` to open the full debug menu overlay. Four tabs:

| Tab | Contents |
|---|---|
| Info | Turn, player position, HP, world seed, tile seed, world position, enemy count, item count, storm intensity/timer |
| Performance | FPS gauge, memory gauge, per-system timing metrics |
| States | List of saved debug state files |
| Commands | Quick reference of all console commands |

Navigate tabs with `Tab` / `Shift+Tab`. Close with `F12`.

---

## Getting Debug Information

To capture a snapshot of the current game state for a bug report:

1. Open the console (`` ` ``)
2. Run `debug_info` — key stats print to the game log
3. Run `save_debug <descriptive_name>` — saves full state to `debug_states/`
4. The `.ron` file can be attached to a bug report or shared for reproduction

---

## Reporting Issues In-Game

Type `report_issue` in the debug console. A 6-step guided form opens:

| Step | Field | Notes |
|---|---|---|
| 1 | Description | Brief summary of the issue |
| 2 | Reproduction steps | Add each step with `Enter`, blank `Enter` to advance |
| 3 | Expected behavior | What should have happened |
| 4 | Actual behavior | What actually happened |
| 5 | Severity | `Space` to cycle: Low → Medium → High → Critical |
| 6 | Category | `Space` to cycle: Gameplay, UI, Performance, Save, Combat, AI, Map, Other |

Review screen shows the full report before submission. `Enter` to submit, `Backspace` to go back, `Esc` to cancel.

On submit, two files are written:
- `issue_reports/<id>.json` — structured report
- `debug_states/<id>.ron` — full game state snapshot

A dev can reproduce by running `load_debug <id>` in the debug console.

**Severity guide:**
- `Critical` — crash, data loss, softlock
- `High` — major feature broken, blocks progress
- `Medium` — incorrect behavior, workaround exists
- `Low` — visual glitch, minor annoyance
