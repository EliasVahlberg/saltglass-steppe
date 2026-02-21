# Skill Catalog Implementation Plan

## Overview
Transform the current 8-skill system into a comprehensive 22-skill tree with prerequisites and 40-50 active abilities, fully integrated into all game systems.

## Design Summary (by content-writer)

### 22 Skills Across 5 Categories

**Combat (4 skills):**
- Blade Mastery (melee combat expansion)
- Shard Casting (ranged glass attacks)
- Salt Ward (defensive techniques)
- Light Weaving (light-based combat, requires Shard Casting)

**Survival (5 skills):**
- Desert Conditioning (environmental resistance)
- Mirage Walking (stealth in desert)
- **Wayfaring** (travel speed +25%, encounter avoidance 75%, flee chance +50%)
- Steppe Lore (knowledge and navigation)
- Flesh Mending (healing and medicine)

**Social (4 skills):**
- Silver Tongue (negotiation and persuasion)
- Dread Presence (intimidation)
- Pilgrim Wisdom (lore and insight)
- Salt Trading (commerce and appraisal)

**Psychic (4 skills):**
- Refraction Sight (enhanced perception)
- Glass Communion (crystal interaction)
- Storm Calling (weather manipulation)
- Mind Fortress (mental defense)

**Crafting (5 skills):**
- Glass Shaping (glass item crafting)
- Salt Alchemy (potion brewing)
- Relic Engineering (device creation)
- Scripture Binding (book crafting)

### Key Features
- Prerequisite chains (e.g., Light Weaving requires Shard Casting level 3)
- 2-3 active abilities per skill, unlocked at specific levels
- Passive effects that scale with skill level
- Mythic-reverent naming and descriptions

## Implementation Plan (by implementation-planner)

### Phase 1: Core Data Structures (Priority: HIGH)

**Files to modify:**
- `src/game/skills.rs` - Add prerequisites, passive effects, ability unlocks
- `data/skills.json` - Expand from 8 to 22 skills

**New structs needed:**
```rust
pub struct SkillPrerequisite {
    pub skill_id: String,
    pub required_level: u32,
}

pub struct PassiveEffect {
    pub effect_type: String,  // "damage_bonus", "cost_reduction", etc.
    pub target: String,        // "melee", "trading", "crafting", etc.
    pub value_per_level: f32,
    pub max_value: Option<f32>,
}
```

### Phase 2: Ability System Enhancement (Priority: HIGH)

**Files to modify:**
- `src/game/skills.rs` - Enhanced AbilityDef
- `data/abilities.json` - Expand from 14 to 40-50 abilities

**New fields for AbilityDef:**
- `prerequisites: Vec<SkillPrerequisite>` - Multiple skill requirements
- `effect_type: AbilityEffectType` - Combat, Movement, Utility, etc.
- `target_type: TargetType` - Self, Single, Area, Line, etc.
- `effect_data: AbilityEffectData` - Structured effect parameters

### Phase 3: Integration (Priority: MEDIUM)

**Combat Integration** (`src/game/combat.rs`, `src/game/combat_actions.rs`):
- Add skill-based damage bonuses
- Add skill-based accuracy bonuses
- Integrate active abilities into combat actions

**Trading Integration** (`src/game/trading.rs`):
- Add negotiation skill effects on prices
- Add appraisal skill for item value detection

**Crafting Integration** (`src/game/crafting.rs`):
- Replace simple checks with skill-based crafting
- Add skill-based success rates and quality bonuses

**Encounter Integration** (`src/game/encounter.rs`):
- Wire wayfaring skill into flee chance calculation
- Add skill-based encounter rate modifiers

### Phase 4: UI Updates (Priority: LOW)

**Files to modify:**
- `src/ui/skills_menu.rs` - Show prerequisites, locked skills
- `src/ui/hud.rs` - Display active ability hotbar

### Phase 5: Testing (Priority: MEDIUM)

**DES scenarios needed:**
- Skill progression with prerequisites
- Ability usage and cooldowns
- Integration with combat/trading/crafting

## Task Breakdown for Subagents

### Task 1: Data Files (content-writer)
- Create complete `data/skills.json` with all 22 skills
- Create complete `data/abilities.json` with 40-50 abilities
- Ensure JSON is valid and follows existing schema

### Task 2: Core System (systems-engineer)
- Implement prerequisite validation in `upgrade_skill()`
- Implement passive bonus calculation system
- Add ability prerequisite checking
- Wire passive bonuses into SkillsState

### Task 3: Combat Integration (combat-engineer)
- Add skill bonuses to damage calculations
- Add skill bonuses to accuracy calculations
- Integrate active combat abilities
- Balance ability power levels

### Task 4: Encounter Integration (systems-engineer)
- Wire wayfaring skill into encounter flee chance
- Add skill-based encounter rate modifiers
- Test integration with existing encounter system

### Task 5: UI Updates (ui-developer)
- Update skills menu to show prerequisites
- Gray out locked skills
- Add visual indicators for unlockable skills
- Display passive bonuses in skill descriptions

### Task 6: Testing (qa-tester)
- Create DES scenarios for skill progression
- Test prerequisite chains
- Test ability usage
- Validate integration with all systems

## Estimated Timeline
- **Phase 1**: 4-6 hours (data structures + data files)
- **Phase 2**: 3-4 hours (ability system)
- **Phase 3**: 4-6 hours (integration)
- **Phase 4**: 2-3 hours (UI)
- **Phase 5**: 2-3 hours (testing)

**Total**: 15-22 hours (2-3 days)

## Dependencies
- ✅ No blockers - standalone system
- ✅ Integrates with completed encounter system
- ✅ Enhances existing combat/trading/crafting systems

## Success Criteria
- [ ] 22 skills defined in data/skills.json
- [ ] 40-50 abilities defined in data/abilities.json
- [ ] Prerequisites system working
- [ ] Passive bonuses calculated and applied
- [ ] Wayfaring skill affects encounter system
- [ ] Combat skills affect damage/accuracy
- [ ] Social skills affect trading prices
- [ ] Crafting skills affect recipe success
- [ ] UI shows prerequisites and locked skills
- [ ] DES scenarios pass

## Next Steps
1. Review and approve skill/ability designs
2. Assign tasks to subagents
3. Coordinate implementation
4. Review and integrate changes
5. Test and validate
