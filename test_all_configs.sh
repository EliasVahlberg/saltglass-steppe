#!/bin/bash

# Test all algorithm configurations and generate report
set -e

echo "🧪 Testing All Algorithm Configurations"
echo "======================================="

CONFIG_DIR="enhanced-tile-test-suite/configs"
REPORT_FILE="enhanced-tile-test-suite/TEST_REPORT.md"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Build release binary once if needed
BINARY="./target/release/tilegen-test-tool"
if [ ! -f "$BINARY" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --bin tilegen-test-tool --release --quiet
else
    echo -e "${YELLOW}Using existing release binary...${NC}"
fi

# Counters
TOTAL=0
PASSED=0
FAILED=0

# Start report
cat > "$REPORT_FILE" << EOF
# Algorithm Configuration Test Report

Generated: $(date)

## Test Results

| Test | Status | Quality Score | Constraints | PNG | Evaluation |
|------|--------|---------------|-------------|-----|------------|
EOF

echo -e "${YELLOW}Processing configurations...${NC}"

# Test each config
for config in "$CONFIG_DIR"/*.json; do
    if [ -f "$config" ]; then
        filename=$(basename "$config" .json)
        echo -n "Testing $filename.json... "
        
        TOTAL=$((TOTAL + 1))
        
        if $BINARY --config "$config" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ PASSED${NC}"
            
            # Extract seed from config to find evaluation file
            seed=$(jq -r '.seed' "$config" 2>/dev/null || echo "unknown")
            eval_file="enhanced-tile-test-suite/evaluations/${seed}_evaluation.json"
            
            if [ -f "$eval_file" ]; then
                quality_score=$(jq -r '.evaluation.quality_score // "N/A"' "$eval_file" 2>/dev/null || echo "N/A")
                constraints_passed=$(jq -r '.evaluation.constraints | map(select(.passed == true)) | length' "$eval_file" 2>/dev/null || echo "0")
                constraints_total=$(jq -r '.evaluation.constraints | length' "$eval_file" 2>/dev/null || echo "0")
                constraints="${constraints_passed}/${constraints_total}"
            else
                quality_score="N/A"
                constraints="N/A"
            fi
            
            png_file="pngs/${seed}_base_terrain.png"
            eval_file_link="evaluations/${seed}_evaluation.json"
            
            echo "| $filename | ✅ PASSED | $quality_score | $constraints | ![PNG]($png_file) | [JSON]($eval_file_link) |" >> "$REPORT_FILE"
            PASSED=$((PASSED + 1))
        else
            echo -e "${RED}❌ FAILED${NC}"
            echo "| $filename | ❌ FAILED | N/A | N/A | N/A | N/A |" >> "$REPORT_FILE"
            FAILED=$((FAILED + 1))
        fi
    fi
done

# Add summary to report
cat >> "$REPORT_FILE" << EOF

## Summary

- **Total Configurations**: $TOTAL
- **Passed**: $PASSED
- **Failed**: $FAILED
- **Success Rate**: $(( PASSED * 100 / TOTAL ))%

## Generated Files

- Text outputs: \`enhanced-tile-test-suite/text/\`
- PNG visualizations: \`enhanced-tile-test-suite/pngs/\`
- Evaluation data: \`enhanced-tile-test-suite/evaluations/\`

EOF

echo ""
echo -e "${YELLOW}📊 Test Summary:${NC}"
echo "  Total: $TOTAL"
echo "  Passed: $PASSED"
echo "  Failed: $FAILED"
echo "  Success Rate: $(( PASSED * 100 / TOTAL ))%"
echo ""
echo -e "${GREEN}📄 Report generated: $REPORT_FILE${NC}"
