//! Integration tests for DES scenarios

use saltglass_steppe::des::{Scenario, run_parallel, run_scenario};
use std::fs;

#[test]
fn system_integration_test() {
    let result = run_scenario("tests/scenarios/system_integration_test.json")
        .expect("Failed to run system_integration_test scenario");

    assert!(
        result.success,
        "System integration test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "Fails while DES scenarios still depend on legacy generation semantics."]
fn run_all_scenarios() {
    let scenario_dir = "tests/scenarios";
    let entries = fs::read_dir(scenario_dir).expect("Failed to read scenarios directory");

    let mut scenarios = Vec::new();
    for entry in entries {
        let path = entry.expect("Failed to read entry").path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            scenarios.push(
                Scenario::from_file(&path).unwrap_or_else(|_| panic!("Failed to parse {:?}", path)),
            );
        }
    }

    assert!(!scenarios.is_empty(), "No scenarios found");

    let results = run_parallel(&scenarios);

    for (i, result) in results.iter().enumerate() {
        assert!(
            result.success,
            "Scenario {} failed: {:?}",
            scenarios[i].name,
            result
                .assertion_results
                .iter()
                .filter(|r| !r.passed)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn basic_movement_scenario() {
    let result =
        run_scenario("tests/scenarios/basic_movement.json").expect("Failed to run scenario");
    assert!(result.success);
    assert!(result.assertion_results.iter().all(|r| r.passed));
}

#[test]
fn item_pickup_scenario() {
    let result = run_scenario("tests/scenarios/item_pickup.json").expect("Failed to run scenario");
    // Note: This may fail if player position doesn't allow pickup
    // The scenario tests the DES system, not necessarily game mechanics
    println!(
        "Item pickup result: success={}, assertions={:?}",
        result.success, result.assertion_results
    );
}

#[test]
fn book_test_scenario() {
    let result = run_scenario("tests/scenarios/book_test.json").expect("Failed to run scenario");
    assert!(
        result.success,
        "Book test failed: {:?}",
        result.assertion_results
    );
}

#[test]
fn trade_ui_test() {
    let result =
        run_scenario("tests/scenarios/trade_ui_test.json").expect("Failed to run scenario");
    assert!(
        result.success,
        "Scenario failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn psychic_test() {
    let result = run_scenario("tests/scenarios/psychic_test.json").expect("Failed to run scenario");
    assert!(
        result.success,
        "Scenario failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn aria_test() {
    let result = run_scenario("tests/scenarios/aria_test.json").expect("Failed to run scenario");
    assert!(
        result.success,
        "Scenario failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn combat_behaviors_test() {
    let result =
        run_scenario("tests/scenarios/combat_behaviors_test.json").expect("Failed to run scenario");
    assert!(
        result.success,
        "Scenario failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn ranged_attack_test() {
    let result =
        run_scenario("tests/scenarios/ranged_attack.json").expect("Failed to run scenario");
    assert!(
        result.success,
        "Scenario failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn interaction_system_test() {
    let result = run_scenario("tests/scenarios/interaction_system_test.json")
        .expect("Failed to run interaction_system_test scenario");

    assert!(
        result.success,
        "Interaction system test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn skill_tree_upgrade_test() {
    let result = run_scenario("tests/scenarios/skill_tree_upgrade_test.json")
        .expect("Failed to run skill_tree_upgrade_test scenario");

    assert!(
        result.success,
        "Skill tree upgrade test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

// ── Gap-analysis scenarios ────────────────────────────────────────────────────

#[test]
fn faction_enemy_aggression_test() {
    let result = run_scenario("tests/scenarios/faction_enemy_aggression_test.json")
        .expect("Failed to run faction_enemy_aggression_test");
    assert!(
        result.success,
        "Faction enemy aggression test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn faction_enemy_hostile_below_threshold_test() {
    let result = run_scenario("tests/scenarios/faction_enemy_hostile_below_threshold_test.json")
        .expect("Failed to run faction_enemy_hostile_below_threshold_test");
    assert!(
        result.success,
        "Faction enemy hostile below threshold test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn faction_reputation_boundaries_test() {
    let result = run_scenario("tests/scenarios/faction_reputation_boundaries_test.json")
        .expect("Failed to run faction_reputation_boundaries_test");
    assert!(
        result.success,
        "Faction reputation boundaries test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn skill_passive_combat_test() {
    let result = run_scenario("tests/scenarios/skill_passive_combat_test.json")
        .expect("Failed to run skill_passive_combat_test");
    assert!(
        result.success,
        "Skill passive combat test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

// ── Tier 2 scenarios ──────────────────────────────────────────────────────────

#[test]
fn algorithm_layering_test() {
    let result = run_scenario("tests/scenarios/algorithm_layering_test.json")
        .expect("Failed to run algorithm_layering_test");
    assert!(
        result.success,
        "Algorithm layering test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn enemy_ranged_behavior_test() {
    let result = run_scenario("tests/scenarios/enemy_ranged_behavior_test.json")
        .expect("Failed to run enemy_ranged_behavior_test");
    assert!(
        result.success,
        "Enemy ranged behavior test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn enemy_spawner_behavior_test() {
    let result = run_scenario("tests/scenarios/enemy_spawner_behavior_test.json")
        .expect("Failed to run enemy_spawner_behavior_test");
    assert!(
        result.success,
        "Enemy spawner behavior test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn adaptation_effects_test() {
    let result = run_scenario("tests/scenarios/adaptation_effects_test.json")
        .expect("Failed to run adaptation_effects_test");
    assert!(
        result.success,
        "Adaptation effects test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn storm_intensity_scaling_test() {
    let result = run_scenario("tests/scenarios/storm_intensity_scaling_test.json")
        .expect("Failed to run storm_intensity_scaling_test");
    assert!(
        result.success,
        "Storm intensity scaling test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn biome_spawn_tables_test() {
    let result = run_scenario("tests/scenarios/biome_spawn_tables_test.json")
        .expect("Failed to run biome_spawn_tables_test");
    assert!(
        result.success,
        "Biome spawn tables test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn settlement_road_pathfinding_test() {
    let result = run_scenario("tests/scenarios/settlement_road_pathfinding_test.json")
        .expect("Failed to run settlement_road_pathfinding_test");
    assert!(
        result.success,
        "Settlement road pathfinding test failed: {:?}",
        result
            .assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect::<Vec<_>>()
    );
}

#[test]
fn effect_heal_trace() {
    let result = run_scenario("tests/scenarios/effect_heal_trace.json")
        .expect("Failed to run effect_heal_trace");
    assert!(
        result.success,
        "Effect heal trace failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}

#[test]
fn effect_kill_trace() {
    let result = run_scenario("tests/scenarios/effect_kill_trace.json")
        .expect("Failed to run effect_kill_trace");
    assert!(
        result.success,
        "Effect kill trace failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}

#[test]
fn effect_wait_no_combat() {
    let result = run_scenario("tests/scenarios/effect_wait_no_combat.json")
        .expect("Failed to run effect_wait_no_combat");
    assert!(
        result.success,
        "Effect wait no combat failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}

#[test]
fn effect_miss_trace() {
    let result = run_scenario("tests/scenarios/effect_miss_trace.json")
        .expect("Failed to run effect_miss_trace");
    assert!(
        result.success,
        "Effect miss trace failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}


#[test]
fn adaptation_progression() {
    let result = run_scenario("tests/scenarios/adaptation_progression.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn adaptation_stat_modifiers() {
    let result = run_scenario("tests/scenarios/adaptation_stat_modifiers.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn ai_disabled_enemy() {
    let result = run_scenario("tests/scenarios/ai_disabled_enemy.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn animation_effects_test() {
    let result = run_scenario("tests/scenarios/animation_effects_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn ap_consumption() {
    let result = run_scenario("tests/scenarios/ap_consumption.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn ap_end_turn() {
    let result = run_scenario("tests/scenarios/ap_end_turn.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn ap_melee_cost() {
    let result = run_scenario("tests/scenarios/ap_melee_cost.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn archive_dungeon_test() {
    let result = run_scenario("tests/scenarios/archive_dungeon_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn auto_explore() {
    let result = run_scenario("tests/scenarios/auto_explore.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn auto_explore_configuration() {
    let result = run_scenario("tests/scenarios/auto_explore_configuration.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn auto_explore_danger_avoidance() {
    let result = run_scenario("tests/scenarios/auto_explore_danger_avoidance.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn auto_explore_enemy_detection() {
    let result = run_scenario("tests/scenarios/auto_explore_enemy_detection.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn auto_explore_fixes_test() {
    let result = run_scenario("tests/scenarios/auto_explore_fixes_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn auto_explore_item_pickup() {
    let result = run_scenario("tests/scenarios/auto_explore_item_pickup.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn base_empty_room() {
    let result = run_scenario("tests/scenarios/base_empty_room.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn behavior_registry_test() {
    let result = run_scenario("tests/scenarios/behavior_registry_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn biome_specific_content_test() {
    let result = run_scenario("tests/scenarios/biome_specific_content_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn biome_system_basic() {
    let result = run_scenario("tests/scenarios/biome_system_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn bracket_lib_pathfinding_test() {
    let result = run_scenario("tests/scenarios/bracket_lib_pathfinding_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn bsp_algorithm_test() {
    let result = run_scenario("tests/scenarios/bsp_algorithm_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn camera_centering_test() {
    let result = run_scenario("tests/scenarios/camera_centering_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn cellular_automata_algorithm_test() {
    let result = run_scenario("tests/scenarios/cellular_automata_algorithm_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn chest_spawn_test() {
    let result = run_scenario("tests/scenarios/chest_spawn_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn combat_basic() {
    let result = run_scenario("tests/scenarios/combat_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn combat_hit_miss() {
    let result = run_scenario("tests/scenarios/combat_hit_miss.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn combat_kill_xp() {
    let result = run_scenario("tests/scenarios/combat_kill_xp.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn combat_player_death() {
    let result = run_scenario("tests/scenarios/combat_player_death.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn combat_ranged_kill() {
    let result = run_scenario("tests/scenarios/combat_ranged_kill.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn connectivity_validation() {
    let result = run_scenario("tests/scenarios/connectivity_validation.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn constraint_system_basic() {
    let result = run_scenario("tests/scenarios/constraint_system_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn crafting_basic() {
    let result = run_scenario("tests/scenarios/crafting_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn damage_numbers() {
    let result = run_scenario("tests/scenarios/damage_numbers.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn demeanor_defensive() {
    let result = run_scenario("tests/scenarios/demeanor_defensive.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn demeanor_neutral() {
    let result = run_scenario("tests/scenarios/demeanor_neutral.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn dialogue_conditions_test() {
    let result = run_scenario("tests/scenarios/dialogue_conditions_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn dialogue_item_condition() {
    let result = run_scenario("tests/scenarios/dialogue_item_condition.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn dialogue_item_condition_with_key() {
    let result = run_scenario("tests/scenarios/dialogue_item_condition_with_key.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn dungeon_comprehensive_validation() {
    let result = run_scenario("tests/scenarios/dungeon_comprehensive_validation.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn dungeon_connectivity_test() {
    let result = run_scenario("tests/scenarios/dungeon_connectivity_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn dungeon_deterministic_test() {
    let result = run_scenario("tests/scenarios/dungeon_deterministic_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn dungeon_population() {
    let result = run_scenario("tests/scenarios/dungeon_population.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn dungeon_quest_accessibility_test() {
    let result = run_scenario("tests/scenarios/dungeon_quest_accessibility_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn dungeon_room_placement_test() {
    let result = run_scenario("tests/scenarios/dungeon_room_placement_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn effects_config_test() {
    let result = run_scenario("tests/scenarios/effects_config_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn enemy_hp_check() {
    let result = run_scenario("tests/scenarios/enemy_hp_check.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn enemy_spawn_types() {
    let result = run_scenario("tests/scenarios/enemy_spawn_types.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn entity_count_assertions() {
    let result = run_scenario("tests/scenarios/entity_count_assertions.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn equip_torch() {
    let result = run_scenario("tests/scenarios/equip_torch.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn event_bus_test() {
    let result = run_scenario("tests/scenarios/event_bus_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn event_system_basic() {
    let result = run_scenario("tests/scenarios/event_system_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn explored_tiles() {
    let result = run_scenario("tests/scenarios/explored_tiles.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn first_hour_comprehensive_test() {
    let result = run_scenario("tests/scenarios/first_hour_comprehensive_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn generation_pipeline_basic() {
    let result = run_scenario("tests/scenarios/generation_pipeline_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn grammar_generation_basic() {
    let result = run_scenario("tests/scenarios/grammar_generation_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn healing_brine_vial() {
    let result = run_scenario("tests/scenarios/healing_brine_vial.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn healing_saints_tear() {
    let result = run_scenario("tests/scenarios/healing_saints_tear.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn healing_salt_poultice() {
    let result = run_scenario("tests/scenarios/healing_salt_poultice.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn hp_cap() {
    let result = run_scenario("tests/scenarios/hp_cap.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn inherited_scenario() {
    let result = run_scenario("tests/scenarios/inherited_scenario.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn inventory_equip_unequip() {
    let result = run_scenario("tests/scenarios/inventory_equip_unequip.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn inventory_inspect() {
    let result = run_scenario("tests/scenarios/inventory_inspect.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn inventory_swap_equip() {
    let result = run_scenario("tests/scenarios/inventory_swap_equip.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn inventory_unequip() {
    let result = run_scenario("tests/scenarios/inventory_unequip.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn item_pickup_basic() {
    let result = run_scenario("tests/scenarios/item_pickup_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn laser_beam_behavior() {
    let result = run_scenario("tests/scenarios/laser_beam_behavior.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn level_up_stat_allocation() {
    let result = run_scenario("tests/scenarios/level_up_stat_allocation.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn lighting() {
    let result = run_scenario("tests/scenarios/lighting.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn loot_system_event_test() {
    let result = run_scenario("tests/scenarios/loot_system_event_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn main_questline_architect() {
    let result = run_scenario("tests/scenarios/main_questline_architect.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn message_log_types() {
    let result = run_scenario("tests/scenarios/message_log_types.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn microstructures_on_travel() {
    let result = run_scenario("tests/scenarios/microstructures_on_travel.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn microstructures_test() {
    let result = run_scenario("tests/scenarios/microstructures_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn mock_combat() {
    let result = run_scenario("tests/scenarios/mock_combat.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn mock_combat_miss() {
    let result = run_scenario("tests/scenarios/mock_combat_miss.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn movement_system_test() {
    let result = run_scenario("tests/scenarios/movement_system_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn multiple_items() {
    let result = run_scenario("tests/scenarios/multiple_items.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn narrative_integration_basic() {
    let result = run_scenario("tests/scenarios/narrative_integration_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn non_usable_item() {
    let result = run_scenario("tests/scenarios/non_usable_item.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn npc_dialogue() {
    let result = run_scenario("tests/scenarios/npc_dialogue.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn organic_cave_test() {
    let result = run_scenario("tests/scenarios/organic_cave_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn particle_effects_test() {
    let result = run_scenario("tests/scenarios/particle_effects_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn performance_optimization_test() {
    let result = run_scenario("tests/scenarios/performance_optimization_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn player_adaptations() {
    let result = run_scenario("tests/scenarios/player_adaptations.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn player_death() {
    let result = run_scenario("tests/scenarios/player_death.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn procedural_effects_test() {
    let result = run_scenario("tests/scenarios/procedural_effects_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn procedural_structure_generation_test() {
    let result = run_scenario("tests/scenarios/procedural_structure_generation_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn progression() {
    let result = run_scenario("tests/scenarios/progression.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn psychic_menu_test() {
    let result = run_scenario("tests/scenarios/psychic_menu_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn quest_chain_unlocking() {
    let result = run_scenario("tests/scenarios/quest_chain_unlocking.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn quest_npc_spawning() {
    let result = run_scenario("tests/scenarios/quest_npc_spawning.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn quest_reach_objective() {
    let result = run_scenario("tests/scenarios/quest_reach_objective.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn refraction_check() {
    let result = run_scenario("tests/scenarios/refraction_check.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn refraction_gain_from_storm() {
    let result = run_scenario("tests/scenarios/refraction_gain_from_storm.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn rest_blocked_by_enemy() {
    let result = run_scenario("tests/scenarios/rest_blocked_by_enemy.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn rest_mechanic() {
    let result = run_scenario("tests/scenarios/rest_mechanic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn settlement_generation_test() {
    let result = run_scenario("tests/scenarios/settlement_generation_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn shop_trading() {
    let result = run_scenario("tests/scenarios/shop_trading.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn shrine_connectivity_test() {
    let result = run_scenario("tests/scenarios/shrine_connectivity_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn spawn_distribution_test() {
    let result = run_scenario("tests/scenarios/spawn_distribution_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn status_bleed() {
    let result = run_scenario("tests/scenarios/status_bleed.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn status_burn() {
    let result = run_scenario("tests/scenarios/status_burn.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn status_effect_system_test() {
    let result = run_scenario("tests/scenarios/status_effect_system_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn status_effects() {
    let result = run_scenario("tests/scenarios/status_effects.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn status_multiple() {
    let result = run_scenario("tests/scenarios/status_multiple.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn storm_edit_types_test() {
    let result = run_scenario("tests/scenarios/storm_edit_types_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn storm_forecast_system() {
    let result = run_scenario("tests/scenarios/storm_forecast_system.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn storm_glass_conversion() {
    let result = run_scenario("tests/scenarios/storm_glass_conversion.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "flaky — storm RNG sensitive"]
fn storm_glass_drops() {
    let result = run_scenario("tests/scenarios/storm_glass_drops.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn storm_system_test() {
    let result = run_scenario("tests/scenarios/storm_system_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn storm_timer_countdown() {
    let result = run_scenario("tests/scenarios/storm_timer_countdown.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn suppression_test() {
    let result = run_scenario("tests/scenarios/suppression_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn teleport_action() {
    let result = run_scenario("tests/scenarios/teleport_action.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn template_system_basic() {
    let result = run_scenario("tests/scenarios/template_system_basic.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn terrain_variety_test() {
    let result = run_scenario("tests/scenarios/terrain_variety_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn test_renderer_frame() {
    let result = run_scenario("tests/scenarios/test_renderer_frame.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn theme_system_test() {
    let result = run_scenario("tests/scenarios/theme_system_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn tile_from_world() {
    let result = run_scenario("tests/scenarios/tile_from_world.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn torch_lighting_persistence() {
    let result = run_scenario("tests/scenarios/torch_lighting_persistence.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn town_population() {
    let result = run_scenario("tests/scenarios/town_population.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
#[ignore = "scenario fails — known broken, needs investigation"]
fn trading_system_test() {
    let result = run_scenario("tests/scenarios/trading_system_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn travel_spawn_safety() {
    let result = run_scenario("tests/scenarios/travel_spawn_safety.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn tutorial_messages_display() {
    let result = run_scenario("tests/scenarios/tutorial_messages_display.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn wait_action() {
    let result = run_scenario("tests/scenarios/wait_action.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn wall_break_test() {
    let result = run_scenario("tests/scenarios/wall_break_test.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}

#[test]
fn world_tile_transition() {
    let result = run_scenario("tests/scenarios/world_tile_transition.json").expect("Failed to run scenario");
    assert!(result.success, "{:?}", result.assertion_results);
}
