# In-Game QA Checklist

> Last updated: 2026-03-08  
> Covers all completed Tier 1 & Tier 2 features.  
> Use debug console (`/`) — `sturdy` for god mode, `show tile` for god view.

---

# Save/Load

- [x] Save game mid-combat, load back — combat state preserved (HP, enemy positions, turn order)
- [x] Save with full inventory, load back — all items present with correct quantities
- [x] Save after completing quest, load back — quest marked complete in log
- [ ] Save after gaining faction reputation, load back — reputation values unchanged
- [x] Save after learning new skill, load back — skill appears in skills menu
- [ ] Save after gaining adaptation/mutation, load back — adaptation active and visible
- [ ] Save at specific map position, load back — player spawns at exact same coordinates
- [ ] Save with specific HP values, load back — health values match exactly
- [ ] Save with active storm effects, load back — storm state and timers preserved
- [x] Save with equipped items, load back — equipment slots filled correctly
- [x] Load corrupted save file — displays clear error message, returns to main menu - edited "(version:2,state:(player:(..." to "(version:2,states:(player:(..." and it crashed the game in a bad way. need to add error handling for save parsing.
- [ ] Load save from different game version — shows version mismatch warning
- [ ] Save and load multiple times in sequence — no data corruption
- [ ] Save during overworld travel, load back — overworld position and travel state preserved

---

# Overworld Travel

- [ ] Move to adjacent tile — movement succeeds, player position updates on world map
- [ ] Attempt move to non-adjacent tile — movement blocked, error message displayed
- [ ] Travel between tiles — turns advance (check turn counter in HUD)
- [ ] Travel 10 tiles rapidly — encounter triggers within expected rate (~25%)
- [ ] Encounter hostile enemy — combat screen opens, can fight or flee
- [ ] Encounter neutral NPC — dialogue options appear, can trade or talk
- [ ] Encounter beneficial event — positive effect applied (items, XP, healing)
- [ ] Flee from hostile encounter — returns to overworld, flee cooldown active
- [ ] Attempt flee during cooldown — flee option disabled or shows cooldown timer
- [ ] Win combat encounter — gain XP reward, return to overworld travel
- [ ] Use arrow keys for fast movement — rapid tile-to-tile movement without menu
- [ ] Enter inspect mode — can examine tiles without moving to them
- [ ] Travel to same tile repeatedly — 50-turn cooldown prevents immediate re-entry
- [ ] Travel after cooldown expires — can re-enter previously visited tile
- [ ] Check encounter distribution over 20+ encounters — roughly 50% hostile, 30% neutral, 20% beneficial

---

# Skill Catalog

- [ ] Open skill tree UI — all 7 categories visible: SaltAlchemy, Crafting, Social, Survival, Medical, MeleeCombat, RangedCombat
- [ ] Pan the skill tree canvas in all directions with arrow keys
- [ ] Upgrade an available skill node — skill points consumed, node updates
- [ ] Attempt to upgrade a locked skill (missing prerequisites) — upgrade blocked
- [ ] Upgrade a melee skill — melee_accuracy_bonus increases in combat
- [ ] Upgrade a ranged skill — ranged_accuracy_bonus increases in combat
- [ ] Passive bonuses from upgraded skills persist after save/load

---

# Faction System

- [ ] Open faction menu — all 7 factions listed: mirror_monks, glassborn, sand_engineers, salt_traders, storm_cults, refraction_outcasts, archive_drones
- [ ] Reputation values displayed with color coding (hostile red → friendly green)
- [ ] Create different character classes — starting reputation varies by class
- [ ] Kill a faction enemy — reputation with that faction decreases
- [ ] Complete a faction quest — reputation with that faction increases
- [ ] With faction reputation ≥ 25 — enemies of that faction do not attack player
- [ ] With faction reputation < 25 — enemies of that faction attack on sight
- [ ] Toggle faction territory overlay on world map — territories color-coded by faction
- [ ] Visit faction shops at different reputation levels — prices change accordingly

---

# Settlement Generation

### Basic Generation

- [ ] Enter Town POI on world map — settlement generates and is enterable
- [ ] Village tier — 80×60 map with 5–10 buildings
- [ ] Town tier — 120×90 map with 15–30 buildings
- [ ] City tier — 180×120 map with 40–80 buildings
- [ ] Buildings do not overlap each other
- [ ] Buildings do not spawn on impassable terrain
- [ ] Roads connect all buildings
- [ ] Settlement has clear entrance/exit points

### NPC Placement

- [ ] Vendors spawn in shops/markets
- [ ] Innkeepers spawn in inns/taverns
- [ ] Guards spawn in town halls/barracks
- [ ] Priests spawn in temples/shrines
- [ ] Crafters spawn in workshops/forges
- [ ] NPCs match building type (no vendors in temples)

### Faction Integration

- [ ] Mirror Monks territory — light_temple, meditation_chamber, scripture_archive appear
- [ ] Glassborn territory — crystal_forge, transformation_clinic, shimmer_gallery appear
- [ ] Dominant faction aesthetic applied to walls and decorations
- [ ] Contested territory — mixed faction buildings present

### Determinism

- [ ] Same seed + same tier → identical layout on repeated runs
- [ ] Same seed + same tier → identical NPC placement
- [ ] Different seeds → different layouts
- [ ] Verify with mapgen-tool: `cargo run --bin mapgen-tool settlement 12345 village`

### Persistence

- [ ] Save inside settlement, load back — layout preserved
- [ ] Save inside settlement, load back — NPCs preserved
- [ ] Exit settlement and re-enter — same layout maintained
- [ ] Multiple settlements in one session — each generates independently

### Edge Cases

- [ ] Settlement with minimal faction presence — only core buildings spawn
- [ ] Generate very small village (5 buildings minimum)
- [ ] Generate very large city (80 buildings maximum)
