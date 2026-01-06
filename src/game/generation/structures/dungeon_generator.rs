use crate::game::generation::structures::{StructureGenerator, Rectangle, StructureParams, Structure, StructureType, Room, StructureFeature, POIType};
use crate::game::map::{Map, Tile};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonGeneratorParams {
    pub width: u32,
    pub height: u32,
    pub organic_blend_factor: f64,
}

impl Default for DungeonGeneratorParams {
    fn default() -> Self {
        Self {
            width: 80,
            height: 40,
            organic_blend_factor: 0.3,
        }
    }
}

pub struct DungeonGenerator {
    params: DungeonGeneratorParams,
}

impl DungeonGenerator {
    pub fn new(params: DungeonGeneratorParams) -> Self {
        Self { params }
    }

    pub fn with_default() -> Self {
        Self::new(DungeonGeneratorParams::default())
    }

    /// Generate simple dungeon for demonstration
    fn generate_simple(&self, _rng: &mut ChaCha8Rng) -> Map {
        let total_tiles = (self.params.width * self.params.height) as usize;
        let tiles = vec![Tile::Wall { id: "stone".to_string(), hp: 100 }; total_tiles];
        
        let mut map = Map {
            tiles,
            width: self.params.width as usize,
            height: self.params.height as usize,
            lights: Vec::new(),
            inscriptions: Vec::new(),
            area_description: Some("Simple generated dungeon".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        // Create some simple rooms
        let room_width = 8;
        let room_height = 6;
        let rooms_x = (self.params.width / (room_width + 2)).max(1);
        let rooms_y = (self.params.height / (room_height + 2)).max(1);

        for ry in 0..rooms_y {
            for rx in 0..rooms_x {
                let start_x = rx * (room_width + 2) + 1;
                let start_y = ry * (room_height + 2) + 1;
                
                // Carve room
                for y in start_y..start_y + room_height {
                    for x in start_x..start_x + room_width {
                        if x < self.params.width && y < self.params.height {
                            let idx = (y * self.params.width + x) as usize;
                            if idx < map.tiles.len() {
                                map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                            }
                        }
                    }
                }
                
                // Add corridors
                if rx < rooms_x - 1 {
                    // Horizontal corridor
                    let corridor_y = start_y + room_height / 2;
                    for x in start_x + room_width..start_x + room_width + 2 {
                        if x < self.params.width && corridor_y < self.params.height {
                            let idx = (corridor_y * self.params.width + x) as usize;
                            if idx < map.tiles.len() {
                                map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                            }
                        }
                    }
                }
                
                if ry < rooms_y - 1 {
                    // Vertical corridor
                    let corridor_x = start_x + room_width / 2;
                    for y in start_y + room_height..start_y + room_height + 2 {
                        if corridor_x < self.params.width && y < self.params.height {
                            let idx = (y * self.params.width + corridor_x) as usize;
                            if idx < map.tiles.len() {
                                map.tiles[idx] = Tile::Floor { id: "stone".to_string() };
                            }
                        }
                    }
                }
            }
        }

        map
    }
}

impl StructureGenerator for DungeonGenerator {
    fn generate(&self, _params: &StructureParams, rng: &mut ChaCha8Rng) -> Option<Structure> {
        let bounds = Rectangle::new(0, 0, self.params.width, self.params.height);
        let map = self.generate_simple(rng);
        
        let mut rooms = Vec::new();
        let mut features = Vec::new();
        
        // Create a single large "room" representing the entire dungeon
        let main_room = Room {
            bounds: bounds.clone(),
            room_type: "dungeon_chamber".to_string(),
            depth_from_entrance: 0,
        };
        rooms.push(main_room);
        
        // Add wall and floor features
        for (idx, tile) in map.tiles.iter().enumerate() {
            let x = (idx % map.width) as u32;
            let y = (idx / map.width) as u32;
            let pos = (x, y);
            
            match tile {
                Tile::Wall { .. } => {
                    features.push(StructureFeature {
                        position: pos,
                        feature_type: "wall".to_string(),
                        properties: std::collections::HashMap::new(),
                    });
                },
                Tile::Floor { .. } => {
                    features.push(StructureFeature {
                        position: pos,
                        feature_type: "floor".to_string(),
                        properties: std::collections::HashMap::new(),
                    });
                },
                _ => {}
            }
        }

        Some(Structure {
            structure_type: StructureType::Dungeon,
            bounds,
            rooms,
            corridors: Vec::new(),
            features,
            spawn_points: Vec::new(),
            metadata: std::collections::HashMap::new(),
        })
    }

    fn get_supported_poi_types(&self) -> Vec<POIType> {
        vec![POIType::Dungeon]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_simple_dungeon_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let generator = DungeonGenerator::with_default();
        
        let map = generator.generate_simple(&mut rng);
        
        // Verify map dimensions
        assert_eq!(map.width, 80);
        assert_eq!(map.height, 40);
        
        // Count floors and walls
        let mut floor_count = 0;
        let mut wall_count = 0;
        
        for tile in &map.tiles {
            match tile {
                Tile::Floor { .. } => floor_count += 1,
                Tile::Wall { .. } => wall_count += 1,
                _ => {}
            }
        }
        
        // Should have reasonable floor/wall ratio
        assert!(floor_count > 0, "Should have floors");
        assert!(wall_count > 0, "Should have walls");
        assert!(floor_count < map.tiles.len(), "Should not be all floors");
    }

    #[test]
    fn test_deterministic_generation() {
        let params = DungeonGeneratorParams::default();
        let generator = DungeonGenerator::new(params);
        
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        
        let map1 = generator.generate_simple(&mut rng1);
        let map2 = generator.generate_simple(&mut rng2);
        
        // Maps should be identical with same seed
        assert_eq!(map1.tiles.len(), map2.tiles.len());
        for (i, (tile1, tile2)) in map1.tiles.iter().zip(map2.tiles.iter()).enumerate() {
            assert_eq!(
                std::mem::discriminant(tile1),
                std::mem::discriminant(tile2),
                "Tiles should match at index {}", i
            );
        }
    }
}
