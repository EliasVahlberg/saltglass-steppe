# Inspirations for Saltglass Steppe

This document catalogs the key influences and inspirations for the Saltglass Steppe TUI RPG, organized by category and impact on the project's design.

## Primary Inspirations

### Caves of Qud
**Website:** https://www.cavesofqud.com/  
**Influence:** Core gameplay loop, mutation system, post-apocalyptic science fantasy setting

Caves of Qud serves as the primary inspiration for Saltglass Steppe's approach to:
- **Mutation mechanics** with social consequences and meaningful choices
- **Post-apocalyptic science fantasy** blending technology and mysticism
- **Procedural generation** combined with handcrafted content
- **Rich lore** delivered through environmental storytelling and item descriptions
- **ASCII aesthetics** that prioritize clarity and imagination over graphics

Key elements adapted: mutation progression systems, faction reputation mechanics, the balance between weird science and mystical elements.

### Dwarf Fortress
**Website:** http://www.bay12games.com/dwarves/  
**Influence:** ASCII interface design, complex systems interaction, emergent storytelling

Dwarf Fortress demonstrates how ASCII interfaces can convey complex information elegantly:
- **Information density** without overwhelming the player
- **Symbolic representation** where each character has clear meaning
- **Emergent narratives** arising from system interactions
- **Depth over presentation** - gameplay complexity through simple visuals

Key elements adapted: ASCII art for inspection mode, layered information display, systems that create emergent stories.

## Classic Roguelike Foundations

### NetHack
**Website:** https://www.nethack.org/  
**Influence:** Traditional roguelike mechanics, ASCII conventions, item identification

NetHack established many conventions that Saltglass Steppe follows:
- **@ symbol for player** character
- **Letter-based monster** representation (h for hounds, b for beetles)
- **Symbol-based items** (! for potions, ? for scrolls)
- **Turn-based tactical** combat with positioning importance
- **Permadeath consequences** that make every decision meaningful

### ADOM (Ancient Domains of Mystery)
**Website:** https://www.adom.de/  
**Influence:** Overworld exploration, corruption mechanics, faction systems

ADOM's approach to character transformation and world exploration:
- **Corruption system** that changes the character over time (similar to our Refraction)
- **Overworld map** connecting multiple dungeon areas
- **Faction relationships** that affect gameplay options
- **Moral choices** with long-term consequences

### Brogue
**Website:** https://sites.google.com/site/broguegame/  
**Influence:** Modern ASCII aesthetics, elegant interface design, color usage

Brogue shows how ASCII can be beautiful and functional:
- **Sophisticated color usage** to convey information and atmosphere
- **Smooth animations** in text mode
- **Clean, readable interface** that prioritizes player understanding
- **Environmental storytelling** through visual design

Key elements adapted: color-coded information systems, visual effects in ASCII, elegant UI layout.

## Modern TUI Design

### Cogmind
**Website:** https://www.gridsagegames.com/cogmind/  
**Influence:** Advanced TUI design, information visualization, sci-fi aesthetics

Cogmind demonstrates cutting-edge TUI design principles:
- **Modular interface** with resizable panels
- **Real-time information** updates and feedback
- **Sci-fi terminal aesthetics** that feel authentic
- **Complex data visualization** in text mode
- **Accessibility considerations** for different screen sizes

Key elements adapted: HUD design principles, status display methods, sci-fi terminal feel.

## Thematic and Narrative Influences

### Literary Science Fiction

#### Gene Wolfe's "Book of the New Sun"
**Influence:** Post-apocalyptic mysticism, unreliable narration, dying earth atmosphere

Wolfe's work provides the template for Saltglass Steppe's tone:
- **Dying earth** atmosphere where the past is mysterious and powerful
- **Technology indistinguishable from magic** in a post-technological society
- **Unreliable perspectives** on historical events
- **Mythic language** that makes the mundane feel numinous

#### Roadside Picnic (Strugatsky Brothers)
**Influence:** Alien artifacts, zone logic, transformation themes

The concept of areas where normal rules don't apply:
- **Anomalous zones** with their own physics
- **Artifacts** with mysterious properties and purposes
- **Gradual transformation** of those who enter the zones
- **Scientific vs. mystical** interpretations of phenomena

### Tabletop RPG Influences

#### Gamma World
**Website:** Various editions by TSR/Wizards of the Coast  
**Influence:** Post-apocalyptic mutation mechanics, gonzo science fantasy

Gamma World's approach to mutation and post-apocalyptic society:
- **Random mutations** with both benefits and drawbacks
- **Technology worship** by post-apocalyptic societies
- **Faction conflicts** over resources and ideology
- **Gonzo elements** that blend humor with horror

#### Mutant: Year Zero
**Website:** https://freeleaguepublishing.com/games/mutant-year-zero/  
**Influence:** Community building, resource scarcity, mutation as identity

Modern take on post-apocalyptic mutation themes:
- **Community survival** as core gameplay loop
- **Mutation as character identity** rather than just mechanics
- **Resource management** in harsh environments
- **Hope vs. despair** balance in storytelling

## Technical and Design Influences

### Terminal UI Libraries and Tools

#### Ratatui (Rust TUI Library)
**Website:** https://ratatui.rs/  
**Influence:** Modern TUI development patterns, widget systems

Ratatui provides the technical foundation for modern TUI development:
- **Widget-based architecture** for modular interfaces
- **Cross-platform compatibility** across terminal types
- **Performance optimization** for smooth text-mode graphics
- **Event handling** for responsive user interaction

#### ncurses Legacy
**Influence:** Terminal control standards, color management, input handling

The ncurses library established many conventions we follow:
- **Color pair management** for consistent theming
- **Window management** for complex layouts
- **Input handling** patterns for keyboard navigation
- **Screen management** for flicker-free updates

### Game Development Patterns

#### Data-Driven Design
**Influence:** JSON-based content systems, modding support, rapid iteration

Modern game development emphasizes data-driven approaches:
- **JSON configuration** for easy content modification
- **Separation of data and code** for maintainability
- **Modding support** through external data files
- **Rapid iteration** through hot-reloading content

## Visual and Aesthetic References

### ASCII Art Traditions

#### Bulletin Board System (BBS) Art
**Influence:** Character-based graphics, ANSI color usage, terminal aesthetics

The BBS era established many ASCII art conventions:
- **ANSI color codes** for vibrant terminal graphics
- **Character art** using extended ASCII sets
- **Terminal aesthetics** that feel authentic to computing history
- **Information density** in limited screen space

#### Demoscene ASCII Art
**Influence:** Advanced character graphics, animation techniques, artistic expression

The demoscene pushed ASCII art to artistic heights:
- **Complex character graphics** using standard character sets
- **Animation techniques** in text mode
- **Artistic expression** within technical constraints
- **Community standards** for quality and innovation

### Science Fiction Visual Design

#### Cyberpunk Terminal Aesthetics
**Influence:** Green-on-black terminals, data visualization, hacker culture

Cyberpunk media established visual languages for futuristic interfaces:
- **Monospace fonts** and terminal windows
- **Data streams** and scrolling text
- **Neon color schemes** against dark backgrounds
- **Information overload** as aesthetic choice

#### Retro-Futurism
**Influence:** 1970s-80s vision of the future, analog-digital hybrid aesthetics

Retro-futuristic design informs the Heliograph-era technology:
- **Analog-digital hybrids** mixing old and new technology
- **Brutalist architecture** in technological structures
- **Optimistic technology** that became dystopian
- **Period-appropriate** interface design

## Gameplay Mechanic Influences

### Immersive Sim Principles
**Influence:** Emergent gameplay, system interaction, player agency

Games like Deus Ex and System Shock inform our approach to:
- **Multiple solutions** to problems through different character builds
- **System interaction** creating emergent gameplay
- **Player agency** in narrative and mechanical choices
- **Consequence systems** that respond to player actions

### Social Simulation Games
**Influence:** Reputation systems, faction dynamics, relationship mechanics

Games with complex social systems provide models for:
- **Reputation tracking** across multiple factions
- **Dynamic relationships** that change based on player actions
- **Social consequences** for character choices
- **Faction conflict** creating meaningful player decisions

## Anti-Influences (What We Avoid)

### Generic Fantasy Tropes
We deliberately avoid standard fantasy elements to maintain the unique science-fantasy atmosphere:
- **Medieval fantasy** settings and terminology
- **Standard RPG classes** like fighter/mage/thief
- **Magic systems** without scientific grounding
- **Fantasy races** like elves and dwarves

### Modern UI Conventions
We resist modern interface trends that would break the TUI aesthetic:
- **Mouse-dependent** interfaces
- **Graphical icons** and buttons
- **Modern color schemes** that don't fit terminal aesthetics
- **Animation effects** that require high refresh rates

### Grimdark Aesthetics
While the world is harsh, we avoid purely grimdark elements:
- **Hopeless nihilism** without wonder or beauty
- **Gratuitous violence** for shock value
- **Cynical worldbuilding** without positive elements
- **Edgelord content** that prioritizes darkness over depth

## Research Sources and Further Reading

### Academic Sources
- "The Use of ASCII Graphics in Roguelikes" (ResearchGate) - Analysis of visual design in text-based games
- "Roguelike Review Topic 004" (alt.org) - Historical perspective on roguelike development

### Development Resources
- **TUI Game Development Inspiration** (GitHub: matanlurey/tui-game-inspo) - Curated collection of TUI game examples
- **Roguelike Development Resources** (RogueBasin) - Community wiki for roguelike development
- **Terminal RPG Examples** (lib.rs) - Modern Rust-based terminal games

### Community Resources
- **RogueBasin Wiki** (http://roguebasin.com/) - Comprehensive roguelike development resource
- **Bay12 Forums** - Dwarf Fortress community discussions on ASCII interface design
- **RPG Codex** - Classic RPG discussion and analysis

---

*This document serves as a living reference for the creative vision of Saltglass Steppe. It should be updated as new influences are discovered or as the project's direction evolves.*
