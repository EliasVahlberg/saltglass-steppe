use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Crystal frequency types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrystalFrequency {
    Alpha,
    Beta,
    Gamma,
    Delta,
    Epsilon,
}

impl CrystalFrequency {
    pub fn all() -> Vec<Self> {
        vec![
            CrystalFrequency::Alpha,
            CrystalFrequency::Beta,
            CrystalFrequency::Gamma,
            CrystalFrequency::Delta,
            CrystalFrequency::Epsilon,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            CrystalFrequency::Alpha => "Alpha",
            CrystalFrequency::Beta => "Beta",
            CrystalFrequency::Gamma => "Gamma",
            CrystalFrequency::Delta => "Delta",
            CrystalFrequency::Epsilon => "Epsilon",
        }
    }
}

/// Crystal formation in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalFormation {
    pub x: i32,
    pub y: i32,
    pub frequency: CrystalFrequency,
    pub size: u8,
    pub stability: u8,
    pub growth_stage: u8,
}

impl CrystalFormation {
    pub fn new(x: i32, y: i32, frequency: CrystalFrequency) -> Self {
        Self {
            x,
            y,
            frequency,
            size: 1,
            stability: 100,
            growth_stage: 0,
        }
    }

    pub fn can_grow(&self) -> bool {
        self.growth_stage < 5 && self.stability > 50
    }

    pub fn grow(&mut self) {
        if self.can_grow() {
            self.growth_stage += 1;
            self.stability = (self.stability as i32 - 10).max(0) as u8;
        }
    }
}

/// Harmonic resonance effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicEffect {
    pub x: i32,
    pub y: i32,
    pub frequencies: Vec<CrystalFrequency>,
    pub power: u32,
    pub duration: u32,
    pub effect_type: HarmonicType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarmonicType {
    Healing,
    Enhancement,
    Psychic,
    Structural,
    Chaotic,
}

/// Crystal resonance system state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrystalSystem {
    pub crystal_formations: Vec<CrystalFormation>,
    pub frequency_attunement: HashMap<CrystalFrequency, u32>,
    pub active_harmonics: Vec<HarmonicEffect>,
    pub resonance_energy: u32,
    pub max_resonance_energy: u32,
}

impl CrystalSystem {
    pub fn new() -> Self {
        Self {
            crystal_formations: Vec::new(),
            frequency_attunement: HashMap::new(),
            active_harmonics: Vec::new(),
            resonance_energy: 0,
            max_resonance_energy: 100,
        }
    }

    /// Add crystal formation at position (called during mapgen and item use)
    pub fn add_crystal(&mut self, x: i32, y: i32, frequency: CrystalFrequency) {
        let crystal = CrystalFormation::new(x, y, frequency);
        self.crystal_formations.push(crystal);
    }

    /// Update crystal system each turn
    pub fn update(&mut self, rng: &mut ChaCha8Rng) {
        // Update harmonic effects
        self.active_harmonics.retain_mut(|effect| {
            effect.duration = effect.duration.saturating_sub(1);
            effect.duration > 0
        });

        // Random crystal growth
        for crystal in &mut self.crystal_formations {
            if crystal.can_grow() && rng.gen_bool(0.05) {
                crystal.grow();
            }
        }

        // Crystal stability decay
        for crystal in &mut self.crystal_formations {
            if rng.gen_bool(0.02) {
                crystal.stability = crystal.stability.saturating_sub(1);
            }
        }

        // Remove unstable crystals
        self.crystal_formations
            .retain(|crystal| crystal.stability > 0);

        // Passive resonance energy regeneration
        if rng.gen_bool(0.1) {
            self.resonance_energy = (self.resonance_energy + 1).min(self.max_resonance_energy);
        }
    }
}
