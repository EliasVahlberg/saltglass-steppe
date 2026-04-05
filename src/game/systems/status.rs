use super::System;
use crate::game::event::GameEvent;
use crate::game::state::{GameState, MsgType};
use crate::game::status::get_status_def;

/// Handles status effect ticking for all entities
pub struct StatusEffectSystem;

impl System for StatusEffectSystem {
    fn update(&self, state: &mut GameState) {
        Self::tick_player_effects(state);
        Self::tick_enemy_effects(state);
    }

    fn on_event(&self, state: &mut GameState, event: &GameEvent) {
        if let GameEvent::PlayerDamaged { amount, source } = event {
            // Could trigger bleed if source is slashing, etc.
            let _ = (amount, source);
            let _ = state;
        }
    }
}

impl StatusEffectSystem {
    /// Tick all player status effects
    pub fn tick_player_effects(state: &mut GameState) {
        let mut total_damage = 0;
        let mut messages = Vec::new();

        for effect in &mut state.player.status_effects {
            let dmg = effect.tick();
            if dmg > 0 {
                total_damage += dmg;
                messages.push(format!("{} deals {} damage.", effect.name, dmg));
            }
        }

        for msg in messages {
            state.log_typed(msg, MsgType::Combat);
        }

        if total_damage > 0 {
            state.player.hp -= total_damage;
        }

        state.player.status_effects.retain(|e| !e.is_expired());
    }

    /// Tick all enemy status effects
    pub fn tick_enemy_effects(state: &mut GameState) {
        let mut dead_enemies = Vec::new();

        for (idx, enemy) in state.world.enemies.iter_mut().enumerate() {
            if enemy.hp <= 0 {
                continue;
            }

            let mut enemy_damage = 0;

            for effect in &mut enemy.status_effects {
                if let Some(def) = get_status_def(&effect.id)
                    && def.tick_damage > 0
                {
                    enemy_damage += def.tick_damage;
                }
                effect.duration -= 1;
            }

            enemy.status_effects.retain(|e| e.duration > 0);

            if enemy_damage > 0 {
                enemy.hp -= enemy_damage;
                if enemy.hp <= 0 {
                    dead_enemies.push(idx);
                }
            }
        }

        // Handle enemies killed by status effects
        for idx in dead_enemies.into_iter().rev() {
            let enemy_id = state.world.enemies[idx].id.clone();
            let x = state.world.enemies[idx].x;
            let y = state.world.enemies[idx].y;

            state.spatial.enemy_positions.remove(&(x, y));
            state.log_typed(
                format!(
                    "The {} succumbs to status effects!",
                    state.world.enemies[idx].name()
                ),
                MsgType::Combat,
            );

            // Loot drop + quest progress (replaces event system)
            let loot_output = crate::game::rules::reactions::reaction_loot_drop(&enemy_id, x, y, &mut state.rng);
            for effect in &loot_output.effects {
                state.apply_effect(effect);
            }
            for p in &loot_output.presentation {
                state.apply_presentation(p);
            }
            state.player.quest_log.on_enemy_killed(&enemy_id);
            let completed = state.player.quest_log.check_auto_complete();
            state.log_quest_completions(&completed);
        }
    }

    /// Check if player has a blocking healing effect
    pub fn player_healing_blocked(state: &GameState) -> bool {
        state.player.status_effects.iter().any(|e| {
            get_status_def(&e.id)
                .map(|d| d.blocks_healing)
                .unwrap_or(false)
        })
    }

    /// Get player's accuracy penalty from status effects
    pub fn player_accuracy_penalty(state: &GameState) -> i32 {
        state
            .player
            .status_effects
            .iter()
            .filter_map(|e| get_status_def(&e.id))
            .map(|d| d.reduces_accuracy)
            .sum()
    }

    /// Check if player is stunned
    pub fn player_is_stunned(state: &GameState) -> bool {
        state
            .player
            .status_effects
            .iter()
            .any(|e| e.id == "stun" && e.duration > 0)
    }

    /// Check if enemy is stunned
    pub fn enemy_is_stunned(enemy: &crate::game::Enemy) -> bool {
        enemy
            .status_effects
            .iter()
            .any(|e| e.id == "stun" && e.duration > 0)
    }
}
