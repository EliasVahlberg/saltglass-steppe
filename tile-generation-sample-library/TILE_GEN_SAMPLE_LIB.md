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

### desert_minimal

**Configuration:**
```json
{
  "seed": 6001,
  "width": 80,
  "height": 40,
  "biome": "desert",
  "terrain_type": "flat",
  "poi_type": null,
  "quest_tile": false,
  "enable_microstructures": false,
  "enable_spawns": true,
  "enable_loot": false,
  "enable_narrative": false,
  "custom_biome_attributes": {},
  "output_layers": ["base_terrain", "entity_spawns"],
  "output_format": ["text", "png"],
  "output_dir": "tile-generation-sample-library"
}
```

**Quality Report:**
- Quality Score: 0.7393910884857178/1.00
- Constraints: 2/3 passed
- Connectivity: 0.9318110346794128

**base_terrain Layer:**
![desert_minimal_base_terrain](pngs/6001_base_terrain.png)

**entity_spawns Layer:**
![desert_minimal_entity_spawns](pngs/6001_entity_spawns.png)

---

### desert_town

**Configuration:**
```json
{
  "seed": 2001,
  "width": 80,
  "height": 40,
  "biome": "desert",
  "terrain_type": "hills",
  "poi_type": "town",
  "quest_tile": false,
  "enable_microstructures": true,
  "enable_spawns": true,
  "enable_loot": true,
  "enable_narrative": false,
  "custom_biome_attributes": {},
  "output_layers": ["base_terrain", "poi_structures", "microstructures", "entity_spawns", "loot_placement"],
  "output_format": ["text", "png"],
  "output_dir": "tile-generation-sample-library"
}
```

**Quality Report:**
- Quality Score: 0.895134687423706/1.00
- Constraints: 3/3 passed
- Connectivity: 0.8628367185592651

**base_terrain Layer:**
![desert_town_base_terrain](pngs/2001_base_terrain.png)

**entity_spawns Layer:**
![desert_town_entity_spawns](pngs/2001_entity_spawns.png)

**loot_placement Layer:**
![desert_town_loot_placement](pngs/2001_loot_placement.png)

**microstructures Layer:**
![desert_town_microstructures](pngs/2001_microstructures.png)

**poi_structures Layer:**
![desert_town_poi_structures](pngs/2001_poi_structures.png)

---

### oasis_landmark

**Configuration:**
```json
{
  "seed": 5001,
  "width": 120,
  "height": 60,
  "biome": "oasis",
  "terrain_type": "dunes",
  "poi_type": "landmark",
  "quest_tile": false,
  "enable_microstructures": true,
  "enable_spawns": true,
  "enable_loot": true,
  "enable_narrative": false,
  "custom_biome_attributes": {},
  "output_layers": ["base_terrain", "poi_structures", "microstructures", "entity_spawns", "loot_placement"],
  "output_format": ["text", "png"],
  "output_dir": "tile-generation-sample-library"
}
```

**Quality Report:**
- Quality Score: 0.9397025108337402/1.00
- Constraints: 3/3 passed
- Connectivity: 0.9742562174797058

**base_terrain Layer:**
![oasis_landmark_base_terrain](pngs/5001_base_terrain.png)

**entity_spawns Layer:**
![oasis_landmark_entity_spawns](pngs/5001_entity_spawns.png)

**loot_placement Layer:**
![oasis_landmark_loot_placement](pngs/5001_loot_placement.png)

**microstructures Layer:**
![oasis_landmark_microstructures](pngs/5001_microstructures.png)

**poi_structures Layer:**
![oasis_landmark_poi_structures](pngs/5001_poi_structures.png)

---

### ruins_shrine

**Configuration:**
```json
{
  "seed": 3001,
  "width": 80,
  "height": 40,
  "biome": "ruins",
  "terrain_type": "canyon",
  "poi_type": "shrine",
  "quest_tile": false,
  "enable_microstructures": true,
  "enable_spawns": true,
  "enable_loot": true,
  "enable_narrative": false,
  "custom_biome_attributes": {},
  "output_layers": ["base_terrain", "poi_structures", "microstructures", "entity_spawns", "loot_placement"],
  "output_format": ["text", "png"],
  "output_dir": "tile-generation-sample-library"
}
```

**Quality Report:**
- Quality Score: 0.8999806046485901/1.00
- Constraints: 3/3 passed
- Connectivity: 0.9999514222145081

**base_terrain Layer:**
![ruins_shrine_base_terrain](pngs/3001_base_terrain.png)

**entity_spawns Layer:**
![ruins_shrine_entity_spawns](pngs/3001_entity_spawns.png)

**loot_placement Layer:**
![ruins_shrine_loot_placement](pngs/3001_loot_placement.png)

**microstructures Layer:**
![ruins_shrine_microstructures](pngs/3001_microstructures.png)

**poi_structures Layer:**
![ruins_shrine_poi_structures](pngs/3001_poi_structures.png)

---

### saltflat_basic

**Configuration:**
```json
{
  "seed": 1001,
  "width": 80,
  "height": 40,
  "biome": "saltflat",
  "terrain_type": "flat",
  "poi_type": null,
  "quest_tile": false,
  "enable_microstructures": true,
  "enable_spawns": true,
  "enable_loot": true,
  "enable_narrative": false,
  "custom_biome_attributes": {},
  "output_layers": ["base_terrain", "microstructures", "entity_spawns", "loot_placement"],
  "output_format": ["text", "png"],
  "output_dir": "tile-generation-sample-library"
}
```

**Quality Report:**
- Quality Score: 0.9282537698745728/1.00
- Constraints: 3/3 passed
- Connectivity: 0.9456344246864319

**base_terrain Layer:**
![saltflat_basic_base_terrain](pngs/1001_base_terrain.png)

**entity_spawns Layer:**
![saltflat_basic_entity_spawns](pngs/1001_entity_spawns.png)

**loot_placement Layer:**
![saltflat_basic_loot_placement](pngs/1001_loot_placement.png)

**microstructures Layer:**
![saltflat_basic_microstructures](pngs/1001_microstructures.png)

---

### scrubland_dungeon

**Configuration:**
```json
{
  "seed": 4001,
  "width": 80,
  "height": 40,
  "biome": "scrubland",
  "terrain_type": "mesa",
  "poi_type": "dungeon",
  "quest_tile": false,
  "enable_microstructures": true,
  "enable_spawns": true,
  "enable_loot": true,
  "enable_narrative": false,
  "custom_biome_attributes": {},
  "output_layers": ["base_terrain", "poi_structures", "microstructures", "entity_spawns", "loot_placement"],
  "output_format": ["text", "png"],
  "output_dir": "tile-generation-sample-library"
}
```

**Quality Report:**
- Quality Score: 0.9801124930381775/1.00
- Constraints: 3/3 passed
- Connectivity: 0.9502813220024109

**base_terrain Layer:**
![scrubland_dungeon_base_terrain](pngs/4001_base_terrain.png)

**entity_spawns Layer:**
![scrubland_dungeon_entity_spawns](pngs/4001_entity_spawns.png)

**loot_placement Layer:**
![scrubland_dungeon_loot_placement](pngs/4001_loot_placement.png)

**microstructures Layer:**
![scrubland_dungeon_microstructures](pngs/4001_microstructures.png)

**poi_structures Layer:**
![scrubland_dungeon_poi_structures](pngs/4001_poi_structures.png)

---


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
├── evaluations/     # Quality evaluation JSON files
├── generate_samples.sh  # This generation script
└── TILE_GEN_SAMPLE_LIB.md  # This documentation
```

