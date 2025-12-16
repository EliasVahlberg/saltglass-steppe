use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct ActionsFile {
    actions: HashMap<String, i32>,
    default_player_ap: i32,
    default_enemy_ap: i32,
}

static ACTIONS: Lazy<ActionsFile> = Lazy::new(|| {
    let data = include_str!("../../data/actions.json");
    serde_json::from_str(data).expect("Failed to parse actions.json")
});

pub fn action_cost(action: &str) -> i32 {
    ACTIONS.actions.get(action).copied().unwrap_or(1)
}

pub fn default_player_ap() -> i32 {
    ACTIONS.default_player_ap
}

pub fn default_enemy_ap() -> i32 {
    ACTIONS.default_enemy_ap
}
