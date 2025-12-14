use once_cell::sync::Lazy;
use ratatui::style::Color;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum VisualEffect {
    Blink { speed: u32, color: Color },
    Glow { color: Color },
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffectCondition {
    #[serde(default)]
    pub low_hp: Option<i32>,
    #[serde(default)]
    pub storm_near: Option<u32>,
    #[serde(default)]
    pub has_adaptation: Option<bool>,
    #[serde(default)]
    pub on_tile: Option<String>,
    #[serde(default)]
    pub enemy_type: Option<String>,
    #[serde(default)]
    pub adaptations_hidden: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffectDef {
    pub id: String,
    pub condition: EffectCondition,
    pub target: String,
    pub effect: String,
}

#[derive(Deserialize)]
struct EffectsFile {
    effects: Vec<EffectDef>,
}

static EFFECT_DEFS: Lazy<Vec<EffectDef>> = Lazy::new(|| {
    let data = include_str!("../../data/effects.json");
    let file: EffectsFile = serde_json::from_str(data).expect("Failed to parse effects.json");
    file.effects
});

pub fn all_effects() -> &'static [EffectDef] {
    &EFFECT_DEFS
}

fn parse_color(s: &str) -> Color {
    match s {
        "Red" => Color::Red,
        "Green" => Color::Green,
        "Yellow" => Color::Yellow,
        "Blue" => Color::Blue,
        "Magenta" => Color::Magenta,
        "Cyan" => Color::Cyan,
        "White" => Color::White,
        "DarkGray" => Color::DarkGray,
        "LightRed" => Color::LightRed,
        "LightGreen" => Color::LightGreen,
        "LightYellow" => Color::LightYellow,
        "LightBlue" => Color::LightBlue,
        "LightMagenta" => Color::LightMagenta,
        "LightCyan" => Color::LightCyan,
        _ => Color::White,
    }
}

pub fn parse_effect(s: &str) -> Option<VisualEffect> {
    let s = s.trim();
    if s.starts_with("B(") && s.ends_with(')') {
        // Blink: B(@speed &color)
        let inner = &s[2..s.len()-1];
        let mut speed = 4u32;
        let mut color = Color::White;
        for part in inner.split_whitespace() {
            if part.starts_with('@') {
                speed = part[1..].parse().unwrap_or(4);
            } else if part.starts_with('&') {
                color = parse_color(&part[1..]);
            }
        }
        return Some(VisualEffect::Blink { speed, color });
    }
    if s.starts_with("G(") && s.ends_with(')') {
        // Glow: G(&color)
        let inner = &s[2..s.len()-1];
        let mut color = Color::White;
        for part in inner.split_whitespace() {
            if part.starts_with('&') {
                color = parse_color(&part[1..]);
            }
        }
        return Some(VisualEffect::Glow { color });
    }
    None
}

pub struct EffectContext {
    pub player_hp: i32,
    pub storm_turns: u32,
    pub has_adaptation: bool,
    pub on_glass: bool,
    pub adaptations_hidden: bool,
}

impl EffectCondition {
    pub fn evaluate(&self, ctx: &EffectContext) -> bool {
        if let Some(threshold) = self.low_hp {
            if ctx.player_hp > threshold { return false; }
        }
        if let Some(turns) = self.storm_near {
            if ctx.storm_turns > turns { return false; }
        }
        if let Some(true) = self.has_adaptation {
            if !ctx.has_adaptation { return false; }
        }
        if let Some(ref tile) = self.on_tile {
            if tile == "Glass" && !ctx.on_glass { return false; }
        }
        if let Some(true) = self.adaptations_hidden {
            if !ctx.adaptations_hidden { return false; }
        }
        true
    }
}

pub fn get_active_effects(ctx: &EffectContext, target: &str) -> Vec<VisualEffect> {
    all_effects()
        .iter()
        .filter(|e| e.target == target && e.condition.evaluate(ctx))
        .filter_map(|e| parse_effect(&e.effect))
        .collect()
}

pub fn get_enemy_effects(enemy_id: &str) -> Vec<VisualEffect> {
    all_effects()
        .iter()
        .filter(|e| e.target == "enemy" && e.condition.enemy_type.as_deref() == Some(enemy_id))
        .filter_map(|e| parse_effect(&e.effect))
        .collect()
}
