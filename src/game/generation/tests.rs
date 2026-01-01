#[cfg(test)]
mod generation_tests {
    use crate::game::generation::weighted_table::{WeightedEntry, WeightedTable};
    use crate::game::generation::pipeline::{GenerationConfig, GenerationPass, GenerationPipeline, PassType};
    use rand_chacha::ChaCha8Rng;
    use rand::SeedableRng;

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
}
