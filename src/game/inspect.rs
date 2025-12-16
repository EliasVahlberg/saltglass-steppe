//! Inspection and query functions for game state

use super::combat::get_weapon_def;
use super::item::get_item_def;
use super::state::GameState;

impl GameState {
    /// Describe what's at a given position (for look mode)
    pub fn describe_at(&self, x: i32, y: i32) -> String {
        let idx = self.map.idx(x, y);
        if !self.visible.contains(&idx) && !self.revealed.contains(&idx) {
            return "Unknown".into();
        }
        if x == self.player_x && y == self.player_y {
            return "You".into();
        }
        if let Some(ei) = self.enemy_at(x, y) {
            let e = &self.enemies[ei];
            let desc = e.def().map(|d| d.description.as_str()).unwrap_or("A creature");
            let demeanor = format!("{:?}", e.demeanor()).to_lowercase();
            return format!("{} (HP: {}, {}) - {}", e.name(), e.hp, demeanor, desc);
        }
        if let Some(ni) = self.npc_at(x, y) {
            let n = &self.npcs[ni];
            let desc = n.def().map(|d| d.description.as_str()).unwrap_or("A person");
            return format!("{} - {}", n.name(), desc);
        }
        if let Some(item) = self.items.iter().find(|i| i.x == x && i.y == y) {
            if let Some(def) = get_item_def(&item.id) {
                let item_type = if def.armor_value > 0 { "armor" }
                    else if def.usable { "consumable" }
                    else { "item" };
                return format!("{} ({}) - {}", def.name, item_type, def.description);
            }
            if let Some(wdef) = get_weapon_def(&item.id) {
                return format!("{} (weapon) - {}", wdef.name, wdef.description);
            }
        }
        if let Some(tile) = self.map.get(x, y) {
            return format!("{} - {}", tile.name(), tile.description());
        }
        "Void".into()
    }

    /// Get direction string from player to a position
    pub fn direction_from(&self, x: i32, y: i32) -> &'static str {
        let dx = x - self.player_x;
        let dy = y - self.player_y;
        match (dx.signum(), dy.signum()) {
            (0, -1) => "to the north",
            (0, 1) => "to the south",
            (-1, 0) => "to the west",
            (1, 0) => "to the east",
            (-1, -1) => "to the northwest",
            (1, -1) => "to the northeast",
            (-1, 1) => "to the southwest",
            (1, 1) => "to the southeast",
            _ => "nearby",
        }
    }
}
