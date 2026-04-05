use super::event::GameEvent;
use super::state::GameState;

pub mod ai;
pub mod combat;
pub mod interact;
pub mod items;
pub mod loot;
pub mod movement;
pub mod player;
pub mod quest;
pub mod status;
pub mod storm;
pub mod world;

pub use loot::LootSystem;
pub use status::StatusEffectSystem;
pub use storm::StormSystem;

/// Convert a single Effect to its Mutation equivalent.
/// Used by system handlers that call legacy rule functions returning RuleOutput.
pub fn effect_to_mutation(effect: crate::game::effects::Effect) -> Option<crate::game::mutations::Mutation> {
    use crate::game::effects::{
        CombatEffect, Effect, EventEffect, ItemEffect, MapEffect, PlayerEffect, ResourceEffect,
    };
    use crate::game::mutations::Mutation;

    match effect {
        Effect::Player(e) => Some(match e {
            PlayerEffect::Heal { amount } => {
                // Heal is a delta — convert via SetPlayerHp requires current hp.
                // Emit a dedicated AddHp delta mutation.
                Mutation::AddHp(amount)
            }
            PlayerEffect::SpendAp { amount } => Mutation::SpendAp(amount),
            PlayerEffect::ResetWaitCounter => Mutation::SetWaitCounter(0),
            PlayerEffect::IncrementWaitCounter => Mutation::IncrementWaitCounter,
            PlayerEffect::AdvanceTurn => Mutation::AdvanceTurn,
            PlayerEffect::AllocateStat { stat } => Mutation::AllocateStat(stat),
            PlayerEffect::SuppressAdaptations { turns } => Mutation::SuppressAdaptations { turns },
            PlayerEffect::SetPhaseMode { enabled } => Mutation::SetPhaseMode(enabled),
            PlayerEffect::ApplyStatusEffect { effect_id, duration } => {
                Mutation::AddStatusEffect { id: effect_id, duration }
            }
            PlayerEffect::ModifyRefraction { delta } => Mutation::AddRefraction(delta),
            PlayerEffect::SetPosition { x, y } => Mutation::SetPlayerPosition { x, y },
            PlayerEffect::TakeDamage { amount } => Mutation::AddHp(-amount),
            PlayerEffect::PlaceDecoy { x, y } => Mutation::PlaceDecoy { x, y },
            _ => return None,
        }),
        Effect::Item(e) => Some(match e {
            ItemEffect::RemoveFromInventory { index } => Mutation::RemoveFromInventory(index),
            ItemEffect::AddToInventory { item_id } => Mutation::AddToInventory(item_id),
            ItemEffect::Consume { inventory_index, .. } => Mutation::RemoveFromInventory(inventory_index),
            ItemEffect::Equip { item_id, slot } => Mutation::Equip { slot, item_id },
            ItemEffect::Unequip { slot } => Mutation::Unequip(slot),
            ItemEffect::RecalcStats => Mutation::RecalcStats,
            _ => return None,
        }),
        Effect::Map(e) => Some(match e {
            MapEffect::RevealAll => Mutation::RevealAll,
            MapEffect::DamageWall { x, y, damage } => Mutation::DamageWall { x, y, damage },
            MapEffect::ClearStormHighlight { tile_index } => Mutation::ClearStormHighlight(tile_index),
            _ => return None,
        }),
        Effect::Resource(e) => Some(match e {
            ResourceEffect::GainLightEnergy { amount } => Mutation::SetLightEnergy(amount),
            ResourceEffect::GainVoidEnergy { amount } => Mutation::AddVoidEnergy(amount),
            ResourceEffect::GainVoidExposure { amount } => Mutation::AddVoidExposure(amount),
            ResourceEffect::GainResonanceEnergy { amount } => Mutation::SetResonanceEnergy(amount),
            ResourceEffect::PlaceCrystal { x, y, frequency } => Mutation::PlaceCrystal { x, y, frequency },
        }),
        Effect::Event(e) => Some(match e {
            EventEffect::OpenBook { book_id } => Mutation::OpenBook(book_id),
            EventEffect::QuestNotify { kind } => Mutation::QuestNotify(kind),
            _ => return None,
        }),
        Effect::Combat(e) => Some(match e {
            CombatEffect::StunEnemy { enemy_idx, duration } => {
                Mutation::StunEnemy { idx: enemy_idx, duration }
            }
            _ => return None,
        }),
        _ => None,
    }
}

pub fn rule_output_to_mutations(
    output: crate::game::effects::RuleOutput,
    msg_type_map: fn(&str) -> crate::game::state::MsgType,
) -> Vec<crate::game::mutations::Mutation> {
    use crate::game::effects::Presentation;
    use crate::game::mutations::Mutation;

    let mut out: Vec<Mutation> = output.effects.into_iter()
        .filter_map(effect_to_mutation)
        .collect();
    for p in output.presentation {
        let Presentation::LogMessage { text, msg_type } = p;
        out.push(Mutation::LogMessage { text, msg_type: msg_type_map(&msg_type) });
    }
    out
}

/// Trait for game systems that operate on GameState
pub trait System {
    /// Run the system logic for one turn/frame
    fn update(&self, state: &mut GameState);

    /// Handle specific events
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
