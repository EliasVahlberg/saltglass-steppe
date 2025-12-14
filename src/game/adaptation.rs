use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Adaptation {
    Prismhide,
    Sunveins,
    MirageStep,
    Saltblood,
}

impl Adaptation {
    pub fn name(&self) -> &str {
        match self {
            Self::Prismhide => "Prismhide",
            Self::Sunveins => "Sunveins",
            Self::MirageStep => "Mirage Step",
            Self::Saltblood => "Saltblood",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Prismhide => "Crystalline skin reduces damage",
            Self::Sunveins => "Store light charge, +2 attack damage",
            Self::MirageStep => "Leave decoy when moving (confuses enemies)",
            Self::Saltblood => "Immune to glass terrain damage",
        }
    }
}
