//! Combat action methods for GameState

use super::{
    action::action_cost, adaptation::total_stat_modifiers, combat::CombatResult,
    item::get_item_def, map::Tile, state::GameState, systems::ai::AiSystem,
    systems::combat::CombatSystem,
};

impl GameState {
    /// Melee attack against enemy at position
    pub fn attack_melee(&mut self, target_x: i32, target_y: i32) -> bool {
        CombatSystem::attack_melee(self, target_x, target_y)
    }

    /// Ranged attack at target position
    pub fn try_ranged_attack(&mut self, target_x: i32, target_y: i32) -> bool {
        CombatSystem::ranged_attack(self, target_x, target_y)
    }

    /// Break a wall at position (requires tool)
    pub fn try_break_wall(&mut self, x: i32, y: i32) -> bool {
        let has_pick = self
            .inventory
            .iter()
            .any(|id| get_item_def(id).map(|d| d.breaks_walls).unwrap_or(false));
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
        if self.player_ap < cost {
            return false;
        }

        let idx = self.map.idx(x, y);
        if let Tile::Wall { ref id, hp } = self.map.tiles[idx].clone() {
            self.player_ap -= cost;
            let new_hp = hp - 5;
            if new_hp <= 0 {
                self.map.tiles[idx] = Tile::default_floor();
                self.log("The wall crumbles!");
            } else {
                self.map.tiles[idx] = Tile::Wall {
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

    // Helper for tests/mocks - delegated to CombatSystem if needed, but for now we keep the fields in GameState
    // and CombatSystem reads them.
    pub fn apply_combat_mocks(&self, mut result: CombatResult) -> CombatResult {
        if let Some(force_hit) = self.mock_combat_hit {
            result.hit = force_hit;
            if !force_hit {
                result.damage = 0;
            }
        }
        if let Some(dmg) = self.mock_combat_damage {
            if result.hit {
                result.damage = dmg;
            }
        }
        result
    }

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
        AiSystem::update_enemies(self);
    }
}
