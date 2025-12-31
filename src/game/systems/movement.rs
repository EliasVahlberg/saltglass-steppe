//! Movement System - Handles player movement and related interactions

use crate::game::{
    action::action_cost,
    adaptation::Adaptation,
    event::GameEvent,
    item::get_item_def,
    map::Tile,
    npc::DialogueContext,
    state::{Decoy, GameState, MsgType},
};
use rand::Rng;

pub struct MovementSystem;

impl MovementSystem {
    /// Main entry point for player movement
    /// Returns true if action was taken (even if blocked), false if invalid
    pub fn try_move(state: &mut GameState, dx: i32, dy: i32) -> bool {
        state.wait_counter = 0; // Reset auto-rest counter on movement
        let new_x = state.player_x + dx;
        let new_y = state.player_y + dy;

        // Check for NPC interaction first
        if Self::handle_npc_interaction(state, new_x, new_y) {
            return true;
        }

        // Check for enemy combat
        if Self::handle_enemy_combat(state, new_x, new_y) {
            return true;
        }

        // Handle actual movement
        Self::handle_movement(state, new_x, new_y)
    }

    /// Handle NPC bump-to-talk interaction
    fn handle_npc_interaction(state: &mut GameState, new_x: i32, new_y: i32) -> bool {
        let ni = match state.npc_at(new_x, new_y) {
            Some(i) => i,
            None => return false,
        };

        let cost = action_cost("interact");
        if state.player_ap < cost { return false; }
        state.player_ap -= cost;

        // Build dialogue context
        let visible_adaptations: Vec<Adaptation> = if state.adaptations_hidden_turns > 0 {
            Vec::new()
        } else {
            state.adaptations.clone()
        };
        let inventory_snapshot = state.inventory.clone();
        let ctx = DialogueContext {
            adaptations: &visible_adaptations,
            inventory: &inventory_snapshot,
            salt_scrip: state.salt_scrip,
            faction_reputation: &state.faction_reputation,
        };

        // Get dialogue and actions
        let dialogue = state.npcs[ni].dialogue(&ctx).to_string();
        let name = state.npcs[ni].name().to_string();
        let npc_id = state.npcs[ni].id.clone();
        let actions: Vec<_> = state.npcs[ni].available_actions(&ctx).into_iter().cloned().collect();

        // Store pending dialogue for UI
        state.pending_dialogue = Some((name.clone(), dialogue.clone()));
        state.log_typed(
            format!("{}: \"{}\"", name, dialogue.replace("</nextpage>", " ")),
            MsgType::Dialogue
        );

        // Execute first available action effect
        Self::execute_npc_action_effects(state, &actions, &npc_id);

        // Mark NPC as talked to
        state.npcs[ni].talked = true;
        state.quest_log.on_npc_talked(&state.npcs[ni].id);
        state.meta.discover_npc(&state.npcs[ni].id);
        state.check_auto_end_turn();

        true
    }

    /// Execute effects from NPC dialogue actions
    fn execute_npc_action_effects(state: &mut GameState, actions: &[crate::game::npc::NpcAction], npc_id: &str) {
        for action in actions {
            // Item exchange
            if let (Some(gives), Some(consumes)) = (&action.effect.gives_item, &action.effect.consumes) {
                if let Some(idx) = state.inventory.iter().position(|id| id == consumes) {
                    state.inventory.remove(idx);
                    state.inventory.push(gives.clone());
                    let gives_name = get_item_def(gives).map(|d| d.name.as_str()).unwrap_or("item");
                    state.log_typed(
                        format!("The pilgrim presses {} into your hand.", gives_name),
                        MsgType::Loot
                    );
                    return;
                }
            }
            // Heal action
            if let Some(heal) = action.effect.heal {
                let actual = heal.min(state.player_max_hp - state.player_hp);
                state.player_hp += actual;
                state.log_typed(format!("You rest. (+{} HP)", actual), MsgType::Status);
                return;
            }
            // Trade action
            if action.effect.trade == Some(true) {
                state.pending_trade = Some(npc_id.to_string());
                state.log_typed("The merchant opens their wares.", MsgType::Social);
                return;
            }
        }
    }

    /// Handle enemy bump-to-attack
    fn handle_enemy_combat(state: &mut GameState, new_x: i32, new_y: i32) -> bool {
        if state.enemy_at(new_x, new_y).is_none() {
            return false;
        }

        let cost = action_cost("attack_melee");
        if state.player_ap < cost {
            state.end_turn();
        }
        let hit = state.attack_melee(new_x, new_y);
        if hit { state.check_auto_end_turn(); }
        hit
    }

    /// Handle actual movement to a tile
    fn handle_movement(state: &mut GameState, new_x: i32, new_y: i32) -> bool {
        let tile = match state.map.get(new_x, new_y) {
            Some(t) => t.clone(),
            None => return false,
        };

        let walkable = tile.walkable() || state.debug_phase;
        if !walkable {
            return false;
        }

        let cost = action_cost("move");
        if state.player_ap < cost { return false; }
        state.player_ap -= cost;

        // Handle pre-movement effects (Mirage Step)
        Self::handle_pre_movement(state);

        // Update position
        let _old_x = state.player_x;
        let _old_y = state.player_y;
        state.player_x = new_x;
        state.player_y = new_y;

        // Clear storm change highlighting
        let player_idx = new_y as usize * state.map.width + new_x as usize;
        state.storm_changed_tiles.remove(&player_idx);

        // Notify systems
        state.quest_log.on_position_changed(new_x, new_y);
        state.update_fov();
        state.update_lighting();

        // Pickup items at new position
        Self::pickup_items(state);

        // Handle tile-specific effects
        Self::handle_tile_effects(state, &tile, new_x, new_y);

        // Handle world transition
        Self::handle_world_transition(state, &tile, new_x, new_y);

        state.check_auto_end_turn();
        true
    }

    /// Handle pre-movement effects like Mirage Step
    fn handle_pre_movement(state: &mut GameState) {
        if state.adaptations.iter().any(|a| a.has_ability("mirage_step")) {
            state.decoys.push(Decoy {
                x: state.player_x,
                y: state.player_y,
                turns_remaining: 3,
            });
        }
    }

    /// Handle tile-specific effects (glass damage, glare)
    fn handle_tile_effects(state: &mut GameState, tile: &Tile, _x: i32, _y: i32) {
        match tile {
            Tile::Glass => {
                if state.adaptations.iter().any(|a| a.has_immunity("glass")) {
                    state.log("Your saltblood protects you from the glass.");
                } else {
                    state.player_hp -= 1;
                    state.refraction += 1;
                    state.log("Sharp glass cuts you! (-1 HP, +1 Refraction)");
                    state.check_adaptation_threshold();
                }
            }
            Tile::Glare => {
                state.player_ap = (state.player_ap - 1).max(0);
                state.log("Intense glare impairs your movement! (-1 AP)");
                
                if state.rng.gen_range(0..100) < 30 {
                    state.log("The glare blinds you temporarily!");
                }
            }
            _ => {}
        }
    }

    /// Handle world tile transitions at map edges
    fn handle_world_transition(state: &mut GameState, tile: &Tile, new_x: i32, new_y: i32) {
        if *tile != Tile::WorldExit || state.layer != 0 {
            return;
        }

        let at_north = new_y == 0;
        let at_south = new_y == state.map.height as i32 - 1;
        let at_west = new_x == 0;
        let at_east = new_x == state.map.width as i32 - 1;

        if at_north && state.world_y > 0 {
            state.travel_to_tile(state.world_x, state.world_y - 1);
            state.player_y = state.map.height as i32 - 2;
        } else if at_south && state.world_y < crate::game::world_map::WORLD_HEIGHT - 1 {
            state.travel_to_tile(state.world_x, state.world_y + 1);
            state.player_y = 1;
        } else if at_west && state.world_x > 0 {
            state.travel_to_tile(state.world_x - 1, state.world_y);
            state.player_x = state.map.width as i32 - 2;
        } else if at_east && state.world_x < crate::game::world_map::WORLD_WIDTH - 1 {
            state.travel_to_tile(state.world_x + 1, state.world_y);
            state.player_x = 1;
        }
    }

    /// Pickup items at player's current position
    pub fn pickup_items(state: &mut GameState) {
        let px = state.player_x;
        let py = state.player_y;
        
        let indices = match state.item_positions.remove(&(px, py)) {
            Some(v) => v,
            None => return,
        };

        let mut picked_up = Vec::new();
        
        // Process in reverse order to maintain valid indices
        for &i in indices.iter().rev() {
            if i >= state.items.len() { continue; }
            
            let id = state.items[i].id.clone();
            let def = get_item_def(&id);
            
            // Skip non-pickup items (e.g., light sources)
            if !def.map(|d| d.pickup).unwrap_or(true) {
                continue;
            }
            
            let name = def.map(|d| d.name.as_str()).unwrap_or("item");
            
            // Trigger on_pickup effects
            if let Some(d) = def {
                for e in &d.effects {
                    if e.condition == "on_pickup" {
                        state.trigger_effect(&e.effect, 3);
                    }
                }
            }
            
            state.inventory.push(id.clone());
            state.quest_log.on_item_collected(&id);
            state.emit(GameEvent::ItemPickedUp { item_id: id.clone() });
            state.meta.discover_item(&id);
            state.log_typed(format!("Picked up {}.", name), MsgType::Loot);
            picked_up.push(i);
        }

        // Remove picked up items (reverse order for valid indices)
        for &i in picked_up.iter().rev() {
            if i < state.items.len() {
                state.items.remove(i);
            }
        }
        
        // Rebuild spatial index
        state.rebuild_spatial_index();
    }
}
