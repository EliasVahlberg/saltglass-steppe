# Feature Implementation Log - 2025-12-22

## Overview

Implemented several high-priority technical features required for content expansion, including advanced combat behaviors, dialogue conditions, quest integration, and item mechanics.

## 1. Psychic System Fixes

- **Status Effects**: Added `StatusEffect` struct to `src/game/enemy.rs` to support enemy-side status tracking.
- **Ability Logic**: Fixed `use_psychic_ability` in `src/game/state.rs` to correctly apply effects like "stun_aoe" and "guaranteed_hit".
- **Verification**: Validated with `tests/scenarios/psychic_test.json`.

## 2. ARIA Quest Integration

- **Quest Objectives**: Added `InterfaceWithAria` objective type to `src/game/quest.rs`.
- **Item Properties**: Added `enables_aria_dialogue` flag to `ItemDef`.
- **Logic**: Updated `use_item` to trigger `on_aria_interfaced` in the quest log when appropriate items (e.g., `saint_key`) are used.
- **Verification**: Validated with `tests/scenarios/aria_test.json`.

## 3. Advanced Combat Behaviors

- **New Behaviors**:
  - `split_on_death`: Enemies spawn smaller variants upon death (e.g., Slime -> Small Slimes).
  - `swarm`: Triggering one enemy alerts others within a radius.
  - `laser_beam`: Ranged direct damage attack ignoring cover/evasion.
- **Implementation**: Added logic to `src/game/ai.rs` and `src/game/combat_actions.rs`.
- **Verification**: Validated with `tests/scenarios/combat_behaviors_test.json`.

## 4. Advanced Dialogue Conditions

- **New Conditions**:
  - `min_salt_scrip`: Checks if player has enough currency.
  - `min_reputation`: Checks faction reputation levels.
- **Context**: Updated `DialogueContext` to include `salt_scrip` and `faction_reputation`.
- **DES Support**: Added `SetSaltScrip` action to DES for testing.
- **Verification**: Validated with `tests/scenarios/dialogue_conditions_test.json`.

## 5. Adaptation Suppression

- **Mechanic**: Items like `veil_tincture` can now suppress adaptations.
- **Logic**: Added `effective_adaptation_count` to `DialogueContext`. If adaptations are suppressed, NPCs perceive the player as having none.
- **Verification**: Validated with `tests/scenarios/suppression_test.json`.

## 6. Wall Breaking Mechanic

- **Mechanic**: Certain items (`glass_pick`) can destroy wall tiles.
- **Implementation**:
  - Added `use_item_on_tile` method to `GameState`.
  - Added `UseItemOn` and `SetTile` actions to DES.
  - Implemented wall HP reduction and destruction logic.
- **Verification**: Validated with `tests/scenarios/wall_break_test.json`.

## 7. Bug Fixes

- **Crafting**: Added missing `health_potion` recipe to `data/recipes.json` to fix `crafting_basic` test failure.
