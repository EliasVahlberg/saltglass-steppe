# JSON Schema Directory

This directory contains JSON Schema definitions for validating game data files.

## Standard

**JSON Schema Draft 7** (`http://json-schema.org/draft-07/schema#`)

## Existing Schemas

- `enemies_v1.json` - Enemy definitions (covers `data/enemies/*.json`)
- `items_v1.json` - Item definitions (covers `data/items.json`)
- `weapons_v1.json` - Weapon definitions (covers `data/weapons.json`)
- `quests_v1.json` - Quest definitions (covers `data/quests.json`, `data/main_questline.json`)
- `npcs_v1.json` - NPC and trader definitions (covers `data/npcs.json`, `data/traders.json`)
- Draft schema: `map_elements_v1.json` (planned unified tiles + lights schema; not yet used by data files)
  - Legacy schemas `walls_v1.json`, `floors_v1.json`, `lights_v1.json` are deprecated in favor of `map_elements_v1.json` once migration completes.

## Creating a New Schema

### 1. Schema File Structure

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://saltglass-steppe.game/schemas/your_schema_v1.json",
  "title": "Your Data Type Schema",
  "description": "Schema for [data type] in Saltglass Steppe (v1)",
  "type": "object",
  "required": ["schema", "your_data_key"],
  "properties": {
    "schema": {
      "type": "string",
      "const": "your_schema_v1",
      "description": "Schema version identifier"
    },
    "your_data_key": {
      "type": "array",
      "description": "Array of [data type] definitions",
      "items": {
        "$ref": "#/$defs/your_item"
      }
    }
  },
  "$defs": {
    "your_item": {
      "type": "object",
      "required": ["id", "name"],
      "properties": {
        "id": {
          "type": "string",
          "pattern": "^[a-z][a-z0-9_]*$",
          "description": "Unique identifier (snake_case)"
        },
        "name": {
          "type": "string",
          "minLength": 1,
          "description": "Display name"
        }
      }
    }
  }
}
```

### 2. Naming Conventions

**Schema Files**: `{data_type}_v{version}.json`
- Examples: `enemies_v1.json`, `items_v1.json`, `quests_v1.json`
- Use snake_case
- Include version number (start with v1)

**Schema Identifiers**: `{data_type}_v{version}`
- Must match the `"schema"` field in data files
- Examples: `"enemies_v1"`, `"items_v1"`

**Schema URIs**: `https://saltglass-steppe.game/schemas/{filename}`
- Used in `$id` field
- Not actual URLs, just identifiers

### 3. Required Top-Level Fields

Every schema must include:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://saltglass-steppe.game/schemas/your_schema_v1.json",
  "title": "Human-Readable Title",
  "description": "Brief description of what this schema validates"
}
```

### 4. Data File Format

All data files using schemas must include a `schema` field:

```json
{
  "schema": "your_schema_v1",
  "your_data": [
    { "id": "example", "name": "Example" }
  ]
}
```

This allows:
- Multiple files with the same schema to be merged
- Schema version detection and migration
- Validation tooling to identify the correct schema

### 5. Common Patterns

**ID Fields** (unique identifiers):
```json
"id": {
  "type": "string",
  "pattern": "^[a-z][a-z0-9_]*$",
  "description": "Unique identifier (snake_case)"
}
```

**Enums** (fixed set of values):
```json
"rarity": {
  "type": "string",
  "enum": ["common", "uncommon", "rare", "epic", "legendary"],
  "description": "Item rarity tier"
}
```

**Numeric Ranges**:
```json
"level": {
  "type": "integer",
  "minimum": 1,
  "maximum": 50,
  "description": "Level requirement"
}
```

**Optional Fields with Defaults**:
```json
"weight": {
  "type": "number",
  "minimum": 0,
  "default": 1.0,
  "description": "Selection weight (higher = more common)"
}
```

**Arrays with Item Schemas**:
```json
"tags": {
  "type": "array",
  "items": {
    "type": "string",
    "pattern": "^[a-z_]+$"
  },
  "uniqueItems": true,
  "description": "Categorization tags"
}
```

**Nested Objects**:
```json
"stats": {
  "type": "object",
  "properties": {
    "hp": { "type": "integer", "minimum": 1 },
    "damage": { "type": "integer", "minimum": 0 }
  },
  "required": ["hp"]
}
```

**Conditional Requirements** (oneOf, anyOf, allOf):
```json
"oneOf": [
  {
    "properties": {
      "type": { "const": "weapon" },
      "damage": { "type": "integer", "minimum": 1 }
    },
    "required": ["damage"]
  },
  {
    "properties": {
      "type": { "const": "armor" },
      "defense": { "type": "integer", "minimum": 1 }
    },
    "required": ["defense"]
  }
]
```

### 6. Validation Best Practices

**Do**:
- ✅ Use descriptive field names and descriptions
- ✅ Set reasonable min/max constraints
- ✅ Use `pattern` for string formats (IDs, tags)
- ✅ Mark truly required fields as `required`
- ✅ Provide `default` values for optional fields
- ✅ Use `enum` for fixed value sets
- ✅ Document field purposes in `description`

**Don't**:
- ❌ Over-constrain (allow flexibility for future content)
- ❌ Make everything required (only require essentials)
- ❌ Use overly complex nested schemas
- ❌ Forget to version your schemas

### 7. Documentation

For each schema, create a corresponding documentation file in `docs/features/`:

**File**: `docs/features/{DATA_TYPE}_JSON_SCHEMA_V{VERSION}.md`

**Contents**:
- Schema reference (link to schema file)
- Human-readable field documentation
- Complete examples
- Common mistakes to avoid
- Tier/category guidelines
- Validation rules

---

## Programmatic Generation (Hybrid)

We support a hybrid approach to reduce manual errors:

### A) Generate from Rust Types (schemars)
Best for data that already has Rust structs.

Example:
```
cargo run --bin schema-gen -- types --target items --output schemas/items_v1.json --schema-id https://saltglass-steppe.game/schemas/items_v1.json --title "Item Definition Schema"
```

Targets supported: `items`, `weapons`, `enemies`, `quests`, `main_questline`, `npcs`, `traders`.

### B) Infer from JSON Data
Good for files without Rust structs (fast baseline).

Example:
```
cargo run --bin schema-gen -- infer --input data/biome_spawn_tables.json --output schemas/spawn_tables_v1.json --schema-id https://saltglass-steppe.game/schemas/spawn_tables_v1.json --title "Spawn Tables Schema"
```

Use `--no-required` to avoid over‑constraining required fields.

See `docs/features/ENEMY_JSON_SCHEMA_V1.md` for a complete example.

### 8. Versioning

When making breaking changes to a schema:

1. Create new schema file: `{data_type}_v2.json`
2. Update data files to use `"schema": "{data_type}_v2"`
3. Keep old schema for backward compatibility
4. Document migration path in schema docs

**Breaking changes**:
- Removing required fields
- Changing field types
- Renaming fields
- Changing validation rules that invalidate existing data

**Non-breaking changes** (can update existing schema):
- Adding optional fields
- Relaxing constraints
- Adding enum values
- Improving descriptions

### 9. Testing Schemas

Validate data files against schemas using standard JSON Schema validators:

**Command-line** (using `ajv-cli`):
```bash
npm install -g ajv-cli
ajv validate -s schemas/enemies_v1.json -d data/enemies/common.json
```

**Rust** (using `jsonschema` crate):
```rust
use jsonschema::JSONSchema;

let schema = serde_json::from_str(include_str!("../schemas/enemies_v1.json"))?;
let instance = serde_json::from_str(include_str!("../data/enemies/common.json"))?;
let compiled = JSONSchema::compile(&schema)?;
let result = compiled.validate(&instance);
```

**VS Code** (automatic validation):
Add to `.vscode/settings.json`:
```json
{
  "json.schemas": [
    {
      "fileMatch": ["data/enemies/*.json"],
      "url": "./schemas/enemies_v1.json"
    }
  ]
}
```

### 10. Priority Order for New Schemas

Based on complexity and frequency of edits:

**High Priority**:
1. `items_v1.json` - Items, weapons, equipment
2. `quests_v1.json` - Quest definitions
3. `npcs_v1.json` - NPC definitions
4. `factions_v1.json` - Faction system

**Medium Priority**:
5. `terrain_config_v1.json` - Terrain generation
6. `biome_profiles_v1.json` - Biome content
7. `spawn_tables_v1.json` - Entity spawning
8. `loot_tables_v1.json` - Loot generation

**Lower Priority**:
- Configuration files (storm, encounter, effects)
- Simple data files (walls, floors, lights)
- UI configs (themes, keyboard, render)

## Resources

- [JSON Schema Documentation](https://json-schema.org/understanding-json-schema/)
- [JSON Schema Draft 7 Specification](https://json-schema.org/draft-07/json-schema-release-notes.html)
- [JSON Schema Validator](https://www.jsonschemavalidator.net/)
- [Example: enemies_v1.json](./enemies_v1.json)
- [Example Documentation: ENEMY_JSON_SCHEMA_V1.md](../docs/features/ENEMY_JSON_SCHEMA_V1.md)
