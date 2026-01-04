use saltglass_steppe::game::generation::structures::{BSPAlgorithm, BSPParams, Rectangle};
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

fn main() {
    println!("Testing BSP Algorithm...");
    
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let params = BSPParams::default();
    let algorithm = BSPAlgorithm::new(params);
    
    let bounds = Rectangle::new(0, 0, 40, 30);
    let (rooms, corridors) = algorithm.generate(bounds, &mut rng);
    
    println!("Generated {} rooms and {} corridors", rooms.len(), corridors.len());
    
    // Validate rooms
    for (i, room) in rooms.iter().enumerate() {
        println!("Room {}: {}x{} at ({}, {})", 
                 i, room.bounds.width, room.bounds.height, room.bounds.x, room.bounds.y);
        
        // Basic validation
        assert!(room.bounds.width >= 4, "Room {} too narrow", i);
        assert!(room.bounds.height >= 4, "Room {} too short", i);
        assert!(room.bounds.x + room.bounds.width <= 40, "Room {} exceeds bounds", i);
        assert!(room.bounds.y + room.bounds.height <= 30, "Room {} exceeds bounds", i);
    }
    
    // Validate corridors
    for (i, corridor) in corridors.iter().enumerate() {
        println!("Corridor {}: ({}, {}) -> ({}, {})", 
                 i, corridor.start.0, corridor.start.1, corridor.end.0, corridor.end.1);
        
        assert!(corridor.width > 0, "Corridor {} has no width", i);
    }
    
    println!("✅ BSP Algorithm test passed!");
}
