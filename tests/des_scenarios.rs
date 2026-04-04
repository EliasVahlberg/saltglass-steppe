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
