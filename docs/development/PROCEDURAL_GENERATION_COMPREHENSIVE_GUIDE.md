# Procedural Generation Guide - Saltglass Steppe

**Version:** 1.0  
**Date:** 2026-01-01  
**Purpose:** Comprehensive guide to procedural generation systems, configuration, and usage

---

## Table of Contents

1. [Overview](#overview)
2. [Core Generation Systems](#core-generation-systems)
3. [World Generation](#world-generation)
4. [Tile Generation](#tile-generation)
5. [Entity Spawning](#entity-spawning)
6. [Content Generation](#content-generation)
7. [Configuration Files](#configuration-files)
8. [Advanced Usage](#advanced-usage)
9. [Developer Guide](#developer-guide)
10. [Troubleshooting](#troubleshooting)

---

## Overview

Saltglass Steppe uses a comprehensive procedural generation system to create dynamic, varied, and immersive game worlds. The generation system is **deterministic** (same seed = same result), **data-driven** (configurable via JSON), and **modular** (systems work independently and together).

### Core Principles

- **Deterministic**: All generation uses seeded RNG (`ChaCha8Rng`) for reproducible results
- **Data-Driven**: Content defined in JSON files, no code changes needed for new content
- **Layered**: Multiple systems work together to create rich, varied content
- **Biome-Aware**: Generation adapts to environmental context
- **Contextual**: Content responds to player state, story progression, and world conditions

### What Gets Generated

| System | Generates | Configurable Via |
|--------|-----------|------------------|
| **World Generator** | Biomes, terrain, POIs, roads, elevation | `terrain_config.json` |
| **Tile Generator** | Detailed tile layouts, clearings, features | `terrain_config.json` |
| **Spawn System** | Enemies, NPCs, items by biome/level | `biome_spawn_tables.json` |
| **Loot System** | Treasure, drops, rewards | `loot_tables.json` |
| **Microstructures** | Small buildings, ruins, shrines | `microstructures.json` |
| **Biome System** | Environmental features, hazards | Built-in + config |
| **Grammar System** | Dynamic text, descriptions | `grammars/descriptions.json` |
| **Template System** | Encounters, events, content | `templates/content_templates.json` |
| **Event System** | Dynamic events, triggers | `dynamic_events.json` |
| **Narrative System** | Story fragments, lore placement | `narrative_integration.json` |
| **Story System** | Characters, relationships, history | Code-based |

---

## Core Generation Systems

### 1. Generation Pipeline

**Location**: `src/game/generation/pipeline.rs`

The central coordinator that orchestrates all generation systems.

```rust
// Basic usage
let config = GenerationConfig::default();
let context = GenerationContext::new(seed, biome, terrain);
let pipeline = GenerationPipeline::new(config);
let result = pipeline.generate(context);
```

**Configuration**:
```json
{
  "enable_biome_generation": true,
  "enable_narrative_fragments": true,
  "enable_dynamic_events": true,
  "complexity_threshold": 7,
  "max_generation_depth": 5
}
```

### 2. Weighted Selection System

**Location**: `src/game/generation/weighted_table.rs`

Provides consistent, configurable random selection across all systems.

```rust
// Create weighted table
let entries = vec![
    WeightedEntry { item: "common_item", weight: 70.0 },
    WeightedEntry { item: "rare_item", weight: 20.0 },
    WeightedEntry { item: "legendary_item", weight: 10.0 },
];
let table = WeightedTable::new(entries);
let selected = table.select(&mut rng);
```

**Used By**: Spawn tables, loot generation, POI placement, event triggers

---

## World Generation

### Overview

Creates the overworld map with biomes, terrain, points of interest, and connections.

**Location**: `src/game/generation/world_gen.rs`

### Process Flow

1. **Biome Generation**: Place biomes using noise and rules
2. **Terrain Generation**: Add terrain features within biomes  
3. **POI Placement**: Place towns, dungeons, landmarks with preferences
4. **Road Generation**: Connect POIs using minimum spanning tree
5. **Level Calculation**: Assign threat levels based on distance and features

### POI Placement System

POIs are placed using **biome and terrain preferences**:

```rust
// POI preferences (internal scoring)
let preferences = POIPreferences {
    biome_scores: {
        "town" => { "oasis": 0.9, "saltflat": 0.3, "ruins": 0.1 },
        "dungeon" => { "ruins": 0.8, "mesa": 0.7, "oasis": 0.2 },
        "shrine" => { "mesa": 0.6, "saltflat": 0.5, "scrubland": 0.4 }
    },
    terrain_scores: {
        "town" => { "flat": 0.8, "hills": 0.4, "canyon": 0.1 },
        "dungeon" => { "canyon": 0.9, "mesa": 0.7, "hills": 0.5 }
    }
};
```

### Configuration

**File**: `data/terrain_config.json`

```json
{
  "world_generation": {
    "poi_distribution": {
      "town": { "min_count": 8, "max_count": 12, "spacing": 15 },
      "dungeon": { "min_count": 6, "max_count": 10, "spacing": 12 },
      "shrine": { "min_count": 4, "max_count": 8, "spacing": 10 },
      "landmark": { "min_count": 3, "max_count": 6, "spacing": 8 }
    },
    "biome_weights": {
      "saltflat": 0.4,
      "oasis": 0.15,
      "ruins": 0.2,
      "scrubland": 0.25
    }
  }
}
```

### Usage

```rust
// Generate world
let seed = 12345;
let generator = WorldGenerator::new();
let world_map = generator.generate(seed);

// Access generated content
println!("Generated {} POIs", world_map.pois.len());
for poi in &world_map.pois {
    println!("POI: {} at ({}, {})", poi.poi_type, poi.x, poi.y);
}
```

---

## Tile Generation

### Overview

Creates detailed tile layouts when the player enters a new area.

**Location**: `src/game/generation/tile_gen.rs`

### Process Flow

1. **Base Terrain**: Generate organic terrain using multi-layer noise
2. **Biome Features**: Add biome-specific environmental elements
3. **POI Features**: Add POI-specific structures and clearings
4. **Glass Patterns**: Place glass formations with biome-specific patterns
5. **Content Generation**: Add descriptions, inscriptions, encounters

### Multi-Layer Noise Generation

```rust
// Three noise layers for rich terrain
let base_noise = noise.get([x as f64 * 0.1, y as f64 * 0.1]);
let variation_noise = noise.get([x as f64 * 0.3, y as f64 * 0.3]);
let feature_noise = noise.get([x as f64 * 0.05, y as f64 * 0.05]);

let combined = base_noise + (variation_noise * 0.3) + (feature_noise * 0.2);
```

### Biome-Specific Glass Patterns

Different biomes create unique glass formations:

```rust
match biome {
    Biome::Saltflat => {
        // Crystalline formations using mathematical functions
        let crystal_pattern = (x as f32 * 0.1).sin() * (y as f32 * 0.1).cos();
        if crystal_pattern > 0.7 { place_glass(x, y); }
    },
    Biome::Ruins => {
        // Shattered glass in geometric patterns
        if (x + y) % 7 == 0 && noise > 0.6 { place_glass(x, y); }
    },
    // ... other biomes
}
```

### Configuration

**File**: `data/terrain_config.json`

```json
{
  "biomes": {
    "saltflat": {
      "wall_types": ["salt_crystal", "hardened_salt"],
      "glass_density": 0.15,
      "feature_density": 0.8,
      "variation_intensity": 0.6,
      "unique_features": ["salt_spires", "crystal_gardens"]
    }
  },
  "terrain_types": {
    "canyon": {
      "wall_probability": 0.7,
      "glass_modifier": 1.2,
      "clearing_size_modifier": 0.8
    }
  },
  "poi_features": {
    "town": {
      "central_clearing_size": 8,
      "structure_density": 0.6,
      "special_features": ["town_square", "market_stalls"]
    }
  }
}
```

### Usage

```rust
// Generate tile
let generator = TileGenerator::new();
let map = generator.generate_enhanced(
    seed, biome, terrain, elevation, poi, 
    &mut rng
);

// The generator automatically:
// - Creates organic terrain layouts
// - Places biome-appropriate features
// - Generates contextual descriptions
// - Adds POI-specific structures
```

---

## Entity Spawning

### Overview

Spawns enemies, NPCs, and items based on biome, level, and context.

**Location**: `src/game/generation/spawn.rs`

### Spawn Tables

Each biome has weighted spawn tables for different entity types:

**File**: `data/biome_spawn_tables.json`

```json
{
  "saltflat": {
    "enemies": [
      {
        "id": "salt_mummy",
        "weight": 40,
        "min_level": 1,
        "max_level": 5,
        "room": "any"
      },
      {
        "id": "glass_beetle",
        "weight": 30,
        "min_level": 2,
        "max_level": 8,
        "room": "late"
      }
    ],
    "items": [
      {
        "id": "storm_glass",
        "weight": 25,
        "min_level": 1,
        "max_level": 10
      }
    ],
    "npcs": [
      {
        "id": "hermit_trader",
        "weight": 10,
        "min_level": 3,
        "max_level": 10,
        "room": "first"
      }
    ]
  }
}
```

### Level-Based Filtering

Spawns are filtered by player level and item tiers:

```rust
// Tier thresholds by level
let tier_threshold = match level {
    1 => 1,      // Only tier 1 items
    2..=3 => 2,  // Tier 1-2 items
    4..=6 => 3,  // Tier 1-3 items  
    7..=8 => 4,  // Tier 1-4 items
    9..=10 => 5, // All tiers
    _ => 1,
};
```

### Room Placement

Entities can be placed in specific room types:

- `"first"`: First room encountered
- `"last"`: Final room
- `"late"`: Last 2 rooms
- `"any"`: Any room (default)

### Usage

```rust
// Get spawn table for biome
let table = get_biome_spawn_table(&biome);

// Spawn enemy with level filtering
if let Some(enemy_id) = weighted_pick_by_level_and_tier(
    &table.enemies, level, &mut rng, false
) {
    let enemy = Enemy::new(x, y, enemy_id);
    enemies.push(enemy);
}

// Enhanced weighted selection
if let Some(item_id) = weighted_pick_enhanced(&table.items, &mut rng) {
    let item = Item::new(x, y, item_id);
    items.push(item);
}
```

---

## Content Generation

### Grammar System

**Location**: `src/game/generation/grammar.rs`

Generates dynamic text using rule-based expansion.

**File**: `data/grammars/descriptions.json`

```json
{
  "rules": {
    "area_description": {
      "expansions": [
        "A <material> <structure> <condition>",
        "The <atmosphere> <chamber> <detail>",
        "<weather> <light_quality> illuminates <feature>"
      ],
      "weights": [40.0, 35.0, 25.0]
    },
    "material": {
      "expansions": ["crystalline", "salt-crusted", "glass-fused", "weathered"],
      "weights": [30.0, 25.0, 25.0, 20.0]
    }
  }
}
```

**Usage**:
```rust
let grammar = Grammar::load_from_file("descriptions.json")?;
let context = GrammarContext::new()
    .with_variable("biome", "saltflat")
    .with_variable("weather", "storm");
    
let description = grammar.generate("area_description", &context, &mut rng);
// Output: "A crystalline chamber gleams with storm-light"
```

### Template System

**Location**: `src/game/generation/templates.rs`

Creates structured content with inheritance and variants.

**File**: `data/templates/content_templates.json`

```json
{
  "encounter_basic": {
    "id": "encounter_basic",
    "category": "encounter",
    "parameters": {
      "description": "You encounter ${enemy_type} in the ${biome}",
      "enemy_count": 1,
      "difficulty": "normal"
    }
  },
  "shrine_encounter": {
    "id": "shrine_encounter",
    "category": "encounter",
    "inheritance": "encounter_basic",
    "parameters": {
      "description": "A sacred ${enemy_type} guards the shrine",
      "enemy_count": 1,
      "difficulty": "hard",
      "special_loot": "shrine_relic"
    },
    "variants": [
      {
        "id": "abandoned_shrine",
        "weight": 30.0,
        "conditions": ["storm_intensity=low"],
        "overrides": {
          "enemy_count": 0,
          "description": "An abandoned shrine stands silent"
        }
      }
    ]
  }
}
```

**Usage**:
```rust
let library = TemplateLibrary::load_from_directory("templates/")?;
let context = TemplateContext::new()
    .with_variable("biome", "ruins")
    .with_variable("enemy_type", "glass_wraith");

let template = library.instantiate("shrine_encounter", &context, &mut rng)?;
println!("{}", template.get_parameter("description"));
```

### Biome System

**Location**: `src/game/generation/biomes.rs`

Generates biome-specific environmental content.

```rust
// Generate environmental features
let features = biome_system.generate_environmental_features(
    biome, terrain, &mut rng
);

// Features include:
// - Atmospheric elements (mood, weather effects)
// - Environmental hazards (biome-specific dangers)
// - Resource modifiers (material availability)
// - Mechanical effects (gameplay impacts)
```

**Example Output**:
```
Environmental Features:
- Salt crystals crunch underfoot (atmospheric)
- Caustic salt spray in the air (hazard: -1 HP/turn)
- Abundant salt deposits (resource: +50% salt gathering)
- Reflective surfaces cause glare (mechanical: -1 accuracy)
```

---

## Configuration Files

### Primary Configuration Files

| File | Purpose | Systems Using |
|------|---------|---------------|
| `terrain_config.json` | World/tile generation parameters | WorldGen, TileGen |
| `biome_spawn_tables.json` | Entity spawn weights by biome | SpawnSystem |
| `loot_tables.json` | Loot generation tables | LootSystem |
| `microstructures.json` | Small structure definitions | Microstructures |
| `dynamic_events.json` | Dynamic event triggers | EventSystem |
| `narrative_integration.json` | Story fragment rules | NarrativeSystem |
| `grammars/descriptions.json` | Text generation rules | Grammar |
| `templates/content_templates.json` | Content templates | TemplateSystem |

### Configuration Hierarchy

```
data/
├── terrain_config.json          # Core terrain/world config
├── biome_spawn_tables.json      # Spawn tables by biome
├── loot_tables.json             # Loot generation
├── microstructures.json         # Small structures
├── dynamic_events.json          # Dynamic events
├── narrative_integration.json   # Story fragments
├── grammars/
│   └── descriptions.json        # Text generation rules
└── templates/
    └── content_templates.json   # Content templates
```

### Adding New Content

#### New Enemy Type

1. **Add to spawn table** (`biome_spawn_tables.json`):
```json
{
  "id": "crystal_guardian",
  "weight": 15,
  "min_level": 5,
  "max_level": 10,
  "room": "last"
}
```

2. **Add enemy definition** (`enemies.json`):
```json
{
  "id": "crystal_guardian",
  "name": "Crystal Guardian",
  "glyph": "G",
  "max_hp": 45,
  "damage_min": 8,
  "damage_max": 12
}
```

#### New Loot Table

**File**: `loot_tables.json`
```json
{
  "id": "crystal_cache",
  "name": "Crystal Cache",
  "description": "Crystalline formations containing valuable materials",
  "entries": [
    {
      "item_id": "storm_glass",
      "weight": 40,
      "min_count": 2,
      "max_count": 4,
      "chance": 0.8
    },
    {
      "item_id": "crystal_shard",
      "weight": 30,
      "min_count": 1,
      "max_count": 2,
      "chance": 0.6
    }
  ],
  "min_items": 1,
  "max_items": 3
}
```

#### New Grammar Rules

**File**: `grammars/descriptions.json`
```json
{
  "crystal_formation": {
    "expansions": [
      "Crystalline spires reach toward <direction>",
      "Faceted <crystal_type> formations <action>",
      "A <size> crystal <structure> <condition>"
    ],
    "weights": [40.0, 35.0, 25.0]
  },
  "crystal_type": {
    "expansions": ["quartz", "salt", "glass", "prismatic"],
    "weights": [25.0, 30.0, 25.0, 20.0]
  }
}
```

---

## Advanced Usage

### Custom Generation Pipeline

```rust
// Create custom generation config
let config = GenerationConfig {
    enable_biome_generation: true,
    enable_narrative_fragments: false,  // Disable narratives
    complexity_threshold: 5,            // Lower complexity
    max_generation_depth: 3,            // Faster generation
};

// Create specialized context
let context = GenerationContext::new(seed, biome, terrain)
    .with_player_level(player_level)
    .with_story_state(story_progress)
    .with_faction_reputation(faction_rep);

// Run generation
let pipeline = GenerationPipeline::new(config);
let result = pipeline.generate(context);
```

### Constraint-Based Generation

**Location**: `src/game/generation/constraints.rs`

```rust
// Define constraints
let constraints = vec![
    ConstraintRule::new(
        ConstraintType::EntityPlacement,
        "no_enemies_near_npcs",
        ConstraintSeverity::Hard
    ),
    ConstraintRule::new(
        ConstraintType::ResourcePlacement,
        "balanced_loot_distribution",
        ConstraintSeverity::Soft
    )
];

// Apply constraints during generation
let constraint_system = ConstraintSystem::new(constraints);
let result = constraint_system.validate_and_adjust(generation_result);
```

### Spatial Distribution

```rust
// Use Poisson sampling for natural distribution
let mut sampler = PoissonSampler::new(map_width, map_height, min_distance);
let positions = sampler.sample_points(&candidate_positions, max_count, &mut rng);

// Or use grid-based distribution
let positions = distribute_points_grid(
    &candidate_positions, 
    max_count, 
    min_distance, 
    &mut rng
);
```

### Dynamic Event System

**File**: `dynamic_events.json`
```json
{
  "id": "storm_surge",
  "name": "Storm Surge",
  "description": "Glass storm intensity increases",
  "triggers": [
    {
      "trigger_type": "turn_multiple",
      "conditions": {"multiple": 50},
      "probability": 0.3
    },
    {
      "trigger_type": "biome_match",
      "conditions": {"biome": "saltflat"},
      "probability": 0.5
    }
  ],
  "consequences": [
    {
      "consequence_type": "add_refraction",
      "parameters": {"amount": 10}
    },
    {
      "consequence_type": "environmental_story",
      "parameters": {
        "message": "The storm's intensity surges around you!"
      }
    }
  ],
  "weight": 1.0,
  "cooldown_turns": 25
}
```

---

## Developer Guide

### Adding New Generation Systems

1. **Create System Module**:
```rust
// src/game/generation/my_system.rs
pub struct MyGenerationSystem {
    config: MyConfig,
}

impl MyGenerationSystem {
    pub fn new(config: MyConfig) -> Self {
        Self { config }
    }
    
    pub fn generate(&self, context: &GenerationContext, rng: &mut ChaCha8Rng) -> MyResult {
        // Generation logic here
    }
}
```

2. **Add to Module Exports**:
```rust
// src/game/generation/mod.rs
pub mod my_system;
pub use my_system::*;
```

3. **Integrate with Pipeline**:
```rust
// In GenerationPipeline::generate()
if self.config.enable_my_system {
    let result = my_system.generate(&context, rng);
    // Process result
}
```

### Best Practices

#### Deterministic Generation
```rust
// GOOD: Use provided RNG
let value = rng.gen_range(0..100);

// BAD: Use thread_rng (non-deterministic)
let value = thread_rng().gen_range(0..100);
```

#### Data-Driven Configuration
```rust
// GOOD: Load from config
let spawn_weight = config.get_spawn_weight(&enemy_id);

// BAD: Hardcode values
let spawn_weight = 25; // Magic number
```

#### Biome Awareness
```rust
// GOOD: Adapt to biome
let glass_density = match biome {
    Biome::Saltflat => 0.15,
    Biome::Ruins => 0.25,
    _ => 0.10,
};

// BAD: One-size-fits-all
let glass_density = 0.15; // Same everywhere
```

#### Error Handling
```rust
// GOOD: Graceful fallbacks
let template = library.get_template(id)
    .unwrap_or_else(|| library.get_default_template());

// BAD: Panic on missing content
let template = library.get_template(id).unwrap(); // Crashes game
```

### Testing Generation Systems

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;

    #[test]
    fn test_deterministic_generation() {
        let seed = 12345;
        let mut rng1 = ChaCha8Rng::seed_from_u64(seed);
        let mut rng2 = ChaCha8Rng::seed_from_u64(seed);
        
        let result1 = my_system.generate(&context, &mut rng1);
        let result2 = my_system.generate(&context, &mut rng2);
        
        assert_eq!(result1, result2); // Must be identical
    }
    
    #[test]
    fn test_biome_adaptation() {
        let saltflat_result = generate_for_biome(Biome::Saltflat);
        let ruins_result = generate_for_biome(Biome::Ruins);
        
        assert_ne!(saltflat_result, ruins_result); // Should differ
    }
}
```

---

## Troubleshooting

### Common Issues

#### Non-Deterministic Results
**Problem**: Same seed produces different results
**Cause**: Using `thread_rng()` instead of provided RNG
**Solution**: Always use the `ChaCha8Rng` passed to generation functions

#### Missing Content
**Problem**: Generation fails with missing items/enemies
**Cause**: References to undefined content in config files
**Solution**: Validate all IDs exist in corresponding definition files

#### Poor Distribution
**Problem**: Entities clump together or appear in wrong places
**Cause**: Incorrect spatial distribution or spawn rules
**Solution**: Use `PoissonSampler` or adjust spawn table weights

#### Performance Issues
**Problem**: Generation takes too long
**Cause**: Complex algorithms or excessive iterations
**Solution**: Use generation profiling and optimize hot paths

### Debug Tools

#### Generation Logging
```rust
// Enable detailed logging
log::debug!("Generating {} POIs for biome {:?}", count, biome);
log::trace!("POI placement score: {:.2}", score);
```

#### Mapgen Tool
```bash
# Test world generation
cargo run --bin mapgen-tool world 12345

# Test tile generation with POI
cargo run --bin mapgen-tool tile 12345 town

# Test different biomes
cargo run --bin mapgen-tool tile 12345 shrine
```

#### DES Testing
```bash
# Run generation-specific tests
cargo test --test des_scenarios generation

# Test with specific seed
cargo test --test des_scenarios -- --nocapture
```

### Configuration Validation

#### Spawn Table Validation
```rust
// Check spawn weights sum correctly
let total_weight: u32 = spawn_table.enemies.iter()
    .map(|e| e.weight)
    .sum();
assert!(total_weight > 0, "Spawn table has no valid entries");
```

#### Template Validation
```rust
// Validate template inheritance
for template in &templates {
    if let Some(parent_id) = &template.inheritance {
        assert!(templates.contains_key(parent_id), 
               "Template {} references missing parent {}", 
               template.id, parent_id);
    }
}
```

---

## Performance Considerations

### Generation Optimization

#### Caching
```rust
// Cache expensive calculations
static BIOME_FEATURES: Lazy<HashMap<Biome, Vec<Feature>>> = Lazy::new(|| {
    // Pre-calculate biome features
});
```

#### Lazy Loading
```rust
// Load content on-demand
pub fn get_loot_table(id: &str) -> Option<&'static LootTable> {
    LOOT_TABLES.get(id) // Lazy-loaded static
}
```

#### Batch Operations
```rust
// Generate multiple entities at once
let spawn_positions = spatial_system.distribute_points(&candidates, count, rng);
let entities: Vec<_> = spawn_positions.iter()
    .map(|&pos| spawn_entity_at(pos, &spawn_table, rng))
    .collect();
```

### Memory Management

- Use `&'static` references for config data
- Prefer `Vec<T>` over `HashMap<K, V>` for small collections
- Use `once_cell::Lazy` for expensive initialization
- Avoid cloning large data structures

---

## Conclusion

The procedural generation system in Saltglass Steppe provides a powerful, flexible foundation for creating varied, immersive game content. By combining deterministic algorithms, data-driven configuration, and modular design, it enables rich procedural worlds while maintaining consistency and performance.

Key strengths:
- **Deterministic**: Reproducible results for testing and debugging
- **Data-Driven**: Easy content creation without code changes
- **Modular**: Systems work independently and together
- **Contextual**: Content adapts to game state and player progress
- **Extensible**: Easy to add new generation systems and content types

The system supports everything from basic entity spawning to complex narrative generation, providing the tools needed to create the dynamic, ever-changing world of the Saltglass Steppe.
