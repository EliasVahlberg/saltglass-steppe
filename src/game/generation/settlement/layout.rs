use super::{SettlementConfig, SettlementTier};

/// Calculate settlement dimensions based on tier
pub fn calculate_dimensions(config: &SettlementConfig) -> (usize, usize) {
    match config.tier {
        SettlementTier::Village => (80, 60),
        SettlementTier::Town => (120, 90),
        SettlementTier::City => (180, 120),
    }
}
