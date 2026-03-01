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
        let new_x = state.player.x + dx;
        let new_y = state.player.y + dy;

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
        if state.player.ap < cost {
            return false;
        }
        state.player.ap -= cost;

        // Build dialogue context
        let visible_adaptations: Vec<Adaptation> = if state.player.adaptations_hidden_turns > 0 {
            Vec::new()
        } else {
            state.player.adaptations.clone()
        };
        let inventory_snapshot = state.player.inventory.clone();
        let ctx = DialogueContext {
            adaptations: &visible_adaptations,
            inventory: &inventory_snapshot,
            salt_scrip: state.player.salt_scrip,
            faction_reputation: &state.player.faction_reputation,
        };

        // Get dialogue and actions
        let dialogue = state.world.npcs[ni].dialogue(&ctx).to_string();
        let name = state.world.npcs[ni].name().to_string();
        let npc_id = state.world.npcs[ni].id.clone();
        let actions: Vec<_> = state.world.npcs[ni]
            .available_actions(&ctx)
            .into_iter()
            .cloned()
            .collect();

        // Check if this NPC uses terminal interface
        if let Some(tree) = crate::game::dialogue::get_dialogue_tree(&npc_id) {
            if tree.uses_terminal_interface() {
                if let Some(personality) = &tree.aria_personality {
                    if let Some((aria_text, aria_options)) = crate::game::dialogue::start_aria_dialogue(&npc_id, personality, state) {
                        // Set up ARIA interface instead of regular dialogue
                        state.pending_aria_dialogue = Some((aria_text, aria_options));
                        state.log_typed(
                            format!("ARIA Terminal activated: {}", name),
                            MsgType::System,
                        );
                        return true;
                    }
                }
            }
        }

        // Store pending dialogue for UI
        state.pending_dialogue = Some((name.clone(), dialogue.clone()));
        state.log_typed(
            format!("{}: \"{}\"", name, dialogue.replace("</nextpage>", " ")),
            MsgType::Dialogue,
        );

        // Execute first available action effect
        Self::execute_npc_action_effects(state, &actions, &npc_id);

        // Mark NPC as talked to (but allow re-talking for quest progression)
        let should_mark_talked = !Self::has_pending_quest_objectives(state, &npc_id);
        if should_mark_talked {
            state.world.npcs[ni].talked = true;
        }

        // Emit event — QuestSystem handles quest progression and completion
        state.emit(GameEvent::NpcTalkedTo {
            npc_id: npc_id.clone(),
        });

        state.meta.discover_npc(&state.world.npcs[ni].id);
        state.check_auto_end_turn();

        true
    }

    /// Check if there are pending quest objectives for this NPC
    fn has_pending_quest_objectives(state: &GameState, npc_id: &str) -> bool {
        for quest in &state.player.quest_log.active {
            if let Some(def) = quest.def() {
                for (i, obj) in def.objectives.iter().enumerate() {
                    if let crate::game::quest::ObjectiveType::TalkTo { npc_id: target } =
                        &obj.objective_type
                    {
                        if target == npc_id && !quest.objectives[i].completed {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Execute effects from NPC dialogue actions
    fn execute_npc_action_effects(
        state: &mut GameState,
        actions: &[crate::game::npc::NpcAction],
        npc_id: &str,
    ) {
        for action in actions {
            // Item exchange
            if let (Some(gives), Some(consumes)) =
                (&action.effect.gives_item, &action.effect.consumes)
            {
                if let Some(idx) = state.player.inventory.iter().position(|id| id == consumes) {
                    state.player.inventory.remove(idx);
                    state.player.inventory.push(gives.clone());
                    let gives_name = get_item_def(gives)
                        .map(|d| d.name.as_str())
                        .unwrap_or("item");
                    state.log_typed(
                        format!("The pilgrim presses {} into your hand.", gives_name),
                        MsgType::Loot,
                    );
                    return;
                }
            }
            // Heal action
            if let Some(heal) = action.effect.heal {
                let actual = heal.min(state.player_max_hp() - state.player_hp());
                state.player.hp += actual;
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
        if state.player_ap() < cost {
            state.end_turn();
        }
        let hit = state.attack_melee(new_x, new_y);
        if hit {
            state.check_auto_end_turn();
        }
        hit
    }

    /// Handle actual movement to a tile
    fn handle_movement(state: &mut GameState, new_x: i32, new_y: i32) -> bool {
        let tile = match state.world.map.get(new_x, new_y) {
            Some(t) => t.clone(),
            None => return false,
        };

        let walkable = tile.walkable() || state.debug_phase;
        if !walkable {
            return false;
        }

        let cost = action_cost("move");
        if state.player.ap < cost {
            return false;
        }
        state.player.ap -= cost;

        // Handle pre-movement effects (Mirage Step)
        Self::handle_pre_movement(state);

        // Update position
        let old_x = state.player.x;
        let old_y = state.player.y;
        state.player.x = new_x;
        state.player.y = new_y;

        // Clear storm change highlighting
        let player_idx = new_y as usize * state.world.map.width + new_x as usize;
        state.world.visual_effects.storm_changed_tiles.remove(&player_idx);

        // Emit movement event (QuestSystem handles position-based objectives)
        state.emit(GameEvent::PlayerMoved {
            from_x: old_x,
            from_y: old_y,
            to_x: new_x,
            to_y: new_y,
        });
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
        if state
            .player.adaptations
            .iter()
            .any(|a| a.has_ability("mirage_step"))
        {
            state.decoys.push(Decoy {
                x: state.player.x,
                y: state.player.y,
                turns_remaining: 3,
            });
        }
    }

    /// Handle tile-specific effects (glass damage, glare)
    fn handle_tile_effects(state: &mut GameState, tile: &Tile, _x: i32, _y: i32) {
        match tile {
            Tile::Glass => {
                if state.player.adaptations.iter().any(|a| a.has_immunity("glass")) {
                    state.log("Your saltblood protects you from the glass.");
                } else {
                    state.player.hp -= 1;
                    state.player.refraction += 1;
                    state.log("Sharp glass cuts you! (-1 HP, +1 Refraction)");
                    state.check_adaptation_threshold();
                }
            }
            Tile::Glare => {
                state.player.ap = (state.player.ap - 1).max(0);
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
        if state.test_mode {
            return;
        }
        if *tile != Tile::WorldExit || state.layer() != 0 {
            return;
        }

        let at_north = new_y == 0;
        let at_south = new_y == state.world.map.height as i32 - 1;
        let at_west = new_x == 0;
        let at_east = new_x == state.world.map.width as i32 - 1;

        if at_north && state.world.world_y > 0 {
            state.travel_to_tile(state.world.world_x, state.world.world_y - 1);
            state.player.y = state.world.map.height as i32 - 2;
        } else if at_south && state.world.world_y < crate::game::world_map::WORLD_HEIGHT - 1 {
            state.travel_to_tile(state.world.world_x, state.world.world_y + 1);
            state.player.y = 1;
        } else if at_west && state.world.world_x > 0 {
            state.travel_to_tile(state.world.world_x - 1, state.world.world_y);
            state.player.x = state.world.map.width as i32 - 2;
        } else if at_east && state.world.world_x < crate::game::world_map::WORLD_WIDTH - 1 {
            state.travel_to_tile(state.world.world_x + 1, state.world.world_y);
            state.player.x = 1;
        }
    }

    /// Pickup items at player's current position
    pub fn pickup_items(state: &mut GameState) {
        let px = state.player.x;
        let py = state.player.y;

        let indices = match state.item_positions.remove(&(px, py)) {
            Some(v) => v,
            None => return,
        };

        let mut picked_up = Vec::new();

        // Process in reverse order to maintain valid indices
        for &i in indices.iter().rev() {
            if i >= state.world.items.len() {
                continue;
            }

            let id = state.world.items[i].id.clone();
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

            state.player.inventory.push(id.clone());
            state.emit(GameEvent::ItemPickedUp {
                item_id: id.clone(),
            });
            state.meta.discover_item(&id);
            state.log_typed(format!("Picked up {}.", name), MsgType::Loot);
            picked_up.push(i);
        }

        // Remove picked up items (reverse order for valid indices)
        for &i in picked_up.iter().rev() {
            if i < state.world.items.len() {
                state.world.items.remove(i);
            }
        }

        // Rebuild spatial index
        state.rebuild_spatial_index();
    }
}
