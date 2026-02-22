use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::HashMap;

// ============================================================================
// Data Structures (for JSON loading)
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct PrefabLibraryData {
    pub schema: String,
    pub prefabs: Vec<PrefabData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrefabData {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<String>,
    pub legend: HashMap<String, LegendEntry>,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub faction: Option<String>,
    #[serde(default)]
    pub metadata: PrefabMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegendEntry {
    pub tile: String,
    #[serde(default)]
    pub marker: Option<String>,
    #[serde(default)]
    pub mask: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PrefabMetadata {
    #[serde(default = "default_entrance_side")]
    pub entrance_side: String,
    #[serde(default)]
    pub has_interior: bool,
    #[serde(default)]
    pub npc_count: usize,
    #[serde(default)]
    pub npc_types: Vec<String>,
    #[serde(default)]
    pub description: String,
}

fn default_weight() -> f32 {
    1.0
}

fn default_entrance_side() -> String {
    "any".to_string()
}

// ============================================================================
// Runtime Structures
// ============================================================================

#[derive(Debug, Clone)]
pub struct PrefabCell {
    pub tile: String,
    pub marker: Option<String>,
    pub mask: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Prefab {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<PrefabCell>,
    pub weight: f32,
    pub tags: Vec<String>,
    pub faction: Option<String>,
    pub metadata: PrefabMetadata,
}

impl Prefab {
    /// Parse a prefab from data
    fn from_data(data: PrefabData) -> Result<Self, String> {
        // Validate dimensions
        if data.pattern.len() != data.height {
            return Err(format!(
                "Prefab '{}': pattern has {} rows but height is {}",
                data.name,
                data.pattern.len(),
                data.height
            ));
        }

        for (i, row) in data.pattern.iter().enumerate() {
            if row.len() != data.width {
                return Err(format!(
                    "Prefab '{}': row {} has {} chars but width is {}",
                    data.name,
                    i,
                    row.len(),
                    data.width
                ));
            }
        }

        // Parse pattern into cells
        let mut cells = Vec::with_capacity(data.width * data.height);
        for row in &data.pattern {
            for ch in row.chars() {
                let key = ch.to_string();
                let entry = data.legend.get(&key).ok_or_else(|| {
                    format!(
                        "Prefab '{}': character '{}' not found in legend",
                        data.name, ch
                    )
                })?;

                cells.push(PrefabCell {
                    tile: entry.tile.clone(),
                    marker: entry.marker.clone(),
                    mask: entry.mask.clone(),
                });
            }
        }

        Ok(Prefab {
            name: data.name,
            width: data.width,
            height: data.height,
            cells,
            weight: data.weight,
            tags: data.tags,
            faction: data.faction,
            metadata: data.metadata,
        })
    }

    /// Get cell at (x, y)
    pub fn get(&self, x: usize, y: usize) -> Option<&PrefabCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.cells.get(y * self.width + x)
    }

    /// Rotate 90 degrees clockwise
    pub fn rotated(&self) -> Self {
        let mut rotated_cells = vec![
            PrefabCell {
                tile: String::new(),
                marker: None,
                mask: None
            };
            self.width * self.height
        ];

        for y in 0..self.height {
            for x in 0..self.width {
                let old_idx = y * self.width + x;
                let new_x = self.height - 1 - y;
                let new_y = x;
                let new_idx = new_y * self.height + new_x;
                rotated_cells[new_idx] = self.cells[old_idx].clone();
            }
        }

        Prefab {
            name: format!("{}_rotated", self.name),
            width: self.height,
            height: self.width,
            cells: rotated_cells,
            weight: self.weight,
            tags: self.tags.clone(),
            faction: self.faction.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Mirror horizontally
    pub fn mirrored_horizontal(&self) -> Self {
        let mut mirrored_cells = vec![
            PrefabCell {
                tile: String::new(),
                marker: None,
                mask: None
            };
            self.width * self.height
        ];

        for y in 0..self.height {
            for x in 0..self.width {
                let old_idx = y * self.width + x;
                let new_x = self.width - 1 - x;
                let new_idx = y * self.width + new_x;
                mirrored_cells[new_idx] = self.cells[old_idx].clone();
            }
        }

        Prefab {
            name: format!("{}_mirrored_h", self.name),
            width: self.width,
            height: self.height,
            cells: mirrored_cells,
            weight: self.weight,
            tags: self.tags.clone(),
            faction: self.faction.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Mirror vertically
    pub fn mirrored_vertical(&self) -> Self {
        let mut mirrored_cells = vec![
            PrefabCell {
                tile: String::new(),
                marker: None,
                mask: None
            };
            self.width * self.height
        ];

        for y in 0..self.height {
            for x in 0..self.width {
                let old_idx = y * self.width + x;
                let new_y = self.height - 1 - y;
                let new_idx = new_y * self.width + x;
                mirrored_cells[new_idx] = self.cells[old_idx].clone();
            }
        }

        Prefab {
            name: format!("{}_mirrored_v", self.name),
            width: self.width,
            height: self.height,
            cells: mirrored_cells,
            weight: self.weight,
            tags: self.tags.clone(),
            faction: self.faction.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

// ============================================================================
// Prefab Library
// ============================================================================

pub struct PrefabLibrary {
    prefabs: Vec<Prefab>,
    by_tag: HashMap<String, Vec<usize>>,
    by_faction: HashMap<String, Vec<usize>>,
}

impl PrefabLibrary {
    /// Load prefab library from data
    fn load() -> Result<Self, String> {
        let data = include_str!("../../../../data/prefabs/core.json");
        let library_data: PrefabLibraryData = serde_json::from_str(data)
            .map_err(|e| format!("Failed to parse prefabs/core.json: {}", e))?;

        // Validate schema
        if library_data.schema != "prefabs_v1" {
            return Err(format!(
                "Invalid schema version: expected 'prefabs_v1', got '{}'",
                library_data.schema
            ));
        }

        // Parse all prefabs
        let mut prefabs = Vec::new();
        for data in library_data.prefabs {
            let prefab = Prefab::from_data(data)?;
            prefabs.push(prefab);
        }

        // Build indices
        let mut by_tag: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_faction: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, prefab) in prefabs.iter().enumerate() {
            // Index by tags
            for tag in &prefab.tags {
                by_tag.entry(tag.clone()).or_default().push(idx);
            }

            // Index by faction
            if let Some(faction) = &prefab.faction {
                by_faction.entry(faction.clone()).or_default().push(idx);
            }
        }

        Ok(Self {
            prefabs,
            by_tag,
            by_faction,
        })
    }

    /// Get prefabs by tag
    pub fn get_by_tags(&self, tags: &[String]) -> Vec<&Prefab> {
        let mut result = Vec::new();
        for tag in tags {
            if let Some(indices) = self.by_tag.get(tag) {
                for &idx in indices {
                    if let Some(prefab) = self.prefabs.get(idx) {
                        result.push(prefab);
                    }
                }
            }
        }
        result
    }

    /// Get prefabs by faction
    pub fn get_by_faction(&self, faction: &str) -> Vec<&Prefab> {
        self.by_faction
            .get(faction)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.prefabs.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Select a random prefab using weighted selection
    pub fn select_weighted<'a, R: rand::Rng>(
        &'a self,
        candidates: &[&'a Prefab],
        rng: &mut R,
    ) -> Option<&'a Prefab> {
        if candidates.is_empty() {
            return None;
        }

        candidates.choose_weighted(rng, |p| p.weight).ok().copied()
    }
}

// ============================================================================
// Global Instance
// ============================================================================

static PREFAB_LIBRARY: Lazy<PrefabLibrary> = Lazy::new(|| {
    PrefabLibrary::load().expect("Failed to load prefab library")
});

pub fn get_prefab_library() -> &'static PrefabLibrary {
    &PREFAB_LIBRARY
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_prefab() -> Prefab {
        let data = PrefabData {
            name: "test".to_string(),
            width: 3,
            height: 3,
            pattern: vec!["###".to_string(), "#.#".to_string(), "###".to_string()],
            legend: {
                let mut map = HashMap::new();
                map.insert(
                    "#".to_string(),
                    LegendEntry {
                        tile: "wall".to_string(),
                        marker: None,
                        mask: None,
                    },
                );
                map.insert(
                    ".".to_string(),
                    LegendEntry {
                        tile: "floor".to_string(),
                        marker: None,
                        mask: None,
                    },
                );
                map
            },
            weight: 1.0,
            tags: vec!["test".to_string()],
            faction: None,
            metadata: PrefabMetadata::default(),
        };

        Prefab::from_data(data).unwrap()
    }

    #[test]
    fn test_prefab_from_data() {
        let prefab = create_test_prefab();
        assert_eq!(prefab.width, 3);
        assert_eq!(prefab.height, 3);
        assert_eq!(prefab.cells.len(), 9);
    }

    #[test]
    fn test_prefab_get() {
        let prefab = create_test_prefab();
        assert_eq!(prefab.get(1, 1).unwrap().tile, "floor");
        assert_eq!(prefab.get(0, 0).unwrap().tile, "wall");
        assert!(prefab.get(3, 3).is_none());
    }

    #[test]
    fn test_prefab_rotation() {
        let prefab = create_test_prefab();
        let rotated = prefab.rotated();
        
        assert_eq!(rotated.width, 3);
        assert_eq!(rotated.height, 3);
        assert_eq!(rotated.get(1, 1).unwrap().tile, "floor");
    }

    #[test]
    fn test_prefab_mirror_horizontal() {
        let prefab = create_test_prefab();
        let mirrored = prefab.mirrored_horizontal();
        
        assert_eq!(mirrored.width, 3);
        assert_eq!(mirrored.height, 3);
        assert_eq!(mirrored.get(1, 1).unwrap().tile, "floor");
    }

    #[test]
    fn test_prefab_mirror_vertical() {
        let prefab = create_test_prefab();
        let mirrored = prefab.mirrored_vertical();
        
        assert_eq!(mirrored.width, 3);
        assert_eq!(mirrored.height, 3);
        assert_eq!(mirrored.get(1, 1).unwrap().tile, "floor");
    }

    #[test]
    fn test_load_prefab_library() {
        let library = get_prefab_library();
        
        // Should have 14 core prefabs
        assert!(library.prefabs.len() >= 14, "Expected at least 14 prefabs, got {}", library.prefabs.len());
        
        // Check for specific prefabs
        let town_halls: Vec<_> = library.prefabs.iter()
            .filter(|p| p.name.starts_with("town_hall"))
            .collect();
        assert_eq!(town_halls.len(), 2, "Expected 2 town hall variants");
        
        // Check tag indexing
        let core_buildings = library.get_by_tags(&[String::from("core")]);
        assert!(!core_buildings.is_empty(), "Should have core-tagged buildings");
        
        // Check government buildings
        let government = library.get_by_tags(&[String::from("government")]);
        assert_eq!(government.len(), 2, "Expected 2 government buildings");
    }
}
