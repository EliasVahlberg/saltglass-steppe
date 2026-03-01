# Unified Structure System - Implementation Plan

## Status: ✅ COMPLETE (2026-03-01)

All 4 phases delivered. See `docs/archive/phase_reports/` for phase summaries.

### What Was Built
- `src/game/generation/structure_library.rs` — unified `StructureLibrary` with `by_usage`, `by_faction`, `by_tag`
- `data/structures/structures.json` — 50 structures (35 connectable, 15 standalone)
- `data/structures/patterns/core/` — 35 building pattern files
- `data/structures/patterns/ruins/` — 15 POI pattern files
- `schemas/structures_unified_v1.json` — unified schema
- `src/game/generation/settlement/buildings.rs` — `place_buildings` wired to `StructureLibrary`

### What Was Removed
- `data/prefabs/` (core.json, factions.json)
- `data/structure_templates.json`
- `src/game/generation/settlement/prefab.rs`
- `src/game/structure_templates.rs`
- `schemas/prefabs_v1.json`, `schemas/structure_templates_v1.json`

---



## Motivation

### Current Problems
- **Two separate systems**: `structure_templates.json` and `prefabs/*.json` with different schemas
- **Code duplication**: Two loaders, two data structures, two APIs
- **No pattern reuse**: Same layout must be duplicated for different themes
- **Poor editing experience**: ASCII art embedded in JSON arrays
- **Confusion**: Unclear which system to use for new content

### Solution
- **Single schema** with usage types (standalone vs connectable)
- **Hybrid loading** supporting both external pattern files and inline patterns
- **Pattern reuse** enabling multiple themed variants from one layout
- **Better tooling** with ASCII art in separate .txt files

## Design Goals

1. **Unify structure and prefab systems** into single `Structure` type
2. **Support pattern reuse** - same layout, different themes/materials
3. **Hybrid loading** - external .txt files or inline JSON arrays
4. **Backward compatible** - gradual migration path
5. **Simple implementation** - avoid OOP complexity, use tagged unions

## Architecture

### Core Data Structure

```rust
/// Single structure type for all placeable structures
pub struct Structure {
    pub id: String,
    pub usage: StructureUsage,
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<Vec<char>>,
    pub legend: HashMap<char, LegendEntry>,
    pub metadata: StructureMetadata,
}

/// Usage determines how structure is placed in game
pub enum StructureUsage {
    Standalone,    // Complete POI (ruins, shrines, landmarks)
    Connectable,   // Settlement building (can attach roads/paths)
}

/// Legend entry types for pattern characters
pub enum LegendEntry {
    Wall { id: String },
    Floor { id: String },
    Door,
    Interactable { id: String },
    Npc { id: String, name: Option<String> },
}

/// Structure metadata
pub struct StructureMetadata {
    pub description: Option<String>,
    pub faction: Option<String>,
    pub npc_count: usize,
    pub npc_types: Vec<String>,
    pub tags: Vec<String>,
    pub weight: f32,
    pub clear_area: Option<ClearArea>,
    // Additional fields as needed
}
```

### File Organization

```
data/structures/
├── patterns/
│   ├── core/                    # Reusable building patterns
│   │   ├── house_small.txt
│   │   ├── house_medium.txt
│   │   ├── house_large.txt
│   │   ├── temple_8x8.txt
│   │   ├── inn_12x10.txt
│   │   └── town_hall_10x8.txt
│   ├── ruins/                   # Standalone POI patterns
│   │   ├── mesa_village.txt
│   │   ├── ancient_archive.txt
│   │   └── refraction_cathedral.txt
│   └── special/                 # Unique story locations
│       └── aria_chamber.txt
└── structures.json              # All metadata + pattern references
```

## Examples

### Example 1: Pattern Reuse with Different Themes

**Pattern file** (`data/structures/patterns/core/temple_8x8.txt`):
```
########
#......#
#..AA..#
#......#
#......#
#..AA..#
#......#
########
```

**Multiple themed variants** in `structures.json`:
```json
{
  "structures": [
    {
      "id": "mirror_monks_light_temple",
      "usage": "connectable",
      "pattern_file": "patterns/core/temple_8x8.txt",
      "legend": {
        "#": {"type": "wall", "id": "glass_wall"},
        ".": {"type": "floor", "id": "tile_floor"},
        "A": {"type": "interactable", "id": "altar"}
      },
      "metadata": {
        "description": "A temple of crystalline glass, refracting light into sacred patterns",
        "faction": "mirror_monks",
        "npc_count": 2,
        "npc_types": ["priest", "acolyte"],
        "tags": ["religious", "faction_building"],
        "weight": 1.0
      }
    },
    {
      "id": "storm_cults_storm_shrine",
      "usage": "connectable",
      "pattern_file": "patterns/core/temple_8x8.txt",
      "legend": {
        "#": {"type": "wall", "id": "stone_wall"},
        ".": {"type": "floor", "id": "stone_floor"},
        "A": {"type": "interactable", "id": "altar"}
      },
      "metadata": {
        "description": "A shrine to the storm, walls scarred by wind and glass",
        "faction": "storm_cults",
        "npc_count": 1,
        "npc_types": ["cultist"],
        "tags": ["religious", "faction_building"],
        "weight": 1.0
      }
    }
  ]
}
```

**Result**: One pattern → multiple themed buildings with different materials, NPCs, and descriptions.

### Example 2: Standalone Structure (Ruins)

**Pattern file** (`data/structures/patterns/ruins/mesa_village.txt`):
```
::::::::::::::::::::::::
:::▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓::::
::▓#.......M.......#▓:::
:▓#.................#▓::
:▓#.................#▓::
:▓#.......G.........#▓::
::▓#...............#▓:::
:::▓▓▓▓▓▓DDD▓▓▓▓▓▓▓::::
::::::::::::::::::::::::
```

**Metadata** in `structures.json`:
```json
{
  "id": "mesa_village",
  "usage": "standalone",
  "pattern_file": "patterns/ruins/mesa_village.txt",
  "legend": {
    "#": {"type": "wall", "id": "old_reinforced_concrete"},
    "▓": {"type": "wall", "id": "shale"},
    ".": {"type": "floor", "id": "ancient_tile"},
    ":": {"type": "floor", "id": "soft_sand"},
    "D": {"type": "door"},
    "M": {"type": "npc", "id": "mesa_merchant", "name": "Keth the Trader"},
    "G": {"type": "npc", "id": "village_guard", "name": "Sentinel Vex"}
  },
  "metadata": {
    "description": "A small settlement carved into the mesa walls",
    "tags": ["ruins", "poi", "village"],
    "clear_area": {
      "shape": "circle",
      "radius": 8,
      "center_x": 12,
      "center_y": 8
    }
  }
}
```

### Example 3: Inline Pattern (Small/Test Structures)

```json
{
  "id": "residential_tiny",
  "usage": "connectable",
  "pattern": [
    "####",
    "#b.#",
    "#..D",
    "####"
  ],
  "legend": {
    "#": {"type": "wall", "id": "wood_wall"},
    ".": {"type": "floor", "id": "wood_floor"},
    "b": {"type": "interactable", "id": "bed"},
    "D": {"type": "door"}
  },
  "metadata": {
    "description": "A tiny one-room dwelling",
    "tags": ["residential", "small"],
    "npc_count": 1,
    "npc_types": ["resident"]
  }
}
```

## Implementation

### Loader Implementation

```rust
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
struct StructureFile {
    structures: Vec<StructureData>,
}

#[derive(Deserialize)]
struct StructureData {
    id: String,
    usage: StructureUsage,
    #[serde(default)]
    pattern_file: Option<String>,
    #[serde(default)]
    pattern: Option<Vec<String>>,
    legend: HashMap<char, LegendEntry>,
    metadata: StructureMetadata,
}

impl Structure {
    /// Load structure from data, supporting hybrid pattern loading
    fn from_data(data: StructureData, base_path: &Path) -> Result<Self, String> {
        // Hybrid loading: try file first, fall back to inline
        let pattern = if let Some(file) = data.pattern_file {
            load_pattern_from_file(&base_path.join(&file))?
        } else if let Some(inline) = data.pattern {
            parse_inline_pattern(inline)?
        } else {
            return Err(format!("Structure '{}' must specify pattern_file or pattern", data.id));
        };
        
        // Validate pattern is rectangular
        let height = pattern.len();
        if height == 0 {
            return Err(format!("Structure '{}' has empty pattern", data.id));
        }
        let width = pattern[0].len();
        for (i, row) in pattern.iter().enumerate() {
            if row.len() != width {
                return Err(format!(
                    "Structure '{}' row {} has width {} but expected {}",
                    data.id, i, row.len(), width
                ));
            }
        }
        
        // Validate all pattern characters are in legend
        for (y, row) in pattern.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                if ch != ' ' && !data.legend.contains_key(&ch) {
                    return Err(format!(
                        "Structure '{}' pattern character '{}' at ({}, {}) not in legend",
                        data.id, ch, x, y
                    ));
                }
            }
        }
        
        Ok(Structure {
            id: data.id,
            usage: data.usage,
            width,
            height,
            pattern,
            legend: data.legend,
            metadata: data.metadata,
        })
    }
}

/// Load pattern from external .txt file
fn load_pattern_from_file(path: &Path) -> Result<Vec<Vec<char>>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read pattern file {:?}: {}", path, e))?;
    
    let pattern: Vec<Vec<char>> = content
        .lines()
        .map(|line| line.chars().collect())
        .collect();
    
    if pattern.is_empty() {
        return Err(format!("Pattern file {:?} is empty", path));
    }
    
    Ok(pattern)
}

/// Parse inline pattern from JSON array
fn parse_inline_pattern(lines: Vec<String>) -> Result<Vec<Vec<char>>, String> {
    if lines.is_empty() {
        return Err("Inline pattern is empty".to_string());
    }
    
    Ok(lines.into_iter().map(|line| line.chars().collect()).collect())
}

/// Structure library - single source for all structures
pub struct StructureLibrary {
    structures: HashMap<String, Structure>,
}

impl StructureLibrary {
    /// Load all structures from structures.json
    pub fn load() -> Result<Self, String> {
        let json = include_str!("../../data/structures/structures.json");
        let data: StructureFile = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse structures.json: {}", e))?;
        
        let mut structures = HashMap::new();
        for s in data.structures {
            let id = s.id.clone();
            let structure = Structure::from_data(s, Path::new("data/structures"))?;
            structures.insert(id, structure);
        }
        
        Ok(StructureLibrary { structures })
    }
    
    /// Get structure by ID
    pub fn get(&self, id: &str) -> Option<&Structure> {
        self.structures.get(id)
    }
    
    /// Get all structures with specific usage
    pub fn by_usage(&self, usage: StructureUsage) -> Vec<&Structure> {
        self.structures.values()
            .filter(|s| s.usage == usage)
            .collect()
    }
    
    /// Get all structures for a faction
    pub fn by_faction(&self, faction: &str) -> Vec<&Structure> {
        self.structures.values()
            .filter(|s| s.metadata.faction.as_deref() == Some(faction))
            .collect()
    }
    
    /// Get all structures with a specific tag
    pub fn by_tag(&self, tag: &str) -> Vec<&Structure> {
        self.structures.values()
            .filter(|s| s.metadata.tags.contains(&tag.to_string()))
            .collect()
    }
}
```

### Usage in Game Code

```rust
// Load library once at startup
static STRUCTURE_LIBRARY: Lazy<StructureLibrary> = Lazy::new(|| {
    StructureLibrary::load().expect("Failed to load structure library")
});

// Get standalone structures for POI generation
let ruins = STRUCTURE_LIBRARY.by_usage(StructureUsage::Standalone);
let mesa_village = STRUCTURE_LIBRARY.get("mesa_village").unwrap();

// Get connectable buildings for settlement generation
let buildings = STRUCTURE_LIBRARY.by_usage(StructureUsage::Connectable);
let faction_buildings = STRUCTURE_LIBRARY.by_faction("mirror_monks");

// Get specific building types
let houses = STRUCTURE_LIBRARY.by_tag("residential");
let temples = STRUCTURE_LIBRARY.by_tag("religious");
```

## Migration Strategy

### Phase 1: Create Unified System
**Goal**: Implement new system alongside existing systems

1. Create `src/game/generation/structures.rs` with unified loader
2. Define `Structure`, `StructureUsage`, `LegendEntry` types
3. Implement `StructureLibrary` with hybrid loading
4. Create `data/structures/structures.json` (empty initially)
5. Write unit tests for loader

**Deliverables**:
- Working `StructureLibrary` that loads from new format
- Tests validating pattern loading (file + inline)
- Tests validating legend validation

### Phase 2: Migrate Existing Content
**Goal**: Convert existing structures to new format

1. Extract patterns from `structure_templates.json` → `.txt` files in `patterns/ruins/`
2. Extract patterns from `prefabs/*.json` → `.txt` files in `patterns/core/`
3. Consolidate metadata into `structures.json`
4. Validate all structures load correctly
5. Keep old files for backward compatibility

**Deliverables**:
- All existing structures available in new format
- Pattern files organized by category
- Validation script to compare old vs new

### Phase 3: Update Game Code
**Goal**: Switch game systems to use unified library

1. Update settlement generation to use `StructureLibrary.by_usage(Connectable)`
2. Update POI generation to use `StructureLibrary.by_usage(Standalone)`
3. Replace `PrefabLibrary` calls with `StructureLibrary`
4. Replace `structure_templates` module with unified loader
5. Update tests to use new API

**Deliverables**:
- Settlement generation uses unified system
- POI generation uses unified system
- All tests passing

### Phase 4: Cleanup
**Goal**: Remove old systems and finalize migration

1. Delete `data/structure_templates.json`
2. Delete `data/prefabs/*.json`
3. Remove `src/game/structure_templates.rs`
4. Remove `src/game/generation/settlement/prefab.rs`
5. Update documentation
6. Update `SETTLEMENT_GENERATION_PLAN.md`

**Deliverables**:
- Old systems removed
- Documentation updated
- Clean codebase with single structure system

## Benefits

### Pattern Reuse
- **Before**: 7 factions × 3 buildings = 21 separate patterns for temples
- **After**: 1 temple pattern → 7 themed variants (3× less duplication)

### Better Editing
- **Before**: Edit ASCII in JSON arrays with escaped quotes
- **After**: Edit ASCII in .txt files with any text editor

### Cleaner Version Control
- **Before**: Git diffs show JSON noise, hard to see pattern changes
- **After**: Git diffs show actual ASCII changes in .txt files

### Single API
- **Before**: `PrefabLibrary` for settlements, `structure_templates` for POIs
- **After**: `StructureLibrary` for everything

### Extensibility
- Easy to add new usage types: `Interior`, `Decoration`, `Overlay`
- Easy to add new legend entry types: `Trap`, `Treasure`, `Portal`
- Easy to add new metadata fields without breaking existing structures

## Testing Strategy

### Unit Tests
- Pattern loading from file
- Pattern loading inline
- Legend validation (all chars in legend)
- Rectangular pattern validation
- Metadata parsing

### Integration Tests
- Load all structures from `structures.json`
- Verify pattern reuse (multiple structures share same file)
- Verify usage filtering works
- Verify faction filtering works
- Verify tag filtering works

### Regression Tests
- Generate settlements with new system, compare to old
- Generate POIs with new system, compare to old
- Ensure deterministic generation still works

## Open Questions

1. **Pattern file format**: Plain .txt or add metadata header?
   - **Decision**: Plain .txt for simplicity, metadata in JSON
   
2. **Width/height in JSON**: Explicit or computed from pattern?
   - **Decision**: Computed from pattern, validate if explicit provided
   
3. **Migration timeline**: Big bang or gradual?
   - **Decision**: Gradual - new system alongside old, migrate incrementally
   
4. **Backward compatibility**: Support old format forever?
   - **Decision**: No, deprecate after migration complete

## Future Enhancements

### Pattern Composition
Allow structures to reference other structures:
```json
{
  "id": "house_with_garden",
  "pattern_file": "patterns/house_medium.txt",
  "overlays": [
    {"structure": "garden_small", "offset": [10, 0]}
  ]
}
```

### Procedural Variations
Generate pattern variations at runtime:
```json
{
  "id": "house_random",
  "pattern_file": "patterns/house_base.txt",
  "variations": {
    "furniture": ["minimal", "furnished", "cluttered"],
    "condition": ["pristine", "worn", "ruined"]
  }
}
```

### Visual Editor
Build a TUI tool to edit patterns visually:
```bash
cargo run --bin structure-editor patterns/house_small.txt
```

## References

- Original plan: `docs/development/SETTLEMENT_GENERATION_PLAN.md`
- Prefab system: `docs/development/PREFAB_SYSTEM_DESIGN.md`
- Structure templates: `src/game/structure_templates.rs`
- Prefab library: `src/game/generation/settlement/prefab.rs`
