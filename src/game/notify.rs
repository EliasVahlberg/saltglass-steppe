//! Notification layer — maps StateTransitions to reactive Mutations.
//! This is the single place for all cross-system coordination.
//! Adding a new reaction: add a line to the relevant match arm.

use rand_chacha::ChaCha8Rng;

use crate::game::{
    mutations::{Mutation, StateTransition},
    state::GameState,
    systems::combat,
};

/// Map a batch of transitions to reactive mutations.
/// Called after each apply_mutations round in the cascade loop.
pub fn on_transitions(
    transitions: &[StateTransition],
    state: &GameState,
    rng: &mut ChaCha8Rng,
) -> Vec<Mutation> {
    let mut out = Vec::new();
    for t in transitions {
        match t {
            StateTransition::EnemyHpChanged { idx, old_hp, new_hp } => {
                out.extend(combat::on_enemy_hit(state, *idx, *old_hp, *new_hp));
            }
            StateTransition::EnemyHpReachedZero { idx, enemy_id, x, y } => {
                out.extend(combat::on_enemy_killed(state, *idx, enemy_id, *x, *y, rng));
            }
            // Future: PlayerPositionChanged, TurnAdvanced, ItemAddedToInventory, etc.
            _ => {}
        }
    }
    out
}
