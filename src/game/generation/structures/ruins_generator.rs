use super::{
    POIType, Rectangle, Room, SpawnPoint, Structure, StructureGenerator, StructureParams,
    StructureType,
};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

pub struct RuinsGenerator;

impl RuinsGenerator {
    pub fn new() -> Self {
        Self
    }

    fn create_vitrified_library_layout(
        &self,
        size: (u32, u32),
        _rng: &mut ChaCha8Rng,
    ) -> Structure {
        let bounds = Rectangle::new(0, 0, size.0, size.1);
        let mut rooms = Vec::new();
        let mut spawn_points = Vec::new();

        // Create main hall (central area)
        let main_hall = Room {
            bounds: Rectangle::new(8, 6, 12, 8),
            room_type: "main_hall".to_string(),
            depth_from_entrance: 1,
        };
        rooms.push(main_hall);

        // Create smaller chambers around the main hall
        let chambers = vec![
            Rectangle::new(2, 2, 6, 5),   // Northwest chamber
            Rectangle::new(16, 2, 6, 5),  // Northeast chamber
            Rectangle::new(2, 12, 6, 5),  // Southwest chamber
            Rectangle::new(16, 12, 6, 5), // Southeast chamber
        ];

        for (i, chamber_bounds) in chambers.into_iter().enumerate() {
            let room = Room {
                bounds: chamber_bounds,
                room_type: format!("chamber_{}", i + 1),
                depth_from_entrance: 2,
            };
            rooms.push(room);
        }

        // Add quest item spawn point (broken_saint_key)
        spawn_points.push(SpawnPoint {
            position: (14, 10), // Center of main hall
            spawn_type: "item".to_string(),
            entity_id: "broken_saint_key".to_string(),
        });

        // Add enemy spawn points
        let enemy_types = ["glass_wraith", "crystal_guardian", "shard_stalker"];
        for (i, &enemy_type) in enemy_types.iter().enumerate() {
            let room_idx = (i % (rooms.len() - 1)) + 1; // Skip main hall for enemies
            let room = &rooms[room_idx];
            let spawn_x = room.bounds.x + room.bounds.width / 2;
            let spawn_y = room.bounds.y + room.bounds.height / 2;

            spawn_points.push(SpawnPoint {
                position: (spawn_x, spawn_y),
                spawn_type: "enemy".to_string(),
                entity_id: enemy_type.to_string(),
            });
        }

        let mut metadata = HashMap::new();
        metadata.insert("theme".to_string(), "vitrified_library".to_string());
        metadata.insert("quest_structure".to_string(), "true".to_string());

        Structure {
            structure_type: StructureType::Ruins,
            bounds,
            rooms,
            corridors: Vec::new(), // Simple open layout, no explicit corridors
            features: Vec::new(),
            spawn_points,
            metadata,
        }
    }
}

impl StructureGenerator for RuinsGenerator {
    fn generate(&self, params: &StructureParams, rng: &mut ChaCha8Rng) -> Option<Structure> {
        match params.structure_type {
            StructureType::Ruins => {
                // For now, generate the vitrified library ruins specifically
                if params.theme == "vitrified_library"
                    || params
                        .quest_requirements
                        .contains(&"the_broken_key".to_string())
                {
                    Some(self.create_vitrified_library_layout(params.size, rng))
                } else {
                    // TODO: Add other ruin types
                    None
                }
            }
            _ => None,
        }
    }

    fn get_supported_poi_types(&self) -> Vec<POIType> {
        vec![POIType::Landmark]
    }
}
