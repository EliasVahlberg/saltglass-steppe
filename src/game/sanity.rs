use rand::Rng;
use serde::{Deserialize, Serialize};

/// Mental health system tracking psychological effects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SanitySystem {
    /// Current sanity (0-100)
    pub current_sanity: u32,
    /// Maximum sanity (can be reduced by permanent trauma)
    pub max_sanity: u32,
    /// Active mental effects
    pub mental_effects: Vec<MentalEffect>,
    /// Adaptation tolerance (higher = less sanity loss from adaptations)
    pub adaptation_tolerance: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MentalEffect {
    pub effect_type: MentalEffectType,
    pub duration: u32,
    pub intensity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MentalEffectType {
    Hallucinations,
    Paranoia,
    Confusion,
    Despair,
    Clarity,       // Positive effect
    Determination, // Positive effect
}

impl Default for SanitySystem {
    fn default() -> Self {
        Self {
            current_sanity: 80,
            max_sanity: 100,
            mental_effects: Vec::new(),
            adaptation_tolerance: 0,
        }
    }
}

impl SanitySystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lose sanity from various sources
    pub fn lose_sanity(&mut self, amount: u32, source: &str) -> Vec<String> {
        let mut messages = Vec::new();

        // Apply adaptation tolerance
        let actual_loss = if source.contains("adaptation") {
            amount.saturating_sub(self.adaptation_tolerance)
        } else {
            amount
        };

        if actual_loss > 0 {
            self.current_sanity = self.current_sanity.saturating_sub(actual_loss);
            messages.push(format!(
                "You feel your sanity slipping... (-{} sanity)",
                actual_loss
            ));

            // Check for new mental effects based on sanity level
            if self.current_sanity <= 20 && !self.has_effect(MentalEffectType::Despair) {
                self.add_mental_effect(MentalEffectType::Despair, 10, 3);
                messages.push("Overwhelming despair clouds your thoughts.".to_string());
            } else if self.current_sanity <= 40 && !self.has_effect(MentalEffectType::Paranoia) {
                self.add_mental_effect(MentalEffectType::Paranoia, 8, 2);
                messages.push("You feel like something is watching you...".to_string());
            } else if self.current_sanity <= 60 && !self.has_effect(MentalEffectType::Confusion) {
                self.add_mental_effect(MentalEffectType::Confusion, 5, 1);
                messages.push("Your thoughts feel muddled and unclear.".to_string());
            }
        }

        messages
    }

    /// Restore sanity through various means
    pub fn restore_sanity(&mut self, amount: u32, source: &str) -> String {
        let old_sanity = self.current_sanity;
        self.current_sanity = (self.current_sanity + amount).min(self.max_sanity);
        let actual_gain = self.current_sanity - old_sanity;

        if actual_gain > 0 {
            // Remove negative effects if sanity is restored enough
            if self.current_sanity > 60 {
                self.remove_effect(MentalEffectType::Despair);
                self.remove_effect(MentalEffectType::Paranoia);
            }
            if self.current_sanity > 40 {
                self.remove_effect(MentalEffectType::Confusion);
            }

            format!(
                "You feel more mentally stable. (+{} sanity from {})",
                actual_gain, source
            )
        } else {
            "Your sanity is already at maximum.".to_string()
        }
    }

    /// Add a mental effect
    pub fn add_mental_effect(
        &mut self,
        effect_type: MentalEffectType,
        duration: u32,
        intensity: u32,
    ) {
        // Remove existing effect of same type
        self.remove_effect(effect_type.clone());

        self.mental_effects.push(MentalEffect {
            effect_type,
            duration,
            intensity,
        });
    }

    /// Check if has specific mental effect
    pub fn has_effect(&self, effect_type: MentalEffectType) -> bool {
        self.mental_effects
            .iter()
            .any(|e| e.effect_type == effect_type)
    }

    /// Remove mental effect
    pub fn remove_effect(&mut self, effect_type: MentalEffectType) {
        self.mental_effects.retain(|e| e.effect_type != effect_type);
    }

    /// Tick mental effects each turn
    pub fn tick_effects(&mut self) -> Vec<String> {
        let mut messages = Vec::new();

        // Tick down durations
        self.mental_effects.retain_mut(|effect| {
            effect.duration = effect.duration.saturating_sub(1);
            if effect.duration == 0 {
                messages.push(match effect.effect_type {
                    MentalEffectType::Hallucinations => "The hallucinations fade away.".to_string(),
                    MentalEffectType::Paranoia => {
                        "The feeling of being watched subsides.".to_string()
                    }
                    MentalEffectType::Confusion => "Your thoughts become clearer.".to_string(),
                    MentalEffectType::Despair => "The crushing despair lifts slightly.".to_string(),
                    MentalEffectType::Clarity => "Your moment of clarity passes.".to_string(),
                    MentalEffectType::Determination => "Your determination wavers.".to_string(),
                });
                false
            } else {
                true
            }
        });

        messages
    }

    /// Get sanity-based decision penalties
    pub fn decision_penalty(&self) -> i32 {
        let base_penalty = match self.current_sanity {
            0..=20 => -3,
            21..=40 => -2,
            41..=60 => -1,
            _ => 0,
        };

        // Add mental effect penalties
        let effect_penalty: i32 = self
            .mental_effects
            .iter()
            .map(|e| match e.effect_type {
                MentalEffectType::Confusion => -(e.intensity as i32),
                MentalEffectType::Paranoia => -(e.intensity as i32) / 2,
                MentalEffectType::Despair => -(e.intensity as i32),
                MentalEffectType::Clarity => e.intensity as i32,
                MentalEffectType::Determination => (e.intensity as i32) / 2,
                _ => 0,
            })
            .sum();

        base_penalty + effect_penalty
    }

    /// Check if should trigger hallucination
    pub fn should_hallucinate<R: Rng>(&self, rng: &mut R) -> bool {
        if self.has_effect(MentalEffectType::Hallucinations) {
            rng.gen_ratio(1, 4) // 25% chance per turn
        } else if self.current_sanity <= 30 {
            rng.gen_ratio(1, 20) // 5% chance at very low sanity
        } else {
            false
        }
    }

    /// Generate hallucination message
    pub fn generate_hallucination<R: Rng>(&self, rng: &mut R) -> String {
        let hallucinations = [
            "You see shadows moving in your peripheral vision.",
            "The walls seem to shimmer and breathe.",
            "You hear whispers that aren't there.",
            "Glass shards appear to spell out words before vanishing.",
            "You glimpse a figure that disappears when you look directly.",
            "The ground beneath you feels unstable and shifting.",
            "Colors seem more vivid and wrong somehow.",
            "You smell something burning that isn't there.",
        ];

        hallucinations[rng.gen_range(0..hallucinations.len())].to_string()
    }

    /// Handle adaptation effects on sanity
    pub fn adaptation_effect(&mut self, adaptation_name: &str) -> Vec<String> {
        let sanity_loss = match adaptation_name {
            "glass_sight" => 5,
            "salt_blood" => 8,
            "crystal_bones" => 12,
            "shard_skin" => 15,
            _ => 3, // Default for unknown adaptations
        };

        let mut messages =
            self.lose_sanity(sanity_loss, &format!("adaptation: {}", adaptation_name));

        // Increase adaptation tolerance slightly
        if self.adaptation_tolerance < 10 {
            self.adaptation_tolerance += 1;
            messages.push("You feel slightly more accustomed to the changes.".to_string());
        }

        messages
    }

    /// Ways to restore mental health
    pub fn rest_restoration(&mut self) -> String {
        self.restore_sanity(5, "rest")
    }

    pub fn social_restoration(&mut self) -> String {
        self.restore_sanity(8, "positive social interaction")
    }

    pub fn meditation_restoration(&mut self) -> String {
        self.restore_sanity(10, "meditation")
    }
}
