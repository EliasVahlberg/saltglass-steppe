use crate::game::state::GameState;
use crate::game::event::GameEvent;
use super::System;

/// Handles quest progression via event bus
pub struct QuestSystem;

impl System for QuestSystem {
    fn update(&self, _state: &mut GameState) {
        // QuestSystem is reactive-only (event-driven)
    }
    
    fn on_event(&self, state: &mut GameState, event: &GameEvent) {
        match event {
            GameEvent::EnemyKilled { enemy_id, .. } => {
                state.quest_log.on_enemy_killed(enemy_id);
            }
            GameEvent::ItemPickedUp { item_id } => {
                state.quest_log.on_item_collected(item_id);
            }
            _ => {}
        }
    }
}
