use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    chest::Chest,
    enemy::Enemy,
    interactable::Interactable,
    item::Item,
    lighting::LightMap,
    map::Map,
    npc::Npc,
    storm::Storm,
    visual_effects::VisualEffects,
    world_map::WorldMap,
};

/// Weather conditions affecting visibility and lighting
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Weather {
    #[default]
    Clear,
    Dusty,
    Sandstorm,
}

impl Weather {
    /// Get ambient light modifier for this weather condition
    pub fn ambient_modifier(&self) -> i32 {
        match self {
            Weather::Clear => 0,
            Weather::Dusty => -20,
            Weather::Sandstorm => -50,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WorldState {
    // World navigation
    pub world_map: Option<WorldMap>,
    pub world_x: usize,
    pub world_y: usize,
    pub layer: i32,

    // Current tile
    pub map: Map,
    pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub chests: Vec<Chest>,
    pub interactables: Vec<Interactable>,
    pub microstructures: Vec<super::generation::PlacedMicroStructure>,

    // Environmental
    pub storm: Storm,
    pub time_of_day: u8,
    pub weather: Weather,
    pub ambient_light: u8,

    // Visual effects
    pub visual_effects: VisualEffects,
    #[serde(skip)]
    pub light_map: LightMap,

    // Spatial indexing (computed on load)
    #[serde(skip)]
    pub enemy_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub npc_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub item_positions: HashMap<(i32, i32), Vec<usize>>,
    #[serde(skip)]
    pub chest_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub interactable_positions: HashMap<(i32, i32), usize>,
}

impl WorldState {
    pub fn ensure_spatial_index(&mut self) {
        self.enemy_positions.clear();
        self.npc_positions.clear();
        self.item_positions.clear();
        self.chest_positions.clear();
        self.interactable_positions.clear();

        for (i, enemy) in self.enemies.iter().enumerate() {
            self.enemy_positions.insert((enemy.x, enemy.y), i);
        }

        for (i, npc) in self.npcs.iter().enumerate() {
            self.npc_positions.insert((npc.x, npc.y), i);
        }

        for (i, item) in self.items.iter().enumerate() {
            self.item_positions
                .entry((item.x, item.y))
                .or_insert_with(Vec::new)
                .push(i);
        }

        for (i, chest) in self.chests.iter().enumerate() {
            self.chest_positions.insert((chest.x, chest.y), i);
        }

        for (i, interactable) in self.interactables.iter().enumerate() {
            self.interactable_positions.insert((interactable.x, interactable.y), i);
        }
    }
}