# 13 Event Systems & World Simulation

## Purpose

Define how the world changes over time, how events trigger other events, and how player actions ripple through the simulation.

---

## Event Categories

### 1. Storm Events (Environmental)

Storms are the heartbeat of the Steppe. They're predictable enough to plan around, chaotic enough to create emergent situations.

**Storm Lifecycle:**

```
Forecast → Approach → Active → Aftermath
   │          │          │          │
   │          │          │          └─ Diff report, new nodes, stormborn fade
   │          │          └─ Map edits execute, visibility reduced, stormborn spawn
   │          └─ Warning intensifies, shelter decision point
   └─ Player receives forecast (1-3 turns advance notice)
```

**Storm Types:**

| Type | Edit Behavior | Hazard | Opportunity |
|------|---------------|--------|-------------|
| Silica Squall | Minor swaps, glass deposits | Sharp terrain | New storm glass nodes |
| Refraction Gale | Rotations, light rule changes | Beam hazards, glare | Storm-locked doors open |
| Crucible Storm | Major swaps, wall fusions | High damage, mutation pressure | Rare stormborn merchants |
| The Pulse | Heliograph-targeted, localized | Extreme in target zone | Access to sealed vaults |

**Gameplay Integration:**
- Storms create **risk/reward decisions**: shelter safely or push through for loot/mutation.
- Storm forecasts are a **Monk service** — good Monk rep means better warnings.
- Engineers sell **storm anchors** that reduce edit severity in a radius.

---

### 2. Faction Events (Political)

Factions act on timers and triggers. The player can influence, accelerate, or prevent these events.

**Event Types:**

| Event | Trigger | Player Options |
|-------|---------|----------------|
| Glassborn Raid | Timer (every 5-7 days) | Defend target, warn target, join raid, ignore |
| Engineer Expansion | Resources + time | Help build, sabotage, claim site first |
| Monk Prophecy | Storm pattern match | Investigate, sell info, debunk |
| Archive Sweep | Player "heat" or timer | Evade, hack, fight, use credentials |
| Faction War Escalation | Repeated conflicts | Broker peace, pick a side, profit from chaos |

**Consequence Chains:**

Example: Glassborn raid an Engineer well.
```
Raid succeeds → Well falls → Water prices spike in region
                          → Engineers offer revenge quest
                          → Glassborn gain territory (new safe zone)
                          → Monks see it as "the Steppe's will" (no action)

Raid fails    → Glassborn warband weakened → Rival warband gains power
                                           → Engineers fortify (harder to raid later)
```

**Player Agency:**
- Intervening changes outcomes.
- Ignoring events lets the simulation run — the world changes without you.
- Some events are **time-limited** — miss the window and the outcome is locked.

---

### 3. Personal Events (Character)

Events tied to the player's state: mutation level, reputation, inventory.

**Mutation Thresholds:**

| Refraction Level | Event |
|------------------|-------|
| 25% | First adaptation choice offered |
| 50% | Visible mutations; NPC reactions change |
| 75% | Glassborn recruitment attempt; Monk "reading" offered |
| 100% | Transformation event (endgame path unlock or death spiral) |

**Reputation Triggers:**

| Condition | Event |
|-----------|-------|
| Allied with Monks | Invited to the Inner Reflection (secret location) |
| Allied with Engineers | Offered a "stake" in a new outpost |
| Respected by Glassborn | Challenged by a Crucible claimant |
| High Archive heat | Drone assassination squad dispatched |
| Oathbreaker tag | Bounty hunters from betrayed faction |

**Inventory Triggers:**

| Item | Event |
|------|-------|
| Carrying Crucible relic | Glassborn demand it or offer to buy |
| Carrying scripture shard | Monks offer trade; Glassborn offer theft job |
| Carrying Saint-Key | Archive scans intensify; Engineers offer to "borrow" it |
| Carrying contraband tech | Multiple factions interested (auction opportunity) |

---

## Event Interconnection

Events don't happen in isolation. The system tracks **world state** and events respond to it.

### World State Variables

```
storm_intensity: 0-100 (current storm activity in region)
faction_control[region]: which faction dominates
water_supply[region]: abundance/scarcity
archive_alert_level: 0-3 (how aggressive drones are)
player_heat[faction]: how much attention you've drawn
```

### Event Triggers (Examples)

```
IF water_supply[Last Salt] < 30 
   AND faction_control[Last Salt] == Engineers
THEN trigger "Desperate Measures" quest (Engineers ask for risky salvage)

IF player_heat[Glassborn] > 50
   AND player is in Glassborn territory
THEN trigger "Champion Challenge" event

IF storm_intensity > 70
   AND player has Monk ally
THEN Monk sends "storm reading" message with bonus forecast info
```

---

## The Heliograph Layer (Endgame Events)

As the player progresses, they discover the Heliograph is still active. This unlocks a new event category.

### Heliograph Nodes

Scattered across the Steppe are **control nodes** — ancient terminals that can influence storm behavior.

| Node Type | Effect When Activated |
|-----------|----------------------|
| Dampener | Reduces storm intensity in radius (Engineers want this) |
| Amplifier | Increases storm intensity (Glassborn want this) |
| Redirector | Changes storm targeting (Monks want to "read" the new patterns) |
| Kill Switch | Shuts down Heliograph sector (Archive default protocol) |

### Endgame Event Chains

Activating nodes triggers faction responses:

```
Player activates Dampener
  → Engineers celebrate, offer alliance rewards
  → Glassborn declare player an enemy
  → Monks are conflicted (prophecies now "wrong")
  → Archive attempts to "correct" the change
```

The endgame is about **choosing which events to enable** — the player shapes what kind of Steppe they leave behind.

---

## Event Communication (TUI)

Events must be readable in a text interface.

### Event Log Format

```
[STORM] Silica Squall approaching from the east. ETA: 3 turns.
[FACTION] Glassborn warband spotted near Dry Fork well.
[PERSONAL] Your Prismhide glints in the light. A merchant eyes you warily.
[WORLD] Water prices in Last Salt have increased. (Well raid succeeded.)
```

### Event Panel

A dedicated UI panel shows active/upcoming events:

```
┌─ ACTIVE EVENTS ─────────────────────┐
│ ◈ Storm: Refraction Gale (2 turns)  │
│ ◈ Quest: Defend Dry Fork (expires)  │
│ ◈ Alert: Archive sweep in Reefs     │
└─────────────────────────────────────┘
```

### Forecast Integration

Storm forecasts include predicted events:

```
┌─ STORM FORECAST ────────────────────┐
│ Type: Refraction Gale               │
│ Edits: ROTATE, LIGHT-CHANGE         │
│ Intensity: High                     │
│ Predicted: Storm-locked doors open  │
│            Stormborn merchant spawn │
└─────────────────────────────────────┘
```

---

## Event Pacing

### Per-Session Rhythm

A typical play session (30-60 min) should include:
- 1-2 storm events (at least one with meaningful map edits)
- 1 faction event (raid, quest offer, reputation consequence)
- 1-2 personal events (mutation threshold, inventory trigger)

### Escalation Over Time

Early game:
- Storms are mild (minor edits, low mutation pressure).
- Faction events are local (single outpost, single warband).
- Personal events are introductory (first mutation, first faction contact).

Mid game:
- Storms intensify (major edits, storm-locked content).
- Faction events have regional impact (territory changes, war escalation).
- Personal events force commitment (faction alliance, mutation path).

Late game:
- Heliograph events unlock (player can influence storm system).
- Faction events respond to player's accumulated choices.
- Personal events culminate (transformation, leadership, or destruction).

---

## Design Principles

1. **Events are opportunities, not interruptions.** Every event should offer a choice or reward.

2. **Consequences are visible.** When an event resolves, the player sees what changed (prices, territory, NPC attitudes).

3. **Ignoring is a choice.** The world moves without the player — but the player can always catch up or adapt.

4. **No "gotcha" moments.** Major events are forecasted. Surprises come from *combinations*, not hidden triggers.

5. **Events reinforce pillars.** Storms rewrite maps. Mutations have social cost. Factions matter.
