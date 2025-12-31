use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::game::status::StatusEffect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AIDemeanor {
    #[default]
    Aggressive,  // Attacks on sight
    Defensive,   // Takes cover, attacks when close
    Neutral,     // Ignores unless attacked
    Friendly,    // Helps player, becomes defensive if attacked
    Pacifist,    // Flees when threatened
}

#[derive(Clone, Debug, Deserialize)]
pub struct EntityEffect {
    pub condition: String,
    pub effect: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LootEntry {
    pub item: String,
    pub weight: u32,
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
    #[serde(default)]
    pub range: Option<u32>,
    #[serde(default)]
    pub damage: Option<u32>,
    #[serde(default)]
    pub duration: Option<u32>,
    #[serde(default)]
    pub spawns: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
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
    pub demeanor: AIDemeanor,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub spawns_during_storm: bool,
    #[serde(default)]
    pub swarm: bool,
    #[serde(default)]
    pub xp_value: u32,
    #[serde(default)]
    pub behaviors: Vec<Behavior>,
    #[serde(default)]
    pub effects: Vec<EntityEffect>,
    #[serde(default)]
    pub loot_table: Vec<LootEntry>,
    #[serde(default)]
    pub is_spawner: bool,
    #[serde(default)]
    pub spawn_rate: u32,
    #[serde(default)]
    pub max_spawns: u32,
    #[serde(default)]
    pub spawn_types: Vec<String>,
    #[serde(default)]
    pub behavior_id: Option<String>,
    #[serde(default)]
    pub ranged_attack: bool,
    #[serde(default)]
    pub attack_range: u32,
    #[serde(default)]
    pub aoe_attack: bool,
    #[serde(default)]
    pub aoe_radius: u32,
    #[serde(default)]
    pub aoe_warning_turns: u32,
    #[serde(default = "default_level")]
    pub level: u32,
}

fn default_sight() -> i32 { 6 }
fn default_level() -> u32 { 1 }

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
    #[serde(default)]
    pub provoked: bool,  // Set when attacked by player
    #[serde(default)]
    pub inventory: Vec<String>,  // Items carried by enemy
    #[serde(default)]
    pub status_effects: Vec<StatusEffect>,
    #[serde(default)]
    pub spawned_count: u32,  // For spawners
    #[serde(default)]
    pub last_spawn_turn: u32,  // For spawners
    #[serde(default)]
    pub aoe_target: Option<(i32, i32)>,  // AOE attack target
    #[serde(default)]
    pub aoe_warning_turns: u32,  // Turns until AOE attack
    #[serde(default)]
    pub swarm_leader: bool,  // Is this the swarm leader
    #[serde(default)]
    pub swarm_id: Option<String>,  // Swarm group identifier
}

impl Enemy {
    pub fn new(x: i32, y: i32, id: &str) -> Self {
        let max_hp = get_enemy_def(id).map(|d| d.max_hp).unwrap_or(10);
        Self { 
            x, y, id: id.to_string(), hp: max_hp, 
            ai_disabled: false, provoked: false, 
            inventory: Vec::new(),
            status_effects: Vec::new(),
            spawned_count: 0,
            last_spawn_turn: 0,
            aoe_target: None,
            aoe_warning_turns: 0,
            swarm_leader: false,
            swarm_id: None,
        }
    }

    pub fn new_swarm_member(x: i32, y: i32, id: &str, swarm_id: String, is_leader: bool) -> Self {
        let mut enemy = Self::new(x, y, id);
        enemy.swarm_id = Some(swarm_id);
        enemy.swarm_leader = is_leader;
        enemy
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

    pub fn demeanor(&self) -> AIDemeanor {
        self.def().map(|d| d.demeanor).unwrap_or_default()
    }

    /// Returns true if this enemy should act hostile toward player
    pub fn is_hostile(&self) -> bool {
        match self.demeanor() {
            AIDemeanor::Aggressive => true,
            AIDemeanor::Defensive => true,
            AIDemeanor::Neutral => self.provoked,
            AIDemeanor::Friendly => self.provoked,
            AIDemeanor::Pacifist => false,
        }
    }

    /// Returns true if this enemy should flee
    pub fn should_flee(&self) -> bool {
        match self.demeanor() {
            AIDemeanor::Pacifist => self.provoked,
            AIDemeanor::Defensive => {
                // Defensive enemies flee when HP drops below 30%
                let max_hp = self.def().map(|d| d.max_hp).unwrap_or(10);
                self.hp * 100 / max_hp < 30
            }
            _ => false,
        }
    }

    pub fn can_spawn(&self, current_turn: u32) -> bool {
        if let Some(def) = self.def() {
            def.is_spawner && 
            self.spawned_count < def.max_spawns &&
            current_turn >= self.last_spawn_turn + def.spawn_rate
        } else {
            false
        }
    }

    pub fn has_ranged_attack(&self) -> bool {
        self.def().map(|d| d.ranged_attack).unwrap_or(false)
    }

    pub fn attack_range(&self) -> u32 {
        self.def().map(|d| d.attack_range).unwrap_or(1)
    }

    pub fn has_aoe_attack(&self) -> bool {
        self.def().map(|d| d.aoe_attack).unwrap_or(false)
    }

    pub fn is_preparing_aoe(&self) -> bool {
        self.aoe_target.is_some() && self.aoe_warning_turns > 0
    }

    pub fn start_aoe_attack(&mut self, target_x: i32, target_y: i32) {
        if let Some(def) = self.def() {
            if def.aoe_attack {
                self.aoe_target = Some((target_x, target_y));
                self.aoe_warning_turns = def.aoe_warning_turns;
            }
        }
    }

    pub fn tick_aoe_warning(&mut self) -> bool {
        if self.aoe_warning_turns > 0 {
            self.aoe_warning_turns -= 1;
            self.aoe_warning_turns == 0
        } else {
            false
        }
    }

    pub fn apply_status(&mut self, id: &str, duration: i32) {
        if let Some(effect) = self.status_effects.iter_mut().find(|e| e.id == id) {
            effect.duration = effect.duration.max(duration);
            effect.stacks += 1;
        } else {
            self.status_effects.push(StatusEffect {
                id: id.to_string(),
                name: id.to_string(),
                duration,
                stacks: 1,
            });
        }
    }

    pub fn has_status_effect(&self, id: &str) -> bool {
        self.status_effects.iter().any(|e| e.id == id)
    }
}
