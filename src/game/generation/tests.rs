#[cfg(test)]
mod generation_tests {
    use crate::game::generation::weighted_table::{WeightedEntry, WeightedTable};
    use crate::game::generation::pipeline::{GenerationConfig, GenerationPass, GenerationPipeline, PassType};
    use crate::game::generation::templates::{TemplateLibrary, TemplateContext, ContentTemplate, TemplateVariant};
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_weighted_table_selection() {
        let entries = vec![
            WeightedEntry { item: "common".to_string(), weight: 70.0 },
            WeightedEntry { item: "rare".to_string(), weight: 20.0 },
            WeightedEntry { item: "legendary".to_string(), weight: 10.0 },
        ];
        
        let table = WeightedTable::new(entries);
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        // Test deterministic selection
        let first_selection = table.select(&mut rng);
        assert!(first_selection.is_some());
        
        // Reset RNG and verify same result
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        let second_selection = table.select(&mut rng2);
        assert_eq!(first_selection, second_selection);
    }

    #[test]
    fn test_weighted_table_empty() {
        let table: WeightedTable<String> = WeightedTable::new(vec![]);
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        assert!(table.select(&mut rng).is_none());
        assert!(table.is_empty());
        assert_eq!(table.total_weight(), 0.0);
    }

    #[test]
    fn test_generation_pipeline_dependency_resolution() {
        let config = GenerationConfig {
            passes: vec![
                GenerationPass {
                    id: "terrain".to_string(),
                    pass_type: PassType::Terrain,
                    config: serde_json::Value::Null,
                    dependencies: vec![],
                },
                GenerationPass {
                    id: "features".to_string(),
                    pass_type: PassType::Features,
                    config: serde_json::Value::Null,
                    dependencies: vec!["terrain".to_string()],
                },
            ],
        };
        
        let pipeline = GenerationPipeline::new(config);
        let sorted = pipeline.sort_passes_by_dependencies().unwrap();
        
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].id, "terrain");
        assert_eq!(sorted[1].id, "features");
    }

    #[test]
    fn test_generation_pipeline_circular_dependency() {
        let config = GenerationConfig {
            passes: vec![
                GenerationPass {
                    id: "a".to_string(),
                    pass_type: PassType::Terrain,
                    config: serde_json::Value::Null,
                    dependencies: vec!["b".to_string()],
                },
                GenerationPass {
                    id: "b".to_string(),
                    pass_type: PassType::Features,
                    config: serde_json::Value::Null,
                    dependencies: vec!["a".to_string()],
                },
            ],
        };
        
        let pipeline = GenerationPipeline::new(config);
        let result = pipeline.sort_passes_by_dependencies();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[test]
    fn test_template_basic_instantiation() {
        let mut library = TemplateLibrary::new();
        
        let template = ContentTemplate {
            id: "test_room".to_string(),
            category: "room".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("width".to_string(), Value::Number(8.into()));
                params.insert("height".to_string(), Value::Number(6.into()));
                params.insert("description".to_string(), Value::String("A test room".to_string()));
                params
            },
            variants: vec![],
            inheritance: None,
        };
        
        library.add_template(template);
        
        let context = TemplateContext {
            variables: HashMap::new(),
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = library.instantiate("test_room", &context, &mut rng).unwrap();
        
        assert_eq!(result.get("width").unwrap().as_u64().unwrap(), 8);
        assert_eq!(result.get("height").unwrap().as_u64().unwrap(), 6);
        assert_eq!(result.get("description").unwrap().as_str().unwrap(), "A test room");
    }

    #[test]
    fn test_template_inheritance() {
        let mut library = TemplateLibrary::new();
        
        // Parent template
        let parent = ContentTemplate {
            id: "base_room".to_string(),
            category: "room".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("width".to_string(), Value::Number(8.into()));
                params.insert("height".to_string(), Value::Number(6.into()));
                params.insert("wall_type".to_string(), Value::String("stone".to_string()));
                params
            },
            variants: vec![],
            inheritance: None,
        };
        
        // Child template
        let child = ContentTemplate {
            id: "glass_room".to_string(),
            category: "room".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("wall_type".to_string(), Value::String("glass".to_string()));
                params.insert("special".to_string(), Value::Bool(true));
                params
            },
            variants: vec![],
            inheritance: Some("base_room".to_string()),
        };
        
        library.add_template(parent);
        library.add_template(child);
        
        let context = TemplateContext {
            variables: HashMap::new(),
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = library.instantiate("glass_room", &context, &mut rng).unwrap();
        
        // Should inherit width/height from parent
        assert_eq!(result.get("width").unwrap().as_u64().unwrap(), 8);
        assert_eq!(result.get("height").unwrap().as_u64().unwrap(), 6);
        
        // Should override wall_type from parent
        assert_eq!(result.get("wall_type").unwrap().as_str().unwrap(), "glass");
        
        // Should have child-specific property
        assert_eq!(result.get("special").unwrap().as_bool().unwrap(), true);
    }

    #[test]
    fn test_template_variable_substitution() {
        let mut library = TemplateLibrary::new();
        
        let template = ContentTemplate {
            id: "variable_room".to_string(),
            category: "room".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("description".to_string(), Value::String("A ${material} room with ${count} items".to_string()));
                params
            },
            variants: vec![],
            inheritance: None,
        };
        
        library.add_template(template);
        
        let context = TemplateContext {
            variables: {
                let mut vars = HashMap::new();
                vars.insert("material".to_string(), Value::String("crystal".to_string()));
                vars.insert("count".to_string(), Value::String("3".to_string()));
                vars
            },
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = library.instantiate("variable_room", &context, &mut rng).unwrap();
        
        assert_eq!(result.get("description").unwrap().as_str().unwrap(), "A crystal room with 3 items");
    }

    #[test]
    fn test_template_variant_selection() {
        let mut library = TemplateLibrary::new();
        
        let template = ContentTemplate {
            id: "variant_room".to_string(),
            category: "room".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("width".to_string(), Value::Number(8.into()));
                params
            },
            variants: vec![
                TemplateVariant {
                    id: "small".to_string(),
                    weight: 50.0,
                    conditions: vec!["size=small".to_string()],
                    overrides: {
                        let mut overrides = HashMap::new();
                        overrides.insert("width".to_string(), Value::Number(4.into()));
                        overrides
                    },
                },
                TemplateVariant {
                    id: "large".to_string(),
                    weight: 50.0,
                    conditions: vec!["size=large".to_string()],
                    overrides: {
                        let mut overrides = HashMap::new();
                        overrides.insert("width".to_string(), Value::Number(12.into()));
                        overrides
                    },
                },
            ],
            inheritance: None,
        };
        
        library.add_template(template);
        
        let context = TemplateContext {
            variables: {
                let mut vars = HashMap::new();
                vars.insert("size".to_string(), Value::String("small".to_string()));
                vars
            },
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = library.instantiate("variant_room", &context, &mut rng).unwrap();
        
        // Should use small variant override
        assert_eq!(result.get("width").unwrap().as_u64().unwrap(), 4);
    }
}
