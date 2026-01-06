use std::process::Command;
use std::path::Path;
use std::fs;
use serde_json::Value;

#[test]
fn enhanced_tile_generation_test_suite() {
    let test_configs = [
        "saltflat_basic.json",
        "desert_town.json", 
        "ruins_landmark.json",
        "oasis_shrine.json",
        "scrubland_dungeon.json",
        "high_glass_density.json",
    ];
    
    let mut test_results = Vec::new();
    let mut all_passed = true;
    
    println!("🚀 Running Enhanced Tile Generation Test Suite");
    println!("==============================================");
    
    for config_name in &test_configs {
        let config_path = format!("enhanced-tile-test-suite/configs/{}", config_name);
        
        if !Path::new(&config_path).exists() {
            eprintln!("❌ Config not found: {}", config_path);
            all_passed = false;
            continue;
        }
        
        println!("\n🧪 Testing: {}", config_name);
        
        // Run tilegen-test-tool
        let output = Command::new("cargo")
            .args(&["run", "--bin", "tilegen-test-tool", "--", "--config", &config_path])
            .output()
            .expect("Failed to execute tilegen-test-tool");
        
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if success {
            println!("✅ Generation successful");
            
            // Load and validate evaluation
            let config_content = fs::read_to_string(&config_path).expect("Failed to read config");
            let config: Value = serde_json::from_str(&config_content).expect("Failed to parse config");
            let seed = config["seed"].as_u64().unwrap_or(0);
            
            let eval_path = format!("enhanced-tile-test-suite/evaluations/{}_evaluation.json", seed);
            
            if Path::new(&eval_path).exists() {
                let eval_content = fs::read_to_string(&eval_path).expect("Failed to read evaluation");
                let evaluation: Value = serde_json::from_str(&eval_content).expect("Failed to parse evaluation");
                
                let quality_score = evaluation["evaluation"]["quality_score"].as_f64().unwrap_or(0.0);
                let empty_constraints = vec![];
                let constraints = evaluation["evaluation"]["constraints"].as_array().unwrap_or(&empty_constraints);
                let passed_constraints = constraints.iter()
                    .filter(|c| c["passed"].as_bool().unwrap_or(false))
                    .count();
                
                println!("   📊 Quality Score: {:.3}", quality_score);
                println!("   ✅ Constraints: {}/{} passed", passed_constraints, constraints.len());
                
                test_results.push((config_name.to_string(), true, quality_score, passed_constraints, constraints.len()));
                
                if quality_score < 0.5 || passed_constraints < constraints.len() {
                    println!("   ⚠️  Quality concerns detected");
                }
            } else {
                println!("   ❌ Evaluation file not found");
                all_passed = false;
                test_results.push((config_name.to_string(), false, 0.0, 0, 0));
            }
        } else {
            println!("❌ Generation failed");
            println!("   stdout: {}", stdout);
            println!("   stderr: {}", stderr);
            all_passed = false;
            test_results.push((config_name.to_string(), false, 0.0, 0, 0));
        }
    }
    
    // Generate summary report
    generate_test_report(&test_results);
    
    println!("\n📋 Test Suite Summary");
    println!("====================");
    let passed_tests = test_results.iter().filter(|(_, success, _, _, _)| *success).count();
    println!("Tests passed: {}/{}", passed_tests, test_results.len());
    
    if all_passed {
        println!("🎉 All tests passed!");
    } else {
        println!("❌ Some tests failed");
    }
    
    assert!(all_passed, "Enhanced tile generation test suite failed");
}

fn generate_test_report(results: &[(String, bool, f64, usize, usize)]) {
    let mut report = String::new();
    
    report.push_str("# Enhanced Tile Generation Test Report\n\n");
    report.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    report.push_str("## Test Results\n\n");
    report.push_str("| Test | Status | Quality Score | Constraints | PNG | Evaluation |\n");
    report.push_str("|------|--------|---------------|-------------|-----|------------|\n");
    
    for (config_name, success, quality, passed_constraints, total_constraints) in results {
        let status = if *success { "✅ PASS" } else { "❌ FAIL" };
        let config_base = config_name.replace(".json", "");
        
        // Extract seed from config to build file paths
        let config_path = format!("enhanced-tile-test-suite/configs/{}", config_name);
        let seed = if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<Value>(&content) {
                config["seed"].as_u64().unwrap_or(0)
            } else { 0 }
        } else { 0 };
        
        let png_link = format!("![{}](pngs/{}_base_terrain.png)", config_base, seed);
        let eval_link = format!("[JSON](evaluations/{}_evaluation.json)", seed);
        
        report.push_str(&format!(
            "| {} | {} | {:.3} | {}/{} | {} | {} |\n",
            config_base, status, quality, passed_constraints, total_constraints, png_link, eval_link
        ));
    }
    
    report.push_str("\n## Pipeline Stages\n\n");
    report.push_str("Each test validates the following pipeline stages:\n\n");
    report.push_str("1. **Noise Generation** - Multi-layer Perlin noise sampling\n");
    report.push_str("2. **Terrain Classification** - Floor/wall/glass assignment\n");
    report.push_str("3. **Biome Modification** - Biome-specific adjustments\n");
    report.push_str("4. **POI Integration** - Point of interest features\n");
    report.push_str("5. **Constraint Validation** - Connectivity and quality checks\n");
    
    report.push_str("\n## Quality Metrics\n\n");
    report.push_str("- **Quality Score**: Combined connectivity and floor density score (0.0-1.0)\n");
    report.push_str("- **Constraints**: Validation checks for connectivity, density, and accessibility\n");
    report.push_str("- **Pipeline Stages**: Specific generation phases tested per configuration\n");
    
    let report_path = "enhanced-tile-test-suite/TEST_REPORT.md";
    fs::write(report_path, report).expect("Failed to write test report");
    println!("📄 Test report saved: {}", report_path);
}
