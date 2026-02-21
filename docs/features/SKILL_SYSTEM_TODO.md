# Skill System TODO

## Current State (v1.0 - Basic Implementation)

### ✅ Completed
- Core prerequisite system
- Passive bonus calculation
- Combat integration (melee/ranged damage and accuracy)
- Encounter integration (wayfaring skill)
- UI shows prerequisites and locked skills
- 10 implementable skills with clear mechanics
- Save compatibility maintained

### 🔄 Partially Complete
- Trading passive effects defined but not integrated
- Crafting passive effects defined but not integrated
- Navigation passive effects defined but not integrated
- Medicine passive effects defined but not integrated
- Stealth passive effects defined but not integrated
- Survival passive effects defined but not integrated

**Estimated**: 3-4 hours to integrate remaining passive effects

## Future Enhancements

### 1. Shelved Skills Integration
**Priority**: Medium  
**Blockers**: Requires underlying systems

Review and implement the 12 shelved skills documented in `SHELVED_SKILLS.md`:

**High Priority Shelved Skills:**
- **Light Weaving** - Requires light combat system implementation
- **Storm Calling** - Requires player-controlled storm mechanics
- **Silver Tongue / Dread Presence** - Requires dialogue skill check system

**Medium Priority Shelved Skills:**
- **Salt Alchemy** - Requires alchemy/potion system
- **Glass Communion** - Requires expanded crystal interaction system
- **Refraction Sight** - Requires hidden object/perception system

**Low Priority Shelved Skills:**
- **Pilgrim Wisdom** - Needs clear gameplay mechanics defined
- **Relic Engineering** - Needs device system designed
- **Scripture Binding** - Needs book crafting system
- **Mind Fortress** - Needs psychic enemy types
- **Acrobatics** - Needs mechanics definition (dodge? mobility?)

**Dependencies:**
- Enemy variation (psychic enemies, varied combat encounters)
- Crafting system expansion (alchemy, devices, books)
- Psychic system implementation
- Light combat mechanics
- Dialogue skill check system

### 2. Better Descriptions
**Priority**: Low  
**Effort**: 2-3 hours  
**Blockers**: None

Enhance skill descriptions with:
- Clear explanation of passive effects with numbers
- Examples of gameplay impact
- Synergies with other skills
- Prerequisite reasoning (why does X require Y?)
- Mythic lore integration

**Example:**
```
Current: "Proficiency with close-quarters weapons. Each level increases damage and accuracy with melee attacks."

Enhanced: "The art of blade-work in the Saltglass wastes. Each level grants +5% melee damage and +2% accuracy (max +50%/+20%). Synergizes with Salt Ward for defensive fighters. The Mirror Monks teach that mastery of the blade is mastery of the self."
```

### 3. More Accessible Skills UI
**Priority**: Medium  
**Effort**: 4-6 hours  
**Blockers**: None

Improve skills menu usability:
- Show passive bonus values in skill list (not just details)
- Add skill tree visualization showing prerequisite chains
- Highlight skills that unlock new abilities
- Show total passive bonuses summary panel
- Add skill point projection ("If you upgrade this, you'll have X points left")
- Color-code skills by category
- Add search/filter functionality
- Show skill recommendations based on current build
- Add "Respec" option (with cost/limitation)

**UI Mockup Ideas:**
```
┌─ Skills (Combat) ─────────────────────┐
│ [✓] Blade Mastery    Lv 5/10  [+25%]  │ ← Show current bonus
│ [✓] Shard Casting    Lv 3/10  [+15%]  │
│ [🔒] Light Weaving   Lv 0/10  (Req: Shard Casting 3) │
│                                        │
│ Skill Points: 12                       │
│ Next Upgrade Cost: 8 points            │
└────────────────────────────────────────┘
```

### 4. Build/Class Synergies
**Priority**: Medium  
**Effort**: 6-8 hours  
**Blockers**: Needs more skills (shelved skills)

Design skill combinations that create distinct playstyles:

**Potential Builds:**
- **Glass Warrior** - Blade Mastery + Salt Ward + Desert Conditioning
- **Shard Sniper** - Shard Casting + Mirage Walking + Wayfaring
- **Storm Caller** - Storm Calling + Glass Communion + Refraction Sight (shelved)
- **Desert Trader** - Salt Trading + Wayfaring + Steppe Lore
- **Psychic Adept** - All psychic skills (shelved)
- **Alchemist** - Salt Alchemy + Glass Shaping + Medicine (partially shelved)

**Implementation:**
- Add skill set bonuses (e.g., "Glass Warrior: +10% armor when Blade Mastery and Salt Ward both at 5+")
- Add build templates in character creation
- Show build suggestions in skills UI
- Add achievements for completing builds
- Balance skills to make multiple builds viable

**Requires**: More skills (currently only 10, need ~20+ for build diversity)

### 5. Skill Balance Pass
**Priority**: Medium  
**Effort**: 4-6 hours  
**Blockers**: Needs gameplay data

Balance skills based on actual gameplay:

**Current Concerns:**
- Are combat skills too powerful compared to utility skills?
- Is Wayfaring mandatory (encounter reduction too strong)?
- Are prerequisite chains too restrictive?
- Do passive bonuses scale appropriately?
- Are skill point costs balanced?

**Process:**
1. Gather gameplay data (which skills are used most?)
2. Identify overpowered/underpowered skills
3. Adjust passive effect values
4. Adjust skill point costs
5. Adjust max levels
6. Test with DES scenarios
7. Iterate based on feedback

**Requires**: Playtesting data, enemy variation for combat testing

### 6. Active Abilities System
**Priority**: High  
**Effort**: 8-12 hours  
**Blockers**: None (system partially exists)

Expand the ability system beyond passive bonuses:

**Current State:**
- AbilityDef exists but minimal
- Abilities defined in data/abilities.json
- No ability usage in gameplay

**Enhancements Needed:**
- Ability targeting system (self, single enemy, area, line)
- Ability effects (damage, heal, buff, debuff, utility)
- Ability hotbar in HUD
- Ability cooldowns (already tracked)
- Stamina costs (already tracked)
- Ability unlock conditions (skill level requirements)
- Ability upgrade system (improve with skill level)

**Example Abilities:**
- **Blade Mastery 3**: Unlocks "Whirlwind Strike" (melee AoE attack)
- **Shard Casting 5**: Unlocks "Glass Barrage" (multi-target ranged)
- **Wayfaring 7**: Unlocks "Forced March" (move multiple tiles)
- **Salt Trading 4**: Unlocks "Appraise" (reveal item values)

**Integration Points:**
- Combat system (damage abilities)
- Movement system (mobility abilities)
- Utility system (non-combat abilities)
- UI (hotbar, ability menu)

## Implementation Roadmap

### Phase 1: Complete Current System (3-4 hours)
1. Integrate remaining passive effects (trading, crafting, navigation, medicine, stealth, survival)
2. Test all passive bonuses in gameplay
3. Fix any integration bugs

### Phase 2: Active Abilities (8-12 hours)
1. Design ability system architecture
2. Implement ability targeting
3. Implement ability effects
4. Add ability hotbar to HUD
5. Create 2-3 abilities per skill
6. Test with DES scenarios

### Phase 3: UI Improvements (4-6 hours)
1. Show passive bonus values in skill list
2. Add skill tree visualization
3. Add total bonuses summary panel
4. Add skill recommendations

### Phase 4: Better Descriptions (2-3 hours)
1. Write enhanced descriptions for all skills
2. Add synergy information
3. Add lore integration

### Phase 5: Shelved Skills (Depends on Systems)
1. Implement required systems (light combat, dialogue checks, alchemy, etc.)
2. Add shelved skills incrementally
3. Test and balance

### Phase 6: Build System (6-8 hours)
1. Design skill set bonuses
2. Create build templates
3. Add build suggestions to UI
4. Test build viability

### Phase 7: Balance Pass (4-6 hours)
1. Gather gameplay data
2. Identify balance issues
3. Adjust values and costs
4. Test and iterate

## Estimated Total Time
- **Phase 1**: 3-4 hours (immediate)
- **Phase 2**: 8-12 hours (high priority)
- **Phase 3**: 4-6 hours (medium priority)
- **Phase 4**: 2-3 hours (low priority)
- **Phase 5**: Variable (depends on system implementation)
- **Phase 6**: 6-8 hours (medium priority, needs more skills)
- **Phase 7**: 4-6 hours (medium priority, needs gameplay data)

**Total (excluding Phase 5)**: 27-39 hours

## Dependencies

### System Blockers
- **Enemy Variation**: Needed for combat skill testing and psychic enemies
- **Crafting System Expansion**: Needed for alchemy, devices, books
- **Psychic System**: Needed for psychic skills
- **Light Combat System**: Needed for Light Weaving
- **Dialogue Skill Checks**: Needed for social skills
- **Active Abilities**: Needed for build diversity

### Priority Order
1. Complete Phase 1 (integrate remaining passive effects)
2. Implement active abilities (Phase 2) - unblocks build diversity
3. Improve UI (Phase 3) - improves player experience
4. Implement enemy variation - unblocks combat testing
5. Implement crafting/psychic/light systems - unblocks shelved skills
6. Add shelved skills (Phase 5)
7. Design builds (Phase 6)
8. Balance pass (Phase 7)
9. Polish descriptions (Phase 4)

## Success Metrics

### Phase 1 Complete
- [ ] All 10 skills have integrated passive effects
- [ ] Trading prices affected by Salt Trading skill
- [ ] Crafting success/quality affected by Glass Shaping skill
- [ ] Map reveals POIs based on Steppe Lore skill
- [ ] Healing affected by Flesh Mending skill
- [ ] Detection affected by Mirage Walking skill
- [ ] Resources affected by Desert Conditioning skill

### Phase 2 Complete
- [ ] 20-30 active abilities implemented
- [ ] Ability hotbar in HUD
- [ ] Ability targeting working
- [ ] Ability effects applied correctly
- [ ] Abilities unlock at appropriate skill levels

### Phase 3 Complete
- [ ] Skill tree visualization shows prerequisites
- [ ] Passive bonuses visible in skill list
- [ ] Total bonuses summary panel
- [ ] Skill recommendations based on build

### Full System Complete
- [ ] 20+ skills available (including shelved)
- [ ] 40-50 active abilities
- [ ] 5+ viable builds
- [ ] Balanced skill costs and effects
- [ ] Enhanced descriptions with lore
- [ ] Accessible, intuitive UI

## Notes

- Don't add shelved skills until their systems exist
- Focus on active abilities before adding more skills
- UI improvements can happen in parallel with system work
- Balance pass should wait until more skills are available
- Build system needs 20+ skills to be meaningful
