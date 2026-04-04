use super::System;
use crate::game::{
    enemy::Enemy,
    event::GameEvent,
    state::{GameState, MsgType},
};

pub struct CombatSystem;

impl System for CombatSystem {
    fn update(&self, _state: &mut GameState) {}
    fn on_event(&self, _state: &mut GameState, _event: &GameEvent) {}
}

impl CombatSystem {
    /// Post-processing after Kill effect is applied.
    /// Handles on_death visual effects and split_on_death behavior.
    /// XP, spatial index removal, EnemyKilled event, and discover_enemy
    /// are handled by the Kill effect apply arm and rule output.
    pub fn process_enemy_death_post(state: &mut GameState, enemy_idx: usize) {
        let enemy_x = state.world.enemies[enemy_idx].x;
        let enemy_y = state.world.enemies[enemy_idx].y;

        if let Some(def) = state.world.enemies[enemy_idx].def() {
            for e in &def.effects {
                if e.condition == "on_death" {
                    state.trigger_effect(&e.effect, 3);
                }
            }

            let enemy_name = state.world.enemies[enemy_idx].name().to_string();
            for behavior in &def.behaviors {
                if behavior.behavior_type == "split_on_death"
                    && let Some(child_id) = &behavior.condition
                {
                    let count = behavior.value.unwrap_or(2) as usize;
                    let mut spawned = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            if spawned >= count {
                                break;
                            }
                            let nx = enemy_x + dx;
                            let ny = enemy_y + dy;
                            if state
                                .world
                                .map
                                .get(nx, ny)
                                .map(|t| t.walkable())
                                .unwrap_or(false)
                                && state.enemy_at(nx, ny).is_none()
                                && !(nx == state.player_x() && ny == state.player_y())
                            {
                                state.world.enemies.push(Enemy::new(nx, ny, child_id));
                                state
                                    .spatial.enemy_positions
                                    .insert((nx, ny), state.world.enemies.len() - 1);
                                spawned += 1;
                            }
                        }
                    }
                    if spawned > 0 {
                        state.log_typed(
                            format!("The {} splits into smaller forms!", enemy_name),
                            MsgType::Combat,
                        );
                    }
                }
            }
        }
    }

    /// Trigger aggro for all nearby enemies of the same type (swarm behavior)
    pub fn trigger_swarm_aggro(
        state: &mut GameState,
        target_id: &str,
        center_x: i32,
        center_y: i32,
        range: i32,
    ) {
        let mut alerted_count = 0;
        for enemy in &mut state.world.enemies {
            if enemy.id == target_id && !enemy.provoked {
                let dist = (enemy.x - center_x).abs() + (enemy.y - center_y).abs();
                if dist <= range {
                    enemy.provoked = true;
                    alerted_count += 1;
                }
            }
        }
        if alerted_count > 0 {
            state.log_typed("The swarm is alerted!", MsgType::Combat);
        }
    }
}
