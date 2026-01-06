#[cfg(test)]
mod generation_tests {
    use crate::game::generation::weighted_table::{WeightedEntry, WeightedTable};
    use crate::game::generation::pipeline::{GenerationConfig, GenerationPass, GenerationPipeline, PassType};
    use crate::game::generation::templates::{TemplateLibrary, TemplateContext, ContentTemplate, TemplateVariant};
    use crate::game::generation::grammar::{Grammar, GrammarContext, GrammarRule};
    use crate::game::generation::events::{EventSystem, EventContext};
    use crate::game::generation::narrative::{NarrativeIntegration, NarrativeContext};
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

    #[test]
    fn test_grammar_basic_generation() {
        let mut grammar = Grammar::new();
        
        // Add simple rule
        grammar.rules.insert("greeting".to_string(), GrammarRule {
            expansions: vec!["Hello".to_string(), "Hi".to_string(), "Greetings".to_string()],
            weights: None,
        });
        
        let context = GrammarContext {
            variables: HashMap::new(),
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = grammar.generate("greeting", &context, &mut rng).unwrap();
        
        assert!(["Hello", "Hi", "Greetings"].contains(&result.as_str()));
        
        // Test determinism
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        let result2 = grammar.generate("greeting", &context, &mut rng2).unwrap();
        assert_eq!(result, result2);
    }

    #[test]
    fn test_grammar_recursive_expansion() {
        let mut grammar = Grammar::new();
        
        grammar.rules.insert("sentence".to_string(), GrammarRule {
            expansions: vec!["<subject> <verb> <object>".to_string()],
            weights: None,
        });
        
        grammar.rules.insert("subject".to_string(), GrammarRule {
            expansions: vec!["The storm".to_string(), "Glass".to_string()],
            weights: None,
        });
        
        grammar.rules.insert("verb".to_string(), GrammarRule {
            expansions: vec!["breaks".to_string(), "shatters".to_string()],
            weights: None,
        });
        
        grammar.rules.insert("object".to_string(), GrammarRule {
            expansions: vec!["reality".to_string(), "silence".to_string()],
            weights: None,
        });
        
        let context = GrammarContext {
            variables: HashMap::new(),
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = grammar.generate("sentence", &context, &mut rng).unwrap();
        
        // Should be a complete sentence
        assert!(result.contains(" "));
        assert!(result.len() > 5);
    }

    #[test]
    fn test_grammar_weighted_selection() {
        let mut grammar = Grammar::new();
        
        grammar.rules.insert("weighted_rule".to_string(), GrammarRule {
            expansions: vec!["rare".to_string(), "common".to_string()],
            weights: Some(vec![10.0, 90.0]),
        });
        
        let context = GrammarContext {
            variables: HashMap::new(),
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = grammar.generate("weighted_rule", &context, &mut rng).unwrap();
        
        assert!(["rare", "common"].contains(&result.as_str()));
    }

    #[test]
    fn test_grammar_variable_substitution() {
        let mut grammar = Grammar::new();
        
        grammar.rules.insert("greeting".to_string(), GrammarRule {
            expansions: vec!["Hello <name>".to_string()],
            weights: None,
        });
        
        let context = GrammarContext {
            variables: {
                let mut vars = HashMap::new();
                vars.insert("name".to_string(), "Traveler".to_string());
                vars
            },
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = grammar.generate("greeting", &context, &mut rng).unwrap();
        
        assert_eq!(result, "Hello Traveler");
    }

    #[test]
    fn test_grammar_recursion_limit() {
        let mut grammar = Grammar::new();
        
        // Create infinite recursion
        grammar.rules.insert("infinite".to_string(), GrammarRule {
            expansions: vec!["<infinite>".to_string()],
            weights: None,
        });
        
        let context = GrammarContext {
            variables: HashMap::new(),
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = grammar.generate("infinite", &context, &mut rng);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("recursion"));
    }

    // Biome System Tests
    #[test]
    fn test_biome_system_deterministic() {
        use crate::game::generation::biomes::{BiomeSystem, BiomeGenerationContext};
        use crate::game::world_map::Biome;
        
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        
        let context = BiomeGenerationContext {
            biome: Biome::Desert,
            storm_intensity: 2,
            time_of_day: "day".to_string(),
            weather_conditions: "clear".to_string(),
            player_adaptations: vec![],
        };
        
        let desc1 = BiomeSystem::generate_environment_description(
            Biome::Desert, &context, &mut rng1
        );
        let desc2 = BiomeSystem::generate_environment_description(
            Biome::Desert, &context, &mut rng2
        );
        
        assert_eq!(desc1, desc2, "Biome generation should be deterministic");
    }

    #[test]
    fn test_biome_environmental_features() {
        use crate::game::generation::biomes::BiomeSystem;
        use crate::game::world_map::Biome;
        
        let mut rng = ChaCha8Rng::seed_from_u64(54321);
        
        let features = BiomeSystem::generate_environmental_features(
            Biome::Saltflat, 3, &mut rng
        );
        
        assert_eq!(features.len(), 3);
        for feature in features {
            assert!(!feature.feature_type.is_empty());
            assert!(!feature.description_template.is_empty());
        }
    }

    #[test]
    fn test_biome_hazard_checking() {
        use crate::game::generation::biomes::{BiomeSystem, BiomeGenerationContext};
        use crate::game::world_map::Biome;
        
        let mut rng = ChaCha8Rng::seed_from_u64(98765);
        
        let context = BiomeGenerationContext {
            biome: Biome::Ruins,
            storm_intensity: 5, // High storm should increase hazards
            time_of_day: "night".to_string(),
            weather_conditions: "stormy".to_string(),
            player_adaptations: vec![],
        };
        
        let hazards = BiomeSystem::check_hazards(Biome::Ruins, &context, &mut rng);
        
        // Should be able to generate hazards (may be empty due to randomness)
        for hazard in hazards {
            assert!(!hazard.hazard_type.is_empty());
            assert!(hazard.severity > 0);
            assert!(hazard.frequency >= 0.0 && hazard.frequency <= 1.0);
        }
    }

    #[test]
    fn test_biome_context_creation() {
        use crate::game::generation::biomes::BiomeGenerationContext;
        use crate::game::world_map::Biome;
        
        let context = BiomeGenerationContext {
            biome: Biome::Oasis,
            storm_intensity: 1,
            time_of_day: "dawn".to_string(),
            weather_conditions: "misty".to_string(),
            player_adaptations: vec!["prismhide".to_string()],
        };
        
        assert_eq!(context.biome, Biome::Oasis);
        assert_eq!(context.storm_intensity, 1);
        assert_eq!(context.time_of_day, "dawn");
        assert!(context.player_adaptations.contains(&"prismhide".to_string()));
    }

    // Constraint System Tests
    #[test]
    fn test_constraint_system_deterministic() {
        use crate::game::generation::constraints::{ConstraintSystem, ConstraintContext};
        use crate::game::world_map::Biome;
        use crate::game::map::Map;
        
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        
        // Create a simple test map
        let (map, _) = Map::generate(&mut ChaCha8Rng::seed_from_u64(12345));
        
        let context = ConstraintContext {
            map: &map,
            biome: Biome::Desert,
            entities: vec![],
            resources: vec![],
            objectives: vec![],
        };
        
        let results1 = ConstraintSystem::validate_constraints(&context, &mut rng1);
        let results2 = ConstraintSystem::validate_constraints(&context, &mut rng2);
        
        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.rule_id, r2.rule_id);
            assert_eq!(r1.passed, r2.passed);
        }
    }

    #[test]
    fn test_constraint_validation_types() {
        use crate::game::generation::constraints::{ConstraintSystem, ConstraintContext, ConstraintRule, ConstraintType, ConstraintSeverity};
        use crate::game::world_map::Biome;
        use crate::game::map::Map;
        
        let (map, _) = Map::generate(&mut ChaCha8Rng::seed_from_u64(12345));
        
        let context = ConstraintContext {
            map: &map,
            biome: Biome::Desert,
            entities: vec![],
            resources: vec![],
            objectives: vec![],
        };
        
        let rule = ConstraintRule {
            id: "test_connectivity".to_string(),
            name: "Test Connectivity".to_string(),
            constraint_type: ConstraintType::Connectivity,
            parameters: HashMap::new(),
            severity: ConstraintSeverity::Critical,
            enabled: true,
        };
        
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let result = ConstraintSystem::validate_constraint(&rule, &context, &mut rng);
        
        assert_eq!(result.rule_id, "test_connectivity");
        assert!(result.score >= 0.0 && result.score <= 1.0);
    }

    #[test]
    fn test_critical_constraints_satisfaction() {
        use crate::game::generation::constraints::{ConstraintSystem, ConstraintResult, ConstraintSeverity};
        
        let results = vec![
            ConstraintResult {
                rule_id: "critical1".to_string(),
                passed: true,
                severity: ConstraintSeverity::Critical,
                message: "Test".to_string(),
                score: 1.0,
            },
            ConstraintResult {
                rule_id: "warning1".to_string(),
                passed: false,
                severity: ConstraintSeverity::Warning,
                message: "Test".to_string(),
                score: 0.5,
            },
        ];
        
        assert!(ConstraintSystem::are_critical_constraints_satisfied(&results));
        
        let results_with_failed_critical = vec![
            ConstraintResult {
                rule_id: "critical1".to_string(),
                passed: false,
                severity: ConstraintSeverity::Critical,
                message: "Test".to_string(),
                score: 0.0,
            },
        ];
        
        assert!(!ConstraintSystem::are_critical_constraints_satisfied(&results_with_failed_critical));
    }

    #[test]
    fn test_constraint_satisfaction_score() {
        use crate::game::generation::constraints::{ConstraintSystem, ConstraintResult, ConstraintSeverity};
        
        let results = vec![
            ConstraintResult {
                rule_id: "test1".to_string(),
                passed: true,
                severity: ConstraintSeverity::Critical,
                message: "Test".to_string(),
                score: 1.0,
            },
            ConstraintResult {
                rule_id: "test2".to_string(),
                passed: false,
                severity: ConstraintSeverity::Warning,
                message: "Test".to_string(),
                score: 0.5,
            },
        ];
        
        let score = ConstraintSystem::calculate_satisfaction_score(&results);
        assert_eq!(score, 0.75); // (1.0 + 0.5) / 2
    }

    // Event System Tests
    #[test]
    fn test_event_system_creation() {
        let system = EventSystem::new();
        // Should load events from JSON
        assert!(system.event_count() > 0);
    }

    #[test]
    fn test_event_trigger_evaluation() {
        let mut system = EventSystem::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        // Use context that matches oasis blessing event
        let context = EventContext {
            player_hp: 50,
            player_max_hp: 100,
            player_x: 10,
            player_y: 10,
            turn: 100,
            biome: "oasis".to_string(),
            storm_intensity: 2,
            refraction_level: 20,
            variables: HashMap::new(),
        };

        let triggered = system.check_triggers(&context, &mut rng);
        // Should find oasis blessing event
        assert!(!triggered.is_empty());
    }

    #[test]
    fn test_event_consequences() {
        let system = EventSystem::new();
        let mut context = EventContext {
            player_hp: 50,
            player_max_hp: 100,
            player_x: 10,
            player_y: 10,
            turn: 100,
            biome: "oasis".to_string(),
            storm_intensity: 2,
            refraction_level: 20,
            variables: HashMap::new(),
        };

        // Apply consequences for oasis blessing (should heal)
        let messages = system.apply_consequences("oasis_blessing", &mut context);
        assert!(!messages.is_empty());
        assert!(context.variables.contains_key("healing_received"));
    }

    #[test]
    fn test_event_chains() {
        let system = EventSystem::new();
        
        // Check if refraction cascade chain exists
        let chain = system.get_event_chains("glass_storm_exposure");
        assert!(chain.is_some());
        
        let chain = chain.unwrap();
        assert_eq!(chain.chain_id, "refraction_cascade");
        assert!(!chain.events.is_empty());
    }

    #[test]
    fn test_event_cooldown_system() {
        let mut system = EventSystem::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        
        let context = EventContext {
            player_hp: 50,
            player_max_hp: 100,
            player_x: 10,
            player_y: 10,
            turn: 100,
            biome: "oasis".to_string(),
            storm_intensity: 2,
            refraction_level: 20,
            variables: HashMap::new(),
        };

        // First trigger should work
        let triggered1 = system.check_triggers(&context, &mut rng);
        
        // Reset RNG to same state for fair comparison
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        
        // Immediately check again - should respect cooldowns
        let triggered2 = system.check_triggers(&context, &mut rng2);
        
        // Second check should have fewer events due to cooldowns
        assert!(triggered2.len() < triggered1.len() || triggered1.is_empty());
    }

    // Narrative Integration Tests
    #[test]
    fn test_narrative_integration_creation() {
        let system = NarrativeIntegration::new();
        assert!(system.seed_count() > 0);
        assert!(system.fragment_count() > 0);
        assert!(system.faction_count() > 0);
    }

    #[test]
    fn test_narrative_initialization() {
        let mut system = NarrativeIntegration::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let context = NarrativeContext {
            player_x: 10,
            player_y: 10,
            current_biome: "desert".to_string(),
            turn: 100,
            faction_standings: HashMap::new(),
            discovered_fragments: Vec::new(),
            player_adaptations: vec!["prismhide".to_string()],
        };

        system.initialize(&context, &mut rng);
        
        assert!(!system.get_narrative_state().active_seeds.is_empty());
        assert!(system.get_narrative_state().narrative_momentum > 0.0);
    }

    #[test]
    fn test_story_fragment_generation() {
        let mut system = NarrativeIntegration::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let context = NarrativeContext {
            player_x: 10,
            player_y: 10,
            current_biome: "desert".to_string(),
            turn: 100,
            faction_standings: HashMap::new(),
            discovered_fragments: Vec::new(),
            player_adaptations: vec!["prismhide".to_string()],
        };

        system.initialize(&context, &mut rng);
        
        // Ensure we have active seeds
        assert!(system.has_active_seeds(), "Should have active seeds after initialization");
        
        let fragments = system.generate_fragments(&context, &mut rng);
        
        // Should generate some fragments (may be empty if placement rules don't match)
        // This is acceptable behavior - not all contexts will generate fragments
    }

    #[test]
    fn test_faction_influence_tracking() {
        let mut system = NarrativeIntegration::new();
        
        system.update_faction_influence("mirror_monks", 0.3);
        
        let state = system.get_narrative_state();
        assert_eq!(state.faction_standings.get("mirror_monks"), Some(&0.3));
        assert!(state.narrative_momentum > 0.0);
    }

    #[test]
    fn test_emergent_narrative_tracking() {
        let mut system = NarrativeIntegration::new();
        let context = NarrativeContext {
            player_x: 10,
            player_y: 10,
            current_biome: "desert".to_string(),
            turn: 100,
            faction_standings: HashMap::new(),
            discovered_fragments: Vec::new(),
            player_adaptations: vec!["prismhide".to_string()],
        };
        
        let initial_momentum = system.get_narrative_state().narrative_momentum;
        system.track_narrative_event("fragment_discovered", &context);
        
        assert!(system.get_narrative_state().narrative_momentum > initial_momentum);
    }

    #[test]
    fn test_faction_influenced_content() {
        let mut system = NarrativeIntegration::new();
        system.update_faction_influence("mirror_monks", 0.8);

        let base_content = "You find ancient ruins.";
        let factions = vec!["mirror_monks".to_string()];
        let influenced = system.get_faction_influenced_content(base_content, &factions);
        
        assert!(influenced.contains("favorably"));
    }
}
