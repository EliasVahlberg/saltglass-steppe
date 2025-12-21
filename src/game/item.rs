use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct EntityEffect {
    pub condition: String,
    pub effect: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LightSource {
    pub radius: i32,
    pub intensity: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub description: String,
    #[serde(default)]
    pub value: u32,
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub usable: bool,
    #[serde(default)]
    pub heal: i32,
    #[serde(default)]
    pub reveals_map: bool,
    #[serde(default)]
    pub suppresses_adaptations: bool,
    #[serde(default)]
    pub breaks_walls: bool,
    #[serde(default)]
    pub reveals_storm_path: bool,
    #[serde(default)]
    pub reduces_refraction: u32,
    #[serde(default)]
    pub grants_invisibility: bool,
    #[serde(default)]
    pub reveals_storm_timing: bool,
    #[serde(default)]
    pub reveals_locations: bool,
    #[serde(default)]
    pub stackable: bool,
    #[serde(default)]
    pub grows_over_time: bool,
    #[serde(default)]
    pub armor_value: i32,
    #[serde(default)]
    pub equip_slot: Option<String>,
    #[serde(default)]
    pub effects: Vec<EntityEffect>,
    #[serde(default)]
    pub hidden_properties: Vec<String>,
    #[serde(default = "default_pickup")]
    pub pickup: bool,
    #[serde(default)]
    pub light_source: Option<LightSource>,
}

fn default_pickup() -> bool { true }

#[derive(Deserialize)]
struct ItemsFile {
    items: Vec<ItemDef>,
}

static ITEM_DEFS: Lazy<HashMap<String, ItemDef>> = Lazy::new(|| {
    let data = include_str!("../../data/items.json");
    let file: ItemsFile = serde_json::from_str(data).expect("Failed to parse items.json");
    file.items.into_iter().map(|d| (d.id.clone(), d)).collect()
});

pub fn get_item_def(id: &str) -> Option<&'static ItemDef> {
    ITEM_DEFS.get(id)
}

pub fn all_item_ids() -> Vec<&'static str> {
    ITEM_DEFS.keys().map(|s| s.as_str()).collect()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    pub id: String,
}

impl Item {
    pub fn new(x: i32, y: i32, id: &str) -> Self {
        Self { x, y, id: id.to_string() }
    }

    pub fn def(&self) -> Option<&'static ItemDef> {
        get_item_def(&self.id)
    }

    pub fn glyph(&self) -> char {
        self.def().map(|d| d.glyph.chars().next().unwrap_or('?')).unwrap_or('?')
    }

    pub fn name(&self) -> &str {
        self.def().map(|d| d.name.as_str()).unwrap_or("Unknown")
    }
}
