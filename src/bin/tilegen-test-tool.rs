use clap::{Arg, Command};
use saltglass_steppe::game::world_map::Biome;
use saltglass_steppe::game::map::{Map, Tile};
use saltglass_steppe::tilegen_tools::*;
use saltglass_steppe::game::generation::{
    algorithm::{AlgorithmContext, AlgorithmParameters, ParameterValue, GenerationAlgorithm},
    registry::{get_global_registry, AlgorithmConfig, PerformanceProfile, SpeedCategory, MemoryCategory, CpuCategory, SupportedLayers},
    algorithms::perlin_noise::PerlinNoiseAlgorithm,
};
use rand_chacha::ChaCha8Rng;
use rand::{SeedableRng, Rng};
use std::collections::HashMap;
use std::fs;
use image::{ImageBuffer, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use chrono;

mod biome_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(biome: &Biome, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match biome {
            Biome::Saltflat => "saltflat",
            Biome::Desert => "desert",
            Biome::Ruins => "ruins",
            Biome::Scrubland => "scrubland",
            Biome::Oasis => "oasis",
        };
        serializer.serialize_str(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Biome, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "saltflat" => Ok(Biome::Saltflat),
            "desert" => Ok(Biome::Desert),
            "ruins" => Ok(Biome::Ruins),
            "scrubland" => Ok(Biome::Scrubland),
            "oasis" => Ok(Biome::Oasis),
            _ => Ok(Biome::Saltflat),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileTestConfig {
    pub seed: u64,
    pub width: i32,
    pub height: i32,
    #[serde(with = "biome_serde")]
    pub biome: Biome,
    pub terrain_type: String,
    pub poi_type: Option<String>,
    pub quest_tile: bool,
    pub enable_microstructures: bool,
    pub enable_spawns: bool,
    pub enable_loot: bool,
    pub enable_narrative: bool,
    pub custom_biome_attributes: HashMap<String, f64>,
    pub output_layers: Vec<String>,
    pub output_format: Vec<String>, // "text", "png", or both
    pub output_dir: String,
    pub save_eval_report: Option<bool>, // New field for evaluation report
}

impl TileTestConfig {
    pub fn from_json_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: TileTestConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn to_json_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for TileTestConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            width: 80,
            height: 40,
            biome: Biome::Saltflat,
            terrain_type: "flat".to_string(),
            poi_type: None,
            quest_tile: false,
            enable_microstructures: true,
            enable_spawns: true,
            enable_loot: true,
            enable_narrative: true,
            custom_biome_attributes: HashMap::new(),
            output_layers: vec!["all".to_string()],
            output_format: vec!["text".to_string()],
            output_dir: "tile_test_output".to_string(),
            save_eval_report: Some(false),
        }
    }
}

pub struct LayerSnapshot {
    pub name: String,
    pub description: String,
    pub map: Map,
    pub entities: Vec<String>, // Entity descriptions for this layer
    pub metadata: HashMap<String, String>,
}

pub struct LayeredTileGeneration {
    config: TileTestConfig,
    snapshots: Vec<LayerSnapshot>,
    rng: ChaCha8Rng,
}

impl LayeredTileGeneration {
    pub fn new(config: TileTestConfig) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(config.seed);
        Self {
            config,
            snapshots: Vec::new(),
            rng,
        }
    }
    
    fn copy_map(original: &Map) -> Map {
        Map {
            tiles: original.tiles.clone(),
            width: original.width,
            height: original.height,
            lights: original.lights.clone(),
            inscriptions: original.inscriptions.clone(),
            area_description: original.area_description.clone(),
            metadata: original.metadata.clone(),
        }
    }

    pub fn generate_with_layers(&mut self) -> &[LayerSnapshot] {
        println!("🎯 Starting layered tile generation...");
        println!("   Seed: {}", self.config.seed);
        println!("   Size: {}x{}", self.config.width, self.config.height);
        println!("   Biome: {:?}", self.config.biome);
        
        let should_generate_all = self.config.output_layers.contains(&"all".to_string());
        let needs_base_terrain = should_generate_all || 
            self.config.output_layers.iter().any(|layer| layer != "base_terrain");
        
        // Layer 1: Base Terrain (always generate if other layers need it)
        if should_generate_all || self.config.output_layers.contains(&"base_terrain".to_string()) || needs_base_terrain {
            self.generate_base_terrain();
        }
        
        // Layer 2: POI Structures
        if (should_generate_all || self.config.output_layers.contains(&"poi_structures".to_string())) 
            && self.config.poi_type.is_some() {
            self.generate_poi_structures();
        }
        
        // Layer 3: Microstructures
        if (should_generate_all || self.config.output_layers.contains(&"microstructures".to_string())) 
            && self.config.enable_microstructures {
            self.generate_microstructures();
        }
        
        // Layer 4: Entity Spawns
        if (should_generate_all || self.config.output_layers.contains(&"entity_spawns".to_string())) 
            && self.config.enable_spawns {
            self.generate_entity_spawns();
        }
        
        // Layer 5: Loot Placement
        if (should_generate_all || self.config.output_layers.contains(&"loot_placement".to_string())) 
            && self.config.enable_loot {
            self.generate_loot_placement();
        }
        
        // Layer 6: Narrative Elements
        if (should_generate_all || self.config.output_layers.contains(&"narrative_elements".to_string())) 
            && self.config.enable_narrative {
            self.generate_narrative_elements();
        }
        
        self.snapshots.as_slice()
    }

    fn generate_base_terrain(&mut self) {
        println!("🔧 Using new algorithm architecture...");
        
        // Register Perlin noise algorithm
        let perlin_algorithm = std::sync::Arc::new(PerlinNoiseAlgorithm::new());
        let config = AlgorithmConfig {
            id: "perlin_noise".to_string(),
            name: "Perlin Noise".to_string(),
            description: "Multi-octave Perlin noise terrain generation".to_string(),
            default_parameters: AlgorithmParameters::new(),
            category: "terrain".to_string(),
            performance_profile: PerformanceProfile {
                speed: SpeedCategory::Fast,
                memory_usage: MemoryCategory::Low,
                cpu_intensity: CpuCategory::Light,
                parallelizable: false,
            },
            supported_layers: SupportedLayers {
                input_layers: vec![],
                output_layers: vec!["heightmap".to_string()],
                required_inputs: vec![],
                optional_inputs: vec![],
            },
        };
        
        let registry = get_global_registry();
        registry.register_algorithm(perlin_algorithm.clone(), config)
            .expect("Failed to register Perlin noise algorithm");
        
        // Create algorithm context
        let mut parameters = AlgorithmParameters::new();
        parameters.set("octaves".to_string(), ParameterValue::Integer(4));
        parameters.set("frequency".to_string(), ParameterValue::Float(0.1));
        parameters.set("amplitude".to_string(), ParameterValue::Float(1.0));
        parameters.set("lacunarity".to_string(), ParameterValue::Float(2.0));
        parameters.set("persistence".to_string(), ParameterValue::Float(0.5));
        
        let context = AlgorithmContext {
            width: self.config.width as usize,
            height: self.config.height as usize,
            seed: self.config.seed,
            biome: format!("{:?}", self.config.biome).to_lowercase(),
            poi_type: self.config.poi_type.clone(),
            input_layers: HashMap::new(),
            parameters,
            quest_ids: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // Generate using new architecture
        let result = perlin_algorithm.as_ref().generate(&context)
            .expect("Failed to generate terrain");
        
        // Convert result to Map (simplified conversion)
        let mut tiles = vec![Tile::Floor { id: "ground".to_string() }; (self.config.width * self.config.height) as usize];
        
        if let Some(heightmap) = result.output_layers.get("heightmap") {
            for y in 0..self.config.height as usize {
                for x in 0..self.config.width as usize {
                    if y < heightmap.data.len() && x < heightmap.data[0].len() {
                        let height_value = heightmap.data[y][x];
                    let tile = if height_value > 0.3 {
                        Tile::Wall { id: "stone".to_string(), hp: 100 }
                    } else if height_value > 0.1 {
                        Tile::Floor { id: "ground".to_string() }
                    } else {
                        Tile::Glass
                    };
                    tiles[y * self.config.width as usize + x] = tile;
                    }
                }
            }
        }
        
        let map = Map {
            tiles,
            width: self.config.width as usize,
            height: self.config.height as usize,
            lights: Vec::new(),
            inscriptions: Vec::new(),
            area_description: None,
            metadata: HashMap::new(),
        };
        
        let mut metadata = HashMap::new();
        metadata.insert("biome".to_string(), format!("{:?}", self.config.biome));
        metadata.insert("terrain_type".to_string(), self.config.terrain_type.clone());
        metadata.insert("algorithm".to_string(), "perlin_noise".to_string());
        
        self.snapshots.push(LayerSnapshot {
            name: "base_terrain".to_string(),
            description: "Base terrain generation using new algorithm architecture".to_string(),
            map,
            entities: Vec::new(),
            metadata,
        });
        
        println!("✅ Layer 1: Base terrain generated using new architecture");
    }

    fn generate_poi_structures(&mut self) {
        let mut map = Self::copy_map(&self.snapshots.last().unwrap().map);
        let mut entities = Vec::new();
        
        if let Some(poi_type) = &self.config.poi_type {
            let center_x = self.config.width / 2;
            let center_y = self.config.height / 2;
            
            match poi_type.as_str() {
                "town" => {
                    // Create central clearing for town
                    for dy in -3..=3 {
                        for dx in -3..=3 {
                            let x = (center_x as i32 + dx) as usize;
                            let y = (center_y as i32 + dy) as usize;
                            if x < map.width && y < map.height {
                                map.tiles[y * map.width + x] = Tile::Floor { id: "cobblestone".to_string() };
                            }
                        }
                    }
                    entities.push("Town center clearing".to_string());
                }
                "shrine" => {
                    // Create shrine structure with special floor
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let x = (center_x as i32 + dx) as usize;
                            let y = (center_y as i32 + dy) as usize;
                            if x < map.width && y < map.height {
                                map.tiles[y * map.width + x] = Tile::Floor { id: "marble".to_string() };
                            }
                        }
                    }
                    entities.push("Shrine clearing".to_string());
                }
                "dungeon" => {
                    // Add some chamber-like structures
                    for dy in -4..=4 {
                        for dx in -4..=4 {
                            let x = (center_x as i32 + dx) as usize;
                            let y = (center_y as i32 + dy) as usize;
                            if x < map.width && y < map.height && (dx.abs() == 4 || dy.abs() == 4) {
                                map.tiles[y * map.width + x] = Tile::Wall { id: "stone_brick".to_string(), hp: 50 };
                            }
                        }
                    }
                    entities.push("Dungeon chambers".to_string());
                }
                _ => {}
            }
        }
        
        let mut metadata = HashMap::new();
        metadata.insert("poi_type".to_string(), self.config.poi_type.clone().unwrap_or_default());
        
        self.snapshots.push(LayerSnapshot {
            name: "poi_structures".to_string(),
            description: "Point of Interest structures and clearings".to_string(),
            map,
            entities,
            metadata,
        });
        
        println!("✅ Layer 2: POI structures generated");
    }

    fn generate_microstructures(&mut self) {
        let mut map = Self::copy_map(&self.snapshots.last().unwrap().map);
        let entities = vec!["Microstructure placements".to_string()];
        
        // Add some scattered microstructures (small walls/obstacles)
        let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed + 1);
        for _ in 0..15 {
            let x = rng.gen_range(0..map.width);
            let y = rng.gen_range(0..map.height);
            if let Some(tile) = map.tiles.get_mut(y * map.width + x) {
                if matches!(tile, Tile::Floor { .. }) {
                    *tile = Tile::Wall { id: "rubble".to_string(), hp: 5 };
                }
            }
        }
        
        self.snapshots.push(LayerSnapshot {
            name: "microstructures".to_string(),
            description: "Small structures and environmental details".to_string(),
            map,
            entities,
            metadata: HashMap::new(),
        });
        
        println!("✅ Layer 3: Microstructures generated");
    }

    fn generate_entity_spawns(&mut self) {
        let mut map = Self::copy_map(&self.snapshots.last().unwrap().map);
        let entities = vec![
            "Enemy spawn points".to_string(),
            "NPC placements".to_string(),
        ];
        
        // Add some spawn markers (using Glare tiles as visual indicators)
        let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed + 2);
        for _ in 0..8 {
            let x = rng.gen_range(0..map.width);
            let y = rng.gen_range(0..map.height);
            if let Some(tile) = map.tiles.get_mut(y * map.width + x) {
                if matches!(tile, Tile::Floor { .. }) {
                    *tile = Tile::Glare; // Use glare as spawn point marker
                }
            }
        }
        
        self.snapshots.push(LayerSnapshot {
            name: "entity_spawns".to_string(),
            description: "Enemy and NPC spawn locations".to_string(),
            map,
            entities,
            metadata: HashMap::new(),
        });
        
        println!("✅ Layer 4: Entity spawns generated");
    }

    fn generate_loot_placement(&mut self) {
        let mut map = Self::copy_map(&self.snapshots.last().unwrap().map);
        let entities = vec!["Loot containers and items".to_string()];
        
        // Add some additional glass shards as loot indicators
        let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed + 3);
        for _ in 0..5 {
            let x = rng.gen_range(0..map.width);
            let y = rng.gen_range(0..map.height);
            if let Some(tile) = map.tiles.get_mut(y * map.width + x) {
                if matches!(tile, Tile::Floor { .. }) {
                    *tile = Tile::Glass; // Add glass as loot marker
                }
            }
        }
        
        self.snapshots.push(LayerSnapshot {
            name: "loot_placement".to_string(),
            description: "Item and container placement".to_string(),
            map,
            entities,
            metadata: HashMap::new(),
        });
        
        println!("✅ Layer 5: Loot placement generated");
    }

    fn generate_narrative_elements(&mut self) {
        let map = Self::copy_map(&self.snapshots.last().unwrap().map);
        let entities = vec!["Narrative fragments and story elements".to_string()];
        
        self.snapshots.push(LayerSnapshot {
            name: "narrative_elements".to_string(),
            description: "Story fragments and atmospheric elements".to_string(),
            map,
            entities,
            metadata: HashMap::new(),
        });
        
        println!("✅ Layer 6: Narrative elements generated");
    }
}

pub struct TileVisualizer;

impl TileVisualizer {
    pub fn render_text_layer(snapshot: &LayerSnapshot) -> String {
        let mut output = String::new();
        output.push_str(&format!("=== {} ===\n", snapshot.name.to_uppercase()));
        output.push_str(&format!("{}\n\n", snapshot.description));
        
        // Render map
        for y in 0..snapshot.map.height {
            for x in 0..snapshot.map.width {
                let glyph = if let Some(tile) = snapshot.map.tiles.get(y * snapshot.map.width + x) {
                    tile.glyph()
                } else {
                    ' '
                };
                output.push(glyph);
            }
            output.push('\n');
        }
        
        // Add entity information
        if !snapshot.entities.is_empty() {
            output.push_str("\nEntities:\n");
            for entity in &snapshot.entities {
                output.push_str(&format!("- {}\n", entity));
            }
        }
        
        // Add metadata
        if !snapshot.metadata.is_empty() {
            output.push_str("\nMetadata:\n");
            for (key, value) in &snapshot.metadata {
                output.push_str(&format!("- {}: {}\n", key, value));
            }
        }
        
        output.push_str("\n");
        output
    }
    
    pub fn render_png_layer(snapshot: &LayerSnapshot, scale: u32) -> RgbImage {
        let width = snapshot.map.width as u32 * scale;
        let height = snapshot.map.height as u32 * scale;
        let mut img = ImageBuffer::new(width, height);
        
        for y in 0..snapshot.map.height {
            for x in 0..snapshot.map.width {
                let color = if let Some(tile) = snapshot.map.tiles.get(y * snapshot.map.width + x) {
                    match tile {
                        Tile::Wall { .. } => Rgb([64, 64, 64]),      // Dark gray
                        Tile::Floor { .. } => Rgb([200, 200, 200]), // Light gray
                        Tile::Glass => Rgb([0, 255, 255]),          // Cyan
                        Tile::Glare => Rgb([255, 255, 0]),          // Yellow
                        _ => Rgb([128, 128, 128]),                   // Medium gray for other types
                    }
                } else {
                    Rgb([0, 0, 0]) // Black
                };
                
                // Fill scaled pixel block
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x as u32 * scale + dx;
                        let py = y as u32 * scale + dy;
                        if px < width && py < height {
                            img.put_pixel(px, py, color);
                        }
                    }
                }
            }
        }
        
        img
    }
    
    pub fn save_text_output(snapshots: &[LayerSnapshot], output_dir: &str, config: &TileTestConfig) {
        std::fs::create_dir_all(output_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create output directory: {}", e);
        });
        
        let should_output_all = config.output_layers.contains(&"all".to_string());
        
        for snapshot in snapshots {
            if should_output_all || config.output_layers.contains(&snapshot.name) {
                let filename = format!("{}/{}_{}.txt", output_dir, config.seed, snapshot.name);
                let content = Self::render_text_layer(snapshot);
                
                std::fs::write(&filename, content).unwrap_or_else(|e| {
                    eprintln!("Failed to write {}: {}", filename, e);
                });
                
                println!("📄 Saved: {}", filename);
            }
        }
    }
    
    pub fn save_png_output(snapshots: &[LayerSnapshot], output_dir: &str, config: &TileTestConfig) {
        std::fs::create_dir_all(output_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create output directory: {}", e);
        });
        
        let scale = 4; // 4x4 pixels per tile
        let should_output_all = config.output_layers.contains(&"all".to_string());
        
        for snapshot in snapshots {
            if should_output_all || config.output_layers.contains(&snapshot.name) {
                let filename = format!("{}/{}_{}.png", output_dir, config.seed, snapshot.name);
                let img = Self::render_png_layer(snapshot, scale);
                
                img.save(&filename).unwrap_or_else(|e| {
                    eprintln!("Failed to save PNG {}: {}", filename, e);
                });
                
                println!("🖼️  Saved: {}", filename);
            }
        }
    }
}

fn main() {
    let matches = Command::new("Tile Generation Test Tool")
        .version("1.0")
        .about("Comprehensive tile generation testing with layer visualization")
        .after_help("EXAMPLES:
    # Generate basic tile with default settings
    tilegen-test-tool

    # Generate desert town with PNG output
    tilegen-test-tool --biome desert --poi town --format png

    # Load configuration from JSON file
    tilegen-test-tool --config my_config.json

    # Save current configuration to JSON file
    tilegen-test-tool --biome ruins --poi shrine --save-config my_config.json

    # Generate only specific layers
    tilegen-test-tool --layers base_terrain,poi_structures --format both

    # Large map with shrine POI
    tilegen-test-tool --width 120 --height 60 --poi shrine --terrain hills")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("CONFIG_FILE")
            .help("Load configuration from JSON file"))
        .arg(Arg::new("save-config")
            .long("save-config")
            .value_name("CONFIG_FILE")
            .help("Save current configuration to JSON file and exit"))
        .arg(Arg::new("seed")
            .short('s')
            .long("seed")
            .value_name("SEED")
            .help("Random seed for generation")
            .default_value("12345"))
        .arg(Arg::new("width")
            .short('w')
            .long("width")
            .value_name("WIDTH")
            .help("Tile width")
            .default_value("80"))
        .arg(Arg::new("height")
            .long("height")
            .value_name("HEIGHT")
            .help("Tile height")
            .default_value("40"))
        .arg(Arg::new("biome")
            .short('b')
            .long("biome")
            .value_name("BIOME")
            .help("Biome type (saltflat, desert, ruins, scrubland, oasis)")
            .default_value("saltflat"))
        .arg(Arg::new("terrain")
            .short('t')
            .long("terrain")
            .value_name("TERRAIN")
            .help("Terrain type (flat, hills, canyon, mesa, dunes)")
            .default_value("flat"))
        .arg(Arg::new("poi")
            .short('p')
            .long("poi")
            .value_name("POI")
            .help("Point of Interest type (town, shrine, landmark, dungeon)"))
        .arg(Arg::new("quest-tile")
            .long("quest-tile")
            .help("Generate as quest tile")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-microstructures")
            .long("no-microstructures")
            .help("Disable microstructure generation")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-spawns")
            .long("no-spawns")
            .help("Disable entity spawn generation")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-loot")
            .long("no-loot")
            .help("Disable loot placement generation")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("save-eval-report")
            .long("save-eval-report")
            .help("Save evaluation report as JSON file")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("no-narrative")
            .long("no-narrative")
            .help("Disable narrative elements")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("output-dir")
            .short('o')
            .long("output-dir")
            .value_name("DIR")
            .help("Output directory for generated files")
            .default_value("tile_test_output"))
        .arg(Arg::new("layers")
            .short('l')
            .long("layers")
            .value_name("LAYERS")
            .help("Comma-separated list of layers to output (all, base_terrain, poi_structures, etc.)")
            .default_value("all"))
        .arg(Arg::new("format")
            .short('f')
            .long("format")
            .value_name("FORMAT")
            .help("Output format: text, png, or both")
            .default_value("text"))
        .get_matches();

    let mut config = if let Some(config_file) = matches.get_one::<String>("config") {
        TileTestConfig::from_json_file(config_file).unwrap_or_else(|e| {
            eprintln!("Failed to load config file {}: {}", config_file, e);
            std::process::exit(1);
        })
    } else {
        TileTestConfig::default()
    };
    
    // Parse arguments (only override config if explicitly provided)
    if let Some(seed_str) = matches.get_one::<String>("seed") {
        if matches.value_source("seed") == Some(clap::parser::ValueSource::CommandLine) {
            config.seed = seed_str.parse().unwrap_or(12345);
        }
    }
    if let Some(width_str) = matches.get_one::<String>("width") {
        if matches.value_source("width") == Some(clap::parser::ValueSource::CommandLine) {
            config.width = width_str.parse().unwrap_or(80);
        }
    }
    if let Some(height_str) = matches.get_one::<String>("height") {
        if matches.value_source("height") == Some(clap::parser::ValueSource::CommandLine) {
            config.height = height_str.parse().unwrap_or(40);
        }
    }
    if let Some(terrain_str) = matches.get_one::<String>("terrain") {
        if matches.value_source("terrain") == Some(clap::parser::ValueSource::CommandLine) {
            config.terrain_type = terrain_str.clone();
        }
    }
    if let Some(poi_str) = matches.get_one::<String>("poi") {
        config.poi_type = Some(poi_str.clone());
    }
    if matches.get_flag("quest-tile") {
        config.quest_tile = true;
    }
    if matches.get_flag("no-microstructures") {
        config.enable_microstructures = false;
    }
    if matches.get_flag("no-spawns") {
        config.enable_spawns = false;
    }
    if matches.get_flag("no-loot") {
        config.enable_loot = false;
    }
    if matches.get_flag("no-narrative") {
        config.enable_narrative = false;
    }
    if matches.get_flag("save-eval-report") {
        config.save_eval_report = Some(true);
    }
    if let Some(output_dir_str) = matches.get_one::<String>("output-dir") {
        if matches.value_source("output-dir") == Some(clap::parser::ValueSource::CommandLine) {
            config.output_dir = output_dir_str.clone();
        }
    }
    
    // Parse biome (only override if explicitly provided)
    if let Some(biome_str) = matches.get_one::<String>("biome") {
        if matches.value_source("biome") == Some(clap::parser::ValueSource::CommandLine) {
            config.biome = match biome_str.as_str() {
                "saltflat" => Biome::Saltflat,
                "desert" => Biome::Desert,
                "ruins" => Biome::Ruins,
                "scrubland" => Biome::Scrubland,
                "oasis" => Biome::Oasis,
                _ => Biome::Saltflat,
            };
        }
    }
    
    // Parse layers (only override if explicitly provided)
    if let Some(layers_str) = matches.get_one::<String>("layers") {
        if matches.value_source("layers") == Some(clap::parser::ValueSource::CommandLine) {
            config.output_layers = if layers_str == "all" {
                vec!["all".to_string()]
            } else {
                layers_str.split(',').map(|s| s.trim().to_string()).collect()
            };
        }
    }
    
    // Parse format (only override if explicitly provided)
    if let Some(format_str) = matches.get_one::<String>("format") {
        if matches.value_source("format") == Some(clap::parser::ValueSource::CommandLine) {
            config.output_format = if format_str == "both" {
                vec!["text".to_string(), "png".to_string()]
            } else {
                vec![format_str.clone()]
            };
        }
    }
    
    // Handle save-config option
    if let Some(save_path) = matches.get_one::<String>("save-config") {
        if let Err(e) = config.to_json_file(save_path) {
            eprintln!("Failed to save config to {}: {}", save_path, e);
            std::process::exit(1);
        }
        println!("✅ Configuration saved to: {}", save_path);
        return;
    }
    
    println!("🚀 Tile Generation Test Tool");
    println!("============================");
    
    // Generate tiles with layers
    let mut generator = LayeredTileGeneration::new(config.clone());
    let snapshots = generator.generate_with_layers();
    
    // Output results
    if config.output_format.contains(&"text".to_string()) {
        TileVisualizer::save_text_output(&snapshots, &config.output_dir, &config);
    }
    
    if config.output_format.contains(&"png".to_string()) {
        TileVisualizer::save_png_output(&snapshots, &config.output_dir, &config);
    }
    
    // Generate evaluation report if requested
    if config.save_eval_report.unwrap_or(false) {
        if let Some(final_snapshot) = snapshots.last() {
            let evaluation = evaluate_map_quality(&final_snapshot.map);
            let metrics = calculate_map_metrics(&final_snapshot.map);
            
            // Create comprehensive evaluation data
            let eval_data = serde_json::json!({
                "config": config,
                "evaluation": evaluation,
                "metrics": metrics,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "layers_generated": snapshots.len()
            });
            
            let eval_filename = format!("{}/{}_evaluation.json", config.output_dir, config.seed);
            if let Err(e) = fs::write(&eval_filename, serde_json::to_string_pretty(&eval_data).unwrap()) {
                eprintln!("Failed to write evaluation report: {}", e);
            } else {
                println!("📊 Saved evaluation report: {}", eval_filename);
            }
        }
    }
    
    println!("\n✅ Generation complete!");
    println!("📁 Output saved to: {}", config.output_dir);
    println!("📊 Generated {} layers", snapshots.len());
}
