# 12 Faction Dynamics & Conflict Web

## Purpose

Define how factions interact with each other and the player, creating emergent conflict and meaningful reputation consequences.

---

## Faction Relationship Matrix

|                  | Mirror Monks | Sand-Engineers | Glassborn | Archive Drones |
|------------------|--------------|----------------|-----------|----------------|
| **Mirror Monks** | —            | Wary Trade     | Hostile   | Indifferent    |
| **Sand-Engineers** | Wary Trade | —              | War       | Exploitative   |
| **Glassborn**    | Hostile      | War            | —         | Prey           |
| **Archive Drones** | Indifferent | Exploitative | Prey      | —              |

### Relationship Types

- **Wary Trade**: Will deal but watch for betrayal. Helping one doesn't hurt the other much.
- **Hostile**: Active conflict. Helping one significantly damages standing with the other.
- **War**: Open violence. Faction members attack on sight in contested zones.
- **Indifferent**: No strong opinion. Archive Drones don't care about Monk theology.
- **Exploitative**: One faction uses the other. Engineers hack Archive systems; Archive doesn't retaliate intelligently.
- **Prey**: One faction hunts the other. Glassborn raid Archive convoys for parts.

---

## Faction Goals & Conflicts

### Mirror Monks

**Goal:** Preserve the storms as divine scripture; read the future in refracted light.

**Conflicts:**
- vs. Engineers: Monks oppose "storm dampening" technology. Engineers see Monks as superstitious obstacles.
- vs. Glassborn: Monks believe mutations should be *received*, not *forced*. Glassborn "storm bathing" is heresy.

**What they want from the player:**
- Recover scripture shards from dangerous ruins.
- Verify or debunk prophecies (map a location, confirm a storm pattern).
- Sabotage Engineer storm-shelter projects (optional, high-commitment).

**What they offer:**
- Storm forecasts (advance warning of map edits).
- "Reflection readings" (hints about hidden locations or quest solutions).
- Sanctuary during storms (their monasteries are safe zones).

---

### Sand-Engineers

**Goal:** Stabilize the Steppe. Build infrastructure. Make the world livable.

**Conflicts:**
- vs. Monks: Engineers want to control storms; Monks want to preserve them.
- vs. Glassborn: Glassborn raid Engineer caravans and sabotage wells. Open war.

**What they want from the player:**
- Salvage pre-storm tech from ruins.
- Protect caravans or outposts from Glassborn raids.
- Scout locations for new wells or shelters.

**What they offer:**
- Crafting stations and upgrades.
- Storm shelters (reduce Refraction exposure).
- Water at fair prices (if reputation is good).

---

### Glassborn Raiders

**Goal:** Strength through adaptation. The Steppe chooses the worthy.

**Conflicts:**
- vs. Everyone: Glassborn raid for resources and "prove" themselves through combat.
- Internal: Rival warbands compete for Crucible legitimacy.

**What they want from the player:**
- Duel their champions (earn respect or die).
- Raid Engineer targets (caravans, wells, outposts).
- Retrieve Crucible relics to legitimize a warband's claim.

**What they offer:**
- Mutation acceleration (they know storm-bathing techniques).
- Safe passage through Glassborn territory (if you're respected).
- Named gear from defeated champions.

---

### Archive Drones

**Goal:** Execute last-known protocols. Preserve authorized data. Protect designated sites.

**Conflicts:**
- vs. Everyone carrying "unauthorized relics" (most valuable loot).
- Not truly a faction — more like environmental hazard with rules.

**What they want from the player:**
- Nothing. They don't negotiate (usually).
- Exception: Valid Saint-Keys grant temporary "authorized" status.

**What they offer (if hacked or credential-bypassed):**
- Access to sealed data wells (lore, maps, quest info).
- Deactivation of local patrols.
- Rare pre-storm tech in Archive vaults.

---

## Reputation System

### Tags

The player accumulates **reputation tags** that NPCs recognize:

| Tag | Source | Effect |
|-----|--------|--------|
| Storm-Reader | Monk quests | Monks trust you; Engineers suspect you |
| Brine-Bonded | Engineer quests | Engineer discounts; Glassborn see you as soft |
| Crucible-Marked | Glassborn duels/raids | Glassborn respect; others fear/distrust |
| Archive-Authorized | Saint-Key use | Drones ignore you (temporarily); factions want your keys |
| Oathbreaker | Betraying any faction | Universal distrust; some NPCs refuse service |

### Thresholds

Reputation is tracked per faction on a scale:

```
Hostile → Distrusted → Neutral → Friendly → Allied
```

**Threshold effects:**
- Hostile: Attacked on sight in faction territory.
- Distrusted: No quests, bad prices, guards watch you.
- Neutral: Basic trade, generic quests.
- Friendly: Good prices, faction-specific quests, safe houses.
- Allied: Faction champions assist you, unique gear/abilities unlocked.

### Zero-Sum Tensions

Some actions are **zero-sum** — helping one faction necessarily hurts another:

| Action | Positive | Negative |
|--------|----------|----------|
| Sabotage a storm shelter | Monks +2 | Engineers -3 |
| Defend an Engineer caravan | Engineers +2 | Glassborn -2 |
| Win a Glassborn duel | Glassborn +1 | (No penalty, but you're now "one of them") |
| Hack an Archive node | Engineers +1 | Archive hostility in that zone |
| Return a scripture shard to Monks | Monks +2 | (Glassborn wanted to sell it: -1 if they know) |

---

## Emergent Faction Events

The world isn't static. Factions act based on conditions:

### Raid Cycles

Glassborn raid Engineer outposts on a timer. If the player doesn't intervene:
- Outpost may fall (services lost, prices spike elsewhere).
- Glassborn gain territory (new safe zones for them, danger zones for others).

### Storm Shelter Projects

Engineers attempt to build shelters that reduce storm intensity in a region. If completed:
- Storms are weaker (fewer map edits, less loot, less mutation exposure).
- Monks may sabotage the shelter (quest opportunity for either side).

### Prophecy Events

Monks occasionally announce prophecies. These are procedurally generated from upcoming storm patterns:
- "A door will open in the Shard Arcade after the next storm."
- If the player investigates, they find a storm-locked vault.
- Ignoring prophecies has no penalty, but following them rewards exploration.

### Archive Sweeps

Periodically, Archive Drones expand patrol routes to "reclaim" areas. During a sweep:
- More drones, tighter patrols, higher detection risk.
- But also: more hackable nodes, more loot if you survive.

---

## Player Positioning

The player is always an **outsider** at game start. This means:

- No faction is automatically hostile (except Archive, which is hostile to everyone without credentials).
- Early quests from multiple factions let the player sample each path.
- Commitment comes later — zero-sum choices force the player to pick sides (or stay neutral at a cost).

### Neutral Path

It's possible to stay neutral, but:
- No faction-specific gear or abilities.
- Prices are never great.
- Some areas remain inaccessible (faction-controlled zones).

### Multi-Faction Path

Skilled players can balance relationships:
- Do Monk quests that don't hurt Engineers.
- Duel Glassborn without raiding (respect without alliance).
- Use Saint-Keys sparingly to avoid Archive attention.

This is harder but unlocks the most content.

---

## Faction Content Hooks (Quest Seeds)

| Faction | Quest Type | Example |
|---------|-----------|---------|
| Monks | Prophecy verification | "Confirm the Angle Cathedra's east wing rotates after storms." |
| Monks | Relic recovery | "Retrieve Kael's Second Shard from Glassborn hands." |
| Engineers | Salvage run | "Extract a water pump from the Crucible Block." |
| Engineers | Defense | "Protect the Salt Road caravan from raiders." |
| Glassborn | Duel | "Defeat Shard-Eye Keth to earn passage through the Reefs." |
| Glassborn | Raid | "Hit the Engineer well at Dry Fork. We split the brine." |
| Archive | (Indirect) | "Find a Saint-Key to access the Signal Spire data well." |
| Cross-faction | Diplomacy | "Broker a temporary truce for the storm season." |
