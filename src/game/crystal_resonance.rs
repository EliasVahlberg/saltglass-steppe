use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Crystal frequency types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrystalFrequency {
    Alpha,   // 1-3 Hz - Deep resonance, structural effects
    Beta,    // 4-7 Hz - Mental clarity, psychic enhancement
    Gamma,   // 8-12 Hz - Energy flow, healing
    Delta,   // 13-30 Hz - High energy, combat enhancement
    Epsilon, // 31+ Hz - Chaotic, unpredictable effects
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

    pub fn base_power(self) -> u32 {
        match self {
            CrystalFrequency::Alpha => 5,
            CrystalFrequency::Beta => 3,
            CrystalFrequency::Gamma => 4,
            CrystalFrequency::Delta => 6,
            CrystalFrequency::Epsilon => 8,
        }
    }
}

/// Crystal formation in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalFormation {
    pub x: i32,
    pub y: i32,
    pub frequency: CrystalFrequency,
    pub size: u8,         // 1-10, affects resonance range and power
    pub stability: u8,    // 1-100, affects how long it lasts
    pub growth_stage: u8, // 0-5, crystals can grow over time
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

    pub fn resonance_range(&self) -> u8 {
        self.size + self.growth_stage
    }

    pub fn power_output(&self) -> u32 {
        let base = self.frequency.base_power();
        let size_multiplier = self.size as u32;
        let growth_multiplier = self.growth_stage as u32 + 1;
        let stability_factor = self.stability as f32 / 100.0;

        ((base * size_multiplier * growth_multiplier) as f32 * stability_factor) as u32
    }

    pub fn can_grow(&self) -> bool {
        self.growth_stage < 5 && self.stability > 50
    }

    pub fn grow(&mut self) {
        if self.can_grow() {
            self.growth_stage += 1;
            self.stability = (self.stability as i32 - 10).max(0) as u8; // Growing reduces stability
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
    Healing,     // Gamma + Beta
    Enhancement, // Delta + Alpha
    Psychic,     // Beta + Epsilon
    Structural,  // Alpha + Gamma
    Chaotic,     // Epsilon + any other
}

/// Crystal resonance system state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrystalSystem {
    pub crystal_formations: Vec<CrystalFormation>,
    pub frequency_attunement: HashMap<CrystalFrequency, u32>, // Player's attunement to each frequency
    pub active_harmonics: Vec<HarmonicEffect>,
    pub resonance_energy: u32, // Energy gained from crystal interactions
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

    /// Add crystal formation at position
    pub fn add_crystal(&mut self, x: i32, y: i32, frequency: CrystalFrequency) {
        let crystal = CrystalFormation::new(x, y, frequency);
        self.crystal_formations.push(crystal);
    }

    /// Get crystals within range of position
    pub fn get_crystals_in_range(&self, x: i32, y: i32, range: u8) -> Vec<&CrystalFormation> {
        self.crystal_formations
            .iter()
            .filter(|crystal| {
                let distance = ((x - crystal.x).abs() + (y - crystal.y).abs()) as u8;
                distance <= range
            })
            .collect()
    }

    /// Calculate resonance power at position
    pub fn calculate_resonance_power(&self, x: i32, y: i32) -> HashMap<CrystalFrequency, u32> {
        let mut power_by_frequency = HashMap::new();

        for crystal in &self.crystal_formations {
            let distance = ((x - crystal.x).abs() + (y - crystal.y).abs()) as u8;
            if distance <= crystal.resonance_range() {
                let power = crystal.power_output() / (distance as u32 + 1);
                *power_by_frequency.entry(crystal.frequency).or_insert(0) += power;
            }
        }

        power_by_frequency
    }

    /// Attune to crystal frequency
    pub fn attune_frequency(&mut self, frequency: CrystalFrequency, amount: u32) {
        let current = self
            .frequency_attunement
            .get(&frequency)
            .copied()
            .unwrap_or(0);
        self.frequency_attunement
            .insert(frequency, (current + amount).min(100));
    }

    /// Get attunement level for frequency
    pub fn get_attunement(&self, frequency: CrystalFrequency) -> u32 {
        self.frequency_attunement
            .get(&frequency)
            .copied()
            .unwrap_or(0)
    }

    /// Create harmonic resonance between crystals
    pub fn create_harmonic(
        &mut self,
        x: i32,
        y: i32,
        frequencies: Vec<CrystalFrequency>,
        power: u32,
    ) {
        let effect_type = self.determine_harmonic_type(&frequencies);
        let duration = 5 + (power / 10);

        let harmonic = HarmonicEffect {
            x,
            y,
            frequencies,
            power,
            duration,
            effect_type,
        };

        self.active_harmonics.push(harmonic);
    }

    /// Determine harmonic effect type based on frequencies
    fn determine_harmonic_type(&self, frequencies: &[CrystalFrequency]) -> HarmonicType {
        if frequencies.contains(&CrystalFrequency::Epsilon) {
            return HarmonicType::Chaotic;
        }

        if frequencies.contains(&CrystalFrequency::Gamma)
            && frequencies.contains(&CrystalFrequency::Beta)
        {
            HarmonicType::Healing
        } else if frequencies.contains(&CrystalFrequency::Delta)
            && frequencies.contains(&CrystalFrequency::Alpha)
        {
            HarmonicType::Enhancement
        } else if frequencies.contains(&CrystalFrequency::Beta) {
            HarmonicType::Psychic
        } else if frequencies.contains(&CrystalFrequency::Alpha) {
            HarmonicType::Structural
        } else {
            HarmonicType::Enhancement
        }
    }

    /// Check for harmonic effects at position
    pub fn get_harmonic_effects(&self, x: i32, y: i32) -> Vec<&HarmonicEffect> {
        self.active_harmonics
            .iter()
            .filter(|effect| {
                let distance = ((x - effect.x).abs() + (y - effect.y).abs()) as u32;
                distance <= effect.power / 5 // Effect range based on power
            })
            .collect()
    }

    /// Resonate with crystals at position
    pub fn resonate(&mut self, x: i32, y: i32) -> u32 {
        let power_map = self.calculate_resonance_power(x, y);
        let mut total_energy = 0;

        for (frequency, power) in power_map {
            // Gain attunement
            self.attune_frequency(frequency, power / 10);

            // Gain resonance energy
            let energy = power * (self.get_attunement(frequency) + 10) / 20;
            total_energy += energy;
        }

        self.resonance_energy =
            (self.resonance_energy + total_energy).min(self.max_resonance_energy);
        total_energy
    }

    /// Trigger crystal growth in area
    pub fn trigger_growth(&mut self, x: i32, y: i32, range: u8) -> u32 {
        let mut grown = 0;

        for crystal in &mut self.crystal_formations {
            let distance = ((x - crystal.x).abs() + (y - crystal.y).abs()) as u8;
            if distance <= range && crystal.can_grow() {
                crystal.grow();
                grown += 1;
            }
        }

        grown
    }

    /// Use resonance energy for abilities
    pub fn use_resonance_energy(&mut self, amount: u32) -> bool {
        if self.resonance_energy >= amount {
            self.resonance_energy -= amount;
            true
        } else {
            false
        }
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
                // 5% chance per turn
                crystal.grow();
            }
        }

        // Crystal stability decay
        for crystal in &mut self.crystal_formations {
            if rng.gen_bool(0.02) {
                // 2% chance per turn
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

    /// Create crystal seed (player ability)
    pub fn create_crystal_seed(
        &mut self,
        x: i32,
        y: i32,
        frequency: CrystalFrequency,
        cost: u32,
    ) -> bool {
        if !self.use_resonance_energy(cost) {
            return false;
        }

        self.add_crystal(x, y, frequency);
        true
    }

    /// Shatter crystals in area (player ability)
    pub fn shatter_crystals(&mut self, x: i32, y: i32, range: u8, cost: u32) -> u32 {
        if !self.use_resonance_energy(cost) {
            return 0;
        }

        let mut shattered = 0;
        self.crystal_formations.retain(|crystal| {
            let distance = ((x - crystal.x).abs() + (y - crystal.y).abs()) as u8;
            if distance <= range {
                shattered += 1;
                false // Remove crystal
            } else {
                true // Keep crystal
            }
        });

        shattered
    }

    /// Harmonize crystals to create powerful effect
    pub fn harmonize(&mut self, x: i32, y: i32, range: u8, cost: u32) -> bool {
        if !self.use_resonance_energy(cost) {
            return false;
        }

        let crystals = self.get_crystals_in_range(x, y, range);
        if crystals.len() < 2 {
            return false; // Need at least 2 crystals for harmony
        }

        let frequencies: Vec<CrystalFrequency> = crystals.iter().map(|c| c.frequency).collect();
        let total_power: u32 = crystals.iter().map(|c| c.power_output()).sum();

        self.create_harmonic(x, y, frequencies, total_power);
        true
    }
}
