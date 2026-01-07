use std::process::Command;
use std::path::Path;
use std::fs;
use serde_json::Value;

pub fn run_algorithm_test_suite(
    test_configs: &[&str],
    config_dir: &str,
    report_name: &str,
    suite_description: &str,
) -> bool {
    let mut test_results = Vec::new();
    let mut all_passed = true;
    
    println!("🚀 Running {} Test Suite", suite_description);
    println!("{}", "=".repeat(50 + suite_description.len()));
    
    for config_name in test_configs {
        let config_path = format!("{}/{}", config_dir, config_name);
        
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
        
        if success {
            println!("✅ Generation successful");
            
            // Load and validate evaluation
            let config_content = fs::read_to_string(&config_path).expect("Failed to read config");
            let config: Value = serde_json::from_str(&config_content).expect("Failed to parse config");
            let seed = config["seed"].as_u64().unwrap_or(0);
            let output_dir = config["output_dir"].as_str().unwrap_or("enhanced-tile-test-suite");
            
            let eval_path = format!("{}/evaluations/{}_evaluation.json", output_dir, seed);
            
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
                
                test_results.push((config_name.to_string(), true, quality_score, passed_constraints, constraints.len(), output_dir.to_string()));
                
                if quality_score < 0.5 || passed_constraints < constraints.len() {
                    println!("   ⚠️  Quality concerns detected");
                }
            } else {
                println!("   ❌ Evaluation file not found");
                all_passed = false;
                test_results.push((config_name.to_string(), false, 0.0, 0, 0, "".to_string()));
            }
        } else {
            println!("❌ Generation failed");
            all_passed = false;
            test_results.push((config_name.to_string(), false, 0.0, 0, 0, "".to_string()));
        }
    }
    
    // Generate custom report
    generate_algorithm_test_report(&test_results, report_name, suite_description);
    
    println!("\n📋 {} Test Suite Summary", suite_description);
    println!("{}", "=".repeat(30 + suite_description.len()));
    let passed_tests = test_results.iter().filter(|(_, success, _, _, _, _)| *success).count();
    println!("Tests passed: {}/{}", passed_tests, test_results.len());
    
    if all_passed {
        println!("🎉 All tests passed!");
    } else {
        println!("❌ Some tests failed");
    }
    
    all_passed
}

fn generate_algorithm_test_report(
    results: &[(String, bool, f64, usize, usize, String)], 
    report_name: &str,
    suite_description: &str
) {
    let mut report = String::new();
    
    report.push_str(&format!("# {} Test Report\n\n", suite_description));
    report.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    report.push_str("## Test Results\n\n");
    report.push_str("| Test | Status | Quality Score | Constraints | PNG | Evaluation |\n");
    report.push_str("|------|--------|---------------|-------------|-----|------------|\n");
    
    for (config_name, success, quality, passed_constraints, total_constraints, output_dir) in results {
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
    
    report.push_str(&format!("\n## {} Algorithm Details\n\n", suite_description));
    report.push_str("This test suite validates specific algorithm implementations:\n\n");
    
    // Add algorithm-specific documentation based on suite type
    match suite_description.to_lowercase().as_str() {
        s if s.contains("bsp") => {
            report.push_str("### Binary Space Partitioning (BSP)\n");
            report.push_str("- **Purpose**: Room-based dungeon generation\n");
            report.push_str("- **Method**: Recursive space subdivision\n");
            report.push_str("- **Validation**: Room connectivity, corridor placement\n");
        },
        s if s.contains("cellular") => {
            report.push_str("### Cellular Automata\n");
            report.push_str("- **Purpose**: Organic cave generation\n");
            report.push_str("- **Method**: Iterative neighbor-based rules\n");
            report.push_str("- **Validation**: Cave connectivity, natural formations\n");
        },
        s if s.contains("dungeon") => {
            report.push_str("### Dungeon Generation\n");
            report.push_str("- **Purpose**: Structured underground areas\n");
            report.push_str("- **Method**: Multiple algorithm combination\n");
            report.push_str("- **Validation**: Room placement, accessibility\n");
        },
        _ => {
            report.push_str("### Algorithm-Specific Testing\n");
            report.push_str("- **Purpose**: Validate specific generation methods\n");
            report.push_str("- **Method**: Targeted test configurations\n");
            report.push_str("- **Validation**: Algorithm-specific constraints\n");
        }
    }
    
    report.push_str("\n## Quality Metrics\n\n");
    report.push_str("- **Quality Score**: Algorithm-specific quality measurement (0.0-1.0)\n");
    report.push_str("- **Constraints**: Algorithm-specific validation checks\n");
    report.push_str("- **Visual Output**: PNG files show generated structures\n");
    
    let report_path = format!("enhanced-tile-test-suite/{}", report_name);
    fs::write(&report_path, report).expect("Failed to write test report");
    println!("📄 Test report saved: {}", report_path);
}

// Macro for easy test suite creation
#[macro_export]
macro_rules! algorithm_test_suite {
    ($test_name:ident, $configs:expr, $report:expr, $description:expr) => {
        #[test]
        fn $test_name() {
            let success = run_algorithm_test_suite(
                $configs,
                "enhanced-tile-test-suite/configs",
                $report,
                $description,
            );
            assert!(success, "{} test suite failed", $description);
        }
    };
}
