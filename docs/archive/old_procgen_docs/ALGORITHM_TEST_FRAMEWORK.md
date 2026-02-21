# Algorithm Test Framework

A flexible framework for testing new generation algorithms with custom configurations and reports.

## Quick Start

### 1. Generate Test Configs for New Algorithm
```bash
./generate_algorithm_configs.sh wave_function_collapse 6000
```

This creates:
- `configs/wave_function_collapse_basic.json`
- `configs/wave_function_collapse_variant.json` 
- `tests/wave_function_collapse_test_suite.rs`

### 2. Customize Algorithm Parameters
Edit the generated config files to match your algorithm's parameters:

```json
{
  "algorithm": "wave_function_collapse",
  "algorithm_params": {
    "tile_size": 3,
    "overlap": 1,
    "entropy_heuristic": "minimum_entropy"
  },
  "pipeline_stages": ["pattern_extraction", "constraint_propagation", "tile_placement"]
}
```

### 3. Run Algorithm-Specific Tests
```bash
cargo test wave_function_collapse_algorithm_test_suite
```

Generates: `enhanced-tile-test-suite/TEST_REPORT_WAVE_FUNCTION_COLLAPSE.md`

## Configuration Format

### Algorithm-Specific Fields
```json
{
  "algorithm": "algorithm_name",           // Algorithm identifier
  "algorithm_params": {                    // Algorithm-specific parameters
    "param1": "value",
    "param2": 42
  },
  "test_suite": "ALGORITHM_UPPER",         // Test suite grouping
  "pipeline_stages": [                     // Algorithm-specific stages
    "stage1", "stage2", "stage3"
  ]
}
```

### Standard Fields
- `seed`: Deterministic generation seed
- `width`/`height`: Map dimensions
- `biome`: Biome type for generation context
- `poi`: Point of interest type
- `terrain_type`: Base terrain configuration
- `output_layers`: Generation layers to output
- `output_format`: File formats (text, png)
- `enable_evaluation`: Generate evaluation metrics

## Creating Test Suites

### Using the Macro
```rust
mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    my_algorithm_test_suite,           // Test function name
    &[                                 // Config files to test
        "my_algorithm_basic.json",
        "my_algorithm_variant.json",
    ],
    "TEST_REPORT_MY_ALGORITHM.md",     // Report filename
    "My Algorithm"                     // Suite description
);
```

### Manual Test Suite
```rust
#[test]
fn custom_algorithm_test() {
    let success = run_algorithm_test_suite(
        &["config1.json", "config2.json"],
        "enhanced-tile-test-suite/configs",
        "TEST_REPORT_CUSTOM.md",
        "Custom Algorithm Suite"
    );
    assert!(success, "Custom algorithm test failed");
}
```

## Algorithm Implementation

### 1. Add Algorithm Support to tilegen-test-tool
```rust
fn generate_enhanced_map(config: &EnhancedConfig) -> Result<(Map, serde_json::Value), Box<dyn std::error::Error>> {
    match config.algorithm.as_deref() {
        Some("bsp") => generate_bsp_map(config),
        Some("cellular_automata") => generate_cellular_map(config),
        Some("wave_function_collapse") => generate_wfc_map(config),
        _ => generate_default_map(config), // Fallback to bracket-noise
    }
}
```

### 2. Implement Algorithm-Specific Generation
```rust
fn generate_wfc_map(config: &EnhancedConfig) -> Result<(Map, serde_json::Value), Box<dyn std::error::Error>> {
    let params = config.algorithm_params.as_ref().unwrap();
    let tile_size = params["tile_size"].as_u64().unwrap_or(3);
    
    // Implement WFC algorithm
    let map = wave_function_collapse_generate(config.seed, tile_size);
    let evaluation = generate_evaluation(&map, config, 0);
    
    Ok((map, evaluation))
}
```

### 3. Add Algorithm-Specific Evaluation
```rust
fn generate_wfc_evaluation(map: &Map, config: &EnhancedConfig) -> serde_json::Value {
    // Algorithm-specific metrics
    let pattern_diversity = calculate_pattern_diversity(map);
    let constraint_satisfaction = check_wfc_constraints(map);
    
    // Include in evaluation JSON
    serde_json::json!({
        "algorithm_metrics": {
            "pattern_diversity": pattern_diversity,
            "constraint_satisfaction": constraint_satisfaction
        }
    })
}
```

## Report Generation

### Automatic Reports
Each test suite generates a markdown report with:
- Test status and quality scores
- Algorithm-specific documentation
- PNG visualizations and evaluation links
- Pipeline stage validation results

### Custom Report Sections
The framework automatically adds algorithm-specific documentation:

- **BSP**: Room connectivity, corridor placement
- **Cellular Automata**: Cave connectivity, natural formations  
- **Custom**: Add your algorithm's validation criteria

### Report Naming Convention
- `TEST_REPORT_BSP.md` - BSP algorithm tests
- `TEST_REPORT_CELLULAR_AUTOMATA.md` - Cellular automata tests
- `TEST_REPORT_DUNGEONS.md` - Combined dungeon algorithms
- `TEST_REPORT_[ALGORITHM].md` - Custom algorithm tests

## Example Algorithms

### BSP (Binary Space Partitioning)
```json
{
  "algorithm": "bsp",
  "algorithm_params": {
    "min_room_size": 6,
    "max_room_size": 12,
    "corridor_width": 1,
    "split_ratio": 0.6
  },
  "pipeline_stages": ["space_partitioning", "room_placement", "corridor_generation"]
}
```

### Cellular Automata
```json
{
  "algorithm": "cellular_automata", 
  "algorithm_params": {
    "initial_density": 0.45,
    "iterations": 5,
    "birth_limit": 4,
    "death_limit": 3
  },
  "pipeline_stages": ["initial_noise", "cellular_iterations", "connectivity_check"]
}
```

## Integration Workflow

1. **Generate Configs**: Use `generate_algorithm_configs.sh`
2. **Customize Parameters**: Edit algorithm_params and pipeline_stages
3. **Implement Algorithm**: Add support to tilegen-test-tool
4. **Run Tests**: Execute algorithm-specific test suite
5. **Review Report**: Check generated markdown report
6. **Iterate**: Adjust parameters and re-test

## Best Practices

### Config Organization
- Use descriptive config names: `algorithm_variant.json`
- Group related configs by algorithm type
- Include algorithm parameters in filename when helpful

### Test Coverage
- Test edge cases (small/large parameters)
- Test different biome/POI combinations
- Include failure cases to validate error handling

### Evaluation Metrics
- Define algorithm-specific quality metrics
- Validate algorithm constraints (connectivity, etc.)
- Include performance measurements when relevant

### Documentation
- Document algorithm parameters in config comments
- Explain pipeline stages and their purpose
- Include expected quality score ranges

---

*This framework enables systematic testing of new generation algorithms with minimal boilerplate code.*
