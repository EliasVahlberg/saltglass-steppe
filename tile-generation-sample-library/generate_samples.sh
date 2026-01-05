#!/bin/bash

# Tile Generation Sample Library Builder
# Generates all sample configurations and creates documentation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SAMPLE_LIB_DIR="$SCRIPT_DIR"
CONFIG_DIR="$SAMPLE_LIB_DIR/config"
TEXT_DIR="$SAMPLE_LIB_DIR/text"
PNG_DIR="$SAMPLE_LIB_DIR/pngs"
DOC_FILE="$SAMPLE_LIB_DIR/TILE_GEN_SAMPLE_LIB.md"

echo "Building Tile Generation Sample Library..."
echo "Project root: $PROJECT_ROOT"
echo "Sample library: $SAMPLE_LIB_DIR"

# Ensure tilegen-test-tool and dungeon-test-tool are built
echo "Building test tools..."
cd "$PROJECT_ROOT"
cargo build --bin tilegen-test-tool --release
cargo build --bin dungeon-test-tool --release

# Create directories
mkdir -p "$TEXT_DIR" "$PNG_DIR" "$SAMPLE_LIB_DIR/evaluations"

# Clear existing output files
echo "Clearing existing output files..."
rm -f "$TEXT_DIR"/*.txt "$PNG_DIR"/*.png "$DOC_FILE"

# Initialize documentation file
cat > "$DOC_FILE" << 'EOF'
# Tile Generation Sample Library

This library demonstrates the capabilities of the tile generation system across different biomes, terrain types, and Points of Interest (POI).

## Generation Parameters

Each sample is generated using a JSON configuration file that specifies:
- **Seed**: Deterministic random seed for reproducible results
- **Dimensions**: Width and height of the generated tile map
- **Biome**: Environmental theme (saltflat, desert, ruins, scrubland, oasis)
- **Terrain**: Topographical features (flat, hills, canyon, mesa, dunes)
- **POI**: Point of Interest structures (town, shrine, landmark, dungeon, or null)
- **Output Layers**: Which generation phases to include in output

## Samples

EOF

# Process each config file
for config_file in "$CONFIG_DIR"/*.json; do
    if [[ -f "$config_file" ]]; then
        config_name=$(basename "$config_file" .json)
        echo "Processing $config_name..."
        
        # Check if this is a dungeon config
        if [[ "$config_name" == dungeon_* ]]; then
            # Use dungeon-test-tool for dungeon configs
            cd "$PROJECT_ROOT"
            
            # Extract seed from config
            seed=$(jq -r '.seed // 12345' "$config_file")
            
            # Generate dungeon samples with different presets
            echo "Generating dungeon samples for $config_name (seed: $seed)..."
            
            # Generate text output
            ./target/release/dungeon-test-tool "$seed" default > "$TEXT_DIR/${config_name}_default.txt" 2>/dev/null || true
            ./target/release/dungeon-test-tool "$seed" small > "$TEXT_DIR/${config_name}_small.txt" 2>/dev/null || true
            ./target/release/dungeon-test-tool "$seed" organic > "$TEXT_DIR/${config_name}_organic.txt" 2>/dev/null || true
            ./target/release/dungeon-test-tool "$seed" structured > "$TEXT_DIR/${config_name}_structured.txt" 2>/dev/null || true
            
        else
            # Use tilegen-test-tool for regular configs
            cd "$PROJECT_ROOT"
            ./target/release/tilegen-test-tool --config "$config_file" --save-eval-report
            
            # Move generated files to proper locations
            mv "${config_name}"_*.txt "$TEXT_DIR/" 2>/dev/null || true
            mv "${config_name}"_*.png "$PNG_DIR/" 2>/dev/null || true
            mv "${config_name}"_evaluation.json "$SAMPLE_LIB_DIR/evaluations/" 2>/dev/null || true
        fi
    fi
done

# Generate documentation after all files are processed
for config_file in "$CONFIG_DIR"/*.json; do
    if [[ -f "$config_file" ]]; then
        config_name=$(basename "$config_file" .json)
        
        # Add to documentation
        echo "### $config_name" >> "$DOC_FILE"
        echo "" >> "$DOC_FILE"
        echo "**Configuration:**" >> "$DOC_FILE"
        echo '```json' >> "$DOC_FILE"
        cat "$config_file" >> "$DOC_FILE"
        echo '```' >> "$DOC_FILE"
        echo "" >> "$DOC_FILE"
        
        # Add PNG images for each layer
        for png_file in "$PNG_DIR"/${config_name}_*.png; do
            if [[ -f "$png_file" ]]; then
                layer_name=$(basename "$png_file" .png | sed "s/${config_name}_//")
                echo "**${layer_name} Layer:**" >> "$DOC_FILE"
                echo "![${config_name}_${layer_name}](pngs/$(basename "$png_file"))" >> "$DOC_FILE"
                echo "" >> "$DOC_FILE"
            fi
        done
        
        echo "---" >> "$DOC_FILE"
        echo "" >> "$DOC_FILE"
    fi
done

# Add footer to documentation
cat >> "$DOC_FILE" << 'EOF'

## Usage

To regenerate any sample:

```bash
cd /path/to/saltglass-steppe
cargo run --bin tilegen-test-tool --config tile-generation-sample-library/config/SAMPLE_NAME.json
```

To regenerate the entire library:

```bash
cd tile-generation-sample-library
./generate_samples.sh
```

## File Structure

```
tile-generation-sample-library/
├── config/           # JSON configuration files
├── text/            # ASCII text output files
├── pngs/            # PNG image output files
├── generate_samples.sh  # This generation script
└── TILE_GEN_SAMPLE_LIB.md  # This documentation
```

EOF

echo "Sample library generation complete!"
echo "Generated files:"
echo "  Configs: $(ls "$CONFIG_DIR"/*.json | wc -l)"
echo "  Text files: $(ls "$TEXT_DIR"/*.txt 2>/dev/null | wc -l || echo 0)"
echo "  PNG files: $(ls "$PNG_DIR"/*.png 2>/dev/null | wc -l || echo 0)"
echo "  Documentation: $DOC_FILE"
