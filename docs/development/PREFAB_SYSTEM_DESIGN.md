# Prefab Library System Design

**Date**: 2026-02-21  
**Status**: Design Phase  
**Task**: 0 (Critical Path)

---

## Research Summary

### Terrain-Forge Implementation

**Strengths**:
- Clean JSON schema with `pattern`, `legend`, `weight`, `tags`
- Support for rotation and mirroring
- Weighted selection and tag filtering
- Directory loading (`load_from_dir`)
- Legend system maps characters to tiles/markers/masks

**Schema**:
```json
{
  "prefabs": [
    {
      "name": "treasure_room",
      "width": 6,
      "height": 5,
      "pattern": [
        "######",
        "#..M.#",
        "#....#",
        "#..N.#",
        "######"
      ],
      "weight": 1.0,
      "tags": ["room", "legend"],
      "legend": {
        ".": {"tile": "floor"},
        "#": {"tile": "wall"},
        "M": {"tile": "floor", "marker": "loot_slot"},
        "N": {"tile": "floor", "mask": "no_spawn"}
      }
    }
  ]
}
```

**Limitations**:
- Basic tile types only (floor/wall)
- No door/window/furniture support
- No faction theming
- No entrance markers

### Roguelike Community Patterns

**One More Level** (Bozar):
- Text file prefabs with REXPaint
- Classification by size (full-size, jigsaw, island) and purity (pure, mixed)
- Flip/rotate operations for variety
- Coordinate transformation: `new_x = max_y - y - 1; new_y = x` (90° rotation)

**Golden Krone Hotel** (Jeremiah Reid):
- Prefabs as plain text files
- Connection points marked with `*****`
- Special characters: `?` for "don't care", `0-9` for floor types
- Hallway extension algorithm: extend then scale back until prefab fits
- Post-processing: connect adjacent connectors, L-shaped hallways

**Key Insights**:
1. **Connection points are critical** - prefabs need explicit entrance/exit markers
2. **Flexibility matters** - "don't care" tiles allow overlapping
3. **Variety through transformation** - rotation/mirroring multiplies content
4. **Weighted selection** - control rarity of special rooms
5. **Tag filtering** - select prefabs by category (residential, faction, core)

---

## Design Specification

### JSON Schema

```json
{
  "prefabs": [
    {
      "name": "town_hall",
      "width": 10,
      "height": 8,
      "pattern": [
        "##########",
        "#........#",
        "#..TT....#",
        "#..TT....#",
        "#........#",
        "#........#",
        "#...DD...#",
        "##########"
      ],
      "weight": 1.0,
      "tags": ["core", "town_hall", "large"],
      "faction": null,
      "legend": {
        "#": {"tile": "StoneWall"},
        ".": {"tile": "Floor"},
        "D": {"tile": "Door", "marker": "entrance"},
        "T": {"tile": "Table"},
        "?": {"tile": "any"}
      },
      "metadata": {
        "description": "Central town hall with meeting tables",
        "min_tier": "village",
        "npc_spawns": [
          {"type": "mayor", "marker": "M"},
          {"type": "clerk", "marker": "C"},
          {"type": "guard", "marker": "G"}
        ]
      }
    }
  ]
}
```

### Extended Legend System

**Tile Types**:
- Walls: `#` (generic), `W` (wood), `S` (stone), `B` (brick), `G` (glass), `M` (metal)
- Floors: `.` (generic), `0-9` (floor variants)
- Doors: `D` (door), `+` (closed door), `-` (open door)
- Windows: `w` (window)
- Furniture: `T` (table), `C` (chair), `b` (bed), `c` (counter), `s` (shelf), `x` (chest), `k` (workbench), `A` (altar), `H` (throne)
- Decorations: `F` (fountain), `t` (tree), `f` (flower), `L` (lamp), `B` (banner), `I` (sign), `*` (statue), `g` (garden)
- Special: `?` (any/don't care), `@` (player start), `>` (stairs down), `<` (stairs up)

**Markers** (for NPC spawning, quest locations):
- `entrance` - Building entrance
- `exit` - Building exit
- `npc_spawn` - Generic NPC spawn point
- `vendor` - Merchant spawn
- `quest_giver` - Quest NPC spawn
- `loot` - Treasure spawn
- `no_spawn` - Prevent spawning here

**Masks** (for generation constraints):
- `required` - Must be placed
- `optional` - Can be skipped
- `no_overlap` - Cannot overlap with other prefabs
- `no_spawn` - No entity spawning

### Rust Data Structures

```rust
// src/game/generation/settlement/prefab.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::game::map::TileType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabLibraryData {
    pub prefabs: Vec<PrefabData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabData {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<String>,
    pub weight: f32,
    pub tags: Vec<String>,
    pub faction: Option<String>,
    pub legend: HashMap<String, LegendEntry>,
    #[serde(default)]
    pub metadata: PrefabMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrefabMetadata {
    pub description: String,
    pub min_tier: Option<String>, // "village", "town", "city"
    pub npc_spawns: Vec<NpcSpawn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSpawn {
    pub npc_type: String,
    pub marker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendEntry {
    pub tile: String, // TileType name or "any"
    #[serde(default)]
    pub marker: Option<String>,
    #[serde(default)]
    pub mask: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrefabCell {
    pub tile: Option<TileType>,
    pub marker: Option<String>,
    pub mask: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Prefab {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<PrefabCell>,
    pub weight: f32,
    pub tags: Vec<String>,
    pub faction: Option<String>,
    pub metadata: PrefabMetadata,
}

pub struct PrefabLibrary {
    prefabs: Vec<Prefab>,
    by_tag: HashMap<String, Vec<usize>>,
    by_faction: HashMap<String, Vec<usize>>,
}

impl PrefabLibrary {
    pub fn new() -> Self {
        Self {
            prefabs: Vec::new(),
            by_tag: HashMap::new(),
            by_faction: HashMap::new(),
        }
    }

    pub fn load_from_dir<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        // Load all JSON files from directory
        // Parse and add to library
        // Index by tags and faction
    }

    pub fn add_prefab(&mut self, prefab: Prefab) {
        // Add to prefabs vec
        // Index by tags
        // Index by faction
    }

    pub fn get_by_tags(&self, tags: &[String]) -> Vec<&Prefab> {
        // Return prefabs matching ALL tags
    }

    pub fn get_by_faction(&self, faction: &str) -> Vec<&Prefab> {
        // Return prefabs for specific faction
    }

    pub fn select_weighted(&self, prefabs: &[&Prefab], rng: &mut ChaCha8Rng) -> Option<&Prefab> {
        // Weighted random selection
    }
}

impl Prefab {
    pub fn from_data(data: PrefabData) -> Result<Self, String> {
        // Parse pattern using legend
        // Create cells
        // Validate dimensions
    }

    pub fn rotated(&self) -> Self {
        // 90° clockwise rotation
    }

    pub fn mirrored_horizontal(&self) -> Self {
        // Horizontal flip
    }

    pub fn mirrored_vertical(&self) -> Self {
        // Vertical flip
    }

    pub fn get_entrances(&self) -> Vec<(usize, usize)> {
        // Return all cells with "entrance" marker
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
}
```

---

## Implementation Plan

### Phase 1: Core Prefab System (2-3 hours)

**Step 1: Define data structures** (30 min)
- Create `src/game/generation/settlement/prefab.rs`
- Define `PrefabData`, `Prefab`, `PrefabLibrary` structs
- Define `LegendEntry`, `PrefabCell`, `PrefabMetadata`

**Step 2: Implement JSON loading** (1 hour)
- `PrefabLibrary::load_from_dir()` - load all JSON files
- `Prefab::from_data()` - parse pattern using legend
- Character-to-TileType mapping
- Validate prefab dimensions and legend

**Step 3: Implement transformations** (30 min)
- `Prefab::rotated()` - 90° clockwise rotation
- `Prefab::mirrored_horizontal()` - horizontal flip
- `Prefab::mirrored_vertical()` - vertical flip

**Step 4: Implement selection** (30 min)
- `PrefabLibrary::get_by_tags()` - filter by tags
- `PrefabLibrary::get_by_faction()` - filter by faction
- `PrefabLibrary::select_weighted()` - weighted random selection

**Step 5: Write tests** (30 min)
- Unit test: Load prefab from JSON
- Unit test: Rotation/mirroring
- Unit test: Tag filtering
- Unit test: Weighted selection

### Phase 2: Example Prefabs (1 hour)

**Create directory structure**:
```
data/prefabs/
├── core/
│   ├── town_hall.json
│   ├── general_store.json
│   ├── inn.json
│   ├── temple.json
│   └── residential.json
├── mirror_monks/
│   ├── light_temple.json
│   ├── meditation_chamber.json
│   └── scripture_archive.json
└── [other factions]/
```

**Create 3 example prefabs**:
1. `town_hall.json` - 10×8 building with tables and entrance
2. `general_store.json` - 8×6 building with counter and shelves
3. `residential_small.json` - 6×6 house with bed and table

### Phase 3: Integration (30 min)

**Integrate with settlement generation**:
- Load prefab library in `settlement/mod.rs`
- Pass library to building placement algorithm
- Place prefabs at semantic markers

---

## Character Legend Reference

### Walls
- `#` - Generic wall (context-dependent)
- `W` - WoodWall
- `S` - StoneWall
- `B` - BrickWall
- `G` - GlassWall
- `M` - MetalWall

### Floors
- `.` - Generic floor
- `0-9` - Floor variants (stone, wood, tile, etc.)

### Doors & Windows
- `D` - Door (generic)
- `+` - Closed door
- `-` - Open door
- `w` - Window

### Furniture
- `T` - Table
- `C` - Chair
- `b` - Bed
- `c` - Counter
- `s` - Shelf
- `x` - Chest
- `k` - Workbench
- `A` - Altar
- `H` - Throne

### Decorations
- `F` - Fountain
- `t` - Tree
- `f` - Flower
- `L` - Lamp
- `B` - Banner
- `I` - Sign
- `*` - Statue
- `g` - Garden

### Special
- `?` - Any/don't care (flexible tile)
- `@` - Player start
- `>` - Stairs down
- `<` - Stairs up
- ` ` (space) - Empty/void

---

## Example Prefabs

### Town Hall (10×8)

```json
{
  "name": "town_hall",
  "width": 10,
  "height": 8,
  "pattern": [
    "SSSSSSSSSS",
    "S........S",
    "S..TT....S",
    "S..TT....S",
    "S........S",
    "S........S",
    "S...DD...S",
    "SSSSSSSSSS"
  ],
  "weight": 1.0,
  "tags": ["core", "town_hall", "large"],
  "faction": null,
  "legend": {
    "S": {"tile": "StoneWall"},
    ".": {"tile": "Floor"},
    "D": {"tile": "Door", "marker": "entrance"},
    "T": {"tile": "Table"}
  },
  "metadata": {
    "description": "Central town hall with meeting tables",
    "min_tier": "village",
    "npc_spawns": []
  }
}
```

### General Store (8×6)

```json
{
  "name": "general_store",
  "width": 8,
  "height": 6,
  "pattern": [
    "WWWWWWWW",
    "W......W",
    "W.cccc.W",
    "W.ssss.W",
    "W..DD..W",
    "WWWWWWWW"
  ],
  "weight": 1.0,
  "tags": ["core", "store", "medium"],
  "faction": null,
  "legend": {
    "W": {"tile": "WoodWall"},
    ".": {"tile": "Floor"},
    "D": {"tile": "Door", "marker": "entrance"},
    "c": {"tile": "Counter"},
    "s": {"tile": "Shelf"}
  },
  "metadata": {
    "description": "General store with counter and shelves",
    "min_tier": "village",
    "npc_spawns": [
      {"npc_type": "merchant", "marker": "vendor"}
    ]
  }
}
```

### Residential Small (6×6)

```json
{
  "name": "residential_small",
  "width": 6,
  "height": 6,
  "pattern": [
    "######",
    "#....#",
    "#.b..#",
    "#.T..#",
    "#.D..#",
    "######"
  ],
  "weight": 3.0,
  "tags": ["residential", "small"],
  "faction": null,
  "legend": {
    "#": {"tile": "WoodWall"},
    ".": {"tile": "Floor"},
    "D": {"tile": "Door", "marker": "entrance"},
    "b": {"tile": "Bed"},
    "T": {"tile": "Table"}
  },
  "metadata": {
    "description": "Small residential house",
    "min_tier": "village",
    "npc_spawns": []
  }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_prefab_from_json() {
        let json = r#"
        {
          "prefabs": [{
            "name": "test_room",
            "width": 3,
            "height": 3,
            "pattern": ["###", "#.#", "###"],
            "weight": 1.0,
            "tags": ["test"],
            "faction": null,
            "legend": {
              "#": {"tile": "Wall"},
              ".": {"tile": "Floor"}
            }
          }]
        }
        "#;
        
        let data: PrefabLibraryData = serde_json::from_str(json).unwrap();
        assert_eq!(data.prefabs.len(), 1);
        assert_eq!(data.prefabs[0].name, "test_room");
    }

    #[test]
    fn test_prefab_rotation() {
        // Create 2×3 prefab
        // Rotate 90° clockwise
        // Verify dimensions are 3×2
        // Verify cells are correctly rotated
    }

    #[test]
    fn test_prefab_mirroring() {
        // Create asymmetric prefab
        // Mirror horizontally
        // Verify cells are flipped
    }

    #[test]
    fn test_tag_filtering() {
        let mut library = PrefabLibrary::new();
        // Add prefabs with different tags
        // Filter by tag
        // Verify correct prefabs returned
    }

    #[test]
    fn test_weighted_selection() {
        // Create prefabs with different weights
        // Select 1000 times
        // Verify distribution matches weights
    }
}
```

### Integration Tests

```rust
#[test]
fn test_load_prefab_directory() {
    let library = PrefabLibrary::load_from_dir("data/prefabs/core").unwrap();
    assert!(library.prefabs.len() > 0);
}

#[test]
fn test_faction_prefabs() {
    let library = PrefabLibrary::load_from_dir("data/prefabs").unwrap();
    let mirror_monk_prefabs = library.get_by_faction("MirrorMonks");
    assert!(mirror_monk_prefabs.len() >= 3); // At least 3 buildings
}
```

---

## Open Questions

1. **Should we support multi-floor prefabs?**
   - For now: No, keep it simple
   - Future: Could add `floors` array for multi-level buildings

2. **How to handle prefab validation?**
   - Validate dimensions match pattern
   - Validate all legend characters are used
   - Validate all pattern characters are in legend
   - Warn on unused legend entries

3. **Should prefabs support scripting/logic?**
   - For now: No, pure data
   - Future: Could add `on_enter`, `on_exit` hooks

4. **How to handle prefab versioning?**
   - Add `version` field to PrefabData
   - Migrate old prefabs on load
   - Log warnings for deprecated fields

5. **Should we support prefab inheritance?**
   - For now: No
   - Future: Could add `extends` field to reuse base prefabs

---

## Next Steps

1. **Review this design** with stakeholders
2. **Implement Phase 1** (core prefab system)
3. **Create example prefabs** (Phase 2)
4. **Test and iterate** based on results
5. **Document prefab creation guide** for content creators

---

## References

- [Terrain-Forge Prefab System](../../../terrain-forge/src/algorithms/prefab.rs)
- [One More Level Prefab Guide](https://github.com/Bozar/DevBlog/wiki/Design_DungeonPrefabs)
- [Golden Krone Hotel Prefab Article](https://www.goldenkronehotel.com/wp/2017/02/18/how-i-learned-to-stop-worrying-and-love-prefabs/)
- [Settlement Generation Plan](SETTLEMENT_GENERATION_PLAN.md)
