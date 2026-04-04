use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Void exposure levels and effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoidExposureLevel {
    None,
    Minimal,
    Moderate,
    High,
    Extreme,
}

impl VoidExposureLevel {
    pub fn from_exposure(exposure: u32) -> Self {
        match exposure {
            0..=10 => VoidExposureLevel::None,
            11..=25 => VoidExposureLevel::Minimal,
            26..=50 => VoidExposureLevel::Moderate,
            51..=75 => VoidExposureLevel::High,
            _ => VoidExposureLevel::Extreme,
        }
    }

    pub fn reality_distortion_chance(self) -> f32 {
        match self {
            VoidExposureLevel::None => 0.0,
            VoidExposureLevel::Minimal => 0.05,
            VoidExposureLevel::Moderate => 0.15,
            VoidExposureLevel::High => 0.30,
            VoidExposureLevel::Extreme => 0.50,
        }
    }
}

/// Void-based abilities (kept for serialization compatibility and menu display)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoidAbility {
    VoidStep,
    RealityRend,
    VoidShield,
    PhaseWalk,
    VoidDrain,
}

impl VoidAbility {
    pub fn energy_cost(self) -> u32 {
        match self {
            VoidAbility::VoidStep => 15,
            VoidAbility::RealityRend => 25,
            VoidAbility::VoidShield => 20,
            VoidAbility::PhaseWalk => 30,
            VoidAbility::VoidDrain => 10,
        }
    }

    pub fn min_exposure_required(self) -> u32 {
        match self {
            VoidAbility::VoidStep => 20,
            VoidAbility::RealityRend => 40,
            VoidAbility::VoidShield => 30,
            VoidAbility::PhaseWalk => 60,
            VoidAbility::VoidDrain => 15,
        }
    }
}

/// Reality distortion effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityDistortion {
    pub x: i32,
    pub y: i32,
    pub intensity: u8,
    pub duration: u32,
    pub effect_type: DistortionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistortionType {
    Temporal,
    Spatial,
    Material,
    Psychic,
}

/// Void energy system state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoidSystem {
    pub void_exposure: u32,
    pub void_energy: u32,
    pub max_void_energy: u32,
    pub unlocked_abilities: Vec<VoidAbility>,
    pub active_distortions: Vec<RealityDistortion>,
    pub phase_walk_turns: u32,
}

impl VoidSystem {
    pub fn new() -> Self {
        Self {
            void_exposure: 0,
            void_energy: 0,
            max_void_energy: 50,
            unlocked_abilities: Vec::new(),
            active_distortions: Vec::new(),
            phase_walk_turns: 0,
        }
    }

    /// Get current exposure level
    pub fn exposure_level(&self) -> VoidExposureLevel {
        VoidExposureLevel::from_exposure(self.void_exposure)
    }

    /// Increase void exposure (called from item use)
    pub fn add_exposure(&mut self, amount: u32) -> bool {
        let old_level = self.exposure_level();
        self.void_exposure = (self.void_exposure + amount).min(100);
        self.max_void_energy = 50 + (self.void_exposure / 2);
        self.check_ability_unlocks();
        old_level != self.exposure_level()
    }

    /// Check and unlock new abilities based on exposure
    fn check_ability_unlocks(&mut self) {
        let abilities = [
            VoidAbility::VoidStep,
            VoidAbility::VoidDrain,
            VoidAbility::VoidShield,
            VoidAbility::RealityRend,
            VoidAbility::PhaseWalk,
        ];

        for ability in abilities {
            if self.void_exposure >= ability.min_exposure_required()
                && !self.unlocked_abilities.contains(&ability)
            {
                self.unlocked_abilities.push(ability);
            }
        }
    }

    /// Gain void energy (called from item use)
    pub fn gain_energy(&mut self, amount: u32) {
        self.void_energy = (self.void_energy + amount).min(self.max_void_energy);
    }

    /// Update void system each turn
    pub fn update(&mut self, rng: &mut ChaCha8Rng) {
        // Decay phase walk
        if self.phase_walk_turns > 0 {
            self.phase_walk_turns -= 1;
        }

        // Update distortions
        self.active_distortions.retain_mut(|distortion| {
            distortion.duration = distortion.duration.saturating_sub(1);
            distortion.duration > 0
        });

        // Random reality distortions based on exposure
        let distortion_chance = self.exposure_level().reality_distortion_chance();
        if rng.gen_bool(distortion_chance as f64) {
            self.trigger_random_distortion(rng);
        }

        // Passive void energy regeneration at high exposure
        if self.void_exposure >= 50 && rng.gen_bool(0.3) {
            self.gain_energy(1);
        }
    }

    /// Trigger random reality distortion (called from update)
    fn trigger_random_distortion(&mut self, rng: &mut ChaCha8Rng) {
        let distortion_types = [
            DistortionType::Temporal,
            DistortionType::Spatial,
            DistortionType::Material,
            DistortionType::Psychic,
        ];

        let effect_type = distortion_types[rng.gen_range(0..distortion_types.len())];
        let intensity = rng.gen_range(1..=5);
        let duration = rng.gen_range(3..=8);
        let x = rng.gen_range(-5..=5);
        let y = rng.gen_range(-5..=5);

        self.active_distortions.push(RealityDistortion {
            x,
            y,
            intensity,
            duration,
            effect_type,
        });
    }
}
