use rand_chacha::ChaCha8Rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Event trigger conditions based on player/world state
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventTrigger {
    pub trigger_type: String,
    pub conditions: HashMap<String, serde_json::Value>,
    pub probability: f32,
}

/// Event consequence that modifies game state
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventConsequence {
    pub consequence_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Event chain linking multiple events
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventChain {
    pub chain_id: String,
    pub events: Vec<String>,
    pub delay_turns: Vec<u32>,
}

/// Dynamic event definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub triggers: Vec<EventTrigger>,
    pub consequences: Vec<EventConsequence>,
    pub chains_to: Option<String>,
    pub weight: f32,
    pub cooldown_turns: u32,
}

/// Event context for trigger evaluation
#[derive(Debug, Clone)]
pub struct EventContext {
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub player_x: i32,
    pub player_y: i32,
    pub turn: u32,
    pub biome: String,
    pub storm_intensity: u8,
    pub refraction_level: u32,
    pub variables: HashMap<String, serde_json::Value>,
}

/// Event system for managing dynamic events
pub struct EventSystem {
    events: HashMap<String, DynamicEvent>,
    chains: HashMap<String, EventChain>,
    #[allow(dead_code)]
    active_events: Vec<String>,
    event_cooldowns: HashMap<String, u32>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            events: EVENTS.clone(),
            chains: CHAINS.clone(),
            active_events: Vec::new(),
            event_cooldowns: HashMap::new(),
        }
    }

    /// Check for triggered events based on context
    pub fn check_triggers(&mut self, context: &EventContext, rng: &mut ChaCha8Rng) -> Vec<String> {
        let mut triggered_events = Vec::new();

        for (event_id, event) in &self.events {
            // Skip if on cooldown
            if let Some(&cooldown) = self.event_cooldowns.get(event_id) {
                if cooldown > context.turn {
                    continue;
                }
            }

            // Check all triggers
            let mut all_triggers_met = true;
            for trigger in &event.triggers {
                if !self.evaluate_trigger(trigger, context, rng) {
                    all_triggers_met = false;
                    break;
                }
            }

            if all_triggers_met {
                triggered_events.push(event_id.clone());
                // Set cooldown
                self.event_cooldowns.insert(
                    event_id.clone(),
                    context.turn + event.cooldown_turns,
                );
            }
        }

        triggered_events
    }

    /// Evaluate a single trigger condition
    fn evaluate_trigger(&self, trigger: &EventTrigger, context: &EventContext, rng: &mut ChaCha8Rng) -> bool {
        // Check probability first
        if rng.gen_range(0.0..1.0) > trigger.probability {
            return false;
        }

        match trigger.trigger_type.as_str() {
            "player_hp_below" => {
                if let Some(threshold) = trigger.conditions.get("threshold") {
                    if let Some(threshold) = threshold.as_f64() {
                        return (context.player_hp as f64 / context.player_max_hp as f64) < threshold;
                    }
                }
            }
            "biome_match" => {
                if let Some(biome) = trigger.conditions.get("biome") {
                    if let Some(biome) = biome.as_str() {
                        return context.biome == biome;
                    }
                }
            }
            "storm_intensity" => {
                if let Some(min_intensity) = trigger.conditions.get("min_intensity") {
                    if let Some(min_intensity) = min_intensity.as_u64() {
                        return context.storm_intensity >= min_intensity as u8;
                    }
                }
            }
            "turn_multiple" => {
                if let Some(multiple) = trigger.conditions.get("multiple") {
                    if let Some(multiple) = multiple.as_u64() {
                        return context.turn % multiple as u32 == 0;
                    }
                }
            }
            "refraction_level" => {
                if let Some(min_level) = trigger.conditions.get("min_level") {
                    if let Some(min_level) = min_level.as_u64() {
                        return context.refraction_level >= min_level as u32;
                    }
                }
            }
            _ => return false,
        }

        false
    }

    /// Apply event consequences
    pub fn apply_consequences(&self, event_id: &str, context: &mut EventContext) -> Vec<String> {
        let mut messages = Vec::new();

        if let Some(event) = self.events.get(event_id) {
            for consequence in &event.consequences {
                match consequence.consequence_type.as_str() {
                    "damage_player" => {
                        if let Some(amount) = consequence.parameters.get("amount") {
                            if let Some(amount) = amount.as_i64() {
                                context.variables.insert(
                                    "damage_taken".to_string(),
                                    serde_json::Value::Number(serde_json::Number::from(amount)),
                                );
                                messages.push(format!("You take {} damage from {}", amount, event.name));
                            }
                        }
                    }
                    "heal_player" => {
                        if let Some(amount) = consequence.parameters.get("amount") {
                            if let Some(amount) = amount.as_i64() {
                                context.variables.insert(
                                    "healing_received".to_string(),
                                    serde_json::Value::Number(serde_json::Number::from(amount)),
                                );
                                messages.push(format!("You recover {} health from {}", amount, event.name));
                            }
                        }
                    }
                    "add_refraction" => {
                        if let Some(amount) = consequence.parameters.get("amount") {
                            if let Some(amount) = amount.as_u64() {
                                context.variables.insert(
                                    "refraction_gained".to_string(),
                                    serde_json::Value::Number(serde_json::Number::from(amount)),
                                );
                                messages.push(format!("Glass energy courses through you (+{} Refraction)", amount));
                            }
                        }
                    }
                    "environmental_story" => {
                        if let Some(message) = consequence.parameters.get("message") {
                            if let Some(message) = message.as_str() {
                                messages.push(message.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        messages
    }

    /// Get event chains triggered by an event
    pub fn get_event_chains(&self, event_id: &str) -> Option<&EventChain> {
        if let Some(event) = self.events.get(event_id) {
            if let Some(chain_id) = &event.chains_to {
                return self.chains.get(chain_id);
            }
        }
        None
    }

    /// Get event definition by ID
    pub fn get_event(&self, event_id: &str) -> Option<&DynamicEvent> {
        self.events.get(event_id)
    }

    /// Get number of loaded events (for testing)
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Event definitions loaded from JSON
#[derive(Deserialize)]
struct EventsFile {
    events: Vec<DynamicEvent>,
    chains: Vec<EventChain>,
}

static EVENTS: Lazy<HashMap<String, DynamicEvent>> = Lazy::new(|| {
    let data = include_str!("../../../data/dynamic_events.json");
    let file: EventsFile = serde_json::from_str(data).expect("Failed to parse dynamic_events.json");
    file.events.into_iter().map(|e| (e.id.clone(), e)).collect()
});

static CHAINS: Lazy<HashMap<String, EventChain>> = Lazy::new(|| {
    let data = include_str!("../../../data/dynamic_events.json");
    let file: EventsFile = serde_json::from_str(data).expect("Failed to parse dynamic_events.json");
    file.chains.into_iter().map(|c| (c.chain_id.clone(), c)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn create_test_context() -> EventContext {
        EventContext {
            player_hp: 50,
            player_max_hp: 100,
            player_x: 10,
            player_y: 10,
            turn: 100,
            biome: "desert".to_string(),
            storm_intensity: 3,
            refraction_level: 25,
            variables: HashMap::new(),
        }
    }

    #[test]
    fn test_event_system_creation() {
        let system = EventSystem::new();
        assert!(!system.events.is_empty());
    }

    #[test]
    fn test_hp_trigger_evaluation() {
        let system = EventSystem::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        let trigger = EventTrigger {
            trigger_type: "player_hp_below".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("threshold".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.6).unwrap()));
                map
            },
            probability: 1.0,
        };

        let context = create_test_context(); // 50/100 = 0.5 < 0.6
        assert!(system.evaluate_trigger(&trigger, &context, &mut rng));
    }

    #[test]
    fn test_biome_trigger_evaluation() {
        let system = EventSystem::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        let trigger = EventTrigger {
            trigger_type: "biome_match".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("biome".to_string(), serde_json::Value::String("desert".to_string()));
                map
            },
            probability: 1.0,
        };

        let context = create_test_context();
        assert!(system.evaluate_trigger(&trigger, &context, &mut rng));
    }

    #[test]
    fn test_storm_intensity_trigger() {
        let system = EventSystem::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        let trigger = EventTrigger {
            trigger_type: "storm_intensity".to_string(),
            conditions: {
                let mut map = HashMap::new();
                map.insert("min_intensity".to_string(), serde_json::Value::Number(serde_json::Number::from(2)));
                map
            },
            probability: 1.0,
        };

        let context = create_test_context(); // storm_intensity = 3 >= 2
        assert!(system.evaluate_trigger(&trigger, &context, &mut rng));
    }

    #[test]
    fn test_event_consequences() {
        let system = EventSystem::new();
        let mut context = create_test_context();

        // Create a test event
        let event = DynamicEvent {
            id: "test_event".to_string(),
            name: "Test Event".to_string(),
            description: "A test event".to_string(),
            triggers: vec![],
            consequences: vec![
                EventConsequence {
                    consequence_type: "damage_player".to_string(),
                    parameters: {
                        let mut map = HashMap::new();
                        map.insert("amount".to_string(), serde_json::Value::Number(serde_json::Number::from(10)));
                        map
                    },
                },
                EventConsequence {
                    consequence_type: "environmental_story".to_string(),
                    parameters: {
                        let mut map = HashMap::new();
                        map.insert("message".to_string(), serde_json::Value::String("The glass shimmers ominously.".to_string()));
                        map
                    },
                },
            ],
            chains_to: None,
            weight: 1.0,
            cooldown_turns: 10,
        };

        // Temporarily add event to system for testing
        let mut test_system = EventSystem::new();
        test_system.events.insert("test_event".to_string(), event);

        let messages = test_system.apply_consequences("test_event", &mut context);
        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("10 damage"));
        assert!(messages[1].contains("glass shimmers"));
        assert!(context.variables.contains_key("damage_taken"));
    }

    #[test]
    fn test_event_cooldown() {
        let mut system = EventSystem::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let context = create_test_context();

        // Set a cooldown
        system.event_cooldowns.insert("test_event".to_string(), context.turn + 5);

        // Create a test event that would normally trigger
        let event = DynamicEvent {
            id: "test_event".to_string(),
            name: "Test Event".to_string(),
            description: "A test event".to_string(),
            triggers: vec![EventTrigger {
                trigger_type: "biome_match".to_string(),
                conditions: {
                    let mut map = HashMap::new();
                    map.insert("biome".to_string(), serde_json::Value::String("desert".to_string()));
                    map
                },
                probability: 1.0,
            }],
            consequences: vec![],
            chains_to: None,
            weight: 1.0,
            cooldown_turns: 10,
        };

        system.events.insert("test_event".to_string(), event);

        let triggered = system.check_triggers(&context, &mut rng);
        assert!(!triggered.contains(&"test_event".to_string()));
    }
}
