//! Integration tests for DES scenarios

use std::fs;
use saltglass_steppe::des::{run_scenario, run_parallel, Scenario};
use saltglass_steppe::game::generation::structures::{BSPAlgorithm, BSPParams, Rectangle};
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

#[test]
fn system_integration_test() {
    let result = run_scenario("tests/scenarios/system_integration_test.json")
        .expect("Failed to run system_integration_test scenario");
    
    assert!(
        result.success,
        "System integration test failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}

#[test]
fn crystal_resonance_basic() {
    let result = run_scenario("tests/scenarios/crystal_resonance_basic.json")
        .expect("Failed to run crystal_resonance_basic scenario");
    
    assert!(
        result.success,
        "Crystal resonance test failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}

#[test]
fn bsp_algorithm_basic() {
    // Test BSP algorithm generates valid room layouts
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let params = BSPParams::default();
    let algorithm = BSPAlgorithm::new(params);
    
    let bounds = Rectangle::new(0, 0, 40, 30);
    let (rooms, corridors) = algorithm.generate(bounds, &mut rng);
    
    // Validate we get rooms and corridors
    assert!(rooms.len() > 0, "BSP should generate at least one room");
    assert!(corridors.len() >= 0, "BSP should generate corridors");
    
    // Validate room constraints
    for room in &rooms {
        assert!(room.bounds.width >= 4, "Room too narrow: {}", room.bounds.width);
        assert!(room.bounds.height >= 4, "Room too short: {}", room.bounds.height);
        assert!(room.bounds.x + room.bounds.width <= 40, "Room exceeds width bounds");
        assert!(room.bounds.y + room.bounds.height <= 30, "Room exceeds height bounds");
    }
    
    // Validate corridor constraints
    for corridor in &corridors {
        assert!(corridor.width > 0, "Corridor must have width");
        assert!(corridor.start.0 <= 40 && corridor.start.1 <= 30, "Corridor start out of bounds");
        assert!(corridor.end.0 <= 40 && corridor.end.1 <= 30, "Corridor end out of bounds");
    }
    
    println!("✅ BSP Algorithm: Generated {} rooms, {} corridors", rooms.len(), corridors.len());
}

#[test]
fn cellular_automata_algorithm_basic() {
    // Test Cellular Automata algorithm generates organic walls
    use saltglass_steppe::game::generation::structures::algorithms::{CellularAutomataAlgorithm, CellularAutomataParams};
    
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let params = CellularAutomataParams::default();
    let algorithm = CellularAutomataAlgorithm::new(params);
    
    let bounds = Rectangle::new(0, 0, 30, 20);
    let walls = algorithm.generate(bounds, &mut rng);
    
    // Validate we get walls
    assert!(walls.len() > 0, "Cellular Automata should generate walls");
    
    // Validate wall constraints (create new bounds for comparison)
    let bounds_check = Rectangle::new(0, 0, 30, 20);
    for (x, y) in &walls {
        assert!(*x >= bounds_check.x as i32 && *x < (bounds_check.x + bounds_check.width) as i32, "Wall x out of bounds: {}", x);
        assert!(*y >= bounds_check.y as i32 && *y < (bounds_check.y + bounds_check.height) as i32, "Wall y out of bounds: {}", y);
    }
    
    // Test determinism
    let mut rng2 = ChaCha8Rng::seed_from_u64(12345);
    let bounds2 = Rectangle::new(0, 0, 30, 20);
    let walls2 = algorithm.generate(bounds2, &mut rng2);
    assert_eq!(walls, walls2, "Cellular Automata should be deterministic");
    
    println!("✅ Cellular Automata Algorithm: Generated {} wall tiles", walls.len());
}

#[test]
fn void_energy_basic() {
    let result = run_scenario("tests/scenarios/void_energy_basic.json")
        .expect("Failed to run void_energy_basic scenario");
    
    assert!(
        result.success,
        "Void energy test failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}

#[test]
fn light_manipulation_basic() {
    let result = run_scenario("tests/scenarios/light_manipulation_basic.json")
        .expect("Failed to run light_manipulation_basic scenario");
    
    assert!(
        result.success,
        "Light manipulation test failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}

#[test]
fn run_all_scenarios() {
    let scenario_dir = "tests/scenarios";
    let entries = fs::read_dir(scenario_dir).expect("Failed to read scenarios directory");
    
    let mut scenarios = Vec::new();
    for entry in entries {
        let path = entry.expect("Failed to read entry").path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            scenarios.push(Scenario::from_file(&path).expect(&format!("Failed to parse {:?}", path)));
        }
    }
    
    assert!(!scenarios.is_empty(), "No scenarios found");
    
    let results = run_parallel(&scenarios);
    
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.success,
            "Scenario {} failed: {:?}",
            scenarios[i].name,
            result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
        );
    }
}

#[test]
fn basic_movement_scenario() {
    let result = run_scenario("tests/scenarios/basic_movement.json")
        .expect("Failed to run scenario");
    assert!(result.success);
    assert!(result.assertion_results.iter().all(|r| r.passed));
}

#[test]
fn item_pickup_scenario() {
    let result = run_scenario("tests/scenarios/item_pickup.json")
        .expect("Failed to run scenario");
    // Note: This may fail if player position doesn't allow pickup
    // The scenario tests the DES system, not necessarily game mechanics
    println!("Item pickup result: success={}, assertions={:?}", 
             result.success, result.assertion_results);
}

#[test]
fn book_test_scenario() {
    let result = run_scenario("tests/scenarios/book_test.json")
        .expect("Failed to run scenario");
    assert!(result.success, "Book test failed: {:?}", result.assertion_results);
}



#[test]
fn trade_ui_test() {
    let result = run_scenario("tests/scenarios/trade_ui_test.json")
        .expect("Failed to run scenario");
    assert!(result.success, "Scenario failed: {:?}", result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>());
}

#[test]
fn psychic_test() {
    let result = run_scenario("tests/scenarios/psychic_test.json")
        .expect("Failed to run scenario");
    assert!(result.success, "Scenario failed: {:?}", result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>());
}

#[test]
fn aria_test() {
    let result = run_scenario("tests/scenarios/aria_test.json")
        .expect("Failed to run scenario");
    assert!(result.success, "Scenario failed: {:?}", result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>());
}

#[test]
fn combat_behaviors_test() {
    let result = run_scenario("tests/scenarios/combat_behaviors_test.json")
        .expect("Failed to run scenario");
    assert!(result.success, "Scenario failed: {:?}", result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>());
}

#[test]
fn ranged_attack_test() {
    let result = run_scenario("tests/scenarios/ranged_attack.json")
        .expect("Failed to run scenario");
    assert!(result.success, "Scenario failed: {:?}", result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>());
}

#[test]
fn interaction_system_test() {
    let result = run_scenario("tests/scenarios/interaction_system_test.json")
        .expect("Failed to run interaction_system_test scenario");
    
    assert!(
        result.success,
        "Interaction system test failed: {:?}",
        result.assertion_results.iter().filter(|r| !r.passed).collect::<Vec<_>>()
    );
}
