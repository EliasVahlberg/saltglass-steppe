//! Auto-explore system — moved from state.rs.

use std::collections::HashSet;
use bracket_pathfinding::prelude::BaseMap;
use crate::game::state::GameState;

impl GameState {
    pub fn auto_explore(&mut self) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;

        let config = get_auto_explore_config();

        // Check for nearby enemies first
        if config.stop_on_enemies && self.has_nearby_enemies(config.enemy_detection_range) {
            self.messages.push(crate::game::GameMessage::new(
                "Auto-explore stopped: enemy detected nearby".to_string(),
                crate::game::MsgType::System,
                self.turn,
            ));
            return false;
        }

        // Pick up items at current position if configured
        if config.pickup_items {
            self.pickup_filtered_items();
        }

        let start = self.world.map.idx(self.player.x, self.player.y);

        // BFS to find nearest item or unexplored walkable tile
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, vec![start]));
        visited.insert(start);

        let target = loop {
            let (idx, path) = match queue.pop_front() {
                Some(p) => p,
                None => return false, // No items or unexplored tiles reachable
            };

            let mut found_target = false;

            // Check if this tile has items we want to pick up
            if config.pickup_items {
                let x = (idx % self.world.map.width) as i32;
                let y = (idx / self.world.map.width) as i32;

                for item in &self.world.items {
                    if item.x == x && item.y == y && config.should_pickup_item(&item.id) {
                        found_target = true;
                        break;
                    }
                }
            }

            // Check if this tile has an NPC we haven't talked to yet
            if !found_target {
                let x = (idx % self.world.map.width) as i32;
                let y = (idx / self.world.map.width) as i32;

                for npc in &self.world.npcs {
                    // Only target NPCs that haven't been talked to AND don't have ongoing quest interactions
                    if npc.x == x
                        && npc.y == y
                        && !npc.talked
                        && !self.has_interacted_with_npc(&npc.id)
                    {
                        found_target = true;
                        break;
                    }
                }
            }

            // Check if this tile is unexplored
            if !found_target && !self.revealed.contains(&idx) {
                found_target = true;
            }

            if found_target {
                // Return the first step toward this tile
                if path.len() > 1 {
                    break Some(path[1]);
                }
                return false;
            }

            // Add neighbors, but avoid dangerous tiles if configured
            for (next_idx, _) in self.world.map.get_available_exits(idx) {
                if !visited.contains(&next_idx) {
                    // Check if we should avoid this tile due to dangers
                    if config.avoid_dangers && self.is_dangerous_tile(next_idx) {
                        continue;
                    }

                    // Check if there's an NPC we've already talked to on this tile
                    if self.has_talked_npc_at_idx(next_idx)
                        || self.has_interacted_npc_at_idx(next_idx)
                    {
                        continue;
                    }

                    visited.insert(next_idx);
                    let mut new_path = path.clone();
                    new_path.push(next_idx);
                    queue.push_back((next_idx, new_path));
                }
            }
        };

        if let Some(next_idx) = target {
            let nx = (next_idx % self.world.map.width) as i32;
            let ny = (next_idx / self.world.map.width) as i32;
            let dx = nx - self.player.x;
            let dy = ny - self.player.y;

            // Final check: don't move to dangerous tile
            let target_x = self.player.x + dx;
            let target_y = self.player.y + dy;
            let target_idx = self.world.map.idx(target_x, target_y);

            if config.avoid_dangers && self.is_dangerous_tile(target_idx) {
                return false;
            }

            let old_pos = (self.player.x, self.player.y);
            self.dispatch(crate::game::effects::Command::Move { dx, dy });
            self.player.x != old_pos.0 || self.player.y != old_pos.1
        } else {
            false
        }
    }

    /// Check if there are enemies nearby within the given range
    fn has_nearby_enemies(&self, range: i32) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();

        for enemy in &self.world.enemies {
            // Skip dead enemies
            if enemy.hp <= 0 {
                continue;
            }

            let distance = (enemy.x - self.player.x).abs() + (enemy.y - self.player.y).abs();
            if distance <= range {
                // If ignoring weak enemies, check enemy HP
                if config.ignore_weak_enemies && enemy.hp <= config.weak_enemy_threshold {
                    continue;
                }
                return true;
            }
        }
        false
    }

    /// Check if a tile is dangerous based on configuration
    fn is_dangerous_tile(&self, idx: usize) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();

        let tile = &self.world.map.tiles[idx];

        // Check for glass tiles
        if config.is_danger_type("glass") && matches!(tile, crate::game::map::Tile::Glass) {
            return true;
        }

        // Check for glare tiles
        if config.is_danger_type("glare") && matches!(tile, crate::game::map::Tile::Glare) {
            return true;
        }

        false
    }

    /// Check if there's an NPC we've already talked to at the given tile index
    fn has_talked_npc_at_idx(&self, idx: usize) -> bool {
        let x = (idx % self.world.map.width) as i32;
        let y = (idx / self.world.map.width) as i32;

        self.world
            .npcs
            .iter()
            .any(|npc| npc.x == x && npc.y == y && npc.talked)
    }

    /// Check if there's an NPC at this tile that we've interacted with via quest objectives
    fn has_interacted_npc_at_idx(&self, idx: usize) -> bool {
        let x = (idx % self.world.map.width) as i32;
        let y = (idx / self.world.map.width) as i32;

        self.world
            .npcs
            .iter()
            .any(|npc| npc.x == x && npc.y == y && self.has_interacted_with_npc(&npc.id))
    }

    /// Check if we've interacted with an NPC (either talked or has quest progress)
    fn has_interacted_with_npc(&self, npc_id: &str) -> bool {
        // Check if any TalkTo objective for this NPC has been completed in active quests
        for quest in &self.player.quest_log.active {
            if let Some(def) = quest.def() {
                for (i, obj) in def.objectives.iter().enumerate() {
                    if let crate::game::quest::ObjectiveType::TalkTo { npc_id: target } =
                        &obj.objective_type
                        && target == npc_id
                        && quest.objectives[i].completed
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Pick up items at current position, filtered by configuration
    fn pickup_filtered_items(&mut self) {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();

        let player_idx = self.world.map.idx(self.player.x, self.player.y);
        let mut items_to_remove = Vec::new();

        for (i, item) in self.world.items.iter().enumerate() {
            let item_idx = self.world.map.idx(item.x, item.y);
            if item_idx == player_idx && config.should_pickup_item(&item.id) {
                items_to_remove.push(i);
            }
        }

        // Remove items in reverse order to maintain indices
        for &i in items_to_remove.iter().rev() {
            let item = self.world.items.remove(i);
            self.player.inventory.push(item.id.clone());
            self.messages.push(crate::game::GameMessage::new(
                format!("Picked up {}", item.name()),
                crate::game::MsgType::System,
                self.turn,
            ));
        }
    }

}
