# Skill Implementation Analysis

## Evaluation Criteria
- ✅ KEEP: Clear mechanics using existing game systems
- ⏸️ SHELF: Requires new systems not yet implemented

## Combat Skills

### ✅ Blade Mastery (melee_combat)
**Current System**: Combat system with melee damage/accuracy
**Implementation**: Passive bonuses to melee damage and accuracy
**Status**: KEEP - Already implemented

### ✅ Shard Casting (ranged_combat)
**Current System**: Combat system with ranged damage/accuracy
**Implementation**: Passive bonuses to ranged damage and accuracy
**Status**: KEEP - Already implemented

### ✅ Salt Ward (defense)
**Current System**: Combat system with armor values
**Implementation**: Passive armor bonus
**Status**: KEEP - Already implemented

### ⏸️ Light Weaving
**Current System**: None - light combat mechanics not implemented
**Would Need**: Light beam targeting, reflection mechanics, glare effects
**Status**: SHELF - Requires light combat system (future feature)

## Survival Skills

### ✅ Desert Conditioning (survival)
**Current System**: Resource management, environmental effects
**Implementation**: Reduce resource consumption, resist environmental damage
**Status**: KEEP - Already implemented

### ✅ Mirage Walking (stealth)
**Current System**: Enemy detection, FOV system
**Implementation**: Reduce enemy detection range
**Status**: KEEP - Detection system exists

### ✅ Wayfaring
**Current System**: Encounter system, worldmap travel
**Implementation**: Reduce encounter rate, increase flee chance
**Status**: KEEP - Already implemented

### ✅ Steppe Lore (navigation)
**Current System**: Worldmap, quest system
**Implementation**: Reveal POIs on worldmap, quest hints
**Status**: KEEP - Can show hidden map info

### ✅ Flesh Mending (medicine)
**Current System**: Healing items, HP system
**Implementation**: Increase healing effectiveness
**Status**: KEEP - Already implemented

## Social Skills

### ⏸️ Silver Tongue (persuasion)
**Current System**: Basic dialogue system
**Would Need**: Dialogue skill checks, branching outcomes
**Status**: SHELF - Dialogue system not robust enough

### ⏸️ Dread Presence (intimidation)
**Current System**: Basic dialogue system
**Would Need**: Dialogue skill checks, enemy morale system
**Status**: SHELF - Dialogue system not robust enough

### ⏸️ Pilgrim Wisdom (lore)
**Current System**: Quest system, dialogue
**Would Need**: Lore checks, hidden dialogue options
**Status**: SHELF - Unclear gameplay impact

### ✅ Salt Trading (bartering)
**Current System**: Trading system with shop prices
**Implementation**: Reduce buy prices, increase sell prices
**Status**: KEEP - Trading system exists

## Psychic Skills

### ⏸️ Refraction Sight
**Current System**: FOV system
**Would Need**: Enhanced perception mechanics, hidden object detection
**Status**: SHELF - Unclear what this reveals beyond normal FOV

### ⏸️ Glass Communion
**Current System**: Crystal resonance system (exists but minimal)
**Would Need**: Crystal interaction mechanics, crystal powers
**Status**: SHELF - Crystal system not fleshed out

### ⏸️ Storm Calling
**Current System**: Storm system (exists but environmental)
**Would Need**: Player-controlled storm manipulation
**Status**: SHELF - Major feature, needs design

### ⏸️ Mind Fortress
**Current System**: Sanity system
**Would Need**: Mental attack mechanics, sanity defense
**Status**: SHELF - No mental attacks in game yet

## Crafting Skills

### ✅ Glass Shaping (crafting)
**Current System**: Crafting system with recipes
**Implementation**: Increase craft success rate, improve quality
**Status**: KEEP - Already implemented

### ⏸️ Salt Alchemy
**Current System**: Item system
**Would Need**: Potion brewing system, consumable effects
**Status**: SHELF - No alchemy system

### ⏸️ Relic Engineering
**Current System**: Item system
**Would Need**: Device crafting, special item effects
**Status**: SHELF - Unclear what "devices" are

### ⏸️ Scripture Binding
**Current System**: Item system
**Would Need**: Book crafting system, book effects
**Status**: SHELF - No book crafting mechanics

## Summary

### ✅ KEEP (10 skills with clear implementation)
1. **melee_combat** (Blade Mastery) - Damage/accuracy bonuses
2. **ranged_combat** (Shard Casting) - Damage/accuracy bonuses
3. **defense** (Salt Ward) - Armor bonuses
4. **survival** (Desert Conditioning) - Resource efficiency
5. **stealth** (Mirage Walking) - Detection reduction
6. **wayfaring** - Encounter rate/flee chance
7. **navigation** (Steppe Lore) - Reveal map info
8. **medicine** (Flesh Mending) - Healing bonuses
9. **bartering** (Salt Trading) - Shop price modifiers
10. **crafting** (Glass Shaping) - Craft quality/success

### ⏸️ SHELF (12 skills requiring new systems)
1. **light_combat** (Light Weaving) - Needs light beam mechanics
2. **persuasion** (Silver Tongue) - Needs dialogue skill checks
3. **intimidation** (Dread Presence) - Needs dialogue skill checks
4. **lore** (Pilgrim Wisdom) - Unclear gameplay impact
5. **psychic_sight** (Refraction Sight) - Unclear mechanics
6. **crystal_communion** (Glass Communion) - Needs crystal system
7. **storm_calling** (Storm Calling) - Major feature
8. **mental_defense** (Mind Fortress) - No mental attacks yet
9. **alchemy** (Salt Alchemy) - Needs alchemy system
10. **engineering** (Relic Engineering) - Unclear mechanics
11. **bookbinding** (Scripture Binding) - No book crafting
12. **acrobatics** - Unclear mechanics (dodge? mobility?)

## Recommendation

Implement the **10 KEEP skills** now with clear, minimal mechanics:
- All use existing game systems
- Clear passive bonus implementations
- No new systems required
- Immediate gameplay impact

Shelf the 12 others for future expansion when their underlying systems are implemented.
