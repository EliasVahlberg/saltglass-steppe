use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::item::Item;
use super::spawn::weighted_pick;
use super::generation::{WeightedTable, WeightedEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootEntry {
    pub item_id: String,
    pub weight: u32,
    pub min_count: u32,
    pub max_count: u32,
    pub chance: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTable {
    pub id: String,
    pub name: String,
    pub description: String,
    pub entries: Vec<LootEntry>,
    pub min_items: u32,
    pub max_items: u32,
}

static LOOT_TABLES: Lazy<HashMap<String, LootTable>> = Lazy::new(|| {
    let data = include_str!("../../data/loot_tables.json");
    let tables: Vec<LootTable> = serde_json::from_str(data).expect("Failed to parse loot_tables.json");
    tables.into_iter().map(|table| (table.id.clone(), table)).collect()
});

pub fn get_loot_table(id: &str) -> Option<&'static LootTable> {
    LOOT_TABLES.get(id)
}

pub fn generate_loot(table_id: &str, x: i32, y: i32, rng: &mut ChaCha8Rng) -> Vec<Item> {
    let table = match get_loot_table(table_id) {
        Some(t) => t,
        None => return Vec::new(),
    };
    
    let mut loot = Vec::new();
    let item_count = rng.gen_range(table.min_items..=table.max_items);
    
    for _ in 0..item_count {
        // Filter entries by chance
        let available_entries: Vec<_> = table.entries.iter()
            .filter(|entry| rng.gen_range(0.0..1.0) < entry.chance)
            .collect();
            
        if available_entries.is_empty() {
            continue;
        }
        
        // Create weighted spawn entries for selection
        let weighted_entries: Vec<_> = available_entries.iter()
            .map(|entry| super::spawn::WeightedSpawn {
                id: entry.item_id.clone(),
                weight: entry.weight,
                room: None,
                min_level: 1,
                max_level: 10,
            })
            .collect();
            
        if let Some(item_id) = weighted_pick(&weighted_entries, rng) {
            let entry = available_entries.iter()
                .find(|e| e.item_id == item_id)
                .unwrap();
                
            let count = rng.gen_range(entry.min_count..=entry.max_count);
            for _ in 0..count {
                loot.push(Item::new(x, y, item_id));
            }
        }
    }
    
    loot
}

/// Enhanced loot generation using WeightedTable system
pub fn generate_loot_enhanced(table_id: &str, x: i32, y: i32, rng: &mut ChaCha8Rng) -> Vec<Item> {
    let table = match get_loot_table(table_id) {
        Some(t) => t,
        None => return Vec::new(),
    };
    
    let mut loot = Vec::new();
    let item_count = rng.gen_range(table.min_items..=table.max_items);
    
    for _ in 0..item_count {
        // Filter entries by chance and create weighted table
        let available_entries: Vec<_> = table.entries.iter()
            .filter(|entry| rng.gen_range(0.0..1.0) < entry.chance)
            .collect();
            
        if available_entries.is_empty() {
            continue;
        }
        
        // Create weighted table directly
        let weighted_entries: Vec<WeightedEntry<&LootEntry>> = available_entries.iter()
            .map(|entry| WeightedEntry { item: *entry, weight: entry.weight as f32 })
            .collect();
            
        let weighted_table = WeightedTable::new(weighted_entries);
        
        if let Some(entry) = weighted_table.select(rng) {
            let count = rng.gen_range(entry.min_count..=entry.max_count);
            for _ in 0..count {
                loot.push(Item::new(x, y, &entry.item_id));
            }
        }
    }
    
    loot
}
