use saltglass_steppe::game::generation::{
    TerrainForgeGenerator,
    structures::{RuinsGenerator, StructureGenerator, StructureParams, StructureType}
};
use saltglass_steppe::game::world_map::{POI, Biome, Terrain};
use saltglass_steppe::game::map::Map;
use saltglass_steppe::game::constants::{MAP_WIDTH, MAP_HEIGHT};
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use std::env;

#[deprecated(note = "Legacy tile generation CLI; will be superseded by terrain-forge tooling.")]
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let command = &args[1];
    let seed = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(12345);
    
    match command.as_str() {
        "tile" => {
            let poi_type = args.get(3).map(|s| s.as_str());
            let biome = args.get(4).map(|s| s.as_str());
            generate_tile_map(seed, poi_type, biome);
        },
        "structure" => {
            let structure_type = args.get(3).map(|s| s.as_str()).unwrap_or("ruins");
            generate_structure_only(seed, structure_type);
        },
        "composite" => {
            let scenario = args.get(3).map(|s| s.as_str()).unwrap_or("quest_location");
            generate_composite_scenario(seed, scenario);
        },
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Saltglass Steppe - Comprehensive Tile Generation Tool");
    println!();
    println!("USAGE:");
    println!("  cargo run --bin tilegen-tool <command> [seed] [options]");
    println!();
    println!("COMMANDS:");
    println!("  tile [seed] [poi] [biome]      - Generate tile with optional POI/biome");
    println!("  structure [seed] [type]        - Generate structure only");
    println!("  composite [seed] [scenario]    - Generate composite scenarios");
    println!();
    println!("POI TYPES:");
    println!("  town, shrine, landmark, dungeon");
    println!();
    println!("BIOMES:");
    println!("  saltflat, desert, ruins, scrubland, oasis");
    println!();
    println!("STRUCTURE TYPES:");
    println!("  ruins, dungeon, town, shrine");
    println!();
    println!("COMPOSITE SCENARIOS:");
    println!("  quest_location  - Quest-driven structure generation");
    println!("  biome_variety   - Different biome + structure combinations");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin tilegen-tool tile 123 landmark ruins");
    println!("  cargo run --bin tilegen-tool structure 456 ruins");
    println!("  cargo run --bin tilegen-tool composite 789 quest_location");
}

fn generate_tile_map(seed: u64, poi_type: Option<&str>, biome: Option<&str>) {
    println!("=== TILE MAP GENERATION (Seed: {}) ===", seed);
    
    let poi = poi_type.and_then(|p| match p {
        "town" => Some(POI::Town),
        "shrine" => Some(POI::Shrine),
        "landmark" => Some(POI::Landmark),
        "dungeon" => Some(POI::Dungeon),
        _ => None,
    });
    
    let biome_str = biome.unwrap_or("saltflat");
    let biome = match biome_str {
        "saltflat" => Biome::Saltflat,
        "desert" => Biome::Desert,
        "ruins" => Biome::Ruins,
        "scrubland" => Biome::Scrubland,
        "oasis" => Biome::Oasis,
        _ => Biome::Saltflat,
    };
    
    println!("POI: {:?}", poi);
    println!("Biome: {:?}", biome);
    
    let quest_ids = if poi == Some(POI::Landmark) && biome == Biome::Ruins {
        vec!["the_broken_key".to_string()]
    } else {
        Vec::new()
    };

    let generator = TerrainForgeGenerator::new();
    let (map, clearings) = generator.generate_tile_with_seed(
        biome,
        Terrain::Canyon,
        50,
        poi.unwrap_or(POI::None),
        seed,
        &quest_ids,
    );
    
    display_tile_map(&map);
    
    // Show generation info
    println!("\n=== GENERATION INFO ===");
    println!("Seed: {}", seed);
    println!("Biome: {:?}", biome);
    println!("POI: {:?}", poi);
    println!("Clearings found: {}", clearings.len());
    
    if !map.metadata.is_empty() {
        println!("\n=== METADATA ===");
        for (key, value) in &map.metadata {
            println!("{}: {}", key, value);
        }
    }
}

fn generate_structure_only(seed: u64, structure_type: &str) {
    println!("=== STRUCTURE GENERATION (Seed: {}, Type: {}) ===", seed, structure_type);
    
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    
    let (struct_type, size, theme) = match structure_type {
        "ruins" => (StructureType::Ruins, (25, 20), "vitrified_library"),
        "dungeon" => (StructureType::Dungeon, (40, 30), "glass_cavern"),
        "town" => (StructureType::Town, (60, 50), "salt_settlement"),
        "shrine" => (StructureType::Shrine, (20, 20), "mirror_shrine"),
        _ => (StructureType::Ruins, (25, 20), "default"),
    };
    
    let params = StructureParams {
        structure_type: struct_type,
        size,
        theme: theme.to_string(),
        quest_requirements: vec!["the_broken_key".to_string()],
        biome_context: "ruins".to_string(),
        organic_walls: false,
    };
    
    match structure_type {
        "ruins" => {
            let generator = RuinsGenerator::new();
            if let Some(structure) = generator.generate(&params, &mut rng) {
                display_structure(&structure);
            } else {
                println!("Failed to generate structure");
            }
        },
        _ => {
            println!("Structure type '{}' not yet implemented", structure_type);
            println!("Available: ruins");
            println!("Coming soon: dungeon, town, shrine");
        }
    }
}

fn generate_composite_scenario(seed: u64, scenario: &str) {
    println!("=== COMPOSITE SCENARIO: {} (Seed: {}) ===", scenario.to_uppercase(), seed);
    
    match scenario {
        "quest_location" => demo_quest_location(seed),
        "biome_variety" => demo_biome_variety(seed),
        _ => {
            println!("Unknown scenario: {}", scenario);
            println!("Available: quest_location, biome_variety");
        }
    }
}

fn demo_quest_location(seed: u64) {
    println!("Demonstrating quest-driven structure generation...");
    println!("Scenario: Player travels to (50,50) for 'The Broken Key' quest");
    
    let quest_ids = vec!["the_broken_key".to_string()];
    let generator = TerrainForgeGenerator::new();
    let (map, clearings) = generator.generate_tile_with_seed(
        Biome::Ruins,
        Terrain::Canyon,
        50,
        POI::Landmark,
        seed,
        &quest_ids,
    );
    
    println!("\nGeneration Flow:");
    println!("1. Quest system detects 'the_broken_key' quest");
    println!("2. POI type: Landmark, Biome: Ruins");
    println!("3. Bracket-noise generates organic terrain");
    println!("4. Structure integrated with terrain");
    println!("5. Quest items and enemies placed");
    println!("6. Clearings found: {}", clearings.len());
    
    display_tile_map(&map);
    
    if let Some(spawns) = map.metadata.get("vitrified_library_spawns") {
        println!("\nQuest Integration:");
        println!("Spawn data: {}", spawns);
    }
}

fn demo_biome_variety(seed: u64) {
    println!("Demonstrating biome-specific generation...");
    
    let biomes = vec![
        (Biome::Saltflat, POI::Town),
        (Biome::Desert, POI::Shrine), 
        (Biome::Ruins, POI::Landmark),
        (Biome::Scrubland, POI::Dungeon),
    ];
    
    for (i, (biome, poi)) in biomes.iter().enumerate() {
        let biome_seed = seed + i as u64 * 1000;
        println!("\n--- {:?} + {:?} (Seed: {}) ---", biome, poi, biome_seed);
        
        let quest_ids = if *poi == POI::Landmark && *biome == Biome::Ruins {
            vec!["the_broken_key".to_string()]
        } else {
            Vec::new()
        };
        
        let generator = TerrainForgeGenerator::new();
        let (map, clearings) = generator.generate_tile_with_seed(
            *biome,
            Terrain::Canyon,
            50,
            *poi,
            biome_seed,
            &quest_ids,
        );
        
        println!("Clearings: {}", clearings.len());
        
        // Show small preview
        println!("Preview (top-left 20x10):");
        for y in 0..10 {
            for x in 0..20 {
                let idx = y * MAP_WIDTH + x;
                if idx < map.tiles.len() {
                    let char = match &map.tiles[idx] {
                        saltglass_steppe::game::map::Tile::Wall { .. } => '#',
                        saltglass_steppe::game::map::Tile::Floor { .. } => '.',
                        saltglass_steppe::game::map::Tile::Glass { .. } => '*',
                        _ => ' ',
                    };
                    print!("{}", char);
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
}

fn display_tile_map(map: &Map) {
    println!("\nTile Map ({}x{}):", MAP_WIDTH, MAP_HEIGHT);
    
    for y in 0..MAP_HEIGHT.min(40) {
        for x in 0..MAP_WIDTH.min(80) {
            let idx = y * MAP_WIDTH + x;
            if idx < map.tiles.len() {
                let char = match &map.tiles[idx] {
                    saltglass_steppe::game::map::Tile::Wall { .. } => '#',
                    saltglass_steppe::game::map::Tile::Floor { .. } => '.',
                    saltglass_steppe::game::map::Tile::Glass { .. } => '*',
                    _ => ' ',
                };
                print!("{}", char);
            } else {
                print!(" ");
            }
        }
        println!();
    }
    
    println!("\nLegend: # = Wall, . = Floor, * = Glass, (space) = Empty");
}

fn display_structure(structure: &saltglass_steppe::game::generation::structures::Structure) {
    println!("Structure Type: {:?}", structure.structure_type);
    println!("Bounds: {}x{} at ({}, {})", 
             structure.bounds.width, structure.bounds.height,
             structure.bounds.x, structure.bounds.y);
    println!("Rooms: {}", structure.rooms.len());
    println!("Spawn Points: {}", structure.spawn_points.len());
    
    // Create a simple ASCII representation
    let mut grid = vec![vec![' '; structure.bounds.width as usize]; structure.bounds.height as usize];
    
    // Draw rooms
    for (i, room) in structure.rooms.iter().enumerate() {
        let char = if i == 0 { 'M' } else { ('A' as u8 + i as u8 - 1) as char }; // M for main hall, A,B,C,D for chambers
        
        for y in room.bounds.y..room.bounds.y + room.bounds.height {
            for x in room.bounds.x..room.bounds.x + room.bounds.width {
                if x < structure.bounds.width && y < structure.bounds.height {
                    grid[y as usize][x as usize] = char;
                }
            }
        }
    }
    
    // Mark spawn points
    for spawn in &structure.spawn_points {
        if spawn.position.0 < structure.bounds.width && spawn.position.1 < structure.bounds.height {
            let char = match spawn.spawn_type.as_str() {
                "item" => 'I',
                "enemy" => 'E',
                _ => '?',
            };
            grid[spawn.position.1 as usize][spawn.position.0 as usize] = char;
        }
    }
    
    println!("\nStructure Layout:");
    for row in &grid {
        for &cell in row {
            print!("{}", cell);
        }
        println!();
    }
    
    println!("\nLegend: M = Main Hall, A-D = Chambers, I = Item, E = Enemy");
    
    if !structure.metadata.is_empty() {
        println!("\nMetadata:");
        for (key, value) in &structure.metadata {
            println!("  {}: {}", key, value);
        }
    }
}
