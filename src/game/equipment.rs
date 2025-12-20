//! Equipment system with slots and stat bonuses

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipSlot {
    Weapon,
    RangedWeapon,
    Head,
    Jacket,
    Pants,
    Boots,
    Gloves,
    LeftWrist,
    RightWrist,
    Necklace,
    Accessory,
    Backpack,
}

impl FromStr for EquipSlot {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "weapon" => Ok(EquipSlot::Weapon),
            "ranged_weapon" | "ranged" => Ok(EquipSlot::RangedWeapon),
            "head" | "helmet" => Ok(EquipSlot::Head),
            "jacket" | "chest" | "torso" => Ok(EquipSlot::Jacket),
            "pants" | "legs" => Ok(EquipSlot::Pants),
            "boots" | "feet" => Ok(EquipSlot::Boots),
            "gloves" | "hands" => Ok(EquipSlot::Gloves),
            "left_wrist" => Ok(EquipSlot::LeftWrist),
            "right_wrist" => Ok(EquipSlot::RightWrist),
            "necklace" | "neck" => Ok(EquipSlot::Necklace),
            "accessory" => Ok(EquipSlot::Accessory),
            "backpack" => Ok(EquipSlot::Backpack),
            _ => Err(()),
        }
    }
}

impl EquipSlot {
    pub fn all() -> &'static [EquipSlot] {
        &[
            EquipSlot::Weapon,
            EquipSlot::RangedWeapon,
            EquipSlot::Head,
            EquipSlot::Jacket,
            EquipSlot::Pants,
            EquipSlot::Boots,
            EquipSlot::Gloves,
            EquipSlot::LeftWrist,
            EquipSlot::RightWrist,
            EquipSlot::Necklace,
            EquipSlot::Accessory,
            EquipSlot::Backpack,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EquipSlot::Weapon => "Melee Weapon",
            EquipSlot::RangedWeapon => "Ranged Weapon",
            EquipSlot::Head => "Head",
            EquipSlot::Jacket => "Jacket",
            EquipSlot::Pants => "Pants",
            EquipSlot::Boots => "Boots",
            EquipSlot::Gloves => "Gloves",
            EquipSlot::LeftWrist => "Left Wrist",
            EquipSlot::RightWrist => "Right Wrist",
            EquipSlot::Necklace => "Necklace",
            EquipSlot::Accessory => "Accessory",
            EquipSlot::Backpack => "Backpack",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub ranged_weapon: Option<String>,
    pub head: Option<String>,
    pub jacket: Option<String>,
    pub pants: Option<String>,
    pub boots: Option<String>,
    pub gloves: Option<String>,
    pub left_wrist: Option<String>,
    pub right_wrist: Option<String>,
    pub necklace: Option<String>,
    pub accessory: Option<String>,
    pub backpack: Option<String>,
}

impl Equipment {
    pub fn get(&self, slot: EquipSlot) -> Option<&String> {
        match slot {
            EquipSlot::Weapon => self.weapon.as_ref(),
            EquipSlot::RangedWeapon => self.ranged_weapon.as_ref(),
            EquipSlot::Head => self.head.as_ref(),
            EquipSlot::Jacket => self.jacket.as_ref(),
            EquipSlot::Pants => self.pants.as_ref(),
            EquipSlot::Boots => self.boots.as_ref(),
            EquipSlot::Gloves => self.gloves.as_ref(),
            EquipSlot::LeftWrist => self.left_wrist.as_ref(),
            EquipSlot::RightWrist => self.right_wrist.as_ref(),
            EquipSlot::Necklace => self.necklace.as_ref(),
            EquipSlot::Accessory => self.accessory.as_ref(),
            EquipSlot::Backpack => self.backpack.as_ref(),
        }
    }

    pub fn set(&mut self, slot: EquipSlot, item: Option<String>) -> Option<String> {
        match slot {
            EquipSlot::Weapon => std::mem::replace(&mut self.weapon, item),
            EquipSlot::RangedWeapon => std::mem::replace(&mut self.ranged_weapon, item),
            EquipSlot::Head => std::mem::replace(&mut self.head, item),
            EquipSlot::Jacket => std::mem::replace(&mut self.jacket, item),
            EquipSlot::Pants => std::mem::replace(&mut self.pants, item),
            EquipSlot::Boots => std::mem::replace(&mut self.boots, item),
            EquipSlot::Gloves => std::mem::replace(&mut self.gloves, item),
            EquipSlot::LeftWrist => std::mem::replace(&mut self.left_wrist, item),
            EquipSlot::RightWrist => std::mem::replace(&mut self.right_wrist, item),
            EquipSlot::Necklace => std::mem::replace(&mut self.necklace, item),
            EquipSlot::Accessory => std::mem::replace(&mut self.accessory, item),
            EquipSlot::Backpack => std::mem::replace(&mut self.backpack, item),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (EquipSlot, Option<&String>)> {
        EquipSlot::all().iter().map(|&slot| (slot, self.get(slot)))
    }
}
