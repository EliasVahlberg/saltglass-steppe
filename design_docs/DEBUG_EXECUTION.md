# Debug Execution System : Implementation Details

Goal: Allmost all additions to the game should able to automatically test.

## Overview

The Debug Execution System (DES) is a way to run the game without rendering graphics or user input. This allows for automated testing of game features, mechanics, and systems. The DES will simulate game execution starting with only the player character and a empty world. The DES will take as input a json file that contains:

- List of entities to spawn with their properties:
  - location
  - health
  - ai/no-ai
  - inventory
  - equipment
- List of equipment and items to equip the player with:
  - weapons
  - armor
  - consumables
  - key items
- Player properties:
  - stats
  - skills
  - abilities
  - hp/max hp
  - psy/max psy
- List of actions to perform, and who should perform these actions and on what turns:
  - Move/TP to location
  - Attack entity
  - Use item
  - Interact with entity
  - Wait for X turns
  - Log specific game state information (e.g. player health, entity status, inventory contents) (can be used on the same turn as other actions)
- (Optional) A base DES json file to use as a starting point for the test scenario.

The DES will then execute the actions in order, updating the game state accordingly. For this to work, all game systems must be decoupled from the rendering and input systems. This means that game logic should not rely on user input or graphical output to function correctly.
Logging and state tracking systems will be implemented to record the game state after each action. This will allow for verification of expected outcomes and identification of any discrepancies or bugs.

## Implementation Details

1. **Game State Management**: Implement a robust game state management system that can be easily
   serialized and deserialized. This will allow the DES to save and load game states as needed.
2. **Action Queue**: Create an action queue system that can process a list of actions in sequence. Each action should have a defined set of parameters and expected outcomes.
3. **Entity Management**: Develop an entity management system that can spawn, update, and
   despawn entities based on the DES input. This system should handle all entity properties and behaviors.
4. **Logging System**: Implement a logging system that records the game state after each action.
5. **Testing Framework**: Integrate the DES with a testing framework that can validate the game state against expected outcomes after each action. This framework should support assertions and reporting of test results.
6. **Decoupling Game Logic**: Refactor existing game systems to ensure that game logic is decoupled from rendering and input systems. This may involve creating interfaces or abstract classes that separate game mechanics from user interactions.
