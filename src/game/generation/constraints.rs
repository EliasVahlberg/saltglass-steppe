use once_cell::sync::Lazy;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::collections::HashMap;

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
    Tactical,
    SafeZone,
    EscapeRoute,
    ObjectiveAccessibility,
}

/// Severity levels for constraint violations
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintSeverity {
    Critical,   // Must be satisfied
    Warning,    // Should be satisfied
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
            ConstraintType::Tactical => Self::validate_tactical(rule, context),
            ConstraintType::SafeZone => Self::validate_safe_zone(rule, context),
            ConstraintType::EscapeRoute => Self::validate_escape_route(rule, context),
            ConstraintType::ObjectiveAccessibility => {
                Self::validate_objective_accessibility(rule, context)
            }
        }
    }

    /// Check if critical constraints are satisfied
    pub fn are_critical_constraints_satisfied(results: &[ConstraintResult]) -> bool {
        results
            .iter()
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
    fn validate_connectivity(
        rule: &ConstraintRule,
        context: &ConstraintContext,
    ) -> ConstraintResult {
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
            message: format!(
                "Connectivity: {}/{} paths connected",
                connected_count, total_pairs
            ),
            score,
        }
    }

    /// Validate distance constraints
    fn validate_distance(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let min_distance = Self::extract_f32(&rule.parameters, "min_distance").unwrap_or(5.0);
        let max_distance = Self::extract_f32(&rule.parameters, "max_distance").unwrap_or(50.0);
        let entity_type = Self::extract_string(&rule.parameters, "entity_type");

        let entities: Vec<_> = context
            .entities
            .iter()
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
                    (entities[j].x, entities[j].y),
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
            message: format!(
                "Distance: {}/{} pairs within range",
                valid_distances, total_distances
            ),
            score,
        }
    }

    /// Validate accessibility constraints
    fn validate_accessibility(
        rule: &ConstraintRule,
        context: &ConstraintContext,
    ) -> ConstraintResult {
        let required_objectives: Vec<_> =
            context.objectives.iter().filter(|o| o.required).collect();

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
            message: format!(
                "Accessibility: {}/{} objectives reachable",
                accessible_count,
                required_objectives.len()
            ),
            score,
        }
    }

    /// Validate balance constraints
    fn validate_balance(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let resource_type = Self::extract_string(&rule.parameters, "resource_type");
        let min_amount = Self::extract_u32(&rule.parameters, "min_amount").unwrap_or(1);
        let max_amount = Self::extract_u32(&rule.parameters, "max_amount").unwrap_or(100);

        let total_resources: u32 = if resource_type
            .as_ref()
            .map_or(false, |rt| rt == "open_space")
        {
            // Count walkable tiles (Floor and Glass)
            context
                .map
                .tiles
                .iter()
                .filter(|tile| matches!(tile, Tile::Floor { .. } | Tile::Glass))
                .count() as u32
        } else {
            context
                .resources
                .iter()
                .filter(|r| {
                    resource_type
                        .as_ref()
                        .map_or(true, |rt| &r.resource_type == rt)
                })
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
            message: format!(
                "Balance: {} resources (range: {}-{})",
                total_resources, min_amount, max_amount
            ),
            score,
        }
    }

    /// Validate placement constraints
    fn validate_placement(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let entity_type = Self::extract_string(&rule.parameters, "entity_type");
        let forbidden_biomes = Self::extract_string_array(&rule.parameters, "forbidden_biomes");

        let entities: Vec<_> = context
            .entities
            .iter()
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
            message: format!(
                "Placement: {} entities in {} biome",
                entities.len(),
                biome_name
            ),
            score,
        }
    }

    /// Validate resource constraints
    fn validate_resource(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let resource_density = Self::extract_f32(&rule.parameters, "max_density").unwrap_or(0.1);
        let map_area = (context.map.width * context.map.height) as f32;
        let max_resources = (map_area * resource_density) as usize;

        let passed = context.resources.len() <= max_resources;
        let score = if passed {
            1.0
        } else {
            max_resources as f32 / context.resources.len() as f32
        };

        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!(
                "Resource density: {}/{} resources",
                context.resources.len(),
                max_resources
            ),
            score,
        }
    }

    /// Check if a path exists between two points using BFS
    fn is_path_exists(map: &Map, start: (i32, i32), end: (i32, i32)) -> bool {
        if start == end {
            return true;
        }

        // Use bracket-lib's A* pathfinding for consistency
        let start_idx = map.idx(start.0, start.1);
        let end_idx = map.idx(end.0, end.1);
        let path = bracket_pathfinding::prelude::a_star_search(start_idx, end_idx, map);
        path.success
    }

    /// Calculate Euclidean distance between two points
    fn calculate_distance(p1: (i32, i32), p2: (i32, i32)) -> f32 {
        let dx = (p1.0 - p2.0) as f32;
        let dy = (p1.1 - p2.1) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Get constraint rules from static data
    fn get_constraint_rules() -> &'static Vec<ConstraintRule> {
        &CONSTRAINT_RULES
    }

    // Helper methods for parameter extraction
    fn extract_positions(
        params: &HashMap<String, serde_json::Value>,
        key: &str,
    ) -> Vec<(i32, i32)> {
        params
            .get(key)
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
        params
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Validate tactical constraints (chokepoints, open areas)
    fn validate_tactical(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let min_chokepoints = Self::extract_u32(&rule.parameters, "min_chokepoints").unwrap_or(2);
        let max_open_ratio = Self::extract_f32(&rule.parameters, "max_open_ratio").unwrap_or(0.4);

        let mut chokepoints = 0;
        let mut open_tiles = 0;
        let total_floor_tiles = context.map.tiles.iter().filter(|t| t.walkable()).count();

        // Count chokepoints (floor tiles with <= 2 adjacent floor tiles)
        for y in 1..(context.map.height - 1) {
            for x in 1..(context.map.width - 1) {
                let idx = (y * context.map.width + x) as usize;
                if context.map.tiles[idx].walkable() {
                    let adjacent_floors =
                        Self::count_adjacent_floors(context.map, x as i32, y as i32);
                    if adjacent_floors <= 2 {
                        chokepoints += 1;
                    }
                    if adjacent_floors >= 6 {
                        open_tiles += 1;
                    }
                }
            }
        }

        let open_ratio = open_tiles as f32 / total_floor_tiles as f32;
        let chokepoint_score = if chokepoints >= min_chokepoints {
            1.0
        } else {
            chokepoints as f32 / min_chokepoints as f32
        };
        let open_score = if open_ratio <= max_open_ratio {
            1.0
        } else {
            max_open_ratio / open_ratio
        };

        let score = (chokepoint_score + open_score) / 2.0;
        let passed = score >= 0.7;

        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!(
                "Tactical: {} chokepoints, {:.1}% open areas",
                chokepoints,
                open_ratio * 100.0
            ),
            score,
        }
    }

    /// Validate safe zone constraints
    fn validate_safe_zone(rule: &ConstraintRule, context: &ConstraintContext) -> ConstraintResult {
        let min_safe_distance =
            Self::extract_u32(&rule.parameters, "min_safe_distance").unwrap_or(5);
        let required_coverage =
            Self::extract_f32(&rule.parameters, "safe_zone_coverage").unwrap_or(0.3);

        let mut safe_tiles = 0;
        let total_floor_tiles = context.map.tiles.iter().filter(|t| t.walkable()).count();

        // Check each floor tile for safety (distance from enemies/hazards)
        for y in 0..context.map.height {
            for x in 0..context.map.width {
                let idx = (y * context.map.width + x) as usize;
                if context.map.tiles[idx].walkable() {
                    let mut is_safe = true;

                    // Check distance from enemies
                    for entity in &context.entities {
                        if entity.entity_type == "enemy" {
                            let distance = Self::calculate_distance(
                                (x as i32, y as i32),
                                (entity.x, entity.y),
                            );
                            if distance < min_safe_distance as f32 {
                                is_safe = false;
                                break;
                            }
                        }
                    }

                    if is_safe {
                        safe_tiles += 1;
                    }
                }
            }
        }

        let coverage = safe_tiles as f32 / total_floor_tiles as f32;
        let score = (coverage / required_coverage).min(1.0);
        let passed = coverage >= required_coverage;

        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!(
                "Safe zones: {:.1}% coverage ({} safe tiles)",
                coverage * 100.0,
                safe_tiles
            ),
            score,
        }
    }

    /// Validate escape route constraints
    fn validate_escape_route(
        rule: &ConstraintRule,
        context: &ConstraintContext,
    ) -> ConstraintResult {
        let min_exit_routes = Self::extract_u32(&rule.parameters, "min_exit_routes").unwrap_or(2);
        let max_dead_end_depth =
            Self::extract_u32(&rule.parameters, "max_dead_end_depth").unwrap_or(3);

        let mut valid_areas = 0;
        let mut total_areas = 0;
        let mut deep_dead_ends = 0;

        // Sample key positions and check escape routes
        for y in (5..context.map.height).step_by(10) {
            for x in (5..context.map.width).step_by(10) {
                let idx = (y * context.map.width + x) as usize;
                if context.map.tiles[idx].walkable() {
                    total_areas += 1;

                    let exit_routes = Self::count_exit_routes(context.map, x as i32, y as i32);
                    if exit_routes >= min_exit_routes {
                        valid_areas += 1;
                    }

                    let dead_end_depth =
                        Self::calculate_dead_end_depth(context.map, x as i32, y as i32);
                    if dead_end_depth > max_dead_end_depth {
                        deep_dead_ends += 1;
                    }
                }
            }
        }

        let route_score = if total_areas > 0 {
            valid_areas as f32 / total_areas as f32
        } else {
            1.0
        };
        let dead_end_penalty = (deep_dead_ends as f32 / total_areas.max(1) as f32).min(0.5);
        let score = (route_score - dead_end_penalty).max(0.0);
        let passed = score >= 0.7;

        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!(
                "Escape routes: {}/{} areas valid, {} deep dead ends",
                valid_areas, total_areas, deep_dead_ends
            ),
            score,
        }
    }

    /// Validate objective accessibility constraints
    fn validate_objective_accessibility(
        rule: &ConstraintRule,
        context: &ConstraintContext,
    ) -> ConstraintResult {
        let min_path_complexity =
            Self::extract_u32(&rule.parameters, "min_path_complexity").unwrap_or(3);
        let max_path_complexity =
            Self::extract_u32(&rule.parameters, "max_path_complexity").unwrap_or(50);

        let spawn_point = (context.map.width as i32 / 2, context.map.height as i32 / 2);
        let mut accessible_objectives = 0;
        let mut valid_complexity = 0;

        for objective in &context.objectives {
            if objective.required {
                let path_length = Self::calculate_path_length(
                    context.map,
                    spawn_point,
                    (objective.x, objective.y),
                );

                if path_length > 0 {
                    accessible_objectives += 1;

                    if path_length >= min_path_complexity && path_length <= max_path_complexity {
                        valid_complexity += 1;
                    }
                }
            }
        }

        let required_objectives = context.objectives.iter().filter(|o| o.required).count();
        let accessibility_score = if required_objectives > 0 {
            accessible_objectives as f32 / required_objectives as f32
        } else {
            1.0
        };

        let complexity_score = if accessible_objectives > 0 {
            valid_complexity as f32 / accessible_objectives as f32
        } else {
            1.0
        };

        let score = (accessibility_score + complexity_score) / 2.0;
        let passed = accessibility_score >= 0.9 && complexity_score >= 0.7;

        ConstraintResult {
            rule_id: rule.id.clone(),
            passed,
            severity: rule.severity.clone(),
            message: format!(
                "Objectives: {}/{} accessible, {}/{} valid complexity",
                accessible_objectives, required_objectives, valid_complexity, accessible_objectives
            ),
            score,
        }
    }

    /// Helper: Count adjacent floor tiles
    fn count_adjacent_floors(map: &Map, x: i32, y: i32) -> u32 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < map.width as i32 && ny >= 0 && ny < map.height as i32 {
                    let idx = (ny * map.width as i32 + nx) as usize;
                    if map.tiles[idx].walkable() {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Helper: Count exit routes from a position
    fn count_exit_routes(map: &Map, x: i32, y: i32) -> u32 {
        // Simple implementation: count distinct directions with paths
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        let mut routes = 0;

        for (dx, dy) in directions {
            let mut steps = 0;
            let mut cx = x;
            let mut cy = y;

            // Follow direction for up to 10 steps
            while steps < 10 {
                cx += dx;
                cy += dy;
                steps += 1;

                if cx < 0 || cx >= map.width as i32 || cy < 0 || cy >= map.height as i32 {
                    routes += 1; // Reached map edge
                    break;
                }

                let idx = (cy * map.width as i32 + cx) as usize;
                if !map.tiles[idx].walkable() {
                    break; // Hit wall
                }
            }
        }

        routes
    }

    /// Helper: Calculate dead end depth
    fn calculate_dead_end_depth(map: &Map, x: i32, y: i32) -> u32 {
        // Simple flood fill to find depth from nearest multi-path area
        let mut visited = vec![false; map.tiles.len()];
        let mut queue = vec![(x, y, 0)];
        let start_idx = (y * map.width as i32 + x) as usize;
        visited[start_idx] = true;

        while let Some((cx, cy, depth)) = queue.pop() {
            let adjacent_floors = Self::count_adjacent_floors(map, cx, cy);
            if adjacent_floors > 2 {
                return depth; // Found multi-path area
            }

            if depth >= 10 {
                return depth; // Limit search depth
            }

            for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = cx + dx;
                let ny = cy + dy;

                if nx >= 0 && nx < map.width as i32 && ny >= 0 && ny < map.height as i32 {
                    let idx = (ny * map.width as i32 + nx) as usize;
                    if !visited[idx] && map.tiles[idx].walkable() {
                        visited[idx] = true;
                        queue.push((nx, ny, depth + 1));
                    }
                }
            }
        }

        10 // Max depth if no multi-path area found
    }

    /// Helper: Calculate path length between two points
    fn calculate_path_length(map: &Map, start: (i32, i32), end: (i32, i32)) -> u32 {
        // Simple BFS pathfinding
        let mut visited = vec![false; map.tiles.len()];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start.0, start.1, 0));
        let start_idx = (start.1 * map.width as i32 + start.0) as usize;

        if start_idx >= map.tiles.len() || !map.tiles[start_idx].walkable() {
            return 0; // Invalid start position
        }

        visited[start_idx] = true;

        while let Some((x, y, dist)) = queue.pop_front() {
            if x == end.0 && y == end.1 {
                return dist;
            }

            if dist >= 100 {
                continue; // Path too long, skip this branch
            }

            for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;

                if nx >= 0 && nx < map.width as i32 && ny >= 0 && ny < map.height as i32 {
                    let idx = (ny * map.width as i32 + nx) as usize;
                    if idx < map.tiles.len() && !visited[idx] && map.tiles[idx].walkable() {
                        visited[idx] = true;
                        queue.push_back((nx, ny, dist + 1));
                    }
                }
            }
        }

        0 // No path found
    }
}

// Static constraint rules data
static CONSTRAINT_RULES: Lazy<Vec<ConstraintRule>> = Lazy::new(|| {
    let data = include_str!("../../../data/constraint_rules.json");
    let file: ConstraintRulesFile =
        serde_json::from_str(data).expect("Failed to parse constraint_rules.json");
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

        assert!(ConstraintSystem::are_critical_constraints_satisfied(
            &results
        ));

        let results_with_failed_critical = vec![ConstraintResult {
            rule_id: "critical1".to_string(),
            passed: false,
            severity: ConstraintSeverity::Critical,
            message: "Test".to_string(),
            score: 0.0,
        }];

        assert!(!ConstraintSystem::are_critical_constraints_satisfied(
            &results_with_failed_critical
        ));
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
            resources: vec![ResourcePlacement {
                resource_type: "glass".to_string(),
                x: 10,
                y: 10,
                amount: 50,
            }],
            objectives: vec![ObjectivePlacement {
                objective_type: "exit".to_string(),
                x: 18,
                y: 18,
                required: true,
            }],
        }
    }
}
