//! Enemy AI behavior

use bracket_pathfinding::prelude::a_star_search;
use rand::Rng;

use super::enemy::BehaviorContext;
use super::state::GameState;

impl GameState {
    pub fn update_enemies(&mut self) {
        if self.player_hp <= 0 { return; }
        let px = self.player_x;
        let py = self.player_y;
        let inventory = self.inventory.clone();
        let adaptation_count = self.adaptations.len();
        
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
            for behavior in &def.behaviors {
                match behavior.behavior_type.as_str() {
                    "passive_if" => {
                        if behavior.condition_met(&ctx) { is_passive = true; }
                    }
                    "flee_if" => {
                        if behavior.condition_met(&ctx) { should_flee = true; }
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
            
            if dist == 1 {
                let dmg = self.rng.gen_range(def.damage_min..=def.damage_max);
                self.player_hp -= dmg;
                let dir = self.direction_from(ex, ey);
                self.log(format!("{} {} attacks you for {} damage!", self.enemies[i].name(), dir, dmg));
                
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
                            self.log(format!("Glass shards pierce you. (+{} Refraction)", val));
                            self.check_adaptation_threshold();
                        }
                    }
                }
                
                if self.player_hp <= 0 { return; }
            } else if dist < sight {
                let enemy_idx = self.map.idx(ex, ey);
                let (nx, ny) = if self.visible.contains(&enemy_idx) {
                    // Visible: use A* pathfinding
                    let path = a_star_search(enemy_idx, self.map.idx(px, py), &self.map);
                    if path.success && path.steps.len() > 1 {
                        let next = path.steps[1];
                        ((next % self.map.width) as i32, (next / self.map.width) as i32)
                    } else {
                        continue;
                    }
                } else {
                    // Not visible: simple directional movement toward player
                    let dx = (px - ex).signum();
                    let dy = (py - ey).signum();
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
