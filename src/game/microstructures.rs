use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use super::{
    chest::Chest,
    item::Item,
    npc::Npc,
    loot::generate_loot,
    map::{Map, Tile},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroStructureTile {
    pub x: i32,
    pub y: i32,
    pub tile_type: String, // "wall", "floor", "glass", "special"
    pub glyph: Option<char>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroStructureSpawn {
    pub spawn_type: String, // "npc", "chest", "item"
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub chance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroStructureDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<MicroStructureTile>,
    pub spawns: Vec<MicroStructureSpawn>,
    pub biome_weights: HashMap<String, u32>,
    pub min_distance_from_player: u32,
    pub min_distance_between: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedMicroStructure {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub spawned_npcs: Vec<usize>,
    pub spawned_chests: Vec<usize>,
    pub spawned_items: Vec<usize>,
}

static MICROSTRUCTURE_DEFS: Lazy<HashMap<String, MicroStructureDef>> = Lazy::new(|| {
    let data = include_str!("../../data/microstructures.json");
    let defs: Vec<MicroStructureDef> = serde_json::from_str(data).expect("Failed to parse microstructures.json");
    defs.into_iter().map(|def| (def.id.clone(), def)).collect()
});

pub fn get_microstructure_def(id: &str) -> Option<&'static MicroStructureDef> {
    MICROSTRUCTURE_DEFS.get(id)
}

pub fn all_microstructure_ids() -> Vec<String> {
    MICROSTRUCTURE_DEFS.keys().cloned().collect()
}

pub fn get_biome_microstructures(biome: &str) -> Vec<&'static MicroStructureDef> {
    MICROSTRUCTURE_DEFS.values()
        .filter(|def| def.biome_weights.contains_key(biome))
        .collect()
}

pub fn place_microstructures(
    map: &mut Map,
    biome: &str,
    clearings: &[(i32, i32)],
    player_pos: (i32, i32),
    rng: &mut ChaCha8Rng,
) -> (Vec<PlacedMicroStructure>, Vec<Npc>, Vec<Chest>, Vec<Item>) {
    let mut placed_structures = Vec::new();
    let mut npcs = Vec::new();
    let mut chests = Vec::new();
    let mut items = Vec::new();
    
    let available_structures = get_biome_microstructures(biome);
    if available_structures.is_empty() {
        return (placed_structures, npcs, chests, items);
    }
    
    // Filter clearings that are far enough from player and each other
    let mut valid_positions = Vec::new();
    for &(x, y) in clearings {
        let dist_from_player = ((x - player_pos.0).pow(2) + (y - player_pos.1).pow(2)) as f32;
        if dist_from_player.sqrt() >= 20.0 { // Minimum distance from player
            valid_positions.push((x, y));
        }
    }
    
    // Place structures with spatial distribution
    let max_structures = (valid_positions.len() / 8).max(1).min(4); // 1-4 structures per tile
    let mut placed_positions = Vec::new();
    
    for _ in 0..max_structures {
        if valid_positions.is_empty() {
            break;
        }
        
        // Select random structure weighted by biome
        let structure = select_weighted_structure(&available_structures, biome, rng);
        if structure.is_none() {
            continue;
        }
        let structure = structure.unwrap();
        
        // Find valid placement position
        let mut attempts = 0;
        while attempts < 20 && !valid_positions.is_empty() {
            let pos_idx = rng.gen_range(0..valid_positions.len());
            let (x, y) = valid_positions[pos_idx];
            
            // Check minimum distance from other structures
            let too_close = placed_positions.iter().any(|&(px, py)| {
                let dx = (x - px) as f32;
                let dy = (y - py) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                dist < structure.min_distance_between as f32
            });
            
            if !too_close && can_place_structure(map, structure, x, y) {
                // Place the structure
                place_structure_on_map(map, structure, x, y);
                
                // Spawn entities
                let (structure_npcs, structure_chests, structure_items) = 
                    spawn_structure_entities(structure, x, y, rng);
                
                let npc_indices: Vec<usize> = (npcs.len()..npcs.len() + structure_npcs.len()).collect();
                let chest_indices: Vec<usize> = (chests.len()..chests.len() + structure_chests.len()).collect();
                let item_indices: Vec<usize> = (items.len()..items.len() + structure_items.len()).collect();
                
                npcs.extend(structure_npcs);
                chests.extend(structure_chests);
                items.extend(structure_items);
                
                placed_structures.push(PlacedMicroStructure {
                    id: structure.id.clone(),
                    x,
                    y,
                    spawned_npcs: npc_indices,
                    spawned_chests: chest_indices,
                    spawned_items: item_indices,
                });
                
                placed_positions.push((x, y));
                valid_positions.remove(pos_idx);
                break;
            }
            
            attempts += 1;
        }
    }
    
    (placed_structures, npcs, chests, items)
}

fn select_weighted_structure<'a>(
    structures: &[&'a MicroStructureDef],
    biome: &str,
    rng: &mut ChaCha8Rng,
) -> Option<&'a MicroStructureDef> {
    let total_weight: u32 = structures.iter()
        .map(|s| s.biome_weights.get(biome).unwrap_or(&0))
        .sum();
        
    if total_weight == 0 {
        return None;
    }
    
    let mut roll = rng.gen_range(0..total_weight);
    for structure in structures {
        let weight = *structure.biome_weights.get(biome).unwrap_or(&0);
        if roll < weight {
            return Some(structure);
        }
        roll -= weight;
    }
    
    None
}

fn can_place_structure(map: &Map, structure: &MicroStructureDef, x: i32, y: i32) -> bool {
    // Check if structure fits within map bounds
    if x < 0 || y < 0 || 
       x + structure.width as i32 >= map.width as i32 || 
       y + structure.height as i32 >= map.height as i32 {
        return false;
    }
    
    // Check if area is mostly clear (allow some walls to be overwritten)
    let mut floor_count = 0;
    let total_tiles = structure.width * structure.height;
    
    for dy in 0..structure.height as i32 {
        for dx in 0..structure.width as i32 {
            let map_x = x + dx;
            let map_y = y + dy;
            if let Some(tile) = map.get(map_x, map_y) {
                if tile.walkable() {
                    floor_count += 1;
                }
            }
        }
    }
    
    // Require at least 60% of the area to be walkable
    floor_count as f32 / total_tiles as f32 >= 0.6
}

fn place_structure_on_map(map: &mut Map, structure: &MicroStructureDef, x: i32, y: i32) {
    for tile_def in &structure.tiles {
        let map_x = x + tile_def.x;
        let map_y = y + tile_def.y;
        
        if map_x >= 0 && map_y >= 0 && 
           map_x < map.width as i32 && map_y < map.height as i32 {
            let idx = map.idx(map_x, map_y);
            
            let new_tile = match tile_def.tile_type.as_str() {
                "wall" => Tile::Wall { id: "stone".to_string(), hp: 100 },
                "floor" => Tile::Floor,
                "glass" => Tile::Glass,
                _ => continue, // Skip unknown tile types
            };
            
            map.tiles[idx] = new_tile;
        }
    }
}

fn spawn_structure_entities(
    structure: &MicroStructureDef,
    base_x: i32,
    base_y: i32,
    rng: &mut ChaCha8Rng,
) -> (Vec<Npc>, Vec<Chest>, Vec<Item>) {
    let mut npcs = Vec::new();
    let mut chests = Vec::new();
    let mut items = Vec::new();
    
    for spawn in &structure.spawns {
        if rng.gen_range(0.0..1.0) > spawn.chance {
            continue;
        }
        
        let spawn_x = base_x + spawn.x;
        let spawn_y = base_y + spawn.y;
        
        match spawn.spawn_type.as_str() {
            "npc" => {
                npcs.push(Npc::new(spawn_x, spawn_y, &spawn.id));
            }
            "chest" => {
                let mut chest = Chest::new(spawn_x, spawn_y, &spawn.id);
                if let Some(def) = super::chest::get_chest_def(&spawn.id) {
                    if let Some(loot_table) = &def.loot_table {
                        let loot = generate_loot(loot_table, spawn_x, spawn_y, rng);
                        for item in loot {
                            chest.add_item(item);
                        }
                    }
                }
                chests.push(chest);
            }
            "item" => {
                items.push(Item::new(spawn_x, spawn_y, &spawn.id));
            }
            _ => {} // Skip unknown spawn types
        }
    }
    
    (npcs, chests, items)
}
