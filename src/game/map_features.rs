use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Player-discovered locations and annotations
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MapFeatures {
    /// Hidden locations revealed by items
    pub revealed_locations: HashMap<(i32, i32), String>,
    /// Safe routes marked by NPCs
    pub safe_routes: Vec<SafeRoute>,
    /// Storm damage tracking on world map
    pub storm_damage: HashMap<(usize, usize), u32>,
    /// Player annotations and waypoints
    pub player_annotations: HashMap<(i32, i32), String>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafeRoute {
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub marked_by: String, // NPC name
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Waypoint {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub description: String,
}

impl MapFeatures {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reveal a hidden location using an item
    pub fn reveal_location(&mut self, x: i32, y: i32, description: String) {
        self.revealed_locations.insert((x, y), description);
    }

    /// Mark a safe route from NPC dialogue
    pub fn add_safe_route(&mut self, from: (i32, i32), to: (i32, i32), npc_name: String, description: String) {
        self.safe_routes.push(SafeRoute {
            from, to, marked_by: npc_name, description
        });
    }

    /// Track storm damage on world map
    pub fn add_storm_damage(&mut self, world_x: usize, world_y: usize, damage: u32) {
        *self.storm_damage.entry((world_x, world_y)).or_insert(0) += damage;
    }

    /// Add player annotation
    pub fn add_annotation(&mut self, x: i32, y: i32, text: String) {
        self.player_annotations.insert((x, y), text);
    }

    /// Add waypoint
    pub fn add_waypoint(&mut self, x: i32, y: i32, name: String, description: String) {
        self.waypoints.push(Waypoint { x, y, name, description });
    }

    /// Check if location is revealed
    pub fn is_location_revealed(&self, x: i32, y: i32) -> bool {
        self.revealed_locations.contains_key(&(x, y))
    }

    /// Get safe routes from a position
    pub fn get_safe_routes_from(&self, x: i32, y: i32) -> Vec<&SafeRoute> {
        self.safe_routes.iter().filter(|r| r.from == (x, y)).collect()
    }
}
