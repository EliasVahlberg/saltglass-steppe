# Constraint System Guide

The constraint system provides validation and automatic correction for procedural generation, ensuring generated content meets quality and playability requirements.

## Overview

The constraint system validates generated maps against configurable rules and can automatically apply fixes when critical constraints fail. It prevents common procedural generation issues like insufficient open space, poor connectivity, or unbalanced resource distribution.

## Architecture

### Core Components

- **`ConstraintSystem`**: Main validation engine
- **`ConstraintContext`**: Holds map and entity data for validation
- **`ConstraintRule`**: Individual constraint definition
- **`ConstraintResult`**: Validation outcome with score and messages

### Constraint Types

| Type | Purpose | Example Use |
|------|---------|-------------|
| `Connectivity` | Path validation between points | Ensure spawn-to-exit paths exist |
| `Distance` | Spacing validation | Prevent entities from clustering |
| `Accessibility` | Reachability validation | All objectives must be accessible |
| `Balance` | Resource/content validation | Minimum open space requirements |
| `Placement` | Location validation | Biome-appropriate entity placement |
| `Resource` | Density validation | Prevent resource over-saturation |

### Severity Levels

- **`Critical`**: Must be satisfied (generation fails if not met)
- **`Warning`**: Should be satisfied (logged but doesn't fail generation)
- **`Suggestion`**: Nice to have (informational only)

## Configuration

Constraints are defined in `data/constraint_rules.json`:

```json
{
  "rules": [
    {
      "id": "minimum_open_space",
      "name": "Minimum Open Space",
      "constraint_type": "balance",
      "parameters": {
        "resource_type": "open_space",
        "min_amount": 2000,
        "max_amount": 20000
      },
      "severity": "critical",
      "enabled": true
    }
  ]
}
```

### Parameter Types by Constraint

**Connectivity:**
- `start_points`: Array of {x, y} coordinates
- `end_points`: Array of {x, y} coordinates

**Distance:**
- `entity_type`: String (optional, filters entities)
- `min_distance`: Float (minimum allowed distance)
- `max_distance`: Float (maximum allowed distance)

**Balance:**
- `resource_type`: String ("open_space" for walkable tiles)
- `min_amount`: Integer (minimum required)
- `max_amount`: Integer (maximum allowed)

**Placement:**
- `entity_type`: String (entity to validate)
- `forbidden_biomes`: Array of biome names

## Integration

### Basic Usage

```rust
use crate::game::generation::constraints::{ConstraintSystem, ConstraintContext};

// Create validation context
let context = ConstraintContext {
    map: &generated_map,
    biome: current_biome,
    entities: vec![],
    resources: vec![],
    objectives: vec![],
};

// Validate constraints
let results = ConstraintSystem::validate_constraints(&context, rng);
let critical_satisfied = ConstraintSystem::are_critical_constraints_satisfied(&results);

if !critical_satisfied {
    // Apply fixes or regenerate
}
```

### Tile Generation Integration

The tile generator uses constraints with retry logic:

```rust
const MAX_ATTEMPTS: usize = 5;

for attempt in 0..MAX_ATTEMPTS {
    let (mut map, clearings) = generate_base_terrain(seed, biome, terrain, poi);
    
    // Validate constraints
    let context = ConstraintContext { map: &map, /* ... */ };
    let results = ConstraintSystem::validate_constraints(&context, rng);
    let critical_satisfied = ConstraintSystem::are_critical_constraints_satisfied(&results);
    
    if critical_satisfied || attempt == MAX_ATTEMPTS - 1 {
        if !critical_satisfied {
            apply_emergency_fixes(&mut map, &results);
        }
        return (map, clearings);
    }
}
```

## Emergency Fixes

When critical constraints fail on the final attempt, emergency fixes automatically correct common issues:

### Open Space Fix
Converts walls to floors in a grid pattern to ensure minimum walkable area:

```rust
fn ensure_minimum_open_space(&self, map: &mut Map) {
    // Convert walls to floors every 3 tiles to create connected open space
    for y in (10..MAP_HEIGHT-10).step_by(3) {
        for x in (10..MAP_WIDTH-10).step_by(3) {
            if matches!(map.tiles[idx], Tile::Wall { .. }) {
                map.tiles[idx] = Tile::Floor;
            }
        }
    }
}
```

### Connectivity Fix
Creates corridors from center to map edges:

```rust
fn ensure_basic_connectivity(&self, map: &mut Map) {
    let center = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
    let edges = vec![(10, 10), (MAP_WIDTH - 10, 10), /* ... */];
    
    for edge in edges {
        create_corridor(map, center, edge);
    }
}
```

## Adding New Constraints

### 1. Define Constraint Rule

Add to `data/constraint_rules.json`:

```json
{
  "id": "enemy_density",
  "name": "Enemy Density Limit",
  "constraint_type": "resource",
  "parameters": {
    "max_density": 0.02
  },
  "severity": "warning",
  "enabled": true
}
```

### 2. Implement Validation Logic

For new constraint types, add validation in `ConstraintSystem`:

```rust
fn validate_custom_constraint(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
    // Custom validation logic
    let passed = /* validation check */;
    let score = if passed { 1.0 } else { 0.0 };
    
    ConstraintResult {
        rule_id: rule.id.clone(),
        passed,
        severity: rule.severity.clone(),
        message: format!("Custom validation result"),
        score,
    }
}
```

### 3. Add Emergency Fix (Optional)

For critical constraints, implement automatic fixes:

```rust
fn apply_emergency_fixes(&self, map: &mut Map, results: &[ConstraintResult]) {
    for result in results {
        if result.severity == ConstraintSeverity::Critical && !result.passed {
            match result.rule_id.as_str() {
                "enemy_density" => self.reduce_enemy_density(map),
                _ => {}
            }
        }
    }
}
```

## Best Practices

### Constraint Design
- **Start with warnings**: Test new constraints as warnings before making them critical
- **Provide clear messages**: Include actionable information in constraint messages
- **Use appropriate severity**: Only make constraints critical if they break gameplay

### Performance
- **Validate early**: Run constraints before expensive generation steps
- **Cache results**: Store validation results to avoid repeated calculations
- **Limit retries**: Use reasonable attempt limits to prevent infinite loops

### Emergency Fixes
- **Minimal changes**: Apply the smallest fix that satisfies the constraint
- **Preserve intent**: Maintain the original generation's character when possible
- **Test thoroughly**: Ensure fixes don't break other constraints

## Future Extensions

### Planned Features
- **Constraint dependencies**: Rules that depend on other constraint results
- **Dynamic parameters**: Constraint parameters that adapt based on game state
- **Constraint templates**: Reusable constraint patterns for common scenarios
- **Visual debugging**: Tools to visualize constraint violations and fixes

### Integration Opportunities
- **World generation**: Apply constraints to world-level features and POI placement
- **Entity spawning**: Validate spawn distributions and enemy placement
- **Quest generation**: Ensure quest objectives are achievable and well-distributed
- **Narrative placement**: Validate story fragment accessibility and pacing

## Examples

### Ensuring Boss Accessibility
```json
{
  "id": "boss_accessible",
  "name": "Boss Room Accessible",
  "constraint_type": "accessibility",
  "parameters": {},
  "severity": "critical",
  "enabled": true
}
```

### Balanced Loot Distribution
```json
{
  "id": "loot_balance",
  "name": "Loot Distribution Balance",
  "constraint_type": "balance",
  "parameters": {
    "resource_type": "loot",
    "min_amount": 5,
    "max_amount": 20
  },
  "severity": "warning",
  "enabled": true
}
```

### Biome-Appropriate Spawns
```json
{
  "id": "desert_no_water_enemies",
  "name": "No Water Enemies in Desert",
  "constraint_type": "placement",
  "parameters": {
    "entity_type": "water_elemental",
    "forbidden_biomes": ["desert", "dunes"]
  },
  "severity": "critical",
  "enabled": true
}
```

The constraint system provides a powerful, flexible foundation for ensuring procedural generation quality while maintaining the organic, varied content that makes each playthrough unique.
