use jsonschema::JSONSchema;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct DataSource<'a> {
    pub label: &'a str,
    pub data: &'a str,
}

impl<'a> DataSource<'a> {
    pub fn new(label: &'a str, data: &'a str) -> Self {
        Self { label, data }
    }
}

pub trait HasId {
    fn id(&self) -> &str;
}

pub struct DataLoader<T> {
    data: HashMap<String, T>,
}

impl<T: DeserializeOwned + HasId> DataLoader<T> {
    pub fn from_map(data: HashMap<String, T>) -> Self {
        Self { data }
    }

    pub fn into_map(self) -> HashMap<String, T> {
        self.data
    }

    pub fn load_single(source: DataSource<'_>, list_key: &str, expected_schema: &str) -> Self {
        Self::load_multiple(&[source], list_key, expected_schema)
    }

    pub fn load_multiple(
        sources: &[DataSource<'_>],
        list_key: &str,
        expected_schema: &str,
    ) -> Self {
        let mut data = HashMap::new();

        for source in sources {
            let entries = Self::parse_source(source, list_key, expected_schema);
            for entry in entries {
                let id = entry.id().to_string();
                if data.contains_key(&id) {
                    panic!("Duplicate ID '{}' found in {}", id, source.label);
                }
                data.insert(id, entry);
            }
        }

        Self { data }
    }

    pub fn get(&self, id: &str) -> Option<&T> {
        self.data.get(id)
    }

    pub fn all(&self) -> Vec<&T> {
        self.data.values().collect()
    }

    pub fn ids(&self) -> Vec<&str> {
        self.data.keys().map(|id| id.as_str()).collect()
    }

    fn parse_source(
        source: &DataSource<'_>,
        list_key: &str,
        expected_schema: &str,
    ) -> Vec<T> {
        let root: Value = serde_json::from_str(source.data)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", source.label, e));

        Self::validate_against_schema(expected_schema, source.label, &root);

        let schema = root
            .get("schema")
            .and_then(|value| value.as_str())
            .unwrap_or("");

        if schema.is_empty() {
            panic!(
                "Missing schema field in {} (expected '{}')",
                source.label, expected_schema
            );
        }

        if schema != expected_schema {
            panic!(
                "Invalid schema version in {}: expected '{}', got '{}'",
                source.label, expected_schema, schema
            );
        }

        let entries = root
            .get(list_key)
            .unwrap_or_else(|| panic!("Missing '{}' array in {}", list_key, source.label));

        serde_json::from_value(entries.clone()).unwrap_or_else(|e| {
            panic!(
                "Failed to parse '{}' entries in {}: {}",
                list_key, source.label, e
            )
        })
    }

    fn validate_against_schema(expected_schema: &str, label: &str, root: &Value) {
        let schema_json = match expected_schema {
            "enemies_v1" => include_str!("../../schemas/enemies_v1.json"),
            "items_v1" => include_str!("../../schemas/items_v1.json"),
            "weapons_v1" => include_str!("../../schemas/weapons_v1.json"),
            "quests_v1" => include_str!("../../schemas/quests_v1.json"),
            "npcs_v1" => include_str!("../../schemas/npcs_v1.json"),
            _ => return,
        };

        let schema_value: Value = serde_json::from_str(schema_json)
            .unwrap_or_else(|e| panic!("Failed to parse schema {}: {}", expected_schema, e));

        let compiled = JSONSchema::compile(&schema_value).unwrap_or_else(|e| {
            panic!(
                "Failed to compile schema {} for {}: {}",
                expected_schema, label, e
            )
        });

        if let Err(errors) = compiled.validate(root) {
            let mut messages = Vec::new();
            for error in errors.take(5) {
                messages.push(error.to_string());
            }
            panic!(
                "Schema validation failed for {} ({}): {}",
                label,
                expected_schema,
                messages.join("; ")
            );
        }
    }
}
