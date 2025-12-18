//! Equipment system with slots and stat bonuses

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipSlot {
    Weapon,
    Accessory,
    Jacket,
    Boots,
    Gloves,
    Backpack,
    Necklace,
}

impl EquipSlot {
    pub fn all() -> &'static [EquipSlot] {
        &[
            EquipSlot::Weapon,
            EquipSlot::Accessory,
            EquipSlot::Jacket,
            EquipSlot::Boots,
            EquipSlot::Gloves,
            EquipSlot::Backpack,
            EquipSlot::Necklace,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EquipSlot::Weapon => "Weapon",
            EquipSlot::Accessory => "Accessory",
            EquipSlot::Jacket => "Jacket",
            EquipSlot::Boots => "Boots",
            EquipSlot::Gloves => "Gloves",
            EquipSlot::Backpack => "Backpack",
            EquipSlot::Necklace => "Necklace",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub accessory: Option<String>,
    pub jacket: Option<String>,
    pub boots: Option<String>,
    pub gloves: Option<String>,
    pub backpack: Option<String>,
    pub necklace: Option<String>,
}

impl Equipment {
    pub fn get(&self, slot: EquipSlot) -> Option<&String> {
        match slot {
            EquipSlot::Weapon => self.weapon.as_ref(),
            EquipSlot::Accessory => self.accessory.as_ref(),
            EquipSlot::Jacket => self.jacket.as_ref(),
            EquipSlot::Boots => self.boots.as_ref(),
            EquipSlot::Gloves => self.gloves.as_ref(),
            EquipSlot::Backpack => self.backpack.as_ref(),
            EquipSlot::Necklace => self.necklace.as_ref(),
        }
    }

    pub fn set(&mut self, slot: EquipSlot, item: Option<String>) -> Option<String> {
        match slot {
            EquipSlot::Weapon => std::mem::replace(&mut self.weapon, item),
            EquipSlot::Accessory => std::mem::replace(&mut self.accessory, item),
            EquipSlot::Jacket => std::mem::replace(&mut self.jacket, item),
            EquipSlot::Boots => std::mem::replace(&mut self.boots, item),
            EquipSlot::Gloves => std::mem::replace(&mut self.gloves, item),
            EquipSlot::Backpack => std::mem::replace(&mut self.backpack, item),
            EquipSlot::Necklace => std::mem::replace(&mut self.necklace, item),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (EquipSlot, Option<&String>)> {
        EquipSlot::all().iter().map(|&slot| (slot, self.get(slot)))
    }
}
