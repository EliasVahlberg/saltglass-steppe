use once_cell::sync::Lazy;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct WeaponDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub damage_min: i32,
    pub damage_max: i32,
    pub accuracy: i32,
    pub range: i32,
    #[serde(default = "default_ap_cost")]
    pub ap_cost: i32,
    #[serde(default)]
    pub ammo_type: Option<String>,
    #[serde(default)]
    pub description: String,
}

fn default_ap_cost() -> i32 { 2 }

#[derive(Deserialize)]
struct WeaponsFile {
    weapons: Vec<WeaponDef>,
}

static WEAPON_DEFS: Lazy<HashMap<String, WeaponDef>> = Lazy::new(|| {
    let data = include_str!("../../data/weapons.json");
    let file: WeaponsFile = serde_json::from_str(data).expect("Failed to parse weapons.json");
    file.weapons.into_iter().map(|w| (w.id.clone(), w)).collect()
});

pub fn get_weapon_def(id: &str) -> Option<&'static WeaponDef> {
    WEAPON_DEFS.get(id)
}

pub fn default_weapon() -> &'static WeaponDef {
    WEAPON_DEFS.get("fists").expect("fists weapon must exist")
}

/// Combat result from an attack
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub hit: bool,
    pub damage: i32,
    pub crit: bool,
}

/// Calculate hit chance: base_accuracy - target_reflex + attacker_bonus
/// Returns percentage (0-100)
pub fn calc_hit_chance(accuracy: i32, target_reflex: i32, cover_bonus: i32) -> i32 {
    (accuracy - target_reflex - cover_bonus).clamp(5, 95)
}

/// Calculate damage: base_damage - armor, minimum 1
pub fn calc_damage(base_damage: i32, armor: i32) -> i32 {
    (base_damage - armor).max(1)
}

/// Roll attack with weapon against target
pub fn roll_attack<R: Rng>(
    rng: &mut R,
    weapon: &WeaponDef,
    target_reflex: i32,
    target_armor: i32,
    cover_bonus: i32,
) -> CombatResult {
    let hit_chance = calc_hit_chance(weapon.accuracy, target_reflex, cover_bonus);
    let roll = rng.gen_range(1..=100);
    
    if roll > hit_chance {
        return CombatResult { hit: false, damage: 0, crit: false };
    }
    
    // Crit on roll <= 5 (5% chance when hit)
    let crit = roll <= 5;
    let base_damage = rng.gen_range(weapon.damage_min..=weapon.damage_max);
    let damage = if crit { base_damage * 2 } else { base_damage };
    let final_damage = calc_damage(damage, target_armor);
    
    CombatResult { hit: true, damage: final_damage, crit }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_chance_clamped() {
        assert_eq!(calc_hit_chance(100, 0, 0), 95); // Max 95%
        assert_eq!(calc_hit_chance(0, 100, 0), 5);  // Min 5%
        assert_eq!(calc_hit_chance(80, 10, 20), 50);
    }

    #[test]
    fn damage_minimum_one() {
        assert_eq!(calc_damage(5, 10), 1); // Armor > damage = 1
        assert_eq!(calc_damage(10, 5), 5);
    }

    #[test]
    fn weapon_defs_load() {
        assert!(get_weapon_def("fists").is_some());
        assert!(get_weapon_def("salt_knife").is_some());
    }
}
