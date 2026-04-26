//! Physical Skills and Abilities System

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    SaltAlchemy,
    Crafting,
    Social,
    Survival,
    Medical,
    MeleeCombat,
    RangedCombat,
    // Legacy variants kept for save compatibility
    Combat,
    Athletics,
}

pub const SKILL_CATEGORIES: &[SkillCategory] = &[
    SkillCategory::SaltAlchemy,
    SkillCategory::Crafting,
    SkillCategory::Social,
    SkillCategory::Survival,
    SkillCategory::Medical,
    SkillCategory::MeleeCombat,
    SkillCategory::RangedCombat,
];

pub fn category_name(cat: &SkillCategory) -> &'static str {
    match cat {
        SkillCategory::SaltAlchemy => "Salt Alchemy",
        SkillCategory::Crafting => "Crafting",
        SkillCategory::Social => "Social",
        SkillCategory::Survival => "Survival",
        SkillCategory::Medical => "Medical",
        SkillCategory::MeleeCombat => "Melee Combat",
        SkillCategory::RangedCombat => "Ranged Combat",
        SkillCategory::Combat => "Combat",
        SkillCategory::Athletics => "Athletics",
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillPrerequisite {
    pub skill_id: String,
    pub required_level: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PassiveEffect {
    pub effect_type: String,
    pub target: String,
    pub value_per_level: f32,
    pub max_value: Option<f32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub category: SkillCategory,
    pub description: String,
    pub max_level: u32,
    pub base_cost: u32, // XP cost for first level
    #[serde(default)]
    pub prerequisites: Vec<SkillPrerequisite>,
    #[serde(default)]
    pub passive_effects: Vec<PassiveEffect>,
    #[serde(default)]
    pub tree_parent: Option<String>,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub active: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AbilityDef {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub stamina_cost: u32,
    pub cooldown: u32,
    pub required_skill: String,
    pub required_level: u32,
    #[serde(default)]
    pub effect: Option<String>,
    #[serde(default)]
    pub effect_type: Option<String>,
    #[serde(default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub effect_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillsState {
    pub stamina: u32,
    pub max_stamina: u32,
    pub skills: HashMap<String, u32>, // skill_id -> level
    pub unlocked_abilities: Vec<String>,
    pub cooldowns: HashMap<String, u32>,
    pub skill_points: u32,
    #[serde(default)]
    pub passive_bonuses: HashMap<String, f32>, // effect_type -> total_bonus
}

impl Default for SkillsState {
    fn default() -> Self {
        Self {
            stamina: 50,
            max_stamina: 50,
            skills: HashMap::new(),
            unlocked_abilities: Vec::new(),
            cooldowns: HashMap::new(),
            skill_points: 5, // Start with 5 skill points
            passive_bonuses: HashMap::new(),
        }
    }
}

impl SkillsState {
    /// Tick cooldowns and regenerate stamina
    pub fn tick(&mut self) {
        // Reduce cooldowns
        self.cooldowns.retain(|_, cd| {
            *cd = cd.saturating_sub(1);
            *cd > 0
        });

        // Regenerate stamina (1 per turn)
        if self.stamina < self.max_stamina {
            self.stamina += 1;
        }
    }

    /// Use an ability if possible
    pub fn use_ability(&mut self, ability_id: &str) -> Result<String, String> {
        let def = get_ability_def(ability_id)
            .ok_or_else(|| format!("Unknown ability: {}", ability_id))?;

        // Check if unlocked
        if !self.unlocked_abilities.contains(&ability_id.to_string()) {
            return Err("Ability not unlocked".to_string());
        }

        // Check cooldown
        if self.cooldowns.get(ability_id).unwrap_or(&0) > &0 {
            return Err("Ability on cooldown".to_string());
        }

        // Check stamina
        if self.stamina < def.stamina_cost {
            return Err("Not enough stamina".to_string());
        }

        // Check skill requirement
        let skill_level = self.skills.get(&def.required_skill).unwrap_or(&0);
        if *skill_level < def.required_level {
            return Err("Skill level too low".to_string());
        }

        // Use ability
        self.stamina -= def.stamina_cost;
        self.cooldowns.insert(ability_id.to_string(), def.cooldown);

        let effect = def
            .effect
            .clone()
            .or_else(|| def.effect_type.clone())
            .unwrap_or_else(|| "unknown".to_string());
        Ok(effect)
    }

    /// Check if a skill can be upgraded (prerequisites and resources)
    pub fn can_upgrade_skill(&self, skill_id: &str) -> Result<(), String> {
        let def = get_skill_def(skill_id).ok_or_else(|| format!("Unknown skill: {}", skill_id))?;

        let current_level = self.skills.get(skill_id).unwrap_or(&0);
        if *current_level >= def.max_level {
            return Err("Skill already at max level".to_string());
        }

        // Check prerequisites
        for prereq in &def.prerequisites {
            let prereq_level = self.skills.get(&prereq.skill_id).unwrap_or(&0);
            if *prereq_level < prereq.required_level {
                return Err(format!(
                    "Requires {} level {}",
                    prereq.skill_id, prereq.required_level
                ));
            }
        }

        let cost = calculate_skill_cost(&def.id, *current_level);
        if self.skill_points < cost {
            return Err("Not enough skill points".to_string());
        }

        Ok(())
    }

    /// Recalculate all passive bonuses from skills
    pub fn recalculate_passive_bonuses(&mut self) {
        self.passive_bonuses.clear();

        for (skill_id, &level) in &self.skills {
            if let Some(def) = get_skill_def(skill_id) {
                for effect in &def.passive_effects {
                    let bonus_value = effect.value_per_level * level as f32;
                    let final_value = if let Some(max_val) = effect.max_value {
                        bonus_value.min(max_val)
                    } else {
                        bonus_value
                    };

                    *self
                        .passive_bonuses
                        .entry(effect.effect_type.clone())
                        .or_insert(0.0) += final_value;
                }
            }
        }
    }

    /// Upgrade a skill with skill points
    pub fn upgrade_skill(&mut self, skill_id: &str) -> Result<(), String> {
        self.can_upgrade_skill(skill_id)?;

        let def = get_skill_def(skill_id).ok_or_else(|| format!("Unknown skill: {skill_id}"))?;
        let current_level = self.skills.get(skill_id).unwrap_or(&0);
        let cost = calculate_skill_cost(&def.id, *current_level);

        self.skill_points -= cost;
        self.skills.insert(skill_id.to_string(), current_level + 1);

        // Recalculate passive bonuses and check for new abilities
        self.recalculate_passive_bonuses();
        self.check_ability_unlocks();

        Ok(())
    }

    /// Check if any new abilities are unlocked
    pub fn check_ability_unlocks(&mut self) {
        for ability_id in all_ability_ids() {
            if self.unlocked_abilities.contains(&ability_id.to_string()) {
                continue;
            }

            if let Some(def) = get_ability_def(ability_id) {
                let skill_level = self.skills.get(&def.required_skill).unwrap_or(&0);
                if *skill_level >= def.required_level {
                    self.unlocked_abilities.push(ability_id.to_string());
                }
            }
        }
    }

    /// Get skill level
    pub fn get_skill_level(&self, skill_id: &str) -> u32 {
        *self.skills.get(skill_id).unwrap_or(&0)
    }

    // --- Typed passive accessors ---
    pub fn melee_accuracy_bonus(&self) -> f32 {
        self.passive_bonuses
            .get("melee_accuracy_bonus")
            .copied()
            .unwrap_or(0.0)
    }
    pub fn melee_damage_bonus(&self) -> f32 {
        self.passive_bonuses
            .get("melee_damage_bonus")
            .copied()
            .unwrap_or(0.0)
    }
    pub fn ranged_accuracy_bonus(&self) -> f32 {
        self.passive_bonuses
            .get("ranged_accuracy_bonus")
            .copied()
            .unwrap_or(0.0)
    }
    pub fn ranged_damage_bonus(&self) -> f32 {
        self.passive_bonuses
            .get("ranged_damage_bonus")
            .copied()
            .unwrap_or(0.0)
    }
    pub fn passive(&self, key: &str) -> f32 {
        self.passive_bonuses.get(key).copied().unwrap_or(0.0)
    }
}

/// Calculate XP cost for upgrading a skill
pub fn calculate_skill_cost(skill_id: &str, current_level: u32) -> u32 {
    let base_cost = get_skill_def(skill_id)
        .map(|def| def.base_cost)
        .unwrap_or(10);

    base_cost * (current_level + 1)
}

#[derive(Deserialize)]
struct AbilitiesFile {
    #[serde(default)]
    skills: Vec<SkillDef>,
    #[serde(default)]
    abilities: Vec<AbilityDef>,
}

// Data loading
#[derive(Deserialize)]
struct SkillTreesFile {
    #[serde(default)]
    skills: Vec<SkillDef>,
}

static SKILLS: Lazy<HashMap<String, SkillDef>> = Lazy::new(|| {
    let mut skills_map = HashMap::new();

    // Load legacy skills from abilities.json
    let abilities_data = include_str!("../../data/abilities.json");
    if let Ok(file) = serde_json::from_str::<AbilitiesFile>(abilities_data) {
        for skill in file.skills {
            skills_map.insert(skill.id.clone(), skill);
        }
    }

    // Load tree-structured skills from skill_trees.json (overrides on conflict)
    let tree_data = include_str!("../../data/skill_trees.json");
    if let Ok(file) = serde_json::from_str::<SkillTreesFile>(tree_data) {
        for skill in file.skills {
            skills_map.insert(skill.id.clone(), skill);
        }
    }

    skills_map
});

static ABILITIES: Lazy<HashMap<String, AbilityDef>> = Lazy::new(|| {
    let data = include_str!("../../data/abilities.json");
    match serde_json::from_str::<AbilitiesFile>(data) {
        Ok(file) => {
            let mut abilities_map = HashMap::new();
            for ability in file.abilities {
                abilities_map.insert(ability.id.clone(), ability);
            }
            abilities_map
        }
        Err(e) => {
            eprintln!("Failed to parse abilities.json (abilities): {}", e);
            HashMap::new()
        }
    }
});

pub fn get_skill_def(id: &str) -> Option<&SkillDef> {
    SKILLS.get(id)
}

pub fn get_ability_def(id: &str) -> Option<&AbilityDef> {
    ABILITIES.get(id)
}

pub fn all_skill_ids() -> Vec<&'static str> {
    SKILLS.keys().map(|s| s.as_str()).collect()
}

pub fn all_ability_ids() -> Vec<&'static str> {
    ABILITIES.keys().map(|s| s.as_str()).collect()
}

pub fn get_skills_by_category(category: &SkillCategory) -> Vec<&SkillDef> {
    SKILLS
        .values()
        .filter(|def| &def.category == category)
        .collect()
}

pub fn get_abilities_by_category(category: &SkillCategory) -> Vec<&AbilityDef> {
    let wanted = format!("{:?}", category).to_lowercase();
    ABILITIES
        .values()
        .filter(|def| def.category.to_lowercase() == wanted)
        .collect()
}

/// Root skills for a category (no tree_parent), sorted by id for stable layout
pub fn get_category_roots(category: &SkillCategory) -> Vec<&'static SkillDef> {
    let mut roots: Vec<&SkillDef> = SKILLS
        .values()
        .filter(|def| &def.category == category && def.tree_parent.is_none())
        .collect();
    roots.sort_by(|a, b| a.id.cmp(&b.id));
    roots
}

/// Direct children of a skill node, sorted by id for stable layout
pub fn get_skill_children(parent_id: &str) -> Vec<&'static SkillDef> {
    let mut children: Vec<&SkillDef> = SKILLS
        .values()
        .filter(|def| def.tree_parent.as_deref() == Some(parent_id))
        .collect();
    children.sort_by(|a, b| a.id.cmp(&b.id));
    children
}
