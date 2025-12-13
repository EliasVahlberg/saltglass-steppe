## Saltglass Steppe --- Expanded Concept (TUI Roguelike RPG)

### High concept

A desert called the Saltglass Steppe was once a coastal metropolis belt. Repeated "glass storms" (silica cyclones + ancient orbital light) fused buildings, bones, and machines into glittering vitrified reefs. The result is a world where:

- Light is dangerous (refraction burns, mirage predators, signal-cults).
- Maps are unreliable (storms physically rewrite layouts).
- Identity is mutable (you adapt to storms; society judges your "refraction caste").

You play a scavenger, pilgrim, or outlaw who survives by learning the Steppe's physics and politics.

---

## Core gameplay loop

1.  Take rumors/contracts in a settlement (caravanserai, monastery, salt-port, scrapyard).
2.  Travel overmap through dunes, glass flats, and storm corridors (resource + encounter management).
3.  Enter a site (glassed ruin, mirror canyon, saint-tomb, data well).
4.  Extract value: relics, water, "scripture shards," storm glass, faction favors.
5.  Return/press deeper depending on risk: storm fronts, injuries, heat, reputation.

This keeps the Qud-like rhythm: curiosity → danger → loot → weird consequence.

---

## World structure (procedural but legible)

### Regions (biomes)

- Salt Flats: open sightlines, snipers, mirage hazards; easy navigation but storms hit hard.
- Glassed Reefs (ruins): dense labyrinths of vitrified towers and fused streets; high loot.
- Mirror Canyons: reflective walls that duplicate sightlines; beam-based enemies; navigation puzzles.
- Singing Dunes: sound triggers encounters; stealth is inverted (noise attracts/repels different things).
- The Brine Under: underground salt aquifers + drowned metro tunnels; water-rich, disease-heavy.

### Sites (dungeon archetypes)

- Refraction Cathedra: a "holy" ruin generating patterned light; puzzle dungeon with beam routing.
- Storm-Locked Vault: doors open only during/after storms; encourages planned risk-taking.
- Crucible Block: fused industrial block with craft components and dormant assembly arms.
- Pilgrim Necropolis: ossified glass-bone spires; social dungeon with rituals and taboo mechanics.
- Signal Warden Spires: old comm towers with active drones; hacking/reputation routes.

---

## Signature mechanic: Glass storms that rewrite the map

Storms are not just weather---they're procedural editors.

### What a storm can do (pick 2--4 per storm)

- Rotate a sub-area (a wing of the ruin rotates 90°; corridors realign).
- Swap room "modules" (two rooms exchange positions; doors now lead elsewhere).
- Fuse walls into glass (creates new line-of-sight lanes; makes combat more lethal).
- Deposit storm glass (new resource nodes, but also sharp terrain).
- Spawn stormborn entities (temporary enemies/merchants/phenomena).
- Change light rules (mirrors strengthen beams; some tiles become "hot light" damage zones).

### Player-facing clarity (TUI-friendly)

- Pre-storm: a forecast bar: `WIND ++ SILICA +++ LIGHT ++ EDIT: ROTATE/SWAP`
- During storm: the map "shimmers" with a simple overlay (e.g., `~` tint), and you get log lines like:
  - "The west wing *refracts*... corridors realign."
  - "A new glass seam forms to the south."
- Post-storm: a diff report (optional) highlights changed rooms in a distinct color until visited.

The goal is "surprising but not unfair": you get warning, and changes are local, not whole-map chaos.

---

## Character progression: Refraction Adaptations (mutations)

You slowly become "storm-shaped." Mutations are powerful but socially expensive.

### The Refraction meter

- Tracks exposure to storms, glassed ruins, and certain relics.
- At thresholds you choose (or risk-random) an adaptation.
- High Refraction also increases certain encounters (hunters, cult attention).

### Example adaptation trees

Prismhide (defensive/utility)

- Prismhide I: +armor vs lasers/heat, -stealth (glints)
- Prismhide II: reflect a % of beam damage
- Prismhide III: emit a brief "flare" to blind (but reveals you on the map)

Sunveins (offense)

- Store "light charge" in sunlight/stormlight
- Spend charge to fire a line-beam (ASCII line targeting)
- Overcharge risks "glass fever" (temporary debuff or unwanted mutation)

Mirage Step (mobility/stealth)

- Create a decoy after moving (enemy targeting confusion)
- Short blink to a visible tile during storm shimmer
- NPCs treat you as "untrustworthy/uncanny" (trade penalties with some factions)

Saltblood (survival/economy)

- Drink brine without sickness
- Excrete salt crystals (crafting reagent, barter)
- Certain predators track you by "salt scent"

### Social consequence: Refraction Caste tags

NPCs and factions react to visible traits:

- Mirror Monks admire Prismhide, fear Mirage Step.
- Sand-Engineers value Saltblood and "practical" adaptations.
- Glassborn raiders respect Sunveins; they try to recruit or harvest you.

---

## Factions (with gameplay roles)

### Mirror Monks (predictive mystics)

- Belief: The future is visible in angled reflections; storms are scriptures.
- Gameplay: Give "reflection riddles" (quests that reveal map edits or hidden doors).
- Conflict: They oppose anyone who "dulls the light" (certain tech/crafts).

### Sand-Engineers (salvagers & builders)

- Belief: The Steppe can be domesticated with infrastructure.
- Gameplay: Crafting stations, upgrades, bridge-building, storm shelters.
- Conflict: Hunted by Glassborn; distrust Monks.

### Glassborn Raiders (storm-adapted warbands)

- Belief: The Steppe chooses the strong; glass is bloodline.
- Gameplay: Dynamic raids, ambushes, rival champions with named gear.
- Conflict: They claim ruins by rite; you can duel, bargain, or infiltrate.

### Archive Drones ("Saint-Librarians")

- Belief: None---just mission logic: preserve data and "authorized relics."
- Gameplay: Patrol beams, scanning cones, hackable; can be reasoned with using credentials.
- Conflict: They "fine" you with violence for carrying certain artifacts.

(Optionally add a 5th): The Brine Choir (understeppe cult) who can "sing" storms into being.

---

## Combat & tactics (TUI strengths)

### Readable "light physics"

- Beams are clear: `----->` lines with ricochets off mirrors `/ \`.
- Glass terrain:
  - Sharp glass: movement damage unless you have boots/traits.
  - Glare tiles: reduce accuracy / increase detection.
  - Meltglass (rare): slows, burns, but can be shaped with tools.

### Enemy roster (examples)

- Mirage Hounds: appear as duplicates; only the "shadow" is real.
- Glass Beetles: turn into reflective shells; bounce your beams back.
- Salt Mummies: dry undead that release blinding salt puffs.
- Refraction Wraiths: spawn during storms; die if you "ground" the light (smoke, dust, darkness).
- Archive Drones: scanning cones, alarm states, hacking risk/reward.

---

## Items, relics, and crafting

### Key resources

- Storm Glass: used for lenses, prisms, blade edges; dangerous to carry (glints attract).
- Saint-Keys (credentials): authorize you with Archive systems.
- Brine Vials: hydration + crafting; can be contaminated.
- Scripture Shards: lore fragments that also act as "spell components" for miracles (if you include that system later).

### Crafting themes

- Lenscraft: scope upgrades, beam splitters, "glare filters" (stealth).
- Sheltercraft: portable awnings, storm anchors, reflective cloaks (reduce exposure).
- Saltchem: brine bombs, dust clouds (anti-beam), preservation salves.

Crafting is intentionally "weird utility," not just +damage.

---

## Quests that fit the setting (and generate emergent play)

1.  The Map That Lies

    - A settlement pays you to verify a ruin layout---but storms change it.
    - You must produce a "truth map" by triangulating post-storm diffs.

2.  Saint of the Broken Angle

    - Retrieve a lens-relic from a cathedra.
    - Choice: give it to Monks (rep + prophecy), Engineers (tech unlock), or keep it (new mutation path unlock).

3.  Raid Debt

    - Glassborn demand tribute after you loot "their" reef.
    - Resolve by duel, stealth repayment, faction politics, or framing Archive Drones.

4.  The Storm Shelter

    - Build/activate an ancient shelter grid that reduces map edits in a radius.
    - This becomes a strategic hub you can return to, and factions fight over.

---

## Overmap travel & survival (simple but meaningful)

- Heat / hydration: brine helps but increases sickness risk; shade reduces refraction exposure.
- Storm forecasts: choose to shelter, outrun, or deliberately "bathe" in storms to accelerate mutation growth.
- Navigation: compasses can lie near mirror canyons; you can use "sun fixes" or monk techniques to reduce getting lost.

---

## A strong starting "vertical slice" (first 2--4 hours)

### Starting settlement: Last Salt

- Has a well, a small Mirror Monk shrine, an Engineer salvage bay, and a black-market "glint fence."
- Tutorialized choices: accept a monk riddle, an engineer repair job, or a raider tribute demand.

### First dungeon: The Shard Arcade

- A fused shopping complex turned glass maze.
- Teaches:
  - beam reflection hazards,
  - sharp glass terrain,
  - a small storm event that rotates one wing,
  - one named relic at the bottom (e.g., Angle-Split Lens).

### Early "build identity" moment

- After the first storm exposure you pick 1 adaptation (or suppress it using a rare shelter item).
- NPC reactions change immediately (prices, greetings, guards).

---

## TUI presentation ideas that sell the fantasy

- Glare/shine represented with subtle color shifts or glyph changes (`.` becomes `-` becomes `*`).
- Refraction status as a simple triangle meter: `◁◁◁` that fills and changes color.
- Storm edit warnings as a side panel "Storm Behavior" list with icons/short verbs.
- Rumor board as a "cartography ledger": entries like "A door that only exists after a storm" or "A spire that casts two shadows at noon."

---

## Optional: endgame direction (clear goal without railroading)

You learn that the storms are being tuned by an ancient system called the Heliograph. Multiple factions want control:

- Monks want to "read" it (keep storms as scripture),
- Engineers want to stabilize it (terraform),
- Glassborn want to intensify it (selection/ascension),
- Archive wants it locked down.

Winning can be:

- stabilize storms (reduce edits, make world safer),
- ascend (become a storm-saint, embrace max refraction),
- break the Heliograph (unleash chaotic edits, world becomes unrecognizable but free).

---
