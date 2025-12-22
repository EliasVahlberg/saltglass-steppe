//! Combat action methods for GameState

use super::{
    action::action_cost,
    adaptation::total_stat_modifiers,
    combat::{default_weapon, get_weapon_def, roll_attack, CombatResult},
    event::GameEvent,
    item::get_item_def,
    map::Tile,
    state::{GameState, MsgType},
};

impl GameState {
    /// Apply mock settings to combat result if configured
    fn apply_combat_mocks(&self, mut result: CombatResult) -> CombatResult {
        if let Some(force_hit) = self.mock_combat_hit {
            result.hit = force_hit;
            if !force_hit { result.damage = 0; }
        }
        if let Some(dmg) = self.mock_combat_damage {
            if result.hit { result.damage = dmg; }
        }
        result
    }

    /// Trigger aggro for all nearby enemies of the same type (swarm behavior)
    fn trigger_swarm_aggro(&mut self, target_id: &str, center_x: i32, center_y: i32, range: i32) {
        let mut alerted_count = 0;
        for enemy in &mut self.enemies {
            if enemy.id == target_id && !enemy.provoked {
                let dist = (enemy.x - center_x).abs() + (enemy.y - center_y).abs();
                if dist <= range {
                    enemy.provoked = true;
                    alerted_count += 1;
                }
            }
        }
        if alerted_count > 0 {
            self.log_typed("The swarm is alerted!", MsgType::Combat);
        }
    }

    /// Melee attack against enemy at position
    pub fn attack_melee(&mut self, target_x: i32, target_y: i32) -> bool {
        let ei = match self.enemy_at(target_x, target_y) {
            Some(i) => i,
            None => return false,
        };

        let cost = action_cost("attack_melee");
        if self.player_ap < cost { return false; }
        self.player_ap -= cost;

        self.enemies[ei].provoked = true;
        
        // Swarm behavior
        if self.enemies[ei].def().map(|d| d.swarm).unwrap_or(false) {
            let id = self.enemies[ei].id.clone();
            let x = self.enemies[ei].x;
            let y = self.enemies[ei].y;
            self.trigger_swarm_aggro(&id, x, y, 8);
        }

        let weapon = self.equipped_weapon.as_ref()
            .and_then(|id| get_weapon_def(id))
            .unwrap_or_else(default_weapon);

        let enemy_reflex = self.enemies[ei].def().map(|d| d.reflex).unwrap_or(0);
        let enemy_armor = self.enemies[ei].def().map(|d| d.armor).unwrap_or(0);

        let result = roll_attack(&mut self.rng, weapon, enemy_reflex, enemy_armor, 0);
        let result = self.apply_combat_mocks(result);
        let name = self.enemies[ei].name().to_string();
        let dir = self.direction_from(target_x, target_y);

        if !result.hit {
            self.log_typed(format!("You miss the {} {}.", name, dir), MsgType::Combat);
            return true;
        }

        let mut dmg = result.damage;
        // Apply adaptation damage bonus
        let adapt_mods = total_stat_modifiers(&self.adaptations);
        dmg += adapt_mods.damage_bonus;
        self.enemies[ei].hp -= dmg;
        self.trigger_hit_flash(target_x, target_y);
        self.spawn_damage_number(target_x, target_y, dmg, false);

        if let Some(def) = self.enemies[ei].def() {
            for e in &def.effects {
                if e.condition == "on_hit" {
                    self.trigger_effect(&e.effect, 2);
                }
            }
            for behavior in &def.behaviors {
                if behavior.behavior_type == "reflect_damage" {
                    let percent = behavior.percent.unwrap_or(25);
                    let reflected = (dmg as u32 * percent / 100) as i32;
                    if reflected > 0 {
                        self.player_hp -= reflected;
                        self.trigger_hit_flash(self.player_x, self.player_y);
                        self.log_typed(format!("Light bends—your attack refracts back! (-{} HP)", reflected), MsgType::Combat);
                    }
                }
            }
        }

        if self.enemies[ei].hp <= 0 {
            let enemy_id = self.enemies[ei].id.clone();
            let enemy_x = self.enemies[ei].x;
            let enemy_y = self.enemies[ei].y;
            self.enemy_positions.remove(&(target_x, target_y));
            if let Some(def) = self.enemies[ei].def() {
                // Trigger on_death effects
                for e in &def.effects {
                    if e.condition == "on_death" {
                        self.trigger_effect(&e.effect, 3);
                    }
                }
                // Award XP
                if def.xp_value > 0 {
                    self.gain_xp(def.xp_value);
                }
                // Drop loot
                if !def.loot_table.is_empty() {
                    self.drop_enemy_loot(&def.loot_table, enemy_x, enemy_y);
                }
                
                // Split on death
                for behavior in &def.behaviors {
                    if behavior.behavior_type == "split_on_death" {
                        if let Some(child_id) = &behavior.condition { // Using condition field for child ID
                            let count = behavior.value.unwrap_or(2) as usize;
                            let mut spawned = 0;
                            // Try to spawn around death point
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if dx == 0 && dy == 0 { continue; }
                                    if spawned >= count { break; }
                                    let nx = enemy_x + dx;
                                    let ny = enemy_y + dy;
                                    if self.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false) 
                                        && self.enemy_at(nx, ny).is_none() 
                                        && !(nx == self.player_x && ny == self.player_y) {
                                        
                                        self.enemies.push(super::enemy::Enemy::new(nx, ny, child_id));
                                        self.enemy_positions.insert((nx, ny), self.enemies.len() - 1);
                                        spawned += 1;
                                    }
                                }
                            }
                            if spawned > 0 {
                                self.log_typed(format!("The {} splits into smaller forms!", name), MsgType::Combat);
                            }
                        }
                    }
                }
            }
            self.quest_log.on_enemy_killed(&enemy_id);
            self.emit(GameEvent::EnemyKilled {
                enemy_id: enemy_id.clone(),
                x: target_x, y: target_y
            });
            self.meta.discover_enemy(&enemy_id);
            self.log_typed(format!("You kill the {} {}!", name, dir), MsgType::Combat);
        } else {
            let crit_str = if result.crit { " CRITICAL!" } else { "" };
            self.log_typed(format!("You hit the {} {} for {} damage.{}", name, dir, dmg, crit_str), MsgType::Combat);
        }
        true
    }

    /// Ranged attack at target position
    pub fn try_ranged_attack(&mut self, target_x: i32, target_y: i32) -> bool {
        let weapon = match self.equipped_weapon.as_ref().and_then(|id| get_weapon_def(id)) {
            Some(w) if w.range > 1 => w,
            _ => {
                self.log_typed("No ranged weapon equipped.", MsgType::Combat);
                return false;
            }
        };

        let dist = (target_x - self.player_x).abs() + (target_y - self.player_y).abs();
        if dist > weapon.range {
            self.log_typed("Target out of range.", MsgType::Combat);
            return false;
        }

        let target_idx = self.map.idx(target_x, target_y);
        if !self.visible.contains(&target_idx) {
            self.log_typed("Can't see target.", MsgType::Combat);
            return false;
        }

        let cost = weapon.ap_cost;
        if self.player_ap < cost { return false; }

        if let Some(ammo_type) = &weapon.ammo_type {
            if !self.inventory.iter().any(|id| id == ammo_type) {
                self.log_typed(format!("Out of {}.", ammo_type.replace('_', " ")), MsgType::Combat);
                return false;
            }
            if let Some(idx) = self.inventory.iter().position(|id| id == ammo_type) {
                self.inventory.remove(idx);
            }
        }

        self.player_ap -= cost;
        
        // Spawn projectile trail
        let proj_char = if weapon.range > 3 { '*' } else { '-' };
        self.spawn_projectile((self.player_x, self.player_y), (target_x, target_y), proj_char);

        let ei = match self.enemy_at(target_x, target_y) {
            Some(i) => i,
            None => {
                self.log_typed("No target there.", MsgType::Combat);
                self.check_auto_end_turn();
                return true;
            }
        };

        self.enemies[ei].provoked = true;
        
        // Swarm behavior
        if self.enemies[ei].def().map(|d| d.swarm).unwrap_or(false) {
            let id = self.enemies[ei].id.clone();
            let x = self.enemies[ei].x;
            let y = self.enemies[ei].y;
            self.trigger_swarm_aggro(&id, x, y, 8);
        }

        let enemy_reflex = self.enemies[ei].def().map(|d| d.reflex).unwrap_or(0);
        let enemy_armor = self.enemies[ei].def().map(|d| d.armor).unwrap_or(0);
        let result = roll_attack(&mut self.rng, weapon, enemy_reflex, enemy_armor, 0);
        let result = self.apply_combat_mocks(result);
        let name = self.enemies[ei].name().to_string();

        if !result.hit {
            self.log_typed(format!("Your shot misses the {}.", name), MsgType::Combat);
            self.check_auto_end_turn();
            return true;
        }

        let dmg = result.damage;
        self.enemies[ei].hp -= dmg;
        self.trigger_hit_flash(target_x, target_y);
        self.spawn_damage_number(target_x, target_y, dmg, false);

        if self.enemies[ei].hp <= 0 {
            let enemy_id = self.enemies[ei].id.clone();
            let enemy_x = self.enemies[ei].x;
            let enemy_y = self.enemies[ei].y;
            self.enemy_positions.remove(&(target_x, target_y));
            if let Some(def) = self.enemies[ei].def() {
                // Award XP
                if def.xp_value > 0 {
                    self.gain_xp(def.xp_value);
                }
                // Drop loot
                if !def.loot_table.is_empty() {
                    self.drop_enemy_loot(&def.loot_table, enemy_x, enemy_y);
                }
                
                // Split on death
                for behavior in &def.behaviors {
                    if behavior.behavior_type == "split_on_death" {
                        if let Some(child_id) = &behavior.condition { // Using condition field for child ID
                            let count = behavior.value.unwrap_or(2) as usize;
                            let mut spawned = 0;
                            // Try to spawn around death point
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if dx == 0 && dy == 0 { continue; }
                                    if spawned >= count { break; }
                                    let nx = enemy_x + dx;
                                    let ny = enemy_y + dy;
                                    if self.map.get(nx, ny).map(|t| t.walkable()).unwrap_or(false) 
                                        && self.enemy_at(nx, ny).is_none() 
                                        && !(nx == self.player_x && ny == self.player_y) {
                                        
                                        self.enemies.push(super::enemy::Enemy::new(nx, ny, child_id));
                                        self.enemy_positions.insert((nx, ny), self.enemies.len() - 1);
                                        spawned += 1;
                                    }
                                }
                            }
                            if spawned > 0 {
                                self.log_typed(format!("The {} splits into smaller forms!", name), MsgType::Combat);
                            }
                        }
                    }
                }
            }
            self.quest_log.on_enemy_killed(&enemy_id);
            self.emit(GameEvent::EnemyKilled {
                enemy_id: enemy_id.clone(),
                x: target_x, y: target_y
            });
            self.meta.discover_enemy(&enemy_id);
            self.log_typed(format!("You kill the {} with a ranged shot!", name), MsgType::Combat);
        } else {
            let crit_str = if result.crit { " CRITICAL!" } else { "" };
            self.log_typed(format!("You hit the {} for {} damage.{}", name, dmg, crit_str), MsgType::Combat);
        }

        self.check_auto_end_turn();
        true
    }

    /// Break a wall at position (requires tool)
    pub fn try_break_wall(&mut self, x: i32, y: i32) -> bool {
        let has_pick = self.inventory.iter().any(|id| {
            get_item_def(id).map(|d| d.breaks_walls).unwrap_or(false)
        });
        if !has_pick {
            self.log("You need a tool to break walls.");
            return false;
        }

        let dist = (x - self.player_x).abs() + (y - self.player_y).abs();
        if dist != 1 {
            self.log("Too far to break.");
            return false;
        }

        let cost = action_cost("break_wall");
        if self.player_ap < cost { return false; }

        let idx = self.map.idx(x, y);
        if let Tile::Wall { ref id, hp } = self.map.tiles[idx].clone() {
            self.player_ap -= cost;
            let new_hp = hp - 5;
            if new_hp <= 0 {
                self.map.tiles[idx] = Tile::Floor;
                self.log("The wall crumbles!");
            } else {
                self.map.tiles[idx] = Tile::Wall { id: id.clone(), hp: new_hp };
                self.log(format!("Cracks spread through the wall. (HP: {})", new_hp));
            }
            self.check_auto_end_turn();
            return true;
        }
        self.log("Nothing to break there.");
        false
    }
}
