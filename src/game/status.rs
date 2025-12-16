use serde::{Deserialize, Serialize};

/// Status effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusType {
    Poison,
    Burn,
    Stun,
    Bleed,
    Slow,
}

/// Active status effect on a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub effect_type: StatusType,
    pub duration: u32,
    pub potency: i32,
}

impl StatusEffect {
    pub fn new(effect_type: StatusType, duration: u32, potency: i32) -> Self {
        Self { effect_type, duration, potency }
    }

    /// Tick the effect, returning damage dealt (if any)
    pub fn tick(&mut self) -> i32 {
        if self.duration > 0 {
            self.duration -= 1;
        }
        match self.effect_type {
            StatusType::Poison => self.potency,
            StatusType::Burn => self.potency,
            StatusType::Bleed => self.potency,
            _ => 0,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.duration == 0
    }

    pub fn name(&self) -> &'static str {
        match self.effect_type {
            StatusType::Poison => "Poisoned",
            StatusType::Burn => "Burning",
            StatusType::Stun => "Stunned",
            StatusType::Bleed => "Bleeding",
            StatusType::Slow => "Slowed",
        }
    }
}

/// Check if stunned (can't act)
pub fn is_stunned(effects: &[StatusEffect]) -> bool {
    effects.iter().any(|e| e.effect_type == StatusType::Stun && e.duration > 0)
}

/// Get movement penalty from slow
pub fn slow_penalty(effects: &[StatusEffect]) -> i32 {
    effects.iter()
        .filter(|e| e.effect_type == StatusType::Slow && e.duration > 0)
        .map(|e| e.potency)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poison_deals_damage() {
        let mut effect = StatusEffect::new(StatusType::Poison, 3, 2);
        assert_eq!(effect.tick(), 2);
        assert_eq!(effect.duration, 2);
    }

    #[test]
    fn effect_expires() {
        let mut effect = StatusEffect::new(StatusType::Stun, 1, 0);
        effect.tick();
        assert!(effect.is_expired());
    }
}
