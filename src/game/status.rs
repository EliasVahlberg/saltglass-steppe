use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusEffect {
    pub id: String,
    pub name: String,
    pub duration: i32, // Turns remaining
    pub stacks: u32,   // How many times applied
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusEffectDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub max_stacks: u32,
    #[serde(default)]
    pub tick_damage: i32,
    #[serde(default)]
    pub blocks_healing: bool,
    #[serde(default)]
    pub reduces_accuracy: i32,
    #[serde(default)]
    pub reduces_damage: i32,
    #[serde(default)]
    pub grants_invisibility: bool,
}

#[derive(Deserialize)]
struct StatusEffectsFile {
    status_effects: Vec<StatusEffectDef>,
}

static STATUS_EFFECT_DEFS: Lazy<HashMap<String, StatusEffectDef>> = Lazy::new(|| {
    let data = include_str!("../../data/status_effects.json");
    let file: StatusEffectsFile =
        serde_json::from_str(data).expect("Failed to parse status_effects.json");
    file.status_effects
        .into_iter()
        .map(|d| (d.id.clone(), d))
        .collect()
});

pub fn get_status_def(id: &str) -> Option<&'static StatusEffectDef> {
    STATUS_EFFECT_DEFS.get(id)
}

impl StatusEffect {
    pub fn new(id: &str, duration: i32) -> Self {
        let name = get_status_def(id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| id.to_string());
        Self {
            id: id.to_string(),
            name,
            duration,
            stacks: 1,
        }
    }

    pub fn tick(&mut self) -> i32 {
        self.duration -= 1;
        get_status_def(&self.id).map(|d| d.tick_damage).unwrap_or(0)
    }

    pub fn add_stack(&mut self, max_stacks: u32) {
        if self.stacks < max_stacks {
            self.stacks += 1;
        }
    }

    pub fn is_expired(&self) -> bool {
        self.duration <= 0
    }
}

// Legacy compatibility functions
pub fn is_stunned(_effects: &[StatusEffect]) -> bool {
    false // Placeholder
}

pub fn slow_penalty(_effects: &[StatusEffect]) -> i32 {
    0 // Placeholder
}

// Legacy type alias
pub type StatusType = String;
