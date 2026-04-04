use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Reputation thresholds
pub const REP_HATED: i32 = -100;
pub const REP_HOSTILE: i32 = -50;
pub const REP_UNFRIENDLY: i32 = -25;
pub const REP_NEUTRAL: i32 = 0;
pub const REP_FRIENDLY: i32 = 25;
pub const REP_HONORED: i32 = 50;
pub const REP_EXALTED: i32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
}

#[derive(Deserialize)]
struct FactionsFile {
    factions: Vec<Faction>,
}

static FACTIONS: Lazy<HashMap<String, Faction>> = Lazy::new(|| {
    let data = include_str!("../../data/factions.json");
    let file: FactionsFile = serde_json::from_str(data).expect("Failed to parse factions.json");

    file.factions
        .into_iter()
        .map(|f| (f.id.clone(), f))
        .collect()
});

/// Get all faction IDs
pub fn all_faction_ids() -> Vec<String> {
    FACTIONS.keys().cloned().collect()
}

/// Get faction by ID
pub fn get_faction(id: &str) -> Option<&'static Faction> {
    FACTIONS.get(id)
}

/// Get reputation standing text
pub fn get_standing(rep: i32) -> &'static str {
    match rep {
        i32::MIN..=-50 => "Hostile",
        -49..=-25 => "Unfriendly",
        -24..=24 => "Neutral",
        25..=49 => "Friendly",
        50..=99 => "Honored",
        100..=i32::MAX => "Exalted",
    }
}

/// Get reputation standing color (for UI)
pub fn get_standing_color(rep: i32) -> ratatui::style::Color {
    use ratatui::style::Color;
    match rep {
        i32::MIN..=-50 => Color::Red,
        -49..=-25 => Color::LightRed,
        -24..=24 => Color::White,
        25..=49 => Color::Green,
        50..=99 => Color::Cyan,
        100..=i32::MAX => Color::Magenta,
    }
}

/// Get starting reputation for a character class
pub fn get_starting_reputation(class: &str) -> HashMap<String, i32> {
    let mut rep = HashMap::new();

    match class {
        "pilgrim" => {
            rep.insert("MirrorMonks".to_string(), 10);
            rep.insert("ArchiveDrones".to_string(), 5);
        }
        "scavenger" => {
            rep.insert("SaltTraders".to_string(), 10);
            rep.insert("SandEngineers".to_string(), 5);
        }
        "outcast" => {
            rep.insert("RefractionOutcasts".to_string(), 15);
            // Penalty to all others
            for id in all_faction_ids() {
                if id != "RefractionOutcasts" {
                    rep.insert(id, -10);
                }
            }
        }
        "cultist" => {
            rep.insert("StormCults".to_string(), 20);
            rep.insert("MirrorMonks".to_string(), -15);
        }
        _ => {
            // Default: all neutral (0)
        }
    }

    rep
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factions_load() {
        let ids = all_faction_ids();
        assert!(!ids.is_empty(), "Should load factions");
        assert!(ids.contains(&"MirrorMonks".to_string()));
        assert!(ids.contains(&"SaltTraders".to_string()));
    }

    #[test]
    fn test_get_faction() {
        let faction = get_faction("MirrorMonks");
        assert!(faction.is_some());
        assert_eq!(faction.unwrap().name, "Mirror Monks");
    }

    #[test]
    fn test_standing_thresholds() {
        assert_eq!(get_standing(-100), "Hostile");
        assert_eq!(get_standing(-50), "Hostile");
        assert_eq!(get_standing(-49), "Unfriendly");
        assert_eq!(get_standing(0), "Neutral");
        assert_eq!(get_standing(25), "Friendly");
        assert_eq!(get_standing(50), "Honored");
        assert_eq!(get_standing(100), "Exalted");
    }

    #[test]
    fn test_starting_reputation() {
        let rep = get_starting_reputation("pilgrim");
        assert_eq!(rep.get("MirrorMonks"), Some(&10));
        assert_eq!(rep.get("ArchiveDrones"), Some(&5));

        let rep = get_starting_reputation("outcast");
        assert_eq!(rep.get("RefractionOutcasts"), Some(&15));
        assert_eq!(rep.get("MirrorMonks"), Some(&-10));
    }
}
