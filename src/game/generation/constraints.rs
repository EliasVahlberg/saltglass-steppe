use rand_chacha::ChaCha8Rng;
use serde::{Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};
use once_cell::sync::Lazy;

use crate::game::map::{Map, Tile};
use crate::game::world_map::Biome;

/// Constraint validation system for procedural generation
#[derive(Debug, Clone, Deserialize)]
pub struct ConstraintRule {
    pub id: String,
    pub name: String,
    pub constraint_type: ConstraintType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub severity: ConstraintSeverity,
    pub enabled: bool,
}

/// Types of constraints that can be validated
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    Connectivity,
    Distance,
    Accessibility,
    Balance,
    Placement,
    Resource,
}

/// Severity levels for constraint violations
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintSeverity {
    Critical,  // Must be satisfied
    Warning,   // Should be satisfied
    Suggestion, // Nice to have
}

/// Result of constraint validation
#[derive(Debug, Clone)]
pub struct ConstraintResult {
    pub rule_id: String,
    pub passed: bool,
    pub severity: ConstraintSeverity,
    pub message: String,
    pub score: f32, // 0.0 = failed, 1.0 = perfect
}

/// Context for constraint validation
pub struct ConstraintContext<'a> {
    pub map: &'a Map,
    pub biome: Biome,
    pub entities: Vec<EntityPlacement>,
    pub resources: Vec<ResourcePlacement>,
    pub objectives: Vec<ObjectivePlacement>,
}

/// Entity placement information for constraint checking
#[derive(Debug, Clone)]
pub struct EntityPlacement {
    pub entity_type: String,
    pub x: i32,
    pub y: i32,
    pub properties: HashMap<String, String>,
}

/// Resource placement information
#[derive(Debug, Clone)]
pub struct ResourcePlacement {
    pub resource_type: String,
    pub x: i32,
    pub y: i32,
    pub amount: u32,
}

/// Objective placement information
#[derive(Debug, Clone)]
pub struct ObjectivePlacement {
    pub objective_type: String,
    pub x: i32,
    pub y: i32,
    pub required: bool,
}

/// Constraint validation system
pub struct ConstraintSystem;

impl ConstraintSystem {
    /// Validate all constraints for a given context
    pub fn validate_constraints(
        context: &ConstraintContext,
        rng: &mut ChaCha8Rng,
    ) -> Vec<ConstraintResult> {
        let rules = Self::get_constraint_rules();
        let mut results = Vec::new();
        
        for rule in rules.iter().filter(|r| r.enabled) {
            let result = Self::validate_constraint(rule, context, rng);
            results.push(result);
        }
        
        results
    }
    
    /// Validate a single constraint rule
    pub fn validate_constraint(
        rule: &ConstraintRule,
        context: &ConstraintContext,
        _rng: &mut ChaCha8Rng,
    ) -> ConstraintResult {
        match rule.constraint_type {
            ConstraintType::Connectivity => Self::validate_connectivity(rule, context),
            ConstraintType::Distance => Self::validate_distance(rule, context),
            ConstraintType::Accessibility => Self::validate_accessibility(rule, context),
            ConstraintType::Balance => Self::validate_balance(rule, context),
            ConstraintType::Placement => Self::validate_placement(rule, context),
            ConstraintType::Resource => Self::validate_resource(rule, context),
        }
    }
    
    /// Check if critical constraints are satisfied
    pub fn are_critical_constraints_satisfied(results: &[ConstraintResult]) -> bool {
        results.iter()
            .filter(|r| r.severity == ConstraintSeverity::Critical)
            .all(|r| r.passed)
    }
    
    /// Calculate overall constraint satisfaction score
    pub fn calculate_satisfaction_score(results: &[ConstraintResult]) -> f32 {
        if results.is_empty() {
            return 1.0;
        }
        
        let total_score: f32 = results.iter().map(|r| r.score).sum();
        total_score / results.len() as f32
    }
    
    /// Validate connectivity constraints
    fn validate_connectivity(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let start_points = Self::extract_positions(&rule.parameters, "start_points");
        let end_points = Self::extract_positions(&rule.parameters, "end_points");
        
        if start_points.is_empty() || end_points.is_empty() {
            return ConstraintResult {
                rule_id: rule.id.clone(),
                passed: true,
                severity: rule.severity.clone(),
                message: "No connectivity points to validate".to_string(),
                score: 1.0,
            };
        }
        
        let mut connected_count = 0;
        let total_pairs = start_points.len() * end_points.len();
        
        for start in &start_points {
            for end in &end_points {
                if Self::is_path_exists(&context.map, *start, *end) {
                    connected_count += 1;
                }
            }
        }
        
        let score = connected_count as f32 / total_pairs as f32;
        let passed = score >= 0.8; // 80% connectivity required
        
        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!("Connectivity: {}/{} paths connected", connected_count, total_pairs),
            score,
        }
    }
    
    /// Validate distance constraints
    fn validate_distance(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let min_distance = Self::extract_f32(&rule.parameters, "min_distance").unwrap_or(5.0);
        let max_distance = Self::extract_f32(&rule.parameters, "max_distance").unwrap_or(50.0);
        let entity_type = Self::extract_string(&rule.parameters, "entity_type");
        
        let entities: Vec<_> = context.entities.iter()
            .filter(|e| entity_type.as_ref().map_or(true, |et| &e.entity_type == et))
            .collect();
        
        if entities.len() < 2 {
            return ConstraintResult {
                rule_id: rule.id.clone(),
                passed: true,
                severity: rule.severity.clone(),
                message: "Insufficient entities for distance validation".to_string(),
                score: 1.0,
            };
        }
        
        let mut valid_distances = 0;
        let mut total_distances = 0;
        
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let distance = Self::calculate_distance(
                    (entities[i].x, entities[i].y),
                    (entities[j].x, entities[j].y)
                );
                
                total_distances += 1;
                if distance >= min_distance && distance <= max_distance {
                    valid_distances += 1;
                }
            }
        }
        
        let score = valid_distances as f32 / total_distances as f32;
        let passed = score >= 0.7; // 70% of distances should be valid
        
        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!("Distance: {}/{} pairs within range", valid_distances, total_distances),
            score,
        }
    }
    
    /// Validate accessibility constraints
    fn validate_accessibility(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let required_objectives: Vec<_> = context.objectives.iter()
            .filter(|o| o.required)
            .collect();
        
        if required_objectives.is_empty() {
            return ConstraintResult {
                rule_id: rule.id.clone(),
                passed: true,
                severity: rule.severity.clone(),
                message: "No required objectives to validate".to_string(),
                score: 1.0,
            };
        }
        
        // Check if all required objectives are accessible from spawn point
        let spawn_point = (context.map.width as i32 / 2, context.map.height as i32 / 2); // Assume center spawn
        let mut accessible_count = 0;
        
        for objective in &required_objectives {
            if Self::is_path_exists(context.map, spawn_point, (objective.x, objective.y)) {
                accessible_count += 1;
            }
        }
        
        let score = accessible_count as f32 / required_objectives.len() as f32;
        let passed = score == 1.0; // All required objectives must be accessible
        
        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!("Accessibility: {}/{} objectives reachable", accessible_count, required_objectives.len()),
            score,
        }
    }
    
    /// Validate balance constraints
    fn validate_balance(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let resource_type = Self::extract_string(&rule.parameters, "resource_type");
        let min_amount = Self::extract_u32(&rule.parameters, "min_amount").unwrap_or(1);
        let max_amount = Self::extract_u32(&rule.parameters, "max_amount").unwrap_or(100);
        
        let total_resources: u32 = if resource_type.as_ref().map_or(false, |rt| rt == "open_space") {
            // Count walkable tiles (Floor and Glass)
            context.map.tiles.iter()
                .filter(|tile| matches!(tile, Tile::Floor { .. } | Tile::Glass))
                .count() as u32
        } else {
            context.resources.iter()
                .filter(|r| resource_type.as_ref().map_or(true, |rt| &r.resource_type == rt))
                .map(|r| r.amount)
                .sum()
        };
        
        let passed = total_resources >= min_amount && total_resources <= max_amount;
        let score = if passed { 
            1.0 
        } else if total_resources < min_amount {
            total_resources as f32 / min_amount as f32
        } else {
            max_amount as f32 / total_resources as f32
        };
        
        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!("Balance: {} resources (range: {}-{})", total_resources, min_amount, max_amount),
            score,
        }
    }
    
    /// Validate placement constraints
    fn validate_placement(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let entity_type = Self::extract_string(&rule.parameters, "entity_type");
        let forbidden_biomes = Self::extract_string_array(&rule.parameters, "forbidden_biomes");
        
        let entities: Vec<_> = context.entities.iter()
            .filter(|e| entity_type.as_ref().map_or(true, |et| &e.entity_type == et))
            .collect();
        
        let biome_name = format!("{:?}", context.biome).to_lowercase();
        let is_forbidden = forbidden_biomes.contains(&biome_name);
        
        let passed = entities.is_empty() || !is_forbidden;
        let score = if passed { 1.0 } else { 0.0 };
        
        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!("Placement: {} entities in {} biome", entities.len(), biome_name),
            score,
        }
    }
    
    /// Validate resource constraints
    fn validate_resource(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let resource_density = Self::extract_f32(&rule.parameters, "max_density").unwrap_or(0.1);
        let map_area = (context.map.width * context.map.height) as f32;
        let max_resources = (map_area * resource_density) as usize;
        
        let passed = context.resources.len() <= max_resources;
        let score = if passed { 1.0 } else { max_resources as f32 / context.resources.len() as f32 };
        
        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!("Resource density: {}/{} resources", context.resources.len(), max_resources),
            score,
        }
    }
    
    /// Check if a path exists between two points using BFS
    fn is_path_exists(map: &Map, start: (i32, i32), end: (i32, i32)) -> bool {
        if start == end {
            return true;
        }
        
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);
        
        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == end {
                return true;
            }
            
            // Check 4-directional neighbors
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;
                
                if nx >= 0 && nx < map.width as i32 && ny >= 0 && ny < map.height as i32 {
                    let pos = (nx, ny);
                    if !visited.contains(&pos) && Self::is_tile_walkable(map, nx, ny) {
                        visited.insert(pos);
                        queue.push_back(pos);
                    }
                }
            }
        }
        
        false
    }
    
    /// Calculate Euclidean distance between two points
    fn calculate_distance(p1: (i32, i32), p2: (i32, i32)) -> f32 {
        let dx = (p1.0 - p2.0) as f32;
        let dy = (p1.1 - p2.1) as f32;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Check if a tile is walkable (simplified check)
    fn is_tile_walkable(map: &Map, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
            return false;
        }
        
        if let Some(idx) = map.pos_to_idx(x, y) {
            matches!(map.tiles[idx], Tile::Floor { .. } | Tile::Glass)
        } else {
            false
        }
    }
    
    /// Get constraint rules from static data
    fn get_constraint_rules() -> &'static Vec<ConstraintRule> {
        &CONSTRAINT_RULES
    }
    
    // Helper methods for parameter extraction
    fn extract_positions(params: &HashMap<String, serde_json::Value>, key: &str) -> Vec<(i32, i32)> {
        params.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        if let Some(obj) = item.as_object() {
                            let x = obj.get("x")?.as_i64()? as i32;
                            let y = obj.get("y")?.as_i64()? as i32;
                            Some((x, y))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    
    fn extract_f32(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<f32> {
        params.get(key)?.as_f64().map(|v| v as f32)
    }
    
    fn extract_u32(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<u32> {
        params.get(key)?.as_u64().map(|v| v as u32)
    }
    
    fn extract_string(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
        params.get(key)?.as_str().map(|s| s.to_string())
    }
    
    fn extract_string_array(params: &HashMap<String, serde_json::Value>, key: &str) -> Vec<String> {
        params.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

// Static constraint rules data
static CONSTRAINT_RULES: Lazy<Vec<ConstraintRule>> = Lazy::new(|| {
    let data = include_str!("../../../data/constraint_rules.json");
    let file: ConstraintRulesFile = serde_json::from_str(data)
        .expect("Failed to parse constraint_rules.json");
    file.rules
});

#[derive(Deserialize)]
struct ConstraintRulesFile {
    rules: Vec<ConstraintRule>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_constraint_system_deterministic() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
        
        let context = create_test_context();
        
        let results1 = ConstraintSystem::validate_constraints(&context, &mut rng1);
        let results2 = ConstraintSystem::validate_constraints(&context, &mut rng2);
        
        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.rule_id, r2.rule_id);
            assert_eq!(r1.passed, r2.passed);
            assert_eq!((r1.score * 1000.0) as i32, (r2.score * 1000.0) as i32); // Float comparison
        }
    }
    
    #[test]
    fn test_connectivity_validation() {
        use std::sync::OnceLock;
        static TEST_MAP: OnceLock<Map> = OnceLock::new();
        
        let map = TEST_MAP.get_or_init(|| {
            let (map, _) = Map::generate(&mut ChaCha8Rng::seed_from_u64(12345));
            map
        });
        
        let context = ConstraintContext {
            map,
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
        
        let result = ConstraintSystem::validate_connectivity(&rule, &context);
        assert_eq!(result.rule_id, "test_connectivity");
        assert!(result.passed); // Empty points should pass
    }
    
    #[test]
    fn test_distance_validation() {
        let context = create_test_context();
        
        let mut params = HashMap::new();
        params.insert("min_distance".to_string(), serde_json::json!(5.0));
        params.insert("max_distance".to_string(), serde_json::json!(20.0));
        
        let rule = ConstraintRule {
            id: "test_distance".to_string(),
            name: "Test Distance".to_string(),
            constraint_type: ConstraintType::Distance,
            parameters: params,
            severity: ConstraintSeverity::Warning,
            enabled: true,
        };
        
        let result = ConstraintSystem::validate_distance(&rule, &context);
        assert_eq!(result.rule_id, "test_distance");
        assert!(result.score >= 0.0 && result.score <= 1.0);
    }
    
    #[test]
    fn test_accessibility_validation() {
        let context = create_test_context();
        
        let rule = ConstraintRule {
            id: "test_accessibility".to_string(),
            name: "Test Accessibility".to_string(),
            constraint_type: ConstraintType::Accessibility,
            parameters: HashMap::new(),
            severity: ConstraintSeverity::Critical,
            enabled: true,
        };
        
        let result = ConstraintSystem::validate_accessibility(&rule, &context);
        assert_eq!(result.rule_id, "test_accessibility");
        assert!(result.score >= 0.0 && result.score <= 1.0);
    }
    
    #[test]
    fn test_critical_constraints_check() {
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
    fn test_satisfaction_score_calculation() {
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
    
    fn create_test_context() -> ConstraintContext<'static> {
        use std::sync::OnceLock;
        static TEST_MAP: OnceLock<Map> = OnceLock::new();
        
        let map = TEST_MAP.get_or_init(|| {
            let (map, _) = Map::generate(&mut ChaCha8Rng::seed_from_u64(12345));
            map
        });
        
        ConstraintContext {
            map,
            biome: Biome::Desert,
            entities: vec![
                EntityPlacement {
                    entity_type: "enemy".to_string(),
                    x: 5,
                    y: 5,
                    properties: HashMap::new(),
                },
                EntityPlacement {
                    entity_type: "enemy".to_string(),
                    x: 15,
                    y: 15,
                    properties: HashMap::new(),
                },
            ],
            resources: vec![
                ResourcePlacement {
                    resource_type: "glass".to_string(),
                    x: 10,
                    y: 10,
                    amount: 50,
                },
            ],
            objectives: vec![
                ObjectivePlacement {
                    objective_type: "exit".to_string(),
                    x: 18,
                    y: 18,
                    required: true,
                },
            ],
        }
    }
}
