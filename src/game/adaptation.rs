use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct StatModifiers {
    #[serde(default)]
    pub armor: i32,
    #[serde(default)]
    pub damage_bonus: i32,
    #[serde(default)]
    pub reflex: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdaptationEffect {
    #[serde(rename = "type")]
    pub effect_type: String,
    #[serde(default)]
    pub value: Option<i32>,
    #[serde(default)]
    pub damage_source: Option<String>,
    #[serde(default)]
    pub ability: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdaptationDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub threshold: u32,
    #[serde(default)]
    pub stat_modifiers: StatModifiers,
    #[serde(default)]
    pub effects: Vec<AdaptationEffect>,
}

#[derive(Deserialize)]
struct AdaptationsFile {
    adaptations: Vec<AdaptationDef>,
}

static ADAPTATION_DEFS: Lazy<HashMap<String, AdaptationDef>> = Lazy::new(|| {
    let data = include_str!("../../data/adaptations.json");
    let file: AdaptationsFile =
        serde_json::from_str(data).expect("Failed to parse adaptations.json");
    file.adaptations
        .into_iter()
        .map(|d| (d.id.clone(), d))
        .collect()
});

pub fn get_adaptation_def(id: &str) -> Option<&'static AdaptationDef> {
    ADAPTATION_DEFS.get(id)
}

pub fn all_adaptation_ids() -> Vec<&'static str> {
    ADAPTATION_DEFS.keys().map(|s| s.as_str()).collect()
}

/// Legacy enum for backward compatibility with save files
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Adaptation {
    Prismhide,
    Sunveins,
    MirageStep,
    Saltblood,
    QuantumEntanglement,
    PhaseWalking,
    StormAffinity,
    CollectiveInterface,
    ArchiveResonance,
    CrystallineConsciousness,
}

impl Adaptation {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "prismhide" => Some(Self::Prismhide),
            "sunveins" => Some(Self::Sunveins),
            "mirage_step" => Some(Self::MirageStep),
            "saltblood" => Some(Self::Saltblood),
            "quantum_entanglement" => Some(Self::QuantumEntanglement),
            "phase_walking" => Some(Self::PhaseWalking),
            "storm_affinity" => Some(Self::StormAffinity),
            "collective_interface" => Some(Self::CollectiveInterface),
            "archive_resonance" => Some(Self::ArchiveResonance),
            "crystalline_consciousness" => Some(Self::CrystallineConsciousness),
            _ => None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Prismhide => "prismhide",
            Self::Sunveins => "sunveins",
            Self::MirageStep => "mirage_step",
            Self::Saltblood => "saltblood",
            Self::QuantumEntanglement => "quantum_entanglement",
            Self::PhaseWalking => "phase_walking",
            Self::StormAffinity => "storm_affinity",
            Self::CollectiveInterface => "collective_interface",
            Self::ArchiveResonance => "archive_resonance",
            Self::CrystallineConsciousness => "crystalline_consciousness",
        }
    }

    pub fn name(&self) -> &str {
        get_adaptation_def(self.id())
            .map(|d| d.name.as_str())
            .unwrap_or("Unknown")
    }

    pub fn description(&self) -> &str {
        get_adaptation_def(self.id())
            .map(|d| d.description.as_str())
            .unwrap_or("")
    }

    pub fn def(&self) -> Option<&'static AdaptationDef> {
        get_adaptation_def(self.id())
    }

    pub fn has_effect(&self, effect_type: &str) -> bool {
        self.def()
            .map(|d| d.effects.iter().any(|e| e.effect_type == effect_type))
            .unwrap_or(false)
    }

    pub fn effect_value(&self, effect_type: &str) -> Option<i32> {
        self.def().and_then(|d| {
            d.effects
                .iter()
                .find(|e| e.effect_type == effect_type)
                .and_then(|e| e.value)
        })
    }

    pub fn has_immunity(&self, source: &str) -> bool {
        self.def()
            .map(|d| {
                d.effects.iter().any(|e| {
                    e.effect_type == "immunity" && e.damage_source.as_deref() == Some(source)
                })
            })
            .unwrap_or(false)
    }

    pub fn has_ability(&self, ability: &str) -> bool {
        self.def()
            .map(|d| {
                d.effects.iter().any(|e| {
                    e.effect_type == "special_ability" && e.ability.as_deref() == Some(ability)
                })
            })
            .unwrap_or(false)
    }

    /// Get stat modifiers from this adaptation
    pub fn stat_modifiers(&self) -> &'static StatModifiers {
        static DEFAULT: StatModifiers = StatModifiers {
            armor: 0,
            damage_bonus: 0,
            reflex: 0,
        };
        self.def().map(|d| &d.stat_modifiers).unwrap_or(&DEFAULT)
    }
}

/// Calculate total stat modifiers from a list of adaptations
pub fn total_stat_modifiers(adaptations: &[Adaptation]) -> StatModifiers {
    let mut total = StatModifiers::default();
    for a in adaptations {
        let mods = a.stat_modifiers();
        total.armor += mods.armor;
        total.damage_bonus += mods.damage_bonus;
        total.reflex += mods.reflex;
    }
    total
}
