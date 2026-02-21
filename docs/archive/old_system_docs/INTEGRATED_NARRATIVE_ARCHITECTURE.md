# Integrated Narrative Systems Architecture

## Overview

The Saltglass Steppe employs a multi-layered narrative architecture that combines three interconnected systems to create rich, dynamic storytelling. These systems work together to generate contextual world stories that respond to player actions, environmental conditions, and persistent character relationships.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    GAME STATE                               │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐│
│  │  Story Model    │ │ Narrative Gen   │ │ Context System  ││
│  │                 │ │                 │ │                 ││
│  │ • Characters    │ │ • Templates     │ │ • Biome         ││
│  │ • Events        │ │ • Markov Chain  │ │ • Adaptations   ││
│  │ • Relationships │ │ • Variables     │ │ • Faction Rep   ││
│  │ • Factions      │ │ • Corpus        │ │ • Refraction    ││
│  │ • Artifacts     │ │                 │ │                 ││
│  └─────────────────┘ └─────────────────┘ └─────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 WORLD STORY GENERATION                      │
│                                                             │
│  Template Selection → Context Injection → Story Output     │
└─────────────────────────────────────────────────────────────┘
```

## The Three Narrative Systems

### 1. Story Model (Persistent Foundation)
**Purpose**: Provides the foundational narrative framework with persistent characters, events, and relationships.

**Components**:
- **Characters**: Named individuals with traits, achievements, faction affiliations
- **Events**: Chronological history of significant occurrences
- **Relationships**: Dynamic connections between characters with strength and type
- **Factions**: Power dynamics, territory, ideology, and recent actions
- **Artifacts**: Legendary items with creation myths and ownership history
- **Locations**: Significant places with founding events and inhabitants

**Data Persistence**: Fully serialized with game state, survives save/load cycles.

### 2. Narrative Generator (Template Engine)
**Purpose**: Generates dynamic text using template-based substitution and Markov chain generation.

**Components**:
- **Templates**: Structured text with variable placeholders
- **Variables**: Lists of contextually appropriate substitutions
- **Markov Chain**: Order-2 text generation from game corpus
- **Event Categories**: Storm events, faction conflicts, discoveries

**Generation Types**:
- Historical events
- Location descriptions
- Item lore
- Environmental storytelling
- Flavor text

### 3. Context System (State Awareness)
**Purpose**: Captures current game state to make narrative generation responsive to player situation.

**Context Elements**:
- **Biome**: Current terrain type (Saltflats, Dunes, etc.)
- **Adaptations**: Player mutations (Prismhide, Storm Sense, etc.)
- **Faction Reputation**: Standing with each major faction
- **Refraction Level**: Degree of storm transformation
- **Location Type**: Current environment category

## Story Generation Flow

### Phase 1: Foundation Layer (Story Model)
```rust
// Initialize persistent narrative elements
let story_model = StoryModel::new(seed);
// Creates:
// - 5 founding characters (one per faction)
// - 8 initial historical events
// - Faction power dynamics
// - Character relationships
```

### Phase 2: Template Layer (Narrative Generator)
```rust
// Load narrative templates from JSON
let generator = NarrativeGenerator::new()?;
// Provides:
// - Historical event templates
// - Location description templates
// - Contextual description templates
// - Environmental storytelling templates
// - Markov corpus for flavor text
```

### Phase 3: Context Layer (Dynamic State)
```rust
// Create context from current game state
let context = NarrativeContext {
    biome: Some("Saltflats".to_string()),
    adaptations: vec!["prismhide".to_string()],
    faction_reputation: player_faction_standing,
    refraction_level: player_refraction,
    location_type: Some("ruins".to_string()),
};
```

### Phase 4: Story Synthesis
```rust
// Generate contextual story content
let story = generator.generate_contextual_description(&context, &mut rng);
// Result: "Your crystalline skin refracts the ambient light as you observe 
//          the endless salt plains stretching before you..."
```

## Integration Points

### Artifact Inscriptions
**Flow**: Story Model → Character/Event Lookup → Template Application
```rust
// 1. Query story model for artifact
if let Some(artifact) = story_model.artifacts.get("Storm Glass Codex") {
    // 2. Format inscription with creator and legend
    format!("'{}' - {}", artifact.legend, artifact.creator)
}
// Output: "'Ancient wisdom crystallized in glass' - Saint Vex"
```

### Shrine Text
**Flow**: Story Model → Location Events → Narrative Generation
```rust
// 1. Find events at location
let events = story_model.events.iter()
    .filter(|e| e.location == Some(location))
    .collect();
// 2. Generate contextual shrine text
format!("Here {}", event.description.to_lowercase())
// Output: "Here the great storm revealed ancient secrets"
```

### Contextual Descriptions
**Flow**: Context System → Template Selection → Variable Substitution
```rust
// 1. Assess player context
if context.adaptations.contains("prismhide") {
    // 2. Select adaptation-aware template
    template = adaptation_templates.get("prismhide");
    // 3. Generate contextual description
    fill_template(template, rng)
}
// Output: "Your crystalline skin catches and bends nearby illumination"
```

### Environmental Storytelling
**Flow**: Story Model + Context → Template Selection → Atmospheric Text
```rust
// 1. Check for local story elements
if let Some(character) = story_model.characters.get(local_founder) {
    // 2. Generate inscription referencing character
    format!("Etched into the glass: 'In memory of {}'", character.name)
}
// 3. Fall back to template generation
else {
    generator.generate_environmental_text("inscriptions", rng)
}
```

## World Story Generation Process

### Initialization (World Creation)
1. **Seed Derivation**: Base seed generates sub-seeds for each system
   - Story Model: `seed + 3`
   - Narrative Generator: Uses template corpus
   - World History: `seed + 2`

2. **Foundation Generation**:
   ```rust
   // Create founding characters
   let founders = ["Saint Vex", "Keth the Seeker", "Naia Glassborn", 
                   "The Salt Prophet", "Archive Keeper Zara"];
   
   // Generate initial events
   for i in 0..8 {
       let event = create_historical_event(rng, characters, factions);
       story_model.events.push(event);
   }
   
   // Establish faction dynamics
   initialize_faction_power_levels(rng);
   ```

3. **Template Loading**:
   ```rust
   // Load narrative templates from JSON
   let templates = load_templates("data/narrative_templates.json")?;
   
   // Build Markov chain from corpus
   let markov_chain = MarkovChain::new(&templates.markov_corpus);
   ```

### Runtime Generation (During Gameplay)

1. **Context Assessment**:
   ```rust
   let context = create_narrative_context(game_state);
   // Captures: biome, adaptations, faction rep, refraction level
   ```

2. **Story Selection Priority**:
   - **Story Model First**: Check for relevant characters/events/artifacts
   - **Contextual Templates**: Use player state for template selection
   - **Generic Templates**: Fall back to basic template generation
   - **Markov Generation**: Generate atmospheric flavor text

3. **Dynamic Integration**:
   ```rust
   // Example: Examining an artifact
   if let Some(inscription) = get_artifact_inscription(item_name) {
       // Use story model data
       display_message(inscription);
   } else if let Some(lore) = generate_contextual_item_lore(category, context) {
       // Use contextual template
       display_message(lore);
   } else {
       // Use basic template
       display_message(generate_item_lore(category));
   }
   ```

### Player Action Integration

1. **Event Recording**:
   ```rust
   // Player discovers something significant
   story_model.add_player_event(
       EventType::Discovery, 
       "Player uncovered the Prime Lens in the Crucible Ruins"
   );
   ```

2. **Relationship Updates**:
   ```rust
   // Player action affects faction standing
   if player_helps_mirror_monks() {
       faction_reputation["Mirror Monks"] += 10;
       // This affects future contextual generation
   }
   ```

3. **Narrative Feedback Loop**:
   ```rust
   // Future descriptions reference player actions
   if story_model.events.iter().any(|e| e.participants.contains("player")) {
       // Generate text acknowledging player's growing legend
       "Your reputation precedes you in these lands..."
   }
   ```

## System Synergies

### Story Model ↔ Narrative Generator
- **Story Model** provides concrete characters, events, and relationships
- **Narrative Generator** creates atmospheric descriptions and flavor text
- **Integration**: Story model data fills template variables for personalized content

### Context System ↔ Both Systems
- **Context** informs template selection in Narrative Generator
- **Context** influences story model queries (faction-specific content)
- **Integration**: Player state determines which narrative elements are emphasized

### Emergent Storytelling
The combination creates emergent narratives:
1. **Player** gains Prismhide adaptation
2. **Context System** notes adaptation in future generation
3. **Narrative Generator** selects adaptation-aware templates
4. **Story Model** provides faction reactions to transformed individuals
5. **Result**: Personalized story that reflects player's transformation journey

## Performance Considerations

### Memory Efficiency
- **Story Model**: ~5 characters, ~8 events, sparse relationship matrix
- **Templates**: Loaded once at startup, reused throughout gameplay
- **Context**: Created on-demand, minimal memory footprint

### Generation Speed
- **Story Queries**: O(1) HashMap lookups
- **Template Selection**: O(k) where k = number of context categories
- **Text Generation**: O(n) where n = template variables or Markov chain length

### Determinism Guarantee
- **Seeded RNG**: All systems use ChaCha8Rng with derived seeds
- **Reproducibility**: Same seed + same actions = identical stories
- **Save/Load**: Story model persists, templates reload consistently

## Future Evolution

### Planned Enhancements
1. **Dynamic Relationships**: Characters form new bonds during gameplay
2. **Faction Wars**: Large-scale conflicts reshape story model
3. **Generational Stories**: Character aging and succession
4. **Player Legacy**: Actions create lasting story model changes

### Expansion Possibilities
1. **Procedural Quests**: Story-driven missions from character relationships
2. **Cultural Evolution**: Faction ideologies change based on events
3. **Economic Narratives**: Trade relationships and resource conflicts
4. **Temporal Storytelling**: Past/present/future narrative layers

---

*This integrated architecture creates a living narrative ecosystem where persistent story elements, dynamic template generation, and contextual awareness combine to produce rich, personalized storytelling that evolves with player actions and world state.*
