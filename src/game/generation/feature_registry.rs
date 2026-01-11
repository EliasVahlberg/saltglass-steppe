use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureDef {
    pub id: String,
    pub handler: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct FeatureFile {
    features: Vec<FeatureDef>,
}

static FEATURE_MAP: Lazy<HashMap<String, FeatureDef>> = Lazy::new(|| {
    let data = include_str!("../../../data/map_features.json");
    let file: FeatureFile =
        serde_json::from_str(data).expect("Failed to parse data/map_features.json");
    file.features
        .into_iter()
        .map(|f| (f.id.clone(), f))
        .collect()
});

pub fn get_feature_def(id: &str) -> Option<&'static FeatureDef> {
    FEATURE_MAP.get(id)
}

pub fn has_feature(id: &str) -> bool {
    FEATURE_MAP.contains_key(id)
}
