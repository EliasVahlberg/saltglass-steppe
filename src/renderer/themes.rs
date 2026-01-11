use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub themes: HashMap<String, Theme>,
    pub active_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub entities: ThemeEntityColors,
    pub tiles: ThemeTileColors,
    pub lighting: ThemeLightingColors,
    pub ui: ThemeUiColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeEntityColors {
    pub player: String,
    pub enemies: String,
    pub npcs: String,
    pub items: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTileColors {
    pub floor: String,
    pub wall: String,
    pub glass: String,
    pub stairs: String,
    pub world_exit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeLightingColors {
    pub torch: String,
    pub ambient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeUiColors {
    pub revealed_tile: String,
    pub look_cursor_bg: String,
    pub look_cursor_fg: String,
    pub hit_flash_bg: String,
    pub hit_flash_fg: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        let mut themes = HashMap::new();

        // Classic theme
        themes.insert(
            "classic".to_string(),
            Theme {
                name: "Classic".to_string(),
                description: "Traditional roguelike colors".to_string(),
                colors: ThemeColors {
                    entities: ThemeEntityColors {
                        player: "Yellow".to_string(),
                        enemies: "Red".to_string(),
                        npcs: "Green".to_string(),
                        items: "LightMagenta".to_string(),
                    },
                    tiles: ThemeTileColors {
                        floor: "DarkGray".to_string(),
                        wall: "Gray".to_string(),
                        glass: "Cyan".to_string(),
                        stairs: "Yellow".to_string(),
                        world_exit: "Green".to_string(),
                    },
                    lighting: ThemeLightingColors {
                        torch: "Yellow".to_string(),
                        ambient: "DarkGray".to_string(),
                    },
                    ui: ThemeUiColors {
                        revealed_tile: "DarkGray".to_string(),
                        look_cursor_bg: "White".to_string(),
                        look_cursor_fg: "Black".to_string(),
                        hit_flash_bg: "Red".to_string(),
                        hit_flash_fg: "White".to_string(),
                    },
                },
            },
        );

        // Dark theme
        themes.insert(
            "dark".to_string(),
            Theme {
                name: "Dark".to_string(),
                description: "Dark mode with muted colors".to_string(),
                colors: ThemeColors {
                    entities: ThemeEntityColors {
                        player: "LightYellow".to_string(),
                        enemies: "LightRed".to_string(),
                        npcs: "LightGreen".to_string(),
                        items: "LightMagenta".to_string(),
                    },
                    tiles: ThemeTileColors {
                        floor: "Black".to_string(),
                        wall: "DarkGray".to_string(),
                        glass: "DarkGray".to_string(),
                        stairs: "LightYellow".to_string(),
                        world_exit: "LightGreen".to_string(),
                    },
                    lighting: ThemeLightingColors {
                        torch: "LightYellow".to_string(),
                        ambient: "Black".to_string(),
                    },
                    ui: ThemeUiColors {
                        revealed_tile: "Black".to_string(),
                        look_cursor_bg: "Gray".to_string(),
                        look_cursor_fg: "White".to_string(),
                        hit_flash_bg: "DarkGray".to_string(),
                        hit_flash_fg: "LightRed".to_string(),
                    },
                },
            },
        );

        // High contrast theme
        themes.insert(
            "high_contrast".to_string(),
            Theme {
                name: "High Contrast".to_string(),
                description: "High contrast for accessibility".to_string(),
                colors: ThemeColors {
                    entities: ThemeEntityColors {
                        player: "White".to_string(),
                        enemies: "Red".to_string(),
                        npcs: "Green".to_string(),
                        items: "Magenta".to_string(),
                    },
                    tiles: ThemeTileColors {
                        floor: "Black".to_string(),
                        wall: "White".to_string(),
                        glass: "White".to_string(),
                        stairs: "Yellow".to_string(),
                        world_exit: "Green".to_string(),
                    },
                    lighting: ThemeLightingColors {
                        torch: "White".to_string(),
                        ambient: "Black".to_string(),
                    },
                    ui: ThemeUiColors {
                        revealed_tile: "Black".to_string(),
                        look_cursor_bg: "Yellow".to_string(),
                        look_cursor_fg: "Black".to_string(),
                        hit_flash_bg: "White".to_string(),
                        hit_flash_fg: "Black".to_string(),
                    },
                },
            },
        );

        Self {
            themes,
            active_theme: "classic".to_string(),
        }
    }
}

pub struct ThemeManager {
    config: ThemeConfig,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            config: ThemeConfig::default(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ThemeConfig = serde_json::from_str(&content)?;
        Ok(Self { config })
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_active_theme(&self) -> Option<&Theme> {
        self.config.themes.get(&self.config.active_theme)
    }

    pub fn set_active_theme(&mut self, theme_name: &str) -> bool {
        if self.config.themes.contains_key(theme_name) {
            self.config.active_theme = theme_name.to_string();
            true
        } else {
            false
        }
    }

    pub fn list_themes(&self) -> Vec<&Theme> {
        self.config.themes.values().collect()
    }

    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.config.themes.get(name)
    }
}
