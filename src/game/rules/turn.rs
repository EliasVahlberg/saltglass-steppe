use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::game::effects::{Effect, MapEffect, PlayerEffect, RuleOutput};
use crate::game::effects::context::QueryContext;

/// Pure rule: advance time of day every 10 turns, weather change at dawn/dusk.
pub fn rule_tick_time(ctx: &QueryContext, rng: &mut ChaCha8Rng) -> RuleOutput {
    let mut effects = Vec::new();

    if ctx.turn.is_multiple_of(10) {
        let new_time = (ctx.time_of_day as u32 + 1) % 24;
        effects.push(Effect::Map(MapEffect::AdvanceTime { new_time }));

        if new_time == 6 || new_time == 18 {
            let roll = rng.gen_range(0..10);
            let weather = match roll {
                0..=6 => "clear",
                7..=8 => "dusty",
                9 => "sandstorm",
                _ => "clear",
            };
            effects.push(Effect::Map(MapEffect::SetWeather { weather: weather.to_string() }));
        }
    }

    RuleOutput { effects, presentation: Vec::new() }
}

/// Pure rule: check if encounter is complete, grant XP.
pub fn rule_check_encounters(ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();

    if let Some(encounter) = ctx.encounter_state
        && encounter.is_complete(ctx.enemies)
    {
        if let crate::game::encounter::EncounterType::Hostile { threat_points } =
            encounter.encounter_type
        {
            let xp = threat_points * 2;
            effects.push(Effect::Player(PlayerEffect::GainXp { amount: xp }));
        }
        effects.push(Effect::Player(PlayerEffect::ClearEncounter));
    }

    effects.push(Effect::Map(MapEffect::TickEncounterTimer));

    RuleOutput { effects, presentation: Vec::new() }
}

/// Pure rule: check refraction threshold for adaptation gain.
pub fn rule_check_adaptation(ctx: &QueryContext) -> RuleOutput {
    let mut effects = Vec::new();

    let mut available: Vec<(&str, u32)> = crate::game::adaptation::all_adaptation_ids()
        .iter()
        .filter_map(|&id| {
            crate::game::adaptation::get_adaptation_def(id).map(|def| (id, def.threshold))
        })
        .filter(|(id, _)| !ctx.player_adaptations.iter().any(|a| a.id() == *id))
        .collect();

    available.sort_by_key(|(_, threshold)| *threshold);

    if let Some(&(adaptation_id, _)) = available.iter().find(|(_, t)| ctx.player_refraction >= *t) {
        effects.push(Effect::Player(PlayerEffect::GainAdaptation {
            adaptation_id: adaptation_id.to_string(),
        }));
    }

    RuleOutput { effects, presentation: Vec::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use crate::game::effects::context::TestContext;

    #[test]
    fn tick_time_advances_at_10_turn_boundary() {
        let tc = TestContext::new().with_turn(10).with_time_of_day(5);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let output = rule_tick_time(&ctx, &mut rng);
        assert!(output.effects.contains(&Effect::Map(MapEffect::AdvanceTime { new_time: 6 })));
    }

    #[test]
    fn tick_time_weather_at_dawn() {
        let tc = TestContext::new().with_turn(10).with_time_of_day(5); // 5 -> 6 = dawn
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let output = rule_tick_time(&ctx, &mut rng);
        assert!(output.effects.iter().any(|e| matches!(e, Effect::Map(MapEffect::SetWeather { .. }))));
    }

    #[test]
    fn tick_time_no_advance_off_boundary() {
        let tc = TestContext::new().with_turn(7);
        let ctx = tc.build();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let output = rule_tick_time(&ctx, &mut rng);
        assert!(output.effects.is_empty());
    }

    #[test]
    fn encounter_timer_always_ticks() {
        let tc = TestContext::new();
        let ctx = tc.build();
        let output = rule_check_encounters(&ctx);
        assert!(output.effects.contains(&Effect::Map(MapEffect::TickEncounterTimer)));
    }
}
