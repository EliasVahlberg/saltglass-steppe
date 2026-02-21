# Procedural Narrative System

## Overview

The Procedural Narrative System generates dynamic story content for Saltglass Steppe using techniques inspired by Caves of Qud's approach to procedural storytelling. The system combines template-based generation with Markov chain text generation to create contextual lore, descriptions, and historical events that respond to the player's current state, location, and actions.

## Architecture

### Core Components

1. **NarrativeGenerator**: Main interface for generating procedural content
2. **NarrativeTemplate**: Template system with variable substitution
3. **NarrativeContext**: Context-aware generation based on game state
4. **MarkovChain**: Order-2 Markov chain for flavor text generation
5. **HistoricalEvent**: Structured events with narrative templates

### Context-Aware Generation

The system now responds to:
- **Player Adaptations**: Different descriptions for Prismhide, Storm Sense, etc.
- **Current Biome**: Saltflats, Dunes, etc. get unique descriptions
- **Faction Reputation**: High reputation with factions influences narrative tone
- **Refraction Level**: High refraction unlocks special item lore
- **Location Type**: Ruins, natural features, etc. get appropriate descriptions

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
  "contextual_descriptions": {
    "biome_specific": {...},
    "adaptation_aware": {...},
    "faction_influenced": {...}
  },
  "item_lore": {
    "artifacts": [...],
    "contextual_artifacts": [...]
  },
  "environmental_storytelling": {
    "inscriptions": [...],
    "graffiti": [...]
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

### 2. Context-Aware Location Descriptions

Procedural descriptions that adapt to current game state:

- **Biome-Specific**: Different descriptions for Saltflats vs Dunes
- **Adaptation-Aware**: Prismhide characters see light differently
- **Faction-Influenced**: Mirror Monk reputation affects interpretation

### 3. Dynamic Item Lore Generation

Creates backstories for artifacts and items:

- **Origin Stories**: How items were created or discovered
- **Current State**: What condition they're in now
- **Contextual Resonance**: High refraction items respond to player mutations

### 4. Environmental Storytelling

Generates atmospheric text found in the world:

- **Inscriptions**: Etched messages on glass and crystal
- **Graffiti**: Urgent warnings and notes from other travelers
- **Contextual Placement**: Appropriate to location and situation

### 5. Markov Chain Text Generation

Generates flavor text using order-2 Markov chains:

- **Corpus-based**: Uses existing game text as training data
- **Deterministic**: Same seed produces same output
- **Contextual**: Maintains thematic consistency

## Implementation Details

### Context System

The `NarrativeContext` struct captures current game state:

```rust
pub struct NarrativeContext {
    pub biome: Option<String>,
    pub terrain: Option<String>,
    pub adaptations: Vec<String>,
    pub faction_reputation: HashMap<String, i32>,
    pub refraction_level: u32,
    pub location_type: Option<String>,
}
```

### Template System

Templates use `{variable}` placeholders replaced with random choices:

```rust
{
  "template": "Your crystalline skin {reaction} as you observe {subject}.",
  "variables": {
    "reaction": ["refracts the ambient light", "creates small rainbows"],
    "subject": ["the glass formations around you", "the storm-touched landscape"]
  }
}
```

### Integration with GameState

The narrative system is integrated into `GameState` with context-aware methods:

```rust
impl GameState {
    pub fn generate_contextual_description(&mut self) -> Option<String>
    pub fn generate_contextual_item_lore(&mut self, category: &str) -> Option<String>
    pub fn generate_environmental_text(&mut self, env_type: &str) -> Option<String>
    
    fn create_narrative_context(&self) -> NarrativeContext
}
```

## Usage Examples

### Context-Aware Descriptions

```rust
// Generates description based on current biome, adaptations, faction rep
if let Some(desc) = state.generate_contextual_description() {
    println!("{}", desc);
}
// Output might be: "Your crystalline skin refracts the ambient light as you observe the glass formations around you."
```

### Environmental Storytelling

```rust
if let Some(inscription) = state.generate_environmental_text("inscriptions") {
    println!("{}", inscription);
}
// Output: "Etched into the crystal formation: 'The angle reveals all truths'"
```

### High-Refraction Item Lore

```rust
// With high refraction (>50), gets special contextual lore
if let Some(lore) = state.generate_item_lore("artifacts") {
    println!("{}", lore);
}
// Output: "This quantum mirror resonates deeply with your transformed nature, revealing hidden light patterns."
```

## Creative Direction Alignment

### Tone Consistency

The system maintains Saltglass Steppe's **mythic-reverent** tone:

- Uses setting-appropriate vocabulary (refraction, vitrified, storm)
- Avoids modern slang or generic fantasy terms
- Maintains mystery and wonder without comedy

### Contextual Immersion

All generated content responds to player state:

1. **Adaptation Awareness**: Prismhide characters see light differently
2. **Faction Integration**: Mirror Monk reputation affects narrative perspective
3. **Biome Specificity**: Saltflats and Dunes have distinct atmospheric descriptions
4. **Progression Sensitivity**: High refraction unlocks deeper lore connections

## Performance Characteristics

### Memory Usage

- Templates loaded once at startup
- Context created on-demand from game state
- Minimal runtime memory allocation

### Generation Speed

- Context creation: O(1) - simple field copying
- Template filling: O(n) where n = number of variables
- Contextual selection: O(k) where k = number of context categories
- Suitable for real-time generation during gameplay

### Determinism

- All generation uses seeded ChaCha8Rng
- Same seed + same context = identical output
- Supports save/load and replay systems

## Testing

### Unit Tests

- `test_markov_chain()`: Validates chain building and generation
- `test_template_filling()`: Ensures variable substitution works
- `test_contextual_generation()`: Validates context structure

### DES Integration

- `narrative_system_test.json`: Comprehensive scenario testing
- Validates determinism, content quality, context awareness, and integration

## Future Enhancements

### Planned Features

1. **Dynamic Event Chains**: Link historical events causally
2. **Player Action Integration**: Generate lore based on player choices
3. **Temporal Context**: Time-of-day and weather-influenced descriptions
4. **Relationship Dynamics**: NPC-specific narrative variations

### Expansion Opportunities

1. **NPC Dialogue Generation**: Context-aware conversation trees
2. **Quest Narrative**: Dynamic quest descriptions and objectives
3. **Adaptive Complexity**: Narrative sophistication based on player progress
4. **Memory Integration**: Reference past player actions in generated content

## Technical Notes

### Error Handling

- Graceful fallback if templates fail to load
- Default text provided when generation fails
- Context creation never fails (uses sensible defaults)

### Serialization

- `NarrativeGenerator` marked with `#[serde(skip)]`
- Generated content (world_history) is serialized
- Context created fresh from serialized game state

### Modularity

- Self-contained module with clean interfaces
- Context system decoupled from specific game mechanics
- Easy to extend with new context types

---

*This enhanced system provides rich, context-aware procedural storytelling that dynamically responds to player state while maintaining the mythic-reverent tone and creative consistency of the Saltglass Steppe.*
