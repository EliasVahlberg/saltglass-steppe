use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TutorialMessage {
    pub id: String,
    pub trigger: String,
    pub text: String,
    pub highlight: String,
    pub dismiss_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TutorialData {
    pub messages: Vec<TutorialMessage>,
    pub control_hints: Vec<String>,
}

static TUTORIAL_DATA: Lazy<TutorialData> = Lazy::new(|| {
    let data_str = include_str!("../../data/tutorial.json");
    serde_json::from_str(data_str).expect("Failed to parse tutorial.json")
});

pub fn get_tutorial_data() -> &'static TutorialData {
    &TUTORIAL_DATA
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TutorialProgress {
    /// Set of tutorial message IDs that have been shown
    shown_messages: HashSet<String>,
    /// Whether tutorial is completely disabled
    tutorial_disabled: bool,
}

impl TutorialProgress {
    pub fn new() -> Self {
        Self {
            shown_messages: HashSet::new(),
            tutorial_disabled: false,
        }
    }

    pub fn has_shown(&self, message_id: &str) -> bool {
        self.shown_messages.contains(message_id)
    }

    pub fn mark_shown(&mut self, message_id: &str) {
        self.shown_messages.insert(message_id.to_string());
    }

    pub fn is_disabled(&self) -> bool {
        self.tutorial_disabled
    }

    pub fn disable(&mut self) {
        self.tutorial_disabled = true;
    }

    /// Check if a tutorial message should be shown based on trigger condition
    pub fn check_trigger(
        &self,
        trigger: &str,
        game_state: &super::state::GameState,
    ) -> bool {
        if self.tutorial_disabled {
            return false;
        }

        match trigger {
            "game_start" => game_state.turn == 0 && !self.has_shown("game_start"),
            "first_enemy_visible" => {
                !self.has_shown("first_enemy_visible")
                    && !game_state.enemies.is_empty()
                    && game_state.enemies.iter().any(|e| {
                        e.is_alive() && game_state.visible.contains(&(e.y as usize * game_state.map.width + e.x as usize))
                    })
            }
            "first_item_visible" => {
                !self.has_shown("first_item_visible")
                    && !game_state.items.is_empty()
                    && game_state.items.iter().any(|item| {
                        game_state.visible.contains(&(item.y as usize * game_state.map.width + item.x as usize))
                    })
            }
            "low_hp" => {
                !self.has_shown("low_hp")
                    && game_state.player_hp < (game_state.player_max_hp / 3)
            }
            "first_npc_visible" => {
                !self.has_shown("first_npc_visible")
                    && !game_state.npcs.is_empty()
                    && game_state.npcs.iter().any(|npc| {
                        game_state.visible.contains(&(npc.y as usize * game_state.map.width + npc.x as usize))
                    })
            }
            "ap_depleted" => {
                !self.has_shown("ap_depleted") && game_state.player_ap == 0
            }
            _ => false,
        }
    }

    /// Get the next tutorial message that should be shown, if any
    pub fn get_next_message(
        &self,
        game_state: &super::state::GameState,
    ) -> Option<&'static TutorialMessage> {
        let data = get_tutorial_data();
        
        for message in &data.messages {
            if self.check_trigger(&message.trigger, game_state) {
                return Some(message);
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_data_loads() {
        let data = get_tutorial_data();
        assert!(!data.messages.is_empty());
        assert!(!data.control_hints.is_empty());
    }

    #[test]
    fn test_tutorial_progress() {
        let mut progress = TutorialProgress::new();
        
        assert!(!progress.has_shown("game_start"));
        progress.mark_shown("game_start");
        assert!(progress.has_shown("game_start"));
        
        assert!(!progress.is_disabled());
        progress.disable();
        assert!(progress.is_disabled());
    }
}
