#!/bin/bash

# Algorithm Test Config Generator
# Usage: ./generate_algorithm_configs.sh <algorithm_name> <base_seed>

ALGORITHM_NAME=$1
BASE_SEED=${2:-5000}

if [ -z "$ALGORITHM_NAME" ]; then
    echo "Usage: $0 <algorithm_name> [base_seed]"
    echo "Example: $0 wave_function_collapse 5000"
    exit 1
fi

CONFIG_DIR="enhanced-tile-test-suite/configs"
ALGORITHM_UPPER=$(echo "$ALGORITHM_NAME" | tr '[:lower:]' '[:upper:]')

echo "🔧 Generating test configs for $ALGORITHM_NAME algorithm..."

# Basic config
cat > "$CONFIG_DIR/${ALGORITHM_NAME}_basic.json" << EOF
{
  "seed": $BASE_SEED,
  "width": 80,
  "height": 60,
  "biome": "saltflat",
  "poi": null,
  "terrain_type": "flat",
  "algorithm": "$ALGORITHM_NAME",
  "algorithm_params": {
    "param1": "value1",
    "param2": 42
  },
  "use_bracket_noise": false,
  "output_layers": ["base_terrain"],
  "output_format": ["text", "png"],
  "enable_evaluation": true,
  "pipeline_stages": ["algorithm_stage_1", "algorithm_stage_2"],
  "test_suite": "$ALGORITHM_UPPER",
  "output_dir": "enhanced-tile-test-suite"
}
EOF

# Variant config
cat > "$CONFIG_DIR/${ALGORITHM_NAME}_variant.json" << EOF
{
  "seed": $((BASE_SEED + 1)),
  "width": 80,
  "height": 60,
  "biome": "ruins",
  "poi": "dungeon",
  "terrain_type": "canyon",
  "algorithm": "$ALGORITHM_NAME",
  "algorithm_params": {
    "param1": "variant_value",
    "param2": 84
  },
  "use_bracket_noise": false,
  "output_layers": ["base_terrain"],
  "output_format": ["text", "png"],
  "enable_evaluation": true,
  "pipeline_stages": ["algorithm_stage_1", "algorithm_stage_2"],
  "test_suite": "$ALGORITHM_UPPER",
  "output_dir": "enhanced-tile-test-suite"
}
EOF

# Generate test suite code
cat > "tests/${ALGORITHM_NAME}_test_suite.rs" << EOF
mod algorithm_test_framework;
use algorithm_test_framework::*;

algorithm_test_suite!(
    ${ALGORITHM_NAME}_algorithm_test_suite,
    &[
        "${ALGORITHM_NAME}_basic.json",
        "${ALGORITHM_NAME}_variant.json",
    ],
    "TEST_REPORT_${ALGORITHM_UPPER}.md",
    "$ALGORITHM_NAME Algorithm"
);
EOF

echo "✅ Generated configs:"
echo "   - $CONFIG_DIR/${ALGORITHM_NAME}_basic.json"
echo "   - $CONFIG_DIR/${ALGORITHM_NAME}_variant.json"
echo "   - tests/${ALGORITHM_NAME}_test_suite.rs"
echo ""
echo "📝 Next steps:"
echo "   1. Edit the algorithm_params in the config files"
echo "   2. Update pipeline_stages to match your algorithm"
echo "   3. Implement algorithm support in tilegen-test-tool"
echo "   4. Run: cargo test ${ALGORITHM_NAME}_algorithm_test_suite"
