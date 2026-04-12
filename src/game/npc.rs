use super::adaptation::Adaptation;
use super::entity::Entity;
use super::status::StatusEffect;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::game::data_loader::{DataLoader, DataSource, HasId};
/// Context for evaluating dialogue conditions
pub struct DialogueContext<'a> {
    pub adaptations: &'a [Adaptation],
    pub inventory: &'a [String],
    pub salt_scrip: u32,
    pub faction_reputation: &'a HashMap<String, i32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DialogueCondition {
    #[serde(default)]
    pub has_adaptation: Option<String>,
    #[serde(default)]
    pub adaptation_count_gte: Option<usize>,
    #[serde(default)]
    pub has_item: Option<String>,
    #[serde(default)]
    pub min_salt_scrip: Option<u32>,
    #[serde(default)]
    pub min_reputation: Option<HashMap<String, i32>>,
    /// Captures unrecognized condition keys so they cause the condition to fail
    /// rather than being silently ignored.
    #[serde(flatten)]
    #[schemars(skip)]
    pub unknown: HashMap<String, serde_json::Value>,
}

impl DialogueCondition {
    pub fn evaluate(&self, ctx: &DialogueContext) -> bool {
        // Unrecognized condition fields always fail
        if !self.unknown.is_empty() {
            return false;
        }
        if let Some(ref name) = self.has_adaptation
            && !ctx.adaptations.iter().any(|a| a.name() == name)
        {
            return false;
        }
        if let Some(count) = self.adaptation_count_gte
            && ctx.adaptations.len() < count
        {
            return false;
        }
        if let Some(ref item_id) = self.has_item
            && !ctx.inventory.iter().any(|i| i == item_id)
        {
            return false;
        }
        if let Some(amount) = self.min_salt_scrip
            && ctx.salt_scrip < amount
        {
            return false;
        }
        if let Some(ref reqs) = self.min_reputation {
            for (faction, min_rep) in reqs {
                let current = ctx.faction_reputation.get(faction).copied().unwrap_or(0);
                if current < *min_rep {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DialogueEntry {
    #[serde(default)]
    pub conditions: Vec<DialogueCondition>,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ActionEffect {
    #[serde(default)]
    pub heal: Option<i32>,
    #[serde(default)]
    pub trade: Option<bool>,
    #[serde(default)]
    pub gives_item: Option<String>,
    #[serde(default)]
    pub consumes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NpcAction {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub conditions: Vec<DialogueCondition>,
    pub effect: ActionEffect,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NpcDef {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub faction: String,
    #[serde(default)]
    pub description: String,
    pub dialogue: Vec<DialogueEntry>,
    #[serde(default)]
    pub actions: Vec<NpcAction>,
    /// Items available for purchase (item_id)
    #[serde(default)]
    pub shop_inventory: Vec<String>,
}

impl HasId for NpcDef {
    fn id(&self) -> &str {
        &self.id
    }
}

static NPC_DEFS: Lazy<DataLoader<NpcDef>> = Lazy::new(|| {
    DataLoader::load_single(
        DataSource::new("data/npcs.json", include_str!("../../data/npcs.json")),
        "npcs",
        "npcs_v1",
    )
});

pub fn get_npc_def(id: &str) -> Option<&'static NpcDef> {
    NPC_DEFS.get(id)
}

pub fn all_npc_ids() -> Vec<&'static str> {
    NPC_DEFS.ids()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub x: i32,
    pub y: i32,
    pub id: String,
    pub talked: bool,
    #[serde(default)]
    pub backstory: Option<String>,
}

impl Npc {
    pub fn new(x: i32, y: i32, id: &str) -> Self {
        Self {
            x,
            y,
            id: id.to_string(),
            talked: false,
            backstory: None,
        }
    }

    pub fn def(&self) -> Option<&'static NpcDef> {
        get_npc_def(&self.id)
    }

    pub fn glyph(&self) -> char {
        self.def()
            .map(|d| d.glyph.chars().next().unwrap_or('?'))
            .unwrap_or('?')
    }

    pub fn name(&self) -> &str {
        self.def().map(|d| d.name.as_str()).unwrap_or("Unknown")
    }

    pub fn description(&self) -> &str {
        self.def().map(|d| d.description.as_str()).unwrap_or("")
    }

    pub fn backstory(&self) -> Option<&str> {
        self.backstory.as_deref()
    }

    pub fn dialogue(&self, ctx: &DialogueContext) -> &str {
        if let Some(def) = self.def() {
            for entry in &def.dialogue {
                let all_match =
                    entry.conditions.is_empty() || entry.conditions.iter().all(|c| c.evaluate(ctx));
                if all_match {
                    return &entry.text;
                }
            }
        }
        "..."
    }

    pub fn available_actions(&self, ctx: &DialogueContext) -> Vec<&'static NpcAction> {
        if let Some(def) = self.def() {
            def.actions
                .iter()
                .filter(|a| a.conditions.is_empty() || a.conditions.iter().all(|c| c.evaluate(ctx)))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Entity for Npc {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    fn hp(&self) -> Option<i32> {
        None
    }
    fn set_hp(&mut self, _hp: i32) {}
    fn max_hp(&self) -> Option<i32> {
        None
    }

    fn status_effects(&self) -> &[StatusEffect] {
        &[]
    }
    fn status_effects_mut(&mut self) -> &mut Vec<StatusEffect> {
        panic!("NPCs do not have status effects")
    }

    fn name(&self) -> &str {
        self.def().map(|d| d.name.as_str()).unwrap_or(&self.id)
    }

    fn glyph(&self) -> char {
        self.def()
            .and_then(|d| d.glyph.chars().next())
            .unwrap_or('@')
    }
}
