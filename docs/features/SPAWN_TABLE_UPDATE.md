# Spawn Table Update Summary

## Overview
Updated all biome spawn tables in `data/biome_spawn_tables.json` to use the 30 new tiered enemies from the enemies_v1 schema.

## Changes Made

### Enemy Mapping
Replaced old enemy IDs with new ones based on theme and level range:

**Old → New Mappings:**
- `dust_wraith` → `salt_wraith` or `dust_devil`
- `crystal_spider` → `glass_crawler`
- `storm_hawk` → `storm_herald`
- `void_swimmer` → `void_whisper`
- `glass_golem` → `shard_sentinel`
- `storm_archer` → `storm_herald`
- `psychic_wraith` → `refraction_wraith`
- `alpha_mirage_hound` → `mirage_hound` (higher level)
- `storm_marksman` → `storm_caller`
- `adaptation_horror` → `refraction_beast`
- `crystal_devastator` → `prism_tyrant`
- `mind_sovereign` → `void_sovereign`
- `sandstorm_elemental` → `storm_sovereign`
- `void_stalker` → `void_whisper`
- `archive_drone` → `archive_custodian`
- `salt_harvester` → `salt_mummy`
- `brine_symbiont` → `brine_leech`
- `storm_echo` → `storm_herald`
- `refraction_wisp` → `refraction_wraith`
- `glass_weaver` → `glass_crawler`
- `prism_guardian` → `shard_sentinel`
- `crystal_bloom` → `crystal_scarab`

**Removed (no equivalent):**
- `shard_nest` (spawner)
- `glass_bomber`

### Biome-Specific Theming

**saltflat** - Salt-themed enemies:
- Common: salt_mummy, salt_wraith, dust_devil
- Rare: salt_lord
- Boss: salt_emperor

**glass_gardens** - Glass/crystal enemies:
- Common: glass_crawler
- Uncommon: shard_sentinel
- Rare: crystal_scarab, light_eater

**storm_scars** - Storm enemies:
- Uncommon: storm_herald, dust_devil
- Rare: storm_phantom
- Elite: storm_caller
- Boss: storm_sovereign

**refraction_fields** - Void enemies:
- Uncommon: void_whisper, light_eater
- Elite: void_sovereign
- Boss: void_emperor

**shattered_citadel** - Archive enemies:
- Uncommon: archive_custodian
- Elite: archive_sentinel
- Boss: archive_overmind

**ruins** - Pilgrim/wraith enemies:
- Uncommon: refraction_wraith
- Rare: pilgrim_hollow, refraction_beast
- Elite: void_sovereign

**oasis** - Peaceful glass enemies:
- Common: glass_beetle, glass_crawler
- Rare: crystal_scarab

**scrubland** - Spider/swarm enemies:
- Common: shard_spider, brine_leech
- Uncommon: storm_herald
- Elite: crystal_widow, storm_caller

**salt_mines** - Salt/undead enemies:
- Common: salt_mummy, salt_wraith, brine_leech, dust_devil
- Rare: salt_lord

### Level Ranges Updated
Adjusted level ranges to match new enemy tiers:
- Common: 1-6
- Uncommon: 4-12
- Rare: 8-20
- Elite: 12-30
- Boss: 25-45

## Integration Points

### Encounter System
The encounter system (`src/game/state.rs::spawn_encounter_entities`) uses `weighted_pick_by_level_and_tier` which pulls from these spawn tables. No code changes needed - encounters will automatically use new enemies.

### Map Generation
The map generation system (`src/game/generation/spawn.rs`) loads these tables via `SPAWN_TABLES` static. No code changes needed.

## Testing Needed

1. **Spawn verification**: Verify all 30 enemies can spawn in appropriate biomes
2. **Level scaling**: Check that enemy difficulty matches dungeon level
3. **Biome theming**: Confirm thematic consistency (salt enemies in salt biomes, etc.)
4. **Encounter balance**: Test that threat point budgets spawn appropriate enemy counts
5. **Boss spawns**: Verify boss enemies appear in high-level dungeons

## Commits
- `7dc53c3` - feat: update spawn tables to use new tiered enemies

## Next Steps
1. Test enemy spawning in-game
2. Verify loot drops work correctly
3. Check enemy behaviors trigger properly
4. Balance spawn weights if needed
