use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

use super::{
    TerrainForgeGenerator,
    connectivity::{GSBParams, ensure_connectivity},
    distribute_points_grid, get_biome_spawn_table, place_microstructures,
    settlement::{
        SettlementConfig, SettlementTier, clear_settlement_footprint, generate_settlement,
        paint_roads, place_decorations, stamp_settlement,
    },
    structure_library::StructureLibrary,
    weighted_pick_by_level_and_tier,
};
use crate::game::{
    chest::Chest,
    constants::{MAP_HEIGHT, MAP_WIDTH},
    enemy::Enemy,
    item::Item,
    map::{Map, Tile},
    npc::Npc,
    world_map::{Biome, POI, Terrain},
};

pub struct TileParams {
    pub seed: u64,
    pub biome: Biome,
    pub terrain: Terrain,
    pub elevation: u8,
    pub poi: POI,
    pub level: u32,
    pub faction_control: Vec<(String, f32)>,
    pub quest_ids: Vec<String>,
}

pub struct GeneratedTile {
    pub map: Map,
    pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub chests: Vec<Chest>,
    pub spawn_pos: (i32, i32),
    pub walkable_positions: Vec<(i32, i32)>,
}

impl TileParams {
    pub fn from_world_state(state: &crate::game::state::GameState, wx: usize, wy: usize) -> Self {
        let world_map = state.world.world_map.as_ref().unwrap();
        let (biome, terrain, elevation, poi, _resources, _connected, level) = world_map.get(wx, wy);
        let tile_seed = world_map.tile_seed(wx, wy);
        let faction_control = match world_map.get_faction_territory(wx, wy) {
            Some(f) => vec![(f.to_string(), 1.0f32)],
            None => vec![],
        };
        let quest_ids = state.get_quest_ids_for_location(wx, wy);

        Self {
            seed: tile_seed,
            biome,
            terrain,
            elevation,
            poi,
            level,
            faction_control,
            quest_ids,
        }
    }
}

fn find_safe_spawn_position(map: &Map) -> (i32, i32) {
    // Try center area first
    for y in (MAP_HEIGHT / 2 - 10)..(MAP_HEIGHT / 2 + 10) {
        for x in (MAP_WIDTH / 2 - 10)..(MAP_WIDTH / 2 + 10) {
            let idx = y * MAP_WIDTH + x;
            if idx < map.tiles.len() && map.tiles[idx].walkable() {
                return (x as i32, y as i32);
            }
        }
    }

    // Fallback: find any walkable tile
    for (idx, tile) in map.tiles.iter().enumerate() {
        if tile.walkable() {
            let x = idx % MAP_WIDTH;
            let y = idx / MAP_WIDTH;
            return (x as i32, y as i32);
        }
    }

    // Ultimate fallback
    (MAP_WIDTH as i32 / 2, MAP_HEIGHT as i32 / 2)
}

pub fn generate_tile(params: &TileParams) -> GeneratedTile {
    let mut rng = ChaCha8Rng::seed_from_u64(params.seed);

    // Generate new tile map via terrain-forge adapter
    let mut map = {
        let generator = TerrainForgeGenerator::new();
        let (map, _) = generator.generate_tile_with_seed(
            params.biome,
            params.terrain,
            params.elevation,
            params.poi,
            params.seed,
            &params.quest_ids,
        );
        map
    };

    // Find safe spawn position
    let (px, py) = find_safe_spawn_position(&map);

    // Collect walkable positions for later use
    let walkable_positions: Vec<(i32, i32)> = map
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(idx, tile)| {
            if tile.walkable() {
                let x = (idx % map.width) as i32;
                let y = (idx / map.width) as i32;
                Some((x, y))
            } else {
                None
            }
        })
        .collect();

    // Spawn enemies based on POI and quest structure data
    let mut enemies = Vec::new();

    // Check for quest structure spawns first
    if let Some(spawn_data) = map.metadata.get("vitrified_library_spawns") {
        if let Ok(spawns) = serde_json::from_str::<Vec<(i32, i32, String, String)>>(spawn_data) {
            for (x, y, spawn_type, id) in spawns {
                if spawn_type == "enemy" {
                    enemies.push(Enemy::new(x, y, &id));
                }
            }
        }
    }

    // Add regular enemies if not a quest structure location
    if enemies.is_empty() {
        let table = get_biome_spawn_table(&params.biome);
        let enemy_count = match params.poi {
            POI::Town => 0,
            POI::Shrine => 2,
            POI::Landmark => 3,
            POI::Dungeon => 5,
            POI::None => 3,
        };

        let safe_positions: Vec<(i32, i32)> = walkable_positions
            .iter()
            .filter(|&&(x, y)| {
                let dx = (x - px).abs();
                let dy = (y - py).abs();
                dx >= 15 || dy >= 15
            })
            .cloned()
            .collect();

        let distributed_positions =
            distribute_points_grid(&safe_positions, enemy_count, 20, &mut rng);

        for (rx, ry) in distributed_positions {
            if let Some(id) =
                weighted_pick_by_level_and_tier(&table.enemies, params.level, &mut rng, false)
            {
                enemies.push(Enemy::new(rx, ry, id));
            }
        }
    }

    // Spawn items
    let mut items = Vec::new();
    let mut used_positions = HashSet::new();

    if let Some(spawn_data) = map.metadata.get("vitrified_library_spawns") {
        if let Ok(spawns) = serde_json::from_str::<Vec<(i32, i32, String, String)>>(spawn_data) {
            for (x, y, spawn_type, id) in spawns {
                if spawn_type == "item" {
                    items.push(Item::new(x, y, &id));
                    used_positions.insert((x, y));
                }
            }
        }
    }

    if items.len() < 3 {
        let table = get_biome_spawn_table(&params.biome);
        for spawn in &table.items {
            for _ in 0..spawn.weight {
                let mut attempts = 0;
                while attempts < 10 {
                    let idx = rng.gen_range(0..map.tiles.len());
                    if map.tiles[idx].walkable() {
                        let ix = (idx % map.width) as i32;
                        let iy = (idx / map.width) as i32;
                        if !used_positions.contains(&(ix, iy)) {
                            used_positions.insert((ix, iy));
                            if let Some(item_def) = crate::game::item::get_item_def(&spawn.id) {
                                let tier_threshold = match params.level {
                                    1 => 1,
                                    2..=3 => 2,
                                    4..=6 => 3,
                                    7..=8 => 4,
                                    9..=10 => 5,
                                    _ => 1,
                                };
                                if item_def.tier <= tier_threshold {
                                    items.push(Item::new(ix, iy, &spawn.id));
                                    break;
                                }
                            }
                        }
                    }
                    attempts += 1;
                }
            }
        }
    }

    let biome_str = params.biome.as_str();
    let (_microstructures, mut structure_npcs, structure_chests, mut structure_items) =
        place_microstructures(&mut map, biome_str, &walkable_positions, (px, py), &mut rng);
    items.append(&mut structure_items);

    // Spawn biome NPCs
    let mut npcs = Vec::new();
    let npc_table = get_biome_spawn_table(&params.biome);
    for spawn in &npc_table.npcs {
        if spawn.weight > 0 && rng.gen_ratio(spawn.weight.min(10), 10) {
            if let Some(&(nx, ny)) = walkable_positions
                .iter()
                .filter(|&&(x, y)| {
                    let dx = (x - px).abs();
                    let dy = (y - py).abs();
                    dx >= 8 || dy >= 8
                })
                .nth(rng.gen_range(0..walkable_positions.len().max(1)))
            {
                npcs.push(Npc::new(nx, ny, &spawn.id));
            }
        }
    }
    npcs.append(&mut structure_npcs);

    // Stamp settlement buildings for towns
    if params.poi == POI::Town {
        let config = SettlementConfig {
            seed: params.seed,
            tier: SettlementTier::Town,
            faction_control: params.faction_control.clone(),
        };
        let mut settlement_rng = ChaCha8Rng::seed_from_u64(params.seed);
        let mut settlement = generate_settlement(config, &mut settlement_rng);

        // Center settlement on the map
        let ox = (map.width as i32 - settlement.width as i32) / 2;
        let oy = (map.height as i32 - settlement.height as i32) / 2;
        for b in &mut settlement.buildings {
            b.x += ox;
            b.y += oy;
        }
        clear_settlement_footprint(&mut map, &settlement);
        stamp_settlement(&mut map, &settlement);
        paint_roads(&mut map, &settlement);
        place_decorations(&mut map, &settlement, &mut settlement_rng);

        // Path from spawn to nearest settlement walkable tile using A*
        if let Some((tx, ty)) = map
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if t.walkable() {
                    let x = (i % map.width) as i32;
                    let y = (i / map.width) as i32;
                    if x < settlement.width as i32 && y < settlement.height as i32 {
                        Some((x, y))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .min_by_key(|&(x, y)| (x - px).abs() + (y - py).abs())
        {
            use crate::game::generation::settlement::road_pathfinding;
            let costs = road_pathfinding::build_cost_grid(&map);
            if let Some(path) =
                road_pathfinding::astar_path(&costs, map.width, map.height, (px, py), (tx, ty))
            {
                for (cx, cy) in path {
                    if cx >= 0 && cy >= 0 && cx < map.width as i32 && cy < map.height as i32 {
                        if !matches!(map.get_tile(cx, cy), Tile::Floor { id } if id == "dirt_path")
                        {
                            map.set_tile(
                                cx as usize,
                                cy as usize,
                                Tile::Floor {
                                    id: "dirt_path".to_string(),
                                },
                            );
                        }
                    }
                }
            }
        }

        // Refresh walkable_positions after stamping
        let walkable_positions: Vec<(i32, i32)> = map
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| {
                if tile.walkable() {
                    Some(((idx % map.width) as i32, (idx / map.width) as i32))
                } else {
                    None
                }
            })
            .collect();

        if let Ok(library) = StructureLibrary::load() {
            for building in &settlement.buildings {
                if let Some(structure) = library.get(&building.prefab_name) {
                    for npc_type in &structure.metadata.npc_types {
                        if let Some(&(nx, ny)) = walkable_positions
                            .iter()
                            .min_by_key(|&&(x, y)| {
                                let dx = (x - building.x).abs();
                                let dy = (y - building.y).abs();
                                dx + dy
                            })
                            .filter(|&&pos| {
                                !npcs.iter().any(|npc| npc.x == pos.0 && npc.y == pos.1)
                            })
                        {
                            npcs.push(Npc::new(nx, ny, npc_type));
                        }
                    }
                }
            }
        }
    }

    // Final connectivity pass — carve tunnels to connect any isolated regions
    ensure_connectivity(&mut map, (px, py), &GSBParams::fast(), &mut rng);

    // Refresh walkable_positions for the returned struct
    let walkable_positions: Vec<(i32, i32)> = map
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(idx, tile)| {
            if tile.walkable() {
                Some(((idx % map.width) as i32, (idx / map.width) as i32))
            } else {
                None
            }
        })
        .collect();

    GeneratedTile {
        map,
        enemies,
        npcs,
        items,
        chests: structure_chests,
        spawn_pos: (px, py),
        walkable_positions,
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TileTestConfig {
    pub name: String,
    pub biome: Biome,
    pub terrain: Terrain,
    pub elevation: u8,
    pub poi: POI,
    pub level: u32,
    pub faction_territory: Option<String>,
    pub seed: Option<u64>,
}

impl TileTestConfig {
    /// Load all configs from data/tile_tests/*.json
    pub fn load_all() -> Vec<TileTestConfig> {
        let dir = std::path::Path::new("data/tile_tests");
        let mut configs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(text) = std::fs::read_to_string(entry.path()) {
                        if let Ok(cfg) = serde_json::from_str::<TileTestConfig>(&text) {
                            configs.push(cfg);
                        }
                    }
                }
            }
        }
        configs.sort_by(|a, b| a.name.cmp(&b.name));
        configs
    }

    pub fn to_tile_params(&self) -> TileParams {
        let seed = self.seed.unwrap_or_else(|| {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            self.name.hash(&mut h);
            h.finish()
        });
        TileParams {
            seed,
            biome: self.biome,
            terrain: self.terrain,
            elevation: self.elevation,
            poi: self.poi,
            level: self.level,
            faction_control: self
                .faction_territory
                .as_ref()
                .map(|f| vec![(f.clone(), 1.0f32)])
                .unwrap_or_default(),
            quest_ids: vec![],
        }
    }
}
