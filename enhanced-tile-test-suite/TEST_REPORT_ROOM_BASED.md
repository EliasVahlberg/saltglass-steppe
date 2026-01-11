# Room-Based Generation Algorithms Test Report

**Generated:** 2026-01-10 16:40:57 UTC

## Test Results

| Test | Status | Quality Score | Constraints | PNG | Evaluation |
|------|--------|---------------|-------------|-----|------------|
| bsp_basic | ✅ PASS | 0.633 | 0/0 | ![bsp_basic](pngs/3000_base_terrain.png) | [JSON](evaluations/3000_evaluation.json) |
| bsp_small_dense | ✅ PASS | 0.605 | 1/2 | ![bsp_small_dense](pngs/5003_base_terrain.png) | [JSON](evaluations/5003_evaluation.json) |
| bsp_large_sparse | ✅ PASS | 0.653 | 2/2 | ![bsp_large_sparse](pngs/5004_base_terrain.png) | [JSON](evaluations/5004_evaluation.json) |
| simple_rooms_basic | ✅ PASS | 0.665 | 2/2 | ![simple_rooms_basic](pngs/3002_base_terrain.png) | [JSON](evaluations/3002_evaluation.json) |
| maze_basic | ✅ PASS | 0.646 | 1/2 | ![maze_basic](pngs/3003_base_terrain.png) | [JSON](evaluations/3003_evaluation.json) |

## Room-Based Generation Algorithms Algorithm Details

This test suite validates specific algorithm implementations:

### Algorithm-Specific Testing
- **Purpose**: Validate specific generation methods
- **Method**: Targeted test configurations
- **Validation**: Algorithm-specific constraints

## Quality Metrics

- **Quality Score**: Algorithm-specific quality measurement (0.0-1.0)
- **Constraints**: Algorithm-specific validation checks
- **Visual Output**: PNG files show generated structures
