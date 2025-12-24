# Creative Direction Summary — Saltglass Steppe

**Version:** 1.0  
**Date:** 2024-12-14  
**Owner:** Creative Director  
**Purpose:** Align all disciplines on visual style, tone, narrative themes, and aesthetic consistency.

---

## Executive Summary

Saltglass Steppe is a TUI roguelike RPG set in a post-post-apocalyptic desert where ancient glass storms fused cities into shimmering labyrinths. The player is a scavenger-priest navigating a world where light is dangerous, maps are unreliable, and identity is mutable through refraction adaptations.

**The Core Fantasy:** You are becoming something strange in a world that was already strange. Every storm changes the land; every adaptation changes how the world sees you.

---

## 1. Creative Pillars

These are non-negotiable. Every feature, asset, and design decision must support at least one pillar without contradicting another.

### Pillar 1: Mutation with Social Consequences
Refraction adaptations grant power but alter how factions perceive you. Transformation is never free. *This is the core identity hook—"you are becoming something strange."*

### Pillar 2: Storms Rewrite Maps
Glass storms are not weather—they're procedural editors. The world physically changes, and players must adapt their mental maps. *The signature system that makes this game distinct.*

### Pillar 3: Readable Light Tactics
Combat and navigation use light/reflection as a core mechanic. Beams, glare, and sightlines must be visually clear in ASCII.

### Pillar 4: TUI as Aesthetic Strength
The text interface is not a limitation—it's the medium. Clarity, pattern recognition, and log messaging are the visual language.

### Pillar 5: Authored Weirdness
The world is strange but consistent. Every anomaly has rules the player can learn. No "random for random's sake."

---

## 2. Non-Goals (What This Game Is Not)

- **Not comedic-weird:** Tone is mythic and reverent, not Qud's absurdist humor.
- **Not grimdark survival:** Bleak moments exist, but hope and wonder are present.
- **Not graphically complex:** No sprites, no animations—pure TUI elegance.
- **Not kitchen-sink fantasy:** Inspirations are invisible; the world feels original.
- **Not unfair roguelike:** Surprising but telegraphed. Deaths should feel earned.

---

## 3. Tone & Voice

### The Weirdness Dial
Set to **Mythic-Reverent** (6/10 on the weird scale).

The Saltglass should feel *numinous*—awe mixed with unease. Not horror, but dread as a constant undercurrent. "A lens that belonged to three dead saints" works because it's strange *and* slightly unsettling.

| Too Normal | Target Zone | Too Weird |
|------------|-------------|-----------|
| "A desert with ruins" | "Glass storms fuse cities into labyrinths" | "Sentient colors argue about philosophy" |
| "You find a sword" | "You find a lens that belonged to three dead saints" | "The sword is also a door to another dimension" |

### Vocabulary Rules

**Use:**
- Refraction, vitrified, fused, glint, shimmer, glare
- Salt, brine, storm, glass, light, angle
- Pilgrim, scavenger, saint, scripture, relic
- Adaptation, mutation, caste, mark

**Avoid:**
- Modern slang, tech jargon (except Archive contexts)
- Generic fantasy terms (mana, spell, magic)
- Excessive proper nouns in moment-to-moment play

### Log Line Voice
Log messages are the player's primary narrative window. They should be:
- **Terse but evocative:** "The west wing *refracts*... corridors realign."
- **Directional:** "A mirage hound to the north attacks you."
- **Consequential:** "Sharp glass cuts you! (-1 HP, +1 Refraction)"

**Example Log Lines (On-Tone):**
```
⚡ GLASS STORM! Intensity 3
The salt flats shimmer and twist.
A new glass seam forms to the south.
You gain Prismhide! Your skin catches light.
The Mirror Monk regards your crystalline hands with reverence.
Archive Drone scans you. Credentials: INVALID.
```

**Example Log Lines (Off-Tone — Avoid):**
```
Storm happened! Map changed!
You got a mutation lol
The robot is mad at you
```

---

## 4. Visual Style Guide (TUI)

### Color Palette

| Element | Color | Hex/ANSI | Rationale |
|---------|-------|----------|-----------|
| Player | Yellow | `Color::Yellow` | Warm, human, stands out |
| Walls | Gray | `Color::Gray` | Neutral, structural |
| Floor | Dark Gray | `Color::DarkGray` | Recedes, doesn't compete |
| Glass terrain | Cyan | `Color::Cyan` | Cool, dangerous, refractive |
| Enemies (general) | Red | `Color::Red` | Danger, hostility |
| Mirage Hound | Light Yellow | `Color::LightYellow` | Heat shimmer, camouflage |
| Glass Beetle | Cyan | `Color::Cyan` | Matches glass, refractive |
| Salt Mummy | White | `Color::White` | Desiccated, pale |
| Items | Light Magenta | `Color::LightMagenta` | Valuable, distinct |
| Adaptations | Magenta | `Color::Magenta` | Mutation, transformation |
| Storm warning | Red (≤3) / Orange (4-5) / Yellow (6+) | Contextual | Three-stage urgency |
| HP (low) | Red | `Color::Red` | Danger state |
| HP (healthy) | Green | `Color::Green` | Safe state |
| Refraction meter | Cyan | `Color::Cyan` | Matches glass theme |

### Glyph Conventions

| Element | Glyph | Notes |
|---------|-------|-------|
| Player | `@` | Traditional roguelike |
| Wall | `#` | Solid, opaque |
| Floor | `.` | Open, walkable |
| Glass terrain | `*` | Dangerous, walkable, refractive |
| Mirage Hound | `h` | Lowercase = creature |
| Glass Beetle | `b` | |
| Salt Mummy | `m` | |
| Storm Glass (item) | `◆` | Unicode diamond = valuable |
| Scripture Shard | `?` | Mystery, lore |
| Brine Vial | `!` | Consumable (roguelike tradition) |
| Saint-Key | `⚷` | Unicode key |
| Angle-Split Lens | `◎` | Circular = optical |
| Fog of war | `~` | Uncertainty, memory |
| Unexplored | ` ` (space) | True unknown |

### Future Glyph Reservations

| Element | Proposed Glyph | Notes |
|---------|----------------|-------|
| Beam/ray | `-` `|` `/` `\` | Directional lines |
| Mirror surface | `/` `\` | Reflective walls |
| Glare tile | `░` | Light hazard |
| Storm shimmer | `≈` | Active storm overlay |
| Archive Drone | `Δ` | Geometric, artificial |
| Refraction Wraith | `◊` | Ethereal, storm-born |

---

## 5. Narrative Themes

### Primary Themes

1. **Transformation and Identity**
   - Your body changes; society's perception of you changes.
   - Adaptations are power and burden simultaneously.

2. **Impermanence and Adaptation**
   - The map changes. Plans fail. Survival requires flexibility.
   - "The only constant is the storm."

3. **Faith vs. Technology**
   - Mirror Monks see storms as scripture; Engineers see problems to solve.
   - Neither is wholly right or wrong.

4. **Legacy and Archaeology**
   - The world is built on ruins. Every relic has a history.
   - The past is not dead; it's fused into the glass.

### Faction Voices

| Faction | Voice | Example Dialogue |
|---------|-------|------------------|
| Mirror Monks | Cryptic, reverent, prophetic | "The angle speaks. Do you hear it?" |
| Sand-Engineers | Practical, terse, problem-focused | "Well's dry. Need parts from the Crucible Block." |
| Glassborn | Transformed, alien pride, storm-touched | "You flinch at the shimmer. We were born in it." |
| Archive Drones | Procedural, cold, protocol-bound | "CREDENTIAL CHECK. STATUS: UNAUTHORIZED. COMPLIANCE REQUIRED." |

---

## 6. Current Implementation Assessment

### What's Working

✅ **Core TUI structure:** Map view, status bar, log panel are functional and readable.  
✅ **Color scheme:** Consistent use of Yellow (player), Cyan (glass), Red (danger), Magenta (items).  
✅ **Thematic naming:** Items and enemies have evocative, setting-appropriate names.  
✅ **Storm system:** Basic map editing (walls → glass) is implemented.  
✅ **Adaptation system:** Four adaptations with mechanical effects exist.  
✅ **Log messaging:** Directional combat messages, storm announcements are on-tone.

### Gaps to Address

| Gap | Priority | Notes |
|-----|----------|-------|
| No shimmer/glare visual effects | High | Core to the "light is dangerous" pillar |
| No beam/ray visualization | High | Required for "readable light tactics" |
| Limited enemy variety | Medium | Only 3 of 5+ designed enemies implemented |
| No faction presence | Medium | No NPCs, reputation, or faction quests |
| Minimal storm forecast display | Medium | Should show edit types, not just countdown |
| No post-storm diff highlighting | Medium | Changed tiles should be marked until visited |
| Color palette not codified in code | Low | Colors are inline; should be constants |
| No glare/hot light tiles | Low | Designed but not implemented |

---

## 7. Actionable Guidelines

### For Developers

1. **Extract color constants:** Create a `colors.rs` module with named constants matching this document.
2. **Implement shimmer overlay:** During storms, overlay `≈` or color shift on affected tiles.
3. **Add post-storm diff:** Track tiles changed by storms; render in distinct color until player visits.
4. **Beam visualization:** When implementing beam weapons, use `-|/\` characters with Cyan color.

### For Content Creators

1. **Enemy design:** New enemies must have:
   - Thematic name (no generic "goblin" types)
   - Unique glyph (lowercase letter or Unicode)
   - Color that communicates threat type
   - Log lines for attack, death, and special abilities

2. **Item design:** New items must have:
   - Name that implies function or origin
   - Glyph that suggests category (! = consumable, ? = lore, ◆ = valuable)
   - Description under 60 characters

3. **Quest design:** Quests must:
   - Reinforce at least one pillar
   - Have clear TUI communication (log lines for progress)
   - Create meaningful choice (not just "go here, kill that")

### For Writers

1. **Log lines:** Max 60 characters. Use em-dashes for dramatic pauses. Include direction when relevant.
2. **Descriptions:** Evocative but brief. "Crystallized storm energy" not "A piece of glass from a storm."
3. **Faction voice:** Maintain distinct voices per faction. Monks are cryptic; Engineers are practical.

---

## 8. Content Approval Checklist

Before adding new content, verify:

- [ ] Reinforces at least one pillar
- [ ] Does not contradict any pillar
- [ ] Has clear TUI representation (glyph, color, log lines)
- [ ] Fits the tone (mythic-reverent, not comedic or grimdark)
- [ ] Creates interesting choice or consequence
- [ ] Naming follows vocabulary rules
- [ ] No "reference soup" (inspirations are invisible)

---

## 9. Visual Mockups

### Status Bar (Current)
```
┌─ HP:15/20 | Ref:25 | Turn 42 | Storm:3 | Prismhide ─┐
```

### Status Bar (Target)
```
┌─ HP:15/20 | Ref:◁◁◁ | Turn 42 | Storm:3 [ROTATE/SWAP] | Prismhide ─┐
```

### Storm Forecast Panel (Target)
```
┌─ Storm Forecast ─┐
│ Turns: 3         │
│ Intensity: ███░░ │
│ Edits: ROTATE    │
│        GLASS     │
└──────────────────┘
```

### Post-Storm Diff (Target)
Changed tiles rendered in `Color::LightCyan` with `*` until player visits them.

---

## 10. First 5 Minutes

What does the player see, hear, and do that tells them this isn't generic fantasy?

### Opening Moments
1. **Storm warning on screen** — Before the player moves, they see "Storm: 7 turns" in the status bar. The world is already counting down.
2. **Glass terrain visible** — Cyan `*` tiles are present from the start. Walking on them hurts and increases Refraction.
3. **First log message** — Something evocative: "The salt wind carries glass dust. You taste copper."

### First Encounter
- A **Mirage Hound** (`h` in Light Yellow) shimmers at the edge of FOV. It doesn't attack immediately—it watches.
- Combat log: "The mirage hound flickers closer." Not "The enemy moves."

### First Storm (Turn ~10-15)
- Warning escalates: Yellow → Orange → Red
- Storm hits: "⚡ GLASS STORM! The west corridor *refracts*..."
- Map visibly changes. Walls become glass. New paths open.
- Player gains Refraction. If threshold crossed: "Your skin catches light. You've adapted."

### First NPC (Target)
- A **Mirror Monk** stands in a room. Dialogue varies based on player's adaptations:
  - No adaptations: "You walk unmarked. The storm has not yet spoken to you."
  - With Prismhide: "Your skin refracts. The angle has chosen you."

### What This Communicates
- The world is hostile and strange (glass hurts, storms change maps)
- You are changing (Refraction meter, adaptations)
- Others notice your change (NPC reactions)
- This is not Nethack with a coat of paint

---

## 11. Reference Touchstones

These are internal references for the team—never visible to players.

**Tone:**
- Caves of Qud (mechanical depth, procedural lore)
- Roadside Picnic (alien artifacts, zone logic)
- Dune (desert survival, religious factions)
- Annihilation (transformation, the shimmer)

**Visual:**
- Dwarf Fortress (ASCII clarity, information density)
- Brogue (color as meaning, elegant TUI)
- Cogmind (modern TUI aesthetics)

**NOT references:**
- Generic fantasy RPGs
- Comedic roguelikes (DCSS humor, Nethack jokes)
- Pixel art games (we are TUI, not retro graphics)

---

## 12. Appendix: Glossary

| Term | Definition |
|------|------------|
| Refraction | Exposure to glass storms; accumulates toward adaptations |
| Adaptation | Mutation gained from high refraction; grants power, changes social standing |
| Glass Storm | Periodic event that physically rewrites the map |
| Storm Glass | Crystallized storm energy; valuable trade resource |
| Saint-Key | Pre-storm credential that grants Archive access |
| Scripture Shard | Fragment of ancient religious text; lore + crafting component |
| Heliograph | Ancient orbital mirror system; source of storms |
| Crucible | Glassborn rite of passage; also refers to Vex Crucible's legacy |

---

*This document is the creative authority for Saltglass Steppe. All disciplines should reference it for tone, style, and content decisions. Updates require Creative Director approval.*
