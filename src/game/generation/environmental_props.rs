use once_cell::sync::Lazy;
use rand::Rng;
use serde::Deserialize;

use crate::game::map::{Map, Tile};

#[derive(Debug, Deserialize)]
struct PropDef {
    id: String,
    biomes: Option<Vec<String>>,
    place_on: Vec<String>,
    density: f64,
    cluster_chance: f64,
    cluster_size: [usize; 2],
}

#[derive(Deserialize)]
struct PropsFile {
    #[allow(dead_code)]
    schema: String,
    props: Vec<PropDef>,
}

static PROPS: Lazy<Vec<PropDef>> = Lazy::new(|| {
    let data =
        std::fs::read_to_string("data/environmental_props.json").expect("Failed to read environmental_props.json");
    let file: PropsFile =
        serde_json::from_str(&data).expect("Failed to parse environmental_props.json");
    file.props
});

const SPAWN_EXCLUSION: i32 = 5;

/// Place environmental props on a generated map. Runs after terrain generation,
/// before microstructures. Props are floor tile replacements — purely visual.
pub fn place_environmental_props(
    map: &mut Map,
    biome: &str,
    spawn_pos: (i32, i32),
    rng: &mut impl Rng,
) {
    let eligible: Vec<&PropDef> = PROPS
        .iter()
        .filter(|p| match &p.biomes {
            Some(biomes) => biomes.iter().any(|b| b == biome),
            None => true,
        })
        .collect();

    if eligible.is_empty() {
        return;
    }

    let (w, h) = (map.width, map.height);

    for y in 0..h {
        for x in 0..w {
            let dx = (x as i32 - spawn_pos.0).abs();
            let dy = (y as i32 - spawn_pos.1).abs();
            if dx < SPAWN_EXCLUSION && dy < SPAWN_EXCLUSION {
                continue;
            }

            let floor_id = match &map.tiles[y * w + x] {
                Tile::Floor { id } => id.clone(),
                _ => continue,
            };

            for prop in &eligible {
                if !prop.place_on.contains(&floor_id) {
                    continue;
                }
                if !rng.gen_bool(prop.density) {
                    continue;
                }

                map.set_tile(x, y, Tile::Floor { id: prop.id.clone() });

                // Cluster placement
                if prop.cluster_chance > 0.0 && rng.gen_bool(prop.cluster_chance) {
                    let count = rng.gen_range(prop.cluster_size[0]..=prop.cluster_size[1]);
                    place_cluster(map, x, y, &prop.id, &prop.place_on, count, rng);
                }

                break; // one prop per tile
            }
        }
    }
}

fn place_cluster(
    map: &mut Map,
    cx: usize,
    cy: usize,
    prop_id: &str,
    place_on: &[String],
    count: usize,
    rng: &mut impl Rng,
) {
    let (w, h) = (map.width, map.height);
    let mut placed = 0;
    // Spiral outward from center
    for r in 1..=3i32 {
        if placed >= count {
            break;
        }
        for dy in -r..=r {
            for dx in -r..=r {
                if placed >= count {
                    break;
                }
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let idx = ny as usize * w + nx as usize;
                if let Tile::Floor { id } = &map.tiles[idx] && place_on.contains(id) && rng.gen_bool(0.5) {
                    map.tiles[idx] = Tile::Floor {
                        id: prop_id.to_string(),
                    };
                    placed += 1;
                }
            }
        }
    }
}
