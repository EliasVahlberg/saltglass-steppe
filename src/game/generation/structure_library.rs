use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ============================================================================
// Core Types
// ============================================================================

/// Single structure type for all placeable structures
#[derive(Debug, Clone)]
pub struct Structure {
    pub id: String,
    pub usage: StructureUsage,
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<Vec<char>>,
    pub legend: HashMap<char, LegendEntry>,
    pub metadata: StructureMetadata,
}

/// Usage determines how structure is placed in game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureUsage {
    Standalone,   // Complete POI (ruins, shrines, landmarks)
    Connectable,  // Settlement building (can attach roads/paths)
}

/// Legend entry types for pattern characters
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LegendEntry {
    Wall { id: String },
    Floor { id: String },
    Door,
    Interactable { id: String },
    Npc { id: String, name: Option<String> },
    Structure { id: String },
    /// Leave the underlying terrain tile unchanged (outdoor/open area within bounding box)
    Ground,
    /// Stamp a path/road tile (replaced with settlement road material during city generation)
    Path,
}

/// Structure metadata
#[derive(Debug, Clone, Default, Deserialize)]
pub struct StructureMetadata {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub faction: Option<String>,
    #[serde(default)]
    pub npc_count: usize,
    #[serde(default)]
    pub npc_types: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub clear_area: Option<ClearArea>,
    #[serde(default)]
    pub entrance_side: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClearArea {
    pub shape: String,
    #[serde(default)]
    pub radius: Option<usize>,
    #[serde(default)]
    pub width: Option<usize>,
    #[serde(default)]
    pub height: Option<usize>,
    pub center_x: usize,
    pub center_y: usize,
}

fn default_weight() -> f32 {
    1.0
}

// ============================================================================
// JSON Deserialization Types
// ============================================================================

#[derive(Deserialize)]
struct StructureFile {
    structures: Vec<StructureData>,
}

#[derive(Deserialize)]
struct StructureData {
    id: String,
    usage: StructureUsage,
    #[serde(default)]
    pattern_file: Option<String>,
    #[serde(default)]
    pattern: Option<Vec<String>>,
    legend: HashMap<char, LegendEntry>,
    #[serde(default)]
    metadata: StructureMetadata,
}

// ============================================================================
// Structure Implementation
// ============================================================================

impl Structure {
    /// Load structure from data, supporting hybrid pattern loading
    fn from_data(data: StructureData, base_path: &Path) -> Result<Self, String> {
        // Hybrid loading: try file first, fall back to inline
        let pattern = if let Some(file) = data.pattern_file {
            load_pattern_from_file(&base_path.join(&file))?
        } else if let Some(inline) = data.pattern {
            parse_inline_pattern(inline)?
        } else {
            return Err(format!(
                "Structure '{}' must specify pattern_file or pattern",
                data.id
            ));
        };

        // Validate/normalize pattern: pad shorter rows to match widest row
        let height = pattern.len();
        if height == 0 {
            return Err(format!("Structure '{}' has empty pattern", data.id));
        }
        let width = pattern.iter().map(|r| r.len()).max().unwrap_or(0);
        let pattern: Vec<Vec<char>> = pattern
            .into_iter()
            .map(|mut row| {
                while row.len() < width {
                    row.push(' ');
                }
                row
            })
            .collect();

        // Validate all pattern characters are in legend (except space)
        for (y, row) in pattern.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                if ch != ' ' && !data.legend.contains_key(&ch) {
                    return Err(format!(
                        "Structure '{}' pattern character '{}' at ({}, {}) not in legend",
                        data.id, ch, x, y
                    ));
                }
            }
        }

        Ok(Structure {
            id: data.id,
            usage: data.usage,
            width,
            height,
            pattern,
            legend: data.legend,
            metadata: data.metadata,
        })
    }
}

/// Load pattern from external .txt file
fn load_pattern_from_file(path: &Path) -> Result<Vec<Vec<char>>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read pattern file {:?}: {}", path, e))?;

    let pattern: Vec<Vec<char>> = content.lines().map(|line| line.chars().collect()).collect();

    if pattern.is_empty() {
        return Err(format!("Pattern file {:?} is empty", path));
    }

    Ok(pattern)
}

/// Parse inline pattern from JSON array
fn parse_inline_pattern(lines: Vec<String>) -> Result<Vec<Vec<char>>, String> {
    if lines.is_empty() {
        return Err("Inline pattern is empty".to_string());
    }

    Ok(lines.into_iter().map(|line| line.chars().collect()).collect())
}

// ============================================================================
// Structure Library
// ============================================================================

/// Structure library - single source for all structures
pub struct StructureLibrary {
    structures: HashMap<String, Structure>,
}

impl StructureLibrary {
    /// Load structures from a JSON string
    pub fn from_json(json: &str, base_path: &Path) -> Result<Self, String> {
        let data: StructureFile =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut structures = HashMap::new();
        for s in data.structures {
            let id = s.id.clone();
            let structure = Structure::from_data(s, base_path)?;
            structures.insert(id, structure);
        }

        Ok(StructureLibrary { structures })
    }

    /// Load all structures from default location (`data/structures/structures.json`)
    pub fn load() -> Result<Self, String> {
        let json = fs::read_to_string("data/structures/structures.json")
            .map_err(|e| format!("Failed to read structures.json: {}", e))?;
        Self::from_json(&json, Path::new("data/structures"))
    }

    /// Get structure by ID
    pub fn get(&self, id: &str) -> Option<&Structure> {
        self.structures.get(id)
    }

    /// Get all structures with specific usage
    pub fn by_usage(&self, usage: StructureUsage) -> Vec<&Structure> {
        self.structures.values().filter(|s| s.usage == usage).collect()
    }

    /// Get all structures for a faction
    pub fn by_faction(&self, faction: &str) -> Vec<&Structure> {
        self.structures
            .values()
            .filter(|s| s.metadata.faction.as_deref() == Some(faction))
            .collect()
    }

    /// Get all structures with a specific tag
    pub fn by_tag(&self, tag: &str) -> Vec<&Structure> {
        self.structures
            .values()
            .filter(|s| s.metadata.tags.contains(&tag.to_string()))
            .collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_structures() {
        let library = StructureLibrary::load().expect("Failed to load structure library");
        assert_eq!(library.structures.len(), 50, "Expected 50 structures");
        assert!(!library.by_usage(StructureUsage::Connectable).is_empty());
        assert!(!library.by_usage(StructureUsage::Standalone).is_empty());
    }

    #[test]
    fn test_parse_inline_pattern() {
        let lines = vec!["###".to_string(), "#.#".to_string(), "###".to_string()];
        let pattern = parse_inline_pattern(lines).unwrap();
        assert_eq!(pattern.len(), 3);
        assert_eq!(pattern[0], vec!['#', '#', '#']);
        assert_eq!(pattern[1], vec!['#', '.', '#']);
        assert_eq!(pattern[2], vec!['#', '#', '#']);
    }

    #[test]
    fn test_parse_inline_pattern_empty() {
        let lines = vec![];
        let result = parse_inline_pattern(lines);
        assert!(result.is_err());
    }

    #[test]
    fn test_structure_validation_rectangular() {
        // Non-rectangular patterns are padded with spaces, not rejected
        let data = StructureData {
            id: "test".to_string(),
            usage: StructureUsage::Standalone,
            pattern_file: None,
            pattern: Some(vec!["###".to_string(), "##".to_string()]),
            legend: [('#', LegendEntry::Wall { id: "stone".to_string() })]
                .iter()
                .cloned()
                .collect(),
            metadata: StructureMetadata::default(),
        };

        let result = Structure::from_data(data, Path::new("."));
        assert!(result.is_ok());
        let s = result.unwrap();
        assert_eq!(s.width, 3);
        assert_eq!(s.pattern[1][2], ' '); // padded with space
    }

    #[test]
    fn test_structure_validation_legend() {
        let data = StructureData {
            id: "test".to_string(),
            usage: StructureUsage::Standalone,
            pattern_file: None,
            pattern: Some(vec!["###".to_string(), "#.#".to_string(), "###".to_string()]),
            legend: [('#', LegendEntry::Wall { id: "stone".to_string() })]
                .iter()
                .cloned()
                .collect(),
            metadata: StructureMetadata::default(),
        };

        let result = Structure::from_data(data, Path::new("."));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in legend"));
    }

    #[test]
    fn test_structure_valid() {
        let data = StructureData {
            id: "test".to_string(),
            usage: StructureUsage::Standalone,
            pattern_file: None,
            pattern: Some(vec!["###".to_string(), "#.#".to_string(), "###".to_string()]),
            legend: [
                ('#', LegendEntry::Wall { id: "stone".to_string() }),
                ('.', LegendEntry::Floor { id: "dirt".to_string() }),
            ]
            .iter()
            .cloned()
            .collect(),
            metadata: StructureMetadata::default(),
        };

        let result = Structure::from_data(data, Path::new("."));
        assert!(result.is_ok());
        let structure = result.unwrap();
        assert_eq!(structure.width, 3);
        assert_eq!(structure.height, 3);
    }

    #[test]
    fn test_library_load() {
        let json = "{\"structures\":[{\"id\":\"test_house\",\"usage\":\"connectable\",\"pattern\":[\"####\",\"#..#\",\"#..D\",\"####\"],\"legend\":{\"#\":{\"type\":\"wall\",\"id\":\"wood_wall\"},\".\":{\"type\":\"floor\",\"id\":\"wood_floor\"},\"D\":{\"type\":\"door\"}},\"metadata\":{\"tags\":[\"test\",\"residential\"]}}]}";
        
        let library = StructureLibrary::from_json(json, Path::new(".")).expect("Failed to load");
        
        assert!(library.get("test_house").is_some());
        let house = library.get("test_house").unwrap();
        assert_eq!(house.width, 4);
        assert_eq!(house.height, 4);
        assert_eq!(house.usage, StructureUsage::Connectable);
        assert!(house.metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_library_filtering() {
        let json = "{\"structures\":[{\"id\":\"house\",\"usage\":\"connectable\",\"pattern\":[\"##\",\"##\"],\"legend\":{\"#\":{\"type\":\"wall\",\"id\":\"wood\"}},\"metadata\":{\"tags\":[\"residential\"]}},{\"id\":\"ruins\",\"usage\":\"standalone\",\"pattern\":[\"##\",\"##\"],\"legend\":{\"#\":{\"type\":\"wall\",\"id\":\"stone\"}},\"metadata\":{\"tags\":[\"poi\"]}}]}";
        
        let library = StructureLibrary::from_json(json, Path::new(".")).expect("Failed to load");
        
        let connectable = library.by_usage(StructureUsage::Connectable);
        assert_eq!(connectable.len(), 1);
        
        let standalone = library.by_usage(StructureUsage::Standalone);
        assert_eq!(standalone.len(), 1);
        
        let residential = library.by_tag("residential");
        assert_eq!(residential.len(), 1);
    }

    #[test]
    fn test_pattern_file_loading() {
        use std::env;
        use std::fs;
        
        let temp_dir = env::temp_dir();
        let pattern_path = temp_dir.join("test_pattern.txt");
        fs::write(&pattern_path, "###\n#.#\n###").expect("Failed to write test pattern");
        
        let json = format!(
            "{{\"structures\":[{{\"id\":\"test_from_file\",\"usage\":\"standalone\",\"pattern_file\":\"{}\",\"legend\":{{\"#\":{{\"type\":\"wall\",\"id\":\"stone\"}},\".\":{{\"type\":\"floor\",\"id\":\"dirt\"}}}},\"metadata\":{{}}}}]}}",
            pattern_path.file_name().unwrap().to_str().unwrap()
        );
        
        let library = StructureLibrary::from_json(&json, &temp_dir).expect("Failed to load");
        let structure = library.get("test_from_file").expect("Structure not found");
        
        assert_eq!(structure.width, 3);
        assert_eq!(structure.height, 3);
        
        fs::remove_file(pattern_path).ok();
    }

    #[test]
    fn test_pattern_reuse() {
        let json = "{\"structures\":[{\"id\":\"temple_glass\",\"usage\":\"connectable\",\"pattern\":[\"###\",\"#A#\",\"###\"],\"legend\":{\"#\":{\"type\":\"wall\",\"id\":\"glass_wall\"},\"A\":{\"type\":\"interactable\",\"id\":\"altar\"}},\"metadata\":{\"faction\":\"mirror_monks\"}},{\"id\":\"temple_stone\",\"usage\":\"connectable\",\"pattern\":[\"###\",\"#A#\",\"###\"],\"legend\":{\"#\":{\"type\":\"wall\",\"id\":\"stone_wall\"},\"A\":{\"type\":\"interactable\",\"id\":\"altar\"}},\"metadata\":{\"faction\":\"storm_cults\"}}]}";
        
        let library = StructureLibrary::from_json(json, Path::new(".")).expect("Failed to load");
        
        let glass_temple = library.get("temple_glass").unwrap();
        let stone_temple = library.get("temple_stone").unwrap();
        
        assert_eq!(glass_temple.width, stone_temple.width);
        assert_eq!(glass_temple.height, stone_temple.height);
        assert_eq!(glass_temple.metadata.faction, Some("mirror_monks".to_string()));
        assert_eq!(stone_temple.metadata.faction, Some("storm_cults".to_string()));
    }
}
