//! Meta progression - persistent unlocks across game runs

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const META_FILE: &str = "meta_progress.ron";

#[derive(Debug, Clone, Deserialize)]
pub struct ClassDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub unlocked_by_default: bool,
    #[serde(default)]
    pub unlock_condition: Option<String>,
    pub starting_hp: i32,
    pub starting_ap: i32,
    #[serde(default)]
    pub starting_items: Vec<String>,
    #[serde(default)]
    pub starting_adaptations: Vec<String>,
}

#[derive(Deserialize)]
struct ClassesFile {
    classes: Vec<ClassDef>,
}

static CLASSES: Lazy<Vec<ClassDef>> = Lazy::new(|| {
    let data = include_str!("../../data/classes.json");
    let file: ClassesFile = serde_json::from_str(data).expect("Failed to parse classes.json");
    file.classes
});

pub fn all_classes() -> &'static [ClassDef] {
    &CLASSES
}

pub fn get_class(id: &str) -> Option<&'static ClassDef> {
    CLASSES.iter().find(|c| c.id == id)
}

/// Persistent meta progress data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaProgress {
    pub unlocked_classes: HashSet<String>,
    pub total_items_collected: u32,
    pub max_refraction_reached: u32,
    pub total_enemies_killed: u32,
    pub runs_completed: u32,
    #[serde(default)]
    pub discovered_items: HashSet<String>,
    #[serde(default)]
    pub discovered_enemies: HashSet<String>,
    #[serde(default)]
    pub discovered_npcs: HashSet<String>,
}

impl MetaProgress {
    pub fn load() -> Self {
        if Path::new(META_FILE).exists() {
            if let Ok(data) = fs::read_to_string(META_FILE) {
                if let Ok(meta) = ron::from_str(&data) {
                    return meta;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        fs::write(META_FILE, data)?;
        Ok(())
    }

    pub fn is_class_unlocked(&self, class_id: &str) -> bool {
        if let Some(class) = get_class(class_id) {
            if class.unlocked_by_default {
                return true;
            }
        }
        self.unlocked_classes.contains(class_id)
    }

    pub fn discover_item(&mut self, id: &str) {
        if self.discovered_items.insert(id.to_string()) {
            let _ = self.save();
        }
    }

    pub fn discover_enemy(&mut self, id: &str) {
        if self.discovered_enemies.insert(id.to_string()) {
            let _ = self.save();
        }
    }

    pub fn discover_npc(&mut self, id: &str) {
        if self.discovered_npcs.insert(id.to_string()) {
            let _ = self.save();
        }
    }

    pub fn check_unlocks(&mut self) {
        for class in all_classes() {
            if class.unlocked_by_default || self.unlocked_classes.contains(&class.id) {
                continue;
            }
            let unlocked = match class.unlock_condition.as_deref() {
                Some("collect_50_items") => self.total_items_collected >= 50,
                Some("reach_refraction_50") => self.max_refraction_reached >= 50,
                Some("kill_100_enemies") => self.total_enemies_killed >= 100,
                _ => false,
            };
            if unlocked {
                self.unlocked_classes.insert(class.id.clone());
            }
        }
    }

    pub fn record_run_stats(&mut self, items: u32, refraction: u32, kills: u32) {
        self.total_items_collected += items;
        self.max_refraction_reached = self.max_refraction_reached.max(refraction);
        self.total_enemies_killed += kills;
        self.runs_completed += 1;
        self.check_unlocks();
        let _ = self.save();
    }
}
