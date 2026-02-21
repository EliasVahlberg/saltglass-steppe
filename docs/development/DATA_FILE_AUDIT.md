# Data File Audit Results

**Date**: 2026-02-21  
**Total JSON files**: 54  
**Loaded by game**: 44  
**Test-only**: 1  
**Dead/Unused**: 9

---

## ✅ Active Files (Loaded by Game)

### Core Game Systems
| File | Loaded By | Purpose |
|------|-----------|---------|
| `actions.json` | `game/action.rs` | Player action definitions |
| `classes.json` | `game/meta.rs` | Character class definitions |
| `factions.json` | `game/faction.rs` | Faction definitions and colors |
| `keyboard_config.json` | `game/keyboard_config.rs` | Keybinding configuration |
| `progression.json` | `game/progression.rs` | Level-up and XP tables |
| `tutorial.json` | `game/tutorial.rs` | Tutorial message definitions |

### Combat & Enemies
| File | Loaded By | Purpose |
|------|-----------|---------|
| `enemies/common.json` | `game/enemy.rs` | Common tier enemies |
| `enemies/uncommon.json` | `game/enemy.rs` | Uncommon tier enemies |
| `enemies/rare.json` | `game/enemy.rs` | Rare tier enemies |
| `enemies/elite.json` | `game/enemy.rs` | Elite tier enemies |
| `enemies/boss.json` | `game/enemy.rs` | Boss tier enemies |
| `weapons.json` | `game/combat.rs` | Weapon definitions |
| `status_effects.json` | `game/status.rs` | Status effect definitions |
| `effects.json` | `game/effect.rs` | Visual/gameplay effects |

### Items & Inventory
| File | Loaded By | Purpose |
|------|-----------|---------|
| `items.json` | `game/item.rs` | Item definitions |
| `chests.json` | `game/chest.rs` | Chest loot tables |
| `loot_tables.json` | `game/generation/loot.rs` | Loot generation tables |

### Crafting & Trading
| File | Loaded By | Purpose |
|------|-----------|---------|
| `recipes.json` | `game/crafting.rs` | Crafting recipe definitions |
| `traders.json` | `game/trading.rs` | Trader definitions and inventories |

### Skills & Abilities
| File | Loaded By | Purpose |
|------|-----------|---------|
| `skills.json` | `game/skills.rs` | Skill tree definitions |
| `abilities.json` | `game/skills.rs` | Active ability definitions |
| `adaptations.json` | `game/adaptation.rs` | Mutation/adaptation definitions |
| `psychic_abilities.json` | `game/psychic.rs` | Psychic power definitions |

### Quests & Dialogue
| File | Loaded By | Purpose |
|------|-----------|---------|
| `quests.json` | `game/quest.rs` | Side quest definitions |
| `main_questline.json` | `game/quest.rs` | Main story quest chain |
| `dialogues.json` | `game/dialogue.rs` | NPC dialogue trees |
| `aria_dialogues.json` | `game/dialogue.rs` | ARIA interface dialogues |
| `npcs.json` | `game/npc.rs` | NPC definitions |

### World & Generation
| File | Loaded By | Purpose |
|------|-----------|---------|
| `terrain_config.json` | `game/generation/terrain_forge_adapter.rs` | **PRIMARY** terrain generation config |
| `biome_spawn_tables.json` | `game/generation/spawn.rs` | Enemy spawn tables per biome |
| `biome_profiles.json` | `game/generation/biomes.rs` | Biome generation profiles (⚠️ SUSPECT) |
| `map_features.json` | `game/generation/feature_registry.rs` | Map feature definitions |
| `microstructures.json` | `game/generation/microstructures.rs` | Small structure templates |
| `structure_templates.json` | `game/structure_templates.rs` | Large structure templates |
| `constraint_rules.json` | `game/generation/constraints.rs` | Generation constraint rules |
| `narrative_integration.json` | `game/generation/narrative.rs` | Narrative event integration |
| `dynamic_events.json` | `game/generation/events.rs` | Dynamic event definitions |

### Map & Rendering
| File | Loaded By | Purpose |
|------|-----------|---------|
| `walls.json` | `game/map.rs` | Wall tile definitions |
| `floors.json` | `game/map.rs` | Floor tile definitions |
| `lights.json` | `game/light_defs.rs` | Light source definitions |
| `interactables.json` | `game/interactable.rs` | Interactable object definitions |

### Travel & Encounters
| File | Loaded By | Purpose |
|------|-----------|---------|
| `travel_config.json` | `game/travel.rs` | Travel cost and mechanics |
| `encounter_config.json` | `game/encounter.rs` | Encounter generation config |
| `auto_explore_config.json` | `game/auto_explore.rs` | Auto-explore behavior config |
| `storm_config.json` | `game/storm.rs` | Storm system configuration |

---

## 🧪 Test-Only Files

| File | Loaded By | Purpose |
|------|-----------|---------|
| `structure_generation.json` | `bin/tilegen-tool.rs` | **TEST TOOL ONLY** - Structure gen testing |

**Recommendation**: Move to `tests/data/` or document as test-only.

---

## ❌ Dead/Unused Files (Not Loaded)

These files are not referenced by any code:

| File | Status | Recommendation |
|------|--------|----------------|
| `npc_spawn_config.json` | Dead | Remove (biome_spawn_tables.json handles spawning) |
| `quest_constraints.json` | Dead | Remove (constraint_rules.json is used) |
| `structure_spawn_config.json` | Dead | Remove (structure_templates.json is used) |
| `grammars/descriptions.json` | Dead | Remove (no grammar system active) |
| `grammars/names.json` | Dead | Remove (no grammar system active) |
| `templates/content_templates.json` | Dead | Remove (no template system active) |

**Total dead files**: 6 (11% of total)

---

## ⚠️ Suspect Files (Needs Investigation)

### `biome_profiles.json` - UNUSED BUT NOT REDUNDANT
- **Loaded by**: `game/generation/biomes.rs` (BiomeSystem)
- **Status**: BiomeSystem is ONLY used in tests, never in production gameplay
- **Content**: Rich environmental storytelling data (atmospheric elements, hazards, ambient descriptions)
- **Overlap with terrain_config.json**: Minimal - only resource_modifiers overlap
- **Purpose**: Different from terrain_config.json (storytelling vs generation parameters)
- **Recommendation**: Either implement BiomeSystem in gameplay OR remove biome_profiles.json
- **Decision**: Keep for now (potential future feature), but mark as unused

---

## ✅ Files Previously Thought Dead (Actually Used)

These files ARE loaded but not via `include_str!`:

| File | Loaded By | Purpose |
|------|-----------|---------|
| `books.json` | `game/book.rs` | Book content data |
| `effects_config.json` | `renderer/mod.rs` | Effects management config |
| `narrative_templates.json` | `game/generation/narrative_templates.rs` | Narrative generation templates |
| `render_config.json` | `renderer/mod.rs` | Renderer configuration |
| `themes.json` | `renderer/mod.rs` | Visual theme definitions |
| `structure_generation.json` | `bin/tilegen-tool.rs` | Test tool only (not game code) |

**Note**: These use different loading mechanisms (not `include_str!`)

---

## Summary

- **Active files**: 49 (91%)
- **Test-only**: 1 (2%) - structure_generation.json
- **Dead files**: 6 (11%)
- **Unused but valid**: 1 (biome_profiles.json - future feature)

**Cleanup potential**: Remove 6 dead files (~11% reduction)

**Next steps**:
1. ✅ Investigated biome_profiles.json - unused but not redundant (keep for future)
2. ✅ Verified structure_generation.json is test-only
3. Remove 6 confirmed dead files
4. Document remaining active files
