pub mod prefab;
pub mod layout;
pub mod buildings;
pub mod faction_theme;
pub mod population;

pub use prefab::{Prefab, PrefabLibrary};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Configuration for settlement generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    pub seed: u64,
    pub tier: SettlementTier,
    pub faction_control: Vec<(String, f32)>, // (faction_id, control_percentage)
}

/// Settlement tier determines size and complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementTier {
    Village,
    Town,
    City,
}

/// A placed building in the settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub prefab_name: String,
    pub x: i32,
    pub y: i32,
    pub faction: Option<String>,
}

/// Generated settlement with all placed buildings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    pub config: SettlementConfig,
    pub buildings: Vec<Building>,
    pub width: usize,
    pub height: usize,
}

/// Main entry point for settlement generation
pub fn generate_settlement<R: Rng>(config: SettlementConfig, rng: &mut R) -> Settlement {
    let (width, height) = layout::calculate_dimensions(&config);
    let buildings = buildings::place_buildings(&config, width, height, rng);
    
    Settlement {
        config,
        buildings,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_generate_settlement_village() {
        let config = SettlementConfig {
            seed: 12345,
            tier: SettlementTier::Village,
            faction_control: vec![("MirrorMonks".to_string(), 0.6)],
        };
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let settlement = generate_settlement(config.clone(), &mut rng);
        
        assert_eq!(settlement.width, 80);
        assert_eq!(settlement.height, 60);
        assert_eq!(settlement.config.tier, SettlementTier::Village);
    }

    #[test]
    fn test_generate_settlement_town() {
        let config = SettlementConfig {
            seed: 54321,
            tier: SettlementTier::Town,
            faction_control: vec![
                ("SaltTraders".to_string(), 0.4),
                ("Glassborn".to_string(), 0.3),
            ],
        };
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let settlement = generate_settlement(config, &mut rng);
        
        assert_eq!(settlement.width, 120);
        assert_eq!(settlement.height, 90);
    }

    #[test]
    fn test_faction_theme_dominant() {
        let config = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Village,
            faction_control: vec![
                ("MirrorMonks".to_string(), 0.5),
                ("Glassborn".to_string(), 0.3),
            ],
        };
        
        let dominant = faction_theme::get_dominant_faction(&config);
        assert_eq!(dominant, Some("MirrorMonks".to_string()));
    }

    #[test]
    fn test_faction_theme_significant() {
        let config = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Town,
            faction_control: vec![
                ("MirrorMonks".to_string(), 0.4),
                ("Glassborn".to_string(), 0.3),
                ("SaltTraders".to_string(), 0.2),
            ],
        };
        
        let significant = faction_theme::get_significant_factions(&config);
        assert_eq!(significant.len(), 2); // Only >25%
        assert!(significant.contains(&"MirrorMonks".to_string()));
        assert!(significant.contains(&"Glassborn".to_string()));
    }

    #[test]
    fn test_population_calculation() {
        let village = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Village,
            faction_control: vec![],
        };
        assert_eq!(population::calculate_population(&village), 20);

        let town = SettlementConfig {
            seed: 1,
            tier: SettlementTier::Town,
            faction_control: vec![],
        };
        assert_eq!(population::calculate_population(&town), 50);

        let city = SettlementConfig {
            seed: 1,
            tier: SettlementTier::City,
            faction_control: vec![],
        };
        assert_eq!(population::calculate_population(&city), 100);
    }
}
