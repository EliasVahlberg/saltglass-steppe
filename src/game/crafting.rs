//! Data-driven crafting system

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

/// A crafting recipe
#[derive(Debug, Clone, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Required materials: item_id -> count
    pub materials: HashMap<String, u32>,
    /// Output item ID
    pub output: String,
    /// Output quantity (default 1)
    #[serde(default = "default_output_count")]
    pub output_count: u32,
    /// Skill level required (0 = no skill needed)
    #[serde(default)]
    pub skill_required: u32,
    /// Crafting station required (null = can craft anywhere)
    pub station_required: Option<String>,
    /// Faction required (null = no faction restriction)
    pub faction_required: Option<String>,
}

fn default_output_count() -> u32 {
    1
}

#[derive(Deserialize)]
struct RecipesFile {
    recipes: Vec<Recipe>,
}

static RECIPES: Lazy<HashMap<String, Recipe>> = Lazy::new(|| {
    let data = include_str!("../../data/recipes.json");
    let file: RecipesFile = serde_json::from_str(data).expect("Failed to parse recipes.json");
    file.recipes
        .into_iter()
        .map(|r| (r.id.clone(), r))
        .collect()
});

pub fn get_recipe(id: &str) -> Option<&'static Recipe> {
    RECIPES.get(id)
}

pub fn all_recipe_ids() -> Vec<&'static str> {
    RECIPES.keys().map(|s| s.as_str()).collect()
}

/// Check if player has materials for a recipe
pub fn can_craft(recipe: &Recipe, inventory: &[String]) -> bool {
    for (item_id, &required) in &recipe.materials {
        let count = inventory.iter().filter(|id| *id == item_id).count() as u32;
        if count < required {
            return false;
        }
    }
    true
}

/// Check if player can craft recipe (materials + skill + station + faction)
pub fn can_craft_advanced(
    recipe: &Recipe,
    inventory: &[String],
    player_level: u32,
    available_stations: &[String],
    faction_reputation: &HashMap<String, i32>,
) -> bool {
    // Check materials
    if !can_craft(recipe, inventory) {
        return false;
    }

    // Check skill requirement (use player level as crafting skill)
    if player_level < recipe.skill_required {
        return false;
    }

    // Check crafting station
    if let Some(required_station) = &recipe.station_required {
        if !available_stations.contains(required_station) {
            return false;
        }
    }

    // Check faction requirement (need positive reputation)
    if let Some(required_faction) = &recipe.faction_required {
        if faction_reputation.get(required_faction).unwrap_or(&0) <= &0 {
            return false;
        }
    }

    true
}

/// Get all recipes player can currently craft
pub fn available_recipes(
    inventory: &[String],
    player_level: u32,
    available_stations: &[String],
    faction_reputation: &HashMap<String, i32>,
) -> Vec<&'static Recipe> {
    RECIPES
        .values()
        .filter(|recipe| {
            can_craft_advanced(
                recipe,
                inventory,
                player_level,
                available_stations,
                faction_reputation,
            )
        })
        .collect()
}

/// Calculate crafting success chance based on skill vs requirement
pub fn crafting_success_chance(player_level: u32, recipe_skill: u32) -> f32 {
    if player_level >= recipe_skill + 2 {
        1.0 // Guaranteed success
    } else if player_level >= recipe_skill {
        0.8 // High chance
    } else if player_level + 1 >= recipe_skill {
        0.6 // Medium chance
    } else {
        0.3 // Low chance
    }
}
