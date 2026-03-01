use super::{Building, SettlementConfig};
use crate::game::generation::structure_library::{StructureLibrary, StructureUsage};
use rand::Rng;

/// Place buildings in the settlement using StructureLibrary
pub fn place_buildings<R: Rng>(
    config: &SettlementConfig,
    width: usize,
    height: usize,
    rng: &mut R,
) -> Vec<Building> {
    let library = match StructureLibrary::load() {
        Ok(lib) => lib,
        Err(_) => return Vec::new(),
    };

    let dominant_faction = super::faction_theme::get_dominant_faction(config);

    // Prefer faction-specific buildings, fall back to all connectable
    let candidates: Vec<_> = match &dominant_faction {
        Some(faction) => {
            let faction_buildings = library.by_faction(faction);
            if faction_buildings.is_empty() {
                library.by_usage(StructureUsage::Connectable)
            } else {
                faction_buildings
            }
        }
        None => library.by_usage(StructureUsage::Connectable),
    };

    if candidates.is_empty() {
        return Vec::new();
    }

    let margin = 2usize;
    let spacing = 2usize;
    let mut buildings = Vec::new();
    let mut cursor_x = margin;
    let mut cursor_y = margin;
    let mut row_height = 0usize;

    while cursor_y + margin < height {
        // Pick a random candidate weighted by metadata weight
        let total_weight: f32 = candidates.iter().map(|s| s.metadata.weight).sum();
        let mut roll = rng.gen_range(0.0f32..total_weight.max(1.0));
        let structure = candidates
            .iter()
            .find(|s| {
                if roll < s.metadata.weight {
                    true
                } else {
                    roll -= s.metadata.weight;
                    false
                }
            })
            .unwrap_or(&candidates[0]);

        // Wrap to next row if it doesn't fit
        if cursor_x + structure.width + margin > width {
            cursor_x = margin;
            cursor_y += row_height + spacing;
            row_height = 0;
            if cursor_y + structure.height + margin > height {
                break;
            }
        }

        buildings.push(Building {
            prefab_name: structure.id.clone(),
            x: cursor_x as i32,
            y: cursor_y as i32,
            faction: dominant_faction.clone(),
        });

        cursor_x += structure.width + spacing;
        row_height = row_height.max(structure.height);

        // Stop after a reasonable number of buildings per tier
        let max_buildings = match config.tier {
            super::SettlementTier::Village => 6,
            super::SettlementTier::Town => 12,
            super::SettlementTier::City => 20,
        };
        if buildings.len() >= max_buildings {
            break;
        }
    }

    buildings
}
