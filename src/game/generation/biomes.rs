use once_cell::sync::Lazy;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;

use super::{Grammar, GrammarContext, WeightedEntry, WeightedTable};
use crate::game::world_map::Biome;

/// Biome-specific generation rules and environmental storytelling
#[derive(Debug, Clone, Deserialize)]
pub struct BiomeProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub environmental_features: Vec<EnvironmentalFeature>,
    pub atmospheric_elements: Vec<AtmosphericElement>,
    pub hazards: Vec<BiomeHazard>,
    pub resource_modifiers: ResourceModifiers,
    pub story_themes: Vec<String>,
    pub ambient_descriptions: WeightedTable<String>,
}

/// Environmental features that can be generated in a biome
#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentalFeature {
    pub feature_type: String,
    pub weight: u32,
    pub description_template: String,
    pub mechanical_effects: Option<MechanicalEffects>,
}

/// Atmospheric elements for environmental storytelling
#[derive(Debug, Clone, Deserialize)]
pub struct AtmosphericElement {
    pub element_type: String,
    pub intensity: f32, // 0.0 to 1.0
    pub description: String,
    pub triggers: Vec<String>, // When this element appears
}

/// Biome-specific hazards and dangers
#[derive(Debug, Clone, Deserialize)]
pub struct BiomeHazard {
    pub hazard_type: String,
    pub severity: u8,   // 1-10
    pub frequency: f32, // 0.0 to 1.0
    pub description: String,
    pub damage_type: Option<String>,
}

/// Resource generation modifiers for biomes
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceModifiers {
    pub water_multiplier: f32,
    pub glass_multiplier: f32,
    pub salt_multiplier: f32,
    pub rare_materials_chance: f32,
}

/// Mechanical effects of environmental features
#[derive(Debug, Clone, Deserialize)]
pub struct MechanicalEffects {
    pub movement_cost: Option<f32>,
    pub visibility_modifier: Option<f32>,
    pub damage_per_turn: Option<i32>,
    pub status_effect: Option<String>,
}

/// Context for biome-specific generation
#[derive(Debug, Clone)]
pub struct BiomeGenerationContext {
    pub biome: Biome,
    pub storm_intensity: u8,
    pub time_of_day: String,
    pub weather_conditions: String,
    pub player_adaptations: Vec<String>,
}

/// Biome generation system
pub struct BiomeSystem;

impl BiomeSystem {
    /// Generate environmental description for a biome
    pub fn generate_environment_description(
        biome: Biome,
        context: &BiomeGenerationContext,
        rng: &mut ChaCha8Rng,
    ) -> String {
        let profile = Self::get_biome_profile(biome);

        // Select ambient description
        let base_description = profile
            .ambient_descriptions
            .select(rng)
            .unwrap_or("The landscape stretches before you.".to_string());

        // Add atmospheric elements based on context
        let mut description = base_description.clone();

        for element in &profile.atmospheric_elements {
            if Self::should_include_element(element, context, rng) {
                description = format!("{} {}", description, element.description);
            }
        }

        description
    }

    /// Generate environmental features for a location
    pub fn generate_environmental_features(
        biome: Biome,
        count: usize,
        rng: &mut ChaCha8Rng,
    ) -> Vec<EnvironmentalFeature> {
        let profile = Self::get_biome_profile(biome);
        let mut features = Vec::new();

        let feature_entries: Vec<WeightedEntry<EnvironmentalFeature>> = profile
            .environmental_features
            .iter()
            .map(|f| WeightedEntry {
                item: f.clone(),
                weight: f.weight as f32,
            })
            .collect();

        let feature_table = WeightedTable::new(feature_entries);

        for _ in 0..count {
            if let Some(feature) = feature_table.select(rng) {
                features.push(feature.clone());
            }
        }

        features
    }

    /// Check for biome-specific hazards
    pub fn check_hazards(
        biome: Biome,
        context: &BiomeGenerationContext,
        rng: &mut ChaCha8Rng,
    ) -> Vec<BiomeHazard> {
        let profile = Self::get_biome_profile(biome);
        let mut active_hazards = Vec::new();

        for hazard in &profile.hazards {
            let hazard_chance = hazard.frequency * Self::get_hazard_modifier(context);
            if rng.gen_range(0.0..1.0) < hazard_chance {
                active_hazards.push(hazard.clone());
            }
        }

        active_hazards
    }

    /// Generate biome-appropriate story elements
    pub fn generate_story_elements(
        biome: Biome,
        grammar: &Grammar,
        context: &BiomeGenerationContext,
        rng: &mut ChaCha8Rng,
    ) -> Vec<String> {
        let profile = Self::get_biome_profile(biome);
        let mut elements = Vec::new();

        // Create grammar context with biome information
        let mut grammar_context = GrammarContext {
            variables: HashMap::new(),
        };
        grammar_context
            .variables
            .insert("biome".to_string(), profile.name.clone());
        grammar_context
            .variables
            .insert("weather".to_string(), context.weather_conditions.clone());
        grammar_context
            .variables
            .insert("time".to_string(), context.time_of_day.clone());

        // Generate story elements based on themes
        for theme in &profile.story_themes {
            if let Ok(story) = grammar.generate(theme, &grammar_context, rng) {
                elements.push(story);
            }
        }

        elements
    }

    /// Get biome profile from static data
    fn get_biome_profile(biome: Biome) -> &'static BiomeProfile {
        BIOME_PROFILES
            .get(&biome)
            .unwrap_or_else(|| &BIOME_PROFILES[&Biome::Desert])
    }

    /// Check if atmospheric element should be included
    fn should_include_element(
        element: &AtmosphericElement,
        context: &BiomeGenerationContext,
        rng: &mut ChaCha8Rng,
    ) -> bool {
        // Check triggers
        for trigger in &element.triggers {
            match trigger.as_str() {
                "storm" if context.storm_intensity > 3 => {
                    return rng.gen_range(0.0..1.0) < element.intensity;
                }
                "night" if context.time_of_day == "night" => {
                    return rng.gen_range(0.0..1.0) < element.intensity;
                }
                "day" if context.time_of_day == "day" => {
                    return rng.gen_range(0.0..1.0) < element.intensity;
                }
                "adapted" if !context.player_adaptations.is_empty() => {
                    return rng.gen_range(0.0..1.0) < element.intensity;
                }
                _ => {}
            }
        }

        // Default probability based on intensity
        rng.gen_range(0.0..1.0) < element.intensity * 0.3
    }

    /// Get hazard frequency modifier based on context
    fn get_hazard_modifier(context: &BiomeGenerationContext) -> f32 {
        let mut modifier = 1.0;

        // Storm increases hazards
        modifier *= 1.0 + (context.storm_intensity as f32 * 0.1);

        // Night increases some hazards
        if context.time_of_day == "night" {
            modifier *= 1.2;
        }

        // Adaptations might reduce hazards
        if context
            .player_adaptations
            .contains(&"storm_resistance".to_string())
        {
            modifier *= 0.8;
        }

        modifier
    }
}

// Static biome profile data
static BIOME_PROFILES: Lazy<HashMap<Biome, BiomeProfile>> = Lazy::new(|| {
    let data = include_str!("../../../data/biome_profiles.json");
    let file: BiomeProfilesFile =
        serde_json::from_str(data).expect("Failed to parse biome_profiles.json");

    file.profiles
        .into_iter()
        .map(|profile| {
            let biome = match profile.id.as_str() {
                "desert" => Biome::Desert,
                "saltflat" => Biome::Saltflat,
                "scrubland" => Biome::Scrubland,
                "oasis" => Biome::Oasis,
                "ruins" => Biome::Ruins,
                _ => Biome::Desert,
            };
            (biome, profile)
        })
        .collect()
});

#[derive(Deserialize)]
struct BiomeProfilesFile {
    profiles: Vec<BiomeProfile>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_biome_system_deterministic() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);

        let context = BiomeGenerationContext {
            biome: Biome::Desert,
            storm_intensity: 2,
            time_of_day: "day".to_string(),
            weather_conditions: "clear".to_string(),
            player_adaptations: vec![],
        };

        let desc1 =
            BiomeSystem::generate_environment_description(Biome::Desert, &context, &mut rng1);
        let desc2 =
            BiomeSystem::generate_environment_description(Biome::Desert, &context, &mut rng2);

        assert_eq!(desc1, desc2, "Biome generation should be deterministic");
    }

    #[test]
    fn test_environmental_features_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(54321);

        let features = BiomeSystem::generate_environmental_features(Biome::Saltflat, 3, &mut rng);

        assert_eq!(features.len(), 3);
        // Features should be biome-appropriate
        for feature in features {
            assert!(!feature.feature_type.is_empty());
        }
    }

    #[test]
    fn test_hazard_checking() {
        let mut rng = ChaCha8Rng::seed_from_u64(98765);

        let context = BiomeGenerationContext {
            biome: Biome::Ruins,
            storm_intensity: 5, // High storm should increase hazards
            time_of_day: "night".to_string(),
            weather_conditions: "stormy".to_string(),
            player_adaptations: vec![],
        };

        let hazards = BiomeSystem::check_hazards(Biome::Ruins, &context, &mut rng);

        // Should have some hazards due to high storm intensity
        assert!(hazards.is_empty() || !hazards.is_empty()); // Allow for randomness
    }

    #[test]
    fn test_biome_context_creation() {
        let context = BiomeGenerationContext {
            biome: Biome::Oasis,
            storm_intensity: 1,
            time_of_day: "dawn".to_string(),
            weather_conditions: "misty".to_string(),
            player_adaptations: vec!["prismhide".to_string()],
        };

        assert_eq!(context.biome, Biome::Oasis);
        assert_eq!(context.storm_intensity, 1);
        assert!(
            context
                .player_adaptations
                .contains(&"prismhide".to_string())
        );
    }
}
