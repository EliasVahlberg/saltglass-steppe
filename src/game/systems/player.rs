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
    vec![
        Mutation::UsePsychicAbility { ability_id: ability_id.to_string() },
        Mutation::IncrementActivity(crate::game::player_state::ActivityField::PsychicUses),
    ]
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

/// Refraction thresholds for each adaptation tier.
const TIER_THRESHOLDS: [(u8, u32); 4] = [(1, 150), (2, 400), (3, 800), (4, 1400)];

/// Check refraction threshold and queue an adaptation choice if a new tier is reached.
/// Called after movement and storm. Sets pending_ui.adaptation_choice if triggered.
pub fn check_adaptation_threshold(state: &mut GameState) {
    use crate::game::adaptation::{all_adaptation_ids, get_adaptation_def, AdaptationCategory};

    // Find the highest tier whose threshold has been crossed but not yet triggered
    let triggered_tier = TIER_THRESHOLDS.iter().find(|(tier, threshold)| {
        state.player.refraction >= *threshold
            && !state.player.adaptation_tiers_triggered.contains(tier)
    });

    let Some(&(tier, _)) = triggered_tier else { return };

    // Mark this tier as triggered immediately to prevent re-triggering
    state.player.adaptation_tiers_triggered.push(tier);

    // Compute category scores from activity counters
    let act = &state.player.activity;
    let scores: [(AdaptationCategory, f32); 4] = [
        (AdaptationCategory::Survival,
            act.storms_survived as f32 * 3.0 + act.glass_tiles_walked as f32 * 0.1 + act.damage_taken_total as f32 * 0.05),
        (AdaptationCategory::Predator,
            act.enemies_killed_melee as f32 * 2.0 + act.elite_enemies_killed as f32 * 5.0),
        (AdaptationCategory::Precision,
            act.enemies_killed_ranged as f32 * 2.0 + act.psychic_uses as f32 * 3.0),
        (AdaptationCategory::Artificer,
            act.items_crafted as f32 * 4.0 + act.items_used as f32 * 1.0),
    ];

    // Find dominant and secondary categories
    let mut sorted_scores = scores.iter().collect::<Vec<_>>();
    sorted_scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    let dominant  = &sorted_scores[0].0;
    let secondary = &sorted_scores[1].0;

    // Build weighted pool: (id, weight)
    let owned_ids: Vec<&str> = state.player.adaptations.iter().map(|a| a.id()).collect();
    let pool: Vec<(&str, f32)> = all_adaptation_ids()
        .into_iter()
        .filter_map(|id| {
            let def = get_adaptation_def(id)?;
            // Must match this tier
            if def.tier != tier { return None; }
            // Must not already be owned
            if owned_ids.contains(&id) { return None; }
            // Must meet unlock condition
            if !def.unlock_condition.is_met(&state.player.activity) { return None; }
            let weight = match &def.category {
                Some(cat) if cat == dominant  => 3.0,
                Some(cat) if cat == secondary => 1.5,
                _ => 1.0,
            };
            Some((id, weight))
        })
        .collect();

    if pool.is_empty() {
        // No valid adaptations for this tier — skip silently
        return;
    }

    // Draw up to 3 options, max 1 per category
    let mut chosen: Vec<String> = Vec::new();
    let mut used_categories: Vec<Option<AdaptationCategory>> = Vec::new();
    let mut rng = state.rng.clone();

    for _ in 0..3 {
        // Filter pool to exclude already-chosen and same-category duplicates
        let available: Vec<(&str, f32)> = pool.iter()
            .filter(|(id, _)| !chosen.contains(&id.to_string()))
            .filter(|(id, _)| {
                let cat = get_adaptation_def(id).and_then(|d| d.category.clone());
                !used_categories.contains(&cat)
            })
            .cloned()
            .collect();

        if available.is_empty() { break; }

        // Weighted random pick
        let total: f32 = available.iter().map(|(_, w)| w).sum();
        let mut roll = rng.gen_range(0.0..total);
        let picked = available.iter().find(|(_, w)| { roll -= w; roll <= 0.0 })
            .or_else(|| available.last())
            .map(|(id, _)| *id);

        if let Some(id) = picked {
            let cat = get_adaptation_def(id).and_then(|d| d.category.clone());
            used_categories.push(cat);
            chosen.push(id.to_string());
        }
    }

    state.rng = rng;

    if chosen.is_empty() { return; }

    state.log_typed(
        format!("⬡ Refraction threshold reached — your body is ready to change. (Tier {})", tier),
        MsgType::System,
    );
    state.pending_ui.adaptation_choice = Some(chosen);
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
