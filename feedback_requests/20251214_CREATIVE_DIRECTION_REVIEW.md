# Creative Direction Review — Feedback Request

**Date:** 2025-12-14  
**Author:** CreativeDirector (delegated)  
**Document:** `design_docs/CREATIVE_DIRECTION_SUMMARY.md`

---

## Summary of Additions

The CreativeDirector produced a comprehensive 320-line document covering:

1. **5 Creative Pillars** — Non-negotiable design constraints (Storms Rewrite Maps, Mutation with Social Consequences, Readable Light Tactics, TUI as Aesthetic Strength, Authored Weirdness)

2. **Non-Goals** — What the game explicitly is NOT (not comedic-weird, not grimdark, not graphically complex, not kitchen-sink fantasy, not unfair)

3. **Tone & Voice** — "Mythic-Reverent" (6/10 weird), vocabulary rules (use: refraction, vitrified, glint; avoid: mana, spell, modern slang), log line voice guidelines with on-tone/off-tone examples

4. **Visual Style Guide** — Full color palette (14 elements mapped to ANSI colors with rationale), glyph conventions (current + future reservations for beams, mirrors, glare)

5. **Narrative Themes** — Transformation/Identity, Impermanence, Faith vs Technology, Legacy/Archaeology + faction voice guides for Mirror Monks, Sand-Engineers, Glassborn, Archive Drones

6. **Implementation Assessment** — What's working (6 items) vs gaps to address (8 items with priority ratings)

7. **Actionable Guidelines** — Specific instructions for developers, content creators, and writers

8. **Content Approval Checklist** — 7-point verification before adding new content

9. **Visual Mockups** — Target status bar, storm forecast panel, post-storm diff rendering

10. **Reference Touchstones** — Internal-only inspirations (Qud, Roadside Picnic, Dune, Brogue, Cogmind) and explicit NOT-references

11. **Glossary** — 8 key terms defined (Refraction, Adaptation, Glass Storm, Storm Glass, Saint-Key, Scripture Shard, Heliograph, Crucible)

---

## Feedback Requested

- [ ] Does the tone calibration ("Mythic-Reverent" at 6/10 weird) feel right?
- [ ] Are the 5 pillars the correct priorities, or should any be reordered/replaced?
- [ ] Is the color palette sufficient, or are there missing elements?
- [ ] Any faction voices that feel off or need adjustment?
- [ ] Should any gaps be reprioritized (e.g., faction NPCs higher than beam visualization)?

---

## Related Changes

- Updated `TODO.md` with creative-direction-aligned tasks (High/Medium/Low priority)

---

## Creative Director Response

**Date:** 2025-12-14  
**Reviewer:** CreativeDirector

---

### Tone Calibration (6/10 Weird) — Approved with note

The "Mythic-Reverent" positioning is correct. The examples in the weirdness dial table nail the target zone. One refinement: consider adding a "dread" axis alongside weird. The Saltglass should feel *numinous*—awe mixed with unease. "A lens that belonged to three dead saints" is perfect because it's strange *and* slightly unsettling.

---

### The 5 Pillars — Reorder recommended

Current order buries the player fantasy. Suggested reorder:

1. **Mutation with Social Consequences** — This is the core identity hook ("you are becoming something strange")
2. **Storms Rewrite Maps** — The signature system that makes this game distinct
3. **Readable Light Tactics** — Gameplay clarity
4. **TUI as Aesthetic Strength** — Medium constraint
5. **Authored Weirdness** — Quality bar

The first two are *what players will remember*. Lead with them.

---

### Color Palette — Sufficient, one addition

Missing: **Storm warning gradient**. Currently only Red (≤3) and Yellow are specified. Add Orange (`Color::LightRed` or similar) for the 4-5 turn range. Three-stage urgency reads better than two.

---

### Faction Voices — One adjustment

**Glassborn** voice reads too generic ("Prove you're worth the glass in your veins"). They should sound *transformed*—like people who've internalized the storm. Try: "You flinch at the shimmer. We were born in it." Their pride should be alien, not just tough-guy posturing.

---

### Gap Reprioritization — Yes

Current priorities undervalue faction presence. Recommended change:

| Gap | Current | Recommended |
|-----|---------|-------------|
| No faction presence | Medium | **High** |
| No beam/ray visualization | High | Medium |

Rationale: Pillar 2 (Mutation with Social Consequences) is meaningless without NPCs who *react*. A single Mirror Monk encounter that comments on your Prismhide adaptation does more for the core fantasy than perfect beam rendering. Beams can wait for the light-tactics vertical slice.

---

### Additional Notes

- The log line examples are excellent. "The west wing *refracts*... corridors realign." should be the gold standard.
- The content approval checklist is practical and usable—keep it.
- Consider adding a "First 5 Minutes" section: what does the player see, hear, and do that tells them this isn't generic fantasy?
