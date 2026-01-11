use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use saltglass_steppe::game::generation::structures::StructureGenerator;
use saltglass_steppe::game::generation::structures::dungeon_generator::{
    DungeonGenerator, DungeonGeneratorParams,
};
use saltglass_steppe::game::generation::structures::{StructureParams, StructureType};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let seed = if args.len() > 1 {
        args[1].parse().unwrap_or(12345)
    } else {
        12345
    };

    let preset = if args.len() > 2 {
        args[2].as_str()
    } else {
        "default"
    };

    println!("=== Dungeon Generator Test Tool ===");
    println!("Seed: {}", seed);
    println!("Preset: {}", preset);
    println!();

    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let params = match preset {
        "small" => DungeonGeneratorParams {
            width: 40,
            height: 20,
            organic_blend_factor: 0.2,
        },
        "large" => DungeonGeneratorParams {
            width: 120,
            height: 60,
            organic_blend_factor: 0.4,
        },
        "organic" => DungeonGeneratorParams {
            width: 80,
            height: 40,
            organic_blend_factor: 0.8,
        },
        "structured" => DungeonGeneratorParams {
            width: 80,
            height: 40,
            organic_blend_factor: 0.1,
        },
        _ => DungeonGeneratorParams::default(),
    };

    let generator = DungeonGenerator::new(params.clone());

    let structure_params =
        StructureParams::new(StructureType::Dungeon, (params.width, params.height));

    if let Some(structure) = generator.generate(&structure_params, &mut rng) {
        println!("Generated dungeon: {}x{}", params.width, params.height);
        println!("Rooms: {}", structure.rooms.len());
        println!("Features: {}", structure.features.len());
        println!();

        // Display the dungeon
        display_dungeon(&structure, params.width, params.height);

        // Print statistics
        print_statistics(&structure, params.width, params.height);
    } else {
        println!("Failed to generate dungeon!");
    }
}

fn display_dungeon(
    structure: &saltglass_steppe::game::generation::structures::Structure,
    width: u32,
    height: u32,
) {
    let mut grid = vec![vec![' '; width as usize]; height as usize];

    // Fill in features
    for feature in &structure.features {
        let (x, y) = feature.position;
        if x < width && y < height {
            let symbol = match feature.feature_type.as_str() {
                "wall" => '#',
                "floor" => '.',
                _ => '?',
            };
            grid[y as usize][x as usize] = symbol;
        }
    }

    // Print the grid
    for row in &grid {
        for &cell in row {
            print!("{}", cell);
        }
        println!();
    }
}

fn print_statistics(
    structure: &saltglass_steppe::game::generation::structures::Structure,
    width: u32,
    height: u32,
) {
    let total_tiles = (width * height) as usize;
    let mut wall_count = 0;
    let mut floor_count = 0;

    for feature in &structure.features {
        match feature.feature_type.as_str() {
            "wall" => wall_count += 1,
            "floor" => floor_count += 1,
            _ => {}
        }
    }

    let wall_percentage = (wall_count as f32 / total_tiles as f32) * 100.0;
    let floor_percentage = (floor_count as f32 / total_tiles as f32) * 100.0;

    println!();
    println!("=== Statistics ===");
    println!("Total tiles: {}", total_tiles);
    println!("Walls: {} ({:.1}%)", wall_count, wall_percentage);
    println!("Floors: {} ({:.1}%)", floor_count, floor_percentage);
    println!("Rooms: {}", structure.rooms.len());
    println!("Corridors: {}", structure.corridors.len());
}
