use crate::game::state::{GameState, MsgType};
use crate::game::event::GameEvent;
use crate::game::status::get_status_def;
use super::System;

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
    fn tick_player_effects(state: &mut GameState) {
        let mut total_damage = 0;
        let mut messages = Vec::new();
        
        for effect in &mut state.status_effects {
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
            state.player_hp -= total_damage;
        }
        
        state.status_effects.retain(|e| !e.is_expired());
    }
    
    /// Tick all enemy status effects
    fn tick_enemy_effects(state: &mut GameState) {
        let mut dead_enemies = Vec::new();
        
        for (idx, enemy) in state.enemies.iter_mut().enumerate() {
            if enemy.hp <= 0 { continue; }
            
            let mut enemy_damage = 0;
            
            for effect in &mut enemy.status_effects {
                if let Some(def) = get_status_def(&effect.id) {
                    if def.tick_damage > 0 {
                        enemy_damage += def.tick_damage;
                    }
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
            let enemy_id = state.enemies[idx].id.clone();
            let x = state.enemies[idx].x;
            let y = state.enemies[idx].y;
            
            state.enemy_positions.remove(&(x, y));
            state.log_typed(
                format!("The {} succumbs to status effects!", state.enemies[idx].name()),
                MsgType::Combat
            );
            
            state.emit(GameEvent::EnemyKilled { enemy_id, x, y });
        }
    }
    
    /// Check if player has a blocking healing effect
    pub fn player_healing_blocked(state: &GameState) -> bool {
        state.status_effects.iter().any(|e| {
            get_status_def(&e.id).map(|d| d.blocks_healing).unwrap_or(false)
        })
    }
    
    /// Get player's accuracy penalty from status effects
    pub fn player_accuracy_penalty(state: &GameState) -> i32 {
        state.status_effects.iter()
            .filter_map(|e| get_status_def(&e.id))
            .map(|d| d.reduces_accuracy)
            .sum()
    }
    
    /// Check if player is stunned
    pub fn player_is_stunned(state: &GameState) -> bool {
        state.status_effects.iter().any(|e| e.id == "stun" && e.duration > 0)
    }
    
    /// Check if enemy is stunned
    pub fn enemy_is_stunned(enemy: &crate::game::Enemy) -> bool {
        enemy.status_effects.iter().any(|e| e.id == "stun" && e.duration > 0)
    }
}
