use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemFilters {
    pub enabled: bool,
    pub blacklist: Vec<String>,
    pub whitelist: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutoExploreConfig {
    pub pickup_items: bool,
    pub avoid_dangers: bool,
    pub stop_on_enemies: bool,
    pub enemy_detection_range: i32,
    pub ignore_weak_enemies: bool,
    pub weak_enemy_threshold: i32,
    pub item_filters: ItemFilters,
    pub danger_types: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AutoExploreConfigFile {
    auto_explore: AutoExploreConfig,
}

static AUTO_EXPLORE_CONFIG: Lazy<AutoExploreConfig> = Lazy::new(|| {
    let data = include_str!("../../data/auto_explore_config.json");
    let file: AutoExploreConfigFile = serde_json::from_str(data)
        .expect("Failed to parse auto_explore_config.json");
    file.auto_explore
});

pub fn get_auto_explore_config() -> &'static AutoExploreConfig {
    &AUTO_EXPLORE_CONFIG
}

impl AutoExploreConfig {
    pub fn should_pickup_item(&self, item_id: &str) -> bool {
        if !self.pickup_items {
            return false;
        }
        
        if !self.item_filters.enabled {
            return true;
        }
        
        // If whitelist is not empty, only pick up whitelisted items
        if !self.item_filters.whitelist.is_empty() {
            return self.item_filters.whitelist.contains(&item_id.to_string());
        }
        
        // Otherwise, pick up everything except blacklisted items
        !self.item_filters.blacklist.contains(&item_id.to_string())
    }
    
    pub fn is_danger_type(&self, danger_type: &str) -> bool {
        self.danger_types.contains(&danger_type.to_string())
    }
}
