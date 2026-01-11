use clap::{Arg, Command};
use saltglass_steppe::game::map::{Map, Tile};
use saltglass_steppe::game::generation::TerrainForgeGenerator;
use saltglass_steppe::game::world_map::{Biome, Terrain, POI};
use std::collections::{HashMap, HashSet, VecDeque};
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
    pub constraints: Option<ConstraintConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintConfig {
    pub connectivity: Option<ConnectivityConstraint>,
    pub balance: Option<BalanceConstraint>,
    pub quality: Option<QualityConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityConstraint {
    pub min_reachable_percentage: f64,
    pub require_loops: bool,
    pub max_dead_ends: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceConstraint {
    pub min_open_space_percentage: f64,
    pub max_open_space_percentage: f64,
    pub min_room_count: Option<usize>,
    pub max_room_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConstraint {
    pub min_variety_score: f64,
    pub require_interesting_features: bool,
    pub max_repetitive_patterns: Option<usize>,
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
        
        let constraints = json.get("constraints")
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        
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
            constraints,
        })
    }
}

fn generate_enhanced_map(config: &EnhancedConfig) -> Result<(Map, serde_json::Value), Box<dyn std::error::Error>> {
    let generator = TerrainForgeGenerator::new();
 
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
    
    // Generate using terrain-forge adapter
    let (map, clearings) = generator.generate_tile_with_seed(biome, terrain, 50, poi, config.seed, &[]);
    
    // Generate evaluation data
    let evaluation = generate_evaluation(&map, config, clearings.len());
    
    Ok((map, evaluation))
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
    let connectivity_ratio = floor_count as f64 / (floor_count + glass_count) as f64;
    
    // Run constraint validation if configured
    let constraint_results = validate_constraints(map, config);
    let constraint_json: Vec<serde_json::Value> = constraint_results.iter().map(|result| {
        serde_json::json!({
            "constraint_type": result.constraint_name,
            "passed": result.passed,
            "score": result.score,
            "message": result.message
        })
    }).collect();
    
    // Add default constraints if no custom ones specified
    let mut all_constraints = constraint_json;
    if config.constraints.is_none() {
        all_constraints.extend(vec![
            serde_json::json!({
                "constraint_type": "connectivity",
                "expected_value": 0.8,
                "actual_value": connectivity_ratio,
                "passed": connectivity_ratio >= 0.8,
                "message": format!("Connectivity ratio {:.2} {} minimum 0.80", 
                    connectivity_ratio, if connectivity_ratio >= 0.8 { "meets" } else { "below" })
            }),
            serde_json::json!({
                "constraint_type": "floor_density", 
                "expected_value": 0.3,
                "actual_value": floor_ratio,
                "passed": floor_ratio >= 0.3,
                "message": format!("Floor density {:.2} {} minimum 0.30",
                    floor_ratio, if floor_ratio >= 0.3 { "meets" } else { "below" })
            })
        ]);
    }
    
    // Calculate overall quality score
    let constraint_score = if !constraint_results.is_empty() {
        constraint_results.iter().map(|r| r.score).sum::<f64>() / constraint_results.len() as f64
    } else {
        (connectivity_ratio + floor_ratio) / 2.0
    };
    
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
            "constraints": all_constraints,
            "quality_score": constraint_score
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

#[deprecated(note = "Legacy tile generation CLI; will be superseded by terrain-forge tooling.")]
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

#[derive(Debug, Clone)]
pub struct ConstraintValidationResult {
    pub constraint_name: String,
    pub passed: bool,
    pub score: f64,
    pub message: String,
}

fn validate_constraints(map: &Map, config: &EnhancedConfig) -> Vec<ConstraintValidationResult> {
    let mut results = Vec::new();
    
    if let Some(constraints) = &config.constraints {
        // Connectivity validation
        if let Some(connectivity) = &constraints.connectivity {
            results.extend(validate_connectivity(map, connectivity));
        }
        
        // Balance validation
        if let Some(balance) = &constraints.balance {
            results.extend(validate_balance(map, balance));
        }
        
        // Quality validation
        if let Some(quality) = &constraints.quality {
            results.extend(validate_quality(map, quality));
        }
    }
    
    results
}

fn validate_connectivity(map: &Map, constraint: &ConnectivityConstraint) -> Vec<ConstraintValidationResult> {
    let mut results = Vec::new();
    
    // Find all floor tiles
    let mut floor_positions = Vec::new();
    for (i, tile) in map.tiles.iter().enumerate() {
        if matches!(tile, Tile::Floor { .. }) {
            let x = i % map.width;
            let y = i / map.width;
            floor_positions.push((x, y));
        }
    }
    
    if floor_positions.is_empty() {
        results.push(ConstraintValidationResult {
            constraint_name: "connectivity_reachable".to_string(),
            passed: false,
            score: 0.0,
            message: "No floor tiles found".to_string(),
        });
        return results;
    }
    
    // Flood fill from first floor tile to find reachable area
    let start = floor_positions[0];
    let reachable = flood_fill_reachable(map, start);
    let reachable_percentage = reachable.len() as f64 / floor_positions.len() as f64;
    
    let connectivity_passed = reachable_percentage >= constraint.min_reachable_percentage;
    results.push(ConstraintValidationResult {
        constraint_name: "connectivity_reachable".to_string(),
        passed: connectivity_passed,
        score: reachable_percentage,
        message: format!("Reachable: {:.1}% (required: {:.1}%)", 
                        reachable_percentage * 100.0, 
                        constraint.min_reachable_percentage * 100.0),
    });
    
    // Check for loops if required
    if constraint.require_loops {
        let has_loops = detect_loops(map, &reachable);
        results.push(ConstraintValidationResult {
            constraint_name: "connectivity_loops".to_string(),
            passed: has_loops,
            score: if has_loops { 1.0 } else { 0.0 },
            message: format!("Loops detected: {}", has_loops),
        });
    }
    
    results
}

fn validate_balance(map: &Map, constraint: &BalanceConstraint) -> Vec<ConstraintValidationResult> {
    let mut results = Vec::new();
    
    let total_tiles = map.tiles.len() as f64;
    let floor_count = map.tiles.iter()
        .filter(|tile| matches!(tile, Tile::Floor { .. }))
        .count() as f64;
    let open_space_percentage = floor_count / total_tiles;
    
    let balance_passed = open_space_percentage >= constraint.min_open_space_percentage 
                        && open_space_percentage <= constraint.max_open_space_percentage;
    
    results.push(ConstraintValidationResult {
        constraint_name: "balance_open_space".to_string(),
        passed: balance_passed,
        score: if balance_passed { 1.0 } else { 0.5 },
        message: format!("Open space: {:.1}% (required: {:.1}%-{:.1}%)", 
                        open_space_percentage * 100.0,
                        constraint.min_open_space_percentage * 100.0,
                        constraint.max_open_space_percentage * 100.0),
    });
    
    results
}

fn validate_quality(map: &Map, constraint: &QualityConstraint) -> Vec<ConstraintValidationResult> {
    let mut results = Vec::new();
    
    // Calculate variety score based on tile distribution
    let variety_score = calculate_variety_score(map);
    let variety_passed = variety_score >= constraint.min_variety_score;
    
    results.push(ConstraintValidationResult {
        constraint_name: "quality_variety".to_string(),
        passed: variety_passed,
        score: variety_score,
        message: format!("Variety score: {:.3} (required: {:.3})", 
                        variety_score, constraint.min_variety_score),
    });
    
    results
}

fn flood_fill_reachable(map: &Map, start: (usize, usize)) -> HashSet<(usize, usize)> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start);
    
    while let Some((x, y)) = queue.pop_front() {
        if visited.contains(&(x, y)) {
            continue;
        }
        visited.insert((x, y));
        
        // Check 4-connected neighbors
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            
            if nx >= 0 && ny >= 0 && (nx as usize) < map.width && (ny as usize) < map.height {
                let nx = nx as usize;
                let ny = ny as usize;
                let idx = ny * map.width + nx;
                
                if idx < map.tiles.len() && matches!(map.tiles[idx], Tile::Floor { .. }) && !visited.contains(&(nx, ny)) {
                    queue.push_back((nx, ny));
                }
            }
        }
    }
    
    visited
}

fn detect_loops(map: &Map, reachable: &HashSet<(usize, usize)>) -> bool {
    // Simple loop detection: check if any floor tile has more than 2 floor neighbors
    for &(x, y) in reachable {
        let mut neighbor_count = 0;
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            
            if nx >= 0 && ny >= 0 && (nx as usize) < map.width && (ny as usize) < map.height {
                let nx = nx as usize;
                let ny = ny as usize;
                if reachable.contains(&(nx, ny)) {
                    neighbor_count += 1;
                }
            }
        }
        
        if neighbor_count > 2 {
            return true; // Found a junction, indicating loops
        }
    }
    false
}

fn calculate_variety_score(map: &Map) -> f64 {
    // Calculate variety based on spatial distribution of features
    let mut pattern_counts = HashMap::new();
    
    // Sample 3x3 patterns across the map
    for y in 0..(map.height - 2) {
        for x in 0..(map.width - 2) {
            let mut pattern = String::new();
            for py in 0..3 {
                for px in 0..3 {
                    let idx = (y + py) * map.width + (x + px);
                    if idx < map.tiles.len() {
                        pattern.push(match map.tiles[idx] {
                            Tile::Floor { .. } => 'F',
                            Tile::Wall { .. } => 'W',
                            _ => 'O',
                        });
                    }
                }
            }
            *pattern_counts.entry(pattern).or_insert(0) += 1;
        }
    }
    
    // Variety score is based on pattern diversity
    let total_patterns = pattern_counts.values().sum::<i32>() as f64;
    let unique_patterns = pattern_counts.len() as f64;
    
    if total_patterns > 0.0 {
        unique_patterns / total_patterns.sqrt()
    } else {
        0.0
    }
}
