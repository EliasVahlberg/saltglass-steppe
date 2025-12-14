use serde::{Deserialize, Serialize};
use super::adaptation::Adaptation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    MirrorMonks,
    SandEngineers,
    Glassborn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub x: i32,
    pub y: i32,
    pub faction: Faction,
    pub talked: bool,
}

impl Npc {
    pub fn new(x: i32, y: i32, faction: Faction) -> Self {
        Self { x, y, faction, talked: false }
    }

    pub fn glyph(&self) -> char {
        match self.faction {
            Faction::MirrorMonks => 'M',
            Faction::SandEngineers => 'E',
            Faction::Glassborn => 'G',
        }
    }

    pub fn name(&self) -> &'static str {
        match self.faction {
            Faction::MirrorMonks => "Mirror Monk",
            Faction::SandEngineers => "Sand-Engineer",
            Faction::Glassborn => "Glassborn",
        }
    }

    pub fn dialogue(&self, adaptations: &[Adaptation]) -> &'static str {
        match self.faction {
            Faction::MirrorMonks => {
                if adaptations.contains(&Adaptation::Prismhide) {
                    "Your skin refracts. The angle has chosen you."
                } else if adaptations.contains(&Adaptation::Sunveins) {
                    "Light burns in your veins. You carry the storm's fire."
                } else if !adaptations.is_empty() {
                    "The storm speaks through your flesh. Listen."
                } else {
                    "You walk unmarked. The storm has not yet spoken to you."
                }
            }
            Faction::SandEngineers => {
                if adaptations.contains(&Adaptation::Saltblood) {
                    "Saltblood, eh? Useful. Glass won't cut you."
                } else if !adaptations.is_empty() {
                    "Adapted, I see. Stay useful, stay alive."
                } else {
                    "Fresh meat. Watch where you step—glass cuts deep."
                }
            }
            Faction::Glassborn => {
                if adaptations.len() >= 2 {
                    "You begin to understand. The shimmer is not pain—it is becoming."
                } else if !adaptations.is_empty() {
                    "You flinch at the shimmer. We were born in it."
                } else {
                    "Unmarked flesh. The storm will teach you, or break you."
                }
            }
        }
    }
}
