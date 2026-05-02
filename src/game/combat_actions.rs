//! Combat action methods and stat resolution for GameState.

use super::{
    action::action_cost,
    item::get_item_def,
    map::Tile,
    state::GameState,
    stat_effect::{resolve_stat_i32, StatEffectSource},
    systems::ai::AiSystem,
};

impl GameState {
    /// Collect all active stat effects from adaptations, equipment, and status effects.
    pub fn active_stat_effects(&self) -> Vec<super::stat_effect::StatEffect> {
        self.player.collect_stat_effects()
    }

    /// Effective armor: sum of all armor effects.
    pub fn effective_armor(&self) -> i32 {
        let effects = self.active_stat_effects();
        resolve_stat_i32(0, "armor", &effects)
    }

    /// Effective reflex: base + all reflex effects.
    pub fn effective_reflex(&self) -> i32 {
        let effects = self.active_stat_effects();
        resolve_stat_i32(self.player.reflex, "reflex", &effects)
    }

    /// Flat damage bonus from all sources (adaptations + status debuffs).
    pub fn effective_damage_bonus_flat(&self) -> i32 {
        let effects = self.active_stat_effects();
        resolve_stat_i32(0, "damage_bonus", &effects)
    }

    /// Accuracy penalty from status effects (positive = worse accuracy).
    pub fn effective_accuracy_penalty(&self) -> i32 {
        let effects = self.active_stat_effects();
        resolve_stat_i32(0, "accuracy_penalty", &effects)
    }

    /// Break a wall at position (requires tool)
    pub fn try_break_wall(&mut self, x: i32, y: i32) -> bool {
        let has_pick = self
            .player
            .inventory
            .iter()
            .any(|id| get_item_def(id).map(|d| d.breaks_walls).unwrap_or(false));
        if !has_pick {
            self.log("You need a tool to break walls.");
            return false;
        }

        let dist = (x - self.player.x).abs() + (y - self.player.y).abs();
        if dist != 1 {
            self.log("Too far to break.");
            return false;
        }

        let cost = action_cost("break_wall");
        if self.player.ap < cost {
            return false;
        }

        let idx = self.world.map.idx(x, y);
        if let Tile::Wall { ref id, hp } = self.world.map.tiles[idx].clone() {
            self.player.ap -= cost;
            let new_hp = hp - 5;
            if new_hp <= 0 {
                self.world.map.tiles[idx] = Tile::default_floor();
                self.log("The wall crumbles!");
            } else {
                self.world.map.tiles[idx] = Tile::Wall {
                    id: id.clone(),
                    hp: new_hp,
                };
                self.log(format!("Cracks spread through the wall. (HP: {})", new_hp));
            }
            self.check_auto_end_turn();
            return true;
        }
        self.log("Nothing to break there.");
        false
    }

    pub fn update_enemies(&mut self) {
        AiSystem::update_enemies(self);
    }
}
