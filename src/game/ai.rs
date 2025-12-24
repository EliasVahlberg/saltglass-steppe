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
        let current_turn = self.turn;
        
        // First pass: handle spawners and AOE warnings
        for i in 0..self.enemies.len() {
            if self.enemies[i].hp <= 0 { continue; }
            
            // Handle spawners
            if self.enemies[i].can_spawn(current_turn) {
                if let Some(def) = self.enemies[i].def() {
                    if !def.spawn_types.is_empty() {
                        let spawn_type = &def.spawn_types[self.rng.gen_range(0..def.spawn_types.len())];
                        
                        // Find nearby spawn location
                        for dx in -2..=2 {
                            for dy in -2..=2 {
                                if dx == 0 && dy == 0 { continue; }
                                let sx = self.enemies[i].x + dx;
                                let sy = self.enemies[i].y + dy;
                                
                                if self.map.get(sx, sy).map(|t| t.walkable()).unwrap_or(false) 
                                    && self.enemy_at(sx, sy).is_none() 
                                    && !(sx == px && sy == py) {
                                    
                                    let mut new_enemy = super::enemy::Enemy::new(sx, sy, spawn_type);
                                    if def.swarm {
                                        new_enemy.swarm_id = Some(format!("spawner_{}", i));
                                    }
                                    
                                    self.enemies.push(new_enemy);
                                    self.enemies[i].spawned_count += 1;
                                    self.enemies[i].last_spawn_turn = current_turn;
                                    
                                    self.log_typed(format!("{} spawns a {}!", self.enemies[i].name(), spawn_type), MsgType::Combat);
                                    self.trigger_effect("S(@3 &LightCyan &White)", 2);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            
            // Handle AOE warning countdown
            if self.enemies[i].tick_aoe_warning() {
                // AOE attack is ready to execute
                if let Some((target_x, target_y)) = self.enemies[i].aoe_target {
                    if let Some(def) = self.enemies[i].def() {
                        let radius = def.aoe_radius as i32;
                        let damage = self.rng.gen_range(def.damage_min..=def.damage_max);
                        
                        self.log_typed(format!("{} unleashes an area attack!", self.enemies[i].name()), MsgType::Combat);
                        
                        // Check if player is in AOE
                        let player_dist = ((px - target_x).pow(2) + (py - target_y).pow(2)) as f32;
                        if player_dist <= (radius as f32).powi(2) {
                            let final_damage = (damage - player_armor).max(1);
                            self.player_hp -= final_damage;
                            self.trigger_hit_flash(px, py);
                            self.spawn_damage_number(px, py, final_damage, false);
                            self.log_typed(format!("You take {} damage from the area attack!", final_damage), MsgType::Combat);
                        }
                        
                        // Visual effect
                        self.trigger_effect(&format!("B(@{} &LightRed)", radius), 4);
                        
                        // Reset AOE state
                        self.enemies[i].aoe_target = None;
                        self.enemies[i].aoe_warning_turns = 0;
                    }
                }
            }
        }
        
        // Second pass: regular AI behavior
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
            let has_ranged = def.ranged_attack;
            let attack_range = def.attack_range as i32;

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
                        laser_dmg = behavior.damage.unwrap_or(behavior.value.unwrap_or(5)) as i32;
                    }
                    "fear_aura" => {
                        if dist <= 3 && !self.has_status_effect("fear") {
                            self.apply_status_effect("fear", 2);
                            self.log_typed(format!("The {}'s presence terrifies you!", self.enemies[i].name()), MsgType::Status);
                        }
                    }
                    "kite_enemy" => {
                        // Ranged enemies try to maintain distance
                        if dist <= 2 && has_ranged {
                            should_flee = true;
                        }
                    }
                    "charge_enemy" => {
                        // Bombers charge toward player
                        if dist > 1 && dist <= sight {
                            // Move directly toward player, ignoring obstacles if necessary
                            let dx = (px - ex).signum();
                            let dy = (py - ey).signum();
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
                    }
                    "suicide_bomber" => {
                        // Explode when adjacent to player
                        if dist == 1 {
                            let bomb_damage = behavior.damage.unwrap_or(8) as i32;
                            let bomb_radius = behavior.value.unwrap_or(2) as i32;
                            
                            let final_damage = (bomb_damage - player_armor).max(1);
                            self.player_hp -= final_damage;
                            self.trigger_hit_flash(px, py);
                            self.spawn_damage_number(px, py, final_damage, false);
                            self.log_typed(format!("{} explodes for {} damage!", self.enemies[i].name(), final_damage), MsgType::Combat);
                            
                            // Visual effect
                            self.trigger_effect(&format!("B(@{} &LightRed)", bomb_radius), 4);
                            
                            // Kill the bomber
                            self.enemies[i].hp = 0;
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            
            // Demeanor-based behavior
            if !self.enemies[i].is_hostile() { is_passive = true; }
            if self.enemies[i].should_flee() { should_flee = true; }
            
            if is_passive { continue; }
            
            // Check if preparing AOE attack
            if self.enemies[i].is_preparing_aoe() {
                // Show warning effect
                if let Some((_target_x, _target_y)) = self.enemies[i].aoe_target {
                    self.trigger_effect("F(@2 &Red)", 1);
                    if self.enemies[i].aoe_warning_turns == 1 {
                        self.log_typed(format!("{} is preparing a devastating attack!", self.enemies[i].name()), MsgType::Warning);
                    }
                }
                continue; // Don't move while preparing AOE
            }
            
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
            
            // Ranged attack check
            if has_ranged && target_dist <= attack_range && target_dist > 1 && !target_is_decoy {
                let base_dmg = self.rng.gen_range(def.damage_min..=def.damage_max);
                let dmg = (base_dmg - player_armor).max(1);
                self.player_hp -= dmg;
                self.trigger_hit_flash(px, py);
                self.spawn_damage_number(px, py, dmg, false);
                self.log_typed(format!("{} fires a ranged attack for {} damage!", self.enemies[i].name(), dmg), MsgType::Combat);
                
                // Visual effect for ranged attack
                self.spawn_beam((ex, ey), (px, py), super::state::BeamType::Arrow, 6);
                
                if self.player_hp <= 0 { return; }
                continue; // Don't move after ranged attack
            }
            
            // AOE attack initiation
            if def.aoe_attack && target_dist <= sight && target_dist <= 4 && !target_is_decoy && self.enemies[i].aoe_target.is_none() {
                self.enemies[i].start_aoe_attack(target_x, target_y);
                self.log_typed(format!("{} begins charging an area attack!", self.enemies[i].name()), MsgType::Warning);
                continue;
            }
            
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
                    self.spawn_beam((ex, ey), (self.player_x, self.player_y), super::state::BeamType::Laser, 8);
                    
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
