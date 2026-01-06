use clap::{Arg, Command};
use saltglass_steppe::game::map::{Map, Tile};
use saltglass_steppe::game::generation::TileGenerator;
use saltglass_steppe::game::world_map::{Biome, Terrain, POI};
use rand_chacha::ChaCha8Rng;
use rand::{SeedableRng, Rng};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use image::{ImageBuffer, Rgb};
use chrono;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedConfig {
    pub seed: u64,
    pub width: i32,
    pub height: i32,
    pub output_dir: String,
    pub biome: Option<String>,
    pub poi: Option<String>,
    pub terrain_type: Option<String>,
    pub use_bracket_noise: Option<bool>,
    pub output_layers: Option<Vec<String>>,
    pub output_format: Option<Vec<String>>,
    pub enable_evaluation: Option<bool>,
    pub pipeline_stages: Option<Vec<String>>,
    pub algorithm: Option<String>,
    pub algorithm_params: Option<serde_json::Value>,
    pub test_suite: Option<String>,
}

impl EnhancedConfig {
    pub fn from_json_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        
        let seed = json.get("seed").and_then(|v| v.as_u64()).unwrap_or(12345);
        let width = json.get("width").and_then(|v| v.as_i64()).unwrap_or(80) as i32;
        let height = json.get("height").and_then(|v| v.as_i64()).unwrap_or(40) as i32;
        let output_dir = json.get("output_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("enhanced-tile-test-suite")
            .to_string();
        let biome = json.get("biome").and_then(|v| v.as_str()).map(|s| s.to_string());
        let poi = json.get("poi").and_then(|v| v.as_str()).map(|s| s.to_string());
        let terrain_type = json.get("terrain_type").and_then(|v| v.as_str()).map(|s| s.to_string());
        let use_bracket_noise = json.get("use_bracket_noise").and_then(|v| v.as_bool());
        let output_layers = json.get("output_layers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        let output_format = json.get("output_format")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        let enable_evaluation = json.get("enable_evaluation").and_then(|v| v.as_bool());
        let pipeline_stages = json.get("pipeline_stages")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        let algorithm = json.get("algorithm").and_then(|v| v.as_str()).map(|s| s.to_string());
        let algorithm_params = json.get("algorithm_params").cloned();
        let test_suite = json.get("test_suite").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        Ok(EnhancedConfig {
            seed,
            width,
            height,
            output_dir,
            biome,
            poi,
            terrain_type,
            use_bracket_noise,
            output_layers,
            output_format,
            enable_evaluation,
            pipeline_stages,
            algorithm,
            algorithm_params,
            test_suite,
        })
    }
}

fn generate_enhanced_map(config: &EnhancedConfig) -> Result<(Map, serde_json::Value), Box<dyn std::error::Error>> {
    let use_bracket_noise = config.use_bracket_noise.unwrap_or(true);
    
    if use_bracket_noise {
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let tile_gen = TileGenerator::new()?;
        
        let biome = config.biome.as_ref()
            .and_then(|b| match b.as_str() {
                "saltflat" => Some(Biome::Saltflat),
                "desert" => Some(Biome::Desert),
                "ruins" => Some(Biome::Ruins),
                "scrubland" => Some(Biome::Scrubland),
                "oasis" => Some(Biome::Oasis),
                _ => None,
            })
            .unwrap_or(Biome::Saltflat);
            
        let poi = config.poi.as_ref()
            .and_then(|p| match p.as_str() {
                "town" => Some(POI::Town),
                "shrine" => Some(POI::Shrine),
                "landmark" => Some(POI::Landmark),
                "dungeon" => Some(POI::Dungeon),
                _ => None,
            })
            .unwrap_or(POI::None);
            
        let terrain = config.terrain_type.as_ref()
            .and_then(|t| match t.as_str() {
                "canyon" => Some(Terrain::Canyon),
                "mesa" => Some(Terrain::Mesa),
                "hills" => Some(Terrain::Hills),
                "dunes" => Some(Terrain::Dunes),
                "flat" => Some(Terrain::Flat),
                _ => None,
            })
            .unwrap_or(Terrain::Flat);
        
        let (map, clearings) = tile_gen.generate_enhanced_tile_with_quests(
            &mut rng,
            biome,
            terrain,
            50,
            poi,
            &[]
        );
        
        // Generate evaluation data
        let evaluation = generate_evaluation(&map, config, clearings.len());
        
        Ok((map, evaluation))
    } else {
        Err("Simple generation not supported in enhanced mode".into())
    }
}

fn generate_evaluation(map: &Map, config: &EnhancedConfig, clearings_count: usize) -> serde_json::Value {
    let mut tile_counts = HashMap::new();
    let mut floor_count = 0;
    let mut wall_count = 0;
    let mut glass_count = 0;
    let mut other_count = 0;
    
    for tile in &map.tiles {
        match tile {
            Tile::Floor { .. } => {
                floor_count += 1;
                *tile_counts.entry("floor".to_string()).or_insert(0) += 1;
            },
            Tile::Wall { .. } => {
                wall_count += 1;
                *tile_counts.entry("wall".to_string()).or_insert(0) += 1;
            },
            Tile::Glass => {
                glass_count += 1;
                *tile_counts.entry("glass".to_string()).or_insert(0) += 1;
            },
            _ => {
                other_count += 1;
                *tile_counts.entry("other".to_string()).or_insert(0) += 1;
            }
        }
    }
    
    let total_tiles = map.tiles.len();
    let floor_ratio = floor_count as f64 / total_tiles as f64;
    let wall_ratio = wall_count as f64 / total_tiles as f64;
    let connectivity_ratio = floor_count as f64 / (floor_count + glass_count) as f64;
    
    serde_json::json!({
        "config": config,
        "evaluation": {
            "distribution": {
                "counts": {
                    "floor": floor_count,
                    "wall": wall_count,
                    "glass": glass_count,
                    "other": other_count
                },
                "total": total_tiles
            },
            "connectivity": {
                "connectivity_ratio": connectivity_ratio,
                "total_floor_tiles": floor_count,
                "clearings_found": clearings_count
            },
            "constraints": [
                {
                    "constraint_type": "connectivity",
                    "expected_value": 0.8,
                    "actual_value": connectivity_ratio,
                    "passed": connectivity_ratio >= 0.8,
                    "message": format!("Connectivity ratio {:.2} {} minimum 0.80", 
                        connectivity_ratio, if connectivity_ratio >= 0.8 { "meets" } else { "below" })
                },
                {
                    "constraint_type": "floor_density", 
                    "expected_value": 0.3,
                    "actual_value": floor_ratio,
                    "passed": floor_ratio >= 0.3,
                    "message": format!("Floor density {:.2} {} minimum 0.30",
                        floor_ratio, if floor_ratio >= 0.3 { "meets" } else { "below" })
                }
            ],
            "quality_score": (connectivity_ratio + floor_ratio) / 2.0
        },
        "metrics": {
            "width": map.width,
            "height": map.height,
            "total_tiles": total_tiles,
            "tile_counts": tile_counts,
            "openness": floor_ratio,
            "complexity": glass_count as f64 / total_tiles as f64
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })
}

fn save_enhanced_outputs(map: &Map, evaluation: &serde_json::Value, config: &EnhancedConfig) -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = &config.output_dir;
    std::fs::create_dir_all(format!("{}/text", base_dir))?;
    std::fs::create_dir_all(format!("{}/pngs", base_dir))?;
    std::fs::create_dir_all(format!("{}/evaluations", base_dir))?;
    
    let seed = config.seed;
    
    // Save text output
    if config.output_format.as_ref().map_or(true, |f| f.contains(&"text".to_string())) {
        save_text_output(map, config, &format!("{}/text", base_dir))?;
    }
    
    // Save PNG output
    if config.output_format.as_ref().map_or(true, |f| f.contains(&"png".to_string())) {
        save_png_output(map, config, &format!("{}/pngs", base_dir))?;
    }
    
    // Save evaluation
    if config.enable_evaluation.unwrap_or(true) {
        let eval_filename = format!("{}/evaluations/{}_evaluation.json", base_dir, seed);
        std::fs::write(&eval_filename, serde_json::to_string_pretty(evaluation)?)?;
        println!("📊 Saved: {}", eval_filename);
    }
    
    Ok(())
}

fn save_text_output(map: &Map, config: &EnhancedConfig, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    
    let filename = format!("{}/{}_base_terrain.txt", output_dir, config.seed);
    std::fs::write(&filename, output)?;
    println!("📄 Saved: {}", filename);
    Ok(())
}

fn save_png_output(map: &Map, config: &EnhancedConfig, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let scale = 4u32;
    let width = map.width as u32 * scale;
    let height = map.height as u32 * scale;
    let mut img = ImageBuffer::new(width, height);
    
    for y in 0..map.height {
        for x in 0..map.width {
            let color = if let Some(tile) = map.tiles.get(y * map.width + x) {
                match tile {
                    Tile::Wall { .. } => Rgb([64u8, 64u8, 64u8]),      // Dark gray
                    Tile::Floor { .. } => Rgb([200u8, 200u8, 200u8]), // Light gray
                    Tile::Glass => Rgb([0u8, 255u8, 255u8]),          // Cyan
                    _ => Rgb([128u8, 128u8, 128u8]),                   // Medium gray
                }
            } else {
                Rgb([0u8, 0u8, 0u8]) // Black
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
    
    let filename = format!("{}/{}_base_terrain.png", output_dir, config.seed);
    img.save(&filename)?;
    println!("🖼️  Saved: {}", filename);
    Ok(())
}

fn main() {
    let matches = Command::new("Enhanced Tile Generation Test Tool")
        .version("3.0")
        .about("Comprehensive tile generation testing with evaluation and pipeline analysis")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("CONFIG_FILE")
            .help("JSON configuration file")
            .required(true))
        .get_matches();

    let config_file = matches.get_one::<String>("config").unwrap();
    
    println!("🚀 Enhanced Tile Generation Test Tool");
    println!("=====================================");
    
    let config = match EnhancedConfig::from_json_file(config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("🎯 Generating with seed: {}", config.seed);
    println!("🌍 Biome: {:?}", config.biome);
    println!("🏛️  POI: {:?}", config.poi);
    
    match generate_enhanced_map(&config) {
        Ok((map, evaluation)) => {
            if let Err(e) = save_enhanced_outputs(&map, &evaluation, &config) {
                eprintln!("Failed to save outputs: {}", e);
                std::process::exit(1);
            }
            
            // Print summary
            if let Some(eval_obj) = evaluation.get("evaluation") {
                if let Some(quality) = eval_obj.get("quality_score").and_then(|v| v.as_f64()) {
                    println!("📈 Quality Score: {:.3}", quality);
                }
                if let Some(constraints) = eval_obj.get("constraints").and_then(|v| v.as_array()) {
                    let passed = constraints.iter().filter(|c| c.get("passed").and_then(|v| v.as_bool()).unwrap_or(false)).count();
                    println!("✅ Constraints: {}/{} passed", passed, constraints.len());
                }
            }
            
            println!("✅ Generation complete!");
        },
        Err(e) => {
            eprintln!("Generation failed: {}", e);
            std::process::exit(1);
        }
    }
}
