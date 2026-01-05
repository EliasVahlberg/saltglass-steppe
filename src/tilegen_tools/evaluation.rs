use crate::game::map::Map;
use crate::tilegen_tools::{ConnectivityAnalysis, TileDistribution, ConstraintResult};
use serde::{Serialize, Deserialize};

/// Comprehensive map evaluation
pub fn evaluate_map_quality(map: &Map) -> MapEvaluation {
    let connectivity = crate::tilegen_tools::analyze_connectivity(map);
    let distribution = crate::tilegen_tools::analyze_tile_distribution(map);
    let constraints = crate::tilegen_tools::validate_standard_constraints(map, &connectivity);
    
    let passed_constraints = constraints.iter().filter(|c| c.passed).count();
    let total_constraints = constraints.len();
    let quality_score = calculate_quality_score(&connectivity, &distribution, &constraints);
    
    MapEvaluation {
        connectivity,
        distribution,
        constraints,
        quality_score,
        passed_constraints,
        total_constraints,
    }
}

/// Calculate overall quality score (0.0 to 1.0)
pub fn calculate_quality_score(
    connectivity: &ConnectivityAnalysis,
    distribution: &TileDistribution,
    constraints: &[ConstraintResult],
) -> f32 {
    let constraint_score = constraints.iter().filter(|c| c.passed).count() as f32 / constraints.len() as f32;
    let connectivity_score = connectivity.connectivity_ratio;
    let balance_score = calculate_balance_score(distribution);
    
    (constraint_score * 0.4 + connectivity_score * 0.4 + balance_score * 0.2).min(1.0)
}

/// Calculate balance score based on tile distribution
fn calculate_balance_score(distribution: &TileDistribution) -> f32 {
    let floor_ratio = *distribution.counts.get("floor").unwrap_or(&0) as f32 / distribution.total as f32;
    let wall_ratio = *distribution.counts.get("wall").unwrap_or(&0) as f32 / distribution.total as f32;
    
    // Ideal ratios: 40-60% floor, 30-50% wall
    let floor_score = if floor_ratio >= 0.4 && floor_ratio <= 0.6 { 1.0 } else { 0.5 };
    let wall_score = if wall_ratio >= 0.3 && wall_ratio <= 0.5 { 1.0 } else { 0.5 };
    
    (floor_score + wall_score) / 2.0
}

/// Generate human-readable evaluation report
pub fn generate_evaluation_report(evaluation: &MapEvaluation) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("Map Quality Score: {:.2}/1.00\n", evaluation.quality_score));
    report.push_str(&format!("Constraints: {}/{} passed\n\n", evaluation.passed_constraints, evaluation.total_constraints));
    
    report.push_str("Connectivity Analysis:\n");
    report.push_str(&format!("  - Total floor tiles: {}\n", evaluation.connectivity.total_floor_tiles));
    report.push_str(&format!("  - Connected regions: {}\n", evaluation.connectivity.connected_regions));
    report.push_str(&format!("  - Connectivity ratio: {:.2}\n", evaluation.connectivity.connectivity_ratio));
    report.push_str(&format!("  - Isolated regions: {}\n\n", evaluation.connectivity.isolated_regions));
    
    report.push_str("Constraint Results:\n");
    for constraint in &evaluation.constraints {
        let status = if constraint.passed { "✓" } else { "✗" };
        report.push_str(&format!("  {} {}\n", status, constraint.message));
    }
    
    report
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEvaluation {
    pub connectivity: ConnectivityAnalysis,
    pub distribution: TileDistribution,
    pub constraints: Vec<ConstraintResult>,
    pub quality_score: f32,
    pub passed_constraints: usize,
    pub total_constraints: usize,
}
