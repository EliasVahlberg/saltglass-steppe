use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct LightDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub radius: i32,
    pub intensity: u8,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpawnRule {
    pub lights_per_room: [u32; 2],
    pub weights: HashMap<String, u32>,
}

#[derive(Deserialize)]
struct LightsFile {
    lights: Vec<LightDef>,
    spawn_rules: HashMap<String, SpawnRule>,
}

static LIGHTS_DATA: Lazy<LightsFile> = Lazy::new(|| {
    let data = include_str!("../../data/lights.json");
    serde_json::from_str(data).expect("Failed to parse lights.json")
});

pub fn get_light_def(id: &str) -> Option<&'static LightDef> {
    LIGHTS_DATA.lights.iter().find(|l| l.id == id)
}

pub fn get_spawn_rule(biome: &str) -> &'static SpawnRule {
    LIGHTS_DATA
        .spawn_rules
        .get(biome)
        .or_else(|| LIGHTS_DATA.spawn_rules.get("default"))
        .expect("No default spawn rule")
}

pub fn pick_light_type(rule: &SpawnRule, rng: &mut impl rand::Rng) -> Option<String> {
    let total: u32 = rule.weights.values().sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng.gen_range(0..total);
    for (id, weight) in &rule.weights {
        if roll < *weight {
            return Some(id.clone());
        }
        roll -= weight;
    }
    None
}
