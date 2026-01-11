use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryModel {
    pub events: Vec<StoryEvent>,
    pub characters: HashMap<String, StoryCharacter>,
    pub relationships: HashMap<String, HashMap<String, Relationship>>,
    pub faction_dynamics: HashMap<String, FactionState>,
    pub artifacts: HashMap<String, ArtifactStory>,
    pub locations: HashMap<String, LocationStory>,
    pub current_age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryEvent {
    pub id: String,
    pub age: u32,
    pub event_type: EventType,
    pub participants: Vec<String>,
    pub location: Option<String>,
    pub description: String,
    pub consequences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Birth,
    Death,
    Discovery,
    Conflict,
    Alliance,
    Betrayal,
    Transformation,
    StormEvent,
    ArtifactCreation,
    LocationFounding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryCharacter {
    pub id: String,
    pub name: String,
    pub faction: String,
    pub birth_age: u32,
    pub death_age: Option<u32>,
    pub traits: Vec<String>,
    pub achievements: Vec<String>,
    pub relationships: Vec<String>,
    pub adaptations: Vec<String>,
    pub reputation: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relationship_type: RelationshipType,
    pub strength: i32, // -100 to 100
    pub history: Vec<String>,
    pub formed_age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Ally,
    Enemy,
    Mentor,
    Student,
    Rival,
    Friend,
    Lover,
    Family,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionState {
    pub name: String,
    pub power_level: i32,
    pub territory: Vec<String>,
    pub leaders: Vec<String>,
    pub ideology: String,
    pub recent_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStory {
    pub name: String,
    pub creator: Option<String>,
    pub creation_age: u32,
    pub creation_event: String,
    pub owners: Vec<String>,
    pub powers: Vec<String>,
    pub current_location: Option<String>,
    pub legend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationStory {
    pub name: String,
    pub founder: Option<String>,
    pub founding_age: u32,
    pub founding_event: String,
    pub significant_events: Vec<String>,
    pub current_state: String,
    pub inhabitants: Vec<String>,
}

impl StoryModel {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut model = Self {
            events: Vec::new(),
            characters: HashMap::new(),
            relationships: HashMap::new(),
            faction_dynamics: HashMap::new(),
            artifacts: HashMap::new(),
            locations: HashMap::new(),
            current_age: 0,
        };

        model.initialize_factions(&mut rng);
        model.generate_founding_characters(&mut rng);
        model.generate_initial_events(&mut rng);

        model
    }

    fn initialize_factions(&mut self, rng: &mut ChaCha8Rng) {
        let factions = vec![
            ("Mirror Monks", "Seek enlightenment through refraction", 70),
            ("Sand-Engineers", "Rebuild civilization with technology", 60),
            ("Glassborn Collective", "Embrace storm transformation", 50),
            ("Salt Hermits", "Preserve ancient knowledge", 40),
            ("Archive Remnants", "Maintain pre-storm protocols", 30),
        ];

        for (name, ideology, power) in factions {
            self.faction_dynamics.insert(
                name.to_string(),
                FactionState {
                    name: name.to_string(),
                    power_level: power + rng.gen_range(-10..=10),
                    territory: Vec::new(),
                    leaders: Vec::new(),
                    ideology: ideology.to_string(),
                    recent_actions: Vec::new(),
                },
            );
        }
    }

    fn generate_founding_characters(&mut self, rng: &mut ChaCha8Rng) {
        let names = vec![
            "Saint Vex",
            "Keth the Seeker",
            "Naia Glassborn",
            "The Salt Prophet",
            "Archive Keeper Zara",
        ];
        let factions: Vec<String> = self.faction_dynamics.keys().cloned().collect();

        for (i, name) in names.iter().enumerate() {
            let faction = factions[i % factions.len()].clone();
            let character = StoryCharacter {
                id: format!("founder_{}", i),
                name: name.to_string(),
                faction: faction.clone(),
                birth_age: rng.gen_range(0..50),
                death_age: None,
                traits: vec!["Legendary".to_string(), "Founder".to_string()],
                achievements: Vec::new(),
                relationships: Vec::new(),
                adaptations: Vec::new(),
                reputation: rng.gen_range(60..100),
            };

            self.characters.insert(character.id.clone(), character);
            if let Some(faction_state) = self.faction_dynamics.get_mut(&faction) {
                faction_state.leaders.push(format!("founder_{}", i));
            }
        }
    }

    fn generate_initial_events(&mut self, rng: &mut ChaCha8Rng) {
        for i in 0..8 {
            let event_type = match rng.gen_range(0..4) {
                0 => EventType::Discovery,
                1 => EventType::StormEvent,
                2 => EventType::Conflict,
                _ => EventType::Alliance,
            };

            let char_keys: Vec<String> = self.characters.keys().cloned().collect();
            let num_participants = rng.gen_range(1..=3.min(char_keys.len()));
            let participants: Vec<String> = char_keys
                .choose_multiple(rng, num_participants)
                .cloned()
                .collect();

            let event = StoryEvent {
                id: format!("event_{}", i),
                age: rng.gen_range(0..100),
                event_type,
                participants,
                location: None,
                description: self.generate_event_description(rng),
                consequences: vec![self.generate_consequence(rng)],
            };

            self.events.push(event);
        }

        self.events.sort_by_key(|e| e.age);
    }

    fn generate_event_description(&self, rng: &mut ChaCha8Rng) -> String {
        let templates = vec![
            "A great storm revealed ancient secrets",
            "The factions clashed over sacred territory",
            "A powerful artifact was discovered",
            "An alliance was forged in desperation",
            "The glass sang with new harmonies",
        ];
        templates.choose(rng).unwrap().to_string()
    }

    fn generate_consequence(&self, rng: &mut ChaCha8Rng) -> String {
        let consequences = vec![
            "The balance of power shifted",
            "New adaptations emerged",
            "Ancient knowledge was lost",
            "The storms grew stronger",
            "Relationships were forever changed",
        ];
        consequences.choose(rng).unwrap().to_string()
    }

    pub fn add_player_event(&mut self, event_type: EventType, description: String) {
        let event = StoryEvent {
            id: format!("player_event_{}", self.events.len()),
            age: self.current_age,
            event_type,
            participants: vec!["player".to_string()],
            location: None,
            description,
            consequences: Vec::new(),
        };

        self.events.push(event);
    }

    pub fn get_artifact_inscription(&self, artifact_name: &str) -> Option<String> {
        if let Some(artifact) = self.artifacts.get(artifact_name) {
            Some(format!(
                "'{}' - {}",
                artifact.legend,
                artifact.creator.as_ref().unwrap_or(&"Unknown".to_string())
            ))
        } else {
            None
        }
    }

    pub fn get_shrine_text(&self, location: &str) -> Option<String> {
        // Find events that happened at this location
        let local_events: Vec<&StoryEvent> = self
            .events
            .iter()
            .filter(|e| e.location.as_ref() == Some(&location.to_string()))
            .collect();

        if let Some(event) = local_events.first() {
            Some(format!("Here {}", event.description.to_lowercase()))
        } else {
            None
        }
    }

    pub fn get_character_relationships(&self, character_id: &str) -> Vec<String> {
        if let Some(char_relationships) = self.relationships.get(character_id) {
            char_relationships
                .iter()
                .map(|(other_id, rel)| {
                    let other_name = self
                        .characters
                        .get(other_id)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| other_id.clone());
                    format!(
                        "{:?} of {} (strength: {})",
                        rel.relationship_type, other_name, rel.strength
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_faction_lore(&self, faction_name: &str) -> Option<String> {
        if let Some(faction) = self.faction_dynamics.get(faction_name) {
            let default_action = "maintains their ancient ways".to_string();
            let recent_action = faction.recent_actions.last().unwrap_or(&default_action);
            Some(format!(
                "The {} currently {}. Their ideology: {}",
                faction.name, recent_action, faction.ideology
            ))
        } else {
            None
        }
    }

    pub fn advance_age(&mut self) {
        self.current_age += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_model_creation() {
        let model = StoryModel::new(12345);
        assert!(!model.characters.is_empty());
        assert!(!model.faction_dynamics.is_empty());
        assert!(!model.events.is_empty());
    }

    #[test]
    fn test_faction_lore() {
        let model = StoryModel::new(12345);
        let lore = model.get_faction_lore("Mirror Monks");
        assert!(lore.is_some());
        assert!(lore.unwrap().contains("Mirror Monks"));
    }
}
