use crate::game::{effects::context::QueryContext, mutations::Mutation, state::{GameState, MsgType}};
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
/// Execute flee attempt. Returns atomic mutations — no bridge mutation, no &mut self.rng.
pub fn handle_flee_encounter(ctx: &QueryContext, rng: &mut ChaCha8Rng) -> Vec<Mutation> {
    let Some(enc) = ctx.encounter_state else {
        return vec![Mutation::LogMessage { text: "No active encounter.".into(), msg_type: MsgType::System }];
    };
    if !enc.can_flee(ctx.turn, 1.0) {
        return vec![Mutation::LogMessage { text: "You cannot flee yet!".into(), msg_type: MsgType::Warning }];
    }
    let wayfaring = ctx.player.skills.get_skill_level("wayfaring");
    match crate::game::encounter::attempt_flee(
        ctx.player.x, ctx.player.y,
        ctx.enemies, &enc.spawned_enemies,
        rng, wayfaring,
    ) {
        Ok(()) => vec![
            Mutation::SetEncounterState(None),
            Mutation::LogMessage { text: "You successfully flee the encounter!".into(), msg_type: MsgType::Status },
        ],
        Err(e) => vec![
            Mutation::SetLastFleeAttempt(ctx.turn),
            Mutation::LogMessage { text: e, msg_type: MsgType::Warning },
        ],
    }
}

use rand::Rng;

/// Check refraction threshold and grant adaptations. Called after movement and storm.
pub fn check_adaptation_threshold(state: &mut GameState) {
    let mut available: Vec<(&str, u32)> = crate::game::adaptation::all_adaptation_ids()
        .iter()
        .filter_map(|&id| {
            crate::game::adaptation::get_adaptation_def(id).map(|def| (id, def.threshold))
        })
        .filter(|(id, _)| !state.player.adaptations.iter().any(|a| a.id() == *id))
        .collect();
    available.sort_by_key(|(_, t)| *t);
    if let Some(&(adaptation_id, _)) = available.iter().find(|(_, t)| state.player.refraction >= *t)
        && let Some(adaptation) = crate::game::adaptation::Adaptation::from_id(adaptation_id)
    {
        state.player.adaptations.push(adaptation);
        state.log(format!("🧬 You gain {}!", adaptation.name()));
    }
}

/// Apply light-based effects (glare, item interactions). Called from tick_turn_housekeeping.
pub fn apply_light_effects(state: &mut GameState) {
    if state.debug.disable_glare { return; }
    let light_level = crate::game::lighting::get_light_level(
        &state.light_map, state.player.x, state.player.y,
    );
    for item_id in &state.player.inventory.clone() {
        if let Some(def) = crate::game::item::get_item_def(item_id) {
            if def.reveals_storm_timing && light_level > 150
                && state.rng.gen_range(0..100) < 10 {
                    state.log_typed(
                        "The Storm Chart glows, revealing storm patterns...",
                        MsgType::System,
                    );
                }
            if def.grants_invisibility && light_level < 50 && !state.has_status_effect("invisible") {
                apply_status_effect(state, "invisible", 3);
                state.log_typed("You blend into the shadows...", MsgType::System);
            }
        }
    }
}

/// Apply or merge a status effect on the player.
pub fn apply_status_effect(state: &mut GameState, effect_id: &str, duration: i32) {
    if let Some(existing) = state.player.status_effects.iter_mut().find(|e| e.id == effect_id) {
        existing.duration = existing.duration.max(duration);
        existing.add_stack(5);
    } else {
        state.player.status_effects.push(
            crate::game::status::StatusEffect::new(effect_id, duration),
        );
    }
    state.log_typed(format!("You are affected by {}.", effect_id), MsgType::Combat);
}

/// Recalculate weapon/armor stats from equipment slots.
pub fn recalc_equipment_stats(state: &mut GameState) {
    state.player.equipped_weapon = state.player.equipment.weapon.clone();
    state.player.armor = state.player.equipment.jacket.as_ref()
        .and_then(|id| crate::game::item::get_item_def(id))
        .map(|def| def.armor_value)
        .unwrap_or(0);
}
