use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use saltglass_steppe::game::generation::settlement::{
    SettlementConfig, SettlementTier, generate_settlement,
};
use saltglass_steppe::{Biome, Map, POI, Terrain, Tile, WorldMap};
use std::env;

const WORLD_WIDTH: usize = 192;
const WORLD_HEIGHT: usize = 64;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command> [args...]", args[0]);
        println!("Commands:");
        println!("  world [seed] - Generate and display world map");
        println!("  tile [seed]  - Generate and display tile map");
        println!("  settlement <seed> <tier> - Generate settlement (tier: village, town, city)");
        return;
    }

    match args[1].as_str() {
        "world" => {
            let seed = if args.len() > 2 {
                args[2].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid seed, using default");
                    12345
                })
            } else {
                12345
            };
            display_world_map(seed);
        }
        "tile" => {
            let seed = if args.len() > 2 {
                args[2].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid seed, using default");
                    12345
                })
            } else {
                12345
            };
            display_tile_map(seed);
        }
        "settlement" => {
            if args.len() < 4 {
                eprintln!("Usage: {} settlement <seed> <tier>", args[0]);
                eprintln!("Tier: village, town, city");
                return;
            }
            let seed = args[2].parse().unwrap_or_else(|_| {
                eprintln!("Invalid seed, using default");
                12345
            });
            let tier = match args[3].as_str() {
                "village" => SettlementTier::Village,
                "town" => SettlementTier::Town,
                "city" => SettlementTier::City,
                _ => {
                    eprintln!("Invalid tier: {}. Use village, town, or city", args[3]);
                    return;
                }
            };
            display_settlement(seed, tier);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Use 'world', 'tile', or 'settlement'");
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
    let (map, _spawn_points) =
        Map::generate_from_world_with_poi(&mut rng, Biome::Desert, Terrain::Flat, 128, poi);

    println!("Tile Map ({}x{}):", map.width, map.height);

    for y in 0..map.height {
        for x in 0..map.width {
            if let Some(tile) = map.get(x as i32, y as i32) {
                let char = match tile {
                    Tile::Floor { .. } => '.',
                    Tile::Wall { .. } => '#',
                    Tile::Glass => 'g',
                    Tile::Glare => 'G',
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

fn display_settlement(seed: u64, tier: SettlementTier) {
    use saltglass_steppe::game::generation::settlement::{
        clear_settlement_footprint, place_decorations, stamp_settlement,
    };

    println!(
        "Generating settlement with seed: {} and tier: {:?}",
        seed, tier
    );

    let config = SettlementConfig {
        seed,
        tier,
        faction_control: vec![
            ("MirrorMonks".to_string(), 0.4),
            ("SaltTraders".to_string(), 0.3),
        ],
    };

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let settlement = generate_settlement(config, &mut rng);

    println!(
        "Settlement ({:?}) {}x{} — seed {}",
        settlement.config.tier, settlement.width, settlement.height, seed
    );
    println!("Buildings: {}", settlement.buildings.len());
    for (i, building) in settlement.buildings.iter().enumerate() {
        println!(
            "  [{}] {} at ({}, {}) faction={:?}",
            i, building.prefab_name, building.x, building.y, building.faction
        );
    }
    println!();

    // Stamp onto a map and render
    let mut map = Map::generate_from_world(
        &mut ChaCha8Rng::seed_from_u64(seed),
        Biome::Saltflat,
        Terrain::Flat,
        0,
    )
    .0;
    clear_settlement_footprint(&mut map, &settlement, (0, 0));
    stamp_settlement(&mut map, &settlement);
    place_decorations(&mut map, &settlement, &mut ChaCha8Rng::seed_from_u64(seed));

    for y in 0..settlement.height {
        let row: String = (0..settlement.width)
            .map(|x| match map.get(x as i32, y as i32) {
                Some(Tile::Floor { .. }) => '.',
                Some(Tile::Wall { .. }) => '#',
                Some(Tile::Glass) => 'g',
                Some(Tile::Glare) => 'G',
                Some(Tile::StairsDown) => '>',
                Some(Tile::StairsUp) => '<',
                Some(Tile::WorldExit) => 'X',
                _ => ' ',
            })
            .collect();
        println!("{}", row);
    }
}
