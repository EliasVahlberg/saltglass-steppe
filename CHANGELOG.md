# Changelog

## [Unreleased]

### Added

#### Action Points (AP) System
- Player AP/max AP fields with configurable defaults
- Action costs defined in `data/actions.json` (move=1, attack=2, ranged=3, etc.)
- Auto-end turn when AP depletes
- `end_turn()` resets AP and triggers enemy turns

#### Combat Math
- `WeaponDef` struct with damage, accuracy, range, AP cost, ammo type
- Hit chance: `accuracy - target_reflex - cover_bonus` (clamped 5-95%)
- Damage: `base_damage - armor` (minimum 1)
- Critical hits: 5% chance for 2x damage
- Player combat stats: reflex, armor, equipped weapon
- Enemy combat stats: reflex, armor, accuracy
- Weapons: fists, salt_knife, glass_shard, pilgrim_staff, brine_pistol, storm_bow

#### Ranged Attacks
- Range and line-of-sight checks
- Ammo consumption from inventory
- Ammo items: brine_shot, glass_arrow

#### Status Effects
- Types: Poison, Burn, Stun, Bleed, Slow
- Duration and potency system
- DoT effects tick on turn end
- Auto-expire when duration reaches 0

#### DES (Debug Execution System) Enhancements
- Actions: `EndTurn`, `RangedAttack`, `ApplyStatus`
- Assertions: `PlayerAp`, `HasStatusEffect`, `StatusEffectCount`
- Player setup: `ap`, `max_ap`, `equipped_weapon`
- 8 new test scenarios for mechanics validation

### Changed
- Melee attacks now use combat math (hit/miss/crit)
- Turn advancement moved to `end_turn()` only (fixed double-tick bug)

### Fixed
- `tick_turn()` was called in both movement and `end_turn()`, causing double turn advancement
