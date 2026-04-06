//! World travel, encounter, and subterranean navigation.
//! All functions take `&mut GameState` — moved from state.rs to reduce its LOC.

use crate::game::{
    enemy::Enemy,
    item::Item,
    map::{Map, Tile},
    npc::Npc,
    state::{GameState, MsgType},
};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Tile travel
// ---------------------------------------------------------------------------

pub fn travel_to_tile(state: &mut GameState, new_wx: usize, new_wy: usize) {
    use crate::game::generation::tile_generator::{TileParams, generate_tile};

    let params = TileParams::from_world_state(state, new_wx, new_wy);
    let tile = generate_tile(&params);

    state.world.world_x = new_wx;
    state.world.world_y = new_wy;
    state.world.map = tile.map;
    state.world.enemies = tile.enemies;
    state.world.items = tile.items;
    state.world.npcs = tile.npcs;
    state.world.chests = tile.chests;
    state.player.x = tile.spawn_pos.0;
    state.player.y = tile.spawn_pos.1;

    let biome = params.biome;
    let terrain = params.terrain;
    let poi = params.poi;
    let level = params.level;
    let walkable = tile.walkable_positions;
    let mut rng = ChaCha8Rng::seed_from_u64(params.seed);

    crate::game::generation::feature_materializer::materialize_features(
        state, biome, terrain, poi, level,
    );
    if poi == crate::game::world_map::POI::Town {
        spawn_crafting_stations(state, &walkable, &mut rng);
    }
    spawn_quest_required_npcs(state);
    state.update_fov();
    state.rebuild_spatial_index();
    state.update_lighting();
    crate::game::systems::world::generate_crystal_formations(state, &biome, &walkable, &mut rng);
    state.log(format!("You enter a new area ({:?} {:?}).", biome, terrain));
}

pub fn travel_to_tile_safe(state: &mut GameState, new_wx: usize, new_wy: usize) {
    use crate::game::travel;

    let from = (state.world.world_x, state.world.world_y);
    let to = (new_wx, new_wy);

    if !travel::is_adjacent(from, to) {
        state.log("Too far to travel in one step. Move to an adjacent tile.");
        return;
    }

    if let Some(wm) = &state.world.world_map {
        let (biome, terrain, _elev, _poi, _res, _conn, level) = wm.get(new_wx, new_wy);
        let cost = travel::travel_cost(terrain, biome);
        state.turn += cost;
        state.world.total_tiles_traveled += 1;

        let last_encounter = state.world.encounter_history.get(&(new_wx, new_wy)).copied().unwrap_or(0);
        if crate::game::encounter::should_trigger_encounter(
            state.seed, new_wx, new_wy, state.world.total_tiles_traveled,
            level, last_encounter, state.turn,
            state.player.skills.get_skill_level("wayfaring"),
        ) {
            let encounter = crate::game::encounter::generate_encounter(
                state.seed, new_wx, new_wy, state.world.total_tiles_traveled,
                level, biome.as_str(),
            );
            match &encounter.encounter_type {
                crate::game::encounter::EncounterType::Hostile { threat_points } =>
                    state.log_typed(format!("⚔ Hostile encounter! (Threat: {})", threat_points), MsgType::Warning),
                crate::game::encounter::EncounterType::Neutral { description, .. } =>
                    state.log_typed(description.clone(), MsgType::System),
                crate::game::encounter::EncounterType::Beneficial { boon_points } =>
                    state.log_typed(format!("✨ You discover something! (Value: {})", boon_points), MsgType::Loot),
            }
            state.world.encounter_state = Some(encounter);
            state.world.encounter_history.insert((new_wx, new_wy), state.turn);
        }

        state.log(format!("Traveled to {:?} {:?} ({cost} turns).", terrain, biome));
    }

    travel_to_tile(state, new_wx, new_wy);

    if state.world.encounter_state.is_some() {
        spawn_encounter_entities(state);
    }

    // Ensure player is on a walkable tile
    let (mut px, mut py) = (state.player.x, state.player.y);
    let is_safe = |map: &Map, enemies: &[Enemy], x: i32, y: i32| -> bool {
        matches!(map.get(x, y), Some(t) if t.walkable())
            && !enemies.iter().any(|e| e.x == x && e.y == y && e.hp > 0)
    };
    if !is_safe(&state.world.map, &state.world.enemies, px, py) {
        'search: for radius in 1i32..20 {
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx.abs() == radius || dy.abs() == radius {
                        let (nx, ny) = (px + dx, py + dy);
                        if is_safe(&state.world.map, &state.world.enemies, nx, ny) {
                            px = nx; py = ny;
                            break 'search;
                        }
                    }
                }
            }
        }
    }
    state.player.x = px;
    state.player.y = py;
    state.update_fov();
}

// ---------------------------------------------------------------------------
// Spawning helpers
// ---------------------------------------------------------------------------

pub fn spawn_crafting_stations(state: &mut GameState, walkable: &[(i32, i32)], rng: &mut ChaCha8Rng) {
    use rand::seq::SliceRandom;
    let station_ids = ["crafting_table", "glass_forge"];
    let occupied: std::collections::HashSet<(i32, i32)> = state.world.interactables.iter()
        .map(|i| (i.x, i.y))
        .chain(state.world.npcs.iter().map(|n| (n.x, n.y)))
        .collect();
    let free: Vec<_> = walkable.iter().filter(|p| !occupied.contains(p)).collect();
    for id in &station_ids {
        if let Some(&&(x, y)) = free.choose(rng) {
            state.world.interactables.push(
                crate::game::interactable::Interactable::new(id.to_string(), x, y)
            );
        }
    }
}

pub fn spawn_quest_required_npcs(state: &mut GameState) {
    let mut required_npcs = Vec::new();
    for quest in &state.player.quest_log.active {
        if let Some(def) = crate::game::quest::get_quest_def(&quest.quest_id) {
            for objective in &def.objectives {
                if let crate::game::quest::ObjectiveType::TalkTo { npc_id } = &objective.objective_type
                    && !state.world.npcs.iter().any(|npc| npc.id == *npc_id) {
                        required_npcs.push(npc_id.clone());
                    }
            }
        }
    }
    for npc_id in required_npcs {
        if let Some((x, y)) = find_safe_spawn_position(state) {
            state.world.npcs.push(Npc::new(x, y, &npc_id));
            state.log(format!("A {} appears nearby.", npc_id.replace('_', " ")));
        }
    }
}

pub fn find_safe_spawn_position(state: &GameState) -> Option<(i32, i32)> {
    let offsets = [(1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,-1),(1,-1),(-1,1),(2,0),(-2,0),(0,2),(0,-2)];
    for &(dx, dy) in &offsets {
        let (x, y) = (state.player.x + dx, state.player.y + dy);
        if x >= 0 && y >= 0 && x < state.world.map.width as i32 && y < state.world.map.height as i32 {
            let idx = state.world.map.idx(x, y);
            if state.world.map.tiles[idx].walkable()
                && !state.world.enemies.iter().any(|e| e.x == x && e.y == y)
                && !state.world.npcs.iter().any(|n| n.x == x && n.y == y)
                && !state.world.items.iter().any(|i| i.x == x && i.y == y)
            {
                return Some((x, y));
            }
        }
    }
    None
}

pub fn spawn_encounter_entities(state: &mut GameState) {
    use crate::game::generation::spawn::{get_biome_spawn_table, weighted_pick_by_level_and_tier};
    use rand::seq::SliceRandom;

    let encounter = match &state.world.encounter_state {
        Some(e) => e.clone(),
        None => return,
    };
    let (biome, _, _, _, _, _, level) = match &state.world.world_map {
        Some(wm) => wm.get(encounter.world_x, encounter.world_y),
        None => return,
    };

    match &encounter.encounter_type {
        crate::game::encounter::EncounterType::Hostile { threat_points } => {
            let table = get_biome_spawn_table(&biome);
            let mut remaining = *threat_points;
            let mut spawned = Vec::new();
            let positions: Vec<(i32, i32)> = state.world.map.tiles.iter().enumerate()
                .filter_map(|(idx, tile)| {
                    if tile.walkable() {
                        let x = (idx % state.world.map.width) as i32;
                        let y = (idx / state.world.map.width) as i32;
                        if (x - state.player.x).abs() + (y - state.player.y).abs() >= 15 {
                            Some((x, y))
                        } else { None }
                    } else { None }
                }).collect();
            let mut pos_idx = 0;
            while remaining > 0 && pos_idx < positions.len() {
                if let Some(id) = weighted_pick_by_level_and_tier(&table.enemies, level, &mut state.rng, false) {
                    let cost = (level * 2).min(remaining);
                    remaining = remaining.saturating_sub(cost);
                    let (x, y) = positions[pos_idx];
                    let ei = state.world.enemies.len();
                    state.world.enemies.push(Enemy::new(x, y, id));
                    spawned.push(ei);
                    pos_idx += 1;
                } else { break; }
            }
            state.log(format!("Encounter spawned {} enemies (threat: {})", spawned.len(), threat_points));
            if let Some(enc) = &mut state.world.encounter_state { enc.spawned_enemies = spawned; }
            state.rebuild_spatial_index();
        }
        crate::game::encounter::EncounterType::Neutral { event_id, .. } => {
            if event_id == "trade_caravan" {
                let count = state.rng.gen_range(1..=2);
                for _ in 0..count {
                    if let Some((x, y)) = find_safe_spawn_position(state) {
                        state.world.npcs.push(Npc::new(x, y, "traveling_merchant"));
                    }
                }
                state.rebuild_spatial_index();
            } else if event_id == "animal_herd" {
                state.log("A herd of creatures grazes peacefully nearby.");
            }
        }
        crate::game::encounter::EncounterType::Beneficial { boon_points } => {
            let table = get_biome_spawn_table(&biome);
            let mut remaining = *boon_points;
            let mut spawned = Vec::new();
            while remaining > 0 && !table.items.is_empty() {
                if let Some(item_spawn) = table.items.choose(&mut state.rng) {
                    if let Some(def) = crate::game::item::get_item_def(&item_spawn.id) {
                        let val = def.value.min(remaining);
                        remaining = remaining.saturating_sub(val);
                        if let Some((x, y)) = find_safe_spawn_position(state) {
                            let ii = state.world.items.len();
                            state.world.items.push(Item::new(x, y, &item_spawn.id));
                            spawned.push(ii);
                        }
                    }
                } else { break; }
            }
            if let Some(enc) = &mut state.world.encounter_state { enc.spawned_items = spawned; }
            state.rebuild_spatial_index();
        }
    }
}

// ---------------------------------------------------------------------------
// Encounter flee
// ---------------------------------------------------------------------------

pub fn attempt_flee_encounter(state: &mut GameState) -> Result<(), String> {
    let encounter = match &state.world.encounter_state {
        Some(e) => e.clone(),
        None => return Err("No active encounter.".to_string()),
    };
    if !encounter.can_flee(state.turn, 1.0) {
        return Err("You cannot flee yet!".to_string());
    }
    match crate::game::encounter::attempt_flee(
        state.player.x, state.player.y,
        &state.world.enemies, &encounter.spawned_enemies,
        &mut state.rng, state.player.skills.get_skill_level("wayfaring"),
    ) {
        Ok(()) => {
            state.world.encounter_state = None;
            state.log_typed("You successfully flee the encounter!", MsgType::Status);
            Ok(())
        }
        Err(e) => {
            if let Some(enc) = &mut state.world.encounter_state { enc.last_flee_attempt = state.turn; }
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// Subterranean
// ---------------------------------------------------------------------------

pub fn enter_subterranean(state: &mut GameState) -> bool {
    if state.world.map.get(state.player.x, state.player.y) != Some(&Tile::StairsDown) {
        return false;
    }
    state.world.layer -= 1;
    let seed = state.world.world_map.as_ref()
        .map(|wm| wm.tile_seed(state.world.world_x, state.world.world_y))
        .unwrap_or(42)
        .wrapping_add(state.world.layer.unsigned_abs() as u64 * 1000);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let (map, rooms) = Map::generate_subterranean(&mut rng, state.world.layer);
    let (px, py) = rooms[0];
    state.world.map = map;
    state.world.enemies = Vec::new();
    state.world.items = Vec::new();
    state.world.npcs = Vec::new();
    state.player.x = px;
    state.player.y = py;
    state.update_fov();
    state.rebuild_spatial_index();
    state.update_lighting();
    state.log(format!("You descend to level {}.", -state.world.layer));
    true
}

pub fn exit_subterranean(state: &mut GameState) -> bool {
    if state.world.map.get(state.player.x, state.player.y) != Some(&Tile::StairsUp) {
        return false;
    }
    if state.world.layer >= 0 { return false; }
    state.world.layer += 1;
    if state.world.layer == 0 {
        travel_to_tile(state, state.world.world_x, state.world.world_y);
        state.log("You return to the surface.");
    } else {
        let seed = state.world.world_map.as_ref()
            .map(|wm| wm.tile_seed(state.world.world_x, state.world.world_y))
            .unwrap_or(42)
            .wrapping_add(state.world.layer.unsigned_abs() as u64 * 1000);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (map, rooms) = Map::generate_subterranean(&mut rng, state.world.layer);
        let (px, py) = rooms.last().copied().unwrap_or((5, 5));
        state.world.map = map;
        state.world.enemies = Vec::new();
        state.world.items = Vec::new();
        state.world.npcs = Vec::new();
        state.player.x = px;
        state.player.y = py;
        state.update_fov();
        state.rebuild_spatial_index();
        state.update_lighting();
        state.log(format!("You ascend to level {}.", -state.world.layer));
    }
    true
}

// ---------------------------------------------------------------------------
// World path
// ---------------------------------------------------------------------------

pub fn calculate_world_path(state: &mut GameState, target: (usize, usize)) -> bool {
    if state.world.world_map.is_none() { return false; }
    let mut path = Vec::new();
    let mut current = (state.world.world_x, state.world.world_y);
    let mut steps = 0;
    while current != target && steps < 500 {
        let (cx, cy) = current;
        let (tx, ty) = target;
        let next = if cx < tx { (cx+1, cy) } else if cx > tx { (cx-1, cy) }
                   else if cy < ty { (cx, cy+1) } else if cy > ty { (cx, cy-1) }
                   else { break };
        path.push(next);
        current = next;
        steps += 1;
    }
    if !path.is_empty() {
        state.world.world_map_target = Some(target);
        state.world.world_map_path = path;
        true
    } else { false }
}

pub fn move_along_path(state: &mut GameState) -> Result<bool, String> {
    if state.world.world_map_path.is_empty() { return Ok(false); }
    let next_pos = state.world.world_map_path.remove(0);
    if let Some(_msg) = move_on_world_map(state, next_pos.0, next_pos.1) {
        state.world.world_map_path.clear();
        state.world.world_map_target = None;
        return Ok(true);
    }
    if let Some(target) = state.world.world_map_target
        && (state.world.world_x, state.world.world_y) == target {
            state.world.world_map_target = None;
            state.world.world_map_path.clear();
        }
    Ok(true)
}

pub fn move_on_world_map(state: &mut GameState, new_wx: usize, new_wy: usize) -> Option<String> {
    use crate::game::travel;

    let from = (state.world.world_x, state.world.world_y);
    let to = (new_wx, new_wy);
    if !travel::is_adjacent(from, to) {
        return Some("Too far to travel in one step. Move to an adjacent tile.".to_string());
    }

    if let Some(wm) = &state.world.world_map {
        let (biome, terrain, _elev, _poi, _res, _conn, level) = wm.get(new_wx, new_wy);
        let cost = travel::travel_cost(terrain, biome);
        state.turn += cost;
        state.world.total_tiles_traveled += 1;

        let last_encounter = state.world.encounter_history.get(&(new_wx, new_wy)).copied().unwrap_or(0);
        if crate::game::encounter::should_trigger_encounter(
            state.seed, new_wx, new_wy, state.world.total_tiles_traveled,
            level, last_encounter, state.turn,
            state.player.skills.get_skill_level("wayfaring"),
        ) {
            let encounter = crate::game::encounter::generate_encounter(
                state.seed, new_wx, new_wy, state.world.total_tiles_traveled,
                level, biome.as_str(),
            );
            let msg = match &encounter.encounter_type {
                crate::game::encounter::EncounterType::Hostile { threat_points } =>
                    format!("⚔ Hostile encounter! (Threat: {})", threat_points),
                crate::game::encounter::EncounterType::Neutral { description, .. } =>
                    description.clone(),
                crate::game::encounter::EncounterType::Beneficial { boon_points } =>
                    format!("✨ You discover something! (Value: {})", boon_points),
            };
            state.world.encounter_state = Some(encounter);
            state.world.encounter_history.insert((new_wx, new_wy), state.turn);
            state.world.world_x = new_wx;
            state.world.world_y = new_wy;
            travel_to_tile(state, new_wx, new_wy);
            spawn_encounter_entities(state);
            return Some(msg);
        }

        state.world.world_x = new_wx;
        state.world.world_y = new_wy;
    }
    None
}

// ---------------------------------------------------------------------------
// Dispatch helpers (called from apply_one bridge mutations)
// ---------------------------------------------------------------------------

pub fn dispatch_world_move(state: &mut GameState, new_wx: usize, new_wy: usize) {
    use crate::game::travel;

    let from = (state.world.world_x, state.world.world_y);
    let to = (new_wx, new_wy);
    if !travel::is_adjacent(from, to) {
        state.log("Too far to travel in one step. Move to an adjacent tile.");
        return;
    }
    let (biome, terrain, _elev, _poi, _res, _conn, level) = match &state.world.world_map {
        Some(wm) => wm.get(new_wx, new_wy),
        None => return,
    };
    let cost = travel::travel_cost(terrain, biome);
    let mut mutations: Vec<crate::game::mutations::Mutation> = vec![
        crate::game::mutations::Mutation::IncrementTilesTraveled,
    ];
    for _ in 0..cost { mutations.push(crate::game::mutations::Mutation::AdvanceTurn); }

    let last_encounter = state.world.encounter_history.get(&(new_wx, new_wy)).copied().unwrap_or(0);
    let encounter_triggered = crate::game::encounter::should_trigger_encounter(
        state.seed, new_wx, new_wy, state.world.total_tiles_traveled + 1,
        level, last_encounter, state.turn + cost,
        state.player.skills.get_skill_level("wayfaring"),
    );
    mutations.push(crate::game::mutations::Mutation::SetWorldPosition { wx: new_wx, wy: new_wy });
    state.apply_mutations(mutations);

    if encounter_triggered {
        let encounter = crate::game::encounter::generate_encounter(
            state.seed, new_wx, new_wy, state.world.total_tiles_traveled,
            level, biome.as_str(),
        );
        let msg = match &encounter.encounter_type {
            crate::game::encounter::EncounterType::Hostile { threat_points } =>
                format!("⚔ Hostile encounter! (Threat: {})", threat_points),
            crate::game::encounter::EncounterType::Neutral { description, .. } =>
                description.clone(),
            crate::game::encounter::EncounterType::Beneficial { boon_points } =>
                format!("✨ You discover something! (Value: {})", boon_points),
        };
        state.world.encounter_state = Some(encounter);
        state.world.encounter_history.insert((new_wx, new_wy), state.turn);
        travel_to_tile(state, new_wx, new_wy);
        spawn_encounter_entities(state);
        state.pending_ui.dialogue = Some(("Encounter!".to_string(), msg));
    }
}

pub fn dispatch_world_move_safe(state: &mut GameState, new_wx: usize, new_wy: usize) {
    use crate::game::travel;

    let from = (state.world.world_x, state.world.world_y);
    let to = (new_wx, new_wy);
    let is_same = from == to;
    if !is_same && !travel::is_adjacent(from, to) {
        state.log("Too far to travel in one step. Move to an adjacent tile.");
        return;
    }
    if !is_same
        && let Some(wm) = &state.world.world_map {
            let (biome, terrain, ..) = wm.get(new_wx, new_wy);
            let cost = travel::travel_cost(terrain, biome);
            let mut mutations = vec![crate::game::mutations::Mutation::IncrementTilesTraveled];
            for _ in 0..cost { mutations.push(crate::game::mutations::Mutation::AdvanceTurn); }
            state.apply_mutations(mutations);
        }
    travel_to_tile(state, new_wx, new_wy);
}

pub fn dispatch_follow_world_path(state: &mut GameState) {
    if state.world.world_map_path.is_empty() { return; }
    let next = state.world.world_map_path.remove(0);
    dispatch_world_move(state, next.0, next.1);
    if let Some(target) = state.world.world_map_target
        && (state.world.world_x, state.world.world_y) == target {
            state.world.world_map_target = None;
            state.world.world_map_path.clear();
        }
}

pub fn dispatch_calculate_world_path(state: &mut GameState, target: (usize, usize)) {
    if calculate_world_path(state, target) {
        state.apply_mutations(vec![crate::game::mutations::Mutation::SetWorldPath {
            path: state.world.world_map_path.clone(),
            target: Some(target),
        }]);
    }
}

/// Generate crystal formations for appropriate biomes. Called after tile generation.
pub fn generate_crystal_formations(
    state: &mut crate::game::state::GameState,
    biome: &crate::game::world_map::Biome,
    rooms: &[(i32, i32)],
    rng: &mut rand_chacha::ChaCha8Rng,
) {
    use crate::game::crystal_resonance::CrystalFrequency;
    use crate::game::state::MsgType;
    use rand::Rng;

    let formation_chance = match biome {
        crate::game::world_map::Biome::Ruins => 0.6,
        crate::game::world_map::Biome::Oasis => 0.4,
        crate::game::world_map::Biome::Saltflat => 0.3,
        crate::game::world_map::Biome::Scrubland => 0.2,
        crate::game::world_map::Biome::Desert => 0.1,
    };
    if !rng.gen_bool(formation_chance) { return; }
    let formation_count = match biome {
        crate::game::world_map::Biome::Ruins => rng.gen_range(2..=4),
        crate::game::world_map::Biome::Oasis => rng.gen_range(1..=3),
        _ => rng.gen_range(1..=2),
    };
    let frequencies = CrystalFrequency::all();
    for _ in 0..formation_count {
        if let Some(&(rx, ry)) = rooms.get(rng.gen_range(0..rooms.len())) {
            let x = rx + rng.gen_range(-2..=2);
            let y = ry + rng.gen_range(-2..=2);
            if (x - state.player.x).abs() < 5 && (y - state.player.y).abs() < 5 { continue; }
            let frequency = frequencies[rng.gen_range(0..frequencies.len())];
            state.player.crystal_system.add_crystal(x, y, frequency);
            state.log_typed(
                format!("A {} crystal formation glimmers nearby.", frequency.name().to_lowercase()),
                MsgType::Loot,
            );
        }
    }
}
