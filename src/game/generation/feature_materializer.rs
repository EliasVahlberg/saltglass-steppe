use crate::game::generation::feature_registry::get_feature_def;
use crate::game::generation::spawn::{get_biome_spawn_table, weighted_pick_by_level_and_tier};
use crate::game::light_defs::{get_spawn_rule, pick_light_type};
use crate::game::world_map::{Biome, Terrain, POI};
use crate::game::{Enemy, GameState, Interactable};
use serde_json::Value;

/// Materialize Map.features into runtime entities based on a data-driven registry.
pub fn materialize_features(
    state: &mut GameState,
    biome: Biome,
    _terrain: Terrain,
    _poi: POI,
    level: u32,
) {
    let features = state.map.features.clone();
    let table = get_biome_spawn_table(&biome);
    let mut occupied: HashSet<(i32, i32)> = HashSet::new();
    for e in &state.enemies {
        occupied.insert((e.x, e.y));
    }
    for n in &state.npcs {
        occupied.insert((n.x, n.y));
    }
    for i in &state.items {
        occupied.insert((i.x, i.y));
    }
    for i in &state.interactables {
        occupied.insert((i.x, i.y));
    }

    for feature in features {
        let Some(def) = get_feature_def(&feature.feature_id) else {
            continue;
        };

        // Require walkable tiles and avoid overlaps
        let idx = match state.map.pos_to_idx(feature.x, feature.y) {
            Some(i) => i,
            None => continue,
        };
        if !state.map.tiles[idx].walkable() || occupied.contains(&(feature.x, feature.y)) {
            continue;
        }

        match def.handler.as_str() {
            "light" => place_light(state, &feature, &def.params),
            "loot" => place_loot(state, &feature, &def.params),
            "enemy" => place_enemy(state, &feature, &def.params, level, &table.enemies),
            "npc" => place_npc(state, &feature, &def.params),
            "interactable" => place_interactable(state, &feature, &def.params),
            "story" => emit_story_hook(state, &feature, &def.params),
            _ => {}
        }

        occupied.insert((feature.x, feature.y));
    }
}

fn place_light(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
) {
    let table_id = params
        .get("table")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let rule = get_spawn_rule(table_id);
    let id = {
        let rng = &mut state.rng;
        pick_light_type(rule, rng)
    };
    if let Some(id) = id {
        state.map.lights.push(crate::game::map::MapLight {
            x: feature.x,
            y: feature.y,
            id,
        });
    }
}

fn place_loot(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    _params: &Value,
) {
    // Simple: drop a chest item using existing loot generator by spawn table.
    let id = "generic_chest";
    state
        .items
        .push(crate::game::item::Item::new(feature.x, feature.y, id));
    // Future: hook loot tables here.
}

fn place_enemy(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
    level: u32,
    spawns: &[crate::game::generation::spawn::WeightedSpawn],
) {
    let table_id = params.get("table").and_then(|v| v.as_str()).unwrap_or("default");
    let use_boss = table_id == "boss";
    let picked = {
        let rng = &mut state.rng;
        weighted_pick_by_level_and_tier(spawns, level, rng, use_boss)
    };
    if let Some(id) = picked {
        state.enemies.push(Enemy::new(feature.x, feature.y, id));
    }
}

fn place_npc(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
) {
    let table_id = params.get("table").and_then(|v| v.as_str()).unwrap_or("default");
    // For now, reuse interactables as stand-ins for NPC markers; real NPC table can be added later.
    let npc_id = match table_id {
        "merchant" => "merchant_stall",
        _ => "settler",
    };
    state
        .npcs
        .push(crate::game::npc::Npc::new(feature.x, feature.y, npc_id));
}

fn place_interactable(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
) {
    if let Some(id) = params.get("id").and_then(|v| v.as_str()) {
        state.interactables.push(Interactable::new(id.to_string(), feature.x, feature.y));
    }
}

fn emit_story_hook(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
) {
    let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("environmental");
    state.emit(crate::game::event::GameEvent::StoryHook {
        kind: kind.to_string(),
        x: feature.x,
        y: feature.y,
        context: feature.metadata.clone(),
    });
}
use std::collections::HashSet;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::{Map, MapFeature};

    #[test]
    fn story_hook_materializes() {
        let mut state = GameState::new(12345);
        state.map = Map::new(10, 10);
        state.map.features.push(MapFeature {
            x: 1,
            y: 1,
            feature_id: "story_hook".to_string(),
            source: Some("test".to_string()),
            metadata: std::collections::HashMap::new(),
        });

        materialize_features(&mut state, Biome::Desert, Terrain::Flat, POI::None, 1);

        let hook = state.event_queue.iter().any(|e| matches!(e, crate::game::event::GameEvent::StoryHook { .. }));
        assert!(hook, "story hook event should be enqueued");
    }
}
