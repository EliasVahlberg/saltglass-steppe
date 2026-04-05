//! Dispatch layer — routes Commands to system handlers, runs the cascade loop.
//! Only combat commands are routed here in Stage 2.
//! Other commands still go through GameState::dispatch() (old path).

use rand_chacha::ChaCha8Rng;

use crate::game::{
    effects::{Command, context::QueryContext},
    mutations::Mutation,
    notify,
    state::GameState,
    systems::combat,
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
    state.rng = rng; // write back so loot rolls advance the canonical rng
    apply_recursive(state, reactions, depth + 1);
}

/// Route a combat command to the appropriate system handler.
/// Returns Some(mutations) if this command is handled here, None if it should
/// fall through to the old GameState dispatch path.
pub fn route_command(
    command: &Command,
    ctx: &QueryContext,
    rng: &mut ChaCha8Rng,
) -> Option<Vec<Mutation>> {
    match command {
        Command::Attack { target_x, target_y } => {
            Some(combat::handle_melee(*target_x, *target_y, ctx, rng))
        }
        Command::RangedAttack { target_x, target_y } => {
            Some(combat::handle_ranged(*target_x, *target_y, ctx, rng))
        }
        _ => None,
    }
}
