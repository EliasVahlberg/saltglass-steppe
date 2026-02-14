use super::System;
use crate::game::event::GameEvent;
use crate::game::state::GameState;

/// Handles quest progression via event bus
pub struct QuestSystem;

impl System for QuestSystem {
    fn update(&self, _state: &mut GameState) {
        // QuestSystem is reactive-only (event-driven)
    }

    fn on_event(&self, state: &mut GameState, event: &GameEvent) {
        let completed = match event {
            GameEvent::EnemyKilled { enemy_id, .. } => {
                state.player.quest_log.on_enemy_killed(enemy_id);
                state.player.quest_log.check_auto_complete()
            }
            GameEvent::ItemPickedUp { item_id } => {
                state.player.quest_log.on_item_collected(item_id);
                state.player.quest_log.check_auto_complete()
            }
            GameEvent::PlayerMoved { to_x, to_y, .. } => {
                state.player.quest_log.on_position_changed(*to_x, *to_y);
                state.player.quest_log.check_auto_complete()
            }
            GameEvent::NpcTalkedTo { npc_id } => {
                state.player.quest_log.on_npc_talked(npc_id)
            }
            GameEvent::InteractableUsed { interactable_id } => {
                state.player.quest_log.on_interact(interactable_id);
                state.player.quest_log.check_auto_complete()
            }
            GameEvent::InteractableExamined { interactable_id } => {
                state.player.quest_log.on_examine(interactable_id);
                state.player.quest_log.check_auto_complete()
            }
            GameEvent::AriaInterfaced { item_id } => {
                state.player.quest_log.on_aria_interfaced(item_id);
                state.player.quest_log.check_auto_complete()
            }
            GameEvent::TurnEnded { .. } => {
                state.player.quest_log.on_turn_passed();
                state.player.quest_log.check_auto_complete()
            }
            _ => vec![],
        };

        // Emit QuestCompleted for each completed quest
        for quest_id in completed {
            state.emit(GameEvent::QuestCompleted { quest_id });
        }
    }
}
