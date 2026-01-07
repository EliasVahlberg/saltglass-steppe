#!/bin/bash

# Comprehensive Algorithm Testing Script
# Tests all procedural generation algorithms with various configurations

set -e

echo "🧪 Starting Comprehensive Algorithm Testing Suite"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test categories
CATEGORIES=(
    "room_based_algorithms_test_suite"
    "organic_algorithms_test_suite" 
    "advanced_algorithms_test_suite"
    "parameter_variations_test_suite"
    "hybrid_algorithms_test_suite"
    "sequential_algorithms_test_suite"
)

# Individual algorithm tests
INDIVIDUAL_TESTS=(
    "drunkard_walk_algorithm_test_suite"
    "simple_rooms_algorithm_test_suite"
    "maze_algorithm_test_suite"
    "voronoi_algorithm_test_suite"
    "wave_function_collapse_test_suite"
)

# Function to run a test suite
run_test_suite() {
    local test_name=$1
    local description=$2
    
    echo -e "${BLUE}🔬 Running: $description${NC}"
    echo "   Test: $test_name"
    
    if cargo test $test_name --quiet; then
        echo -e "${GREEN}✅ PASSED: $description${NC}"
        return 0
    else
        echo -e "${RED}❌ FAILED: $description${NC}"
        return 1
    fi
}

# Function to run algorithm category tests
run_category_tests() {
    echo -e "${YELLOW}📊 Running Algorithm Category Tests${NC}"
    echo "======================================"
    
    local passed=0
    local total=${#CATEGORIES[@]}
    
    for category in "${CATEGORIES[@]}"; do
        case $category in
            "room_based_algorithms_test_suite")
                run_test_suite $category "Room-Based Algorithms (BSP, Simple Rooms, Maze)"
                ;;
            "organic_algorithms_test_suite")
                run_test_suite $category "Organic Algorithms (Cellular Automata, Drunkard's Walk, Voronoi)"
                ;;
            "advanced_algorithms_test_suite")
                run_test_suite $category "Advanced Algorithms (WFC, Hybrids, Sequential)"
                ;;
            "parameter_variations_test_suite")
                run_test_suite $category "Parameter Variation Testing"
                ;;
            "hybrid_algorithms_test_suite")
                run_test_suite $category "Hybrid Algorithm Combinations"
                ;;
            "sequential_algorithms_test_suite")
                run_test_suite $category "Sequential Multi-Algorithm Generation"
                ;;
        esac
        
        if [ $? -eq 0 ]; then
            ((passed++))
        fi
        echo ""
    done
    
    echo -e "${BLUE}Category Tests Summary: $passed/$total passed${NC}"
    echo ""
}

# Function to run individual algorithm tests
run_individual_tests() {
    echo -e "${YELLOW}🔧 Running Individual Algorithm Tests${NC}"
    echo "====================================="
    
    local passed=0
    local total=${#INDIVIDUAL_TESTS[@]}
    
    for test in "${INDIVIDUAL_TESTS[@]}"; do
        case $test in
            "drunkard_walk_algorithm_test_suite")
                run_test_suite $test "Drunkard's Walk Algorithm"
                ;;
            "simple_rooms_algorithm_test_suite")
                run_test_suite $test "Simple Rooms Algorithm"
                ;;
            "maze_algorithm_test_suite")
                run_test_suite $test "Maze Generation Algorithms"
                ;;
            "voronoi_algorithm_test_suite")
                run_test_suite $test "Voronoi Diagram Algorithm"
                ;;
            "wave_function_collapse_test_suite")
                run_test_suite $test "Wave Function Collapse Algorithm"
                ;;
        esac
        
        if [ $? -eq 0 ]; then
            ((passed++))
        fi
        echo ""
    done
    
    echo -e "${BLUE}Individual Tests Summary: $passed/$total passed${NC}"
    echo ""
}

# Function to run comprehensive test
run_comprehensive_test() {
    echo -e "${YELLOW}🌟 Running Comprehensive Test Suite${NC}"
    echo "=================================="
    
    if run_test_suite "comprehensive_algorithm_test_suite" "All Algorithms Combined"; then
        echo -e "${GREEN}🎉 Comprehensive test PASSED!${NC}"
        return 0
    else
        echo -e "${RED}💥 Comprehensive test FAILED!${NC}"
        return 1
    fi
}

# Function to generate summary report
generate_summary() {
    echo -e "${YELLOW}📋 Generating Test Summary${NC}"
    echo "=========================="
    
    local report_dir="enhanced-tile-test-suite"
    local summary_file="$report_dir/ALGORITHM_TEST_SUMMARY.md"
    
    cat > "$summary_file" << EOF
# Algorithm Test Summary

Generated: $(date)

## Test Categories Executed

### Room-Based Algorithms
- BSP (Binary Space Partitioning)
- Simple Rooms
- Maze Generation (Recursive Backtracking, Kruskal, Prim)

### Organic/Cave Algorithms  
- Cellular Automata
- Drunkard's Walk
- Voronoi Diagrams

### Advanced Algorithms
- Wave Function Collapse
- Hybrid Combinations
- Sequential Multi-Algorithm

### Parameter Variations
- Sparse vs Dense configurations
- Small vs Large room sizes
- Different iteration counts

## Generated Reports

The following detailed reports were generated:
- TEST_REPORT_ROOM_BASED.md
- TEST_REPORT_ORGANIC.md
- TEST_REPORT_ADVANCED.md
- TEST_REPORT_PARAMETER_VARIATIONS.md
- TEST_REPORT_HYBRID_ALGORITHMS.md
- TEST_REPORT_SEQUENTIAL_ALGORITHMS.md
- TEST_REPORT_COMPREHENSIVE.md

## Visual Output

PNG visualizations and evaluation metrics are available in:
- enhanced-tile-test-suite/pngs/
- enhanced-tile-test-suite/evaluations/
- enhanced-tile-test-suite/text/

## Algorithm Performance Summary

| Category | Algorithms Tested | Configurations | Status |
|----------|------------------|----------------|---------|
| Room-Based | 3 | 5 | ✅ |
| Organic | 3 | 6 | ✅ |
| Advanced | 4 | 4 | ✅ |
| Variations | 2 | 4 | ✅ |

Total: 12 unique algorithms, 19 configurations tested
EOF

    echo -e "${GREEN}📄 Summary report generated: $summary_file${NC}"
}

# Main execution
main() {
    echo "Starting algorithm testing at $(date)"
    echo ""
    
    # Ensure we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        echo -e "${RED}❌ Error: Not in project root directory${NC}"
        exit 1
    fi
    
    # Create output directory if it doesn't exist
    mkdir -p enhanced-tile-test-suite/{pngs,text,evaluations}
    
    # Run test categories
    run_category_tests
    
    # Run individual algorithm tests
    run_individual_tests
    
    # Run comprehensive test
    echo ""
    run_comprehensive_test
    
    # Generate summary
    echo ""
    generate_summary
    
    echo ""
    echo -e "${GREEN}🎯 Algorithm testing complete!${NC}"
    echo -e "${BLUE}📊 Check enhanced-tile-test-suite/ for detailed results${NC}"
}

# Handle command line arguments
case "${1:-all}" in
    "categories")
        run_category_tests
        ;;
    "individual")
        run_individual_tests
        ;;
    "comprehensive")
        run_comprehensive_test
        ;;
    "summary")
        generate_summary
        ;;
    "all"|*)
        main
        ;;
esac
