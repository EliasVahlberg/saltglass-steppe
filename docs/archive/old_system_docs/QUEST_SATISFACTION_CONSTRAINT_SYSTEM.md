# Quest Satisfaction Constraint System

## Overview

The Quest Satisfaction Constraint System ensures that quest content can spawn properly and is reachable/interactable for players. It integrates with the existing constraint validation system to guarantee playable quest content across all quest types.

## Architecture

### Core Components

1. **QuestConstraintSystem** - Main validation engine
2. **QuestConstraint** - Quest-specific constraint definitions
3. **ObjectiveRequirement** - Requirements for individual quest objectives
4. **AccessibilityRequirement** - Overall quest accessibility rules
5. **SpawnRequirement** - Content spawning requirements

### Integration Points

- **Tile Generation** - Validates constraints during map generation
- **World Generation** - Identifies quest locations and applies constraints
- **Quest System** - Provides quest definitions and objective data
- **Constraint System** - Leverages existing validation framework

## Constraint Types

### Accessibility Requirements

Ensure quest content is reachable and completable:

```rust
pub struct AccessibilityRequirement {
    pub min_connectivity_score: f32,      // 0.0-1.0, percentage of map reachable
    pub requires_player_spawn_access: bool, // Must be reachable from spawn
    pub max_blocked_objectives: u32,      // Max objectives that can be unreachable
}
```

**Examples:**
- Main quests: `min_connectivity_score: 0.9` (90% connectivity required)
- Side quests: `min_connectivity_score: 0.7` (70% connectivity acceptable)
- Critical quests: `max_blocked_objectives: 0` (no blocked objectives allowed)

### Spawn Requirements

Control content density and structure placement:

```rust
pub struct SpawnRequirement {
    pub required_biomes: Vec<String>,        // Must spawn in specific biomes
    pub required_microstructures: Vec<String>, // Must contain specific structures
    pub min_structure_coverage: f32,        // Minimum % of tile covered by structures
    pub hostile_density_range: (f32, f32),  // Min/max hostile entities per 100 tiles
}
```

**Examples:**
- Vitrified Library: `min_structure_coverage: 0.5` (50% coverage)
- Combat quests: `hostile_density_range: (0.4, 0.8)` (moderate to high danger)
- Safe zones: `hostile_density_range: (0.0, 0.2)` (low danger)

### Spatial Requirements

Define space and positioning needs for objectives:

```rust
pub struct SpatialRequirement {
    pub min_accessible_area: u32,           // Minimum walkable tiles needed
    pub max_distance_from_spawn: Option<u32>, // Maximum distance from player spawn
    pub requires_line_of_sight: bool,       // Must have clear line of sight
    pub requires_safe_path: bool,           // Must have safe (non-hostile) path
}
```

**Examples:**
- NPC conversations: `min_accessible_area: 4` (space around NPC)
- Archive terminals: `requires_line_of_sight: true` (clear visibility)
- Item collection: `requires_safe_path: true` (reachable without combat)

### Entity Requirements

Ensure required entities spawn and are accessible:

```rust
pub struct EntityRequirement {
    pub entity_type: String,      // "npc", "item", "enemy", "interactable"
    pub entity_id: String,        // Specific entity ID
    pub must_be_reachable: bool,  // Must be accessible to player
    pub must_be_interactable: bool, // Must be in interactable state
    pub min_spawn_chance: f32,    // Minimum spawn probability (0.0-1.0)
}
```

**Examples:**
- Quest NPCs: `min_spawn_chance: 1.0` (guaranteed spawn)
- Quest items: `must_be_reachable: true` (player can reach them)
- Interactive objects: `must_be_interactable: true` (functional state)

## Configuration System

### Quest-Specific Constraints

Define constraints for individual quests in `data/quest_constraints.json`:

```json
{
  "quest_id": "the_broken_key",
  "category": "main",
  "accessibility_requirements": {
    "min_connectivity_score": 0.95,
    "requires_player_spawn_access": true,
    "max_blocked_objectives": 0
  },
  "spawn_requirements": {
    "required_biomes": ["ruins"],
    "required_microstructures": ["vitrified_library_ruins"],
    "min_structure_coverage": 0.5,
    "hostile_density_range": [0.4, 0.8]
  }
}
```

### Default Constraints by Category

Fallback constraints for quest categories:

```json
{
  "default_constraints": {
    "main_quest": {
      "accessibility_requirements": {
        "min_connectivity_score": 0.9,
        "requires_player_spawn_access": true,
        "max_blocked_objectives": 0
      }
    }
  }
}
```

## Validation Process

### 1. Quest Location Detection

When traveling to a tile, the system:
1. Checks active quests for objectives at current coordinates
2. Identifies quest IDs with objectives at this location
3. Passes quest IDs to tile generation system

### 2. Constraint Generation

For each quest:
1. Loads quest-specific constraints from configuration
2. Falls back to category defaults if no specific constraints exist
3. Generates entity requirements based on objective types
4. Creates spatial requirements based on objective complexity

### 3. Validation During Generation

During tile generation:
1. Validates accessibility requirements (connectivity, reachability)
2. Validates spawn requirements (structure coverage, hostile density)
3. Validates spatial requirements for each objective
4. Validates entity requirements (presence, reachability)

### 4. Constraint Enforcement

If critical constraints fail:
1. Regenerates tile with different parameters
2. Applies emergency fixes (connectivity improvements, entity placement)
3. Re-validates constraints after fixes
4. Ensures at least minimum playability standards

## Extensibility

### Adding New Quest Types

1. **Define Constraint Template**:
```rust
// Add to generate_quest_constraint()
"exploration" => (
    AccessibilityRequirement {
        min_connectivity_score: 0.8,
        requires_player_spawn_access: true,
        max_blocked_objectives: 1,
    },
    SpawnRequirement {
        required_biomes: vec!["ruins".to_string(), "oasis".to_string()],
        required_microstructures: vec![],
        min_structure_coverage: 0.2,
        hostile_density_range: (0.1, 0.4),
    }
),
```

2. **Add Configuration Entry**:
```json
{
  "default_constraints": {
    "exploration_quest": {
      "accessibility_requirements": {
        "min_connectivity_score": 0.8,
        "requires_player_spawn_access": true,
        "max_blocked_objectives": 1
      }
    }
  }
}
```

### Adding New Objective Types

1. **Define Spatial Requirements**:
```rust
// Add to generate_quest_constraint()
ObjectiveType::NewObjectiveType { .. } => SpatialRequirement {
    min_accessible_area: 6,
    max_distance_from_spawn: Some(120),
    requires_line_of_sight: true,
    requires_safe_path: false,
},
```

2. **Define Entity Requirements**:
```rust
// Add to generate_entity_requirements()
ObjectiveType::NewObjectiveType { target_id } => vec![
    EntityRequirement {
        entity_type: "special_entity".to_string(),
        entity_id: target_id.clone(),
        must_be_reachable: true,
        must_be_interactable: true,
        min_spawn_chance: 0.9,
    }
],
```

### Adding New Constraint Types

1. **Extend ConstraintType Enum**:
```rust
pub enum ConstraintType {
    // Existing types...
    Narrative,    // Story coherence constraints
    Difficulty,   // Challenge level constraints
    Atmosphere,   // Mood and theme constraints
}
```

2. **Implement Validation Logic**:
```rust
impl QuestConstraintSystem {
    fn validate_narrative_constraint(
        rule: &ConstraintRule,
        context: &ConstraintContext,
        quest_id: &str,
    ) -> ConstraintResult {
        // Custom validation logic
    }
}
```

## Usage Examples

### Basic Quest Validation

```rust
use crate::game::generation::quest_constraints::QuestConstraintSystem;

// Validate quest constraints during tile generation
let quest_ids = vec!["the_broken_key".to_string()];
let (satisfied, results) = QuestConstraintSystem::validate_tile_for_quests(
    &map,
    &quest_ids,
    &entities,
    &mut rng,
);

if !satisfied {
    // Handle constraint failures
    for result in results {
        if !result.passed {
            println!("Constraint failed: {}", result.message);
        }
    }
}
```

### Custom Quest Constraints

```rust
// Define custom constraints for a specific quest
let custom_constraint = QuestConstraint {
    quest_id: "custom_quest".to_string(),
    objective_requirements: vec![
        ObjectiveRequirement {
            objective_id: "find_artifact".to_string(),
            objective_type: ObjectiveType::Collect { 
                item_id: "ancient_artifact".to_string(), 
                count: 1 
            },
            spatial_requirements: SpatialRequirement {
                min_accessible_area: 20,
                max_distance_from_spawn: Some(200),
                requires_line_of_sight: false,
                requires_safe_path: true,
            },
            entity_requirements: vec![
                EntityRequirement {
                    entity_type: "item".to_string(),
                    entity_id: "ancient_artifact".to_string(),
                    must_be_reachable: true,
                    must_be_interactable: true,
                    min_spawn_chance: 1.0,
                }
            ],
        }
    ],
    accessibility_requirements: AccessibilityRequirement {
        min_connectivity_score: 0.85,
        requires_player_spawn_access: true,
        max_blocked_objectives: 0,
    },
    spawn_requirements: SpawnRequirement {
        required_biomes: vec!["ruins".to_string()],
        required_microstructures: vec!["ancient_temple".to_string()],
        min_structure_coverage: 0.4,
        hostile_density_range: (0.3, 0.6),
    },
};
```

## Benefits

### Guaranteed Playability

- **No Unreachable Objectives**: Connectivity validation ensures all objectives are accessible
- **Appropriate Challenge**: Hostile density controls ensure balanced difficulty
- **Required Content**: Entity requirements guarantee necessary NPCs/items spawn
- **Spatial Adequacy**: Area requirements ensure sufficient space for interactions

### Extensible Design

- **Data-Driven Configuration**: Easy to add new quests without code changes
- **Category-Based Defaults**: Consistent behavior for quest types
- **Modular Validation**: Individual constraint types can be extended independently
- **Flexible Requirements**: Mix and match constraint types for complex scenarios

### Development Support

- **Early Problem Detection**: Identifies quest issues during generation, not gameplay
- **Automated Testing**: Constraint validation can be unit tested
- **Performance Optimization**: Failed constraints trigger regeneration, not runtime fixes
- **Debug Information**: Detailed constraint results aid in troubleshooting

## Integration with Existing Systems

### Constraint System

The Quest Satisfaction Constraint System extends the existing constraint validation framework:

- Uses same `ConstraintResult` structure for consistency
- Integrates with `ConstraintSeverity` levels (Critical, Warning, Suggestion)
- Leverages existing emergency fix mechanisms
- Maintains compatibility with standard constraint validation

### Quest System

Seamlessly integrates with the quest system:

- Reads quest definitions from existing quest data files
- Uses `ObjectiveType` enum for type-safe objective handling
- Respects quest categories and progression requirements
- Works with active quest tracking and completion logic

### Generation Pipeline

Fits into the procedural generation pipeline:

- Validates constraints during tile generation phase
- Triggers regeneration when critical constraints fail
- Applies emergency fixes using existing connectivity algorithms
- Maintains deterministic generation with seeded RNG

This system ensures that all quest content is not only generated but is guaranteed to be playable, accessible, and appropriately challenging for players.
