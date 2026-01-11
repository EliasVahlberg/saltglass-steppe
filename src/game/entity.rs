use crate::game::status::StatusEffect;

/// Common trait for all game entities (player, enemies, NPCs)
/// Provides a unified interface for systems that operate on any entity
pub trait Entity {
    /// Get entity's X position
    fn x(&self) -> i32;

    /// Get entity's Y position
    fn y(&self) -> i32;

    /// Set entity's position
    fn set_position(&mut self, x: i32, y: i32);

    /// Get entity's current HP (None if entity doesn't have HP)
    fn hp(&self) -> Option<i32>;

    /// Set entity's HP (no-op if entity doesn't have HP)
    fn set_hp(&mut self, hp: i32);

    /// Get entity's max HP (None if entity doesn't have HP)
    fn max_hp(&self) -> Option<i32>;

    /// Check if entity is alive
    fn is_alive(&self) -> bool {
        self.hp().map(|hp| hp > 0).unwrap_or(true)
    }

    /// Get entity's status effects
    fn status_effects(&self) -> &[StatusEffect];

    /// Get mutable status effects
    fn status_effects_mut(&mut self) -> &mut Vec<StatusEffect>;

    /// Check if entity has a specific status effect
    fn has_status(&self, id: &str) -> bool {
        self.status_effects()
            .iter()
            .any(|e| e.id == id && e.duration > 0)
    }

    /// Get entity's display name
    fn name(&self) -> &str;

    /// Get entity's display glyph
    fn glyph(&self) -> char;
}

/// Entity type identifier for generic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Player,
    Enemy,
    Npc,
}
