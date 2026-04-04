#![allow(dead_code)]
use clap::{Parser, Subcommand};
use schemars::{JsonSchema, schema_for};
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;

use saltglass_steppe::game::combat::WeaponDef;
use saltglass_steppe::game::enemy::EnemyDef;
use saltglass_steppe::game::item::ItemDef;
use saltglass_steppe::game::npc::NpcDef;
use saltglass_steppe::game::quest::QuestDef;
use saltglass_steppe::game::trading::TraderTable;

#[derive(Parser)]
#[command(
    name = "schema-gen",
    about = "Generate JSON Schemas from Rust types or JSON data"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate schema from Rust types (schemars)
    Types {
        #[arg(long)]
        target: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        schema_id: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long, default_value = "draft-07")]
        draft: String,
    },
    /// Infer schema from JSON data
    Infer {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        schema_id: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        no_required: bool,
    },
}

#[derive(JsonSchema)]
struct Wrapper<T> {
    schema: String,
    #[serde(flatten)]
    payload: T,
}

#[derive(JsonSchema)]
struct ItemsPayload {
    items: Vec<ItemDef>,
}

#[derive(JsonSchema)]
struct WeaponsPayload {
    weapons: Vec<WeaponDef>,
}

#[derive(JsonSchema)]
struct EnemiesPayload {
    enemies: Vec<EnemyDef>,
}

#[derive(JsonSchema)]
struct QuestsPayload {
    quests: Vec<QuestDef>,
}

#[derive(JsonSchema)]
struct MainQuestlinePayload {
    main_questline: Vec<QuestDef>,
}

#[derive(JsonSchema)]
struct NpcsPayload {
    npcs: Vec<NpcDef>,
}

#[derive(JsonSchema)]
struct TradersPayload {
    traders: Vec<TraderTable>,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Types {
            target,
            output,
            schema_id,
            title,
            draft,
        } => {
            let mut schema = match target.as_str() {
                "items" => wrap_schema(schema_for!(Wrapper<ItemsPayload>)),
                "weapons" => wrap_schema(schema_for!(Wrapper<WeaponsPayload>)),
                "enemies" => wrap_schema(schema_for!(Wrapper<EnemiesPayload>)),
                "quests" => wrap_schema(schema_for!(Wrapper<QuestsPayload>)),
                "main_questline" => wrap_schema(schema_for!(Wrapper<MainQuestlinePayload>)),
                "npcs" => wrap_schema(schema_for!(Wrapper<NpcsPayload>)),
                "traders" => wrap_schema(schema_for!(Wrapper<TradersPayload>)),
                other => {
                    eprintln!("Unknown target: {}", other);
                    std::process::exit(1);
                }
            };

            apply_top_level_metadata(&mut schema, &draft, schema_id, title);
            write_schema(output, schema);
        }
        Command::Infer {
            input,
            output,
            schema_id,
            title,
            no_required,
        } => {
            let data = fs::read_to_string(&input)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", input.display(), e));
            let value: Value = serde_json::from_str(&data)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", input.display(), e));
            let mut schema = infer_schema(&value, !no_required);
            if let Some(id) = schema_id {
                schema.insert("$id".to_string(), Value::String(id));
            }
            if let Some(t) = title {
                schema.insert("title".to_string(), Value::String(t));
            }
            schema.insert(
                "$schema".to_string(),
                Value::String("http://json-schema.org/draft-07/schema#".to_string()),
            );
            write_schema(output, Value::Object(schema));
        }
    }
}

fn wrap_schema(schema: schemars::schema::RootSchema) -> Value {
    serde_json::to_value(&schema).expect("Failed to serialize schema")
}

fn apply_top_level_metadata(
    schema: &mut Value,
    draft: &str,
    schema_id: Option<String>,
    title: Option<String>,
) {
    let draft_url = match draft {
        "draft-07" => "http://json-schema.org/draft-07/schema#",
        _ => "http://json-schema.org/draft-07/schema#",
    };

    if let Value::Object(map) = schema {
        map.insert("$schema".to_string(), Value::String(draft_url.to_string()));
        if let Some(id) = schema_id {
            map.insert("$id".to_string(), Value::String(id));
        }
        if let Some(t) = title {
            map.insert("title".to_string(), Value::String(t));
        }
    }
}

fn write_schema(output: PathBuf, schema: Value) {
    let serialized = serde_json::to_string_pretty(&schema)
        .unwrap_or_else(|e| panic!("Failed to serialize schema: {}", e));
    fs::write(&output, serialized)
        .unwrap_or_else(|e| panic!("Failed to write {}: {}", output.display(), e));
}

fn infer_schema(value: &Value, use_required: bool) -> Map<String, Value> {
    match value {
        Value::Null => map_type("null"),
        Value::Bool(_) => map_type("boolean"),
        Value::Number(n) => {
            if n.is_i64() {
                map_type("integer")
            } else {
                map_type("number")
            }
        }
        Value::String(_) => map_type("string"),
        Value::Array(arr) => {
            let mut map = Map::new();
            map.insert("type".to_string(), Value::String("array".to_string()));
            if arr.is_empty() {
                map.insert("items".to_string(), Value::Object(map_type("object")));
                return map;
            }

            let mut item_schemas = Vec::new();
            for item in arr {
                item_schemas.push(Value::Object(infer_schema(item, use_required)));
            }

            let items_schema = merge_schemas(item_schemas);
            map.insert("items".to_string(), items_schema);
            map
        }
        Value::Object(obj) => {
            let mut map = Map::new();
            map.insert("type".to_string(), Value::String("object".to_string()));

            let mut props = Map::new();
            let mut required = Vec::new();
            for (key, val) in obj.iter() {
                props.insert(key.clone(), Value::Object(infer_schema(val, use_required)));
                if use_required {
                    required.push(Value::String(key.clone()));
                }
            }

            map.insert("properties".to_string(), Value::Object(props));
            if use_required && !required.is_empty() {
                map.insert("required".to_string(), Value::Array(required));
            }
            map.insert("additionalProperties".to_string(), Value::Bool(false));
            map
        }
    }
}

fn merge_schemas(schemas: Vec<Value>) -> Value {
    let mut unique = Vec::new();
    for schema in schemas {
        if !unique.contains(&schema) {
            unique.push(schema);
        }
    }

    if unique.len() == 1 {
        return unique[0].clone();
    }

    let mut map = Map::new();
    map.insert("oneOf".to_string(), Value::Array(unique));
    Value::Object(map)
}

fn map_type(t: &str) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("type".to_string(), Value::String(t.to_string()));
    map
}
