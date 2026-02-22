use super::SettlementConfig;

/// Get dominant faction from control percentages
pub fn get_dominant_faction(config: &SettlementConfig) -> Option<String> {
    config
        .faction_control
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(faction, _)| faction.clone())
}

/// Get factions with significant control (>25%)
pub fn get_significant_factions(config: &SettlementConfig) -> Vec<String> {
    config
        .faction_control
        .iter()
        .filter(|(_, control)| *control > 0.25)
        .map(|(faction, _)| faction.clone())
        .collect()
}
