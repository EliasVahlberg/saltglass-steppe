# Core Game Mechanics

This document outlines the core mechanics that will form the foundation of the game. These mechanics are designed to create an engaging, immersive, and unique experience for players. They will be further refined and expanded upon during the development process. They are not final and are subject to change based on playtesting and feedback.

## 1. Turn-Based System [0/10]

- **Turn Structure**: The game operates on a turn-based system where each player action (movement, attack, ability use) consumes a certain number of action points (AP). Enemies and NPCs also take turns based on their own AP.
- **Action Points (AP)**: Each character has a set amount of AP per turn. Actions such as moving, attacking, or using abilities will consume varying amounts of AP.
- **Initiative**: At the start of each encounter, characters roll for initiative to determine the order of turns.
- **Environmental Interactions**: Players can interact with the environment (e.g., taking cover, using objects) which may also consume AP.
- **Natural phenomena and timed events**: Certain environmental effects (like weather changes) and timed events will occur at specific intervals, influencing gameplay and strategy.
- **Physics and chemical reactions**: E.g. a fire spreading through a flammable area or a chemical explosion affecting nearby characters. Acid that corrodes metal and leaves puddles.
- **Liquid system**: liquids can flow, mix, and stain: surfaces, characters, and items. E.g. water can extinguish fire, oil can make surfaces slippery, and acid can corrode metal.
- **Status Effects**: Characters can be affected by various status effects (e.g., stunned, poisoned) that can alter their AP or actions during their turn.
- **Interruptions**: Certain actions or abilities can interrupt an enemy's turn, allowing for reactive gameplay and strategic planning.
- **End Turn**: Players can choose to end their turn early, allowing enemies to act sooner, which can be a strategic choice in certain situations.

## 2. Player Movement and Interaction [0/6]

- **Movement**: The player can move in all directions including diagonal movement.
- **Interaction**: Players can interact with objects and NPCs in the environment through context-sensitive actions (e.g., examine, pick up, talk, inspect).
- **Inspection Mode**: Players can enter an inspection mode to closely examine objects, read inscriptions, or view details that are not visible from the standard perspective. This is rendered in ASCII art and is (to begin with) only available for entities of significance landmarks, central NPCs, writings/symbols.
- **Inventory item inspection**: Players can inspect all items that they have in their inventory in ASCII art form. The ASCII art representation of the item will change based on its condition (e.g., pristine, worn, damaged).
- **Line of Sight**: The game uses a line-of-sight system where players can only see areas and objects that are within their character's visual range and unobstructed by obstacles (there are abilities and items that can alter line of sight).
- **Cover System**: Players can take cover behind objects to reduce hit chances from enemy attacks and provide stealth opportunities.
- **Stealth Mechanics**: Players can use stealth to avoid detection by enemies, utilizing lighting, sneaking (crouching), and environmental noise to their advantage.

## 3. UI and HUD [0/4]

Render of the game can be shown in appendix:
Appendix A: Game Render Mockups

- **Top HUD**: Displays essential player information such as health points (HP), action points (AP), armor rating (AR), reflex (RE), movement speed (MS), psychic energy (PSY), weight carried (W), currency, and current time/weather.
- **Main Screen**: The main screen displays the current tile or world map, along with options to view the inventory, inspection modes (landmark/POI/NPC/item/player), and other relevant information.
- **Side Panel**: Displays the player's ASCII art representation along with their equipped gear. This panel is resizable within the menu for better visibility.
- **Bottom Panel**: Contains hotkeys for active and passive skills, gear, and important key bindings (menu, movement, interaction). It also includes an event log to keep track of recent actions and events in the game.

## 4. Map and Environment [0/5]

- **Tile-Based System**: The game world is divided into a grid of tiles, each tile has specific properties (e.g., terrain type, elevation). Unique tiles have one or more defining features e.g. other than terrain/biome/elevation towns, dungeons, landmarks. Connected tiles: rivers, roads, railroads should connect between tiles.
- **Procedural world map generation**: The world map is procedurally generated with layers: 1. Biome layer (desert, forest, tundra, etc), 2. Terrain layer (mountains, hills, plains, etc), 3. Elevation layer (sea level, lowlands, highlands, etc), 4. Resource layer (water sources, mineral deposits, flora/fauna distribution, etc), 5. Points of interest layer (towns, dungeons, landmarks, etc). Layers 1,2,3 use noise and wave function collapse algorithms to create natural-looking terrain. Layers 4 and 5 use procedural placement algorithms to distribute resources and points of interest based on biome and terrain types, the placement of layer 5 should be intentionally spread out more evenly (i.e. penalize proximity to other POI) to encurage exploration.
- **Procedural Tile Generation**: Each tile is procedurally generated when the player first enters it (lazy generation). The tile generation process considers the biome, terrain, elevation, and resources defined in the world map layers to create a unique and coherent environment. The seed for each tile's procedural generation is derived from the world map's seed combined with the tile's coordinates to ensure consistency for the same world seed.
- **Subterranean and layers**: Only certain tiles will have subterranean layers (e.g., caves, dungeons, underground cities). These layers can be accessed through specific entry points on the surface tile. Each subterranean layer will have its own unique layout and challenges.
- **Lighting and Visibility**: The game features a dynamic lighting system that affects visibility. Light sources (e.g., torches, lamps) will illuminate areas, while darkness will limit the player's line of sight. Weather conditions and time of day will also impact lighting and visibility.
- **Environmental Interactions**: Players can interact with various environmental elements (e.g., doors, levers, traps) that can alter the tile's layout or provide strategic advantages during encounters.

## 5. Exploration

- **Auto Explore**: Players can enable an auto-explore feature that allows their character to explore/reveal all reachable areas on the current tile.
- **Render explored areas**: Once static objects and terrain have been explored they will remain visible on the map but dynamic objects like enemies and items will not be visible unless in line of sight. The explored areas will be rendered in a desaturated color palette to differentiate them from the current line of sight areas.

## 6. Combat System [0/10]

- **HP, AP, AR, RE, MS, PSY stats**: Health Points (HP) represent the character's vitality. Action Points (AP) determine how many actions a character can take per turn. Armor Rating (AR) reduces incoming damage. Reflex (RE) affects dodge chances and initiative. Movement Speed (MS) determines how far a character can move per turn. Psychic Energy (PSY) is used for psychic abilities.
- **Non-tracked stats**: These stats are not shown on the HUD but influence gameplay. E.g. Strength affects melee damage and carrying capacity, Intelligence affects skill effectiveness and hacking abilities, Endurance affects HP and resistance to status effects, Agility affects movement speed and dodge chances, Charisma affects NPC interactions and barter prices.
- **Melee attacks**: Characters can perform melee attacks using various weapons or unarmed combat. Melee attacks consume AP and can be influenced by the character's stats and equipment. Special unarmed attacks: Kick, Headbutt, Bite, Grapple, Spit.
- **Ranged attacks**: Characters can use ranged weapons to attack enemies from a distance. Ranged attacks consume AP and are affected by factors such as weapon accuracy, range, and line of sight. Also includes thrown weapons and explosives.
- **Psychic abilities**: Characters with psychic abilities can use PSY to perform special actions, such as mind control, telekinesis, solar ray, or psychic blasts. These abilities consume PSY and may have cooldowns or other limitations.
- **Target parts of the body**: Players can aim for specific body parts (e.g., head, torso, limbs) to exploit weaknesses or inflict status effects. Targeting specific parts may have different hit chances and damage multipliers.
- **Cover mechanics**: Characters can take cover behind objects to reduce the chance of being hit by enemy attacks. Different types of cover provide varying levels of protection.
- **Status effects**: Various status effects (e.g., stunned, poisoned, burning) can be inflicted on characters during combat, affecting their performance and requiring strategic responses.

## 7. Enemy AI and NPCs [0/8]

- **Basic Behavior**: Enemies will have basic AI behaviors such as patrolling, chasing the player, and attacking when in range.
- **AI Demeenor**:
  - Aggressive AI will actively seek out and attack the player on sight.
  - Defensive AI will prioritize taking cover and avoiding damage, only attacking when the player is within a certain range.
  - Neutral AI does not consider the player a threat and will only react if provoked.
  - Friendly AI will assist the player in combat and may provide support abilities (can change to defensive if needlessly attacked multiple times).
  - Pacifist AI will not engage in combat under any circumstances and will attempt to flee or hide if threatened.
- **Aggro System**: A enemy will have agro towards one character, usually the player but could be a companion or another NPC. The enemy will focus its attacks and actions on the character it has aggro towards, the agro target change based on which character has the highest agro value. Agro value is determined by actions such as dealing damage, healing allies, or using abilities that threaten the enemy.
- **Item Usage**: Enemies can use items such as health packs, grenades, or buffs during combat to enhance their effectiveness. Note: Not all enemies will have this capability; it will depend on their type and intelligence.

## 8. Character Progression and Skills [0/7]

- **Experience Points (XP)**: Players earn XP through combat, exploration, completing quests, and other in-game activities. Accumulating XP allows characters to level up and improve their stats and abilities.
- **Leveling Up**: When a character levels up, they gain skill and stat points:
  - Skill Points: Used to unlock or upgrade abilities in various skill trees (e.g., combat, stealth, crafting, psychic).
  - Stat Points: Used to increase core stats such as Strength, Intelligence, Endurance, Agility, and Charisma.
- (Still considering) **Skill Trees**: The game features multiple skill trees that allow players to specialize their characters.
- (Still considering) **Perks and Traits**: Characters can acquire perks and traits that provide unique bonuses or abilities. These can be unlocked through leveling up or completing specific challenges.
- (Still considering) **Specializations**: At certain levels, players can choose specializations that further define their character's role and abilities (e.g., sniper, medic, engineer, psychic adept).
- **Respec Option**: Players have the option to respec their character's skills and stats at specific points in the game, allowing for flexibility in playstyle and adaptation to new challenges.
- **Companion Progression**: Companion characters also have their own progression systems, allowing players to improve their abilities and stats as they accompany the player on their journey.
- **Unique Backgrounds**: Players can choose or discover unique backgrounds for their characters that provide specific starting bonuses, skills, or storylines. These backgrounds can influence gameplay and character development throughout the game.
- **Unconventional Builds and Playstyles**: The game supports a variety of unconventional builds and playstyles, allowing players to experiment with different combinations of skills, stats, and abilities to create unique characters that suit their preferred approach to gameplay. Builds should be somewhat quirky and have both significant pros and cons to balance them out.

## 9. Inventory and Equipment [0/5]

## 10. Crafting System [0/6]

## 11. Quests and Storytelling [0/6]

## 12. Audio and Sound Design [0/4]

## 13. Modularity and Extensibility (modding support) [0/5]

## Appendix A: Game Render Mockups

```text
________________________________________________________________________________________________________
| HP:20/31| Ref:0 |$1871 | Time:08:35 | AP:2 | AR:4 | RE:6 | MS:10 | PSY:14/20 | W:14.8/25kg| Storm: 26|
|______________________________________________________________________________________________________|
|                                                                         |                            |
|                                                                         |                            |
|                            Either:                                      |                            |
|                          1.Current tile                                 |                            |
|                          2.World Map                                    |         Player+Gear        |
|                          3.Inventory                                    |        (ASCII ART)         |
|                          4.Inspection (Landmark/POI/NPC)                |     (Reziseable in menu)   |
|                          5.Item inspection (Item in players hand)       |                            |
|                          6.Player inspection                            |                            |
|                                                                         |                            |
|                                                                         |____________________________|
|                                                                         |                            |
|                                                                         |                            |
|                                                                         |                            |
|                                                                         |                            |
|_________________________________________________________________________|     Hostile mob + gear     |
|Active skills and gear hotkeys         |                                 |                            |
|                                       |         Event log             | |    (if you have a target)  |
|Passive skills and gear hotkeys        |         .........             = |        (ASCII ART)         |
|                                       |         .........             | |     (Reziseable in menu)   |
|Important key bindings:menu+move+intera|         .........             | |                            |
|_______________________________________|_________________________________|____________________________|

```
