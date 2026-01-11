use crate::game::generation::structures::Rectangle;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleRoomsParams {
    pub num_rooms: u32,
    pub min_room_size: (u32, u32),
    pub max_room_size: (u32, u32),
    pub corridor_width: u32,
    pub max_placement_attempts: u32,
    pub room_spacing: u32,
}

impl Default for SimpleRoomsParams {
    fn default() -> Self {
        Self {
            num_rooms: 8,
            min_room_size: (4, 4),
            max_room_size: (10, 8),
            corridor_width: 1,
            max_placement_attempts: 100,
            room_spacing: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimpleRoom {
    pub bounds: Rectangle,
}

pub struct SimpleRoomsAlgorithm {
    params: SimpleRoomsParams,
}

impl SimpleRoomsAlgorithm {
    pub fn new(params: SimpleRoomsParams) -> Self {
        Self { params }
    }

    pub fn generate(
        &self,
        bounds: Rectangle,
        rng: &mut ChaCha8Rng,
    ) -> (Vec<SimpleRoom>, Vec<(u32, u32)>) {
        let mut rooms = Vec::new();
        let mut attempts = 0;

        // Generate rooms
        while rooms.len() < self.params.num_rooms as usize
            && attempts < self.params.max_placement_attempts
        {
            let room_width =
                rng.gen_range(self.params.min_room_size.0..=self.params.max_room_size.0);
            let room_height =
                rng.gen_range(self.params.min_room_size.1..=self.params.max_room_size.1);

            if room_width >= bounds.width || room_height >= bounds.height {
                attempts += 1;
                continue;
            }

            let x = bounds.x + rng.gen_range(1..bounds.width - room_width - 1);
            let y = bounds.y + rng.gen_range(1..bounds.height - room_height - 1);

            let new_room = SimpleRoom {
                bounds: Rectangle {
                    x,
                    y,
                    width: room_width,
                    height: room_height,
                },
            };

            // Check if room overlaps with existing rooms
            if !self.room_overlaps(&new_room, &rooms) {
                rooms.push(new_room);
            }

            attempts += 1;
        }

        // Generate corridors connecting all rooms
        let corridors = self.connect_rooms(&rooms, rng);

        (rooms, corridors)
    }

    fn room_overlaps(&self, new_room: &SimpleRoom, existing_rooms: &[SimpleRoom]) -> bool {
        for room in existing_rooms {
            if self.rooms_intersect(new_room, room) {
                return true;
            }
        }
        false
    }

    fn rooms_intersect(&self, room1: &SimpleRoom, room2: &SimpleRoom) -> bool {
        let spacing = self.params.room_spacing;

        room1.bounds.x < room2.bounds.x + room2.bounds.width + spacing
            && room1.bounds.x + room1.bounds.width + spacing > room2.bounds.x
            && room1.bounds.y < room2.bounds.y + room2.bounds.height + spacing
            && room1.bounds.y + room1.bounds.height + spacing > room2.bounds.y
    }

    fn connect_rooms(&self, rooms: &[SimpleRoom], rng: &mut ChaCha8Rng) -> Vec<(u32, u32)> {
        let mut corridor_tiles = Vec::new();

        if rooms.len() < 2 {
            return corridor_tiles;
        }

        // Connect each room to the next one
        for i in 0..rooms.len() - 1 {
            let room1 = &rooms[i];
            let room2 = &rooms[i + 1];

            let tiles = self.create_corridor(room1, room2, rng);
            corridor_tiles.extend(tiles);
        }

        // Connect last room to first to ensure connectivity
        if rooms.len() > 2 {
            let last_room = &rooms[rooms.len() - 1];
            let first_room = &rooms[0];
            let tiles = self.create_corridor(last_room, first_room, rng);
            corridor_tiles.extend(tiles);
        }

        corridor_tiles
    }

    fn create_corridor(
        &self,
        room1: &SimpleRoom,
        room2: &SimpleRoom,
        rng: &mut ChaCha8Rng,
    ) -> Vec<(u32, u32)> {
        let mut tiles = Vec::new();

        let start_x = room1.bounds.x + rng.gen_range(0..room1.bounds.width);
        let start_y = room1.bounds.y + rng.gen_range(0..room1.bounds.height);
        let end_x = room2.bounds.x + rng.gen_range(0..room2.bounds.width);
        let end_y = room2.bounds.y + rng.gen_range(0..room2.bounds.height);

        // Create L-shaped corridor
        // Horizontal segment
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        for x in min_x..=max_x {
            for w in 0..self.params.corridor_width {
                tiles.push((x, start_y + w));
            }
        }

        // Vertical segment
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);
        for y in min_y..=max_y {
            for w in 0..self.params.corridor_width {
                tiles.push((end_x + w, y));
            }
        }

        tiles
    }
}
