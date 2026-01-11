use super::System;
use crate::game::enemy::{LootEntry, get_enemy_def};
use crate::game::event::GameEvent;
use crate::game::item::{Item, get_item_def};
use crate::game::state::{GameState, MsgType};
use rand::Rng;

/// Handles loot dropping from enemy deaths via event bus
pub struct LootSystem;

impl System for LootSystem {
    fn update(&self, _state: &mut GameState) {
        // LootSystem is reactive-only (event-driven)
    }

    fn on_event(&self, state: &mut GameState, event: &GameEvent) {
        if let GameEvent::EnemyKilled { enemy_id, x, y } = event {
            Self::handle_enemy_death(state, enemy_id, *x, *y);
        }
    }
}

impl LootSystem {
    /// Handle loot drop when an enemy is killed
    fn handle_enemy_death(state: &mut GameState, enemy_id: &str, x: i32, y: i32) {
        let Some(def) = get_enemy_def(enemy_id) else {
            return;
        };

        if def.loot_table.is_empty() {
            return;
        }

        Self::drop_loot(state, &def.loot_table, x, y);
    }

    /// Roll on a loot table and drop an item at the specified position
    pub fn drop_loot(state: &mut GameState, loot_table: &[LootEntry], x: i32, y: i32) {
        if loot_table.is_empty() {
            return;
        }

        let total_weight: u32 = loot_table.iter().map(|entry| entry.weight).sum();
        if total_weight == 0 {
            return;
        }

        let roll = state.rng.gen_range(0..total_weight);
        let mut cumulative = 0;
        for entry in loot_table {
            cumulative += entry.weight;
            if roll < cumulative {
                let item = Item::new(x, y, &entry.item);
                state.items.push(item);
                state.rebuild_spatial_index();
                if let Some(def) = get_item_def(&entry.item) {
                    state.log_typed(format!("The enemy drops {}.", def.name), MsgType::Loot);
                }
                return;
            }
        }
    }
}
