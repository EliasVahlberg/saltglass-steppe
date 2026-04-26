use crossterm::event::KeyCode;
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KeyboardConfig {
    pub gameplay: GameplayKeys,
    pub worldmap: WorldMapKeys,
    pub menus: MenuKeys,
    pub debug: DebugKeys,
}

#[derive(Debug, Deserialize)]
pub struct GameplayKeys {
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub move_up_arrow: String,
    pub move_down_arrow: String,
    pub move_left_arrow: String,
    pub move_right_arrow: String,
    pub wait: String,
    pub look: String,
    pub examine: String,
    pub interact: String,
    pub auto_explore: String,
    pub inventory: String,
    pub quest_log: String,
    pub crafting: String,
    pub wiki: String,
    pub psychic_menu: String,
    pub faction_menu: String,
    pub void_menu: String,
    pub crystal_menu: String,
    pub light_menu: String,
    pub skills_menu: String,
    pub ranged_attack: String,
    pub target_mode: String,
    pub use_stairs: String,
    pub use_item_1: String,
    pub use_item_2: String,
    pub use_item_3: String,
    pub save: String,
    pub load: String,
    pub pause_menu: String,
}

#[derive(Debug, Deserialize)]
pub struct WorldMapKeys {
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub move_up_arrow: String,
    pub move_down_arrow: String,
    pub move_left_arrow: String,
    pub move_right_arrow: String,
    pub inspect_toggle: String,
    pub set_target: String,
    pub auto_move: String,
    pub center_on_player: String,
    pub travel: String,
    pub close: String,
}

#[derive(Debug, Deserialize)]
pub struct MenuKeys {
    pub close: String,
    pub navigate_up: String,
    pub navigate_down: String,
    pub navigate_up_arrow: String,
    pub navigate_down_arrow: String,
    pub select: String,
    pub tab_next: String,
    pub tab_prev: String,
}

#[derive(Debug, Deserialize)]
pub struct DebugKeys {
    pub console_toggle: String,
    pub debug_menu: String,
}

pub static CONFIG: Lazy<KeyboardConfig> = Lazy::new(|| {
    let config_str = include_str!("../../data/keyboard_config.json");
    serde_json::from_str(config_str).expect("Failed to parse keyboard_config.json")
});

pub fn get_config() -> &'static KeyboardConfig {
    &CONFIG
}

fn parse_key(key_str: &str) -> KeyCode {
    match key_str {
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "BackTab" => KeyCode::BackTab,
        "Backspace" => KeyCode::Backspace,
        s if s.starts_with('F') && s.len() > 1 => {
            if let Ok(n) = s[1..].parse::<u8>() {
                KeyCode::F(n)
            } else {
                KeyCode::Char(s.chars().next().unwrap_or(' '))
            }
        }
        s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap_or(' ')),
        _ => KeyCode::Char(' '), // fallback
    }
}

impl KeyboardConfig {
    pub fn matches_gameplay(&self, key: KeyCode, action: &str) -> bool {
        let expected = match action {
            "move_up" => parse_key(&self.gameplay.move_up),
            "move_down" => parse_key(&self.gameplay.move_down),
            "move_left" => parse_key(&self.gameplay.move_left),
            "move_right" => parse_key(&self.gameplay.move_right),
            "wait" => parse_key(&self.gameplay.wait),
            "look" => parse_key(&self.gameplay.look),
            "examine" => parse_key(&self.gameplay.examine),
            "interact" => parse_key(&self.gameplay.interact),
            "auto_explore" => parse_key(&self.gameplay.auto_explore),
            "inventory" => parse_key(&self.gameplay.inventory),
            "quest_log" => parse_key(&self.gameplay.quest_log),
            "crafting" => parse_key(&self.gameplay.crafting),
            "wiki" => parse_key(&self.gameplay.wiki),
            "psychic_menu" => parse_key(&self.gameplay.psychic_menu),
            "faction_menu" => parse_key(&self.gameplay.faction_menu),
            "void_menu" => parse_key(&self.gameplay.void_menu),
            "crystal_menu" => parse_key(&self.gameplay.crystal_menu),
            "light_menu" => parse_key(&self.gameplay.light_menu),
            "skills_menu" => parse_key(&self.gameplay.skills_menu),
            "ranged_attack" => parse_key(&self.gameplay.ranged_attack),
            "target_mode" => parse_key(&self.gameplay.target_mode),
            "use_stairs" => parse_key(&self.gameplay.use_stairs),
            "use_item_1" => parse_key(&self.gameplay.use_item_1),
            "use_item_2" => parse_key(&self.gameplay.use_item_2),
            "use_item_3" => parse_key(&self.gameplay.use_item_3),
            "save" => parse_key(&self.gameplay.save),
            "load" => parse_key(&self.gameplay.load),
            "pause_menu" => parse_key(&self.gameplay.pause_menu),
            _ => return false,
        };
        key == expected
    }

    pub fn matches_worldmap(&self, key: KeyCode, action: &str) -> bool {
        let expected = match action {
            "move_up" => parse_key(&self.worldmap.move_up),
            "move_down" => parse_key(&self.worldmap.move_down),
            "move_left" => parse_key(&self.worldmap.move_left),
            "move_right" => parse_key(&self.worldmap.move_right),
            "inspect_toggle" => parse_key(&self.worldmap.inspect_toggle),
            "set_target" => parse_key(&self.worldmap.set_target),
            "auto_move" => parse_key(&self.worldmap.auto_move),
            "center_on_player" => parse_key(&self.worldmap.center_on_player),
            "travel" => parse_key(&self.worldmap.travel),
            "close" => parse_key(&self.worldmap.close),
            _ => return false,
        };
        key == expected
    }

    pub fn matches_menu(&self, key: KeyCode, action: &str) -> bool {
        let expected = match action {
            "close" => parse_key(&self.menus.close),
            "navigate_up" => parse_key(&self.menus.navigate_up),
            "navigate_down" => parse_key(&self.menus.navigate_down),
            "select" => parse_key(&self.menus.select),
            "tab_next" => parse_key(&self.menus.tab_next),
            "tab_prev" => parse_key(&self.menus.tab_prev),
            _ => return false,
        };
        key == expected
    }

    pub fn matches_debug(&self, key: KeyCode, action: &str) -> bool {
        let expected = match action {
            "console_toggle" => parse_key(&self.debug.console_toggle),
            "debug_menu" => parse_key(&self.debug.debug_menu),
            _ => return false,
        };
        key == expected
    }
}
