//! Turn processing — moved from state.rs.

use crate::game::{
    effects::{QuestNotifyKind, TurnPhase},
    mutations::{Mutation, SubsystemId},
    state::{GameState, MsgType},
    world_state::Weather,
};

impl GameState {
    pub fn end_turn(&mut self) {
        self.ensure_spatial_index();
        for phase in TurnPhase::sequence() {
            self.execute_phase(phase);
        }
    }

    fn execute_phase(&mut self, phase: &TurnPhase) {
        match phase {
            TurnPhase::ResetAp => {
                self.apply_mutations(vec![Mutation::ResetAp, Mutation::SetKillApRefund(false)]);
                // scar_lattice: reset temp armor when no enemies remain (combat over)
                if self.world.enemies.iter().all(|e| e.hp <= 0) {
                    self.player.scar_lattice_armor = 0;
                }
            }
            TurnPhase::TickStatusEffects => {
                self.apply_mutations(vec![Mutation::TickStatusEffects]);
            }
            TurnPhase::TickSubsystems => {
                self.apply_mutations(vec![
                    Mutation::TickSubsystem(SubsystemId::Psychic),
                    Mutation::TickSubsystem(SubsystemId::Skills),
                    Mutation::TickSubsystem(SubsystemId::Light),
                    Mutation::TickSubsystem(SubsystemId::Void),
                    Mutation::TickSubsystem(SubsystemId::Crystal),
                ]);
            }
            TurnPhase::AdvanceTurn => {
                self.apply_mutations(vec![Mutation::AdvanceTurn]);
                self.apply_mutations(vec![Mutation::TickHousekeeping]);
                let ctx = crate::game::effects::context::QueryContext::from_state(self);
                let output = crate::game::rules::turn::rule_check_adaptation(&ctx);
                let mutations: Vec<Mutation> = output.effects.into_iter()
                    .filter_map(crate::game::systems::effect_to_mutation)
                    .collect();
                self.apply_mutations(mutations);
                self.apply_mutations(vec![Mutation::QuestNotify(QuestNotifyKind::Turn)]);
            }
            TurnPhase::RunAI => {
                self.apply_mutations(vec![Mutation::RunAI]);
            }
            TurnPhase::TickStorm => {
                self.apply_mutations(vec![Mutation::TickStorm]);
            }
            TurnPhase::AdvanceTime => {
                use rand::Rng;
                let mut mutations = Vec::new();
                if self.turn.is_multiple_of(10) {
                    let new_time = (self.world.time_of_day as u32 + 1) % 24;
                    mutations.push(Mutation::AdvanceTime { new_time });
                    if new_time == 6 || new_time == 18 {
                        let roll = self.rng.gen_range(0..10u32);
                        let weather = match roll {
                            0..=6 => Weather::Clear,
                            7..=8 => Weather::Dusty,
                            _ => Weather::Sandstorm,
                        };
                        mutations.push(Mutation::SetWeather(weather));
                    }
                }
                self.apply_mutations(mutations);
            }
            TurnPhase::UpdateDerives => {
                self.ensure_spatial_index();
                self.update_lighting();
                self.update_fov();
            }
            TurnPhase::CheckEncounters => {
                let ctx = crate::game::effects::context::QueryContext::from_state(self);
                let output = crate::game::rules::turn::rule_check_encounters(&ctx);
                let mutations: Vec<Mutation> = output.effects.into_iter()
                    .filter_map(crate::game::systems::effect_to_mutation)
                    .collect();
                self.apply_mutations(mutations);
            }
        }
    }

    pub fn apply_status(&mut self, effect: crate::game::status::StatusEffect) {
        self.log_typed(
            format!("You are {}! ({} turns)", effect.name, effect.duration),
            MsgType::System,
        );
        self.player.status_effects.push(effect);
    }

    pub(crate) fn check_auto_end_turn(&mut self) {
        if self.player.ap <= 0 {
            self.end_turn();
        }
    }

    pub(crate) fn tick_turn_housekeeping(&mut self) {
        if self.player.adaptations_hidden_turns > 0 {
            self.player.adaptations_hidden_turns -= 1;
            if self.player.adaptations_hidden_turns == 0 {
                self.log_typed("The tincture wears off. Your glow returns.", MsgType::Status);
            }
        }
        self.triggered_effects.retain_mut(|e| {
            e.turns_remaining = e.turns_remaining.saturating_sub(1);
            e.turns_remaining > 0
        });
        self.decoys.retain_mut(|d| {
            d.turns_remaining = d.turns_remaining.saturating_sub(1);
            d.turns_remaining > 0
        });
        crate::game::systems::player::apply_light_effects(self);
    }
}

