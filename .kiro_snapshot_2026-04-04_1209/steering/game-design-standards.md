# Game Design Standards

## Creative Pillars (Non-Negotiable)

Every feature must support at least one pillar without contradicting another:

1. **Mutation with Social Consequences** - Refraction adaptations grant power but alter faction perception
2. **Storms Rewrite Maps** - Glass storms procedurally edit dungeons (rotate, swap, fuse)
3. **Readable Light Tactics** - Combat uses beams, reflections, glare (clear in ASCII)
4. **TUI as Aesthetic Strength** - Text interface is the medium, not a limitation
5. **Authored Weirdness** - Strange but consistent; every anomaly has learnable rules

## Tone & Voice

### Weirdness Dial: Mythic-Reverent (6/10)

**Target**: Numinous (awe + unease), not horror or comedy

| Too Normal | Target Zone | Too Weird |
|------------|-------------|-----------|
| "A desert with ruins" | "Glass storms fuse cities into labyrinths" | "Sentient colors argue about philosophy" |
| "You find a sword" | "You find a lens that belonged to three dead saints" | "The sword is also a door" |

### Vocabulary

**Use**: refraction, vitrified, fused, glint, shimmer, glare, salt, brine, storm, glass, light, pilgrim, scavenger, saint, scripture, relic, adaptation, mutation, caste

**Avoid**: Modern slang, generic fantasy (mana, spell, magic), excessive proper nouns

### Log Line Style

Terse but evocative, directional, consequential:

```
✓ "The west wing *refracts*... corridors realign."
✓ "A mirage hound to the north attacks you."
✓ "Sharp glass cuts you! (-1 HP, +1 Refraction)"
✗ "Storm happened! Map changed!"
✗ "You got a mutation lol"
```

## Gameplay Design Principles

### Surprising but Fair
- Telegraph danger (storm forecasts, enemy tells)
- Changes are local, not whole-map chaos
- Deaths should feel earned, not random

### Systems Over Content
- Deep mechanics that interact (light + storms + mutations)
- Emergent gameplay from rule combinations
- Replayability through systems, not hand-crafted content

### Determinism First
- Seeded RNG for reproducible worlds
- Same seed = same map, encounters, loot
- Critical for testing and player sharing

### Data-Driven Balance
- Stats, costs, probabilities in JSON/RON
- Tune without recompilation
- Version configs for save compatibility

## Feature Design Checklist

Before implementing a feature, verify:

- [ ] Supports at least one creative pillar
- [ ] Doesn't contradict other pillars
- [ ] Has clear TUI representation (ASCII-friendly)
- [ ] Telegraphed to player (no hidden mechanics)
- [ ] Deterministic (seeded RNG)
- [ ] Data-driven (configs, not hardcoded)
- [ ] Tested with DES scenarios

## Content Creation Guidelines

### Items & Equipment
- Name format: `[Material] [Type]` (e.g., "Storm Glass Lens")
- Lore in description, mechanics in stats
- Unique items have backstories (scripture shards, saint relics)

### Enemies & NPCs
- Behavior driven by AI rules, not scripts
- Faction affiliation affects dialogue/combat
- Visual tells for abilities (glowing = beam attack)

### Quests & Narrative
- Multiple solutions (combat, stealth, social, exploration)
- Consequences affect faction reputation
- Lore reveals world history organically

### Biomes & Sites
- Each biome has unique hazards and resources
- Sites have archetypes (vault, cathedral, necropolis)
- Procedural but recognizable patterns

## Balance Philosophy

### Difficulty Curve
- Early game: Learn mechanics safely
- Mid game: Combine systems creatively
- Late game: Master interactions, high-risk choices

### Risk vs. Reward
- Storms are dangerous but create opportunities
- High Refraction = power + social penalties
- Deeper exploration = better loot + more danger

### Player Agency
- Multiple viable builds (combat, stealth, social, psychic)
- No forced choices (adaptations are optional)
- Failure creates stories, not frustration

## Visual Design (TUI)

### ASCII Clarity
- Distinct glyphs for tiles: `.` floor, `#` wall, `~` glass, `≈` water
- Color coding: Red = danger, Blue = water, Yellow = light, Cyan = glass
- Overlays for effects: `*` shimmer, `!` glare, `~` storm

### UI Layout
- Map viewport: Center focus
- Log: Bottom or side (scrollable)
- HUD: Top or side (HP, stats, forecast)
- Menus: Modal overlays (inventory, skills, quests)

### Animation Principles
- Subtle (1-2 frame flickers for effects)
- Functional (shows game state changes)
- Non-distracting (doesn't obscure gameplay)

## Testing Standards

### DES Scenarios
- One scenario per major feature
- Test edge cases (empty inventory, max stats)
- Regression tests for bug fixes

### Playtesting Focus
- Can players understand mechanics without docs?
- Are storms surprising but fair?
- Do adaptations feel meaningful?
- Is the tone consistent?

## Documentation Requirements

### Feature Specs
- Purpose (why this feature exists)
- Mechanics (how it works)
- TUI representation (what player sees)
- Data format (configs, save data)
- Testing plan (DES scenarios)

### Implementation Docs
- Architecture decisions
- Algorithm explanations
- Performance considerations
- Future extension points

## Non-Goals (What We Don't Do)

- **Not comedic-weird** - Mythic tone, not absurdist
- **Not grimdark** - Bleak moments, but hope exists
- **Not graphical** - Pure TUI, no sprites
- **Not kitchen-sink** - Focused mechanics, not feature bloat
- **Not unfair roguelike** - Telegraphed danger, earned deaths
