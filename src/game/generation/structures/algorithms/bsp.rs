use crate::game::generation::structures::{Corridor, Rectangle, Room};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BSPParams {
    pub min_room_size: (u32, u32),
    pub max_room_size: (u32, u32),
    pub corridor_width: u32,
    pub max_depth: u32,
    pub split_ratio_min: f32,
    pub split_ratio_max: f32,
}

impl Default for BSPParams {
    fn default() -> Self {
        Self {
            min_room_size: (4, 4),
            max_room_size: (12, 8),
            corridor_width: 1,
            max_depth: 5,
            split_ratio_min: 0.3,
            split_ratio_max: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BSPNode {
    pub bounds: Rectangle,
    pub room: Option<Room>,
    pub left: Option<Box<BSPNode>>,
    pub right: Option<Box<BSPNode>>,
    pub depth: u32,
}

pub struct BSPAlgorithm {
    params: BSPParams,
}

impl BSPAlgorithm {
    pub fn new(params: BSPParams) -> Self {
        Self { params }
    }

    pub fn generate(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> (Vec<Room>, Vec<Corridor>) {
        let root = self.partition(bounds, 0, rng);
        let rooms = self.collect_rooms(&root);
        let corridors = self.connect_rooms(&root, rng);
        (rooms, corridors)
    }

    fn partition(&self, bounds: Rectangle, depth: u32, rng: &mut ChaCha8Rng) -> BSPNode {
        let mut node = BSPNode {
            bounds: bounds.clone(),
            room: None,
            left: None,
            right: None,
            depth,
        };

        // Stop partitioning if max depth reached or area too small
        if depth >= self.params.max_depth
            || bounds.width < self.params.min_room_size.0 * 2
            || bounds.height < self.params.min_room_size.1 * 2
        {
            node.room = Some(self.create_room(&bounds, rng));
            return node;
        }

        // Choose split direction (prefer splitting along longer axis)
        let split_horizontal = if bounds.width > bounds.height * 2 {
            false // Split vertically
        } else if bounds.height > bounds.width * 2 {
            true // Split horizontally
        } else {
            rng.gen_bool(0.5)
        };

        let (left_bounds, right_bounds) = self.split_bounds(bounds, split_horizontal, rng);

        node.left = Some(Box::new(self.partition(left_bounds, depth + 1, rng)));
        node.right = Some(Box::new(self.partition(right_bounds, depth + 1, rng)));

        node
    }

    fn split_bounds(
        &self,
        bounds: Rectangle,
        horizontal: bool,
        rng: &mut ChaCha8Rng,
    ) -> (Rectangle, Rectangle) {
        let ratio = rng.gen_range(self.params.split_ratio_min..=self.params.split_ratio_max);

        if horizontal {
            let split_y = bounds.y + (bounds.height as f32 * ratio) as u32;
            let left = Rectangle::new(bounds.x, bounds.y, bounds.width, split_y - bounds.y);
            let right = Rectangle::new(
                bounds.x,
                split_y,
                bounds.width,
                bounds.y + bounds.height - split_y,
            );
            (left, right)
        } else {
            let split_x = bounds.x + (bounds.width as f32 * ratio) as u32;
            let left = Rectangle::new(bounds.x, bounds.y, split_x - bounds.x, bounds.height);
            let right = Rectangle::new(
                split_x,
                bounds.y,
                bounds.x + bounds.width - split_x,
                bounds.height,
            );
            (left, right)
        }
    }

    fn create_room(&self, bounds: &Rectangle, rng: &mut ChaCha8Rng) -> Room {
        let max_width = bounds.width.min(self.params.max_room_size.0);
        let max_height = bounds.height.min(self.params.max_room_size.1);

        // Ensure we have valid ranges
        let width = if self.params.min_room_size.0 <= max_width {
            rng.gen_range(self.params.min_room_size.0..=max_width)
        } else {
            max_width
        };

        let height = if self.params.min_room_size.1 <= max_height {
            rng.gen_range(self.params.min_room_size.1..=max_height)
        } else {
            max_height
        };

        let max_x_offset = if bounds.width > width {
            bounds.width - width
        } else {
            0
        };
        let max_y_offset = if bounds.height > height {
            bounds.height - height
        } else {
            0
        };

        let x = bounds.x
            + if max_x_offset > 0 {
                rng.gen_range(0..=max_x_offset)
            } else {
                0
            };
        let y = bounds.y
            + if max_y_offset > 0 {
                rng.gen_range(0..=max_y_offset)
            } else {
                0
            };

        Room {
            bounds: Rectangle::new(x, y, width, height),
            room_type: "chamber".to_string(),
            depth_from_entrance: 0,
        }
    }

    fn collect_rooms(&self, node: &BSPNode) -> Vec<Room> {
        let mut rooms = Vec::new();

        if let Some(ref room) = node.room {
            rooms.push(room.clone());
        }

        if let Some(ref left) = node.left {
            rooms.extend(self.collect_rooms(left));
        }

        if let Some(ref right) = node.right {
            rooms.extend(self.collect_rooms(right));
        }

        rooms
    }

    fn connect_rooms(&self, node: &BSPNode, rng: &mut ChaCha8Rng) -> Vec<Corridor> {
        let mut corridors = Vec::new();

        if let (Some(left), Some(right)) = (&node.left, &node.right) {
            // Connect left and right subtrees
            if let (Some(left_room), Some(right_room)) = (
                self.find_closest_room(left, right),
                self.find_closest_room(right, left),
            ) {
                corridors.push(self.create_corridor(&left_room, &right_room, rng));
            }

            // Recursively connect within subtrees
            corridors.extend(self.connect_rooms(left, rng));
            corridors.extend(self.connect_rooms(right, rng));
        }

        corridors
    }

    fn find_closest_room(&self, from_node: &BSPNode, to_node: &BSPNode) -> Option<Room> {
        let from_rooms = self.collect_rooms(from_node);
        let to_rooms = self.collect_rooms(to_node);

        let mut closest_room = None;
        let mut min_distance = f32::MAX;

        for from_room in &from_rooms {
            for to_room in &to_rooms {
                let distance = self.room_distance(from_room, to_room);
                if distance < min_distance {
                    min_distance = distance;
                    closest_room = Some(from_room.clone());
                }
            }
        }

        closest_room
    }

    fn room_distance(&self, room1: &Room, room2: &Room) -> f32 {
        let center1 = (
            room1.bounds.x + room1.bounds.width / 2,
            room1.bounds.y + room1.bounds.height / 2,
        );
        let center2 = (
            room2.bounds.x + room2.bounds.width / 2,
            room2.bounds.y + room2.bounds.height / 2,
        );

        let dx = center1.0 as f32 - center2.0 as f32;
        let dy = center1.1 as f32 - center2.1 as f32;

        (dx * dx + dy * dy).sqrt()
    }

    fn create_corridor(&self, room1: &Room, room2: &Room, rng: &mut ChaCha8Rng) -> Corridor {
        let center1 = (
            room1.bounds.x + room1.bounds.width / 2,
            room1.bounds.y + room1.bounds.height / 2,
        );
        let center2 = (
            room2.bounds.x + room2.bounds.width / 2,
            room2.bounds.y + room2.bounds.height / 2,
        );

        // Create L-shaped corridor with random corner choice
        let corner = if rng.gen_bool(0.5) {
            (center1.0, center2.1) // Horizontal first, then vertical
        } else {
            (center2.0, center1.1) // Vertical first, then horizontal
        };

        Corridor {
            start: center1,
            end: corner,
            width: self.params.corridor_width,
        }
    }
}
