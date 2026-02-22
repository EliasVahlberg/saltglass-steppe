# JSON Schema Creation TODO

**Status**: 1/51 schemas complete (2%)  
**Priority**: Medium (improves data validation and IDE support)

---

## Current State

### ✅ Completed (1 schema)

- [x] **enemies_v1.json** - Enemy definitions
  - Covers: `data/enemies/*.json` (5 files)
  - Documentation: `docs/features/ENEMY_JSON_SCHEMA_V1.md`
  - Schema validation: Implemented in loader
  - Status: Complete with comprehensive validation

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
- `src/game/dialogue.rs` - dialogues.json, aria_dialogues.json
- `src/game/npc.rs` - npcs.json
- `src/game/faction.rs` - factions.json
- `src/game/adaptation.rs` - adaptations.json
- `src/game/skills.rs` - skills.json, abilities.json
- `src/game/combat.rs` - weapons.json
- `src/game/map.rs` - walls.json, floors.json
- `src/game/light_defs.rs` - lights.json
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

## Phase 1: High Priority Schemas (8 schemas)

### 1. items_v1.json
- **Files**: `data/items.json`, `data/weapons.json`
- **Complexity**: High (equipment, consumables, quest items, weapons)
- **Loader**: `src/game/item.rs`, `src/game/combat.rs`
- **Estimated Time**: 3-4 hours
- **Fields**: id, name, glyph, description, value, item_type, usable, equippable, equipment_slot, stats, effects, requirements

### 2. quests_v1.json
- **Files**: `data/quests.json`, `data/main_questline.json`
- **Complexity**: High (objectives, conditions, rewards, branching)
- **Loader**: `src/game/quest.rs`
- **Estimated Time**: 3-4 hours
- **Fields**: id, name, description, category, objectives, prerequisites, rewards, faction_effects, unlock_conditions

### 3. npcs_v1.json
- **Files**: `data/npcs.json`, `data/traders.json`
- **Complexity**: Medium (dialogue refs, trading, factions)
- **Loader**: `src/game/npc.rs`
- **Estimated Time**: 2-3 hours
- **Fields**: id, name, glyph, description, faction, dialogue_id, trader, quest_giver, location

### 4. dialogues_v1.json
- **Files**: `data/dialogues.json`, `data/aria_dialogues.json`
- **Complexity**: Medium (conversation trees, choices, conditions)
- **Loader**: `src/game/dialogue.rs`
- **Estimated Time**: 2-3 hours
- **Fields**: id, speaker, text, choices, conditions, effects, next_dialogue

### 5. factions_v1.json
- **Files**: `data/factions.json`
- **Complexity**: Medium (reputation, relationships, buildings)
- **Loader**: `src/game/faction.rs`
- **Estimated Time**: 2 hours
- **Fields**: id, name, description, color, reputation_tiers, relationships, buildings

### 6. adaptations_v1.json
- **Files**: `data/adaptations.json`
- **Complexity**: Medium (mutation trees, costs, effects)
- **Loader**: `src/game/adaptation.rs`
- **Estimated Time**: 2 hours
- **Fields**: id, name, description, tier, cost, prerequisites, effects, social_penalty

### 7. spawn_tables_v1.json
- **Files**: `data/biome_spawn_tables.json`
- **Complexity**: Medium (weighted spawning, biome-specific)
- **Loader**: `src/game/generation/spawn.rs`
- **Estimated Time**: 2 hours
- **Fields**: biome, enemies, npcs, items, weights, level_ranges

### 8. loot_tables_v1.json
- **Files**: `data/loot_tables.json`
- **Complexity**: Medium (weighted drops, quality tiers)
- **Loader**: `src/game/generation/loot.rs`
- **Estimated Time**: 2 hours
- **Fields**: table_id, items, weights, quality_tiers, guaranteed_drops

**Phase 1 Total**: 18-22 hours

---

## Phase 2: Medium Priority Schemas (8 schemas)

### 9. terrain_config_v1.json
- **Files**: `data/terrain_config.json`
- **Complexity**: High (biome algorithms, POI layouts, materials)
- **Loader**: `src/game/generation/terrain_forge_adapter.rs`
- **Estimated Time**: 3 hours

### 10. biome_profiles_v1.json
- **Files**: `data/biome_profiles.json`
- **Complexity**: Medium (environmental content, features)
- **Loader**: `src/game/generation/biomes.rs`
- **Estimated Time**: 2 hours

### 11. storm_config_v1.json
- **Files**: `data/storm_config.json`
- **Complexity**: Medium (timing, effects, transformations)
- **Loader**: `src/game/storm.rs`
- **Estimated Time**: 2 hours

### 12. encounter_config_v1.json
- **Files**: `data/encounter_config.json`
- **Complexity**: Medium (encounter types, triggers)
- **Loader**: `src/game/encounter.rs`
- **Estimated Time**: 2 hours

### 13. dynamic_events_v1.json
- **Files**: `data/dynamic_events.json`
- **Complexity**: Medium (triggers, consequences)
- **Loader**: `src/game/generation/events.rs`
- **Estimated Time**: 2 hours

### 14. narrative_integration_v1.json
- **Files**: `data/narrative_integration.json`
- **Complexity**: Medium (story fragments, placement rules)
- **Loader**: `src/game/generation/narrative.rs`
- **Estimated Time**: 2 hours

### 15. microstructures_v1.json
- **Files**: `data/microstructures.json`
- **Complexity**: Medium (structure definitions, placement)
- **Loader**: `src/game/generation/microstructures.rs`
- **Estimated Time**: 2 hours

### 16. interactables_v1.json
- **Files**: `data/interactables.json`
- **Complexity**: Medium (interactive objects, quest triggers)
- **Loader**: `src/game/interactable.rs`
- **Estimated Time**: 2 hours

**Phase 2 Total**: 17 hours

---

## Phase 3: Lower Priority Schemas (34 schemas)

### Abilities & Skills (3 schemas)
- [ ] abilities_v1.json - `data/abilities.json`
- [ ] skills_v1.json - `data/skills.json`
- [ ] psychic_abilities_v1.json - `data/psychic_abilities.json`

### Status & Effects (3 schemas)
- [ ] status_effects_v1.json - `data/status_effects.json`
- [ ] effects_v1.json - `data/effects.json`
- [ ] effects_config_v1.json - `data/effects_config.json`

### World Objects (3 schemas)
- [ ] chests_v1.json - `data/chests.json`
- [ ] books_v1.json - `data/books.json`
- [ ] structure_templates_v1.json - `data/structure_templates.json`

### Map Elements (3 schemas)
- [ ] walls_v1.json - `data/walls.json`
- [ ] floors_v1.json - `data/floors.json`
- [ ] lights_v1.json - `data/lights.json`

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

## Unified JSON Loading Module (Future Enhancement)

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

pub struct DataLoader<T> {
    data: HashMap<String, T>,
}

impl<T: DeserializeOwned + Clone> DataLoader<T> {
    pub fn load_single(path: &str, expected_schema: &str) -> Self {
        // Load single JSON file with schema validation
    }
    
    pub fn load_multiple(paths: &[(&str, &str)], expected_schema: &str) -> Self {
        // Load multiple JSON files and merge
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
    DataLoader::load_single("../../data/items.json", "items_v1")
});

static ENEMIES: Lazy<DataLoader<EnemyDef>> = Lazy::new(|| {
    DataLoader::load_multiple(&[
        ("common", "../../data/enemies/common.json"),
        ("uncommon", "../../data/enemies/uncommon.json"),
        // ...
    ], "enemies_v1")
});
```

### Benefits
- **DRY**: Single implementation for all data loading
- **Validation**: Automatic schema version checking
- **Error handling**: Consistent, helpful error messages
- **Type safety**: Generic implementation works with any data type
- **Future-proof**: Easy to add hot-reload, caching, etc.

### Implementation Plan
1. Create `src/game/data_loader.rs` module
2. Implement `DataLoader<T>` with schema validation
3. Migrate one module (e.g., `item.rs`) as proof of concept
4. Gradually migrate other modules
5. Add optional JSON Schema validation (using `jsonschema` crate)

**Estimated Time**: 4-6 hours for initial implementation + migration

---

## Total Effort Estimate

- **Phase 1** (High Priority): 18-22 hours
- **Phase 2** (Medium Priority): 17 hours
- **Phase 3** (Lower Priority): 34 hours
- **Unified Loader**: 4-6 hours

**Total**: 73-79 hours (~2 weeks of focused work)

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
