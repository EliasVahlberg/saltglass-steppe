//! Quest Satisfaction Constraint System
//!
//! Ensures that quest content can spawn properly and is reachable/interactable.
//! Integrates with the existing constraint system to guarantee playable quest content.

use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use super::constraints::{ConstraintContext, ConstraintResult, ConstraintSeverity};
use crate::game::map::{Map, Tile};
use crate::game::quest::{ObjectiveType, QuestDef, get_quest_def};

/// Quest-specific constraint validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestConstraint {
    pub quest_id: String,
    pub objective_requirements: Vec<ObjectiveRequirement>,
    pub accessibility_requirements: AccessibilityRequirement,
    pub spawn_requirements: SpawnRequirement,
}

/// Requirements for a specific quest objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveRequirement {
    pub objective_id: String,
    pub objective_type: ObjectiveType,
    pub spatial_requirements: SpatialRequirement,
    pub entity_requirements: Vec<EntityRequirement>,
}

/// Spatial requirements for quest objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialRequirement {
    pub min_accessible_area: u32,
    pub max_distance_from_spawn: Option<u32>,
    pub requires_line_of_sight: bool,
    pub requires_safe_path: bool,
}

/// Entity requirements for quest objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRequirement {
    pub entity_type: String, // "npc", "item", "enemy", "interactable"
    pub entity_id: String,
    pub must_be_reachable: bool,
    pub must_be_interactable: bool,
    pub min_spawn_chance: f32,
}

/// Accessibility requirements for the entire quest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityRequirement {
    pub min_connectivity_score: f32,
    pub requires_player_spawn_access: bool,
    pub max_blocked_objectives: u32,
}

/// Spawn requirements for quest content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRequirement {
    pub required_biomes: Vec<String>,
    pub required_microstructures: Vec<String>,
    pub min_structure_coverage: f32, // Percentage of tile that must be covered by structures
    pub hostile_density_range: (f32, f32), // Min/max hostile entities per area
}

/// Quest constraint validation system
pub struct QuestConstraintSystem;

impl QuestConstraintSystem {
    /// Validate all quest constraints for a tile
    pub fn validate_quest_constraints(
        quest_ids: &[String],
        context: &ConstraintContext,
        rng: &mut ChaCha8Rng,
    ) -> Vec<ConstraintResult> {
        let mut results = Vec::new();

        for quest_id in quest_ids {
            if let Some(quest_def) = get_quest_def(quest_id) {
                let quest_constraint = Self::generate_quest_constraint(quest_def);
                let quest_results = Self::validate_single_quest(&quest_constraint, context, rng);
                results.extend(quest_results);
            }
        }

        results
    }

    /// Generate quest constraints from quest definition
    fn generate_quest_constraint(quest_def: &QuestDef) -> QuestConstraint {
        let mut objective_requirements = Vec::new();

        for objective in &quest_def.objectives {
            let spatial_req = match &objective.objective_type {
                ObjectiveType::Reach { .. } => SpatialRequirement {
                    min_accessible_area: 1,
                    max_distance_from_spawn: Some(100),
                    requires_line_of_sight: false,
                    requires_safe_path: true,
                },
                ObjectiveType::TalkTo { .. } => SpatialRequirement {
                    min_accessible_area: 4, // NPC needs some space around them
                    max_distance_from_spawn: Some(150),
                    requires_line_of_sight: false,
                    requires_safe_path: true,
                },
                ObjectiveType::Collect { .. } => SpatialRequirement {
                    min_accessible_area: 1,
                    max_distance_from_spawn: Some(200),
                    requires_line_of_sight: false,
                    requires_safe_path: true,
                },
                ObjectiveType::Interact { .. } | ObjectiveType::Examine { .. } => {
                    SpatialRequirement {
                        min_accessible_area: 1,
                        max_distance_from_spawn: Some(150),
                        requires_line_of_sight: false,
                        requires_safe_path: true,
                    }
                }
                ObjectiveType::InterfaceWithAria { .. } => SpatialRequirement {
                    min_accessible_area: 9, // Archive terminals need more space
                    max_distance_from_spawn: Some(100),
                    requires_line_of_sight: true,
                    requires_safe_path: true,
                },
                _ => SpatialRequirement {
                    min_accessible_area: 1,
                    max_distance_from_spawn: None,
                    requires_line_of_sight: false,
                    requires_safe_path: false,
                },
            };

            let entity_requirements = Self::generate_entity_requirements(&objective.objective_type);

            objective_requirements.push(ObjectiveRequirement {
                objective_id: objective.id.clone(),
                objective_type: objective.objective_type.clone(),
                spatial_requirements: spatial_req,
                entity_requirements,
            });
        }

        // Determine accessibility and spawn requirements based on quest category
        let (accessibility_req, spawn_req) = match quest_def.category.as_str() {
            "main" => (
                AccessibilityRequirement {
                    min_connectivity_score: 0.9, // Main quests must be highly accessible
                    requires_player_spawn_access: true,
                    max_blocked_objectives: 0, // No blocked objectives allowed
                },
                SpawnRequirement {
                    required_biomes: vec!["ruins".to_string()], // Main quests often in ruins
                    required_microstructures: vec!["vitrified_library_ruins".to_string()],
                    min_structure_coverage: 0.4, // 40% coverage for main quest locations
                    hostile_density_range: (0.3, 0.7), // Moderate to high danger
                },
            ),
            "side" => (
                AccessibilityRequirement {
                    min_connectivity_score: 0.7,
                    requires_player_spawn_access: true,
                    max_blocked_objectives: 1, // One blocked objective allowed
                },
                SpawnRequirement {
                    required_biomes: vec![], // Side quests can be anywhere
                    required_microstructures: vec![],
                    min_structure_coverage: 0.1, // 10% coverage minimum
                    hostile_density_range: (0.1, 0.5), // Lower danger
                },
            ),
            _ => (
                AccessibilityRequirement {
                    min_connectivity_score: 0.5,
                    requires_player_spawn_access: false,
                    max_blocked_objectives: 2,
                },
                SpawnRequirement {
                    required_biomes: vec![],
                    required_microstructures: vec![],
                    min_structure_coverage: 0.05,
                    hostile_density_range: (0.0, 0.3),
                },
            ),
        };

        QuestConstraint {
            quest_id: quest_def.id.clone(),
            objective_requirements,
            accessibility_requirements: accessibility_req,
            spawn_requirements: spawn_req,
        }
    }

    /// Generate entity requirements for an objective type
    fn generate_entity_requirements(objective_type: &ObjectiveType) -> Vec<EntityRequirement> {
        match objective_type {
            ObjectiveType::TalkTo { npc_id } => vec![EntityRequirement {
                entity_type: "npc".to_string(),
                entity_id: npc_id.clone(),
                must_be_reachable: true,
                must_be_interactable: true,
                min_spawn_chance: 0.9,
            }],
            ObjectiveType::Collect { item_id, .. } => vec![EntityRequirement {
                entity_type: "item".to_string(),
                entity_id: item_id.clone(),
                must_be_reachable: true,
                must_be_interactable: true,
                min_spawn_chance: 0.8,
            }],
            ObjectiveType::Interact { target } | ObjectiveType::Examine { target } => {
                vec![EntityRequirement {
                    entity_type: "interactable".to_string(),
                    entity_id: target.clone(),
                    must_be_reachable: true,
                    must_be_interactable: true,
                    min_spawn_chance: 0.9,
                }]
            }
            ObjectiveType::InterfaceWithAria { .. } => vec![EntityRequirement {
                entity_type: "npc".to_string(),
                entity_id: "archive_terminal".to_string(),
                must_be_reachable: true,
                must_be_interactable: true,
                min_spawn_chance: 1.0,
            }],
            _ => vec![], // Kill, Wait, CollectData don't require specific entities
        }
    }

    /// Validate a single quest's constraints
    fn validate_single_quest(
        quest_constraint: &QuestConstraint,
        context: &ConstraintContext,
        _rng: &mut ChaCha8Rng,
    ) -> Vec<ConstraintResult> {
        let mut results = Vec::new();

        // Validate accessibility requirements
        results.push(Self::validate_accessibility_requirement(
            &quest_constraint.accessibility_requirements,
            context,
            &quest_constraint.quest_id,
        ));

        // Validate spawn requirements
        results.push(Self::validate_spawn_requirement(
            &quest_constraint.spawn_requirements,
            context,
            &quest_constraint.quest_id,
        ));

        // Validate each objective requirement
        for obj_req in &quest_constraint.objective_requirements {
            results.extend(Self::validate_objective_requirement(
                obj_req,
                context,
                &quest_constraint.quest_id,
            ));
        }

        results
    }

    /// Validate accessibility requirements
    fn validate_accessibility_requirement(
        req: &AccessibilityRequirement,
        context: &ConstraintContext,
        quest_id: &str,
    ) -> ConstraintResult {
        let connectivity_score = Self::calculate_connectivity_score(context.map);
        let passed = connectivity_score >= req.min_connectivity_score;

        ConstraintResult {
            rule_id: format!("{}_accessibility", quest_id),
            passed,
            severity: ConstraintSeverity::Critical,
            message: if passed {
                format!(
                    "Quest {} accessibility satisfied (score: {:.2})",
                    quest_id, connectivity_score
                )
            } else {
                format!(
                    "Quest {} accessibility failed (score: {:.2}, required: {:.2})",
                    quest_id, connectivity_score, req.min_connectivity_score
                )
            },
            score: if passed {
                1.0
            } else {
                connectivity_score / req.min_connectivity_score
            },
        }
    }

    /// Validate spawn requirements
    fn validate_spawn_requirement(
        req: &SpawnRequirement,
        context: &ConstraintContext,
        quest_id: &str,
    ) -> ConstraintResult {
        let structure_coverage = Self::calculate_structure_coverage(context);
        let hostile_density = Self::calculate_hostile_density(context);

        let coverage_ok = structure_coverage >= req.min_structure_coverage;
        let density_ok = hostile_density >= req.hostile_density_range.0
            && hostile_density <= req.hostile_density_range.1;

        let passed = coverage_ok && density_ok;

        ConstraintResult {
            rule_id: format!("{}_spawn", quest_id),
            passed,
            severity: ConstraintSeverity::Critical,
            message: if passed {
                format!("Quest {} spawn requirements satisfied", quest_id)
            } else {
                format!(
                    "Quest {} spawn requirements failed (coverage: {:.2}, density: {:.2})",
                    quest_id, structure_coverage, hostile_density
                )
            },
            score: if passed { 1.0 } else { 0.5 },
        }
    }

    /// Validate objective requirements
    fn validate_objective_requirement(
        req: &ObjectiveRequirement,
        context: &ConstraintContext,
        quest_id: &str,
    ) -> Vec<ConstraintResult> {
        let mut results = Vec::new();

        // Validate spatial requirements
        results.push(Self::validate_spatial_requirement(
            &req.spatial_requirements,
            context,
            quest_id,
            &req.objective_id,
        ));

        // Validate entity requirements
        for entity_req in &req.entity_requirements {
            results.push(Self::validate_entity_requirement(
                entity_req,
                context,
                quest_id,
                &req.objective_id,
            ));
        }

        results
    }

    /// Validate spatial requirements for an objective
    fn validate_spatial_requirement(
        req: &SpatialRequirement,
        context: &ConstraintContext,
        quest_id: &str,
        objective_id: &str,
    ) -> ConstraintResult {
        let accessible_area = Self::calculate_accessible_area(context.map);
        let passed = accessible_area >= req.min_accessible_area;

        ConstraintResult {
            rule_id: format!("{}_{}_spatial", quest_id, objective_id),
            passed,
            severity: ConstraintSeverity::Critical,
            message: if passed {
                format!("Objective {} spatial requirements satisfied", objective_id)
            } else {
                format!(
                    "Objective {} spatial requirements failed (area: {}, required: {})",
                    objective_id, accessible_area, req.min_accessible_area
                )
            },
            score: if passed {
                1.0
            } else {
                accessible_area as f32 / req.min_accessible_area as f32
            },
        }
    }

    /// Validate entity requirements for an objective
    fn validate_entity_requirement(
        req: &EntityRequirement,
        context: &ConstraintContext,
        quest_id: &str,
        objective_id: &str,
    ) -> ConstraintResult {
        let entity_present = Self::check_entity_presence(req, context);
        let entity_reachable = if req.must_be_reachable {
            Self::check_entity_reachability(req, context)
        } else {
            true
        };

        let passed = entity_present && entity_reachable;

        ConstraintResult {
            rule_id: format!("{}_{}_{}_entity", quest_id, objective_id, req.entity_id),
            passed,
            severity: ConstraintSeverity::Critical,
            message: if passed {
                format!(
                    "Entity {} requirements satisfied for objective {}",
                    req.entity_id, objective_id
                )
            } else {
                format!(
                    "Entity {} requirements failed for objective {} (present: {}, reachable: {})",
                    req.entity_id, objective_id, entity_present, entity_reachable
                )
            },
            score: if passed { 1.0 } else { 0.0 },
        }
    }

    /// Calculate connectivity score for the map
    fn calculate_connectivity_score(map: &Map) -> f32 {
        let total_floor_tiles = map.tiles.iter().filter(|tile| tile.walkable()).count();

        if total_floor_tiles == 0 {
            return 0.0;
        }

        // Find a starting floor tile
        let start_pos = map
            .tiles
            .iter()
            .enumerate()
            .find(|(_, tile)| tile.walkable())
            .map(|(i, _)| (i as i32 % map.width as i32, i as i32 / map.width as i32));

        if let Some((start_x, start_y)) = start_pos {
            let reachable_tiles = Self::flood_fill_reachable(map, start_x, start_y);
            reachable_tiles as f32 / total_floor_tiles as f32
        } else {
            0.0
        }
    }

    /// Calculate structure coverage percentage
    fn calculate_structure_coverage(context: &ConstraintContext) -> f32 {
        let total_tiles = (context.map.width * context.map.height) as f32;
        let structure_tiles = context
            .map
            .tiles
            .iter()
            .filter(|tile| !matches!(tile, Tile::Glass { .. }))
            .count() as f32;

        structure_tiles / total_tiles
    }

    /// Calculate hostile entity density
    fn calculate_hostile_density(context: &ConstraintContext) -> f32 {
        let hostile_count = context
            .entities
            .iter()
            .filter(|e| e.entity_type == "enemy")
            .count() as f32;

        let total_area = (context.map.width * context.map.height) as f32;
        hostile_count / total_area * 100.0 // Entities per 100 tiles
    }

    /// Calculate accessible area
    fn calculate_accessible_area(map: &Map) -> u32 {
        map.tiles.iter().filter(|tile| tile.walkable()).count() as u32
    }

    /// Check if required entity is present
    fn check_entity_presence(req: &EntityRequirement, context: &ConstraintContext) -> bool {
        context.entities.iter().any(|e| {
            e.entity_type == req.entity_type
                && e.properties
                    .get("id")
                    .map_or(false, |id| id == &req.entity_id)
        })
    }

    /// Check if entity is reachable from spawn
    fn check_entity_reachability(req: &EntityRequirement, context: &ConstraintContext) -> bool {
        // Find the entity
        if let Some(entity) = context.entities.iter().find(|e| {
            e.entity_type == req.entity_type
                && e.properties
                    .get("id")
                    .map_or(false, |id| id == &req.entity_id)
        }) {
            // Simple reachability check - ensure the entity is on a walkable tile
            if let Some(tile) = context.map.get(entity.x, entity.y) {
                tile.walkable()
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Flood fill to find reachable tiles
    fn flood_fill_reachable(map: &Map, start_x: i32, start_y: i32) -> usize {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((start_x, start_y));

        while let Some((x, y)) = queue.pop_front() {
            if visited.contains(&(x, y)) {
                continue;
            }

            if let Some(tile) = map.get(x, y) {
                if tile.walkable() {
                    visited.insert((x, y));

                    // Add neighbors
                    for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                        let nx = x + dx;
                        let ny = y + dy;
                        if !visited.contains(&(nx, ny)) {
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }
        }

        visited.len()
    }
}

/// Integration with tile generation
impl QuestConstraintSystem {
    /// Validate quest constraints during tile generation
    pub fn validate_tile_for_quests(
        map: &Map,
        quest_ids: &[String],
        entities: &[super::constraints::EntityPlacement],
        rng: &mut ChaCha8Rng,
    ) -> (bool, Vec<ConstraintResult>) {
        let context = ConstraintContext {
            map,
            biome: crate::game::world_map::Biome::Ruins, // Default for quest locations
            entities: entities.to_vec(),
            resources: vec![],  // Not used for quest validation
            objectives: vec![], // Generated from quest definitions
        };

        let results = Self::validate_quest_constraints(quest_ids, &context, rng);
        let critical_satisfied = results
            .iter()
            .filter(|r| r.severity == ConstraintSeverity::Critical)
            .all(|r| r.passed);

        (critical_satisfied, results)
    }
}
