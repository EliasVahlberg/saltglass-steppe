use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct WallDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub color: String,
    pub hp: i32,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FloorDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub color: String,
    pub description: String,
}

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
struct MapElementsFile {
    #[allow(dead_code)]
    schema: Option<String>,
    tiles: TilesSection,
    lights: LightsSection,
}

#[derive(Deserialize)]
struct TilesSection {
    walls: Vec<WallDef>,
    floors: Vec<FloorDef>,
}

#[derive(Deserialize)]
struct LightsSection {
    defs: Vec<LightDef>,
    spawn_rules: HashMap<String, SpawnRule>,
}

struct MapElementsData {
    walls: HashMap<String, WallDef>,
    floors: HashMap<String, FloorDef>,
    lights: HashMap<String, LightDef>,
    spawn_rules: HashMap<String, SpawnRule>,
}

static MAP_ELEMENTS: Lazy<MapElementsData> = Lazy::new(|| {
    let data = fs::read_to_string("data/map_elements.json")
        .expect("Failed to read data/map_elements.json");
    let root: Value = serde_json::from_str(&data).expect("Failed to parse data/map_elements.json");
    validate_map_elements_schema("data/map_elements.json", &root);
    let file: MapElementsFile =
        serde_json::from_value(root).expect("Failed to parse data/map_elements.json");
    MapElementsData {
        walls: file
            .tiles
            .walls
            .into_iter()
            .map(|d| (d.id.clone(), d))
            .collect(),
        floors: file
            .tiles
            .floors
            .into_iter()
            .map(|d| (d.id.clone(), d))
            .collect(),
        lights: file
            .lights
            .defs
            .into_iter()
            .map(|d| (d.id.clone(), d))
            .collect(),
        spawn_rules: file.lights.spawn_rules,
    }
});

fn validate_map_elements_schema(label: &str, root: &Value) {
    let schema_json = include_str!("../../schemas/map_elements_v1.json");
    let schema_value: Value =
        serde_json::from_str(schema_json).expect("Failed to parse map_elements_v1.json schema");

    let compiled = JSONSchema::compile(&schema_value)
        .unwrap_or_else(|e| panic!("Failed to compile map_elements_v1 schema: {e}"));

    if let Err(errors) = compiled.validate(root) {
        let mut messages = Vec::new();
        for error in errors.take(5) {
            messages.push(error.to_string());
        }
        panic!(
            "Schema validation failed for {} (map_elements_v1): {}",
            label,
            messages.join("; ")
        );
    }
}

pub fn get_wall_def(id: &str) -> Option<&'static WallDef> {
    MAP_ELEMENTS.walls.get(id)
}

pub fn get_floor_def(id: &str) -> Option<&'static FloorDef> {
    MAP_ELEMENTS.floors.get(id)
}

pub fn get_light_def(id: &str) -> Option<&'static LightDef> {
    MAP_ELEMENTS.lights.get(id)
}

pub fn get_spawn_rule(biome: &str) -> &'static SpawnRule {
    MAP_ELEMENTS
        .spawn_rules
        .get(biome)
        .or_else(|| MAP_ELEMENTS.spawn_rules.get("default"))
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
