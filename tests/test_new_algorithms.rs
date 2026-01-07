use saltglass_steppe::game::generation::structures::algorithms::*;
use saltglass_steppe::game::generation::structures::Rectangle;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

#[test]
fn test_all_new_algorithms() {
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let bounds = Rectangle::new(0, 0, 50, 40);
    
    println!("🧪 Testing all new algorithms...");
    
    // Test Drunkard's Walk
    println!("Testing Drunkard's Walk...");
    let dw_params = DrunkardWalkParams::default();
    let dw_generator = DrunkardWalkAlgorithm::new(dw_params);
    let tunnels = dw_generator.generate(bounds, &mut rng);
    println!("✅ Drunkard's Walk generated {} tunnel tiles", tunnels.len());
    assert!(!tunnels.is_empty(), "Drunkard's Walk should generate some tunnels");
    
    // Test Simple Rooms
    println!("Testing Simple Rooms...");
    let sr_params = SimpleRoomsParams::default();
    let sr_generator = SimpleRoomsAlgorithm::new(sr_params);
    let (rooms, corridors) = sr_generator.generate(bounds, &mut rng);
    println!("✅ Simple Rooms generated {} rooms and {} corridor tiles", rooms.len(), corridors.len());
    assert!(!rooms.is_empty(), "Simple Rooms should generate some rooms");
    
    // Test Cellular Automata
    println!("Testing Cellular Automata...");
    let ca_params = CellularAutomataParams::default();
    let ca_generator = CellularAutomataAlgorithm::new(ca_params);
    let walls = ca_generator.generate(bounds, &mut rng);
    println!("✅ Cellular Automata generated {} wall tiles", walls.len());
    assert!(!walls.is_empty(), "Cellular Automata should generate some walls");
    
    // Test BSP
    println!("Testing BSP...");
    let bsp_params = BSPParams::default();
    let bsp_generator = BSPAlgorithm::new(bsp_params);
    let (bsp_rooms, bsp_corridors) = bsp_generator.generate(bounds, &mut rng);
    println!("✅ BSP generated {} rooms and {} corridors", bsp_rooms.len(), bsp_corridors.len());
    assert!(!bsp_rooms.is_empty(), "BSP should generate some rooms");
    
    println!("🎉 All algorithms working correctly!");
}
