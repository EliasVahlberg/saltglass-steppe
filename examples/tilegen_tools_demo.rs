use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use saltglass_steppe::game::map::Map;
use saltglass_steppe::tilegen_tools::*;

fn main() {
    println!("🔧 Tilegen Tools Demo");
    println!("====================");

    // Generate a test map
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let (map, _) = Map::generate(&mut rng);

    // Analyze the map
    let connectivity = analyze_connectivity(&map);
    let distribution = analyze_tile_distribution(&map);
    let metrics = calculate_map_metrics(&map);
    let evaluation = evaluate_map_quality(&map);

    // Display results
    println!("\n📊 Map Analysis:");
    println!("  Size: {}x{}", map.width, map.height);
    println!("  Total tiles: {}", metrics.total_tiles);
    println!("  Openness: {:.2}", metrics.openness);
    println!("  Complexity: {:.2}", metrics.complexity);

    println!("\n🔗 Connectivity:");
    println!("  Floor tiles: {}", connectivity.total_floor_tiles);
    println!("  Connected regions: {}", connectivity.connected_regions);
    println!(
        "  Connectivity ratio: {:.2}",
        connectivity.connectivity_ratio
    );
    println!("  Isolated regions: {}", connectivity.isolated_regions);

    println!("\n📈 Tile Distribution:");
    for (tile_type, count) in &distribution.counts {
        let percentage = *count as f32 / distribution.total as f32 * 100.0;
        println!("  {}: {} ({:.1}%)", tile_type, count, percentage);
    }

    println!("\n✅ Quality Evaluation:");
    println!("  Overall score: {:.2}/1.00", evaluation.quality_score);
    println!(
        "  Constraints passed: {}/{}",
        evaluation.passed_constraints, evaluation.total_constraints
    );

    println!("\n📋 Detailed Report:");
    println!("{}", generate_evaluation_report(&evaluation));
}
