use crate::game::{effects::context::QueryContext, mutations::Mutation, state::MsgType};
use rand_chacha::ChaCha8Rng;

/// Command handler: use psychic ability.
/// Bridges to the existing psychic system via UsePsychicAbility mutation.
pub fn handle_use_psychic(ability_id: &str, _ctx: &QueryContext, _rng: &mut ChaCha8Rng) -> Vec<Mutation> {
    vec![Mutation::UsePsychicAbility { ability_id: ability_id.to_string() }]
}

/// Command handler: flee encounter.
/// Bridges to attempt_flee via AttemptFlee mutation (needs &mut enemies).
pub fn handle_flee_encounter(ctx: &QueryContext, _rng: &mut ChaCha8Rng) -> Vec<Mutation> {
    match ctx.encounter_state {
        None => vec![Mutation::LogMessage { text: "No active encounter.".into(), msg_type: MsgType::System }],
        Some(enc) if !enc.can_flee(ctx.turn, 1.0) => vec![Mutation::LogMessage {
            text: "You cannot flee yet!".into(),
            msg_type: MsgType::Warning,
        }],
        _ => vec![Mutation::AttemptFlee { turn: ctx.turn }],
    }
}
