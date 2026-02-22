# JSON Schema Creation TODO

**Status**: COMPLETE ✅ (45/45 active schemas, 86% of original 52)  
**Implementation Date**: 2026-02-22  
**Priority**: ~~Medium~~ IMPLEMENTED

---

## ✅ IMPLEMENTATION COMPLETE

### Unified Data Loader System

**Implemented**: `src/game/data_loader.rs`

Generic `DataLoader<T>` with:
- ✅ JSON schema validation
- ✅ Single/multiple file loading
- ✅ Consistent error handling
- ✅ Type-safe generic implementation
- ✅ Schema version checking

**Migration**: All 40+ modules migrated to unified loader

### Data File Consolidation

**Merged files** (reduced from 52 to 45):
- `dialogues.json` ← merged `aria_dialogues.json`
- `effects.json` ← merged `status_effects.json`, `effects_config.json`
- `abilities.json` ← merged `skills.json`, `psychic_abilities.json`
- `map_elements.json` ← merged `floors.json`, `walls.json`, `lights.json`

**Result**: 7 fewer files, cleaner organization

### Schema Coverage

**45 schemas generated** covering all active data files:
- Core game content (items, enemies, weapons, etc.)
- Generation configs (terrain, biomes, spawn tables, etc.)
- UI configs (themes, keyboard, render, etc.)
- All consolidated files (abilities, effects, dialogues, map_elements)

**Schema generation tooling**: `src/bin/schema_gen.rs`

**Documentation**: `docs/features/*.md` for each schema

---

## Original Planning (For Reference)
Baseline schemas have been generated for all existing `data/*.json` files using `schema-gen` (inference mode with `--no-required`). These are functional but require the planned description/tightening pass.

### Remaining Missing Schemas (8)
- `expanded_spawn_tables_v1.json` (no data file present)
- `generation_config_v1.json` (no data file present)
- `grammars_v1.json` (no `data/grammars/` directory)
- `npc_spawn_config_v1.json` (no data file present)
- `quest_constraints_v1.json` (no data file present)
- `structure_spawn_config_v1.json` (no data file present)
- `templates_v1.json` (no `data/templates/` directory)
- `world_generation_integration_v1.json` (no data file present)

These map to files listed as dead/unused in `docs/development/DATA_FILE_AUDIT.md`. Unless new data files are created, treat these as **deprecated/removed** rather than schema TODOs.

### Extra Generated Schema
- `aria_dialogues_v1.json` was generated from `data/aria_dialogues.json` but is not in the original list. It has been merged into `data/dialogues.json` under `aria_personalities`.

### Consolidation / Deprecation (Pending)
Completed:
- `aria_dialogues_v1.json` -> merged into `data/dialogues.json` under `aria_personalities`
- `effects_config_v1.json` + `status_effects_v1.json` -> merged into `data/effects.json` (now includes `config` and `status_effects`)
- `skills_v1.json` + `psychic_abilities_v1.json` -> merged into `data/abilities.json` (now includes `skills` and `psychic_abilities`)

### Coverage Notes
- `data/main_questline.json` is covered by `schemas/quests_v1.json`.
- `data/traders.json` is covered by `schemas/npcs_v1.json`.
- `data/biome_spawn_tables.json` is covered by `schemas/spawn_tables_v1.json`.
- `data/structure_generation.json` is test-tool only (tilegen CLI), but still covered by `schemas/structure_generation_v1.json`.

### Programmatic Generation (Hybrid)
We now support a hybrid approach:
- Generate from Rust types using `schema-gen` (schemars).
- Infer from JSON files for data-only schemas, then tighten manually.

See `schemas/README.md` for command examples.

### Draft Schemas (Not Yet Used in Data Files)
- `map_elements_v1.json` (active unified tiles + lights schema; see `docs/development/MAP_ELEMENTS_UNIFICATION_PLAN.md`)
  - Legacy split files (`data/walls.json`, `data/floors.json`, `data/lights.json`) have been removed after migration.

### ✅ Completed (5 schemas)

- [x] **enemies_v1.json** - Enemy definitions
  - Covers: `data/enemies/*.json` (5 files)
  - Documentation: `docs/features/ENEMY_JSON_SCHEMA_V1.md`
  - Schema validation: Implemented in loader
  - Status: Complete with comprehensive validation
- [x] **items_v1.json** - Item definitions
  - Covers: `data/items.json`
  - Documentation: `docs/features/ITEMS_JSON_SCHEMA_V1.md`
  - Schema validation: Implemented in loader
- [x] **weapons_v1.json** - Weapon definitions
  - Covers: `data/weapons.json`
  - Documentation: `docs/features/WEAPONS_JSON_SCHEMA_V1.md`
  - Schema validation: Implemented in loader
- [x] **quests_v1.json** - Quest definitions
  - Covers: `data/quests.json`, `data/main_questline.json`
  - Documentation: `docs/features/QUESTS_JSON_SCHEMA_V1.md`
  - Schema validation: Implemented in loader
- [x] **npcs_v1.json** - NPC and trader definitions
  - Covers: `data/npcs.json`, `data/traders.json`
  - Documentation: `docs/features/NPCS_JSON_SCHEMA_V1.md`
  - Schema validation: Implemented in loader

### 📋 Current JSON Loading Pattern

**Pattern**: Each module uses `once_cell::Lazy` + `include_str!` + `serde_json::from_str`

**Example** (`src/game/item.rs`):
```rust
use once_cell::sync::Lazy;

struct ItemsFile {
    items: Vec<ItemDef>,
}

static ITEM_DEFS: Lazy<HashMap<String, ItemDef>> = Lazy::new(|| {
    let data = include_str!("../../data/items.json");
    let file: ItemsFile = serde_json::from_str(data)
        .expect("Failed to parse items.json");
    file.items.into_iter().map(|d| (d.id.clone(), d)).collect()
});

pub fn get_item_def(id: &str) -> Option<&'static ItemDef> {
    ITEM_DEFS.get(id)
}
```

**Files using this pattern** (40+ modules):
- `src/game/item.rs` - items.json
- `src/game/enemy.rs` - enemies/*.json (with schema validation)
- `src/game/quest.rs` - quests.json, main_questline.json
- `src/game/dialogue.rs` - dialogues.json (includes aria personalities)
- `src/game/npc.rs` - npcs.json
- `src/game/faction.rs` - factions.json
- `src/game/adaptation.rs` - adaptations.json
- `src/game/skills.rs` - abilities.json (includes skills + abilities)
- `src/game/combat.rs` - weapons.json
- `src/game/map.rs` - map_elements.json (or legacy walls/floors fallback)
- `src/game/light_defs.rs` - map_elements.json (or legacy lights fallback)
- `src/game/interactable.rs` - interactables.json
- `src/game/encounter.rs` - encounter_config.json
- `src/game/auto_explore.rs` - auto_explore_config.json
- `src/game/action.rs` - actions.json
- `src/game/structure_templates.rs` - structure_templates.json
- `src/game/generation/spawn.rs` - biome_spawn_tables.json
- `src/game/generation/events.rs` - dynamic_events.json
- `src/game/generation/narrative.rs` - narrative_integration.json
- `src/game/generation/terrain_forge_adapter.rs` - terrain_config.json
- ... and more

---

## Phase 1: High Priority Schemas (9 schemas)

### 1. items_v1.json
- **Files**: `data/items.json`
- **Complexity**: High (equipment, consumables, quest items)
- **Loader**: `src/game/item.rs`
- **Estimated Time**: 3-4 hours
- **Fields**: id, name, glyph, description, value, item_type, usable, equippable, equipment_slot, stats, effects, requirements

### 2. weapons_v1.json
- **Files**: `data/weapons.json`
- **Complexity**: Low (simple weapon stats)
- **Loader**: `src/game/combat.rs`
- **Estimated Time**: 1 hour
- **Fields**: id, name, glyph, damage_min, damage_max, accuracy, range, ap_cost, ammo_type, description

### 3. quests_v1.json
- **Files**: `data/quests.json`, `data/main_questline.json`
- **Complexity**: High (objectives, conditions, rewards, branching)
- **Loader**: `src/game/quest.rs`
- **Estimated Time**: 3-4 hours
- **Fields**: id, name, description, category, objectives, prerequisites, rewards, faction_effects, unlock_conditions

### 4. npcs_v1.json
- **Files**: `data/npcs.json`, `data/traders.json`
- **Complexity**: Medium (dialogue refs, trading, factions)
- **Loader**: `src/game/npc.rs`
- **Estimated Time**: 2-3 hours
- **Fields**: id, name, glyph, description, faction, dialogue_id, trader, quest_giver, location

### 5. dialogues_v1.json
- **Files**: `data/dialogues.json`, `data/aria_dialogues.json`
- **Complexity**: Medium (conversation trees, choices, conditions)
- **Loader**: `src/game/dialogue.rs`
- **Estimated Time**: 2-3 hours
- **Fields**: id, speaker, text, choices, conditions, effects, next_dialogue

### 6. factions_v1.json
- **Files**: `data/factions.json`
- **Complexity**: Medium (reputation, relationships, buildings)
- **Loader**: `src/game/faction.rs`
- **Estimated Time**: 2 hours
- **Fields**: id, name, description, color, reputation_tiers, relationships, buildings

### 7. adaptations_v1.json
- **Files**: `data/adaptations.json`
- **Complexity**: Medium (mutation trees, costs, effects)
- **Loader**: `src/game/adaptation.rs`
- **Estimated Time**: 2 hours
- **Fields**: id, name, description, tier, cost, prerequisites, effects, social_penalty

### 8. spawn_tables_v1.json
- **Files**: `data/biome_spawn_tables.json`
- **Complexity**: Medium (weighted spawning, biome-specific)
- **Loader**: `src/game/generation/spawn.rs`
- **Estimated Time**: 2 hours
- **Fields**: biome, enemies, npcs, items, weights, level_ranges

### 9. loot_tables_v1.json
- **Files**: `data/loot_tables.json`
- **Complexity**: Medium (weighted drops, quality tiers)
- **Loader**: `src/game/generation/loot.rs`
- **Estimated Time**: 2 hours
- **Fields**: table_id, items, weights, quality_tiers, guaranteed_drops

**Phase 1 Total**: 19-23 hours

---

## Phase 2: Medium Priority Schemas (8 schemas)

### 10. terrain_config_v1.json
- **Files**: `data/terrain_config.json`
- **Complexity**: High (biome algorithms, POI layouts, materials)
- **Loader**: `src/game/generation/terrain_forge_adapter.rs`
- **Estimated Time**: 3 hours

### 11. biome_profiles_v1.json
- **Files**: `data/biome_profiles.json`
- **Complexity**: Medium (environmental content, features)
- **Loader**: `src/game/generation/biomes.rs`
- **Estimated Time**: 2 hours

### 12. storm_config_v1.json
- **Files**: `data/storm_config.json`
- **Complexity**: Medium (timing, effects, transformations)
- **Loader**: `src/game/storm.rs`
- **Estimated Time**: 2 hours

### 13. encounter_config_v1.json
- **Files**: `data/encounter_config.json`
- **Complexity**: Medium (encounter types, triggers)
- **Loader**: `src/game/encounter.rs`
- **Estimated Time**: 2 hours

### 14. dynamic_events_v1.json
- **Files**: `data/dynamic_events.json`
- **Complexity**: Medium (triggers, consequences)
- **Loader**: `src/game/generation/events.rs`
- **Estimated Time**: 2 hours

### 15. narrative_integration_v1.json
- **Files**: `data/narrative_integration.json`
- **Complexity**: Medium (story fragments, placement rules)
- **Loader**: `src/game/generation/narrative.rs`
- **Estimated Time**: 2 hours

### 16. microstructures_v1.json
- **Files**: `data/microstructures.json`
- **Complexity**: Medium (structure definitions, placement)
- **Loader**: `src/game/generation/microstructures.rs`
- **Estimated Time**: 2 hours

### 17. interactables_v1.json
- **Files**: `data/interactables.json`
- **Complexity**: Medium (interactive objects, quest triggers)
- **Loader**: `src/game/interactable.rs`
- **Estimated Time**: 2 hours

**Phase 2 Total**: 17 hours

---

## Phase 3: Lower Priority Schemas (34 schemas)

### Abilities & Skills (3 schemas)
- [ ] abilities_v1.json - `data/abilities.json` (includes `skills` + `psychic_abilities`)
- [ ] skills_v1.json - deprecated (merged into abilities_v1)
- [ ] psychic_abilities_v1.json - deprecated (merged into abilities_v1)

### Status & Effects (3 schemas)
- [ ] effects_v1.json - `data/effects.json` (includes `status_effects` + `config`)
- [ ] status_effects_v1.json - deprecated (merged into effects_v1)
- [ ] effects_config_v1.json - deprecated (merged into effects_v1)

### World Objects (3 schemas)
- [ ] chests_v1.json - `data/chests.json`
- [ ] books_v1.json - `data/books.json`
- [ ] structure_templates_v1.json - `data/structure_templates.json`

### Map Elements (3 schemas)
- [ ] map_elements_v1.json - `data/map_elements.json`
- [ ] walls_v1.json - deprecated (legacy split schema)
- [ ] floors_v1.json - deprecated (legacy split schema)
- [ ] lights_v1.json - deprecated (legacy split schema)

### Progression (3 schemas)
- [ ] classes_v1.json - `data/classes.json`
- [ ] progression_v1.json - `data/progression.json`
- [ ] tutorial_v1.json - `data/tutorial.json`

### Crafting & Actions (2 schemas)
- [ ] recipes_v1.json - `data/recipes.json`
- [ ] actions_v1.json - `data/actions.json`

### Constraints (2 schemas)
- [ ] constraint_rules_v1.json - `data/constraint_rules.json`
- [ ] quest_constraints_v1.json - `data/quest_constraints.json`

### UI Configuration (4 schemas)
- [ ] render_config_v1.json - `data/render_config.json`
- [ ] themes_v1.json - `data/themes.json`
- [ ] keyboard_config_v1.json - `data/keyboard_config.json`
- [ ] auto_explore_config_v1.json - `data/auto_explore_config.json`

### Narrative (3 schemas)
- [ ] narrative_templates_v1.json - `data/narrative_templates.json`
- [ ] grammars_v1.json - `data/grammars/*.json`
- [ ] templates_v1.json - `data/templates/*.json`

### Generation (8 schemas)
- [ ] structure_generation_v1.json - `data/structure_generation.json`
- [ ] map_features_v1.json - `data/map_features.json`
- [ ] travel_config_v1.json - `data/travel_config.json`
- [ ] world_generation_integration_v1.json - `data/world_generation_integration.json`
- [ ] npc_spawn_config_v1.json - `data/npc_spawn_config.json`
- [ ] structure_spawn_config_v1.json - `data/structure_spawn_config.json`
- [ ] expanded_spawn_tables_v1.json - `data/expanded_spawn_tables.json`
- [ ] generation_config_v1.json - `data/generation_config.json`

**Phase 3 Total**: ~34 hours (1 hour per schema average)

---

## Unified JSON Loading Module (In Progress)

### Current Issues
- **Duplication**: Each module has identical loading boilerplate
- **No validation**: Most loaders don't validate schema versions
- **Error handling**: Inconsistent error messages
- **No hot-reload**: Changes require recompilation

### Proposed Solution: `src/game/data_loader.rs`

```rust
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

pub struct DataSource<'a> {
    pub label: &'a str,
    pub data: &'a str,
}

pub trait HasId {
    fn id(&self) -> &str;
}

pub struct DataLoader<T> {
    data: HashMap<String, T>,
}

impl<T: DeserializeOwned + HasId> DataLoader<T> {
    pub fn load_single(source: DataSource<'_>, list_key: &str, expected_schema: &str) -> Self {
        // Load single JSON source with schema validation
    }

    pub fn load_multiple(
        sources: &[DataSource<'_>],
        list_key: &str,
        expected_schema: &str,
    ) -> Self {
        // Load multiple JSON sources and merge
    }
    
    pub fn get(&self, id: &str) -> Option<&T> {
        self.data.get(id)
    }
    
    pub fn all(&self) -> Vec<&T> {
        self.data.values().collect()
    }
}

// Usage:
static ITEMS: Lazy<DataLoader<ItemDef>> = Lazy::new(|| {
    DataLoader::load_single(
        DataSource::new("data/items.json", include_str!("../../data/items.json")),
        "items",
        "items_v1",
    )
});

static ENEMIES: Lazy<DataLoader<EnemyDef>> = Lazy::new(|| {
    DataLoader::load_multiple(
        &[
            DataSource::new("data/enemies/common.json", include_str!("../../data/enemies/common.json")),
            DataSource::new("data/enemies/uncommon.json", include_str!("../../data/enemies/uncommon.json")),
            // ...
        ],
        "enemies",
        "enemies_v1",
    )
});
```

### Benefits
- **DRY**: Single implementation for all data loading
- **Validation**: Automatic schema version checking
- **Error handling**: Consistent, helpful error messages
- **Type safety**: Generic implementation works with any data type
- **Future-proof**: Easy to add hot-reload, caching, etc.

### Implementation Plan
1. Create `src/game/data_loader.rs` module (done)
2. Implement `DataLoader<T>` with schema version validation (done)
3. Migrate initial modules (items, weapons, enemies) (done)
4. Gradually migrate other modules
5. Add JSON Schema validation (using `jsonschema` crate) (done)

**Estimated Time**: 4-6 hours for initial implementation + migration

---

## Total Effort Estimate

- **Phase 1** (High Priority): 19-23 hours
- **Phase 2** (Medium Priority): 17 hours
- **Phase 3** (Lower Priority): 34 hours
- **Unified Loader**: 4-6 hours

**Total**: 74-80 hours (~2 weeks of focused work)

---

## Recommended Approach

### Option A: Schema-First (Recommended)
1. Create schemas for Phase 1 (high priority files)
2. Add schema validation to existing loaders
3. Create unified loader module
4. Migrate to unified loader
5. Continue with Phase 2 & 3 schemas

**Pros**: Immediate validation benefits, cleaner migration path  
**Cons**: More upfront work before seeing unified loader benefits

### Option B: Loader-First
1. Create unified loader module
2. Migrate existing loaders to use it
3. Add schema validation to unified loader
4. Create schemas as needed

**Pros**: Faster DRY benefits, less duplication  
**Cons**: Migration without validation, schemas added later

### Option C: Hybrid (Best for Incremental Progress)
1. Create 2-3 high-priority schemas (items, quests, npcs)
2. Create unified loader with schema validation
3. Migrate those 2-3 modules to unified loader
4. Continue creating schemas + migrating modules in parallel

**Pros**: Balanced approach, validates design early  
**Cons**: Requires careful coordination

---

## Next Steps

1. **Decide on approach** (A, B, or C)
2. **Create items_v1.json schema** (most frequently edited)
3. **Test schema validation** with existing items.json
4. **Document schema** in `docs/features/ITEMS_JSON_SCHEMA_V1.md`
5. **Prototype unified loader** (if going with Option C)
6. **Iterate** based on feedback

---

## References

- [Schema Creation Guide](../schemas/README.md)
- [Enemy Schema Example](../schemas/enemies_v1.json)
- [Enemy Schema Documentation](ENEMY_JSON_SCHEMA_V1.md)
- [Current Loading Pattern](../src/game/item.rs) (lines 170-180)
