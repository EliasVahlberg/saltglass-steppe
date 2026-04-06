# state.rs Content Distribution & VERA Migration Analysis

> Last updated: 2026-04-05 (post Batch D/E/F + legacy deletion)
> Current LOC: 3,528

## Content Distribution

| Section | LOC | Description |
|---------|-----|-------------|
| Declarations | 539 | `MsgType`, `GameMessage`, `SpatialIndex`, `DebugState`, `PendingUi`, `GameState` struct (~50 fields) |
| Dispatch helpers | 574 | 11 `&mut self` orchestrators wired during VERA migration (see below) |
| World travel | 486 | `travel_to_tile`, `move_on_world_map`, `enter/exit_subterranean`, `spawn_quest_required_npcs`, `find_safe_spawn_position` |
| `dispatch()` router | 106 | Main `match command { ... }` arm |
| Queries | 278 | `enemy_at`, `npc_at`, `auto_explore`, `has_nearby_enemies`, `is_dangerous_tile`, `has_talked_npc_at_idx`, etc. |
| Quest + craft + trade | 183 | `accept_quest`, `complete_quest`, `craft`, `buy_item`, `sell_item` |
| Encounter + flee | 197 | `attempt_flee_encounter`, `spawn_encounter_entities` |
| Derives + narrative gen | 192 | `update_lighting`, `update_fov`, `generate_item_lore`, `generate_location_description`, etc. |
| Inventory + chest | 166 | `pickup_items`, `open_chest`, `transfer_to/from_chest`, `equip_item`, `unequip_slot` |
| Accessors | 144 | `player_x`, `map()`, `enemies()`, `storm()`, `time_of_day()`, etc. — 30+ trivial one-liners |
| Turn processing | 140 | `end_turn`, `execute_phase` (9 arms), `tick_turn_housekeeping` |
| Misc systems | 130 | Tutorial, reputation, status effects, `apply_light_effects`, save/load |
| Interact + examine | 128 | `interact_at`, `examine_at`, `debug_command` |
| Init + spatial index | 97 | `new_with_class`, `ensure_spatial_index`, `rebuild_spatial_index` |
| VERA infra | 80 | `apply_and_trace`, `run_reactions`, `collect_reactions` |
| Visual effects | 58 | Hit flash, damage numbers, beams, projectiles — thin wrappers |
| Logging | 30 | `log`, `log_typed`, `log_quest_completions` |

### Dispatch helpers (574 LOC)

| Function | LOC (approx) | Notes |
|----------|-------------|-------|
| `dispatch_melee_attack` | 107 | Calls `rule_melee_attack`, post-processes swarm aggro, reflect, split-on-death |
| `dispatch_ranged_attack` | 93 | Calls `rule_ranged_attack`, post-processes behaviors |
| `dispatch_world_move` | 62 | World map movement + map regeneration |
| `dispatch_world_move_safe` | 33 | Safe variant with adjacency check |
| `dispatch_move` | 70 | Calls `rule_move`, handles NPC/combat branches |
| `dispatch_accept_quest` | 37 | Calls `rule_accept_quest` + legacy quest wiring |
| `dispatch_complete_quest` | 51 | Calls `rule_complete_quest` + legacy quest wiring |
| `dispatch_use_psychic` | 13 | Thin wrapper |
| `dispatch_flee_encounter` | 48 | Flee logic |
| `dispatch_follow_world_path` | 24 | Path following |
| `dispatch_calculate_world_path` | 36 | Path calculation |

---

## VERA Migration Targets

These sections have clear `inputs → deterministic outputs → state mutations` that fit the pure rule pattern.

| Section | LOC | Rule signature | Effects produced |
|---------|-----|----------------|-----------------|
| `interact_at` / `examine_at` | ~128 | `rule_interact(x, y, &ctx)` / `rule_examine(x, y, &ctx)` | `EventEffect::QuestNotify`, `Presentation::LogMessage`, `EventEffect::OpenBook` |
| `accept_quest` / `complete_quest` | ~90 | Already have `Command::AcceptQuest/CompleteQuest` — dispatch helpers call legacy methods instead of rules | `QuestEffect::Accept/Complete`, `PlayerEffect::GainXp`, `PlayerEffect::GainSaltScrip` |
| `equip_item` / `unequip_slot` | ~40 | `rule_equip` / `rule_unequip` already exist — legacy methods in state.rs are duplicates not yet deleted | `ItemEffect::Equip/Unequip`, `ItemEffect::RecalcStats` |
| `craft` | ~37 | `rule_craft(recipe_id, &ctx)` | `ItemEffect::Consume × N`, `ItemEffect::AddToInventory` |
| `buy_item` / `sell_item` | ~56 | `rule_buy(trader_id, item_id, &ctx)` / `rule_sell(inv_idx, trader_id, &ctx)` | `PlayerEffect::GainSaltScrip`, `ItemEffect::AddToInventory/RemoveFromInventory` |
| `attempt_flee_encounter` | ~37 | `rule_flee_encounter` already exists in dispatch — legacy method is a duplicate | `PlayerEffect::ClearEncounter`, `PlayerEffect::SetLastFleeAttempt` |

**Total: ~388 LOC** of genuine VERA migration remaining in state.rs.

---

## Not VERA Targets

| Section | LOC | Reason |
|---------|-----|--------|
| World travel (486) | 486 | `travel_to_tile` regenerates the entire map and runs the generation pipeline. Output is a new world state, not a `Vec<Effect>`. Bridge effect at best. |
| Dispatch helpers (574) | 574 | Already the VERA wiring layer — they call rules then do post-processing. Correct as-is. |
| Queries (278) | 278 | Read-only helpers. No mutation, nothing to migrate. |
| Encounter spawning (197) | 197 | `spawn_encounter_entities` generates enemies from data tables and places them. Same problem as world travel. |
| Derives + narrative gen (192) | 192 | `update_fov`/`update_lighting` are derives by definition. Narrative gen is dead code. |
| Accessors (144) | 144 | Trivial getters. Nothing to migrate. |
| Declarations (539) | 539 | Struct definitions. Nothing to migrate. |

---

## Projected state.rs LOC After Full Migration

| Action | LOC delta | Resulting LOC |
|--------|-----------|---------------|
| Current | — | 3,528 |
| Migrate interact/examine/craft/buy/sell/quest (~388 LOC) | −388 | ~3,140 |
| **Floor** (orchestration + world travel + declarations) | — | **~3,100** |

The floor is set by world travel, dispatch orchestration, and declarations — none of which can be pure rules. Further reduction below ~3,100 requires **module extraction**, not VERA migration.

---

## Module Extraction Candidates

If the goal is a smaller state.rs beyond the VERA floor:

| Candidate module | LOC to extract | Contents |
|-----------------|---------------|----------|
| `src/game/world_travel.rs` | ~1,060 | World travel (486) + world travel dispatch helpers (dispatch_world_move, dispatch_world_move_safe, dispatch_follow_world_path, dispatch_calculate_world_path, ~300 LOC) |
| `src/game/encounter.rs` (extend) | ~197 | `spawn_encounter_entities`, `attempt_flee_encounter` — already have the data types there |
| `src/game/queries.rs` | ~278 | All read-only query helpers |

Extracting all three would bring state.rs to approximately **~2,000 LOC**.
