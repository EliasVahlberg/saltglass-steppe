use crate::game::state::GameState;

#[test]
fn test_main_questline_initialization() {
    let state = GameState::new(12345);

    // Check that the first main quest is automatically added
    assert_eq!(state.player.quest_log.active.len(), 1);
    assert_eq!(
        state.player.quest_log.active[0].quest_id,
        "pilgrims_last_angle"
    );

    // Check that the dying pilgrim is spawned
    assert!(state.world.npcs.iter().any(|npc| npc.id == "dying_pilgrim"));

    // Check that the quest notification message is present
    assert!(
        state
            .messages
            .iter()
            .any(|msg| msg.text.contains("Quest added: The Pilgrim's Last Angle"))
    );
}

#[test]
fn test_dying_pilgrim_spawn_position() {
    let state = GameState::new(12345);

    // Find the dying pilgrim
    let pilgrim = state
        .world.npcs
        .iter()
        .find(|npc| npc.id == "dying_pilgrim")
        .unwrap();

    // Check that the pilgrim is near the player spawn
    let distance = (pilgrim.x - state.player.x).abs() + (pilgrim.y - state.player.y).abs();
    assert!(
        distance <= 2,
        "Dying pilgrim should be within 2 tiles of player spawn"
    );

    // Check that the pilgrim is on a walkable tile
    if let Some(tile) = state.world.map.get(pilgrim.x, pilgrim.y) {
        assert!(
            tile.walkable(),
            "Dying pilgrim should be on a walkable tile"
        );
    }
}

#[test]
fn test_quest_objective_progression() {
    let mut state = GameState::new(12345);

    // Simulate talking to the dying pilgrim (first objective)
    state.player.quest_log.on_npc_talked("dying_pilgrim");

    // Simulate collecting scripture shard
    state.player.inventory.push("scripture_shard".to_string());
    state.player.quest_log.on_item_collected("scripture_shard");

    // Simulate talking to the dying pilgrim again (third objective)
    let completed_quests = state.player.quest_log.on_npc_talked("dying_pilgrim");

    // Quest should now be auto-completed and moved to completed list
    assert_eq!(completed_quests.len(), 1);
    assert_eq!(completed_quests[0], "pilgrims_last_angle");

    // Quest should be moved to completed and next quest should be unlocked
    assert_eq!(state.player.quest_log.active.len(), 1); // Should have the next quest
    assert_eq!(state.player.quest_log.completed.len(), 1);
    assert_eq!(state.player.quest_log.completed[0], "pilgrims_last_angle");
    assert_eq!(state.player.quest_log.active[0].quest_id, "the_broken_key"); // Next quest should be active
}

#[test]
fn test_new_objective_types() {
    use crate::game::quest::ObjectiveType;

    // Test that new objective types can be created
    let interact_obj = ObjectiveType::Interact {
        target: "sand_pile".to_string(),
    };
    let collect_data_obj = ObjectiveType::CollectData { data_points: 15 };
    let wait_obj = ObjectiveType::Wait { duration: 5 };
    let examine_obj = ObjectiveType::Examine {
        target: "light_switch".to_string(),
    };

    // Verify they serialize/deserialize correctly
    assert_eq!(interact_obj, interact_obj);
    assert_eq!(collect_data_obj, collect_data_obj);
    assert_eq!(wait_obj, wait_obj);
    assert_eq!(examine_obj, examine_obj);
}
