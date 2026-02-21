# Faction System Implementation

## Overview

The faction system provides reputation mechanics, territorial control, and social consequences for player actions. Players build or lose reputation with seven factions through quests, dialogue choices, and combat.

## Features Implemented

### 1. Reputation System
- **Scale**: -100 (Hated) to +100 (Exalted)
- **Thresholds**:
  - Hostile: < -50
  - Unfriendly: -49 to -25
  - Neutral: -24 to +24
  - Friendly: +25 to +49
  - Honored: +50 to +99
  - Exalted: +100

### 2. Starting Reputation by Class
- **Pilgrim**: +10 Mirror Monks, +5 Archive Drones
- **Scavenger**: +10 Salt Traders, +5 Sand Engineers
- **Outcast**: +15 Refraction Outcasts, -10 all others
- **Cultist**: +20 Storm Cults, -15 Mirror Monks

### 3. Faction Territories
- World map divided into 7 faction territories using Voronoi diagram
- Center 8-tile radius is neutral zone
- Deterministic generation based on world seed
- Press **F** on world map to toggle faction overlay

### 4. Reputation Changes
- **Quest Completion**: Defined in quest data (`reputation_rewards` field)
- **Dialogue Choices**: Can grant/lose reputation (existing system)
- **Trading**: Prices affected by reputation (existing system)
- **Crafting**: Recipes gated by faction standing (existing system)

### 5. UI Integration
- **Faction Menu (F key)**: Shows exact reputation numbers and color-coded standings
- **World Map Overlay (F key on map)**: Shows faction territorial control
- **Game Log**: Reports reputation changes with faction name

## Technical Implementation

### Core Module: `src/game/faction.rs`
```rust
// Load factions from data/factions.json
pub fn all_faction_ids() -> Vec<String>
pub fn get_faction(id: &str) -> Option<&'static Faction>

// Reputation helpers
pub fn get_standing(rep: i32) -> &'static str
pub fn get_standing_color(rep: i32) -> ratatui::style::Color
pub fn get_starting_reputation(class_id: &str) -> HashMap<String, i32>
```

### World Map Integration: `src/game/world_map.rs`
```rust
pub struct WorldMap {
    pub faction_territories: Vec<Option<String>>, // Faction ID per tile
    // ... other fields
}

pub fn get_faction_territory(&self, x: usize, y: usize) -> Option<&str>
pub fn generate_faction_territories(seed: u64) -> Vec<Option<String>>
```

### Quest System: `src/game/quest.rs`
```rust
pub struct QuestReward {
    pub xp: u32,
    pub items: Vec<String>,
    pub salt_scrip: u32,
    pub reputation_rewards: HashMap<String, i32>, // NEW
    // ...
}
```

### Save System: `src/game/save.rs`
- **SAVE_VERSION**: Bumped to 2
- **Migration**: v1 saves regenerate faction territories from world seed
- **Backward Compatible**: Old saves load with empty reputation (neutral)

## Data Format

### Factions: `data/factions.json`
```json
{
  "factions": [
    {
      "id": "MirrorMonks",
      "name": "Mirror Monks",
      "description": "Ascetic scholars who study light refraction...",
      "color": "Cyan"
    }
  ]
}
```

### Quest Rewards: `data/quests.json`
```json
{
  "reward": {
    "xp": 40,
    "items": ["glasswright_token"],
    "salt_scrip": 75,
    "reputation_rewards": {
      "Glassborn": 15,
      "SandEngineers": 5
    }
  }
}
```

## Usage Examples

### Check Reputation in Code
```rust
let rep = state.get_reputation("MirrorMonks");
if rep < -50 {
    // Player is hostile to Mirror Monks
}
```

### Modify Reputation
```rust
state.modify_reputation("SaltTraders", 10); // Gain 10 rep
state.modify_reputation("StormCults", -15); // Lose 15 rep
```

### Quest with Reputation Reward
```json
{
  "id": "help_monks",
  "name": "Aid the Mirror Monks",
  "reward": {
    "xp": 50,
    "reputation_rewards": {
      "MirrorMonks": 20,
      "StormCults": -10
    }
  }
}
```

## Testing

### DES Scenario: `tests/scenarios/faction_system_test.des`
```
new_game 12345 pilgrim
assert player.faction_reputation["MirrorMonks"] == 10
assert player.faction_reputation["ArchiveDrones"] == 5
```

### Manual Testing
1. Start new game with different classes
2. Open faction menu (F) - verify starting reputation
3. Open world map (M), press F - verify faction overlay
4. Complete quests with reputation rewards
5. Check faction menu - verify reputation changed
6. Load old save - verify migration works

## Future Enhancements (Deferred)

### Enemy Faction Tags
- Add `faction` field to enemy definitions
- Killing enemies affects reputation based on tier:
  - Common: -5
  - Uncommon: -10
  - Rare: -15
  - Elite: -20
  - Boss: -30

### Faction-Specific Content
- Faction-aligned enemies spawn in territories
- Faction quests unlock at certain reputation levels
- Faction vendors with unique items
- Faction abilities/perks at high reputation

### Dynamic Territories
- Faction influence can shift based on player actions
- Territory wars and contested zones
- Faction outposts and strongholds

## Integration Points

### Existing Systems Already Wired
1. **Trading** (`src/game/trading.rs`): Price multipliers based on reputation
2. **Crafting** (`src/game/crafting.rs`): Recipe requirements check faction standing
3. **Dialogue** (`src/game/dialogue.rs`): Dialogue options gated by reputation
4. **Quests** (`src/game/quest.rs`): Quest availability checks faction requirements

### New Integration
- **Character Creation**: Starting reputation initialized in `GameState::new_with_class()`
- **Quest Completion**: Reputation rewards applied in `GameState::complete_quest()`
- **World Map**: Faction overlay rendering in `src/ui/world_map.rs`
- **Faction Menu**: Updated to show exact numbers in `src/ui/faction_menu.rs`

## Performance Notes

- Faction data loaded once at startup (static `Lazy<HashMap>`)
- Territory lookup is O(1) array access
- No performance impact on gameplay loop
- Save/load adds ~1KB per save file

## Known Limitations

1. Faction alignment in `QuestLog` is one-time choice (legacy system)
   - New reputation system allows shifting alignment
   - Consider removing `faction_alignment` field in future
2. No visual indicator of faction territory on tile map (only world map)
3. Reputation changes not animated in UI (instant update)

## Commit History

- **Part 1** (b00cdec): Core faction module, territories, starting reputation
- **Part 2** (339ff0f): Quest rewards, UI updates, save migration
- **Part 3** (pending): Documentation and testing

## References

- Faction data: `data/factions.json`
- Quest examples: `data/quests.json`
- DES test: `tests/scenarios/faction_system_test.des`
- Roadmap: Task 4 "Proper Faction System" (completed)
