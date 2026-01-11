pub mod algorithms;
pub mod dungeon_generator;
pub mod ruins_generator;

pub use algorithms::*;
pub use dungeon_generator::*;
use rand_chacha::ChaCha8Rng;
pub use ruins_generator::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureType {
    Dungeon,
    Town,
    Shrine,
    Ruins,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum POIType {
    Dungeon,
    Town,
    Shrine,
    Landmark,
}

#[derive(Debug, Clone)]
pub struct StructureParams {
    pub structure_type: StructureType,
    pub size: (u32, u32),
    pub theme: String,
    pub quest_requirements: Vec<String>,
    pub biome_context: String,
    pub organic_walls: bool,
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub structure_type: StructureType,
    pub bounds: Rectangle,
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub features: Vec<StructureFeature>,
    pub spawn_points: Vec<SpawnPoint>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub bounds: Rectangle,
    pub room_type: String,
    pub depth_from_entrance: u32,
}

#[derive(Debug, Clone)]
pub struct Corridor {
    pub start: (u32, u32),
    pub end: (u32, u32),
    pub width: u32,
}

#[derive(Debug, Clone)]
pub struct StructureFeature {
    pub position: (u32, u32),
    pub feature_type: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SpawnPoint {
    pub position: (u32, u32),
    pub spawn_type: String, // "enemy", "item", "npc"
    pub entity_id: String,
}

pub trait StructureGenerator {
    fn generate(&self, params: &StructureParams, rng: &mut ChaCha8Rng) -> Option<Structure>;
    fn get_supported_poi_types(&self) -> Vec<POIType>;
}

impl POIType {
    pub fn to_structure_type(&self) -> StructureType {
        match self {
            POIType::Dungeon => StructureType::Dungeon,
            POIType::Town => StructureType::Town,
            POIType::Shrine => StructureType::Shrine,
            POIType::Landmark => StructureType::Ruins,
        }
    }
}

impl Rectangle {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

impl StructureParams {
    pub fn new(structure_type: StructureType, size: (u32, u32)) -> Self {
        Self {
            structure_type,
            size,
            theme: "default".to_string(),
            quest_requirements: Vec::new(),
            biome_context: "default".to_string(),
            organic_walls: false,
        }
    }
}
