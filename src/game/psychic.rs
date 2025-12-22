//! Quantum Consciousness / Psychic System

use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PsychicCategory {
    Telepathy,
    Probability,
    Energy,
    Phasing,
    Temporal,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PsychicAbilityDef {
    pub id: String,
    pub name: String,
    pub category: PsychicCategory,
    pub description: String,
    pub coherence_cost: u32,
    pub cooldown: u32,
    pub min_adaptation_level: u32,
    pub effect: String, // Effect ID or script
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PsychicState {
    pub coherence: u32,
    pub max_coherence: u32,
    pub unlocked_abilities: Vec<String>,
    pub cooldowns: HashMap<String, u32>,
    pub active_effects: Vec<ActivePsychicEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivePsychicEffect {
    pub id: String,
    pub duration: u32,
}

impl Default for PsychicState {
    fn default() -> Self {
        Self {
            coherence: 100,
            max_coherence: 100,
            unlocked_abilities: Vec::new(),
            cooldowns: HashMap::new(),
            active_effects: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
struct AbilitiesFile {
    abilities: Vec<PsychicAbilityDef>,
}

static ABILITIES: Lazy<HashMap<String, PsychicAbilityDef>> = Lazy::new(|| {
    // We'll use a default empty list if file doesn't exist yet
    // But ideally we should create the file
    match include_str!("../../data/psychic_abilities.json") {
        data => {
            let file: AbilitiesFile = serde_json::from_str(data).expect("Failed to parse psychic_abilities.json");
            file.abilities.into_iter().map(|a| (a.id.clone(), a)).collect()
        }
    }
});

pub fn get_ability_def(id: &str) -> Option<&'static PsychicAbilityDef> {
    ABILITIES.get(id)
}

pub fn all_ability_ids() -> Vec<&'static str> {
    ABILITIES.keys().map(|s| s.as_str()).collect()
}

impl PsychicState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        // Regenerate coherence slowly
        if self.coherence < self.max_coherence {
            self.coherence += 1;
        }

        // Tick cooldowns
        let mut finished_cooldowns = Vec::new();
        for (id, cd) in self.cooldowns.iter_mut() {
            if *cd > 0 {
                *cd -= 1;
            }
            if *cd == 0 {
                finished_cooldowns.push(id.clone());
            }
        }
        for id in finished_cooldowns {
            self.cooldowns.remove(&id);
        }

        // Tick active effects
        self.active_effects.retain_mut(|e| {
            if e.duration > 0 {
                e.duration -= 1;
                true
            } else {
                false
            }
        });
    }

    pub fn can_use_ability(&self, ability_id: &str) -> Result<(), String> {
        if !self.unlocked_abilities.contains(&ability_id.to_string()) {
            return Err("Ability not unlocked".to_string());
        }
        
        if self.cooldowns.contains_key(ability_id) {
            return Err("Ability on cooldown".to_string());
        }

        let def = get_ability_def(ability_id).ok_or("Unknown ability")?;
        
        if self.coherence < def.coherence_cost {
            return Err("Insufficient coherence".to_string());
        }

        Ok(())
    }

    pub fn use_ability(&mut self, ability_id: &str) -> Result<String, String> {
        self.can_use_ability(ability_id)?;
        
        let def = get_ability_def(ability_id).unwrap();
        self.coherence -= def.coherence_cost;
        self.cooldowns.insert(ability_id.to_string(), def.cooldown);
        
        // Effect application should be handled by GameState, but we can return the effect ID
        Ok(def.effect.clone())
    }
}
