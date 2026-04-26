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

/// Which playstyle category an adaptation belongs to.
/// Used for activity-weighted pool selection.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AdaptationCategory {
    Survival,
    Predator,
    Precision,
    Artificer,
}

impl AdaptationCategory {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Survival  => "Survival",
            Self::Predator  => "Predator",
            Self::Precision => "Precision",
            Self::Artificer => "Artificer",
        }
    }
}

/// How visible the adaptation is to NPCs and factions.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FactionVisibility {
    Low,
    Moderate,
    High,
    Extreme,
}

impl FactionVisibility {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Low      => "Low",
            Self::Moderate => "Moderate",
            Self::High     => "High",
            Self::Extreme  => "Extreme",
        }
    }
}

/// Condition that must be met for this adaptation to appear in the choice pool.
#[derive(Clone, Debug, Deserialize, Default)]
pub struct UnlockCondition {
    #[serde(default)]
    pub storms_survived: u32,
    #[serde(default)]
    pub enemies_killed_melee: u32,
    #[serde(default)]
    pub enemies_killed_ranged: u32,
    #[serde(default)]
    pub elite_enemies_killed: u32,
    #[serde(default)]
    pub items_crafted: u32,
    #[serde(default)]
    pub psychic_uses: u32,
    #[serde(default)]
    pub tiles_explored: u32,
}

impl UnlockCondition {
    pub fn is_met(&self, activity: &crate::game::player_state::ActivityCounters) -> bool {
        activity.storms_survived      >= self.storms_survived
            && activity.enemies_killed_melee  >= self.enemies_killed_melee
            && activity.enemies_killed_ranged >= self.enemies_killed_ranged
            && activity.elite_enemies_killed  >= self.elite_enemies_killed
            && activity.items_crafted         >= self.items_crafted
            && activity.psychic_uses          >= self.psychic_uses
            && activity.tiles_explored        >= self.tiles_explored
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdaptationDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Refraction threshold tier (1, 2, or 3). Replaces the old flat `threshold` field.
    #[serde(default = "default_tier")]
    pub tier: u8,
    /// Legacy flat threshold — kept for save compat, ignored by new system.
    #[serde(default)]
    pub threshold: u32,
    #[serde(default)]
    pub category: Option<AdaptationCategory>,
    #[serde(default)]
    pub faction_visibility: Option<FactionVisibility>,
    #[serde(default)]
    pub unlock_condition: UnlockCondition,
    #[serde(default)]
    pub stat_modifiers: StatModifiers,
    #[serde(default)]
    pub effects: Vec<AdaptationEffect>,
}

fn default_tier() -> u8 { 1 }

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
    // Current adaptations
    Prismhide,
    Sunveins,
    MirageStep,
    Saltblood,
    StormDrinker,
    KillingEdge,
    LensEye,
    SaltSense,
    ScarLattice,
    BoneSpur,
    // Legacy variants — kept for save file compatibility, no longer in data
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
            "prismhide"                  => Some(Self::Prismhide),
            "sunveins"                   => Some(Self::Sunveins),
            "mirage_step"                => Some(Self::MirageStep),
            "saltblood"                  => Some(Self::Saltblood),
            "storm_drinker"              => Some(Self::StormDrinker),
            "killing_edge"               => Some(Self::KillingEdge),
            "lens_eye"                   => Some(Self::LensEye),
            "salt_sense"                 => Some(Self::SaltSense),
            "scar_lattice"               => Some(Self::ScarLattice),
            "bone_spur"                  => Some(Self::BoneSpur),
            "quantum_entanglement"       => Some(Self::QuantumEntanglement),
            "phase_walking"              => Some(Self::PhaseWalking),
            "storm_affinity"             => Some(Self::StormAffinity),
            "collective_interface"       => Some(Self::CollectiveInterface),
            "archive_resonance"          => Some(Self::ArchiveResonance),
            "crystalline_consciousness"  => Some(Self::CrystallineConsciousness),
            _ => None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Prismhide                => "prismhide",
            Self::Sunveins                 => "sunveins",
            Self::MirageStep               => "mirage_step",
            Self::Saltblood                => "saltblood",
            Self::StormDrinker             => "storm_drinker",
            Self::KillingEdge              => "killing_edge",
            Self::LensEye                  => "lens_eye",
            Self::SaltSense                => "salt_sense",
            Self::ScarLattice              => "scar_lattice",
            Self::BoneSpur                 => "bone_spur",
            Self::QuantumEntanglement      => "quantum_entanglement",
            Self::PhaseWalking             => "phase_walking",
            Self::StormAffinity            => "storm_affinity",
            Self::CollectiveInterface      => "collective_interface",
            Self::ArchiveResonance         => "archive_resonance",
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

/// Apply faction reputation multipliers when an adaptation is gained.
/// Mirror Monks revere visible mutations; Salt Traders distrust them.
pub fn apply_adaptation_faction_effects(
    adaptation: &Adaptation,
    faction_reputation: &mut std::collections::HashMap<String, i32>,
) {
    let visibility = adaptation.def().and_then(|d| d.faction_visibility.as_ref());
    let (monks_mult, traders_mult) = match visibility {
        Some(FactionVisibility::Low)      => return, // no effect
        Some(FactionVisibility::Moderate) => (1.1f32, 0.95f32),
        Some(FactionVisibility::High)     => (1.3f32, 0.80f32),
        Some(FactionVisibility::Extreme)  => (1.5f32, 0.50f32),
        None => return,
    };

    for (faction, mult) in [("MirrorMonks", monks_mult), ("SaltTraders", traders_mult)] {
        let current = *faction_reputation.get(faction).unwrap_or(&0);
        if current != 0 {
            let new_rep = ((current as f32 * mult).round() as i32).clamp(-100, 100);
            faction_reputation.insert(faction.to_string(), new_rep);
        }
    }
}
