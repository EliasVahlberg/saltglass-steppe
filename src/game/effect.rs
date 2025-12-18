use once_cell::sync::Lazy;
use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum VisualEffect {
    Blink { speed: u32, color: Color },
    Glow { color: Color },
    HitFlash { frames_remaining: u32, color: Color },
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
struct EffectDef {
    pub condition: EffectCondition,
    pub target: String,
    pub effect: String,
}

#[derive(Debug, Clone)]
pub struct ParsedEffect {
    pub condition: EffectCondition,
    pub effect: VisualEffect,
}

#[derive(Deserialize)]
struct EffectsFile {
    effects: Vec<EffectDef>,
}

struct EffectIndex {
    player_effects: Vec<ParsedEffect>,
    enemy_effects: HashMap<String, Vec<VisualEffect>>,
}

static EFFECT_INDEX: Lazy<EffectIndex> = Lazy::new(|| {
    let data = include_str!("../../data/effects.json");
    let file: EffectsFile = serde_json::from_str(data).expect("Failed to parse effects.json");
    
    let mut player_effects = Vec::new();
    let mut enemy_effects: HashMap<String, Vec<VisualEffect>> = HashMap::new();
    
    for def in file.effects {
        if let Some(effect) = parse_effect(&def.effect) {
            if def.target == "player" {
                player_effects.push(ParsedEffect {
                    condition: def.condition,
                    effect,
                });
            } else if def.target == "enemy" {
                if let Some(enemy_id) = def.condition.enemy_type {
                    enemy_effects.entry(enemy_id).or_default().push(effect);
                }
            }
        }
    }
    
    EffectIndex { player_effects, enemy_effects }
});

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
    if target != "player" { return Vec::new(); }
    EFFECT_INDEX.player_effects
        .iter()
        .filter(|e| e.condition.evaluate(ctx))
        .map(|e| e.effect.clone())
        .collect()
}

pub fn get_enemy_effects(enemy_id: &str) -> Vec<VisualEffect> {
    EFFECT_INDEX.enemy_effects
        .get(enemy_id)
        .cloned()
        .unwrap_or_default()
}
