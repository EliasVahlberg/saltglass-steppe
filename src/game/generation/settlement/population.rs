use super::{SettlementConfig, SettlementTier};

/// Calculate expected population based on tier
pub fn calculate_population(config: &SettlementConfig) -> usize {
    match config.tier {
        SettlementTier::Village => 20,
        SettlementTier::Town => 50,
        SettlementTier::City => 100,
    }
}
