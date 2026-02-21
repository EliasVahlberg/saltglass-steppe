# Encounter System - Remaining Work

## Status: ✅ Core System Complete, Polish Needed

The overworld travel encounter system is **functionally complete** and ready for gameplay. The following items are polish/expansion tasks for future iterations.

---

## High Priority (Gameplay Impact)

### 1. Neutral/Beneficial Encounter Completion
**Problem:** No clear completion trigger for non-hostile encounters.

**Current Behavior:**
- Hostile: Auto-completes when all enemies dead ✅
- Neutral/Beneficial: Never complete, player stuck until they leave tile ❌

**Solutions:**
- **Option A:** Auto-complete after N turns (e.g., 10 turns)
- **Option B:** Add "Leave Encounter" action (press 'L' or similar)
- **Option C:** Complete when player moves to map edge
- **Recommended:** Option B - explicit player action

**Implementation:**
```rust
// In handle_game_input()
KeyCode::Char('l') if state.world.encounter_state.is_some() => {
    if matches!(encounter_type, Neutral | Beneficial) {
        state.world.encounter_state = None;
        state.log("You leave the encounter.");
    }
}
```

### 2. Animal Herd Implementation
**Problem:** `animal_herd` neutral event only logs a message, doesn't spawn anything.

**Current:**
```rust
"animal_herd" => {
    self.log("A herd of creatures grazes peacefully nearby.");
}
```

**Needed:**
- Add non-hostile animal enemy types to spawn tables
- Flag them as `hostile: false` in enemy definitions
- Spawn 3-5 animals that don't attack player
- Optional: Allow hunting them for resources

**Files to modify:**
- `data/enemies.json` - Add animal types (desert_gazelle, salt_lizard, etc.)
- `src/game/enemy.rs` - Add `hostile` field to Enemy struct
- `src/game/state.rs` - Update spawn logic to use animal types

---

## Medium Priority (Balance & Variety)

### 3. Encounter Rate Tuning
**Current rates** (after recent increase):
- Low danger (1-3): 25-28%
- Medium danger (5-7): 28-33%
- High danger (10+): 33-50%

**Needs playtesting to determine if:**
- Too frequent (annoying)
- Too rare (boring)
- Good balance

**Tuning knobs in `data/encounter_config.json`:**
- `base_encounter_rate` (currently 0.25)
- `danger_scaling` (currently 0.03)
- `cooldown_turns` (currently 50)

### 4. More Neutral Events
**Current:** Only 2 neutral events (trade_caravan, animal_herd)

**Suggested additions:**
```json
{
  "id": "wandering_pilgrim",
  "weight": 30,
  "description": "A lone pilgrim shares stories of distant shrines."
}
{
  "id": "abandoned_camp",
  "weight": 20,
  "description": "You find an abandoned campsite with supplies."
}
{
  "id": "strange_monument",
  "weight": 20,
  "description": "A weathered monument stands here, inscribed with cryptic text."
}
```

**Implementation:** Just add to `encounter_config.json`, no code changes needed.

### 5. More Beneficial Events
**Current:** Generic "you discover something" with random items

**Suggested additions:**
- **Buried cache:** Spawn items in a cluster
- **Oasis spring:** Restore HP/water
- **Ancient shrine:** Grant temporary buff
- **Storm glass deposit:** Spawn valuable crafting materials

**Implementation:** Add event types similar to neutral events, spawn specific items/effects.

---

## Low Priority (Polish & UX)

### 6. Encounter Popup Improvements
**Current:** Simple dialog box with text

**Enhancements:**
- Show encounter type icon (⚔ / 🤝 / ✨)
- Display threat/boon points
- Show biome and danger level
- Add "Press any key to continue" prompt
- Color-code by type (red=hostile, yellow=neutral, green=beneficial)

### 7. Flee Mechanic Visibility
**Current:** Flee command exists but not discoverable

**Improvements:**
- Show flee availability in HUD during encounters
- Display cooldown timer: "Flee available in 23 turns"
- Show distance requirement: "Move 3 more tiles to flee"
- Add tutorial message on first hostile encounter

### 8. Encounter History UI
**Current:** Encounter history tracked but not visible

**Additions:**
- Show encounter cooldown on worldmap tiles
- Highlight tiles with recent encounters (different color)
- Add to tile info: "Last encounter: 23 turns ago"

### 9. Threat/Boon Budget Balancing
**Current:** Rough heuristics for enemy spawning

**Issues:**
- Enemy threat estimation: `level * 2` is very rough
- Item value doesn't match actual usefulness
- High threat encounters might spawn too few enemies
- Low boon encounters might spawn nothing

**Needs:**
- Proper enemy threat ratings in enemy definitions
- Item value rebalancing
- Minimum spawn counts (at least 1 enemy/item)

### 10. Biome-Specific Encounters
**Current:** All biomes use same encounter types

**Enhancements:**
- Desert: Sandstorm events, mirages
- Saltflat: Salt crystal deposits, brine pools
- Ruins: Ancient guardians, trapped vaults
- Oasis: Peaceful traders, water sources
- Scrubland: Bandit camps, hidden caches

**Implementation:** Add biome-specific event lists to `encounter_config.json`.

---

## Technical Debt

### 11. Encounter State Serialization
**Status:** ✅ Already implemented with serde

**Verify:**
- Save/load preserves encounter state
- Encounter history persists
- Spawned enemy indices remain valid

### 12. Encounter Testing
**Current:** Manual testing only

**Needed:**
- DES scenarios for each encounter type
- Test encounter completion conditions
- Test flee mechanic edge cases
- Test cooldown behavior

**Example DES scenario:**
```
# Test hostile encounter completion
spawn player 10 10
trigger_encounter hostile 20
assert encounter_active
spawn enemy 30 30 bandit
wait 1
kill_all_enemies
assert encounter_complete
```

---

## Summary

**Core System:** ✅ Complete and functional
- Encounters trigger during worldmap travel
- Three types spawn correctly
- Hostile encounters grant XP
- Flee mechanic works
- Worldmap integration complete

**Critical Gaps:**
1. Neutral/beneficial completion (high priority)
2. Animal herd spawning (medium priority)

**Polish Needed:**
- More event variety
- Better UX/feedback
- Balance tuning
- Testing

**Recommendation:** Move to next task. Return to encounter polish during content expansion phase.

---

## Files Reference

**Core System:**
- `src/game/encounter.rs` - Encounter logic
- `src/game/state.rs` - Spawning and completion
- `data/encounter_config.json` - All parameters
- `src/main.rs` - Worldmap integration

**Related Systems:**
- `src/game/travel.rs` - Travel costs
- `src/game/world_map.rs` - World generation
- `src/ui/world_map.rs` - Worldmap UI
- `src/game/enemy.rs` - Enemy definitions

**Testing:**
- `tests/encounter_probability_test.rs` - Probability analysis (not integrated)
- Need: DES scenarios for encounter testing
