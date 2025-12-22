//! Enemy AI behavior

use bracket_pathfinding::prelude::a_star_search;
use rand::Rng;

use super::adaptation::total_stat_modifiers;
use super::enemy::BehaviorContext;
use super::state::{GameState, MsgType};

impl GameState {
    /// Get effective player armor (base + equipment + adaptations)
    pub fn effective_armor(&self) -> i32 {
        let adapt_mods = total_stat_modifiers(&self.adaptations);
        self.player_armor + adapt_mods.armor
    }

    /// Get effective player reflex (base + adaptations)
    pub fn effective_reflex(&self) -> i32 {
        let adapt_mods = total_stat_modifiers(&self.adaptations);
        self.player_reflex + adapt_mods.reflex
    }

    pub fn update_enemies(&mut self) {
        if self.player_hp <= 0 { return; }
        let px = self.player_x;
        let py = self.player_y;
        let inventory = self.inventory.clone();
        let adaptation_count = self.adaptations.len();
        let player_armor = self.effective_armor();
        let decoys = self.decoys.clone();
        
        for i in 0..self.enemies.len() {
            if self.enemies[i].hp <= 0 { continue; }
            if self.enemies[i].ai_disabled { continue; }
            let ex = self.enemies[i].x;
            let ey = self.enemies[i].y;
            let def = match self.enemies[i].def() {
                Some(d) => d,
                None => continue,
            };
            let sight = def.sight_range;
            let dist = (px - ex).abs() + (py - ey).abs();
            
            // Check behaviors
            let ctx = BehaviorContext {
                player_adaptations: adaptation_count,
                player_items: &inventory,
            };
            let mut is_passive = false;
            let mut should_flee = false;
            let mut attacks = 1;
            let mut laser_dmg = 0;
            let mut has_laser = false;

            for behavior in &def.behaviors {
                match behavior.behavior_type.as_str() {
                    "passive_if" => {
                        if behavior.condition_met(&ctx) { is_passive = true; }
                    }
                    "flee_if" => {
                        if behavior.condition_met(&ctx) { should_flee = true; }
                    }
                    "multiple_attacks" => {
                        attacks = behavior.value.unwrap_or(1) as usize;
                    }
                    "laser_beam" => {
                        has_laser = true;
                        laser_dmg = behavior.value.unwrap_or(5) as i32;
                    }
                    "fear_aura" => {
                        if dist <= 3 && !self.has_status_effect("fear") {
                            self.apply_status_effect("fear", 2);
                            self.log_typed(format!("The {}'s presence terrifies you!", self.enemies[i].name()), MsgType::Status);
                        }
                    }
                    _ => {}
                }
            }
            
            // Demeanor-based behavior
            if !self.enemies[i].is_hostile() { is_passive = true; }
            if self.enemies[i].should_flee() { should_flee = true; }
            
            if is_passive { continue; }
            
            if should_flee && dist < sight {
                // Flee away from player
                let dx = (ex - px).signum();
                let dy = (ey - py).signum();
                let nx = ex + dx;
                let ny = ey + dy;
                if self.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false) 
                    && self.enemy_at(nx, ny).is_none() 
                    && !(nx == px && ny == py) {
                    self.enemy_positions.remove(&(ex, ey));
                    self.enemies[i].x = nx;
                    self.enemies[i].y = ny;
                    self.enemy_positions.insert((nx, ny), i);
                }
                continue;
            }
            
            // Check for nearby decoys - 50% chance to target decoy instead
            let mut target_x = px;
            let mut target_y = py;
            let mut target_is_decoy = false;
            for decoy in &decoys {
                let decoy_dist = (decoy.x - ex).abs() + (decoy.y - ey).abs();
                if decoy_dist < sight && self.rng.gen_bool(0.5) {
                    target_x = decoy.x;
                    target_y = decoy.y;
                    target_is_decoy = true;
                    break;
                }
            }
            let target_dist = (target_x - ex).abs() + (target_y - ey).abs();
            
            if target_dist == 1 {
                if target_is_decoy {
                    // Attack decoy - it dissipates
                    self.decoys.retain(|d| !(d.x == target_x && d.y == target_y));
                    let dir = self.direction_from(ex, ey);
                    self.log_typed(format!("{} {} attacks your decoy!", self.enemies[i].name(), dir), MsgType::Combat);
                } else {
                    // Attack player (possibly multiple times)
                    for _ in 0..attacks {
                        let base_dmg = self.rng.gen_range(def.damage_min..=def.damage_max);
                        let dmg = (base_dmg - player_armor).max(1);
                        self.player_hp -= dmg;
                        self.trigger_hit_flash(self.player_x, self.player_y);
                        self.spawn_damage_number(self.player_x, self.player_y, dmg, false);
                        let dir = self.direction_from(ex, ey);
                        self.log_typed(format!("{} {} attacks you for {} damage!", self.enemies[i].name(), dir, dmg), MsgType::Combat);
                        
                        // Trigger on_hit effects
                        for e in &def.effects {
                            if e.condition == "on_hit" {
                                self.trigger_effect(&e.effect, 2);
                            }
                        }
                        
                        // Check on_hit behaviors
                        for behavior in &def.behaviors {
                            if behavior.behavior_type == "on_hit_refraction" {
                                if let Some(val) = behavior.value {
                                    self.refraction += val;
                                    self.log_typed(format!("Glass shards pierce you. (+{} Refraction)", val), MsgType::Status);
                                    self.check_adaptation_threshold();
                                }
                            }
                        }
                        
                        if self.player_hp <= 0 { return; }
                    }
                }
            } else if target_dist < sight {
                // Laser beam check
                if has_laser && !target_is_decoy && self.visible.contains(&self.map.idx(ex, ey)) {
                    // Fire laser
                    self.player_hp -= laser_dmg;
                    self.trigger_hit_flash(self.player_x, self.player_y);
                    self.spawn_damage_number(self.player_x, self.player_y, laser_dmg, false);
                    self.log_typed(format!("{} fires a laser beam for {} damage!", self.enemies[i].name(), laser_dmg), MsgType::Combat);
                    
                    // Visual effect for beam
                    // We can't easily draw a line here without more helper functions, but we can log it
                    if self.player_hp <= 0 { return; }
                    
                    // Don't move if fired laser
                    continue;
                }

                // Move toward target (player or decoy)
                let enemy_idx = self.map.idx(ex, ey);
                let target_idx = self.map.idx(target_x, target_y);
                let (nx, ny) = if self.visible.contains(&enemy_idx) {
                    // Visible: use A* pathfinding
                    let path = a_star_search(enemy_idx, target_idx, &self.map);
                    if path.success && path.steps.len() > 1 {
                        let next = path.steps[1];
                        ((next % self.map.width) as i32, (next / self.map.width) as i32)
                    } else {
                        continue;
                    }
                } else {
                    // Not visible: simple directional movement toward target
                    let dx = (target_x - ex).signum();
                    let dy = (target_y - ey).signum();
                    (ex + dx, ey + dy)
                };
                
                if self.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false)
                    && self.enemy_at(nx, ny).is_none() 
                    && !(nx == px && ny == py) {
                    self.enemy_positions.remove(&(ex, ey));
                    self.enemies[i].x = nx;
                    self.enemies[i].y = ny;
                    self.enemy_positions.insert((nx, ny), i);
                }
            }
        }
    }
}
