//! Structure template system for placing prebuilt structures

use once_cell::sync::Lazy;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;

use super::map::{Map, Tile, get_floor_def, get_wall_def};
use super::npc::Npc;

#[derive(Debug, Clone, Deserialize)]
pub struct StructureTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub symbol_dict: HashMap<String, SymbolDef>,
    pub clear_area: Option<ClearArea>,
    pub template_rows: Vec<String>,
    pub lore: LoreInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SymbolDef {
    #[serde(rename = "type")]
    pub symbol_type: String,
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClearArea {
    pub shape: String,
    pub radius: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub center_x: usize,
    pub center_y: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoreInfo {
    pub discovery_message: String,
    pub atmosphere: String,
}

#[derive(Deserialize)]
struct StructureTemplatesFile {
    structures: Vec<StructureTemplate>,
}

static STRUCTURE_TEMPLATES: Lazy<HashMap<String, StructureTemplate>> = Lazy::new(|| {
    let data = include_str!("../../data/structure_templates.json");
    let file: StructureTemplatesFile =
        serde_json::from_str(data).expect("Failed to parse structure_templates.json");
    file.structures
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect()
});

pub fn get_structure_template(id: &str) -> Option<&'static StructureTemplate> {
    STRUCTURE_TEMPLATES.get(id)
}

pub fn all_structure_template_ids() -> Vec<&'static str> {
    STRUCTURE_TEMPLATES.keys().map(|s| s.as_str()).collect()
}

/// Apply a structure template to a map at the given position
pub fn apply_structure_template(
    map: &mut Map,
    template_id: &str,
    center_x: usize,
    center_y: usize,
    _rng: &mut ChaCha8Rng,
) -> Result<Vec<Npc>, String> {
    let template = get_structure_template(template_id)
        .ok_or_else(|| format!("Structure template '{}' not found", template_id))?;

    let mut spawned_npcs = Vec::new();

    // Clear area if specified
    if let Some(clear_area) = &template.clear_area {
        apply_clear_area(map, clear_area, center_x, center_y);
    }

    // Calculate template placement offset
    let template_height = template.template_rows.len();
    let template_width = template
        .template_rows
        .get(0)
        .map(|row| row.len())
        .unwrap_or(0);

    let start_x = center_x.saturating_sub(template_width / 2);
    let start_y = center_y.saturating_sub(template_height / 2);

    // Apply template
    for (row_idx, row) in template.template_rows.iter().enumerate() {
        let y = start_y + row_idx;
        if y >= map.height {
            break;
        }

        for (col_idx, symbol_char) in row.chars().enumerate() {
            let x = start_x + col_idx;
            if x >= map.width {
                break;
            }

            let symbol_str = symbol_char.to_string();
            if let Some(symbol_def) = template.symbol_dict.get(&symbol_str) {
                match symbol_def.symbol_type.as_str() {
                    "wall" => {
                        if let Some(wall_def) = get_wall_def(&symbol_def.id) {
                            let idx = y * map.width + x;
                            map.tiles[idx] = Tile::Wall {
                                id: symbol_def.id.clone(),
                                hp: wall_def.hp,
                            };
                        }
                    }
                    "floor" => {
                        if get_floor_def(&symbol_def.id).is_some() {
                            let idx = y * map.width + x;
                            map.tiles[idx] = Tile::Floor {
                                id: symbol_def.id.clone(),
                            };
                        }
                    }
                    "npc" => {
                        // Create NPC at this position
                        let npc = Npc::new(x as i32, y as i32, &symbol_def.id);
                        spawned_npcs.push(npc);

                        // Place floor under NPC
                        let idx = y * map.width + x;
                        map.tiles[idx] = Tile::Floor {
                            id: "ancient_tile".to_string(),
                        };
                    }
                    _ => {
                        // Unknown symbol type, ignore
                    }
                }
            }
        }
    }

    Ok(spawned_npcs)
}

fn apply_clear_area(map: &mut Map, clear_area: &ClearArea, center_x: usize, center_y: usize) {
    match clear_area.shape.as_str() {
        "circle" => {
            if let Some(radius) = clear_area.radius {
                let clear_center_x = center_x + clear_area.center_x;
                let clear_center_y = center_y + clear_area.center_y;

                for y in 0..map.height {
                    for x in 0..map.width {
                        let dx = x as i32 - clear_center_x as i32;
                        let dy = y as i32 - clear_center_y as i32;
                        let distance_sq = dx * dx + dy * dy;

                        if distance_sq <= (radius * radius) as i32 {
                            let idx = y * map.width + x;
                            map.tiles[idx] = Tile::Floor {
                                id: "dry_soil".to_string(),
                            };
                        }
                    }
                }
            }
        }
        "rectangle" => {
            if let (Some(width), Some(height)) = (clear_area.width, clear_area.height) {
                let clear_center_x = center_x + clear_area.center_x;
                let clear_center_y = center_y + clear_area.center_y;

                let start_x = clear_center_x.saturating_sub(width / 2);
                let start_y = clear_center_y.saturating_sub(height / 2);
                let end_x = (start_x + width).min(map.width);
                let end_y = (start_y + height).min(map.height);

                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let idx = y * map.width + x;
                        map.tiles[idx] = Tile::Floor {
                            id: "dry_soil".to_string(),
                        };
                    }
                }
            }
        }
        _ => {
            // Unknown shape, ignore
        }
    }
}

/// Get a random structure template suitable for a POI type
pub fn get_random_structure_for_poi(poi_type: &str, rng: &mut ChaCha8Rng) -> Option<&'static str> {
    let suitable_templates = match poi_type {
        "town" => vec!["mesa_village"],
        "shrine" => vec!["salt_shrine"],
        "ruins" | "archive" => vec!["ruined_archive"],
        _ => vec!["mesa_village", "salt_shrine"], // Default options
    };

    if suitable_templates.is_empty() {
        None
    } else {
        let idx = rng.gen_range(0..suitable_templates.len());
        Some(suitable_templates[idx])
    }
}
