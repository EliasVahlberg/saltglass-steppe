use super::System;
use crate::game::{
    state::{GameState, MsgType},
    event::GameEvent,
};
use bracket_pathfinding::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Trait for AI behaviors
pub trait AiBehavior: Send + Sync {
    /// Execute behavior for a single enemy
    /// Returns true if the enemy took an action (ending its turn)
    fn execute(&self, entity_idx: usize, state: &mut GameState) -> bool;
}

pub struct AiSystem;

impl System for AiSystem {
    fn update(&self, state: &mut GameState) {
        // We need to iterate indices because we mutate state
        // and we can't hold a reference to the enemy while mutating state
        let enemy_count = state.enemies.len();
        
        for i in 0..enemy_count {
            // Skip dead enemies
            if state.enemies[i].hp <= 0 { continue; }
            
            // Skip if AI disabled
            if state.enemies[i].ai_disabled { continue; }
            
            // Determine behavior from enemy definition
            let behavior_id = state.enemies[i].def()
                .and_then(|d| d.behavior_id.as_deref())
                .unwrap_or("standard_melee");
            
            if let Some(behavior) = BEHAVIOR_REGISTRY.get(behavior_id) {
                behavior.execute(i, state);
            } else {
                // Fallback to standard behavior if specified behavior not found
                StandardMeleeBehavior.execute(i, state);
            }
        }
    }

    fn on_event(&self, _state: &mut GameState, _event: &GameEvent) {
        // Handle AI related events
    }
}

impl AiSystem {
    pub fn update_enemies(state: &mut GameState) {
        let system = AiSystem;
        system.update(state);
    }
}

// --- Behaviors ---

struct StandardMeleeBehavior;

impl AiBehavior for StandardMeleeBehavior {
    fn execute(&self, i: usize, state: &mut GameState) -> bool {
        // Extract enemy properties, using data-driven behavior flags
        let (ex, ey, sight, attack_range, has_ranged, laser_damage, attacks) = {
            let e = &state.enemies[i];
            let def = e.def();
            
            // Check for laser_beam behavior in enemy definition
            let laser_dmg = def.as_ref()
                .and_then(|d| d.behaviors.iter()
                    .find(|b| b.behavior_type == "laser_beam")
                    .and_then(|b| b.value.or(b.damage)))
                .map(|v| v as i32)
                .unwrap_or(0);
            
            (
                e.x, e.y,
                def.map(|d| d.sight_range).unwrap_or(6),
                def.map(|d| d.attack_range).unwrap_or(1),
                def.map(|d| d.ranged_attack).unwrap_or(false),
                laser_dmg,
                1, // Attacks count - could also be data-driven
            )
        };
        
        let has_laser = laser_damage > 0;
        
        // First pass: handle spawners and AOE warnings
        // This logic was in the outer loop in ai.rs, but we can put it here for now
        // or move it to a separate "PreTurnBehavior"
        
        // Handle spawners
        if state.enemies[i].can_spawn(state.turn) {
            if let Some(def) = state.enemies[i].def() {
                if !def.spawn_types.is_empty() {
                    let spawn_type = &def.spawn_types[state.rng.gen_range(0..def.spawn_types.len())];
                    
                    // Find nearby spawn location
                    let mut spawned = false;
                    for dx in -2..=2 {
                        for dy in -2..=2 {
                            if dx == 0 && dy == 0 { continue; }
                            let sx = ex + dx;
                            let sy = ey + dy;
                            
                            if state.map.get(sx, sy).map(|t| t.walkable()).unwrap_or(false) 
                                && state.enemy_at(sx, sy).is_none() 
                                && !(sx == state.player_x && sy == state.player_y) {
                                
                                let mut new_enemy = crate::game::enemy::Enemy::new(sx, sy, spawn_type);
                                if def.swarm {
                                    new_enemy.swarm_id = Some(format!("spawner_{}", i));
                                }
                                
                                state.enemies.push(new_enemy);
                                // Update spatial index for new enemy
                                state.enemy_positions.insert((sx, sy), state.enemies.len() - 1);
                                state.enemies[i].spawned_count += 1;
                                state.enemies[i].last_spawn_turn = state.turn;
                                
                                state.log_typed(format!("{} spawns a {}!", state.enemies[i].name(), spawn_type), MsgType::Combat);
                                state.trigger_effect("S(@3 &LightCyan &White)", 2);
                                spawned = true;
                                break;
                            }
                        }
                        if spawned { break; }
                    }
                }
            }
        }
        
        // Handle AOE warning countdown
        if state.enemies[i].tick_aoe_warning() {
            // AOE attack is ready to execute
            if let Some((target_x, target_y)) = state.enemies[i].aoe_target {
                if let Some(def) = state.enemies[i].def() {
                    let radius = def.aoe_radius as i32;
                    let damage = state.rng.gen_range(def.damage_min..=def.damage_max);
                    
                    state.log_typed(format!("{} unleashes an area attack!", state.enemies[i].name()), MsgType::Combat);
                    
                    // Check if player is in AOE
                    let player_dist = ((state.player_x - target_x).pow(2) + (state.player_y - target_y).pow(2)) as f32;
                    if player_dist <= (radius as f32).powi(2) {
                        let player_armor = state.effective_armor();
                        let final_damage = (damage - player_armor).max(1);
                        state.player_hp -= final_damage;
                        state.trigger_hit_flash(state.player_x, state.player_y);
                        state.spawn_damage_number(state.player_x, state.player_y, final_damage, false);
                        state.log_typed(format!("You take {} damage from the area attack!", final_damage), MsgType::Combat);
                    }
                    
                    // Visual effect
                    state.trigger_effect(&format!("B(@{} &LightRed)", radius), 4);
                    
                    // Reset AOE state
                    state.enemies[i].aoe_target = None;
                    state.enemies[i].aoe_warning_turns = 0;
                }
            }
        }
        
        // Main AI Logic
        let dist = (ex - state.player_x).abs() + (ey - state.player_y).abs();
        
        // Check behaviors from def
        if let Some(def) = state.enemies[i].def() {
            let ctx = crate::game::enemy::BehaviorContext {
                player_adaptations: state.adaptations.len(),
                player_items: &state.inventory,
            };
            
            for behavior in &def.behaviors {
                if !behavior.condition_met(&ctx) { continue; }
                
                match behavior.behavior_type.as_str() {
                    "teleport" => {
                        // Teleport away if too close
                        if dist <= 2 {
                            let range = behavior.value.unwrap_or(5) as i32;
                            for _ in 0..10 {
                                let dx = state.rng.gen_range(-range..=range);
                                let dy = state.rng.gen_range(-range..=range);
                                let nx = ex + dx;
                                let ny = ey + dy;
                                if state.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false) 
                                    && state.enemy_at(nx, ny).is_none() 
                                    && !(nx == state.player_x && ny == state.player_y) {
                                    state.enemy_positions.remove(&(ex, ey));
                                    state.enemies[i].x = nx;
                                    state.enemies[i].y = ny;
                                    state.enemy_positions.insert((nx, ny), i);
                                    state.log_typed(format!("{} teleports away!", state.enemies[i].name()), MsgType::Combat);
                                    return true;
                                }
                            }
                        }
                    }
                    "suicide_bomber" => {
                        // Explode when adjacent to player
                        if dist == 1 {
                            let bomb_damage = behavior.damage.unwrap_or(8) as i32;
                            let bomb_radius = behavior.value.unwrap_or(2) as i32;
                            let player_armor = state.effective_armor();
                            
                            let final_damage = (bomb_damage - player_armor).max(1);
                            state.player_hp -= final_damage;
                            state.trigger_hit_flash(state.player_x, state.player_y);
                            state.spawn_damage_number(state.player_x, state.player_y, final_damage, false);
                            state.log_typed(format!("{} explodes for {} damage!", state.enemies[i].name(), final_damage), MsgType::Combat);
                            
                            // Visual effect
                            state.trigger_effect(&format!("B(@{} &LightRed)", bomb_radius), 4);
                            
                            // Kill the bomber
                            state.enemies[i].hp = 0;
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Demeanor-based behavior
        let mut is_passive = false;
        let mut should_flee = false;
        if !state.enemies[i].is_hostile() { is_passive = true; }
        if state.enemies[i].should_flee() { should_flee = true; }
        
        if is_passive { return false; }
        
        // Check if preparing AOE attack
        if state.enemies[i].is_preparing_aoe() {
            // Show warning effect
            if let Some((_target_x, _target_y)) = state.enemies[i].aoe_target {
                state.trigger_effect("F(@2 &Red)", 1);
                if state.enemies[i].aoe_warning_turns == 1 {
                    state.log_typed(format!("{} is preparing a devastating attack!", state.enemies[i].name()), MsgType::Warning);
                }
            }
            return true; // Don't move while preparing AOE
        }
        
        if should_flee && dist < sight {
            // Flee away from player
            let dx = (ex - state.player_x).signum();
            let dy = (ey - state.player_y).signum();
            let nx = ex + dx;
            let ny = ey + dy;
            if state.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false) 
                && state.enemy_at(nx, ny).is_none() 
                && !(nx == state.player_x && ny == state.player_y) {
                state.enemy_positions.remove(&(ex, ey));
                state.enemies[i].x = nx;
                state.enemies[i].y = ny;
                state.enemy_positions.insert((nx, ny), i);
            }
            return true;
        }
        
        // Check for nearby decoys - 50% chance to target decoy instead
        let mut target_x = state.player_x;
        let mut target_y = state.player_y;
        let mut target_is_decoy = false;
        
        // We need to clone decoys to iterate because we might mutate state
        let decoys = state.decoys.clone();
        for decoy in &decoys {
            let decoy_dist = (decoy.x - ex).abs() + (decoy.y - ey).abs();
            if decoy_dist < sight && state.rng.gen_bool(0.5) {
                target_x = decoy.x;
                target_y = decoy.y;
                target_is_decoy = true;
                break;
            }
        }
        let target_dist = (target_x - ex).abs() + (target_y - ey).abs();
        
        // Ranged attack check
        if has_ranged && target_dist <= attack_range as i32 && target_dist > 1 && !target_is_decoy {
            let Some(def) = state.enemies[i].def() else { return true; };
            let base_dmg = state.rng.gen_range(def.damage_min..=def.damage_max);
            let player_armor = state.effective_armor();
            let dmg = (base_dmg - player_armor).max(1);
            state.player_hp -= dmg;
            state.trigger_hit_flash(state.player_x, state.player_y);
            state.spawn_damage_number(state.player_x, state.player_y, dmg, false);
            state.log_typed(format!("{} fires a ranged attack for {} damage!", state.enemies[i].name(), dmg), MsgType::Combat);
            
            // Visual effect for ranged attack
            state.spawn_beam((ex, ey), (state.player_x, state.player_y), crate::game::state::BeamType::Arrow, 6);
            
            return true;
        }
        
        // AOE attack initiation
        if let Some(def) = state.enemies[i].def() {
            if def.aoe_attack && target_dist <= sight && target_dist <= 4 && !target_is_decoy && state.enemies[i].aoe_target.is_none() {
                state.enemies[i].start_aoe_attack(target_x, target_y);
                state.log_typed(format!("{} begins charging an area attack!", state.enemies[i].name()), MsgType::Warning);
                return true;
            }
        }
        
        if target_dist == 1 {
            if target_is_decoy {
                // Attack decoy - it dissipates
                state.decoys.retain(|d| !(d.x == target_x && d.y == target_y));
                let dir = state.direction_from(ex, ey);
                state.log_typed(format!("{} {} attacks your decoy!", state.enemies[i].name(), dir), MsgType::Combat);
            } else {
                // Attack player (possibly multiple times)
                let Some(def) = state.enemies[i].def() else { return true; };
                for _ in 0..attacks {
                    let base_dmg = state.rng.gen_range(def.damage_min..=def.damage_max);
                    let player_armor = state.effective_armor();
                    let dmg = (base_dmg - player_armor).max(1);
                    state.player_hp -= dmg;
                    state.trigger_hit_flash(state.player_x, state.player_y);
                    state.spawn_damage_number(state.player_x, state.player_y, dmg, false);
                    let dir = state.direction_from(ex, ey);
                    state.log_typed(format!("{} {} attacks you for {} damage!", state.enemies[i].name(), dir, dmg), MsgType::Combat);
                    
                    // Trigger on_hit effects
                    for e in &def.effects {
                        if e.condition == "on_hit" {
                            state.trigger_effect(&e.effect, 2);
                        }
                    }
                    
                    // Check on_hit behaviors
                    for behavior in &def.behaviors {
                        if behavior.behavior_type == "on_hit_refraction" {
                            if let Some(val) = behavior.value {
                                state.refraction += val;
                                state.log_typed(format!("Glass shards pierce you. (+{} Refraction)", val), MsgType::Status);
                                state.check_adaptation_threshold();
                            }
                        }
                    }
                    
                    if state.player_hp <= 0 { return true; }
                }
            }
        } else if target_dist < sight {
            // Laser beam check
            if has_laser && !target_is_decoy && state.visible.contains(&state.map.idx(ex, ey)) {
                // Fire laser
                state.player_hp -= laser_damage;
                state.trigger_hit_flash(state.player_x, state.player_y);
                state.spawn_damage_number(state.player_x, state.player_y, laser_damage, false);
                state.log_typed(format!("{} fires a laser beam for {} damage!", state.enemies[i].name(), laser_damage), MsgType::Combat);
                
                // Visual effect for beam
                state.spawn_beam((ex, ey), (state.player_x, state.player_y), crate::game::state::BeamType::Laser, 8);
                
                return true;
            }

            // Move toward target (player or decoy)
            let enemy_idx = state.map.idx(ex, ey);
            let target_idx = state.map.idx(target_x, target_y);
            let (nx, ny) = if state.visible.contains(&enemy_idx) {
                // Visible: use A* pathfinding
                let path = a_star_search(enemy_idx, target_idx, &state.map);
                if path.success && path.steps.len() > 1 {
                    let next = path.steps[1];
                    ((next % state.map.width) as i32, (next / state.map.width) as i32)
                } else {
                    return false;
                }
            } else {
                // Not visible: simple directional movement toward target
                let dx = (target_x - ex).signum();
                let dy = (target_y - ey).signum();
                (ex + dx, ey + dy)
            };
            
            if state.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false)
                && state.enemy_at(nx, ny).is_none() 
                && !(nx == state.player_x && ny == state.player_y) {
                state.enemy_positions.remove(&(ex, ey));
                state.enemies[i].x = nx;
                state.enemies[i].y = ny;
                state.enemy_positions.insert((nx, ny), i);
            }
        }
        
        true
    }
}

/// Ranged-only behavior: maintains distance and fires ranged attacks
struct RangedOnlyBehavior;

impl AiBehavior for RangedOnlyBehavior {
    fn execute(&self, i: usize, state: &mut GameState) -> bool {
        let ex = state.enemies[i].x;
        let ey = state.enemies[i].y;
        let px = state.player_x;
        let py = state.player_y;
        let dist = (ex - px).abs() + (ey - py).abs();
        
        let sight = state.enemies[i].def().map(|d| d.sight_range).unwrap_or(6);
        let attack_range = state.enemies[i].def().map(|d| d.attack_range).unwrap_or(4) as i32;
        let min_range = 3; // Try to stay at least this far away
        
        // Only act if player in sight
        if dist > sight { return true; }
        
        // If too close, retreat
        if dist < min_range {
            let dx = (ex - px).signum();
            let dy = (ey - py).signum();
            let nx = ex + dx;
            let ny = ey + dy;
            
            if state.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false)
                && state.enemy_at(nx, ny).is_none() 
                && !(nx == px && ny == py) {
                state.enemy_positions.remove(&(ex, ey));
                state.enemies[i].x = nx;
                state.enemies[i].y = ny;
                state.enemy_positions.insert((nx, ny), i);
                return true;
            }
        }
        
        // If in range, fire
        if dist <= attack_range {
            if let Some(def) = state.enemies[i].def() {
                let dmg = state.rng.gen_range(def.damage_min..=def.damage_max);
                state.player_hp -= dmg;
                state.log_typed(
                    format!("{} shoots you for {} damage!", state.enemies[i].name(), dmg),
                    MsgType::Combat
                );
                state.spawn_damage_number(px, py, dmg, true);
            }
        }
        true
    }
}

/// Suicide bomber: rushes player and explodes on contact
struct SuicideBomberBehavior;

impl AiBehavior for SuicideBomberBehavior {
    fn execute(&self, i: usize, state: &mut GameState) -> bool {
        let ex = state.enemies[i].x;
        let ey = state.enemies[i].y;
        let px = state.player_x;
        let py = state.player_y;
        let dist = (ex - px).abs() + (ey - py).abs();
        
        let sight = state.enemies[i].def().map(|d| d.sight_range).unwrap_or(8);
        
        if dist > sight { return true; }
        
        // Adjacent to player - explode!
        if dist == 1 {
            let bomb_damage = state.enemies[i].def()
                .and_then(|d| d.behaviors.iter().find(|b| b.behavior_type == "suicide_bomber"))
                .and_then(|b| b.damage)
                .unwrap_or(15) as i32;
            
            state.player_hp -= bomb_damage;
            state.log_typed(
                format!("{} explodes for {} damage!", state.enemies[i].name(), bomb_damage),
                MsgType::Combat
            );
            state.spawn_damage_number(px, py, bomb_damage, true);
            
            // Kill self
            state.enemies[i].hp = 0;
            state.enemy_positions.remove(&(ex, ey));
            return true;
        }
        
        // Rush toward player
        let dx = (px - ex).signum();
        let dy = (py - ey).signum();
        let nx = ex + dx;
        let ny = ey + dy;
        
        if state.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false)
            && state.enemy_at(nx, ny).is_none() 
            && !(nx == px && ny == py) {
            state.enemy_positions.remove(&(ex, ey));
            state.enemies[i].x = nx;
            state.enemies[i].y = ny;
            state.enemy_positions.insert((nx, ny), i);
        }
        true
    }
}

/// Healer behavior: heals nearby allies instead of attacking
struct HealerBehavior;

impl AiBehavior for HealerBehavior {
    fn execute(&self, i: usize, state: &mut GameState) -> bool {
        let ex = state.enemies[i].x;
        let ey = state.enemies[i].y;
        let heal_range = 4;
        let heal_amount = 5;
        
        // Find injured nearby ally
        let mut healed_idx = None;
        let mut lowest_hp_ratio = 1.0f32;
        
        for (idx, enemy) in state.enemies.iter().enumerate() {
            if idx == i || enemy.hp <= 0 { continue; }
            
            let dist = (enemy.x - ex).abs() + (enemy.y - ey).abs();
            if dist > heal_range { continue; }
            
            if let Some(def) = enemy.def() {
                let ratio = enemy.hp as f32 / def.max_hp as f32;
                if ratio < lowest_hp_ratio && ratio < 1.0 {
                    lowest_hp_ratio = ratio;
                    healed_idx = Some(idx);
                }
            }
        }
        
        if let Some(idx) = healed_idx {
            let target_name = state.enemies[idx].name().to_string();
            if let Some(def) = state.enemies[idx].def() {
                state.enemies[idx].hp = (state.enemies[idx].hp + heal_amount).min(def.max_hp);
            }
            state.log_typed(
                format!("{} heals {} for {} HP!", state.enemies[i].name(), target_name, heal_amount),
                MsgType::Combat
            );
            return true;
        }
        
        // No one to heal - fall back to standard behavior
        StandardMeleeBehavior.execute(i, state)
    }
}

static BEHAVIOR_REGISTRY: Lazy<HashMap<String, Box<dyn AiBehavior>>> = Lazy::new(|| {
    let mut m: HashMap<String, Box<dyn AiBehavior>> = HashMap::new();
    m.insert("standard_melee".to_string(), Box::new(StandardMeleeBehavior));
    m.insert("ranged_only".to_string(), Box::new(RangedOnlyBehavior));
    m.insert("suicide_bomber".to_string(), Box::new(SuicideBomberBehavior));
    m.insert("healer".to_string(), Box::new(HealerBehavior));
    m
});
