use std::env;
use tui_rpg::{Map, WorldMap, Tile, Biome, Terrain, POI};

const WORLD_WIDTH: usize = 192;
const WORLD_HEIGHT: usize = 64;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [seed]", args[0]);
        println!("Commands:");
        println!("  world [seed] - Generate and display world map");
        println!("  tile [seed]  - Generate and display tile map");
        return;
    }

    let seed = if args.len() > 2 {
        args[2].parse().unwrap_or_else(|_| {
            eprintln!("Invalid seed, using default");
            12345
        })
    } else {
        12345
    };

    match args[1].as_str() {
        "world" => display_world_map(seed),
        "tile" => display_tile_map(seed),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Use 'world' or 'tile'");
        }
    }
}

fn display_world_map(seed: u64) {
    println!("Generating world map with seed: {}", seed);
    let world_map = WorldMap::generate(seed);
    
    println!("World Map ({}x{}):", WORLD_WIDTH, WORLD_HEIGHT);
    
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let idx = y * WORLD_WIDTH + x;
            let biome = world_map.biomes[idx];
            let terrain = world_map.terrain[idx];
            let poi = world_map.pois[idx];
            
            let char = match poi {
                POI::Town => 'T',
                POI::Dungeon => 'D',
                POI::Landmark => 'L',
                POI::Shrine => 'S',
                POI::None => match biome {
                    Biome::Desert => match terrain {
                        Terrain::Dunes => '~',
                        Terrain::Flat => '.',
                        _ => '^',
                    },
                    Biome::Saltflat => '_',
                    Biome::Scrubland => ',',
                    Biome::Oasis => 'O',
                    Biome::Ruins => 'R',
                },
            };
            print!("{}", char);
        }
        println!();
    }
    
    println!("\nLegend:");
    println!("T=Town, D=Dungeon, L=Landmark, S=Shrine");
    println!("~=Dunes, .=Desert, ^=Hills/Mesa/Canyon, _=Saltflat, ,=Scrubland, O=Oasis, R=Ruins");
}

fn display_tile_map(seed: u64) {
    let args: Vec<String> = std::env::args().collect();
    let poi_type = args.get(3).map(|s| s.as_str()).unwrap_or("none");
    
    let poi = match poi_type {
        "town" => POI::Town,
        "dungeon" => POI::Dungeon,
        "landmark" => POI::Landmark,
        "shrine" => POI::Shrine,
        _ => POI::None,
    };
    
    println!("Generating tile map with seed: {} and POI: {:?}", seed, poi);
    
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let (map, _spawn_points) = Map::generate_from_world_with_poi(
        &mut rng,
        Biome::Desert,
        Terrain::Flat,
        128,
        poi,
    );
    
    println!("Tile Map ({}x{}):", map.width, map.height);
    
    for y in 0..map.height {
        for x in 0..map.width {
            if let Some(tile) = map.get(x as i32, y as i32) {
                let char = match tile {
                    Tile::Floor => '.',
                    Tile::Wall { .. } => '#',
                    Tile::Glass => 'g',
                    Tile::StairsDown => '>',
                    Tile::StairsUp => '<',
                    Tile::WorldExit => 'X',
                };
                print!("{}", char);
            }
        }
        println!();
    }
    
    println!("\nLegend:");
    println!(".=Floor, #=Wall, g=Glass, >=StairsDown, <=StairsUp, X=WorldExit");
}
