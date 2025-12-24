# Implementation Specifications: Quests & Dialogue

**Purpose:** Technical specs for implementing the systems defined in `19_Quest_Structure_and_Endings.md`.

---

## 1. Quest System

### Data Structures (`src/game/quests.rs`)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum QuestStatus {
    NotStarted,
    InProgress(u32), // Current stage index
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestStage {
    pub description: String,
    pub objectives: Vec<Objective>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Objective {
    KillEnemy { id: String, count: u32, current: u32 },
    CollectItem { id: String, count: u32, current: u32 },
    VisitLocation { x: u32, y: u32 },
    TalkToNpc { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestDef {
    pub id: String,
    pub title: String,
    pub stages: Vec<QuestStage>,
    pub rewards: Vec<Reward>,
    pub next_quest_id: Option<String>, // For chains
}
```

### Logic
*   **Tracking:** `GameState` holds a `HashMap<String, QuestStatus>`.
*   **Updates:** Event listeners (EnemyDeath, ItemPickup) trigger `update_objectives()`.
*   **Completion:** When all objectives in a stage are met, advance stage. If last stage, mark Completed and grant rewards.

---

## 2. Dialogue System

### Data Structures (`src/game/dialogue.rs`)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DialogueNode {
    pub id: String,
    pub text: String, // The NPC's line
    pub speaker: String, // "Monk", "Engineer", etc.
    pub options: Vec<DialogueOption>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DialogueOption {
    pub text: String, // The Player's response
    pub next_node_id: Option<String>, // None = End Dialogue
    pub condition: Option<Condition>, // e.g., "HasItem", "Refraction > 50"
    pub effect: Option<Effect>, // e.g., "GiveItem", "StartQuest"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Condition {
    HasItem(String),
    MinRefraction(u32),
    FactionRep(String, i32),
}
```

### UI Implementation (`src/ui/dialogue_view.rs`)
*   **Layout:** A modal popup over the map.
*   **Text:** NPC text at the top, scrollable.
*   **Options:** Numbered list at the bottom.
*   **Styling:** Use Faction Colors for borders (e.g., Cyan for Monks, Orange for Engineers).

---

## 3. End Game State

### The "Ending" Flag
*   `GameState` needs a `game_over_state: Option<EndingType>`.
*   When triggered, the Main Loop stops the game tick and renders the **Epilogue Screen**.

### Epilogue Screen
*   Displays a static image (ASCII art) representing the chosen ending.
*   Displays text describing the consequences (based on `19_Quest_Structure_and_Endings.md`).
*   "Press Any Key to Return to Menu".
