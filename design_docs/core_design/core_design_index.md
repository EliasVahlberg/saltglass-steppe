# Core Design Document Index

## Foundation Documents

- [00_Pillars_OnePager.md](00_Pillars_OnePager.md) — Single-page statement of the game's concept, pillars, non-goals, and what "success" means for the vertical slice.
- [01_Creative_Pitch.md](01_Creative_Pitch.md) — High-level setting/tone pitch with key imagery, hooks, factions, and example player stories for alignment and buy-in.

## World & Narrative

- [02_Narrative_Bible.md](02_Narrative_Bible.md) — Canon for factions, locations, terminology, procedural lore rules, and quest/rumor writing guidelines.
- [11_World_History_Timeline.md](11_World_History_Timeline.md) — The three ages of the Steppe, key historical events, and how history creates discoverable gameplay.
- [12_Faction_Dynamics.md](12_Faction_Dynamics.md) — Faction relationships, reputation system, zero-sum tensions, and emergent faction events.
- [14_Regional_Gazetteer.md](14_Regional_Gazetteer.md) — Each region's geography, history, faction presence, and gameplay character.

## Systems Design

- [03_Design_Bible.md](03_Design_Bible.md) — The core gameplay rules: loops, economy, survival, reputation, difficulty philosophy, and systemic interactions.
- [04_Storm_MapEdit_TDD.md](04_Storm_MapEdit_TDD.md) — Implementable specification for storms and map-edit operations, including constraints, determinism, UI needs, and test cases.
- [05_Combat_LightPhysics.md](05_Combat_LightPhysics.md) — Combat rules and content requirements focused on beams/mirrors/glass terrain, enemy archetypes, and balance knobs.
- [06_Progression_Adaptations.md](06_Progression_Adaptations.md) — Full design for the Refraction meter, mutation/adaptation trees, build tradeoffs, and faction/social reactions.
- [13_Event_Systems.md](13_Event_Systems.md) — How storms, faction actions, and player choices interweave; world simulation rules.

## Content & Production

- [07_Content_Kits/](07_Content_Kits/) — Reusable templates and tables for sites, encounters, rumors/contracts, relics, and other repeatable content pipelines.
- [08_TUI_UX_Spec.md](08_TUI_UX_Spec.md) — UI layout, input model, visual language for ASCII readability, accessibility, and interaction flows (targeting, diffs, ledgers).
- [09_Audio_Direction.md](09_Audio_Direction.md) — Audio goals and cue lists to support storms, factions, UI feedback, and atmosphere despite minimal visuals.
- [10_VerticalSlice_Plan.md](10_VerticalSlice_Plan.md) — Milestones and scope contract defining the slice's required content/systems, metrics, risks, and cut list.

---

## Document Relationships

```
                    ┌─────────────────┐
                    │ 00_Pillars      │ (everything flows from here)
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ WORLD LAYER     │ │ SYSTEMS LAYER   │ │ PRODUCTION      │
│ 02_Narrative    │ │ 03_Design_Bible │ │ 07_Content_Kits │
│ 11_History      │ │ 04_Storm_TDD    │ │ 08_TUI_UX       │
│ 12_Factions     │ │ 05_Combat       │ │ 09_Audio        │
│ 14_Gazetteer    │ │ 06_Progression  │ │ 10_Slice_Plan   │
│                 │ │ 13_Events       │ │                 │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ PLAYABLE GAME   │
                    └─────────────────┘
```

## Key Recommendations

Keep these document pairs tightly linked:
- **04_Storm_TDD ↔ 08_TUI_UX**: The signature feature fails if it's not communicated clearly.
- **12_Factions ↔ 13_Events**: Faction dynamics drive the event system.
- **11_History ↔ 14_Gazetteer**: Every location should echo the world's past.
