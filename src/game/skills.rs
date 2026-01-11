//! Physical Skills and Abilities System

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Combat,
    Athletics,
    Survival,
    Crafting,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub category: SkillCategory,
    pub description: String,
    pub max_level: u32,
    pub base_cost: u32, // XP cost for first level
}

#[derive(Clone, Debug, Deserialize)]
pub struct AbilityDef {
    pub id: String,
    pub name: String,
    pub category: SkillCategory,
    pub description: String,
    pub stamina_cost: u32,
    pub cooldown: u32,
    pub required_skill: String,
    pub required_level: u32,
    pub effect: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillsState {
    pub stamina: u32,
    pub max_stamina: u32,
    pub skills: HashMap<String, u32>, // skill_id -> level
    pub unlocked_abilities: Vec<String>,
    pub cooldowns: HashMap<String, u32>,
    pub skill_points: u32,
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

        Ok(def.effect.clone())
    }

    /// Upgrade a skill with skill points
    pub fn upgrade_skill(&mut self, skill_id: &str) -> Result<(), String> {
        let def = get_skill_def(skill_id).ok_or_else(|| format!("Unknown skill: {}", skill_id))?;

        let current_level = self.skills.get(skill_id).unwrap_or(&0);
        if *current_level >= def.max_level {
            return Err("Skill already at max level".to_string());
        }

        let cost = calculate_skill_cost(&def.id, *current_level);
        if self.skill_points < cost {
            return Err("Not enough skill points".to_string());
        }

        self.skill_points -= cost;
        self.skills.insert(skill_id.to_string(), current_level + 1);

        // Check for new abilities unlocked
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
        self.skills.get(skill_id).unwrap_or(&0).clone()
    }
}

/// Calculate XP cost for upgrading a skill
pub fn calculate_skill_cost(skill_id: &str, current_level: u32) -> u32 {
    let base_cost = get_skill_def(skill_id)
        .map(|def| def.base_cost)
        .unwrap_or(10);

    base_cost * (current_level + 1)
}

// Data loading
static SKILLS: Lazy<HashMap<String, SkillDef>> = Lazy::new(|| {
    let data = include_str!("../../data/skills.json");
    match serde_json::from_str::<Vec<SkillDef>>(data) {
        Ok(skills_vec) => {
            let mut skills_map = HashMap::new();
            for skill in skills_vec {
                skills_map.insert(skill.id.clone(), skill);
            }
            skills_map
        }
        Err(e) => {
            eprintln!("Failed to parse skills.json: {}", e);
            HashMap::new()
        }
    }
});

static ABILITIES: Lazy<HashMap<String, AbilityDef>> = Lazy::new(|| {
    let data = include_str!("../../data/abilities.json");
    match serde_json::from_str::<Vec<AbilityDef>>(data) {
        Ok(abilities_vec) => {
            let mut abilities_map = HashMap::new();
            for ability in abilities_vec {
                abilities_map.insert(ability.id.clone(), ability);
            }
            abilities_map
        }
        Err(e) => {
            eprintln!("Failed to parse abilities.json: {}", e);
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
    ABILITIES
        .values()
        .filter(|def| &def.category == category)
        .collect()
}
