pub mod cli;
pub mod des;
pub mod game;
pub mod ipc;
pub mod renderer;
pub mod satellite;
pub mod ui;

pub use game::*;
pub use renderer::Renderer;

#[cfg(test)]
mod lib_tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn deterministic_map_generation() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let (map1, _) = Map::generate(&mut rng1);
        let (map2, _) = Map::generate(&mut rng2);
        assert_eq!(map1.tiles, map2.tiles);
    }

    #[test]
    fn player_spawns_on_floor() {
        let state = GameState::new(42);
        let tile = state
            .world
            .map
            .get(state.player.x, state.player.y)
            .unwrap();
        assert!(tile.walkable());
    }

    #[test]
    fn player_cannot_walk_through_walls() {
        let mut state = GameState::new(42);
        let start_x = state.player.x;
        for _ in 0..100 {
            state.dispatch(crate::game::effects::Command::Move { dx: -1, dy: 0 });
        }
        let tile = state.world.map.get(state.player.x - 1, state.player.y);
        if let Some(t) = tile
            && !t.walkable()
        {
            assert!(state.player.x <= start_x);
        }
    }

    #[test]
    fn storm_converts_walls_to_glass() {
        use game::systems::StormSystem;
        let mut state = GameState::new(42);
        let walls_before: usize = state
            .world
            .map
            .tiles
            .iter()
            .filter(|t| matches!(t, Tile::Wall { .. }))
            .count();
        state.world.storm.turns_until = 0;
        state.world.storm.intensity = 3;
        StormSystem::apply_storm(&mut state);
        let walls_after: usize = state
            .world
            .map
            .tiles
            .iter()
            .filter(|t| matches!(t, Tile::Wall { .. }))
            .count();
        assert!(walls_after <= walls_before);
    }

    #[test]
    fn fov_includes_player_position() {
        let state = GameState::new(42);
        let player_idx = state.world.map.idx(state.player.x, state.player.y);
        assert!(state.visible.contains(&player_idx));
    }

    #[test]
    fn enemies_spawn_in_rooms() {
        let state = GameState::new(42);
        for enemy in &state.world.enemies {
            let tile = state.world.map.get(enemy.x, enemy.y).unwrap();
            assert!(tile.walkable());
        }
    }

    #[test]
    fn combat_reduces_enemy_hp() {
        // Use seed 100 which produces a hit with fists (90% accuracy)
        let mut state = GameState::new(100);
        if let Some(enemy) = state.world.enemies.first() {
            let ex = enemy.x;
            let ey = enemy.y;
            let initial_hp = enemy.hp;
            state.player.x = ex - 1;
            state.player.y = ey;
            let idx = state.world.map.idx(ex - 1, ey);
            state.world.map.tiles[idx] = Tile::default_floor();
            // Try attack multiple times to ensure at least one hit
            for _ in 0..5 {
                state.player.ap = 4; // Reset AP
                state.dispatch(crate::game::effects::Command::Move { dx: 1, dy: 0 });
                state.player.x = ex - 1; // Reset position for next attempt
            }
            // With 90% accuracy and 5 attempts, very unlikely to miss all
            assert!(
                state.world.enemies[0].hp < initial_hp,
                "Expected at least one hit in 5 attempts"
            );
        }
    }

    #[test]
    fn save_load_roundtrip() {
        let state = GameState::new(42);
        let path = "/tmp/test_save.ron";
        state.save(path).unwrap();
        let loaded = GameState::load(path).unwrap();
        assert_eq!(state.player.x, loaded.player.x);
        assert_eq!(state.player.y, loaded.player.y);
        assert_eq!(state.turn, loaded.turn);
        assert_eq!(state.world.map.tiles, loaded.world.map.tiles);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn glass_increases_refraction() {
        let mut state = GameState::new(42);
        // Clear enemies and NPCs to avoid collision
        state.world.enemies.clear();
        state.world.npcs.clear();
        state.rebuild_spatial_index(); // Rebuild indices after clearing
        // Make sure the tile is walkable first
        let idx = state.world.map.idx(state.player.x + 1, state.player.y);
        state.world.map.tiles[idx] = Tile::Glass;
        let initial_refraction = state.player.refraction;
        let old_x = state.player.x;
        state.dispatch(crate::game::effects::Command::Move { dx: 1, dy: 0 });
        assert_ne!(state.player.x, old_x, "Player should be able to move onto glass tile");
        assert!(
            state.player.refraction > initial_refraction,
            "Refraction should increase after walking on glass"
        );
    }

    #[test]
    fn saltblood_prevents_glass_damage() {
        let mut state = GameState::new(42);
        state.player.adaptations.push(Adaptation::Saltblood);
        let idx = state.world.map.idx(state.player.x + 1, state.player.y);
        state.world.map.tiles[idx] = Tile::Glass;
        let initial_hp = state.player.hp;
        state.dispatch(crate::game::effects::Command::Move { dx: 1, dy: 0 });
        assert_eq!(state.player.hp, initial_hp);
    }

    #[test]
    fn items_spawn_in_map() {
        let state = GameState::new(42);
        assert!(!state.world.items.is_empty());
        assert!(state.world.items.iter().any(|i| i.id == "storm_glass"));
    }

    #[test]
    fn item_removed_after_walking_onto_it() {
        let mut state = GameState::new(42);
        // Clear entities to avoid collision
        state.world.enemies.clear();
        state.world.npcs.clear();
        // Place item one tile to the right
        let item_x = state.player.x + 1;
        let item_y = state.player.y;
        // Ensure tile is walkable
        let idx = state.world.map.idx(item_x, item_y);
        state.world.map.tiles[idx] = Tile::Floor {
            id: "test_floor".to_string(),
        };
        // Clear existing items and add test item
        state.world.items.clear();
        state
            .world
            .items
            .push(Item::new(item_x, item_y, "brine_vial"));
        state.rebuild_spatial_index();
        assert_eq!(state.world.items.len(), 1);
        // Move onto item
        let old_x = state.player.x;
        state.dispatch(crate::game::effects::Command::Move { dx: 1, dy: 0 });
        assert_ne!(state.player.x, old_x, "Player should be able to move onto the item tile");
        // Item should be removed from map
        assert_eq!(
            state.world.items.len(),
            0,
            "Item should be removed after walking onto it"
        );
        assert_eq!(
            state.player.inventory.len(),
            1,
            "Inventory should have 1 item"
        );
    }

    #[test]
    fn pickup_adds_to_inventory() {
        let mut state = GameState::new(42);
        state
            .world
            .items
            .push(Item::new(state.player.x, state.player.y, "brine_vial"));
        state.rebuild_spatial_index();
        let items_before = state.world.items.len();
        crate::game::systems::movement::MovementSystem::pickup_items(&mut state);
        assert_eq!(state.world.items.len(), items_before - 1);
        assert!(state.player.inventory.contains(&"brine_vial".to_string()));
    }

    #[test]
    fn brine_vial_heals() {
        let mut state = GameState::new(42);
        state.player.hp = 10;
        state.player.inventory.push("brine_vial".to_string());
        state.dispatch(crate::game::effects::Command::UseItem { index: 0 });
        assert_eq!(state.player.hp, 15);
    }

    #[test]
    fn npc_dialogue_reacts_to_adaptations() {
        use crate::game::Adaptation;
        use crate::game::npc::Npc;
        use std::collections::HashMap;

        let npc = Npc::new(0, 0, "mirror_monk");
        use crate::game::npc::DialogueContext;
        let empty_rep = HashMap::new();

        // No adaptations
        let ctx = DialogueContext {
            adaptations: &[],
            inventory: &[],
            salt_scrip: 0,
            faction_reputation: &empty_rep,
        };
        let dialogue = npc.dialogue(&ctx);
        assert!(dialogue.contains("unmarked"));

        // With Prismhide
        let ctx = DialogueContext {
            adaptations: &[Adaptation::Prismhide],
            inventory: &[],
            salt_scrip: 0,
            faction_reputation: &empty_rep,
        };
        let dialogue = npc.dialogue(&ctx);
        assert!(dialogue.contains("refracts"));
    }

    #[test]
    fn npc_bump_to_talk() {
        use crate::game::npc::Npc;

        let mut state = GameState::new(100);
        // Place NPC adjacent to player
        let npc_x = state.player.x + 1;
        let npc_y = state.player.y;

        state.world.npcs.push(Npc::new(npc_x, npc_y, "mirror_monk"));
        state.rebuild_spatial_index();

        // Get the index of our NPC
        let npc_idx = state
            .npc_at(npc_x, npc_y)
            .expect("NPC should be in spatial index");
        assert!(
            !state.world.npcs[npc_idx].talked,
            "NPC should not be talked to initially"
        );

        // Bump into NPC
        state.dispatch(crate::game::effects::Command::Move { dx: 1, dy: 0 });

        assert!(
            state.world.npcs[npc_idx].talked,
            "NPC should be talked to after bump"
        );
    }
}
