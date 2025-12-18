//! Color theme system for consistent UI styling

use ratatui::style::Color;

/// Centralized color palette for the game UI
#[derive(Clone, Copy)]
pub struct Theme {
    // Health/resource colors
    pub hp_high: Color,
    pub hp_mid: Color,
    pub hp_low: Color,
    pub ap: Color,
    // Message log colors
    pub msg_combat: Color,
    pub msg_loot: Color,
    pub msg_status: Color,
    pub msg_dialogue: Color,
    pub msg_system: Color,
    pub msg_faded: Color,
    // Status effect colors
    pub status_burning: Color,
    pub status_poisoned: Color,
    pub status_frozen: Color,
    pub status_bleeding: Color,
    // UI chrome
    pub border: Color,
    pub title: Color,
    pub text: Color,
    pub text_dim: Color,
    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::classic()
    }
}

impl Theme {
    /// Classic roguelike color scheme
    pub const fn classic() -> Self {
        Self {
            hp_high: Color::Green,
            hp_mid: Color::Yellow,
            hp_low: Color::Red,
            ap: Color::Cyan,
            msg_combat: Color::Red,
            msg_loot: Color::Yellow,
            msg_status: Color::Magenta,
            msg_dialogue: Color::White,
            msg_system: Color::Gray,
            msg_faded: Color::DarkGray,
            status_burning: Color::Rgb(255, 102, 0),
            status_poisoned: Color::Rgb(0, 200, 0),
            status_frozen: Color::Rgb(0, 200, 255),
            status_bleeding: Color::Rgb(180, 0, 0),
            border: Color::White,
            title: Color::Yellow,
            text: Color::White,
            text_dim: Color::DarkGray,
            highlight: Color::Cyan,
        }
    }
}

/// Global theme instance
static THEME: Theme = Theme::classic();

/// Get the current theme
pub fn theme() -> &'static Theme {
    &THEME
}
