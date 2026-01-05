#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$SCRIPT_DIR/config"
PNG_DIR="$SCRIPT_DIR/pngs"
DOC_FILE="$SCRIPT_DIR/TILE_GEN_SAMPLE_LIB.md"

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

# Generate documentation for each config
for config_file in "$CONFIG_DIR"/*.json; do
    if [[ -f "$config_file" ]]; then
        config_name=$(basename "$config_file" .json)
        
        echo "### $config_name" >> "$DOC_FILE"
        echo "" >> "$DOC_FILE"
        echo "**Configuration:**" >> "$DOC_FILE"
        echo '```json' >> "$DOC_FILE"
        cat "$config_file" >> "$DOC_FILE"
        echo '```' >> "$DOC_FILE"
        echo "" >> "$DOC_FILE"
        
        # Add PNG images for each layer (using seed from config)
        seed=$(grep '"seed"' "$config_file" | sed 's/.*: *\([0-9]*\).*/\1/')
        
        # Check for evaluation report
        eval_file="$SCRIPT_DIR/evaluations/${seed}_evaluation.json"
        if [[ -f "$eval_file" ]]; then
            quality_score=$(jq -r '.evaluation.quality_score' "$eval_file" 2>/dev/null || echo "N/A")
            passed_constraints=$(jq -r '.evaluation.passed_constraints' "$eval_file" 2>/dev/null || echo "N/A")
            total_constraints=$(jq -r '.evaluation.total_constraints' "$eval_file" 2>/dev/null || echo "N/A")
            connectivity=$(jq -r '.evaluation.connectivity.connectivity_ratio' "$eval_file" 2>/dev/null || echo "N/A")
            
            echo "**Quality Report:**" >> "$DOC_FILE"
            echo "- Quality Score: ${quality_score}/1.00" >> "$DOC_FILE"
            echo "- Constraints: ${passed_constraints}/${total_constraints} passed" >> "$DOC_FILE"
            echo "- Connectivity: ${connectivity}" >> "$DOC_FILE"
            echo "" >> "$DOC_FILE"
        fi
        
        for png_file in "$PNG_DIR"/${seed}_*.png; do
            if [[ -f "$png_file" ]]; then
                layer_name=$(basename "$png_file" .png | sed "s/${seed}_//")
                echo "**${layer_name} Layer:**" >> "$DOC_FILE"
                echo "![${config_name}_${layer_name}](pngs/$(basename "$png_file"))" >> "$DOC_FILE"
                echo "" >> "$DOC_FILE"
            fi
        done
        
        echo "---" >> "$DOC_FILE"
        echo "" >> "$DOC_FILE"
    fi
done

# Add footer
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

echo "Documentation generated: $DOC_FILE"
