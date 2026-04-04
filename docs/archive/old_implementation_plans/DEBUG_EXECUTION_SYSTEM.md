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
- A base DES json file to use as a starting point for the test scenario.
  - Base DES files could also have variables that can be overridden by the specific test DES file.
    - E.g. item pickup test could use a base DES file that spawns a player in a room with a table and an item on the table. The specific test DES file could then override the item to be picked up.

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
7. **DES Input Parser**: Develop a parser that can read and interpret the DES input json files, translating them into actionable commands for the DES to execute.
8. **Base DES Files**: Create a library of base DES files for common test scenarios. These files can be used as templates for specific tests, reducing redundancy and improving maintainability.
9. **Action/State Indexing For Error Tracking**: Each action executed by the DES should be indexed, and the game state after each action should also be indexed. This will allow for easy tracking of where errors occur in the sequence of actions. If a test fails, the specific action and resulting game state can be examined to identify the root cause of the issue.
10. **The Option to Run DES in Parallel**: To speed up testing, the DES should have the option to run multiple test scenarios in parallel. This will allow for faster identification of bugs and issues across different game systems. Care must be taken to ensure that shared resources are managed correctly to avoid conflicts between parallel tests.
11. **The option to Mock Certain Systems**: For certain tests, it may be beneficial to mock specific game systems (e.g., AI behavior, random number generation) to ensure consistent and predictable outcomes. The DES should have the option to enable or disable mocking for specific systems based on the test requirements.
12. **Comprehensive Documentation**: Document the DES architecture, usage guidelines, and best practices for creating DES input files. This documentation should be accessible to all team members to facilitate collaboration and ensure consistent use of the DES.
13. **Continuous Integration (CI) Integration**: Integrate the DES into the project's CI pipeline to ensure that tests are automatically executed on code changes. This will help catch regressions and issues early in the development process.
14. **The option to Seed Random Number Generators**: To ensure reproducibility of tests that involve randomness, the DES should have the option to seed random number generators. This will allow for consistent outcomes across test runs, making it easier to identify and fix issues related to random events in the game.
15. **The option to run DES with rendering and slow down execution**: While the primary purpose of the DES is to run without rendering for speed, there may be cases where visual verification is needed. The DES should have an option to enable rendering and slow down execution to allow developers to observe the game state visually during test runs. This can be useful for debugging complex interactions or verifying visual elements.
