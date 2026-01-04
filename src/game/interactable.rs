use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InteractableDef {
    pub id: String,
    pub name: String,
    pub glyph: char,
    pub description: String,
    pub interaction_type: String,
    pub states: Vec<String>,
    pub messages: InteractableMessages,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InteractableMessages {
    pub interact: String,
    pub examine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interactable {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub current_state: usize,
}

impl Interactable {
    pub fn new(id: String, x: i32, y: i32) -> Self {
        Self {
            id,
            x,
            y,
            current_state: 0,
        }
    }

    pub fn def(&self) -> Option<&'static InteractableDef> {
        get_interactable_def(&self.id)
    }

    pub fn interact(&mut self) -> Option<String> {
        if let Some(def) = self.def() {
            match def.interaction_type.as_str() {
                "toggle" => {
                    self.current_state = (self.current_state + 1) % def.states.len();
                }
                "press" => {
                    self.current_state = 1; // Pressed state
                }
                _ => {}
            }
            Some(def.messages.interact.clone())
        } else {
            None
        }
    }

    pub fn examine(&self) -> Option<String> {
        if let Some(def) = self.def() {
            let unknown = "unknown".to_string();
            let state_name = def.states.get(self.current_state).unwrap_or(&unknown);
            let message = def.messages.examine.replace("{state}", state_name);
            Some(message)
        } else {
            None
        }
    }

    pub fn glyph(&self) -> char {
        self.def().map(|d| d.glyph).unwrap_or('?')
    }

    pub fn name(&self) -> String {
        self.def().map(|d| d.name.clone()).unwrap_or_else(|| "Unknown".to_string())
    }
}

#[derive(Deserialize)]
struct InteractablesFile {
    interactables: Vec<InteractableDef>,
}

static INTERACTABLES: Lazy<HashMap<String, InteractableDef>> = Lazy::new(|| {
    let data = include_str!("../../data/interactables.json");
    let file: InteractablesFile = serde_json::from_str(data).expect("Failed to parse interactables.json");
    file.interactables.into_iter().map(|i| (i.id.clone(), i)).collect()
});

pub fn get_interactable_def(id: &str) -> Option<&'static InteractableDef> {
    INTERACTABLES.get(id)
}

pub fn all_interactable_ids() -> Vec<&'static str> {
    INTERACTABLES.keys().map(|s| s.as_str()).collect()
}
