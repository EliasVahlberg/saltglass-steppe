use saltglass_steppe::game::generation::structures::algorithms::{CellularAutomataAlgorithm, CellularAutomataParams};
use saltglass_steppe::game::generation::structures::Rectangle;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

fn main() {
    println!("🧪 Cellular Automata Algorithm Test");
    println!("=====================================");
    
    // Test with default parameters
    let params = CellularAutomataParams::default();
    println!("Parameters:");
    println!("  Initial wall probability: {:.2}", params.initial_wall_probability);
    println!("  Iterations: {}", params.iterations);
    println!("  Survival threshold: {}", params.survival_threshold);
    println!("  Birth threshold: {}", params.birth_threshold);
    println!("  Moore neighborhood: {}", params.use_moore_neighborhood);
    println!();
    
    let algorithm = CellularAutomataAlgorithm::new(params);
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    
    // Test different sizes
    let test_cases = vec![
        Rectangle::new(0, 0, 20, 15),
        Rectangle::new(0, 0, 30, 20),
        Rectangle::new(0, 0, 40, 25),
    ];
    
    for (i, bounds) in test_cases.iter().enumerate() {
        println!("Test Case {}: {}x{} area", i + 1, bounds.width, bounds.height);
        
        let walls = algorithm.generate(bounds.clone(), &mut rng);
        let total_cells = (bounds.width * bounds.height) as usize;
        let wall_percentage = (walls.len() as f64 / total_cells as f64) * 100.0;
        
        println!("  Generated {} walls out of {} cells ({:.1}%)", 
                 walls.len(), total_cells, wall_percentage);
        
        // Validate bounds
        let mut valid = true;
        for (x, y) in &walls {
            if *x < bounds.x as i32 || *x >= (bounds.x + bounds.width) as i32 ||
               *y < bounds.y as i32 || *y >= (bounds.y + bounds.height) as i32 {
                valid = false;
                break;
            }
        }
        
        println!("  Bounds validation: {}", if valid { "✅ PASS" } else { "❌ FAIL" });
        println!();
    }
    
    // Test determinism
    println!("Determinism Test:");
    let bounds = Rectangle::new(0, 0, 25, 20);
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(42);
    
    let walls1 = algorithm.generate(bounds.clone(), &mut rng1);
    let walls2 = algorithm.generate(bounds.clone(), &mut rng2);
    
    let deterministic = walls1 == walls2;
    println!("  Same seed produces same result: {}", if deterministic { "✅ PASS" } else { "❌ FAIL" });
    
    // Test different parameters
    println!("\nParameter Variation Test:");
    let mut high_density_params = CellularAutomataParams::default();
    high_density_params.initial_wall_probability = 0.6;
    high_density_params.iterations = 3;
    
    let high_density_algorithm = CellularAutomataAlgorithm::new(high_density_params);
    let mut rng3 = ChaCha8Rng::seed_from_u64(12345);
    let high_density_walls = high_density_algorithm.generate(bounds.clone(), &mut rng3);
    
    let total_cells = (bounds.width * bounds.height) as usize;
    let high_density_percentage = (high_density_walls.len() as f64 / total_cells as f64) * 100.0;
    
    println!("  High density (60% initial): {} walls ({:.1}%)", 
             high_density_walls.len(), high_density_percentage);
    
    println!("\n🎉 Cellular Automata Algorithm Test Complete!");
}
