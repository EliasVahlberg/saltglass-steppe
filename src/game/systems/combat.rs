use super::System;
use crate::game::{
    action::action_cost,
    adaptation::total_stat_modifiers,
    combat::{CombatResult, default_weapon, get_weapon_def, roll_attack},
    enemy::Enemy,
    event::GameEvent,
    state::{GameState, MsgType},
};

pub struct CombatSystem;

impl System for CombatSystem {
    fn update(&self, _state: &mut GameState) {
        // Combat system doesn't have a per-turn update loop yet
    }

    fn on_event(&self, _state: &mut GameState, _event: &GameEvent) {
        // Handle combat related events
    }
}

impl CombatSystem {
    /// Process enemy death: effects, XP, loot, split behavior, quest tracking
    /// Returns the enemy name for logging purposes
    fn process_enemy_death(
        state: &mut GameState,
        enemy_idx: usize,
        death_x: i32,
        death_y: i32,
    ) -> String {
        let enemy_id = state.world.enemies[enemy_idx].id.clone();
        let enemy_name = state.world.enemies[enemy_idx].name().to_string();
        let enemy_x = state.world.enemies[enemy_idx].x;
        let enemy_y = state.world.enemies[enemy_idx].y;

        // Remove from spatial index
        state.enemy_positions.remove(&(death_x, death_y));

        if let Some(def) = state.world.enemies[enemy_idx].def() {
            // Trigger on_death visual effects
            for e in &def.effects {
                if e.condition == "on_death" {
                    state.trigger_effect(&e.effect, 3);
                }
            }

            // Award XP
            if def.xp_value > 0 {
                state.gain_xp(def.xp_value);
            }

            // Handle split_on_death behavior
            for behavior in &def.behaviors {
                if behavior.behavior_type == "split_on_death" {
                    if let Some(child_id) = &behavior.condition {
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

                                if state.world.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false)
                                    && state.enemy_at(nx, ny).is_none()
                                    && !(nx == state.player_x() && ny == state.player_y())
                                {
                                    state.world.enemies.push(Enemy::new(nx, ny, child_id));
                                    state
                                        .enemy_positions
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

        // Emit event - LootSystem and QuestSystem handle loot drops and quest progress
        state.emit(GameEvent::EnemyKilled {
            enemy_id: enemy_id.clone(),
            x: death_x,
            y: death_y,
        });
        state.meta.discover_enemy(&enemy_id);

        enemy_name
    }

    /// Apply mock settings to combat result if configured
    fn apply_combat_mocks(state: &GameState, mut result: CombatResult) -> CombatResult {
        if let Some(force_hit) = state.mock_combat_hit {
            result.hit = force_hit;
            if !force_hit {
                result.damage = 0;
            }
        }
        if let Some(dmg) = state.mock_combat_damage {
            if result.hit {
                result.damage = dmg;
            }
        }
        result
    }

    /// Trigger aggro for all nearby enemies of the same type (swarm behavior)
    fn trigger_swarm_aggro(
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

    pub fn attack_melee(state: &mut GameState, target_x: i32, target_y: i32) -> bool {
        let ei = match state.enemy_at(target_x, target_y) {
            Some(i) => i,
            None => return false,
        };

        let cost = action_cost("attack_melee");
        if state.player.ap < cost {
            return false;
        }
        state.player.ap -= cost;

        state.world.enemies[ei].provoked = true;

        // Swarm behavior
        if state.world.enemies[ei].def().map(|d| d.swarm).unwrap_or(false) {
            let id = state.world.enemies[ei].id.clone();
            let x = state.world.enemies[ei].x;
            let y = state.world.enemies[ei].y;
            Self::trigger_swarm_aggro(state, &id, x, y, 8);
        }

        let weapon = state
            .player.equipped_weapon
            .as_ref()
            .and_then(|id| get_weapon_def(id))
            .unwrap_or_else(default_weapon);

        let enemy_reflex = state.world.enemies[ei].def().map(|d| d.reflex).unwrap_or(0);
        let enemy_armor = state.world.enemies[ei].def().map(|d| d.armor).unwrap_or(0);

        // Get skill bonuses
        let accuracy_bonus = state.player.skills.passive_bonuses.get("melee_accuracy_bonus").copied().unwrap_or(0.0);
        let damage_bonus = state.player.skills.passive_bonuses.get("melee_damage_bonus").copied().unwrap_or(0.0);

        // Apply accuracy bonus to hit chance
        let cover_bonus = -(accuracy_bonus * 100.0) as i32;
        let result = roll_attack(&mut state.rng, weapon, enemy_reflex, enemy_armor, cover_bonus);
        let result = Self::apply_combat_mocks(state, result);
        let name = state.world.enemies[ei].name().to_string();
        let dir = state.direction_from(target_x, target_y);

        if !result.hit {
            state.log_typed(format!("You miss the {} {}.", name, dir), MsgType::Combat);
            return true;
        }

        let mut dmg = result.damage;
        // Apply skill damage bonus
        dmg = (dmg as f32 * (1.0 + damage_bonus)) as i32;
        // Apply adaptation damage bonus
        let adapt_mods = total_stat_modifiers(&state.player.adaptations);
        dmg += adapt_mods.damage_bonus;
        state.world.enemies[ei].hp -= dmg;
        state.emit(GameEvent::EnemyDamaged { enemy_idx: ei, amount: dmg });
        state.trigger_hit_flash(target_x, target_y);
        state.spawn_damage_number(target_x, target_y, dmg, false);

        if let Some(def) = state.world.enemies[ei].def() {
            for e in &def.effects {
                if e.condition == "on_hit" {
                    state.trigger_effect(&e.effect, 2);
                }
            }
            for behavior in &def.behaviors {
                if behavior.behavior_type == "reflect_damage" {
                    let percent = behavior.percent.unwrap_or(25);
                    let reflected = (dmg as u32 * percent / 100) as i32;
                    if reflected > 0 {
                        state.player.hp -= reflected;
                        state.log_typed(
                            format!("The enemy reflects {} damage back at you!", reflected),
                            MsgType::Combat,
                        );
                    }
                }
            }
        }

        state.player.last_damage_dealt = dmg as u32;

        if state.world.enemies[ei].hp <= 0 {
            let enemy_name = Self::process_enemy_death(state, ei, target_x, target_y);
            state.log_typed(
                format!("You kill the {} {}!", enemy_name, dir),
                MsgType::Combat,
            );
        } else {
            let crit_str = if result.crit { " CRITICAL!" } else { "" };
            state.log_typed(
                format!(
                    "You hit the {} {} for {} damage.{}",
                    name, dir, dmg, crit_str
                ),
                MsgType::Combat,
            );
        }
        true
    }

    pub fn ranged_attack(state: &mut GameState, target_x: i32, target_y: i32) -> bool {
        let weapon = match state
            .player.equipped_weapon
            .as_ref()
            .and_then(|id| get_weapon_def(id))
        {
            Some(w) if w.range > 1 => w,
            _ => {
                state.log_typed("No ranged weapon equipped.", MsgType::Combat);
                return false;
            }
        };

        let dist = (target_x - state.player_x()).abs() + (target_y - state.player_y()).abs();
        if dist > weapon.range {
            state.log_typed("Target out of range.", MsgType::Combat);
            return false;
        }

        let target_idx = state.world.map.idx(target_x, target_y);
        if !state.visible.contains(&target_idx) {
            state.log_typed("Can't see target.", MsgType::Combat);
            return false;
        }

        let cost = weapon.ap_cost;
        if state.player.ap < cost {
            return false;
        }

        if let Some(ammo_type) = &weapon.ammo_type {
            if !state.player.inventory.iter().any(|id| id == ammo_type) {
                state.log_typed(
                    format!("Out of {}.", ammo_type.replace('_', " ")),
                    MsgType::Combat,
                );
                return false;
            }
            if let Some(idx) = state.player.inventory.iter().position(|id| id == ammo_type) {
                state.player.inventory.remove(idx);
            }
        }

        state.player.ap -= cost;

        // Spawn projectile trail
        let proj_char = if weapon.range > 3 { '*' } else { '-' };
        state.spawn_projectile(
            (state.player_x(), state.player_y()),
            (target_x, target_y),
            proj_char,
        );

        let ei = match state.enemy_at(target_x, target_y) {
            Some(i) => i,
            None => {
                state.log_typed("No target there.", MsgType::Combat);
                state.check_auto_end_turn();
                return true;
            }
        };

        state.world.enemies[ei].provoked = true;

        // Swarm behavior
        if state.world.enemies[ei].def().map(|d| d.swarm).unwrap_or(false) {
            let id = state.world.enemies[ei].id.clone();
            let x = state.world.enemies[ei].x;
            let y = state.world.enemies[ei].y;
            Self::trigger_swarm_aggro(state, &id, x, y, 8);
        }

        let enemy_reflex = state.world.enemies[ei].def().map(|d| d.reflex).unwrap_or(0);
        let enemy_armor = state.world.enemies[ei].def().map(|d| d.armor).unwrap_or(0);
        
        // Get skill bonuses
        let accuracy_bonus = state.player.skills.passive_bonuses.get("ranged_accuracy_bonus").copied().unwrap_or(0.0);
        let damage_bonus = state.player.skills.passive_bonuses.get("ranged_damage_bonus").copied().unwrap_or(0.0);

        // Apply accuracy bonus to hit chance
        let cover_bonus = -(accuracy_bonus * 100.0) as i32;
        let result = roll_attack(&mut state.rng, weapon, enemy_reflex, enemy_armor, cover_bonus);
        let result = Self::apply_combat_mocks(state, result);
        let name = state.world.enemies[ei].name().to_string();

        if !result.hit {
            state.log_typed(format!("Your shot misses the {}.", name), MsgType::Combat);
            state.check_auto_end_turn();
            return true;
        }

        let mut dmg = result.damage;
        // Apply skill damage bonus
        dmg = (dmg as f32 * (1.0 + damage_bonus)) as i32;
        state.world.enemies[ei].hp -= dmg;
        state.emit(GameEvent::EnemyDamaged { enemy_idx: ei, amount: dmg });
        state.trigger_hit_flash(target_x, target_y);
        state.spawn_damage_number(target_x, target_y, dmg, false);

        if state.world.enemies[ei].hp <= 0 {
            let enemy_name = Self::process_enemy_death(state, ei, target_x, target_y);
            state.log_typed(
                format!("You kill the {} with a ranged shot!", enemy_name),
                MsgType::Combat,
            );
        } else {
            let crit_str = if result.crit { " CRITICAL!" } else { "" };
            state.log_typed(
                format!("You hit the {} for {} damage.{}", name, dmg, crit_str),
                MsgType::Combat,
            );
        }

        state.check_auto_end_turn();
        true
    }
}
