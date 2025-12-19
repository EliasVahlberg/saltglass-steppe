//! Inspection and query functions for game state

use super::combat::get_weapon_def;
use super::item::get_item_def;
use super::light_defs::get_light_def;
use super::state::GameState;

/// Detailed item information for inspection
#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub name: String,
    pub description: String,
    pub item_type: String,
    pub stats: Vec<(String, String)>,
}

/// Get detailed info for an item by ID (data-driven, respects hidden_properties)
pub fn inspect_item(id: &str) -> Option<ItemInfo> {
    // Check weapons first
    if let Some(w) = get_weapon_def(id) {
        return Some(ItemInfo {
            name: w.name.clone(),
            description: w.description.clone(),
            item_type: "Weapon".into(),
            stats: vec![
                ("Damage".into(), format!("{}-{}", w.damage_min, w.damage_max)),
                ("Accuracy".into(), format!("{}%", w.accuracy)),
                ("Range".into(), w.range.to_string()),
                ("AP Cost".into(), w.ap_cost.to_string()),
            ],
        });
    }
    // Check items - data-driven property display
    let d = get_item_def(id)?;
    let hidden = &d.hidden_properties;
    let mut stats = Vec::new();
    
    macro_rules! add_stat {
        ($key:expr, $cond:expr, $val:expr) => {
            if $cond && !hidden.iter().any(|h| h == $key) {
                stats.push(($key.into(), $val));
            }
        };
    }
    
    add_stat!("value", d.value > 0, d.value.to_string());
    add_stat!("weight", d.weight > 0, d.weight.to_string());
    add_stat!("heal", d.heal > 0, format!("+{} HP", d.heal));
    add_stat!("armor_value", d.armor_value > 0, format!("+{}", d.armor_value));
    add_stat!("reduces_refraction", d.reduces_refraction > 0, format!("-{}", d.reduces_refraction));
    add_stat!("reveals_map", d.reveals_map, "Reveals map".into());
    add_stat!("suppresses_adaptations", d.suppresses_adaptations, "Hides adaptations".into());
    add_stat!("breaks_walls", d.breaks_walls, "Breaks walls".into());
    add_stat!("equip_slot", d.equip_slot.is_some(), d.equip_slot.clone().unwrap_or_default());
    
    let item_type = if d.equip_slot.is_some() { "Equipment" }
        else if d.usable { "Consumable" }
        else { "Item" };
    
    Some(ItemInfo {
        name: d.name.clone(),
        description: d.description.clone(),
        item_type: item_type.into(),
        stats,
    })
}

impl GameState {
    /// Inspect an item in inventory by index
    pub fn inspect_inventory(&self, idx: usize) -> Option<ItemInfo> {
        self.inventory.get(idx).and_then(|id| inspect_item(id))
    }
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
        // Check for map lights (torches, braziers, etc.)
        if let Some(light) = self.map.lights.iter().find(|l| l.x == x && l.y == y) {
            if let Some(def) = get_light_def(&light.id) {
                return format!("{} (light source)", def.name);
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
