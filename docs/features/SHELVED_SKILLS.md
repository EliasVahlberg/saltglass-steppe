# Shelved Skills - Future Implementation

These skills were designed but shelved because they require game systems that don't exist yet or have unclear implementation paths.

## Light Combat Category

### Light Weaving
**Requires**: Light beam targeting system, reflection mechanics, glare effects
**Description**: Combat using light beams, reflections, and tactical positioning
**Why Shelved**: The light combat system is a major feature mentioned in design docs but not yet implemented. Would need:
- Light beam targeting (line-of-sight with reflection)
- Reflection mechanics (bouncing off glass surfaces)
- Glare/blindness status effects
- Light intensity calculations

**Future Implementation**: When light combat system is added (Roadmap item)

---

## Social Category

### Silver Tongue (Persuasion)
**Requires**: Dialogue skill check system, branching dialogue outcomes
**Description**: Negotiation and persuasion in dialogue
**Why Shelved**: Current dialogue system is basic text display. Would need:
- Skill check system in dialogue
- Multiple dialogue branches based on skill level
- Success/failure outcomes
- NPC reaction system

**Future Implementation**: When dialogue system is enhanced

### Dread Presence (Intimidation)
**Requires**: Dialogue skill check system, enemy morale system
**Description**: Intimidation in dialogue and combat
**Why Shelved**: Would need:
- Dialogue skill checks
- Enemy morale/fear mechanics
- Intimidation success/failure outcomes
- Combat intimidation effects (enemies flee/surrender)

**Future Implementation**: When dialogue and morale systems are added

### Pilgrim Wisdom (Lore)
**Requires**: Lore check system, hidden dialogue options
**Description**: Knowledge of Steppe history and secrets
**Why Shelved**: Unclear gameplay impact. Would need:
- Lore-based skill checks
- Hidden dialogue options revealed by lore
- Quest hints or shortcuts
- Specific lore-gated content

**Future Implementation**: When lore system is designed and content is created

---

## Psychic Category

### Refraction Sight
**Requires**: Enhanced perception mechanics, hidden object detection
**Description**: Enhanced perception and seeing hidden things
**Why Shelved**: Unclear what this reveals beyond normal FOV. Would need:
- Hidden object system (invisible enemies, secret doors)
- Enhanced FOV mechanics
- Specific content designed for perception checks

**Future Implementation**: When hidden object system is designed

### Glass Communion
**Requires**: Crystal interaction system, crystal powers
**Description**: Interacting with crystals and glass structures
**Why Shelved**: Crystal resonance system exists but is minimal. Would need:
- Crystal interaction mechanics
- Crystal-based powers or effects
- Crystal-specific content and puzzles

**Future Implementation**: When crystal system is expanded

### Storm Calling
**Requires**: Player-controlled storm manipulation
**Description**: Summoning or controlling glass storms
**Why Shelved**: Storm system exists but is environmental/procedural. Player control would be a major feature requiring:
- Storm summoning mechanics
- Storm direction/intensity control
- Balance considerations (very powerful)
- Specific storm-based content

**Future Implementation**: Major feature, needs design doc

### Mind Fortress
**Requires**: Mental attack mechanics, sanity defense
**Description**: Defending against psychic attacks
**Why Shelved**: No mental attacks in game yet. Would need:
- Psychic enemy types with mental attacks
- Mental damage/status effects
- Defense mechanics
- Sanity system integration

**Future Implementation**: When psychic enemies are added

---

## Crafting Category

### Salt Alchemy
**Requires**: Potion brewing system, consumable effects
**Description**: Brewing potions and alchemical items
**Why Shelved**: No alchemy system exists. Would need:
- Alchemy recipes
- Ingredient system
- Potion effects (buffs, healing, etc.)
- Brewing mechanics

**Future Implementation**: When alchemy system is designed

### Relic Engineering
**Requires**: Device crafting system, special item effects
**Description**: Creating technological devices
**Why Shelved**: Unclear what "devices" are in game world. Would need:
- Device definitions (what are they?)
- Device crafting recipes
- Device effects and usage
- Lore integration (tech level of world)

**Future Implementation**: When devices are designed

### Scripture Binding
**Requires**: Book crafting system, book effects
**Description**: Creating books and scripture
**Why Shelved**: No book crafting mechanics. Would need:
- Book crafting recipes
- Book effects (skill books? lore books?)
- Writing/binding mechanics
- Book content system

**Future Implementation**: When book system is designed

---

## Athletics Category

### Acrobatics
**Requires**: Clear mechanics definition
**Description**: Unclear - dodge chance? mobility bonuses? jump/climb?
**Why Shelved**: No clear implementation path. Could be:
- Dodge chance in combat (needs dodge system)
- Movement speed bonus (overlaps with athletics)
- Jump/climb mechanics (not in game)
- Stamina efficiency (overlaps with athletics)

**Future Implementation**: When mechanics are clearly defined

---

## Implementation Priority

### High Priority (Clear Value)
1. **Light Weaving** - Core feature in design docs
2. **Storm Calling** - Signature mechanic
3. **Silver Tongue / Dread Presence** - Enhance dialogue

### Medium Priority (Nice to Have)
4. **Salt Alchemy** - Adds crafting depth
5. **Glass Communion** - Expands crystal system
6. **Refraction Sight** - Adds exploration depth

### Low Priority (Unclear Value)
7. **Pilgrim Wisdom** - Needs content design
8. **Relic Engineering** - Needs world-building
9. **Scripture Binding** - Needs system design
10. **Mind Fortress** - Needs enemy design
11. **Acrobatics** - Needs mechanics definition

---

## Notes for Future Implementation

When implementing shelved skills:
1. Design the underlying system first (don't add skill without mechanics)
2. Ensure clear gameplay impact (avoid "flavor only" skills)
3. Create content that uses the skill (enemies, items, quests)
4. Balance against existing skills
5. Add prerequisite chains if appropriate
6. Test with DES scenarios
