use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeTemplate {
    pub template: String,
    pub variables: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub template: NarrativeTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolktaleTemplate {
    pub title: String,
    #[serde(flatten)]
    pub template: NarrativeTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeTemplates {
    pub historical_events: HashMap<String, Vec<HistoricalEvent>>,
    pub location_descriptions: HashMap<String, Vec<NarrativeTemplate>>,
    pub contextual_descriptions: HashMap<String, HashMap<String, NarrativeTemplate>>,
    pub item_lore: HashMap<String, Vec<NarrativeTemplate>>,
    pub environmental_storytelling: HashMap<String, Vec<NarrativeTemplate>>,
    #[serde(default)]
    pub rumors: Vec<NarrativeTemplate>,
    #[serde(default)]
    pub folktales: Vec<FolktaleTemplate>,
    pub markov_corpus: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NarrativeContext {
    pub biome: Option<String>,
    pub terrain: Option<String>,
    pub adaptations: Vec<String>,
    pub faction_reputation: HashMap<String, i32>,
    pub refraction_level: u32,
    pub location_type: Option<String>,
}

pub struct NarrativeGenerator {
    templates: NarrativeTemplates,
    markov_chain: MarkovChain,
}

pub struct MarkovChain {
    chain: HashMap<(String, String), Vec<String>>,
    opening_words: Vec<(String, String)>,
}

impl MarkovChain {
    pub fn new(corpus: &[String]) -> Self {
        let mut chain = HashMap::new();
        let mut opening_words = Vec::new();

        for text in corpus {
            let words: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();

            if words.len() < 2 {
                continue;
            }

            // Add opening words
            opening_words.push((words[0].clone(), words[1].clone()));

            // Build chain
            for window in words.windows(3) {
                let key = (window[0].clone(), window[1].clone());
                let next = window[2].clone();

                chain.entry(key).or_insert_with(Vec::new).push(next);
            }
        }

        Self {
            chain,
            opening_words,
        }
    }

    pub fn generate(&self, rng: &mut ChaCha8Rng, max_words: usize) -> String {
        if self.opening_words.is_empty() {
            return String::new();
        }

        let start = self.opening_words.choose(rng).unwrap();
        let mut result = vec![start.0.clone(), start.1.clone()];
        let mut current = start.clone();

        for _ in 0..max_words {
            if let Some(next_words) = self.chain.get(&current) {
                if let Some(next) = next_words.choose(rng) {
                    result.push(next.clone());
                    current = (current.1, next.clone());

                    // Stop at sentence end
                    if next.ends_with('.') || next.ends_with('!') || next.ends_with('?') {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        result.join(" ")
    }
}

impl NarrativeGenerator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let templates_data = std::fs::read_to_string("data/narrative_templates.json")?;
        let templates: NarrativeTemplates = serde_json::from_str(&templates_data)?;
        let markov_chain = MarkovChain::new(&templates.markov_corpus);

        Ok(Self {
            templates,
            markov_chain,
        })
    }

    pub fn generate_historical_event(
        &self,
        event_category: &str,
        rng: &mut ChaCha8Rng,
    ) -> Option<String> {
        let events = self.templates.historical_events.get(event_category)?;
        let event = events.choose(rng)?;
        Some(self.fill_template(&event.template, rng))
    }

    pub fn generate_location_description(
        &self,
        location_type: &str,
        rng: &mut ChaCha8Rng,
    ) -> Option<String> {
        let descriptions = self.templates.location_descriptions.get(location_type)?;
        let template = descriptions.choose(rng)?;
        Some(self.fill_template(template, rng))
    }

    pub fn generate_contextual_description(
        &self,
        context: &NarrativeContext,
        rng: &mut ChaCha8Rng,
    ) -> Option<String> {
        // Try biome-specific descriptions first
        if let Some(ref biome) = context.biome {
            if let Some(biome_templates) =
                self.templates.contextual_descriptions.get("biome_specific")
            {
                if let Some(template) = biome_templates.get(biome) {
                    return Some(self.fill_template(template, rng));
                }
            }
        }

        // Try adaptation-aware descriptions
        if !context.adaptations.is_empty() {
            if let Some(adaptation_templates) = self
                .templates
                .contextual_descriptions
                .get("adaptation_aware")
            {
                for adaptation in &context.adaptations {
                    if let Some(template) = adaptation_templates.get(adaptation) {
                        return Some(self.fill_template(template, rng));
                    }
                }
            }
        }

        // Try faction-influenced descriptions
        if let Some(faction_templates) = self
            .templates
            .contextual_descriptions
            .get("faction_influenced")
        {
            let mut best_faction = None;
            let mut best_reputation = i32::MIN;

            for (faction, reputation) in &context.faction_reputation {
                if *reputation > best_reputation && *reputation > 20 {
                    best_reputation = *reputation;
                    best_faction = Some(faction);
                }
            }

            if let Some(faction) = best_faction {
                if let Some(template) = faction_templates.get(faction) {
                    return Some(self.fill_template(template, rng));
                }
            }
        }

        None
    }

    pub fn generate_item_lore(&self, item_category: &str, rng: &mut ChaCha8Rng) -> Option<String> {
        let lore_templates = self.templates.item_lore.get(item_category)?;
        let template = lore_templates.choose(rng)?;
        Some(self.fill_template(template, rng))
    }

    pub fn generate_contextual_item_lore(
        &self,
        item_category: &str,
        context: &NarrativeContext,
        rng: &mut ChaCha8Rng,
    ) -> Option<String> {
        // Try contextual item lore first
        if context.refraction_level > 50 {
            if let Some(contextual_templates) = self.templates.item_lore.get("contextual_artifacts")
            {
                if let Some(template) = contextual_templates.choose(rng) {
                    return Some(self.fill_template(template, rng));
                }
            }
        }

        // Fall back to regular item lore
        self.generate_item_lore(item_category, rng)
    }

    pub fn generate_environmental_text(
        &self,
        environment_type: &str,
        rng: &mut ChaCha8Rng,
    ) -> Option<String> {
        let env_templates = self
            .templates
            .environmental_storytelling
            .get(environment_type)?;
        let template = env_templates.choose(rng)?;
        Some(self.fill_template(template, rng))
    }

    pub fn generate_rumor(&self, rng: &mut ChaCha8Rng) -> Option<String> {
        if self.templates.rumors.is_empty() {
            return None;
        }
        let template = self.templates.rumors.choose(rng)?;
        Some(self.fill_template(template, rng))
    }

    pub fn generate_folktale(&self, rng: &mut ChaCha8Rng) -> Option<(String, String)> {
        if self.templates.folktales.is_empty() {
            return None;
        }
        let folktale = self.templates.folktales.choose(rng)?;
        let content = self.fill_template(&folktale.template, rng);
        Some((folktale.title.clone(), content))
    }

    pub fn generate_markov_text(&self, rng: &mut ChaCha8Rng, max_words: usize) -> String {
        self.markov_chain.generate(rng, max_words)
    }

    fn fill_template(&self, template: &NarrativeTemplate, rng: &mut ChaCha8Rng) -> String {
        let mut result = template.template.clone();

        for (var_name, options) in &template.variables {
            if let Some(choice) = options.choose(rng) {
                let placeholder = format!("{{{}}}", var_name);
                result = result.replace(&placeholder, choice);
            }
        }

        result
    }

    pub fn generate_world_history(&self, rng: &mut ChaCha8Rng, num_events: usize) -> Vec<String> {
        let mut history = Vec::new();
        let event_types = vec!["storm_events", "faction_events"];

        for _ in 0..num_events {
            if let Some(event_type) = event_types.choose(rng) {
                if let Some(event) = self.generate_historical_event(event_type, rng) {
                    history.push(event);
                }
            }
        }

        history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markov_chain() {
        let corpus = vec![
            "The glass storms reshape the world.".to_string(),
            "The world remembers ancient secrets.".to_string(),
            "Ancient secrets hide in glass.".to_string(),
            "Glass reflects the light.".to_string(),
            "The light bends through crystal.".to_string(),
        ];

        let chain = MarkovChain::new(&corpus);
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let text = chain.generate(&mut rng, 10);
        assert!(!text.is_empty());
        // Test that it generates some reasonable text
        assert!(text.len() > 5);
    }

    #[test]
    fn test_template_filling() {
        let template = NarrativeTemplate {
            template: "The {color} {object} {action}.".to_string(),
            variables: {
                let mut vars = HashMap::new();
                vars.insert(
                    "color".to_string(),
                    vec!["red".to_string(), "blue".to_string()],
                );
                vars.insert(
                    "object".to_string(),
                    vec!["crystal".to_string(), "mirror".to_string()],
                );
                vars.insert(
                    "action".to_string(),
                    vec!["glows".to_string(), "reflects".to_string()],
                );
                vars
            },
        };

        let generator = NarrativeGenerator {
            templates: NarrativeTemplates {
                historical_events: HashMap::new(),
                location_descriptions: HashMap::new(),
                contextual_descriptions: HashMap::new(),
                item_lore: HashMap::new(),
                environmental_storytelling: HashMap::new(),
                rumors: Vec::new(),
                folktales: Vec::new(),
                markov_corpus: Vec::new(),
            },
            markov_chain: MarkovChain::new(&[]),
        };

        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = generator.fill_template(&template, &mut rng);

        assert!(result.starts_with("The "));
        assert!(!result.contains("{"));
        assert!(!result.contains("}"));
    }

    #[test]
    fn test_contextual_generation() {
        let context = NarrativeContext {
            biome: Some("Saltflats".to_string()),
            terrain: None,
            adaptations: vec!["prismhide".to_string()],
            faction_reputation: {
                let mut rep = HashMap::new();
                rep.insert("Mirror Monks".to_string(), 50);
                rep
            },
            refraction_level: 75,
            location_type: None,
        };

        // Test that context is properly structured
        assert_eq!(context.biome, Some("Saltflats".to_string()));
        assert_eq!(context.adaptations.len(), 1);
        assert_eq!(context.refraction_level, 75);
    }
}
