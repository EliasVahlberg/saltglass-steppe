use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct EntityEffect {
    pub condition: String,
    pub effect: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Behavior {
    #[serde(rename = "type")]
    pub behavior_type: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub percent: Option<u32>,
    #[serde(default)]
    pub value: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EnemyDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub max_hp: i32,
    pub damage_min: i32,
    pub damage_max: i32,
    #[serde(default = "default_sight")]
    pub sight_range: i32,
    #[serde(default)]
    pub reflex: i32,
    #[serde(default)]
    pub armor: i32,
    #[serde(default)]
    pub accuracy: i32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub spawns_during_storm: bool,
    #[serde(default)]
    pub swarm: bool,
    #[serde(default)]
    pub behaviors: Vec<Behavior>,
    #[serde(default)]
    pub effects: Vec<EntityEffect>,
}

fn default_sight() -> i32 { 6 }

#[derive(Deserialize)]
struct EnemiesFile {
    enemies: Vec<EnemyDef>,
}

static ENEMY_DEFS: Lazy<HashMap<String, EnemyDef>> = Lazy::new(|| {
    let data = include_str!("../../data/enemies.json");
    let file: EnemiesFile = serde_json::from_str(data).expect("Failed to parse enemies.json");
    file.enemies.into_iter().map(|d| (d.id.clone(), d)).collect()
});

pub fn get_enemy_def(id: &str) -> Option<&'static EnemyDef> {
    ENEMY_DEFS.get(id)
}

pub fn all_enemy_ids() -> Vec<&'static str> {
    ENEMY_DEFS.keys().map(|s| s.as_str()).collect()
}

/// Context for evaluating behavior conditions
pub struct BehaviorContext<'a> {
    pub player_adaptations: usize,
    pub player_items: &'a [String],
}

impl Behavior {
    pub fn condition_met(&self, ctx: &BehaviorContext) -> bool {
        let cond = match &self.condition {
            Some(c) => c,
            None => return true,
        };
        
        if cond.starts_with("player_adaptations >= ") {
            if let Ok(n) = cond[22..].parse::<usize>() {
                return ctx.player_adaptations >= n;
            }
        }
        if cond.starts_with("player_has_item:") {
            let item_id = &cond[16..];
            return ctx.player_items.iter().any(|i| i == item_id);
        }
        false
    }
}

#[derive(Serialize, Deserialize)]
pub struct Enemy {
    pub x: i32,
    pub y: i32,
    pub id: String,
    pub hp: i32,
    #[serde(default)]
    pub ai_disabled: bool,
}

impl Enemy {
    pub fn new(x: i32, y: i32, id: &str) -> Self {
        let max_hp = get_enemy_def(id).map(|d| d.max_hp).unwrap_or(10);
        Self { x, y, id: id.to_string(), hp: max_hp, ai_disabled: false }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn def(&self) -> Option<&'static EnemyDef> {
        get_enemy_def(&self.id)
    }

    pub fn glyph(&self) -> char {
        self.def().map(|d| d.glyph.chars().next().unwrap_or('?')).unwrap_or('?')
    }

    pub fn name(&self) -> &str {
        self.def().map(|d| d.name.as_str()).unwrap_or("Unknown")
    }
}
