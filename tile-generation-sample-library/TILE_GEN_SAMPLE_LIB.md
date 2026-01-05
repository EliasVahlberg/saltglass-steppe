# Tile Generation Sample Library

This library demonstrates the capabilities of the tile generation system with deterministic, JSON-configured generation.

## Generation Parameters

Each sample is generated using a JSON configuration file that specifies:
- **Seed**: Deterministic random seed for reproducible results
- **Dimensions**: Width and height of the generated tile map
- **Output Directory**: Where to save generated files

## Samples

### BSP Test (Seed: 9001)
- **Config**: `bsp_test.json`
- **Dimensions**: 60x30
- **Description**: Test BSP algorithm for room generation

### Cellular Automata Test (Seed: 9002)
- **Config**: `cellular_automata_test.json`
- **Dimensions**: 80x40
- **Description**: Test cellular automata for cave generation

### Desert Minimal (Seed: 6001)
- **Config**: `desert_minimal.json`
- **Dimensions**: 80x40
- **Description**: Minimal desert terrain generation

### Desert Town (Seed: 2001)
- **Config**: `desert_town.json`
- **Dimensions**: 80x40
- **Description**: Desert terrain with town POI

### Dungeon Balanced Hybrid (Seed: 7003)
- **Config**: `dungeon_balanced_hybrid.json`
- **Dimensions**: 80x40
- **Description**: Balanced hybrid dungeon configuration

### Dungeon BSP Small (Seed: 7001)
- **Config**: `dungeon_bsp_small.json`
- **Dimensions**: 80x40
- **Description**: Small dungeon with clear BSP room structure

### Dungeon Organic Cave (Seed: 7002)
- **Config**: `dungeon_organic_cave.json`
- **Dimensions**: 80x40
- **Description**: Organic cave dungeon with heavy CA influence

### Modular Perlin Enhanced (Seed: 8001)
- **Config**: `modular_perlin_enhanced.json`
- **Dimensions**: 80x40
- **Description**: Enhanced Perlin noise generation with modular architecture

### Oasis Landmark (Seed: 5001)
- **Config**: `oasis_landmark.json`
- **Dimensions**: 80x40
- **Description**: Oasis terrain with landmark POI

### Registry Multipass (Seed: 8002)
- **Config**: `registry_multipass.json`
- **Dimensions**: 80x40
- **Description**: Multi-pass generation with algorithm registry

### Ruins Shrine (Seed: 3001)
- **Config**: `ruins_shrine.json`
- **Dimensions**: 80x40
- **Description**: Ruins terrain with shrine POI

### Saltflat Basic (Seed: 1001)
- **Config**: `saltflat_basic.json`
- **Dimensions**: 80x40
- **Description**: Basic saltflat terrain generation

### Scrubland Dungeon (Seed: 4001)
- **Config**: `scrubland_dungeon.json`
- **Dimensions**: 80x40
- **Description**: Scrubland terrain with dungeon POI

## Usage

Generate samples using the tilegen-test-tool:

```bash
# Generate specific sample
./target/release/tilegen-test-tool --config tile-generation-sample-library/config/desert_minimal.json

# Generate all samples
cd tile-generation-sample-library && ./generate_samples.sh
```

## Output Files

- **Text files**: Located in `text/` directory with format `{seed}_base_terrain.txt`
- **PNG files**: Located in `pngs/` directory (if enabled)

## Legend

- `.` = Floor
- `#` = Wall  
- `*` = Glass
