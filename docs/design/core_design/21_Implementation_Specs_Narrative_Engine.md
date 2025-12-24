# Implementation Specifications: Narrative Engine

**Purpose:** Technical specs for tracking player choices and generating the dynamic epilogue described in `The_Chronicle_of_Choices.md`.

---

## 1. The Story Log System

We need a persistent way to track not just "Quests Completed" but "Narrative Choices Made."

### Data Structures (`src/game/narrative.rs`)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoryState {
    // Key flags that determine endings/variations
    pub flags: HashSet<String>,

    // Counters for "invisible" tracking
    pub counters: HashMap<String, i32>, // e.g., "violence_count", "diplomacy_count"

    // The history of major events for the summary
    pub journal_entries: Vec<JournalEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalEntry {
    pub turn: u32,
    pub text: String, // "Met Durgan at the Nexus."
    pub choice_id: Option<String>, // "sided_with_iron"
}

// In GameState
pub struct GameState {
    // ...
    pub story: StoryState,
}
```

### Helper Methods

```rust
impl GameState {
    pub fn set_story_flag(&mut self, flag: &str) {
        self.story.flags.insert(flag.to_string());
    }

    pub fn has_story_flag(&self, flag: &str) -> bool {
        self.story.flags.contains(flag)
    }

    pub fn add_journal(&mut self, text: &str) {
        self.story.journal_entries.push(JournalEntry {
            turn: self.turn,
            text: text.to_string(),
            choice_id: None,
        });
    }
}
```

---

## 2. Dynamic Epilogue Generation

The epilogue is not a static text file. It is constructed at runtime based on the `StoryState`.

### The Epilogue Builder (`src/game/epilogue.rs`)

```rust
pub fn generate_epilogue(state: &GameState) -> String {
    let mut text = String::new();

    // 1. Determine Main Ending
    if state.has_story_flag("ending_restoration") {
        text.push_str(include_str!("../../data/text/endings/restoration_main.txt"));
    } else if state.has_story_flag("ending_ascension") {
        text.push_str(include_str!("../../data/text/endings/ascension_main.txt"));
    }
    // ... other endings

    text.push_str("\n\n");

    // 2. Append Variations
    if state.has_story_flag("saved_hermits") {
        text.push_str("The Salt Hermits were integrated into the workforce...\n");
    } else {
        text.push_str("Without protection, the Hermit enclaves faded away...\n");
    }

    if state.has_story_flag("destroyed_archive") {
        text.push_str("History was lost...\n");
    }

    text
}
```

---

## 3. Integration with Dialogue System

The `DialogueOption` struct (defined in Spec 20) needs to be able to set these flags.

```rust
// Update to DialogueOption
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DialogueOption {
    pub text: String,
    pub next_node_id: Option<String>,

    // NEW: Narrative Effects
    pub set_flag: Option<String>, // e.g., "sided_with_iron"
    pub add_journal: Option<String>, // e.g., "I promised to help Durgan."
}
```

### Execution Logic

When a player selects an option:

1.  Check `condition`.
2.  Apply `effect` (Item/Quest).
3.  **Apply `set_flag`** (Narrative).
4.  **Apply `add_journal`** (Narrative).
5.  Move to `next_node_id`.
