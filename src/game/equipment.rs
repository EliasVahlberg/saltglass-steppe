//! Equipment system with slots and stat bonuses

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipSlot {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub accessory: Option<String>,
}

impl Equipment {
    pub fn get(&self, slot: EquipSlot) -> Option<&String> {
        match slot {
            EquipSlot::Weapon => self.weapon.as_ref(),
            EquipSlot::Armor => self.armor.as_ref(),
            EquipSlot::Accessory => self.accessory.as_ref(),
        }
    }

    pub fn set(&mut self, slot: EquipSlot, item: Option<String>) -> Option<String> {
        let old = match slot {
            EquipSlot::Weapon => std::mem::replace(&mut self.weapon, item),
            EquipSlot::Armor => std::mem::replace(&mut self.armor, item),
            EquipSlot::Accessory => std::mem::replace(&mut self.accessory, item),
        };
        old
    }
}
