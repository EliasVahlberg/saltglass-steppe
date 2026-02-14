use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use saltglass_steppe::game::constants::{MAP_HEIGHT, MAP_WIDTH};
use saltglass_steppe::game::generation::{
    TerrainForgeGenerator,
    structures::{RuinsGenerator, StructureGenerator, StructureParams, StructureType},
};
use saltglass_steppe::game::map::{Map, Tile};
use saltglass_steppe::game::world_map::{Biome, POI, Terrain};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Deserialize)]
struct DungeonPreset {
    room_count: u32,
    min_room_size: [u32; 2],
    max_room_size: [u32; 2],
    organic_blend: f64,
    ca_iterations: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct StructureConfig {
    dungeon_presets: HashMap<String, DungeonPreset>,
}

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
        }
        "structure" => {
            let structure_type = args.get(3).map(|s| s.as_str()).unwrap_or("ruins");
            generate_structure_only(seed, structure_type);
        }
        "composite" => {
            let scenario = args.get(3).map(|s| s.as_str()).unwrap_or("quest_location");
            generate_composite_scenario(seed, scenario);
        }
        "dungeon" => {
            let preset = args.get(3).map(|s| s.as_str()).unwrap_or("small_dungeon");
            generate_dungeon(seed, preset);
        }
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
    println!("  dungeon [seed] [preset]        - Generate dungeon with preset");
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
    println!("DUNGEON PRESETS:");
    println!("  small_dungeon, large_dungeon, organic_cave, structured_archive");
    println!();
    println!("COMPOSITE SCENARIOS:");
    println!("  quest_location  - Quest-driven structure generation");
    println!("  biome_variety   - Different biome + structure combinations");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin tilegen-tool tile 123 landmark ruins");
    println!("  cargo run --bin tilegen-tool structure 456 ruins");
    println!("  cargo run --bin tilegen-tool composite 789 quest_location");
    println!("  cargo run --bin tilegen-tool dungeon 42 large_dungeon");
}

fn generate_dungeon(seed: u64, preset_name: &str) {
    println!(
        "=== DUNGEON GENERATION (Seed: {}, Preset: {}) ===",
        seed, preset_name
    );

    // Load structure config
    let data = include_str!("../../data/structure_generation.json");
    let config: StructureConfig =
        serde_json::from_str(data).expect("Failed to parse structure_generation.json");

    let preset = match config.dungeon_presets.get(preset_name) {
        Some(p) => p,
        None => {
            println!("Unknown preset: {}", preset_name);
            println!(
                "Available presets: {:?}",
                config.dungeon_presets.keys().collect::<Vec<_>>()
            );
            return;
        }
    };

    println!("Preset parameters:");
    println!("  Room count: {}", preset.room_count);
    println!(
        "  Room size: {}x{} to {}x{}",
        preset.min_room_size[0],
        preset.min_room_size[1],
        preset.max_room_size[0],
        preset.max_room_size[1]
    );
    println!("  Organic blend: {:.1}", preset.organic_blend);
    println!("  CA iterations: {}", preset.ca_iterations);

    // Use terrain-forge BSP algorithm
    use terrain_forge::{Grid, Params, Tile as ForgeTile};

    let mut grid: Grid<ForgeTile> = Grid::new(80, 40);
    let mut params = Params::new();
    params.insert("min_room_size".to_string(), serde_json::json!(5));
    params.insert("max_depth".to_string(), serde_json::json!(4));
    params.insert("room_padding".to_string(), serde_json::json!(2));

    terrain_forge::ops::generate("bsp", &mut grid, Some(seed), Some(&params))
        .expect("Failed to generate dungeon");

    // Convert to Map
    let mut map = Map::new(80, 40);
    for y in 0..40 {
        for x in 0..80 {
            let idx = y * 80 + x;
            map.tiles[idx] = match grid.get(x as i32, y as i32) {
                Some(ForgeTile::Wall) => Tile::Wall {
                    id: "stone".to_string(),
                    hp: 100,
                },
                Some(ForgeTile::Floor) => Tile::Floor {
                    id: "stone".to_string(),
                },
                _ => Tile::Wall {
                    id: "stone".to_string(),
                    hp: 100,
                },
            };
        }
    }

    display_dungeon_map(&map);
    display_dungeon_stats(&map);
}

fn display_dungeon_map(map: &Map) {
    println!("\nDungeon Map ({}x{}):", map.width, map.height);

    for y in 0..map.height.min(40) {
        for x in 0..map.width.min(80) {
            let idx = y * map.width + x;
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

fn display_dungeon_stats(map: &Map) {
    let mut wall_count = 0;
    let mut floor_count = 0;
    let mut glass_count = 0;
    let mut other_count = 0;

    for tile in &map.tiles {
        match tile {
            saltglass_steppe::game::map::Tile::Wall { .. } => wall_count += 1,
            saltglass_steppe::game::map::Tile::Floor { .. } => floor_count += 1,
            saltglass_steppe::game::map::Tile::Glass { .. } => glass_count += 1,
            _ => other_count += 1,
        }
    }

    let total_tiles = map.tiles.len();
    let navigable_tiles = floor_count + glass_count;
    let connectivity_ratio = if total_tiles > 0 {
        navigable_tiles as f64 / total_tiles as f64
    } else {
        0.0
    };

    println!("\n=== DUNGEON STATS ===");
    println!("Total tiles: {}", total_tiles);
    println!(
        "Walls: {} ({:.1}%)",
        wall_count,
        wall_count as f64 / total_tiles as f64 * 100.0
    );
    println!(
        "Floors: {} ({:.1}%)",
        floor_count,
        floor_count as f64 / total_tiles as f64 * 100.0
    );
    println!(
        "Glass: {} ({:.1}%)",
        glass_count,
        glass_count as f64 / total_tiles as f64 * 100.0
    );
    println!(
        "Other: {} ({:.1}%)",
        other_count,
        other_count as f64 / total_tiles as f64 * 100.0
    );
    println!("Connectivity ratio: {:.2}", connectivity_ratio);

    if let Some(desc) = &map.area_description {
        println!("Description: {}", desc);
    }
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
    println!(
        "=== STRUCTURE GENERATION (Seed: {}, Type: {}) ===",
        seed, structure_type
    );

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
        }
        _ => {
            println!("Structure type '{}' not yet implemented", structure_type);
            println!("Available: ruins");
            println!("Coming soon: dungeon, town, shrine");
        }
    }
}

fn generate_composite_scenario(seed: u64, scenario: &str) {
    println!(
        "=== COMPOSITE SCENARIO: {} (Seed: {}) ===",
        scenario.to_uppercase(),
        seed
    );

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
    println!(
        "Bounds: {}x{} at ({}, {})",
        structure.bounds.width, structure.bounds.height, structure.bounds.x, structure.bounds.y
    );
    println!("Rooms: {}", structure.rooms.len());
    println!("Spawn Points: {}", structure.spawn_points.len());

    // Create a simple ASCII representation
    let mut grid =
        vec![vec![' '; structure.bounds.width as usize]; structure.bounds.height as usize];

    // Draw rooms
    for (i, room) in structure.rooms.iter().enumerate() {
        let char = if i == 0 {
            'M'
        } else {
            ('A' as u8 + i as u8 - 1) as char
        }; // M for main hall, A,B,C,D for chambers

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
