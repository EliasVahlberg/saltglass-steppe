# Enhanced Tile Generation Test Suite

A comprehensive test suite for validating the bracket-noise based tile generation pipeline in Saltglass Steppe.

## Overview

This test suite validates the enhanced tile generation system across different biomes, POI types, and terrain configurations. Each test generates:

- **Text output** (`.txt`) - ASCII representation of the generated terrain
- **PNG output** (`.png`) - Visual representation with color-coded tiles  
- **Evaluation** (`.json`) - Detailed metrics and constraint validation

## Directory Structure

```
enhanced-tile-test-suite/
├── configs/           # Test configuration files
├── text/             # Generated ASCII terrain files
├── pngs/             # Generated PNG visualizations
├── evaluations/      # Detailed evaluation JSON files
└── TEST_REPORT.md    # Comprehensive test report
```

## Test Configurations

| Config | Biome | POI | Terrain | Seed | Purpose |
|--------|-------|-----|---------|------|---------|
| `saltflat_basic.json` | Saltflat | None | Flat | 2001 | Basic saltflat generation |
| `desert_town.json` | Desert | Town | Dunes | 2002 | Town POI in desert biome |
| `ruins_landmark.json` | Ruins | Landmark | Canyon | 2003 | Quest landmark generation |
| `oasis_shrine.json` | Oasis | Shrine | Hills | 2004 | Shrine POI with biome modifiers |
| `scrubland_dungeon.json` | Scrubland | Dungeon | Mesa | 2005 | Dungeon POI connectivity test |
| `high_glass_density.json` | Saltflat | None | Flat | 2006 | High glass density scenario |

## Running Tests

### Individual Test
```bash
cargo run --bin tilegen-test-tool -- --config enhanced-tile-test-suite/configs/saltflat_basic.json
```

### Full Test Suite
```bash
cargo test enhanced_tile_generation_test_suite
```

## Evaluation Metrics

### Quality Score
Combined metric (0.0-1.0) based on:
- **Connectivity Ratio**: Proportion of reachable floor tiles
- **Floor Density**: Ratio of walkable terrain

### Constraints Validated
1. **Connectivity**: Minimum 80% of floor tiles must be reachable
2. **Floor Density**: Minimum 30% of tiles must be walkable terrain
3. **Accessibility**: Critical areas must remain accessible

### Tile Distribution
- **Floor**: Walkable terrain (various material types)
- **Wall**: Solid barriers (biome-specific materials)
- **Glass**: Dangerous glass terrain from storms
- **Other**: Special tiles (rare)

## Pipeline Stages Tested

1. **Noise Generation**: Multi-layer Perlin noise sampling with bracket-noise
2. **Terrain Classification**: Floor/wall/glass assignment based on thresholds
3. **Biome Modification**: Biome-specific material and density adjustments
4. **POI Integration**: Point of interest features and clearings
5. **Constraint Validation**: Connectivity and accessibility verification

## Configuration Format

```json
{
  "seed": 2001,
  "width": 250,
  "height": 110,
  "biome": "saltflat",
  "poi": null,
  "terrain_type": "flat",
  "use_bracket_noise": true,
  "output_layers": ["base_terrain"],
  "output_format": ["text", "png"],
  "enable_evaluation": true,
  "pipeline_stages": ["noise_generation", "terrain_classification"],
  "output_dir": "enhanced-tile-test-suite"
}
```

### Configuration Options

| Field | Type | Options | Description |
|-------|------|---------|-------------|
| `biome` | String | saltflat, desert, ruins, oasis, scrubland | Biome type for generation |
| `poi` | String/null | town, shrine, landmark, dungeon, null | Point of interest type |
| `terrain_type` | String | flat, dunes, hills, canyon, mesa | Base terrain configuration |
| `pipeline_stages` | Array | Various stage names | Specific pipeline phases to test |
| `output_format` | Array | text, png | Output file formats |
| `enable_evaluation` | Boolean | true/false | Generate evaluation metrics |

## Interpreting Results

### Successful Test
- ✅ **Status**: PASS
- **Quality Score**: > 0.5 (good), > 0.8 (excellent)
- **Constraints**: All critical constraints passed
- **Files Generated**: Text, PNG, and evaluation files created

### Quality Concerns
- ⚠️ **Low Quality Score**: < 0.5 indicates potential issues
- **Failed Constraints**: Connectivity or density below thresholds
- **High Glass Density**: May indicate biome modifier issues

### Common Issues
- **Low Connectivity**: Terrain too dense, needs threshold adjustment
- **Low Floor Density**: Too many walls, check noise parameters
- **Missing Files**: Generation failure, check error logs

## Adding New Tests

1. Create new config file in `configs/` directory
2. Use unique seed (2007+) to avoid conflicts
3. Add config name to test array in `tests/enhanced_tile_generation_suite.rs`
4. Run test suite to validate

## Integration with Development

This test suite serves as:
- **Regression Testing**: Ensures changes don't break terrain generation
- **Quality Assurance**: Validates terrain meets playability standards  
- **Performance Monitoring**: Tracks generation time and quality metrics
- **Documentation**: Visual examples of different biome/POI combinations

## Technical Details

- **Tool**: `tilegen-test-tool` (enhanced version)
- **Test Framework**: Rust `#[test]` with custom evaluation
- **Image Generation**: 4x scaled PNG with color-coded tiles
- **Determinism**: Same seed always produces identical results
- **Validation**: Automated constraint checking with detailed reporting

---

*Generated by Enhanced Tile Generation Test Suite v3.0*
