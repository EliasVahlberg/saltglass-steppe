use crate::game::generation::feature_registry::get_feature_def;
use crate::game::generation::spawn::{get_biome_spawn_table, weighted_pick_by_level_and_tier};
use crate::game::light_defs::{get_spawn_rule, pick_light_type};
use crate::game::world_map::{Biome, Terrain, POI};
use crate::game::{Enemy, GameState, Interactable};
use rand_chacha::ChaCha8Rng;
use serde_json::Value;

/// Materialize Map.features into runtime entities based on a data-driven registry.
pub fn materialize_features(
    state: &mut GameState,
    biome: Biome,
    _terrain: Terrain,
    _poi: POI,
    level: u32,
    rng: &mut ChaCha8Rng,
) {
    let features = state.map.features.clone();
    let table = get_biome_spawn_table(&biome);

    for feature in features {
        let Some(def) = get_feature_def(&feature.feature_id) else {
            continue;
        };

        match def.handler.as_str() {
            "light" => place_light(state, &feature, &def.params, rng),
            "loot" => place_loot(state, &feature, &def.params, rng),
            "enemy" => place_enemy(state, &feature, &def.params, level, &table.enemies, rng),
            "npc" => place_npc(state, &feature, &def.params, rng),
            "interactable" => place_interactable(state, &feature, &def.params),
            "story" => emit_story_hook(state, &feature, &def.params),
            _ => {}
        }
    }
}

fn place_light(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
    rng: &mut ChaCha8Rng,
) {
    let table_id = params
        .get("table")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let rule = get_spawn_rule(table_id);
    if let Some(id) = pick_light_type(rule, rng) {
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
    rng: &mut ChaCha8Rng,
) {
    // Simple: drop a chest item using existing loot generator by spawn table.
    let id = "generic_chest";
    state
        .items
        .push(crate::game::item::Item::new(feature.x, feature.y, id));
    // Future: hook loot tables here.
    let _ = rng; // keep deterministic usage pattern (not used yet)
}

fn place_enemy(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
    level: u32,
    spawns: &[crate::game::generation::spawn::WeightedSpawn],
    rng: &mut ChaCha8Rng,
) {
    let table_id = params.get("table").and_then(|v| v.as_str()).unwrap_or("default");
    let use_boss = table_id == "boss";
    if let Some(id) = weighted_pick_by_level_and_tier(spawns, level, rng, use_boss) {
        state.enemies.push(Enemy::new(feature.x, feature.y, id));
    }
}

fn place_npc(
    state: &mut GameState,
    feature: &crate::game::map::MapFeature,
    params: &Value,
    _rng: &mut ChaCha8Rng,
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
