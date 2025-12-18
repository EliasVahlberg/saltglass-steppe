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
}

fn default_output_count() -> u32 { 1 }

#[derive(Deserialize)]
struct RecipesFile {
    recipes: Vec<Recipe>,
}

static RECIPES: Lazy<HashMap<String, Recipe>> = Lazy::new(|| {
    let data = include_str!("../../data/recipes.json");
    let file: RecipesFile = serde_json::from_str(data).expect("Failed to parse recipes.json");
    file.recipes.into_iter().map(|r| (r.id.clone(), r)).collect()
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

/// Get available recipes based on inventory
pub fn available_recipes(inventory: &[String]) -> Vec<&'static Recipe> {
    RECIPES.values().filter(|r| can_craft(r, inventory)).collect()
}
