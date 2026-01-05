use clap::{Arg, Command};
use saltglass_steppe::game::map::{Map, Tile};
use rand_chacha::ChaCha8Rng;
use rand::{SeedableRng, Rng};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConfig {
    pub seed: u64,
    pub width: i32,
    pub height: i32,
    pub output_dir: String,
}

impl SimpleConfig {
    pub fn from_json_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        
        // Try to parse as any JSON and extract basic fields
        let json: serde_json::Value = serde_json::from_str(&content)?;
        
        let seed = json.get("seed").and_then(|v| v.as_u64()).unwrap_or(12345);
        let width = json.get("width").and_then(|v| v.as_i64()).unwrap_or(80) as i32;
        let height = json.get("height").and_then(|v| v.as_i64()).unwrap_or(40) as i32;
        let output_dir = json.get("output_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("tile-generation-sample-library")
            .to_string();
        
        Ok(SimpleConfig {
            seed,
            width,
            height,
            output_dir,
        })
    }
}

fn generate_simple_map(config: &SimpleConfig) -> Map {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let size = (config.width * config.height) as usize;
    let mut tiles = vec![Tile::Floor { id: "ground".to_string() }; size];
    
    // Simple generation - just add some walls and glass
    for i in 0..size {
        let val = rng.gen_range(0.0..1.0);
        tiles[i] = if val > 0.7 {
            Tile::Wall { id: "stone".to_string(), hp: 100 }
        } else if val < 0.1 {
            Tile::Glass
        } else {
            Tile::Floor { id: "ground".to_string() }
        };
    }
    
    Map {
        tiles,
        width: config.width as usize,
        height: config.height as usize,
        lights: Vec::new(),
        inscriptions: Vec::new(),
        area_description: None,
        metadata: HashMap::new(),
    }
}

fn save_text_output(map: &Map, config: &SimpleConfig) {
    std::fs::create_dir_all(&config.output_dir).unwrap();
    
    let mut output = String::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let glyph = map.tiles.get(y * map.width + x)
                .map(|t| t.glyph())
                .unwrap_or(' ');
            output.push(glyph);
        }
        output.push('\n');
    }
    
    let filename = format!("{}/{}_base_terrain.txt", config.output_dir, config.seed);
    std::fs::write(&filename, output).unwrap();
    println!("📄 Saved: {}", filename);
}

fn main() {
    let matches = Command::new("Tile Generation Test Tool")
        .version("2.0")
        .about("Simple JSON config-based tile generation")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("CONFIG_FILE")
            .help("JSON configuration file")
            .required(true))
        .get_matches();

    let config_file = matches.get_one::<String>("config").unwrap();
    
    println!("🚀 Tile Generation Test Tool");
    println!("============================");
    
    let config = match SimpleConfig::from_json_file(config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("🎯 Generating with seed: {}", config.seed);
    
    let map = generate_simple_map(&config);
    save_text_output(&map, &config);
    
    println!("✅ Generation complete!");
}
