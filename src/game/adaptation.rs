use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct AdaptationEffect {
    #[serde(rename = "type")]
    pub effect_type: String,
    #[serde(default)]
    pub value: Option<i32>,
    #[serde(default)]
    pub damage_source: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdaptationDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub effects: Vec<AdaptationEffect>,
}

#[derive(Deserialize)]
struct AdaptationsFile {
    adaptations: Vec<AdaptationDef>,
}

static ADAPTATION_DEFS: Lazy<HashMap<String, AdaptationDef>> = Lazy::new(|| {
    let data = include_str!("../../data/adaptations.json");
    let file: AdaptationsFile = serde_json::from_str(data).expect("Failed to parse adaptations.json");
    file.adaptations.into_iter().map(|d| (d.id.clone(), d)).collect()
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
}

impl Adaptation {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "prismhide" => Some(Self::Prismhide),
            "sunveins" => Some(Self::Sunveins),
            "mirage_step" => Some(Self::MirageStep),
            "saltblood" => Some(Self::Saltblood),
            _ => None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Prismhide => "prismhide",
            Self::Sunveins => "sunveins",
            Self::MirageStep => "mirage_step",
            Self::Saltblood => "saltblood",
        }
    }

    pub fn name(&self) -> &str {
        get_adaptation_def(self.id()).map(|d| d.name.as_str()).unwrap_or("Unknown")
    }

    pub fn description(&self) -> &str {
        get_adaptation_def(self.id()).map(|d| d.description.as_str()).unwrap_or("")
    }

    pub fn def(&self) -> Option<&'static AdaptationDef> {
        get_adaptation_def(self.id())
    }

    pub fn has_effect(&self, effect_type: &str) -> bool {
        self.def().map(|d| d.effects.iter().any(|e| e.effect_type == effect_type)).unwrap_or(false)
    }

    pub fn effect_value(&self, effect_type: &str) -> Option<i32> {
        self.def().and_then(|d| {
            d.effects.iter().find(|e| e.effect_type == effect_type).and_then(|e| e.value)
        })
    }

    pub fn has_immunity(&self, source: &str) -> bool {
        self.def().map(|d| {
            d.effects.iter().any(|e| e.effect_type == "immunity" && e.damage_source.as_deref() == Some(source))
        }).unwrap_or(false)
    }
}
