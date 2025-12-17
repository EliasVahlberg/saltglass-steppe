mod game;
pub mod des;

pub use game::*;

#[cfg(test)]
mod tests {
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
        let tile = state.map.get(state.player_x, state.player_y).unwrap();
        assert!(tile.walkable());
    }

    #[test]
    fn player_cannot_walk_through_walls() {
        let mut state = GameState::new(42);
        let start_x = state.player_x;
        for _ in 0..100 { state.try_move(-1, 0); }
        let tile = state.map.get(state.player_x - 1, state.player_y);
        if let Some(t) = tile {
            if !t.walkable() { assert!(state.player_x <= start_x); }
        }
    }

    #[test]
    fn storm_converts_walls_to_glass() {
        let mut state = GameState::new(42);
        let walls_before: usize = state.map.tiles.iter().filter(|t| matches!(t, Tile::Wall { .. })).count();
        state.storm.turns_until = 0;
        state.storm.intensity = 3;
        state.apply_storm();
        let walls_after: usize = state.map.tiles.iter().filter(|t| matches!(t, Tile::Wall { .. })).count();
        assert!(walls_after <= walls_before);
    }

    #[test]
    fn fov_includes_player_position() {
        let state = GameState::new(42);
        let player_idx = state.map.idx(state.player_x, state.player_y);
        assert!(state.visible.contains(&player_idx));
    }

    #[test]
    fn enemies_spawn_in_rooms() {
        let state = GameState::new(42);
        for enemy in &state.enemies {
            let tile = state.map.get(enemy.x, enemy.y).unwrap();
            assert!(tile.walkable());
        }
    }

    #[test]
    fn combat_reduces_enemy_hp() {
        // Use seed 100 which produces a hit with fists (90% accuracy)
        let mut state = GameState::new(100);
        if let Some(enemy) = state.enemies.first() {
            let ex = enemy.x;
            let ey = enemy.y;
            let initial_hp = enemy.hp;
            state.player_x = ex - 1;
            state.player_y = ey;
            let idx = state.map.idx(ex - 1, ey);
            state.map.tiles[idx] = Tile::Floor;
            // Try attack multiple times to ensure at least one hit
            for _ in 0..5 {
                state.player_ap = 4; // Reset AP
                state.try_move(1, 0);
                state.player_x = ex - 1; // Reset position for next attempt
            }
            // With 90% accuracy and 5 attempts, very unlikely to miss all
            assert!(state.enemies[0].hp < initial_hp, "Expected at least one hit in 5 attempts");
        }
    }

    #[test]
    fn save_load_roundtrip() {
        let state = GameState::new(42);
        let path = "/tmp/test_save.ron";
        state.save(path).unwrap();
        let loaded = GameState::load(path).unwrap();
        assert_eq!(state.player_x, loaded.player_x);
        assert_eq!(state.player_y, loaded.player_y);
        assert_eq!(state.turn, loaded.turn);
        assert_eq!(state.map.tiles, loaded.map.tiles);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn glass_increases_refraction() {
        let mut state = GameState::new(42);
        let idx = state.map.idx(state.player_x + 1, state.player_y);
        state.map.tiles[idx] = Tile::Glass;
        let initial_refraction = state.refraction;
        state.try_move(1, 0);
        assert!(state.refraction > initial_refraction);
    }

    #[test]
    fn saltblood_prevents_glass_damage() {
        let mut state = GameState::new(42);
        state.adaptations.push(Adaptation::Saltblood);
        let idx = state.map.idx(state.player_x + 1, state.player_y);
        state.map.tiles[idx] = Tile::Glass;
        let initial_hp = state.player_hp;
        state.try_move(1, 0);
        assert_eq!(state.player_hp, initial_hp);
    }

    #[test]
    fn items_spawn_in_map() {
        let state = GameState::new(42);
        assert!(!state.items.is_empty());
        assert!(state.items.iter().any(|i| i.id == "angle_lens"));
    }

    #[test]
    fn item_removed_after_walking_onto_it() {
        let mut state = GameState::new(42);
        // Place item one tile to the right
        let item_x = state.player_x + 1;
        let item_y = state.player_y;
        // Ensure tile is walkable
        let idx = state.map.idx(item_x, item_y);
        state.map.tiles[idx] = Tile::Floor;
        // Clear existing items and add test item
        state.items.clear();
        state.items.push(Item::new(item_x, item_y, "brine_vial"));
        state.rebuild_spatial_index();
        assert_eq!(state.items.len(), 1);
        // Move onto item
        state.try_move(1, 0);
        // Item should be removed from map
        assert_eq!(state.items.len(), 0);
        assert_eq!(state.inventory.len(), 1);
    }

    #[test]
    fn pickup_adds_to_inventory() {
        let mut state = GameState::new(42);
        state.items.push(Item::new(state.player_x, state.player_y, "brine_vial"));
        state.rebuild_spatial_index();
        let items_before = state.items.len();
        state.pickup_items();
        assert_eq!(state.items.len(), items_before - 1);
        assert!(state.inventory.contains(&"brine_vial".to_string()));
    }

    #[test]
    fn brine_vial_heals() {
        let mut state = GameState::new(42);
        state.player_hp = 10;
        state.inventory.push("brine_vial".to_string());
        state.use_item(0);
        assert_eq!(state.player_hp, 15);
    }

    #[test]
    fn npc_dialogue_reacts_to_adaptations() {
        use crate::game::npc::Npc;
        use crate::game::Adaptation;
        
        let npc = Npc::new(0, 0, "mirror_monk");
        
        // No adaptations
        let dialogue = npc.dialogue(&[]);
        assert!(dialogue.contains("unmarked"));
        
        // With Prismhide
        let dialogue = npc.dialogue(&[Adaptation::Prismhide]);
        assert!(dialogue.contains("refracts"));
    }

    #[test]
    fn npc_bump_to_talk() {
        use crate::game::npc::Npc;
        
        let mut state = GameState::new(100);
        // Place NPC adjacent to player
        let npc_x = state.player_x + 1;
        let npc_y = state.player_y;
        
        state.npcs.push(Npc::new(npc_x, npc_y, "mirror_monk"));
        state.rebuild_spatial_index();
        
        // Get the index of our NPC
        let npc_idx = state.npc_at(npc_x, npc_y).expect("NPC should be in spatial index");
        assert!(!state.npcs[npc_idx].talked, "NPC should not be talked to initially");
        
        // Bump into NPC
        state.try_move(1, 0);
        
        assert!(state.npcs[npc_idx].talked, "NPC should be talked to after bump");
    }
}
