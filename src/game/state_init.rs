//! GameState constructors — moved from state.rs to reduce its LOC.

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::{HashMap, HashSet};

use super::{
    adaptation::Adaptation,
    chest::Chest,
    enemy::Enemy,
    generation::{
        distribute_points_grid, generate_loot, get_biome_spawn_table,
        place_microstructures, weighted_pick_by_level_and_tier,
    },
    item::Item,
    lighting::{LightSource, compute_lighting},
    map::{Map, Tile},
    npc::Npc,
    player_state::PlayerState,
    state::{DebugState, GameMessage, GameState, MsgType, PendingUi, SpatialIndex},
    world_map::WorldMap,
    world_state::{Weather, WorldState},
};
use crate::game::{
    map_features::MapFeatures,
    narrative_engine::NarrativeEngine,
    storm::Storm,
};

impl GameState {
    pub fn new(seed: u64) -> Self {
        // Generate world map
        let world_map = WorldMap::generate(seed);
        let world_x = super::world_map::WORLD_WIDTH / 2;
        let world_y = super::world_map::WORLD_HEIGHT / 2;

        // Get world context for starting tile
        let (biome, terrain, elevation, poi, _resources, _connected, level) =
            world_map.get(world_x, world_y);

        // Generate tile map using world context
        let tile_seed = world_map.tile_seed(world_x, world_y);
        let mut rng = ChaCha8Rng::seed_from_u64(tile_seed);
        let (mut map, rooms) =
            Map::generate_from_world_with_poi(&mut rng, biome, terrain, elevation, poi);
        let (px, py) = rooms[0];
        // Clamp spawn point away from map edges to ensure full 5x5 clearing fits
        let px = px.max(3).min(map.width as i32 - 4);
        let py = py.max(3).min(map.height as i32 - 4);

        // Clear 5x5 area around player spawn to ensure walkable space
        for dy in -2..=2 {
            for dx in -2..=2 {
                let cx = px + dx;
                let cy = py + dy;
                if cx >= 1 && cy >= 1 && cx < map.width as i32 - 1 && cy < map.height as i32 - 1 {
                    let idx = map.idx(cx, cy);
                    if !map.tiles[idx].walkable() {
                        map.tiles[idx] = Tile::default_floor();
                    }
                }
            }
        }

        // Add world exit to starting tile (near spawn point)
        let exit_x = (px + 1).min(map.width as i32 - 1) as usize;
        let exit_y = py as usize;
        map.tiles[exit_y * map.width + exit_x] = Tile::WorldExit;

        let visible = crate::game::map::compute_fov(&map, px, py);
        let table = get_biome_spawn_table(&biome);

        // Spawn enemies (fewer on starting tile for hospitable start)
        let mut enemies = Vec::new();
        let max_enemies = 8; // Limit total enemies regardless of clearing count
        let safe_distance = 15; // Minimum distance from player spawn
        let (px, py) = rooms[0]; // Player spawn position

        let safe_rooms: Vec<_> = rooms
            .iter()
            .filter(|&&(rx, ry)| {
                let dx = (rx - px).abs();
                let dy = (ry - py).abs();
                dx >= safe_distance || dy >= safe_distance
            })
            .cloned()
            .collect();

        // Use spatial distribution to spread out enemy spawns
        let distributed_positions = distribute_points_grid(
            &safe_rooms,
            max_enemies,
            20, // Minimum distance between enemies
            &mut rng,
        );

        for (rx, ry) in distributed_positions {
            if let Some(id) =
                weighted_pick_by_level_and_tier(&table.enemies, level, &mut rng, false)
            {
                enemies.push(Enemy::new(rx, ry, id));
            }
        }

        // Spawn NPCs
        let mut npcs = Vec::new();

        // Spawn other NPCs from spawn table
        let late_room = rooms.len().saturating_sub(2);
        for spawn in &table.npcs {

            let room_idx = match spawn.room.as_deref() {
                Some("late") => Some(late_room),
                Some("last") => Some(rooms.len() - 1),
                Some("first") => Some(0),
                _ => {
                    if rng.gen_ratio(spawn.weight.min(10), 10) {
                        Some(rng.gen_range(1..rooms.len()))
                    } else {
                        None
                    }
                }
            };
            if let Some(idx) = room_idx
                && idx < rooms.len()
            {
                let (rx, ry) = rooms[idx];
                // If spawning in first room (where player is), find adjacent position
                let (npc_x, npc_y) = if idx == 0 {
                    // Try adjacent positions around the room center
                    let offsets = [
                        (1, 0),
                        (-1, 0),
                        (0, 1),
                        (0, -1),
                        (1, 1),
                        (-1, -1),
                        (1, -1),
                        (-1, 1),
                    ];
                    let mut spawn_pos = (rx, ry);
                    for &(dx, dy) in &offsets {
                        let test_x = rx + dx;
                        let test_y = ry + dy;
                        if test_x >= 0
                            && test_y >= 0
                            && test_x < map.width as i32
                            && test_y < map.height as i32
                        {
                            let test_idx = map.idx(test_x, test_y);
                            if map.tiles[test_idx].walkable() &&
                                   (test_x != px || test_y != py) // Don't spawn on player
                        {
                                spawn_pos = (test_x, test_y);
                                break;
                            }
                        }
                    }
                    spawn_pos
                } else {
                    (rx, ry)
                };
                npcs.push(Npc::new(npc_x, npc_y, &spawn.id));
            }
        }

        // Spawn items (more on starting tile for hospitable start)
        let mut items = Vec::new();
        let mut used_positions = HashSet::new();

        for spawn in &table.items {
            if let Some("first") = spawn.room.as_deref() {
                // Spawn near player start position
                let offsets = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                for &(dx, dy) in &offsets {
                    let ix = px + dx;
                    let iy = py + dy;
                    if !used_positions.contains(&(ix, iy)) {
                        used_positions.insert((ix, iy));
                        items.push(Item::new(ix, iy, &spawn.id));
                        break;
                    }
                }
                continue;
            }
            if let Some("last") = spawn.room.as_deref() {
                if let Some(&(rx, ry)) = rooms.last()
                    && !used_positions.contains(&(rx, ry))
                {
                    used_positions.insert((rx, ry));
                    // Check tier eligibility for last room items
                    if let Some(item_def) = super::item::get_item_def(&spawn.id) {
                        let tier_threshold = match level {
                            1 => 1,
                            2..=3 => 2,
                            4..=6 => 3,
                            7..=8 => 4,
                            9..=10 => 5,
                            _ => 1,
                        };
                        if item_def.tier <= tier_threshold {
                            items.push(Item::new(rx, ry, &spawn.id));
                        }
                    }
                }
                continue;
            }
            for _ in 0..(spawn.weight + 1) {
                // +1 for hospitable start
                if let Some(&(rx, ry)) = rooms.get(rng.gen_range(1..rooms.len())) {
                    let ix = rx + rng.gen_range(-1..=1);
                    let iy = ry + rng.gen_range(-1..=1);
                    if !used_positions.contains(&(ix, iy)) {
                        used_positions.insert((ix, iy));
                        // Check tier eligibility for regular items
                        if let Some(item_def) = super::item::get_item_def(&spawn.id) {
                            let tier_threshold = match level {
                                1 => 1,
                                2..=3 => 2,
                                4..=6 => 3,
                                7..=8 => 4,
                                9..=10 => 5,
                                _ => 1,
                            };
                            if item_def.tier <= tier_threshold {
                                items.push(Item::new(ix, iy, &spawn.id));
                            }
                        }
                    }
                }
            }
        }

        // Spawn chests in some rooms
        let mut chests = Vec::new();
        let chest_rooms: Vec<_> = rooms
            .iter()
            .skip(2) // Skip first two rooms (player start and adjacent)
            .take(3) // Limit to 3 chests per tile
            .collect();

        for &(rx, ry) in chest_rooms {
            if rng.gen_ratio(3, 10) {
                // 30% chance for chest in each room
                let chest_types = ["wooden_chest", "supply_crate", "glass_cache"];
                let chest_id = chest_types[rng.gen_range(0..chest_types.len())];

                // Generate loot for the chest
                let mut chest = Chest::new(rx, ry, chest_id);
                if let Some(def) = super::chest::get_chest_def(chest_id)
                    && let Some(loot_table) = &def.loot_table
                {
                    let loot = generate_loot(loot_table, rx, ry, &mut rng);
                    for item in loot {
                        chest.add_item(item);
                    }
                }
                chests.push(chest);
            }
        }

        // Place micro-structures
        let biome_str = match biome {
            super::world_map::Biome::Saltflat => "saltflat",
            super::world_map::Biome::Oasis => "oasis",
            super::world_map::Biome::Ruins => "ruins",
            super::world_map::Biome::Scrubland => "scrubland",
            _ => "saltflat",
        };

        let (microstructures, mut structure_npcs, mut structure_chests, mut structure_items) =
            place_microstructures(&mut map, biome_str, &rooms, (px, py), &mut rng);

        // Add structure entities to main collections
        npcs.append(&mut structure_npcs);
        chests.append(&mut structure_chests);
        items.append(&mut structure_items);

        let ambient = 100u8;
        let light_sources = vec![LightSource {
            x: px,
            y: py,
            radius: 8,
            intensity: 120,
        }]; // Reduced from 150 to avoid glare
        let light_map = compute_lighting(&light_sources, ambient);

        let mut player = PlayerState::new();
        player.x = px;
        player.y = py;
        player.hp = 20;
        player.max_hp = 20;
        player.reflex = 5;
        player.salt_scrip = 50;
        if let Some(first_quest) = super::quest::ActiveQuest::new("pilgrims_last_angle") {
            player.quest_log.active.push(first_quest);
        }

        let world = WorldState {
            world_map: Some(world_map),
            world_x,
            world_y,
            layer: 0,
            map,
            enemies,
            npcs,
            items,
            chests,
            interactables: Vec::new(),
            microstructures,
            storm: Storm::forecast(&mut ChaCha8Rng::seed_from_u64(seed + 1)),
            time_of_day: 8,
            weather: Weather::Clear,
            ambient_light: ambient,
            visual_effects: super::visual_effects::VisualEffects::default(),
            light_map: light_map.clone(),
            encounter_state: None,
            encounter_history: HashMap::new(),
            total_tiles_traveled: 0,
            world_map_target: None,
            world_map_path: Vec::new(),
            saved_on_world_map: false,
            enemy_positions: HashMap::new(),
            npc_positions: HashMap::new(),
            item_positions: HashMap::new(),
            chest_positions: HashMap::new(),
            interactable_positions: HashMap::new(),
        };

        let mut state = Self {
            player,
            world,
            visible: visible.clone(),
            revealed: visible,
            light_map,
            messages: vec![
                GameMessage::new("Welcome to the Saltglass Steppe.", MsgType::System, 0),
                GameMessage::new("Quest added: The Pilgrim's Last Angle", MsgType::System, 0),
            ],
            turn: 0,
            rng,
            triggered_effects: Vec::new(),
            decoys: Vec::new(),
            spatial: SpatialIndex { dirty: true, ..Default::default() },
            debug: DebugState::default(),
            pending_ui: PendingUi::default(),
            meta: super::meta::MetaProgress::load(),
            wait_counter: 0,
            narrative: NarrativeEngine::default(),
            map_features: MapFeatures::new(),
            seed,
            trace: Default::default(),
            mutation_log: Vec::new(),
        };

        // Materialize terrain-forge markers into entities
        crate::game::generation::feature_materializer::materialize_features(
            &mut state, biome, terrain, poi, level,
        );

        state.rebuild_spatial_index();
        state
    }
    pub fn new_with_class(seed: u64, class_id: &str) -> Self {
        let mut state = Self::new(seed);

        if let Some(class) = super::meta::get_class(class_id) {
            state.player.hp = class.starting_hp;
            state.player.max_hp = class.starting_hp;
            state.player.ap = class.starting_ap;
            state.player.max_ap = class.starting_ap;

            // Add starting items
            for item_id in &class.starting_items {
                state.player.inventory.push(item_id.clone());
            }

            // Add starting adaptations
            for adapt_id in &class.starting_adaptations {
                if let Some(adapt) = Adaptation::from_id(adapt_id) {
                    state.player.adaptations.push(adapt);
                }
            }

            // Add starting faction reputation
            let starting_rep = super::faction::get_starting_reputation(class_id);
            for (faction_id, rep) in starting_rep {
                state.player.faction_reputation.insert(faction_id, rep);
            }

            state.log(format!("You begin as a {}.", class.name));
        }

        state
    }

}
