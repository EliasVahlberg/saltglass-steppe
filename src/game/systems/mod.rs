use super::state::GameState;
use super::event::GameEvent;

pub mod combat;
pub mod ai;
pub mod movement;
pub mod loot;
pub mod quest;
pub mod status;

pub use loot::LootSystem;
pub use quest::QuestSystem;
pub use status::StatusEffectSystem;

/// Trait for game systems that operate on GameState
pub trait System {
    /// Run the system logic for one turn/frame
    fn update(&self, state: &mut GameState);
    
    /// Handle specific events
    fn on_event(&self, state: &mut GameState, event: &GameEvent);
}
