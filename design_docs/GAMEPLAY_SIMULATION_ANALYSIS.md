# Gameplay Simulation & Experience Analysis

**Date:** December 20, 2025  
**Purpose:** Simulate a complete gameplay session to identify fun factors, friction points, and necessary improvements before implementing storm/adaptation/faction systems.

---

## Simulated Gameplay Sequence (30-45 minutes)

### Phase 1: Game Start & Tutorial (0-5 minutes)

**[1] Game:** Generates world map (64x64) with biomes, elevation, POIs  
**[2] Game:** Places player at starting tile (10, 10) - Scrubland biome, hospitable  
**[3] Game:** Generates first tile from world seed - rooms, corridors, some enemies  
**[4] Player:** Sees HUD - HP: 20/20, AP: 10/10, Reflex: 0, Armor: 0, XP: 0/100  
**[5] Player:** Sees map with FOV radius, few visible tiles, mostly unexplored  
**[6] Game:** No tutorial message, no quest prompt, no guidance

**❌ FRICTION POINT:** Player doesn't know what to do or why  
**❌ FRICTION POINT:** No immediate goal or hook

---

**[7] Player:** Explores randomly using arrow keys (8-directional movement)  
**[8] Game:** FOV updates, reveals rooms and corridors  
**[9] Player:** Finds first item - "brine_vial" (!)  
**[10] Player:** Picks up item (automatic? or press 'g'?)  
**[11] Game:** Item added to inventory, message logged: "You picked up a brine vial"  
**[12] Player:** Continues exploring, reveals more map

**❌ FRICTION POINT:** No indication items are important or what they do  
**✅ POSITIVE:** Exploration reveals things - satisfying discovery loop

---

**[13] Player:** Encounters first enemy - "mirage_hound" (h), 6 tiles away  
**[14] Game:** Enemy AI activates (aggressive demeanor), starts moving toward player  
**[15] Player:** Enemy enters FOV, player sees it approach  
**[16] Player:** Decides to fight (no flee mechanics explained)  
**[17] Player:** Moves adjacent to enemy  
**[18] Player:** Attacks with melee (which key? context unclear)

**❌ FRICTION POINT:** Combat controls not intuitive or explained  
**❌ FRICTION POINT:** No indication of enemy strength vs player strength

---

**[19] Game:** Combat calculation - hit chance: base 75% - enemy reflex 0 = 75%  
**[20] Game:** Roll succeeds, damage: 2 (player has no weapon, fists?)  
**[21] Game:** Enemy HP: 8 → 6, message: "You hit the mirage hound for 2 damage"  
**[22] Game:** Enemy turn - attacks player  
**[23] Game:** Hit succeeds (75%), damage: 2 (1-3 range), player HP: 20 → 18  
**[24] Game:** Message: "The mirage hound hits you for 2 damage"

**✅ POSITIVE:** Combat math is clear and fair  
**❌ FRICTION POINT:** Combat feels slow without meaningful decisions  
**❌ FRICTION POINT:** No tactical options (cover, abilities, positioning)

---

**[25] Player:** Attacks again (3 more exchanges)  
**[26] Player:** Takes 6 more damage (HP: 18 → 12), deals 6 damage  
**[27] Game:** Enemy dies, message: "The mirage hound dies", XP: +15  
**[28] Player:** No loot drop, no item, just XP

**❌ FRICTION POINT:** Combat reward feels weak (just numbers)  
**❌ FRICTION POINT:** No visual or emotional payoff

---

### Phase 2: Exploration & Resource Management (5-15 minutes)

**[29] Player:** Continues exploring, low on HP (12/20)  
**[30] Player:** Opens inventory (i key), sees brine_vial  
**[31] Player:** Reads description: "Salty healing draught"  
**[32] Player:** Uses item (which key? 'u'?)  
**[33] Game:** Heals 5 HP, HP: 12 → 17, message: "You drink the brine vial. +5 HP"

**✅ POSITIVE:** Item use is straightforward  
**❌ FRICTION POINT:** Only had one healing item, now what?

---

**[34] Player:** Encounters another enemy - "glass_beetle" (b), HP: 12  
**[35] Player:** Fights (HP: 17 vs enemy 12)  
**[36] Game:** 4 combat exchanges, player takes 8 damage (HP: 17 → 9)  
**[37] Game:** Enemy dies, XP: +20 (total: 35/100)  
**[38] Player:** Still no loot drops

**❌ FRICTION POINT:** Running low on HP with no healing  
**❌ FRICTION POINT:** No risk/reward decision - just attrition

---

**[39] Player:** Explores more carefully, looking for items  
**[40] Game:** Finds "scripture_shard" (?), "salt_crystal" (,)  
**[41] Player:** Picks up both, inventory: scripture_shard, salt_crystal  
**[42] Player:** No idea what these do (scripture_shard not usable?)

**❌ FRICTION POINT:** Items with no apparent use feel pointless  
**❌ FRICTION POINT:** No quest or NPC to trade with

---

**[43] Player:** Encounters NPC - "hermit_guide" (@), neutral demeanor  
**[44] Player:** Approaches and talks (which key? 't'?)  
**[45] Game:** Dialogue opens: "The hermit watches you silently."  
**[46] Player:** Sees dialogue options:  
 - "Ask about the area"  
 - "Ask about items"  
 - "Leave"  
**[47] Player:** Selects "Ask about items"  
**[48] NPC:** "Storm Glass is valuable. The storms drop it, but they also... change you."

**✅ POSITIVE:** NPC provides lore and hints  
**❌ FRICTION POINT:** Dialogue doesn't offer quest or immediate goal  
**❌ FRICTION POINT:** "Storms" mentioned but player hasn't seen one

---

**[49] Player:** Finds quest board or quest giver? (not clear)  
**[50] Game:** Quest offered: "Pest Control - Kill 3 salt beetles"  
**[51] Player:** Accepts quest (how? automatic?)  
**[52] Game:** Quest added to log, message: "New quest: Pest Control"

**✅ POSITIVE:** Quest gives clear goal  
**❌ FRICTION POINT:** No indication where salt beetles are

---

**[53] Player:** Explores tile, encounters "salt_beetle" (b)  
**[54] Player:** Fights and kills (HP: 9 → 5), Quest: 1/3 beetles killed  
**[55] Player:** Finds another salt_beetle, fights (HP: 5 → 2)  
**[56] Player:** Critically low HP, no healing items

**❌ CRITICAL ISSUE:** Player about to die with no way to recover  
**❌ FRICTION POINT:** No rest mechanic, no health regeneration

---

**[57] Player:** Retreats to safe corner of map  
**[58] Player:** Looks for rest/camp option (none exists?)  
**[59] Player:** Tries to leave tile and return to world map (how?)

**❌ CRITICAL ISSUE:** No escape from bad situations  
**❌ FRICTION POINT:** No world map navigation implemented yet

---

**[60] Player:** Continues carefully, finds "brine_vial" (lucky spawn)  
**[61] Player:** Uses it immediately, HP: 2 → 7  
**[62] Player:** Finds and kills third salt_beetle (HP: 7 → 4)  
**[63] Game:** Quest complete! Reward: 50 XP, "health_potion"  
**[64] Player:** Total XP: 85/100, inventory: health_potion, scripture_shard, salt_crystal

**✅ POSITIVE:** Quest completion feels rewarding  
**❌ FRICTION POINT:** Almost died, felt more stressful than fun

---

### Phase 3: Level Up & Progression (15-20 minutes)

**[65] Player:** Kills one more enemy (mirage_hound)  
**[66] Game:** XP: 85 + 15 = 100/100, LEVEL UP! Level 2  
**[67] Game:** Message: "You gained a level! 3 stat points available"  
**[68] Player:** Opens character screen (which key?)  
**[69] Game:** Shows stats: max_hp, max_ap, reflex, can allocate 3 points  
**[70] Player:** Allocates 2 to max_hp, 1 to reflex  
**[71] Game:** Max HP: 20 → 22, Reflex: 0 → 1, full heal on level up

**✅ POSITIVE:** Level up feels powerful, player now at 22 HP  
**✅ POSITIVE:** Stat choices give player agency  
**❌ FRICTION POINT:** Only 3 stats to choose from, limited build diversity

---

**[72] Player:** Continues exploring, combat feels slightly easier (reflex helps)  
**[73] Player:** Finds "rusty_blade" weapon  
**[74] Player:** Equips weapon (how? auto? 'e' key?)  
**[75] Game:** Weapon equipped, damage increases (now 3-5 instead of 1-3)

**✅ POSITIVE:** Equipment makes noticeable difference

---

**[76] Player:** Clears rest of tile (kills 4 more enemies)  
**[77] Player:** Levels up to Level 3, allocates stats  
**[78] Player:** Tile fully explored, all enemies dead

**❌ FRICTION POINT:** No reason to stay on tile, no new content  
**❌ FRICTION POINT:** Can't progress to world map (feature not implemented)

---

### Phase 4: Missing Content Becomes Obvious (20-30 minutes)

**[79] Player:** Wants to explore new areas but can't access world map  
**[80] Player:** Inventory has random items (scripture_shard, salt_crystal) with no use  
**[81] Player:** No crafting system accessible  
**[82] Player:** No new quests appearing  
**[83] Player:** No NPCs offering deeper interactions

**❌ CRITICAL ISSUE:** Game loop ends prematurely - no progression path  
**❌ CRITICAL ISSUE:** No storms have occurred - THE core mechanic missing  
**❌ CRITICAL ISSUE:** No adaptations - identity mechanic missing  
**❌ CRITICAL ISSUE:** No faction system - social layer missing

---

**[84] Player:** Tries to find more content, re-explores tile  
**[85] Player:** Realizes all items collected, all enemies dead  
**[86] Player:** Opens quest log - one completed quest, no new quests  
**[87] Player:** Game feels "done" but only played 25 minutes

**❌ CRITICAL ISSUE:** No endgame or long-term goals  
**❌ CRITICAL ISSUE:** No replayability hooks

---

## Fun Factor Analysis

### Current State Score: 4/10

**What's Working (The Good):**

1. ✅ **Core combat math is solid** - Hit chances feel fair, damage is readable
2. ✅ **Exploration reveals things** - FOV system creates discovery moments
3. ✅ **Level up feels good** - Stat allocation gives player agency
4. ✅ **Equipment matters** - Finding weapons makes noticeable impact
5. ✅ **Quest completion is satisfying** - Clear goal + reward loop

**What's Missing (The Critical):**

1. ❌ **No core identity** - Game feels generic without storms/adaptations
2. ❌ **No tactical depth** - Combat is just "attack until one dies"
3. ❌ **No long-term goals** - 30 minutes and you're done
4. ❌ **No risk/reward decisions** - Optimal play is always obvious
5. ❌ **No emergent stories** - Nothing memorable happens

**What's Frustrating (The Bad):**

1. ❌ **No tutorial or onboarding** - Player is lost immediately
2. ❌ **Unclear controls** - Many actions not explained
3. ❌ **Resource scarcity too punishing** - Almost died from lack of healing
4. ❌ **Dead-end content** - Items you can't use, quests that don't lead anywhere
5. ❌ **No escape routes** - Stuck in bad situations with no recovery

---

## The "Missing Magic" - Why Storms/Adaptations/Factions Are Critical

### WITHOUT These Systems:

**Current Experience:** "Walk around, kill things, level up, run out of content in 30 minutes."

This is a **competent but forgettable roguelike**. It has solid fundamentals but no soul.

### WITH These Systems (Projected Experience):

**[Storm Event at Turn 50]**  
**[88] Game:** "The light shifts. Glass dust fills the air. A storm approaches!"  
**[89] Game:** Storm timer: 10 turns visible in HUD  
**[90] Player:** Sees walls start to shimmer, realizes map will change  
**[91] Player:** DECISION POINT: Rush to exit, seek shelter, or stay for Storm Glass?

**✅ CREATES TENSION:** Time pressure forces meaningful choice  
**✅ CREATES SPECTACLE:** Map physically changes, walls turn to glass  
**✅ CREATES REWARD:** Storm Glass drops = valuable currency

---

**[92] Player:** Decides to weather storm in corner (risky)  
**[93] Game:** Storm hits - 3 walls near player → glass, corridor opens  
**[94] Game:** Storm Glass drops at 2 locations  
**[95] Game:** Player takes 5 refraction exposure damage  
**[96] Game:** Refraction: 0 → 5, message: "Your skin tingles with strange energy"  
**[97] Player:** Collects 2 Storm Glass (valuable!), map now different

**✅ CREATES CONSEQUENCE:** Refraction gained (permanent change)  
**✅ CREATES REWARD:** Storm Glass = progression currency  
**✅ CREATES VARIETY:** Map layout changed, new paths opened

---

**[98] Player:** Continues, refraction slowly rising (10 → 15 → 20)  
**[99] Game:** At refraction 20: "Your eyes crystallize. You can see through glass walls now."  
**[100] Game:** ADAPTATION UNLOCKED: Crystalline Vision  
**[101] Player:** Suddenly sees through glass walls (tactical advantage!)  
**[102] Player:** Realizes adaptations change gameplay significantly

**✅ CREATES IDENTITY:** Player build emerges organically  
**✅ CREATES POWER FANTASY:** New abilities feel earned  
**✅ CREATES BUILD VARIANCE:** Different runs feel different

---

**[103] Player:** Returns to NPC settlement with high refraction (35)  
**[104] Game:** Monk NPC: "You bear the shimmer... the storms have chosen you."  
**[105] Game:** Monk offers special quest (requires refraction 30+)  
**[106] Game:** Engineer NPC: "Stay back, glassborn. We don't trust your kind."  
**[107] Game:** Engineer refuses to trade (faction reputation -10)

**✅ CREATES SOCIAL CONSEQUENCE:** Choices affect NPC reactions  
**✅ CREATES DILEMMAS:** Can't please everyone  
**✅ CREATES REPLAYABILITY:** Different faction paths each run

---

**[108] Player:** Chooses to align with Monks (accepts adaptation path)  
**[109] Game:** Monk faction reputation +20, Engineer faction -10  
**[110] Game:** Monk offers "Storm Walk" quest - survive storm in chapel  
**[111] Player:** New goal unlocked, new area accessible

**✅ CREATES BRANCHING PATHS:** Faction choice opens unique content  
**✅ CREATES LONG-TERM GOALS:** 2+ hours of content now available

---

## What Needs to Be Added (Beyond Storms/Adaptations/Factions)

### 1. Tutorial & Onboarding (CRITICAL)

**Add:**

- Opening message sequence explaining controls
- First quest that teaches movement, combat, item use
- "Hermit Guide" NPC that offers tutorial dialogue
- Control hints in HUD (e.g., "Press 'i' for inventory")

**Implementation:**

```json
// data/tutorial_sequence.json
{
  "steps": [
    {
      "trigger": "game_start",
      "message": "Welcome to the Saltglass Steppe. Use arrow keys to move.",
      "highlight_controls": ["movement"]
    },
    {
      "trigger": "first_enemy_visible",
      "message": "Press 'a' to attack adjacent enemies.",
      "highlight_controls": ["attack"]
    }
  ]
}
```

---

### 2. Tactical Combat Depth (HIGH PRIORITY)

**Current Problem:** Combat is just "attack until one dies" - no decisions

**Add:**

- **Cover System:** Hide behind objects for +20% dodge chance
- **Positioning Matters:** Flanking gives +10% hit chance
- **Special Attacks:** Spend 2 AP for power attack (1.5x damage)
- **Tactical Retreat:** Disengage without attack of opportunity (costs 2 AP)
- **Environmental Hazards:** Push enemies into glass shards for extra damage

**Data-Driven:**

```json
// data/combat_actions.json
{
  "actions": [
    {
      "id": "power_attack",
      "name": "Power Attack",
      "ap_cost": 2,
      "damage_multiplier": 1.5,
      "hit_chance_penalty": -10
    },
    {
      "id": "aimed_shot",
      "name": "Aimed Shot",
      "ap_cost": 2,
      "hit_chance_bonus": 20,
      "damage_multiplier": 1.0
    }
  ]
}
```

---

### 3. Rest & Resource Management (CRITICAL)

**Current Problem:** No way to recover HP except finding items

**Add:**

- **Camp/Rest Action:** Spend 10 turns to heal 50% HP (can't use in combat)
- **Rest Sites:** Certain tiles have "safe zones" for full recovery
- **Food/Water System (Optional):** Survival layer adds depth
- **Meditation:** Spend AP to recover PSY resource

**Implementation:**

```rust
// In GameState
pub fn rest(&mut self) -> bool {
    if self.enemies.iter().any(|e| e.is_alive()) {
        self.log("You can't rest with enemies nearby.");
        return false;
    }

    self.log("You rest for a moment...");
    let heal = self.player_max_hp / 2;
    self.player_hp = (self.player_hp + heal).min(self.player_max_hp);
    self.advance_turns(10); // Time passes
    true
}
```

---

### 4. Loot & Economy (HIGH PRIORITY)

**Current Problem:** Killing enemies gives XP only, feels unrewarding

**Add:**

- **Enemy Loot Tables:** 30% chance to drop items on death
- **Currency System:** Salt scrip ($) for trading
- **Shopkeeper NPCs:** Trade items for currency
- **Crafting Materials:** Enemies drop crafting components

**Data-Driven:**

```json
// In data/enemies.json - add loot_table
{
  "id": "glass_beetle",
  "loot_table": [
    { "item": "glass_shard", "chance": 0.4 },
    { "item": "beetle_carapace", "chance": 0.2 },
    {
      "item": "salt_scrip",
      "quantity_min": 5,
      "quantity_max": 15,
      "chance": 0.3
    }
  ]
}
```

---

### 5. Environmental Storytelling & Atmosphere (MEDIUM)

**Current Problem:** World feels sterile, no sense of place

**Add:**

- **Environmental Messages:** "You see skeletal remains fused into the glass wall"
- **Ambient Events:** Distant storm flashes, glass chimes, wind sounds (text)
- **Landmark Descriptions:** Rich text for significant locations
- **Journal Entries:** Findable lore that explains world history

**Example:**

```
You enter a vitrified chapel. Pews of fused glass face an altar
where a figure kneels, crystallized mid-prayer. The light through
the walls casts cyan shadows that seem to breathe.

[Press 'x' to examine the crystallized pilgrim]
```

---

### 6. Short-Term Goals & Quest Chain (HIGH PRIORITY)

**Current Problem:** One quest, then nothing

**Add:**

- **Quest Chains:** Completing one quest unlocks the next
- **Multiple Quest Givers:** 3-5 NPCs offering different questlines
- **Faction Quests:** Monks, Engineers, Glassborn each have 3-5 quests
- **Main Questline:** 10-15 quest spine leading to endgame

**Structure:**

```
Tutorial Quest → Choose Faction → Faction Intro Quest →
→ Unlock Storm Compass → First Storm Survival →
→ Adaptation Choice Point → Faction Conflict Quest →
→ Endgame: Vector Choice
```

---

### 7. World Map & Exploration Loop (CRITICAL - Already Planned)

**What This Adds:**

- Exploration goal: "Find the Archive" (POI on world map)
- Biome variance: Each tile feels different
- Discovery: Landmarks, shrines, dungeons to find
- Scale: Game extends from 30 minutes to 2+ hours

---

### 8. Risk/Reward Decisions (CRITICAL)

**Current Problem:** Optimal play is always obvious

**Add Decision Points:**

**Decision 1: Storm Risk**

- Stay and fight for Storm Glass (risk refraction)
- Flee to safety (miss valuable loot)

**Decision 2: Adaptation Path**

- Embrace refraction for power (social consequences)
- Use Saint's Tear to reduce refraction (remain "pure")

**Decision 3: Faction Allegiance**

- Support Monks (mysticism, adaptation focus)
- Support Engineers (tech, stability focus)
- Remain neutral (fewer benefits, no enemies)

**Decision 4: Combat Approach**

- Aggressive (high damage, high risk)
- Defensive (cover, retreat, outlast)
- Stealth (avoid combat, conserve resources)

**Decision 5: Resource Allocation**

- Use healing now (safe) vs save for later (risky)
- Spend Storm Glass on crafting vs save for trading

---

## Improved Gameplay Loop (With All Systems)

### Core Loop (Repeats Every 10-15 Minutes):

1. **Explore new tile** from world map
2. **Encounter enemies** + **Find items**
3. **Storm warning** → **Make decision** (stay/flee/shelter)
4. **Storm hits** → **Map changes** + **Collect Storm Glass** + **Gain refraction**
5. **Adaptation unlocks** (if threshold reached)
6. **Return to settlement** → **Trade** + **Get quests** + **NPC reactions**
7. **Choose faction path** or **Continue neutral**
8. **Upgrade gear** via crafting or shops
9. **Next objective** → Explore new tile (repeat)

### Long-Term Progression (2-3 Hours):

- Level 1-10 character progression
- 0-100 refraction and 5-8 adaptation unlocks
- 3 faction reputations (conflicting)
- 10-15 quest completions
- 5-10 biome tiles explored
- 3-5 landmark discoveries
- Endgame: Vector choice (4 endings)

---

## Final Recommendations

### MUST HAVE (Before Vertical Slice):

1. ✅ **Storm System** - THE defining mechanic
2. ✅ **Adaptation System** - Identity and build variance
3. ✅ **Faction System** - Social consequences and replayability
4. ⚠️ **Tutorial/Onboarding** - Players need guidance
5. ⚠️ **Rest Mechanic** - Recovery without items
6. ⚠️ **Loot Tables** - Enemies drop rewards
7. ⚠️ **Quest Chains** - More than one quest

### SHOULD HAVE (For Polish):

8. 🔹 **Tactical Combat** - Cover, positioning, special attacks
9. 🔹 **Economy System** - Currency and shops
10. 🔹 **Environmental Storytelling** - Atmosphere and lore

### NICE TO HAVE (Post-Minimal):

11. 💡 **Food/Water Survival** - Additional resource layer
12. 💡 **Stealth System** - Alternative to combat
13. 💡 **Companion System** - NPC allies

---

## Estimated Fun Factor (With All Critical Systems)

### Projected Score: 8.5/10

**Why It Will Be Fun:**

- ✅ **Unique Identity:** Storms + adaptations = no other game does this
- ✅ **Meaningful Choices:** Faction, adaptation, risk/reward decisions matter
- ✅ **Emergent Stories:** "I got trapped in a storm and became glassborn"
- ✅ **Tactical Depth:** Combat has options beyond "attack"
- ✅ **Long-Term Goals:** 2-3 hour campaign with multiple endings
- ✅ **Replayability:** Different factions/adaptations/builds each run
- ✅ **Atmospheric:** Lore, visuals, environmental storytelling
- ✅ **Fair Challenge:** Can recover from mistakes, not punishing RNG

**Remaining Concerns:**

- ⚠️ Tutorial must be excellent or players bounce
- ⚠️ Balance is crucial - storms can't be too punishing or too easy
- ⚠️ Faction system needs 3-5 quests per faction minimum
- ⚠️ Need 10+ hours of playtesting and iteration

---

## Conclusion

**Current State:** The game has excellent technical foundations but lacks soul. It's playable but forgettable.

**With Critical Systems:** The game becomes a unique, memorable experience with strong identity, meaningful choices, and emergent storytelling.

**The Missing 20%:** Tutorial, rest mechanics, loot tables, and tactical combat depth are essential quality-of-life features that prevent frustration.

**Bottom Line:** Implement storms/adaptations/factions AS DESIGNED in IMPLEMENTATION_TASKS.md, then add the 7 "MUST HAVE" items listed above. This will create a vertical slice worth showcasing.
