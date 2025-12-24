use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

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
pub struct NarrativeTemplates {
    pub historical_events: HashMap<String, Vec<HistoricalEvent>>,
    pub location_descriptions: HashMap<String, Vec<NarrativeTemplate>>,
    pub item_lore: HashMap<String, Vec<NarrativeTemplate>>,
    pub markov_corpus: Vec<String>,
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
            let words: Vec<String> = text.split_whitespace()
                .map(|s| s.to_string())
                .collect();
            
            if words.len() < 2 { continue; }
            
            // Add opening words
            opening_words.push((words[0].clone(), words[1].clone()));
            
            // Build chain
            for window in words.windows(3) {
                let key = (window[0].clone(), window[1].clone());
                let next = window[2].clone();
                
                chain.entry(key).or_insert_with(Vec::new).push(next);
            }
        }
        
        Self { chain, opening_words }
    }
    
    pub fn generate(&self, rng: &mut ChaCha8Rng, max_words: usize) -> String {
        if self.opening_words.is_empty() { return String::new(); }
        
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
        
        Ok(Self { templates, markov_chain })
    }
    
    pub fn generate_historical_event(&self, event_category: &str, rng: &mut ChaCha8Rng) -> Option<String> {
        let events = self.templates.historical_events.get(event_category)?;
        let event = events.choose(rng)?;
        Some(self.fill_template(&event.template, rng))
    }
    
    pub fn generate_location_description(&self, location_type: &str, rng: &mut ChaCha8Rng) -> Option<String> {
        let descriptions = self.templates.location_descriptions.get(location_type)?;
        let template = descriptions.choose(rng)?;
        Some(self.fill_template(template, rng))
    }
    
    pub fn generate_item_lore(&self, item_category: &str, rng: &mut ChaCha8Rng) -> Option<String> {
        let lore_templates = self.templates.item_lore.get(item_category)?;
        let template = lore_templates.choose(rng)?;
        Some(self.fill_template(template, rng))
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
                vars.insert("color".to_string(), vec!["red".to_string(), "blue".to_string()]);
                vars.insert("object".to_string(), vec!["crystal".to_string(), "mirror".to_string()]);
                vars.insert("action".to_string(), vec!["glows".to_string(), "reflects".to_string()]);
                vars
            },
        };
        
        let generator = NarrativeGenerator {
            templates: NarrativeTemplates {
                historical_events: HashMap::new(),
                location_descriptions: HashMap::new(),
                item_lore: HashMap::new(),
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
}
