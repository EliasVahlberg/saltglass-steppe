use serde::{Deserialize, Serialize};
use rand_chacha::ChaCha8Rng;
use rand::Rng;

/// Void exposure levels and effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoidExposureLevel {
    None,       // 0-10 exposure
    Minimal,    // 11-25 exposure
    Moderate,   // 26-50 exposure
    High,       // 51-75 exposure
    Extreme,    // 76-100 exposure
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

/// Void-based abilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoidAbility {
    VoidStep,       // Short-range teleportation
    RealityRend,    // Damage that ignores armor
    VoidShield,     // Absorb damage using void energy
    PhaseWalk,      // Walk through walls temporarily
    VoidDrain,      // Drain energy from enemies
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
    Temporal,    // Time flows differently
    Spatial,     // Space is warped
    Material,    // Matter becomes unstable
    Psychic,     // Mental effects
}

/// Void energy system state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoidSystem {
    pub void_exposure: u32,
    pub void_energy: u32,
    pub max_void_energy: u32,
    pub unlocked_abilities: Vec<VoidAbility>,
    pub active_distortions: Vec<RealityDistortion>,
    pub phase_walk_turns: u32, // Remaining turns of phase walk
}

impl VoidSystem {
    /// Create new void system with default values
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

    /// Increase void exposure
    pub fn add_exposure(&mut self, amount: u32) -> bool {
        let old_level = self.exposure_level();
        self.void_exposure = (self.void_exposure + amount).min(100);
        
        // Increase max void energy as exposure increases
        self.max_void_energy = 50 + (self.void_exposure / 2);
        
        // Check for new ability unlocks
        self.check_ability_unlocks();
        
        // Return true if exposure level changed
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
                && !self.unlocked_abilities.contains(&ability) {
                self.unlocked_abilities.push(ability);
            }
        }
    }

    /// Use void ability if possible
    pub fn use_ability(&mut self, ability: VoidAbility) -> bool {
        if !self.unlocked_abilities.contains(&ability) {
            return false;
        }

        let cost = ability.energy_cost();
        if self.void_energy < cost {
            return false;
        }

        self.void_energy -= cost;

        match ability {
            VoidAbility::PhaseWalk => {
                self.phase_walk_turns = 3; // 3 turns of phase walking
            }
            _ => {} // Other abilities handled by caller
        }

        true
    }

    /// Gain void energy
    pub fn gain_energy(&mut self, amount: u32) {
        self.void_energy = (self.void_energy + amount).min(self.max_void_energy);
    }

    /// Create reality distortion at position
    pub fn create_distortion(&mut self, x: i32, y: i32, intensity: u8, 
                           duration: u32, effect_type: DistortionType) {
        let distortion = RealityDistortion {
            x, y, intensity, duration, effect_type
        };
        self.active_distortions.push(distortion);
    }

    /// Check if position has reality distortion
    pub fn has_distortion(&self, x: i32, y: i32) -> Option<&RealityDistortion> {
        self.active_distortions.iter()
            .find(|d| d.x == x && d.y == y)
    }

    /// Calculate void damage at position
    pub fn calculate_void_damage(&self, x: i32, y: i32) -> u32 {
        let mut damage = 0;

        for distortion in &self.active_distortions {
            let distance = ((x - distortion.x).abs() + (y - distortion.y).abs()) as u8;
            if distance <= distortion.intensity {
                damage += (distortion.intensity - distance) as u32;
            }
        }

        damage
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

    /// Trigger random reality distortion
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

        // Create distortion at random nearby location
        let x = rng.gen_range(-5..=5);
        let y = rng.gen_range(-5..=5);

        self.create_distortion(x, y, intensity, duration, effect_type);
    }

    /// Check if player can phase walk through walls
    pub fn can_phase_walk(&self) -> bool {
        self.phase_walk_turns > 0
    }

    /// Perform void step teleportation
    pub fn void_step(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
        if !self.use_ability(VoidAbility::VoidStep) {
            return false;
        }

        let distance = ((to_x - from_x).abs() + (to_y - from_y).abs()) as u32;
        if distance > 5 {
            return false; // Max range of 5 tiles
        }

        // Create small distortion at departure point
        self.create_distortion(from_x, from_y, 2, 3, DistortionType::Spatial);

        true
    }

    /// Perform reality rend attack
    pub fn reality_rend(&mut self, target_x: i32, target_y: i32) -> Option<u32> {
        if !self.use_ability(VoidAbility::RealityRend) {
            return None;
        }

        let base_damage = 15 + (self.void_exposure / 10);
        
        // Create distortion at target
        self.create_distortion(target_x, target_y, 3, 2, DistortionType::Material);

        Some(base_damage)
    }

    /// Activate void shield
    pub fn void_shield(&mut self) -> bool {
        self.use_ability(VoidAbility::VoidShield)
    }

    /// Drain void energy from target
    pub fn void_drain(&mut self, target_x: i32, target_y: i32) -> u32 {
        if !self.use_ability(VoidAbility::VoidDrain) {
            return 0;
        }

        let drained = 5 + (self.void_exposure / 20);
        self.gain_energy(drained);

        // Create psychic distortion
        self.create_distortion(target_x, target_y, 2, 2, DistortionType::Psychic);

        drained
    }
}
