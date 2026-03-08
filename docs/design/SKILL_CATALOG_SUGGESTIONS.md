Now I have a thorough picture of the game's lore, systems, and existing skills. Let me synthesize this into a comprehensive skill suggestion list organized by category. I'll draw from:

1. The existing 10 skills in abilities.json
2. The lore: Mirror Monks, Glassborn, Sand Engineers, Salt Traders, Storm Cults, Refraction Outcasts, Archive Drones
3. World elements: glass storms, salt, refraction, void energy, crystal resonance, light manipulation, adaptations, heliograph, saint-keys, brine, scripture shards
4. The psychic abilities already in the data (telepathy, probability, energy, phasing, temporal)
5. Caves of Qud inspiration: stance-based combat, wilderness lore per biome, self-discipline/meditation, tinkering, customs/folklore, physic
6. The game's unique systems: storm editing, adaptation social consequences, light tactics, archive drones

Let me organize by the 5 categories from the implementation plan: Combat, Survival, Psychic, Social, Crafting. I'll also note that "Athletics" exists (Mirage Walking/stealth) and could be renamed or merged.

I want to be exhaustive but each entry can be brief - just a name, brief concept, and maybe a note on what it hooks into.

Let me think about what's unique to this game's lore that could inspire skills:

COMBAT:

- Blade Mastery (exists) - melee damage/accuracy
- Shard Casting (exists) - ranged
- Salt Ward (exists) - defense/armor
- Glass Fighting - bonus vs glass/crystal enemies, glass weapon proficiency
- Storm Stance - stance that changes based on storm proximity (inspired by CoQ's Long Blade stances)
- Crucible Technique - Glassborn fighting style, uses refraction adaptations as weapons
- Seam Breaker - breaking through glass walls, terrain destruction
- Counter-Refraction - deflect light-based attacks back at enemies
- Void Strike - melee attack that drains enemy stamina/energy
- Angle Reading - predict enemy attacks (ties into Mirror Monk lore of "the angle reveals all")
- Salt Flurry - rapid attacks that apply salt/bleed
- Brine Splash - throw brine vials as combat items
- Glass Shard Throw - improvised ranged from environment
- Adaptation Combat - using your own mutations as weapons (Glassborn style)

SURVIVAL:

- Desert Conditioning (exists) - heat/cold resistance
- Wayfaring (exists) - encounter reduction
- Steppe Lore (exists) - map reveal
- Flesh Mending (exists) - healing
- Salt Hermit Ways - extreme endurance, reduced resource consumption (inspired by CoQ's Fasting Way/Mind over Body)
- Storm Reading - predict storm patterns, reduce storm damage
- Brine Distillation - extract water from salt/brine sources
- Scavenging - find items in ruins, better loot from chests
- Wound Packing - field medicine with salt (painful but effective, lore-accurate)
- Dust Walking - move through storm glass terrain without damage
- Shelter Building - create temporary storm shelters
- Biome Lore (per biome) - inspired by CoQ's Wilderness Lore per terrain type
  - Salt Flats Lore
  - Mirror Canyon Lore
  - Singing Dunes Lore
  - Glassed Reef Lore
  - Brine Under Lore
- Corpse Reading - learn about enemies from their remains
- Adaptation Resistance - slow refraction gain rate

PSYCHIC (new category):

- Storm Sense - feel approaching storms, predict edit types
- Void Attunement - void energy capacity, void ability potency
- Refraction Affinity - refraction gain rate, adaptation threshold reduction
- Glass Resonance - communicate with storm glass, detect glass items
- Archive Interface - mental connection to Archive Drone systems
- Heliograph Echo - receive fragments of the orbital mirror's transmissions
- Angle Sight - see around corners via light refraction (Mirror Monk technique)
- Probability Weaving - small luck manipulation (ties into existing probability psychic abilities)
- Temporal Sense - feel time distortions from storms
- Collective Memory - access memories stored in glass/salt formations
- Mind Fortress - resist psychic effects, mental status effects
- Void Dreaming - access void-space during rest for information/visions

SOCIAL:

- Salt Trading (exists) - buy/sell prices
- Faction Lore - reputation gain rate, dialogue options
- Pilgrim Courtesy - road etiquette, safe passage
- Intimidation - enemy morale, flee chance
- Monk Rhetoric - speak in angles and scripture, Mirror Monk reputation
- Engineer Rapport - technical language, Sand Engineer reputation
- Glassborn Kinship - acknowledge transformation bond, Glassborn reputation
- Archive Protocol - machine-saint authority, Archive Drone interactions
- Storm Cult Rites - storm worship language, Storm Cult reputation
- Outcast Signs - hidden language of Refraction Outcasts
- Trader's Tongue - general commerce, haggling
- Reputation Weaving - manage multiple faction relationships simultaneously
- Berate/Shame - demoralize enemies (inspired by CoQ's Berate)
- Proselytize - convert NPCs to your cause

CRAFTING:

- Glass Shaping (exists) - craft glass weapons/armor
- Salt Alchemy - potion/tincture crafting
- Lens Grinding - optical instruments (angle-split lens, storm compass)
- Storm Forging - use storm energy to enhance equipment
- Saint-Key Forgery - create/copy Archive credentials
- Brine Preservation - preserve items, extend durability
- Scrap Salvage - extract components from broken items (inspired by CoQ's Tinkering/Disassemble)
- Adaptation Tinctures - craft veil tinctures and adaptation-affecting items
- Crystal Tuning - work with crystal resonance items
- Prefab Traps - set glass shard traps, salt mines
- Mechanical Repair - fix Sand Engineer devices
- Void Glass Working - work with void-touched materials

Now let me also think about what's interesting from a design perspective that's unique to this game:

1. The "angle" concept from Mirror Monks - geometry as mysticism
2. Storm editing types (Glass, Rotate, Swap, Mirror, Fracture, Crystallize, Vortex) - could inspire skills
3. The social consequence of adaptations - skills that help manage/exploit this
4. Archive Drones following learnable rules - skills to exploit this
5. Light as tactical element - skills around light manipulation
6. The Heliograph network - skills to interface with it
7. Brine/water as currency - skills around water economy
8. Scripture shards as collectibles - skills to find/read them

Let me also think about what CoQ does well that I should borrow:

- Stance-based combat (Long Blade) - could apply to glass fighting
- Wilderness Lore per biome - very thematic for Saltglass
- Customs and Folklore (Trash Divining) - finding secrets in ruins
- Self-discipline/Meditate - could be a Psychic or Survival skill
- Physic (field medicine) - already have Flesh Mending
- Tinkering tiers - could apply to crafting

Now let me write the actual document. I want it to be exhaustive but each entry brief. I'll organize by category and note references where I borrow concepts.

- Completed in 0.9s

> Good, I have everything I need. Here's the full skill suggestion list:

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## Skill Suggestions — Saltglass Steppe

Organized by category. Existing skills marked ✓. References noted inline.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### COMBAT

✓ Blade Mastery — melee damage/accuracy per level. Foundation skill.

✓ Shard Casting — ranged damage/accuracy. Glass projectiles.

✓ Salt Ward — armor/defense. Defensive foundation.

Glass Fighting — bonus damage vs glass/crystal enemies; proficiency with
glass-edged weapons specifically. Glassborn enemies are the most common
threat, so this has constant relevance. Prereq: Blade Mastery 2.

Storm Stance — stance system (inspired by CoQ Long Blade's aggressive/
defensive/dueling stances). Storm Stance: near an active storm, gain
+damage but -accuracy. Calm Stance: away from storms, gain +accuracy and
+dodge. Switches automatically based on storm proximity. Prereq: Blade
Mastery 3.

Angle Reading — Mirror Monk combat philosophy: "the angle reveals all."
Passive chance to predict enemy attack direction, granting a dodge bonus.
Active: spend a turn "reading" an enemy to reveal their next action.
Ties into the lore of Kael-of-Angles and sacred geometry.

Crucible Technique — Glassborn fighting style named after Vex Crucible.
Uses refraction adaptations offensively — each adaptation you have adds a
small damage bonus. High refraction players become more dangerous in
melee. Prereq: Blade Mastery 3, requires at least 1 adaptation.

Seam Breaker — trained use of the Glass Pick as a weapon and tool. Break
through glass walls in combat, create chokepoints, deal bonus damage to
crystallized/vitrified enemies. Ties into the Glass Pick item lore (
"every swing risks shattering").

Counter-Refraction — deflect light-based attacks back at the source.
Passive: chance to reflect light damage. Active: hold a glass shard up to
redirect a beam. Prereq: Salt Ward 3. Ties into the light tactics
system.

Void Strike — melee attack that drains enemy stamina/energy rather than
HP. Useful against high-armor targets. Prereq: requires Void Attunement (
Psychic) 2. Cross-category synergy.

Salt Flurry — rapid series of light cuts that apply a salt-wound debuff (
stacking bleed/armor reduction). Inspired by CoQ's Bloodletter (short
blade bleeding stacks). Prereq: Blade Mastery 4.

Brine Splash — throw a brine vial as a combat action: corrodes armor,
applies salt-wound. Turns a healing item into a tactical weapon. Lore:
"the salt seals what the storm opened" — here it opens instead.

Suppressive Shards — fire glass shards to pin an enemy in place (
suppression mechanic). Inspired by CoQ's Suppressive Fire. Prereq: Shard
Casting 3.

Draw a Bead — mark a target; subsequent ranged attacks against that
target gain accuracy bonus. Inspired directly by CoQ's Draw a Bead / Sure
Fire chain. Prereq: Shard Casting 2.

Fracture Strike — active ability: hit causes the "fractured" status —
enemy takes bonus damage from the next hit. Thematic: fracturing is one
of the storm edit types.

Vortex Footwork — movement-based combat: moving through an enemy's tile
deals damage and repositions both. Named after the Vortex storm edit
type. Prereq: Blade Mastery 5.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### SURVIVAL

✓ Desert Conditioning — resource efficiency, heat/cold resistance.

✓ Wayfaring — encounter reduction, flee bonus.

✓ Steppe Lore — map reveal, POI detection.

✓ Flesh Mending — healing item effectiveness.

Salt Hermit Ways — extreme endurance: hunger/thirst rate halved, stamina
regen while resting doubled. Inspired by CoQ's Fasting Way and Mind over
Body. Lore: "the hermits survive because they're already half-salt."
Prereq: Desert Conditioning 3.

Storm Reading — predict storm arrival turns in advance, reduce storm
damage taken, eventually predict which edit type is coming. Prereq:
Desert Conditioning 2. Feeds into the storm forecast UI.

Dust Walking — move through glass shard terrain without taking damage;
reduced movement penalty in storm conditions. Prereq: Desert Conditioning 2.

Brine Dowsing — locate hidden water sources beneath the salt. Reveals
brine wells and water caches on the map. Lore: "Sand-Engineers discovered
the old aquifers still held water." Prereq: Steppe Lore 2.

Wound Packing — field medicine using salt: painful but fast. Instant HP
recovery at the cost of a small armor penalty for a few turns (salt in
the wound). Lore-accurate to how brine vials work. Prereq: Flesh Mending 2.

Scavenging — better loot from ruins and chests; chance to find items when
moving through rubble tiles. Inspired by CoQ's Scavenger (Tinkering).
Prereq: Steppe Lore 2.

Corpse Reading — examine a dead enemy to learn its weaknesses, faction
affiliation, and what killed it. Inspired by CoQ's Butchery (knowledge
from remains). Standalone skill.

Adaptation Resistance — slow the rate of refraction gain; reduce the
social penalty of visible adaptations. For players who want power without
the social cost. Prereq: Desert Conditioning 3.

Biome Lore: Salt Flats — doubled interesting encounters in Salt Flats;
reduced encounter rate with hostile fauna; faster overworld travel.
Inspired directly by CoQ's Wilderness Lore per terrain type — one skill
per biome is a clean pattern.

Biome Lore: Mirror Canyons — same pattern. Canyon-specific: reduced
chance of ambush, bonus to detecting hidden paths.

Biome Lore: Singing Dunes — dune-specific: reduced storm damage in dune
biome, chance to hear approaching enemies before they appear.

Biome Lore: Glassed Reefs — reef-specific: navigate glass formations
without injury, find glass-embedded loot.

Biome Lore: Brine Under — underground-specific: reduced disorientation,
detect brine pools and underground structures.

Meditate — rest action that triples HP/stamina regen; negative status
effects clear faster. Interrupted by damage. Inspired by CoQ's Meditate (
Self-discipline). Could be a Survival or Psychic skill — fits either.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### PSYCHIC (new category)

Storm Sense — passive: feel storm arrival N turns earlier per level.
Active: spend stamina to get a detailed forecast (intensity, edit type,
direction). Foundation skill for the category.

Void Attunement — increase void energy capacity; void abilities cost
less. Passive: void exposure builds more slowly. Foundation skill. Ties
into the existing void energy system.

Refraction Affinity — increase refraction gain rate (for players who want
to mutate); reduce the adaptation unlock threshold. Prereq: requires at
least 1 adaptation. Glassborn/Outcast flavor.

Glass Resonance — feel the "memory" stored in storm glass. Active:
examine a glass shard to learn where it came from (biome, storm type).
Passive: detect glass items through walls at short range. Lore: "every
piece of storm glass serves as a neuron in a vast mind" (The Refraction
Heresy).

Angle Sight — Mirror Monk technique: see around corners by reading light
angles. Extends FOV around corners. Active: briefly see through a glass
wall. Prereq: Storm Sense 2. Lore: "the angle reveals all truths."

Archive Interface — mental attunement to Archive Drone protocols. Passive
: Archive Drones are slower to aggro. Active: attempt to send a command
to a nearby drone (success based on skill level). Lore: "the machines
cannot tell the difference" between a saint-key and legitimate authority.

Heliograph Echo — receive fragments of the orbital mirror's
transmissions. Passive: occasionally receive cryptic lore fragments as
messages. Active: attempt to "tune in" to a specific Archive node for
information. Lore: "the Heliograph Network was never destroyed — it was
awakened."

Probability Weaving — small luck manipulation. Passive: critical hit
chance slightly increased. Active: spend coherence to guarantee the next
action succeeds (or force an enemy to fail their next save). Ties into
existing probability_shift psychic ability.

Temporal Sense — feel time distortions caused by storms. Passive: never
surprised by storm arrival. Active: briefly slow time (extra action, high
cost). Ties into existing temporal_glimpse psychic ability. Prereq:
Storm Sense 3.

Collective Memory — access memories crystallized in salt and glass
formations. Active: touch a salt pillar or glass formation to receive a
lore fragment or map reveal. Passive: higher chance of finding scripture
shards. Lore: "salt preserves memories in crystalline formations."

Mind Fortress — resist psychic status effects (confusion, fear, mind
control). Passive: mental MA bonus. Active: purge a mental status effect
instantly. Inspired by CoQ's Iron Mind and Lionheart (Self-discipline).
Prereq: Void Attunement 2.

Void Dreaming — during rest, enter a void-space vision. Reveals a random
nearby POI or enemy position. High-level: receive a quest hint or faction
secret. Risk: occasionally attracts void-touched enemies. Prereq: Void
Attunement 4.

Crystal Attunement — attune to crystal resonance frequencies. Passive:
crystal formations near you grow faster. Active: resonate with a crystal
to gain a temporary buff based on its frequency. Ties into the existing
crystal resonance system.

Refraction Veil — hide your adaptations from NPC detection. Active:
temporarily suppress the social penalty of visible adaptations. Lore:
"the veil lets you pretend you're still fully human." Already exists as
an ability — could be the capstone of this skill.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### SOCIAL

✓ Salt Trading — buy/sell price improvement.

Faction Lore — reputation gain rate increased; unlock additional dialogue
options with all factions. Foundation social skill.

Pilgrim Courtesy — speak the old road courtesies. Passive: neutral NPCs
start with slightly better disposition. Active: invoke pilgrim right-of-
way to avoid a hostile encounter. Lore: "Pilgrim Words" already exists as
an ability.

Intimidation — enemy morale checks; chance to make enemies flee without
combat. Inspired by CoQ's Intimidate and Menacing Stare. Prereq: Faction
Lore 2.

Monk Rhetoric — speak in angles and scripture. Unlocks Mirror Monk-
specific dialogue; faster reputation gain with Mirror Monks. Active:
invoke a scripture quote to calm a hostile Monk. Prereq: Faction Lore 2.

Engineer Rapport — speak practically about infrastructure and tools. Sand
Engineer reputation bonus; access to Engineer-exclusive trade goods.
Active: propose a trade deal that benefits both parties. Prereq: Faction
Lore 2.

Glassborn Kinship — acknowledge the bond of transformation. Glassborn
reputation bonus; they react less hostilely to your adaptations. Active:
invoke Crucible lineage (requires high refraction or specific adaptation)
. Prereq: Faction Lore 2, requires 1 adaptation.

Outcast Signs — learn the hidden language of Refraction Outcasts. Unlock
Outcast-specific quests and safe houses. Active: leave a sign that marks
a location as safe for Outcasts (affects future NPC spawns). Prereq:
Faction Lore 3.

Archive Protocol — invoke machine-saint authority. Archive Drones treat
you as a legitimate user. Active: issue a stand-down command to a drone.
Prereq: Faction Lore 4. Ties into the Saint-Key lore.

Storm Cult Rites — speak the language of storm worship. Storm Cult
reputation bonus; they share storm forecasts with you. Active: perform a
brief rite to appease a Storm Cult NPC. Prereq: Faction Lore 2.

Trader's Tongue — general haggling. Unlock a haggle dialogue option at
any merchant; chance to get a better price. Prereq: Salt Trading 3.
Inspired by CoQ's Snake Oiler.

Berate — shame an NPC or enemy into a debuffed state. Inspired directly
by CoQ's Berate (Persuasion). Prereq: Intimidation 2.

Inspiring Presence — your reputation with a faction provides a passive
bonus to nearby allied NPCs. Inspired by CoQ's Inspiring Presence. High-
level social capstone.

Reputation Weaving — manage multiple faction relationships without
penalty. Normally, helping one faction hurts another; this skill reduces
the bleed. Prereq: Faction Lore 4.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### CRAFTING

✓ Glass Shaping — craft success rate and quality.

Salt Alchemy — brew tinctures and potions from salt, brine, and desert
plants. Includes veil tinctures (hide adaptations), brine vials (healing)
, and salt poultices. Foundation crafting skill.

Lens Grinding — craft optical instruments: angle-split lens (reveals
hidden paths), storm compass (navigation), prism shields. Lore: "crafted
by Mirror Monks using storm glass ground to precise angles."

Storm Forging — use storm energy to enhance equipment. Requires being
near an active storm or using stored storm glass. Higher risk, higher
reward. Prereq: Glass Shaping 3.

Brine Preservation — preserve items to prevent degradation; extend the
duration of consumables. Lore: "salt preserves." Prereq: Salt Alchemy 2.

Scrap Salvage — disassemble broken items for components; chance to learn
a recipe from a disassembled item. Inspired by CoQ's Disassemble and
Reverse Engineer (Tinkering). Prereq: Glass Shaping 2.

Adaptation Tinctures — craft items that affect refraction: accelerators (
gain adaptations faster), suppressors (slow adaptation), stabilizers (
lock current adaptation level). High-value, faction-sensitive. Prereq:
Salt Alchemy 3.

Crystal Tuning — work with crystal resonance items. Craft resonance
tuners; attune items to specific frequencies for bonus effects. Prereq:
Glass Shaping 3. Ties into crystal resonance system.

Trap Setting — set glass shard traps, salt mines, and pressure-triggered
brine splashes. Inspired by CoQ's Lay Mine / Set Bomb (Tinkering). Prereq
: Glass Shaping 2.

Mechanical Repair — repair Sand Engineer devices and Archive Drone
components. Unlock Engineer-exclusive crafting recipes. Prereq: Scrap
Salvage 2. Faction-flavored crafting.

Void Glass Working — work with void-touched materials. Craft items with
void properties (phase-through, reality distortion). High-level, requires
Void Attunement (Psychic) 3. Cross-category prereq.

Saint-Key Forgery — create or copy Archive credentials. Extremely high-
value; Archive Drones treat forged keys as legitimate. Lore risk: a
failed forgery triggers alarm states. Capstone crafting skill. Prereq:
Lens Grinding 3, Mechanical Repair 2.

Prefab Shelter — craft a deployable storm shelter from salvaged
materials. Provides storm protection for several turns. Prereq: Scrap
Salvage 2, Desert Conditioning (Survival) 2. Cross-category.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

### NOTES ON SOURCES

- Stance-based combat (Storm Stance, Angle Reading) — inspired by [Caves
  of Qud's Long Blade stances](https://wiki.cavesofqud.com/wiki/Skills)
- Wilderness Lore per biome — directly inspired by [CoQ's Wayfaring skill
  tree](https://wiki.cavesofqud.com/wiki/Skills) (Wilderness Lore: Salt
  Dunes, Canyons, etc.)
- Meditate, Mind Fortress — inspired by CoQ's Self-discipline tree (
  Meditate, Iron Mind, Lionheart)
- Berate, Intimidation, Trader's Tongue — inspired by CoQ's Persuasion
  tree
- Scrap Salvage / Reverse Engineer — inspired by CoQ's Tinkering tree
- Corpse Reading — inspired by CoQ's Butchery
- Trap Setting — inspired by CoQ's Lay Mine / Set Bomb
- Draw a Bead / Suppressive Shards — inspired by CoQ's Bow and Rifle tree
- All lore hooks (Heliograph Echo, Glass Resonance, Collective Memory,
  Archive Interface, Angle Sight) — derived from
  docs/narrative/ENTITIES_LORE.md,
  docs/design/Main_Questline/Quest_Spine.md, and The_Refraction_Heresy.md
- Biome Lore skills — thematic fit with the 5 biomes defined in
  CONTENT_EXPANSION_SUMMARY.md
