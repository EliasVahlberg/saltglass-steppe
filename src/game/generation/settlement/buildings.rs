use super::{Building, SettlementConfig};
use rand::Rng;

/// Place buildings in the settlement
pub fn place_buildings<R: Rng>(
    _config: &SettlementConfig,
    _width: usize,
    _height: usize,
    _rng: &mut R,
) -> Vec<Building> {
    // Stub: return empty for now
    Vec::new()
}
