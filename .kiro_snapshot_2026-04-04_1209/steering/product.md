# Product Context

## What is Saltglass Steppe?

A deterministic, data-driven, turn-based TUI roguelike RPG set in a post-post-apocalyptic crystalline desert. Players navigate a world where glass storms physically rewrite maps, light is tactically dangerous, and character mutations have social consequences.

## Core Identity

**The Fantasy**: You are becoming something strange in a world that was already strange. Every storm changes the land; every adaptation changes how the world sees you.

**Inspired by**: Caves of Qud (systems depth, mutations), Cogmind (TUI elegance), Dwarf Fortress (emergent stories)

## Target Players

- Roguelike veterans seeking novel mechanics (storm-based map editing)
- Systems-driven players who enjoy learning rules and exploiting them
- Narrative explorers drawn to weird fiction and mythic tone
- TUI enthusiasts who appreciate ASCII aesthetics

## Core Features

### Signature Systems
- **Storm-Edited Maps**: Glass storms procedurally modify dungeons (rotate rooms, swap modules, fuse walls)
- **Refraction Adaptations**: Mutation system with social consequences (power vs. faction reputation)
- **Light Tactics**: Combat using beams, reflections, glare, and sightlines
- **Deterministic Generation**: Seeded RNG for reproducible worlds and testing

### Gameplay Loop
1. Take contracts/rumors in settlements
2. Travel overworld (resource management, encounters)
3. Explore procedural sites (ruins, vaults, canyons)
4. Extract value (relics, water, scripture shards, storm glass)
5. Return or press deeper (risk vs. reward)

### World Structure
- **Biomes**: Salt Flats, Glassed Reefs, Mirror Canyons, Singing Dunes, Brine Under
- **Sites**: Refraction Cathedra, Storm-Locked Vaults, Crucible Blocks, Pilgrim Necropolis
- **Factions**: Mirror Monks, Archive Drones, Salt Traders, Storm Cults, Refraction Outcasts

## Development Objectives

### Phase 1: Core Systems (Current)
- Stable TUI rendering with ratatui
- Turn-based gameplay loop
- Procedural generation (world + tile maps)
- Combat, inventory, quests
- Debug Execution System (DES) for automated testing

### Phase 2: Signature Features
- Storm system with map editing
- Refraction adaptation trees
- Light-based combat mechanics
- Faction reputation system

### Phase 3: Content & Polish
- Full quest chains
- Biome-specific content
- Narrative events and lore
- Balance and difficulty tuning

## Success Criteria

- **Technical**: Deterministic, bug-free, performant TUI (60fps)
- **Design**: Storms feel surprising but fair, adaptations create meaningful choices
- **Narrative**: World feels cohesive and strange, not random
- **Player**: "I've never played anything like this" + "I understand the rules"

## Non-Goals

- Not graphical (pure TUI)
- Not multiplayer
- Not comedic-weird (mythic tone, not absurdist)
- Not unfair roguelike (telegraphed danger)
