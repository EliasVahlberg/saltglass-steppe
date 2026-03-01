use super::{SettlementConfig, SettlementTier};
use rand::Rng;

/// Calculate settlement dimensions based on tier
pub fn calculate_dimensions(config: &SettlementConfig) -> (usize, usize) {
    match config.tier {
        SettlementTier::Village => (80, 60),
        SettlementTier::Town => (120, 90),
        SettlementTier::City => (180, 120),
    }
}

/// Generate building placement positions distributed across the settlement area.
/// Returns a grid of candidate positions with spacing appropriate for the tier.
pub fn generate_layout<R: Rng>(
    config: &SettlementConfig,
    width: usize,
    height: usize,
    rng: &mut R,
) -> Vec<(i32, i32)> {
    let (spacing_x, spacing_y, margin) = match config.tier {
        SettlementTier::Village => (18, 14, 8),
        SettlementTier::Town => (16, 12, 6),
        SettlementTier::City => (14, 10, 5),
    };

    let mut positions = Vec::new();
    let mut y = margin;
    while y + margin < height {
        let mut x = margin;
        while x + margin < width {
            // Add small random jitter so buildings don't look perfectly grid-aligned
            let jitter_x = rng.gen_range(-(spacing_x as i32 / 4)..=(spacing_x as i32 / 4));
            let jitter_y = rng.gen_range(-(spacing_y as i32 / 4)..=(spacing_y as i32 / 4));
            let px = (x as i32 + jitter_x).max(margin as i32);
            let py = (y as i32 + jitter_y).max(margin as i32);
            positions.push((px, py));
            x += spacing_x;
        }
        y += spacing_y;
    }

    positions
}
