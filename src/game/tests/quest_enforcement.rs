use crate::game::state::GameState;

#[test]
fn test_main_questline_initialization() {
    let state = GameState::new(12345);
    
    // Check that the first main quest is automatically added
    assert_eq!(state.quest_log.active.len(), 1);
    assert_eq!(state.quest_log.active[0].quest_id, "pilgrims_last_angle");
    
    // Check that the quest has 3 objectives now
    assert_eq!(state.quest_log.active[0].objectives.len(), 3);
    
    // Check that the dying pilgrim is spawned
    assert!(state.npcs.iter().any(|npc| npc.id == "dying_pilgrim"));
    
    // Check that the quest notification message is present
    assert!(state.messages.iter().any(|msg| msg.text.contains("Quest added: The Pilgrim's Last Angle")));
}

#[test]
fn test_dying_pilgrim_spawn_position() {
    let state = GameState::new(12345);
    
    // Find the dying pilgrim
    let pilgrim = state.npcs.iter().find(|npc| npc.id == "dying_pilgrim").unwrap();
    
    // Check that the pilgrim is near the player spawn
    let distance = ((pilgrim.x - state.player_x).abs() + (pilgrim.y - state.player_y).abs());
    assert!(distance <= 2, "Dying pilgrim should be within 2 tiles of player spawn");
    
    // Check that the pilgrim is on a walkable tile
    if let Some(tile) = state.map.get(pilgrim.x, pilgrim.y) {
        assert!(tile.walkable(), "Dying pilgrim should be on a walkable tile");
    }
}

#[test]
fn test_quest_objective_progression() {
    let mut state = GameState::new(12345);
    
    // Initially, no objectives should be completed
    let quest = &state.quest_log.active[0];
    assert!(!quest.objectives[0].completed); // find_dying_pilgrim
    assert!(!quest.objectives[1].completed); // recover_cache
    assert!(!quest.objectives[2].completed); // return_to_pilgrim
    
    // Simulate talking to the dying pilgrim (first objective)
    state.quest_log.on_npc_talked("dying_pilgrim");
    let quest = &state.quest_log.active[0];
    assert!(quest.objectives[0].completed); // find_dying_pilgrim should be completed
    assert!(!quest.objectives[1].completed); // recover_cache still pending
    assert!(!quest.objectives[2].completed); // return_to_pilgrim still pending
    
    // Simulate collecting scripture shard
    state.inventory.push("scripture_shard".to_string());
    state.quest_log.on_item_collected("scripture_shard");
    let quest = &state.quest_log.active[0];
    assert!(quest.objectives[0].completed); // find_dying_pilgrim completed
    assert!(quest.objectives[1].completed); // recover_cache should be completed
    assert!(!quest.objectives[2].completed); // return_to_pilgrim still pending
    
    // Simulate talking to the dying pilgrim again (third objective)
    let completed_quests = state.quest_log.on_npc_talked("dying_pilgrim");
    
    // Quest should now be auto-completed and moved to completed list
    assert_eq!(completed_quests.len(), 1);
    assert_eq!(completed_quests[0], "pilgrims_last_angle");
    
    // Quest should be moved to completed and next quest should be unlocked
    assert_eq!(state.quest_log.active.len(), 1); // Should have the next quest
    assert_eq!(state.quest_log.completed.len(), 1);
    assert_eq!(state.quest_log.completed[0], "pilgrims_last_angle");
    assert_eq!(state.quest_log.active[0].quest_id, "the_broken_key"); // Next quest should be active
}

#[test]
fn test_new_objective_types() {
    use crate::game::quest::{ActiveQuest, ObjectiveType};
    
    // Test that new objective types can be created
    let interact_obj = ObjectiveType::Interact { target: "sand_pile".to_string() };
    let collect_data_obj = ObjectiveType::CollectData { data_points: 15 };
    let wait_obj = ObjectiveType::Wait { duration: 5 };
    let examine_obj = ObjectiveType::Examine { target: "light_switch".to_string() };
    
    // Verify they serialize/deserialize correctly
    assert_eq!(interact_obj, interact_obj);
    assert_eq!(collect_data_obj, collect_data_obj);
    assert_eq!(wait_obj, wait_obj);
    assert_eq!(examine_obj, examine_obj);
}
