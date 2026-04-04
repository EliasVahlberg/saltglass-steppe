---
name: codebase-ui
description: UI screens, menus, input handling, and UiState structure. Use when adding new UI screens, modifying input handling, or understanding how UI state flows into game actions.
---

# Codebase: UI System

**Location**: `src/ui/` (22 modules)

## UiState (`src/ui/input.rs`)

Central UI state struct. Passed to `render()` and `handle_input()` each frame.

```rust
pub struct UiState {
    // Menus (each has .active, .open(), .close(), .toggle())
    pub inventory_menu: InventoryMenu,
    pub quest_log: QuestLogMenu,
    pub crafting_menu: CraftingMenu,
    pub pause_menu: PauseMenu,
    pub debug_menu: DebugMenu,
    pub debug_console: DebugConsole,
    pub wiki_menu: WikiMenu,
    pub trade_menu: TradeMenu,
    pub psychic_menu: PsychicMenu,
    pub faction_menu: FactionMenu,
    pub void_menu: VoidMenu,
    pub crystal_menu: CrystalMenu,
    pub light_menu: LightMenu,
    pub skills_menu: SkillsMenu,
    pub world_map_view: WorldMapView,
    pub issue_reporter: IssueReporter,
    pub aria_interface: AriaInterface,
    pub book_reader: BookReader,

    // Overlays
    pub dialog_box: DialogBox,
    pub tutorial_message: Option<(String, String)>,  // (id, text)
    pub chest_ui: Option<ChestUI>,

    // Look mode
    pub look_mode: LookMode,

    // Camera
    pub camera_x: f32,
    pub camera_y: f32,

    // Targeting
    pub target_enemy: Option<usize>,

    // Misc
    pub frame_count: u64,
    pub show_controls: bool,
}
```

## Screen Priority (render order in `main.rs`)

Fullscreen menus (return early, nothing else renders):
1. `aria_interface` — ARIA terminal interface
2. `trade_menu` — trading with NPCs
3. `inventory_menu` — inventory management
4. `chest_ui` — chest interaction
5. `quest_log` — quest log
6. `crafting_menu` — crafting
7. `wiki_menu` — in-game wiki
8. `psychic_menu` — psychic abilities
9. `faction_menu` — faction status
10. `void_menu` — void energy
11. `crystal_menu` — crystal resonance
12. `light_menu` — light manipulation
13. `skills_menu` — skill tree
14. `world_map_view` — world map

Death screen (if `player.hp <= 0`).

Main game view (default):
- Map + look mode description
- Bottom panel (event log)
- Side panel (stats + gear)
- Target HUD (if enemy targeted)

Overlays (rendered on top of main view):
- `pause_menu`
- `debug_console`
- `debug_menu`
- `issue_reporter`
- `tutorial_message` (centered popup)
- `dialog_box` (highest priority)
- `book_reader`

## Input Handling (`src/ui/input.rs`)

`handle_input(ui, state) -> Result<Action>`

Input is context-sensitive — the active menu determines which keys are handled.

Key bindings (main game):
| Key | Action |
|-----|--------|
| Arrow keys / hjkl / numpad | `Move(dx, dy)` |
| `.` / Space | `Wait` |
| `e` | `EndTurn` |
| `a` | `AutoExplore` |
| `i` | `OpenInventory` |
| `q` | `OpenQuestLog` |
| `c` | `OpenCrafting` |
| `m` | `OpenWorldMap` |
| `l` | `EnterLook` |
| `>` | `UseStairs` |
| `s` | `Save` |
| `L` | `Load` |
| `Esc` | `OpenPauseMenu` |
| `~` | Debug console toggle |
| `f` | Ranged attack mode |
| `t` | Target mode |

## Action Enum

`Action` is the bridge between input and game logic. Defined in `input.rs`, dispatched in `main.rs::update()`.

Key variants:
```rust
pub enum Action {
    Move(i32, i32),
    Wait, EndTurn, AutoExplore, Rest,
    RangedAttack(i32, i32),
    SetTarget(i32, i32),
    UseStairs,
    UseItem(usize),
    EquipSelected, UnequipSelected,
    OpenInventory, OpenQuestLog, OpenCrafting,
    OpenWorldMap, WorldMapTravel(usize, usize),
    OpenPauseMenu, OpenControls,
    OpenDebugMenu, OpenIssueReporter,
    OpenPsychicMenu, OpenFactionMenu, OpenVoidMenu,
    OpenCrystalMenu, OpenLightMenu, OpenSkillsMenu,
    OpenWiki,
    EnterLook,
    TradeBuy(usize), TradeSell(usize),
    ChestTransfer, CloseChest, OpenChest(usize),
    Craft,
    Interact(i32, i32), Examine(i32, i32),
    BreakWall(i32, i32),
    Save, Load,
    ReturnToMainMenu, Quit,
    SubmitIssueReport,
    DebugCommand(String),
    UsePsychicAbility(String),
    UseVoidAbility,
    AllocateStat(String),
    None,
}
```

## Adding a New Screen

1. Create `src/ui/my_screen.rs` with a state struct and `render_my_screen()` function
2. Add `pub mod my_screen;` to `src/ui/mod.rs`
3. Add state field to `UiState`
4. Add render call in `main.rs::render()` (with early return if fullscreen)
5. Add input handling in `handle_input()` when screen is active
6. Add `Action::OpenMyScreen` variant and dispatch in `main.rs::update()`

## Camera

`UiState::update_camera(player_x, player_y)` — smooth lerp toward player position.

`camera_x`, `camera_y` are `f32` for smooth scrolling. Renderer uses these to offset tile rendering.

## Dialog Box

`DialogBox` — modal text overlay with speaker name and text. Ticked each frame via `dialog_box.tick(16)`. Shown via `state.pending_dialogue` which is consumed by the game loop.

## Tutorial System

`state.get_next_tutorial_message()` → `Option<(id, text)>` — checked after each action. Stored in `ui.tutorial_message`. Dismissed on any keypress via `state.dismiss_tutorial_message(id)`.
