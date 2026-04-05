//! Dispatch layer — routes Commands to system handlers, runs the cascade loop.

use crate::game::{
    effects::{Command, context::QueryContext},
    mutations::Mutation,
    notify,
    state::GameState,
    systems::{combat, interact, items, movement, player, quest},
};

/// Apply mutations, collect transitions, run notifications, cascade (depth-limited).
pub fn apply_with_cascade(state: &mut GameState, mutations: Vec<Mutation>) {
    apply_recursive(state, mutations, 0);
    // Derives run once after all cascades settle
    state.update_fov();
    state.update_lighting();
}

fn apply_recursive(state: &mut GameState, mutations: Vec<Mutation>, depth: u32) {
    if depth >= 10 || mutations.is_empty() { return; }

    let transitions = state.apply_mutations(mutations);
    if transitions.is_empty() { return; }

    let mut rng = state.rng.clone();
    let reactions = notify::on_transitions(&transitions, state, &mut rng);
    state.rng = rng;
    apply_recursive(state, reactions, depth + 1);
}

/// Route a command to the appropriate system handler.
/// Returns Some(mutations) if handled here, None to fall through to old path.
pub fn route_command(
    command: &Command,
    state: &mut GameState,
) -> Option<Vec<Mutation>> {

    match command {
        Command::Attack { target_x, target_y } => {
            let ctx = QueryContext::from_state(state);
            let mut rng = state.rng.clone();
            let m = combat::handle_melee(*target_x, *target_y, &ctx, &mut rng);
            state.rng = rng;
            Some(m)
        }
        Command::RangedAttack { target_x, target_y } => {
            let ctx = QueryContext::from_state(state);
            let mut rng = state.rng.clone();
            let m = combat::handle_ranged(*target_x, *target_y, &ctx, &mut rng);
            state.rng = rng;
            Some(m)
        }
        Command::UsePsychic { ability_id } =>
            Some(vec![crate::game::mutations::Mutation::UsePsychicAbility { ability_id: ability_id.clone() }]),
        Command::FleeEncounter => {
            let mutations = {
                let ctx = QueryContext::from_state(state);
                let mut rng = state.rng.clone();
                let m = player::handle_flee_encounter(&ctx, &mut rng);
                state.rng = rng;
                m
            };
            Some(mutations)
        }
        Command::AcceptQuest { quest_id } =>
            Some(quest::handle_accept_quest(quest_id, state)),
        Command::CompleteQuest { quest_id } =>
            Some(quest::handle_complete_quest(quest_id, state)),
        Command::Interact { x, y } =>
            Some(interact::handle_interact(*x, *y, state)),
        Command::Examine { x, y } =>
            Some(interact::handle_examine(*x, *y, state)),
        Command::WorldMove { new_wx, new_wy } =>
            Some(vec![crate::game::mutations::Mutation::WorldMove { wx: *new_wx, wy: *new_wy }]),
        Command::WorldMoveSafe { new_wx, new_wy } =>
            Some(vec![crate::game::mutations::Mutation::WorldMoveSafe { wx: *new_wx, wy: *new_wy }]),
        Command::FollowWorldPath =>
            Some(vec![crate::game::mutations::Mutation::FollowWorldPath]),
        Command::CalculateWorldPath { target_wx, target_wy } =>
            Some(vec![crate::game::mutations::Mutation::CalculateWorldPath { target: (*target_wx, *target_wy) }]),
        Command::EnterSubterranean =>
            Some(vec![crate::game::mutations::Mutation::EnterSubterranean]),
        Command::ExitSubterranean =>
            Some(vec![crate::game::mutations::Mutation::ExitSubterranean]),
        Command::Move { dx, dy } =>
            Some(movement::handle_move(*dx, *dy)),
        Command::Wait => {
            let ctx = QueryContext::from_state(state);
            let mut m = player::handle_wait(&ctx);
            m.push(crate::game::mutations::Mutation::EndTurn);
            Some(m)
        }
        Command::Rest => {
            let ctx = QueryContext::from_state(state);
            let mut m = player::handle_rest(&ctx);
            // If rest produced effects (not blocked by enemies), also run AI/housekeeping
            let has_effects = m.iter().any(|m| !matches!(m, crate::game::mutations::Mutation::LogMessage { .. }));
            if has_effects {
                m.push(crate::game::mutations::Mutation::RestTick);
            }
            Some(m)
        }
        Command::Equip { inv_idx, slot } => {
            let ctx = QueryContext::from_state(state);
            Some(player::handle_equip(*inv_idx, slot, &ctx))
        }
        Command::Unequip { slot } =>
            Some(player::handle_unequip(slot)),
        Command::AllocateStat { stat } => {
            let ctx = QueryContext::from_state(state);
            Some(player::handle_allocate_stat(stat, &ctx))
        }
        Command::UseItem { index } => {
            let ctx = QueryContext::from_state(state);
            Some(items::handle_use_item(*index, &ctx))
        }
        Command::UseItemOnTile { index, x, y } => {
            let ctx = QueryContext::from_state(state);
            Some(items::handle_use_item_on_tile(*index, *x, *y, &ctx))
        }
    }
}
