use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::item::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChestDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub glyph: char,
    pub color: String,
    pub capacity: usize,
    pub loot_table: Option<String>,
    pub locked: bool,
    pub key_required: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chest {
    pub x: i32,
    pub y: i32,
    pub id: String,
    pub inventory: Vec<Item>,
    pub opened: bool,
    pub locked: bool,
}

static CHEST_DEFS: Lazy<HashMap<String, ChestDef>> = Lazy::new(|| {
    let data = include_str!("../../data/chests.json");
    let defs: Vec<ChestDef> = serde_json::from_str(data).expect("Failed to parse chests.json");
    defs.into_iter().map(|def| (def.id.clone(), def)).collect()
});

pub fn get_chest_def(id: &str) -> Option<&'static ChestDef> {
    CHEST_DEFS.get(id)
}

pub fn all_chest_ids() -> Vec<String> {
    CHEST_DEFS.keys().cloned().collect()
}

impl Chest {
    pub fn new(x: i32, y: i32, id: &str) -> Self {
        let def = get_chest_def(id).expect(&format!("Unknown chest: {}", id));
        Self {
            x,
            y,
            id: id.to_string(),
            inventory: Vec::new(),
            opened: false,
            locked: def.locked,
        }
    }

    pub fn new_with_loot(x: i32, y: i32, id: &str, loot: Vec<Item>) -> Self {
        let mut chest = Self::new(x, y, id);
        chest.inventory = loot;
        chest
    }

    pub fn get_def(&self) -> Option<&'static ChestDef> {
        get_chest_def(&self.id)
    }

    pub fn can_add_item(&self) -> bool {
        if let Some(def) = self.get_def() {
            self.inventory.len() < def.capacity
        } else {
            false
        }
    }

    pub fn add_item(&mut self, item: Item) -> bool {
        if self.can_add_item() {
            self.inventory.push(item);
            true
        } else {
            false
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.inventory.len() {
            Some(self.inventory.remove(index))
        } else {
            None
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn can_open(&self, player_has_key: bool) -> bool {
        !self.locked || player_has_key
    }

    pub fn name(&self) -> &str {
        self.get_def()
            .map(|d| d.name.as_str())
            .unwrap_or("Unknown Chest")
    }

    pub fn description(&self) -> &str {
        self.get_def().map(|d| d.description.as_str()).unwrap_or("")
    }
}
