use serde::{Deserialize, Serialize};

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
    pub max_stacks: u32,
    pub tick_damage: i32,
    pub blocks_healing: bool,
    pub reduces_accuracy: i32,
    pub reduces_damage: i32,
    pub grants_invisibility: bool,
}

impl StatusEffect {
    pub fn new(id: &str, duration: i32) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(), // Will be replaced with proper name from def
            duration,
            stacks: 1,
        }
    }

    pub fn tick(&mut self) -> i32 {
        self.duration -= 1;
        0 // Return damage dealt (placeholder for now)
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
