# Procedural Narrative System

## Overview

The Procedural Narrative System generates dynamic story content for Saltglass Steppe using techniques inspired by Caves of Qud's approach to procedural storytelling. The system combines template-based generation with Markov chain text generation to create contextual lore, descriptions, and historical events.

## Architecture

### Core Components

1. **NarrativeGenerator**: Main interface for generating procedural content
2. **NarrativeTemplate**: Template system with variable substitution
3. **MarkovChain**: Order-2 Markov chain for flavor text generation
4. **HistoricalEvent**: Structured events with narrative templates

### Data-Driven Design

All narrative content is defined in `data/narrative_templates.json`:

```json
{
  "historical_events": {
    "storm_events": [...],
    "faction_events": [...]
  },
  "location_descriptions": {
    "ruins": [...],
    "natural": [...]
  },
  "item_lore": {
    "artifacts": [...]
  },
  "markov_corpus": [...]
}
```

## Key Features

### 1. Historical Event Generation

Generates procedural world history using template-based events:

- **Storm Events**: Major glass storms that shaped the world
- **Faction Conflicts**: Clashes between Mirror Monks, Sand-Engineers, etc.
- **Discoveries**: Ancient artifacts and locations found by notable figures

### 2. Location Descriptions

Procedural descriptions for different location types:

- **Ruins**: Ancient structures with mysterious purposes
- **Natural Features**: Glass seams, salt formations, crystal groves

### 3. Item Lore Generation

Creates backstories for artifacts and items:

- **Origin Stories**: How items were created or discovered
- **Current State**: What condition they're in now
- **Mystical Properties**: Hints at their powers

### 4. Markov Chain Text Generation

Generates flavor text using order-2 Markov chains:

- **Corpus-based**: Uses existing game text as training data
- **Deterministic**: Same seed produces same output
- **Contextual**: Maintains thematic consistency

## Implementation Details

### Template System

Templates use `{variable}` placeholders replaced with random choices:

```rust
{
  "template": "The {storm_name} Storm of {year} {action} the {location}.",
  "variables": {
    "storm_name": ["Crimson", "Shattered", "Weeping"],
    "action": ["shattered", "transformed", "consumed"],
    "location": ["ancient citadel", "mirror gardens"]
  }
}
```

### Markov Chain Algorithm

1. **Corpus Processing**: Split text into word pairs (order-2)
2. **Chain Building**: Map word pairs to possible next words
3. **Generation**: Start with opening words, follow chain until sentence end
4. **Deterministic**: Seeded RNG ensures reproducibility

### Integration with GameState

The narrative system is integrated into `GameState`:

```rust
pub struct GameState {
    // ... other fields
    pub narrative_generator: Option<NarrativeGenerator>,
    pub world_history: Vec<String>,
}
```

Methods available:
- `generate_item_lore(category)`: Create item backstories
- `generate_location_description(type)`: Describe locations
- `generate_flavor_text(max_words)`: Generate atmospheric text
- `get_world_history()`: Access generated historical events

## Usage Examples

### Generating World History

```rust
let mut state = GameState::new(seed);
let history = state.get_world_history();
for event in history {
    println!("{}", event);
}
```

### Creating Item Lore

```rust
if let Some(lore) = state.generate_item_lore("artifacts") {
    println!("Item description: {}", lore);
}
```

### Location Descriptions

```rust
if let Some(desc) = state.generate_location_description("ruins") {
    println!("You see: {}", desc);
}
```

## Creative Direction Alignment

### Tone Consistency

The system maintains Saltglass Steppe's **mythic-reverent** tone:

- Uses setting-appropriate vocabulary (refraction, vitrified, storm)
- Avoids modern slang or generic fantasy terms
- Maintains mystery and wonder without comedy

### Thematic Integration

All generated content supports the core pillars:

1. **Mutation with Social Consequences**: References adaptations and faction reactions
2. **Storms Rewrite Maps**: Historical events focus on transformative storms
3. **Readable Light Tactics**: Descriptions emphasize light, reflection, angles
4. **TUI as Aesthetic Strength**: Concise, evocative text suitable for terminal display
5. **Authored Weirdness**: Strange but consistent world-building

## Performance Characteristics

### Memory Usage

- Templates loaded once at startup
- Markov chain built from small corpus (~15 sentences)
- Minimal runtime memory allocation

### Generation Speed

- Template filling: O(n) where n = number of variables
- Markov generation: O(m) where m = max_words parameter
- Suitable for real-time generation during gameplay

### Determinism

- All generation uses seeded ChaCha8Rng
- Same seed + same parameters = identical output
- Supports save/load and replay systems

## Testing

### Unit Tests

- `test_markov_chain()`: Validates chain building and generation
- `test_template_filling()`: Ensures variable substitution works

### DES Integration

- `narrative_system_test.json`: Comprehensive scenario testing
- Validates determinism, content quality, and integration

## Future Enhancements

### Planned Features

1. **Context-Aware Generation**: Adapt content based on player location/state
2. **Faction-Specific Vocabulary**: Different narrative styles per faction
3. **Dynamic Event Chains**: Link historical events causally
4. **Player Action Integration**: Generate lore based on player choices

### Expansion Opportunities

1. **NPC Dialogue Generation**: Procedural conversation trees
2. **Quest Narrative**: Dynamic quest descriptions and objectives
3. **Environmental Storytelling**: Procedural graffiti, inscriptions
4. **Adaptive Difficulty**: Narrative complexity based on player progress

## Technical Notes

### Error Handling

- Graceful fallback if templates fail to load
- Default text provided when generation fails
- Non-blocking initialization (game continues without narrative system)

### Serialization

- `NarrativeGenerator` marked with `#[serde(skip)]`
- Generated content (world_history) is serialized
- Templates reloaded on game load

### Modularity

- Self-contained module with minimal dependencies
- Clean interface through GameState methods
- Easy to disable or replace without affecting core gameplay

---

*This system provides the foundation for rich, procedural storytelling that enhances the Saltglass Steppe experience while maintaining performance and creative consistency.*
