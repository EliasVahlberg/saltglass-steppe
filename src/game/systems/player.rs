use crate::game::{effects::context::QueryContext, mutations::Mutation, state::MsgType};
use rand_chacha::ChaCha8Rng;

pub fn handle_wait(ctx: &QueryContext) -> Vec<Mutation> {
    use crate::game::rules::actions::rule_wait;
    crate::game::systems::rule_output_to_mutations(rule_wait(ctx), msg_type)
}

pub fn handle_rest(ctx: &QueryContext) -> Vec<Mutation> {
    use crate::game::rules::actions::rule_rest;
    crate::game::systems::rule_output_to_mutations(rule_rest(ctx), msg_type)
}

pub fn handle_equip(inv_idx: usize, slot: &str, ctx: &QueryContext) -> Vec<Mutation> {
    use crate::game::rules::actions::rule_equip;
    crate::game::systems::rule_output_to_mutations(rule_equip(inv_idx, slot, ctx), msg_type)
}

pub fn handle_unequip(slot: &str) -> Vec<Mutation> {
    use crate::game::rules::actions::rule_unequip;
    crate::game::systems::rule_output_to_mutations(rule_unequip(slot), msg_type)
}

pub fn handle_allocate_stat(stat: &str, ctx: &QueryContext) -> Vec<Mutation> {
    use crate::game::rules::actions::rule_allocate_stat;
    crate::game::systems::rule_output_to_mutations(rule_allocate_stat(stat, ctx), msg_type)
}

fn msg_type(s: &str) -> MsgType {
    match s {
        "combat" => MsgType::Combat,
        "loot" => MsgType::Loot,
        "warning" => MsgType::Warning,
        "status" => MsgType::Status,
        _ => MsgType::System,
    }
}

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
