use super::event::GameEvent;
use super::state::GameState;

pub mod ai;
pub mod combat;
pub mod loot;
pub mod movement;
pub mod quest;
pub mod status;
pub mod storm;

pub use loot::LootSystem;
pub use quest::QuestSystem;
pub use status::StatusEffectSystem;
pub use storm::StormSystem;

/// Trait for game systems that operate on GameState
pub trait System {
    /// Run the system logic for one turn/frame
    fn update(&self, state: &mut GameState);

    /// Handle specific events
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
