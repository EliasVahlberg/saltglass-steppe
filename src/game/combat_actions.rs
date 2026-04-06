//! Combat action methods for GameState

use super::{
    action::action_cost, adaptation::total_stat_modifiers,
    item::get_item_def, map::Tile, state::GameState, systems::ai::AiSystem,
};

impl GameState {
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

    /// Get effective player armor (base + equipment + adaptations)
    pub fn effective_armor(&self) -> i32 {
        let adapt_mods = total_stat_modifiers(&self.player.adaptations);
        self.player.armor + adapt_mods.armor
    }

    /// Get effective player reflex (base + adaptations)
    pub fn effective_reflex(&self) -> i32 {
        let adapt_mods = total_stat_modifiers(&self.player.adaptations);
        self.player.reflex + adapt_mods.reflex
    }

    pub fn update_enemies(&mut self) {
        AiSystem::update_enemies(self);
    }
}
