use crate::game::generation::constraints::{ConstraintSystem, ConstraintContext, EntityPlacement, ResourcePlacement, ObjectivePlacement};
use crate::game::map::{Map, Tile};
use crate::game::world_map::Biome;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use std::collections::HashMap;

/// Test different map types with gameplay constraints
pub fn test_constraint_maps() {
    println!("=== Testing Gameplay Constraints on Different Map Types ===\n");
    
    // Test 1: Dungeon Map (high tactical requirements)
    test_dungeon_map();
    
    // Test 2: Overworld Map (safe zone focus)
    test_overworld_map();
    
    // Test 3: Town Map (accessibility focus)
    test_town_map();
    
    // Test 4: Combat Arena (tactical focus)
    test_combat_arena();
}

fn test_dungeon_map() {
    println!("--- Testing Dungeon Map ---");
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    
    // Generate a dungeon-like map with rooms and corridors
    let map = create_dungeon_map();
    let context = create_dungeon_context(&map);
    
    let results = ConstraintSystem::validate_constraints(&context, &mut rng);
    print_constraint_results("Dungeon", &results);
    
    // Try to fix issues if any critical constraints failed
    if !ConstraintSystem::are_critical_constraints_satisfied(&results) {
        println!("  Attempting to fix dungeon constraints...");
        let fixed_map = fix_dungeon_constraints(map, &results);
        let fixed_context = create_dungeon_context(&fixed_map);
        let fixed_results = ConstraintSystem::validate_constraints(&fixed_context, &mut rng);
        print_constraint_results("Fixed Dungeon", &fixed_results);
    }
    println!();
}

fn test_overworld_map() {
    println!("--- Testing Overworld Map ---");
    let mut rng = ChaCha8Rng::seed_from_u64(23456);
    
    let map = create_overworld_map();
    let context = create_overworld_context(&map);
    
    let results = ConstraintSystem::validate_constraints(&context, &mut rng);
    print_constraint_results("Overworld", &results);
    
    if !ConstraintSystem::are_critical_constraints_satisfied(&results) {
        println!("  Attempting to fix overworld constraints...");
        let fixed_map = fix_overworld_constraints(map, &results);
        let fixed_context = create_overworld_context(&fixed_map);
        let fixed_results = ConstraintSystem::validate_constraints(&fixed_context, &mut rng);
        print_constraint_results("Fixed Overworld", &fixed_results);
    }
    println!();
}

fn test_town_map() {
    println!("--- Testing Town Map ---");
    let mut rng = ChaCha8Rng::seed_from_u64(34567);
    
    let map = create_town_map();
    let context = create_town_context(&map);
    
    let results = ConstraintSystem::validate_constraints(&context, &mut rng);
    print_constraint_results("Town", &results);
    
    if !ConstraintSystem::are_critical_constraints_satisfied(&results) {
        println!("  Attempting to fix town constraints...");
        let fixed_map = fix_town_constraints(map, &results);
        let fixed_context = create_town_context(&fixed_map);
        let fixed_results = ConstraintSystem::validate_constraints(&fixed_context, &mut rng);
        print_constraint_results("Fixed Town", &fixed_results);
    }
    println!();
}

fn test_combat_arena() {
    println!("--- Testing Combat Arena ---");
    let mut rng = ChaCha8Rng::seed_from_u64(45678);
    
    let map = create_combat_arena();
    let context = create_arena_context(&map);
    
    let results = ConstraintSystem::validate_constraints(&context, &mut rng);
    print_constraint_results("Arena", &results);
    
    if !ConstraintSystem::are_critical_constraints_satisfied(&results) {
        println!("  Attempting to fix arena constraints...");
        let fixed_map = fix_arena_constraints(map, &results);
        let fixed_context = create_arena_context(&fixed_map);
        let fixed_results = ConstraintSystem::validate_constraints(&fixed_context, &mut rng);
        print_constraint_results("Fixed Arena", &fixed_results);
    }
    println!();
}

fn create_dungeon_map() -> Map {
    let mut map = Map::new(40, 30);
    
    // Create rooms connected by narrow corridors (good for chokepoints)
    let rooms = vec![
        (5, 5, 8, 6),   // (x, y, w, h)
        (20, 5, 10, 8),
        (5, 18, 12, 8),
        (25, 20, 10, 6),
    ];
    
    // Fill with walls first
    for tile in map.tiles.iter_mut() {
        *tile = Tile::Wall { id: "sandstone".to_string(), hp: 100 };
    }
    
    // Carve out rooms
    for (rx, ry, rw, rh) in rooms {
        for y in ry..(ry + rh) {
            for x in rx..(rx + rw) {
                let idx = (y * map.width + x) as usize;
                map.tiles[idx] = Tile::Floor { id: "dry_soil".to_string() };
            }
        }
    }
    
    // Create narrow corridors (1-2 tiles wide)
    create_corridor(&mut map, 13, 8, 20, 8);  // Horizontal
    create_corridor(&mut map, 11, 8, 11, 18); // Vertical
    create_corridor(&mut map, 11, 22, 25, 22); // Horizontal
    
    map
}

fn create_overworld_map() -> Map {
    let mut map = Map::new(50, 40);
    
    // Create open terrain with some obstacles
    for tile in map.tiles.iter_mut() {
        *tile = Tile::Floor { id: "dry_soil".to_string() };
    }
    
    // Add some wall clusters (hills, rocks)
    let obstacles = vec![
        (10, 10, 3, 3),
        (25, 15, 4, 2),
        (35, 25, 2, 4),
        (15, 30, 5, 3),
    ];
    
    for (ox, oy, ow, oh) in obstacles {
        for y in oy..(oy + oh) {
            for x in ox..(ox + ow) {
                let idx = (y * map.width + x) as usize;
                map.tiles[idx] = Tile::Wall { id: "sandstone".to_string(), hp: 100 };
            }
        }
    }
    
    map
}

fn create_town_map() -> Map {
    let mut map = Map::new(30, 25);
    
    // Create building blocks with streets
    for tile in map.tiles.iter_mut() {
        *tile = Tile::Floor { id: "cobblestone".to_string() }; // Streets
    }
    
    // Add buildings (walls)
    let buildings = vec![
        (2, 2, 6, 4),
        (10, 2, 8, 5),
        (20, 2, 7, 6),
        (2, 10, 5, 6),
        (10, 12, 6, 4),
        (20, 15, 8, 7),
    ];
    
    for (bx, by, bw, bh) in buildings {
        for y in by..(by + bh) {
            for x in bx..(bx + bw) {
                let idx = (y * map.width + x) as usize;
                map.tiles[idx] = Tile::Wall { id: "brick".to_string(), hp: 150 };
            }
        }
    }
    
    map
}

fn create_combat_arena() -> Map {
    let mut map = Map::new(25, 20);
    
    // Create arena with tactical features
    for tile in map.tiles.iter_mut() {
        *tile = Tile::Floor { id: "arena_floor".to_string() };
    }
    
    // Add walls around perimeter
    for x in 0..map.width {
        map.tiles[x as usize] = Tile::Wall { id: "arena_wall".to_string(), hp: 200 }; // Top
        map.tiles[((map.height - 1) * map.width + x) as usize] = Tile::Wall { id: "arena_wall".to_string(), hp: 200 }; // Bottom
    }
    for y in 0..map.height {
        map.tiles[(y * map.width) as usize] = Tile::Wall { id: "arena_wall".to_string(), hp: 200 }; // Left
        map.tiles[(y * map.width + map.width - 1) as usize] = Tile::Wall { id: "arena_wall".to_string(), hp: 200 }; // Right
    }
    
    // Add central obstacles for cover
    let cover = vec![
        (8, 8, 2, 2),
        (15, 8, 2, 2),
        (12, 12, 1, 3),
        (6, 15, 3, 1),
        (16, 15, 3, 1),
    ];
    
    for (cx, cy, cw, ch) in cover {
        for y in cy..(cy + ch) {
            for x in cx..(cx + cw) {
                let idx = (y * map.width + x) as usize;
                map.tiles[idx] = Tile::Wall { id: "cover".to_string(), hp: 50 };
            }
        }
    }
    
    map
}

fn create_corridor(map: &mut Map, x1: u32, y1: u32, x2: u32, y2: u32) {
    let mut x = x1 as i32;
    let mut y = y1 as i32;
    let target_x = x2 as i32;
    let target_y = y2 as i32;
    
    while x != target_x || y != target_y {
        let idx = (y * map.width as i32 + x) as usize;
        map.tiles[idx] = Tile::Floor { id: "corridor".to_string() };
        
        if x < target_x { x += 1; }
        else if x > target_x { x -= 1; }
        else if y < target_y { y += 1; }
        else if y > target_y { y -= 1; }
    }
}

fn create_dungeon_context(map: &Map) -> ConstraintContext<'_> {
    ConstraintContext {
        map,
        biome: Biome::Ruins,
        entities: vec![
            EntityPlacement { entity_type: "enemy".to_string(), x: 22, y: 7, properties: HashMap::new() },
            EntityPlacement { entity_type: "enemy".to_string(), x: 8, y: 20, properties: HashMap::new() },
            EntityPlacement { entity_type: "enemy".to_string(), x: 28, y: 23, properties: HashMap::new() },
        ],
        resources: vec![
            ResourcePlacement { resource_type: "chest".to_string(), x: 30, y: 8, amount: 1 },
        ],
        objectives: vec![
            ObjectivePlacement { objective_type: "boss".to_string(), x: 30, y: 23, required: true },
            ObjectivePlacement { objective_type: "exit".to_string(), x: 35, y: 25, required: true },
        ],
    }
}

fn create_overworld_context(map: &Map) -> ConstraintContext<'_> {
    ConstraintContext {
        map,
        biome: Biome::Desert,
        entities: vec![
            EntityPlacement { entity_type: "enemy".to_string(), x: 12, y: 12, properties: HashMap::new() },
            EntityPlacement { entity_type: "enemy".to_string(), x: 27, y: 17, properties: HashMap::new() },
        ],
        resources: vec![
            ResourcePlacement { resource_type: "oasis".to_string(), x: 5, y: 5, amount: 1 },
            ResourcePlacement { resource_type: "oasis".to_string(), x: 40, y: 30, amount: 1 },
        ],
        objectives: vec![
            ObjectivePlacement { objective_type: "landmark".to_string(), x: 45, y: 35, required: true },
        ],
    }
}

fn create_town_context(map: &Map) -> ConstraintContext<'_> {
    ConstraintContext {
        map,
        biome: Biome::Oasis,
        entities: vec![
            EntityPlacement { entity_type: "npc".to_string(), x: 5, y: 5, properties: HashMap::new() },
            EntityPlacement { entity_type: "npc".to_string(), x: 14, y: 8, properties: HashMap::new() },
            EntityPlacement { entity_type: "npc".to_string(), x: 24, y: 18, properties: HashMap::new() },
        ],
        resources: vec![
            ResourcePlacement { resource_type: "shop".to_string(), x: 12, y: 4, amount: 1 },
            ResourcePlacement { resource_type: "inn".to_string(), x: 22, y: 4, amount: 1 },
        ],
        objectives: vec![
            ObjectivePlacement { objective_type: "quest_giver".to_string(), x: 5, y: 5, required: true },
            ObjectivePlacement { objective_type: "town_hall".to_string(), x: 14, y: 14, required: true },
        ],
    }
}

fn create_arena_context(map: &Map) -> ConstraintContext<'_> {
    ConstraintContext {
        map,
        biome: Biome::Ruins,
        entities: vec![
            EntityPlacement { entity_type: "enemy".to_string(), x: 5, y: 5, properties: HashMap::new() },
            EntityPlacement { entity_type: "enemy".to_string(), x: 19, y: 5, properties: HashMap::new() },
            EntityPlacement { entity_type: "enemy".to_string(), x: 12, y: 15, properties: HashMap::new() },
        ],
        resources: vec![],
        objectives: vec![
            ObjectivePlacement { objective_type: "survive".to_string(), x: 12, y: 10, required: true },
        ],
    }
}

// Constraint fixing functions
fn fix_dungeon_constraints(mut map: Map, results: &[crate::game::generation::constraints::ConstraintResult]) -> Map {
    for result in results {
        if !result.passed && result.severity == crate::game::generation::constraints::ConstraintSeverity::Critical {
            match result.rule_id.as_str() {
                "tactical_chokepoints" => {
                    // Add more chokepoints by narrowing some corridors
                    narrow_corridors(&mut map);
                },
                "safe_zone_coverage" => {
                    // Create safe rooms by clearing areas far from enemies
                    create_safe_rooms(&mut map);
                },
                "escape_routes" => {
                    // Add alternative paths
                    add_escape_routes(&mut map);
                },
                _ => {}
            }
        }
    }
    map
}

fn fix_overworld_constraints(mut map: Map, results: &[crate::game::generation::constraints::ConstraintResult]) -> Map {
    for result in results {
        if !result.passed && result.severity == crate::game::generation::constraints::ConstraintSeverity::Critical {
            match result.rule_id.as_str() {
                "safe_zone_coverage" => {
                    // Create more open safe areas
                    create_safe_clearings(&mut map);
                },
                "escape_routes" => {
                    // Remove some obstacles to create more paths
                    remove_blocking_obstacles(&mut map);
                },
                _ => {}
            }
        }
    }
    map
}

fn fix_town_constraints(mut map: Map, results: &[crate::game::generation::constraints::ConstraintResult]) -> Map {
    for result in results {
        if !result.passed && result.severity == crate::game::generation::constraints::ConstraintSeverity::Critical {
            match result.rule_id.as_str() {
                "objective_accessibility" => {
                    // Widen streets and remove blocking buildings
                    widen_streets(&mut map);
                },
                "escape_routes" => {
                    // Add more street connections
                    add_street_connections(&mut map);
                },
                _ => {}
            }
        }
    }
    map
}

fn fix_arena_constraints(mut map: Map, results: &[crate::game::generation::constraints::ConstraintResult]) -> Map {
    for result in results {
        if !result.passed && result.severity == crate::game::generation::constraints::ConstraintSeverity::Critical {
            match result.rule_id.as_str() {
                "tactical_chokepoints" => {
                    // Adjust cover placement for better tactical positioning
                    adjust_cover_placement(&mut map);
                },
                "escape_routes" => {
                    // Ensure multiple exit paths
                    ensure_arena_exits(&mut map);
                },
                _ => {}
            }
        }
    }
    map
}

// Helper functions for constraint fixes
fn narrow_corridors(map: &mut Map) {
    // Find wide corridors and narrow them
    for y in 1..(map.height - 1) {
        for x in 1..(map.width - 1) {
            let idx = (y * map.width + x) as usize;
            if map.tiles[idx].walkable() {
                let adjacent_floors = count_adjacent_floors(map, x as i32, y as i32);
                if adjacent_floors >= 5 {
                    // This is a wide area, add some walls to create chokepoints
                    if (x + y) % 3 == 0 {
                        map.tiles[idx] = Tile::Wall { id: "narrow_wall".to_string(), hp: 100 };
                    }
                }
            }
        }
    }
}

fn create_safe_rooms(map: &mut Map) {
    // Create small safe rooms in corners
    let safe_areas = vec![(2, 2, 3, 3), (map.width - 5, 2, 3, 3)];
    for (sx, sy, sw, sh) in safe_areas {
        for y in sy..(sy + sh) {
            for x in sx..(sx + sw) {
                let idx = (y * map.width + x) as usize;
                map.tiles[idx] = Tile::Floor { id: "safe_floor".to_string() };
            }
        }
    }
}

fn add_escape_routes(map: &mut Map) {
    // Add alternative corridors
    let mid_y = map.height / 2;
    let end_x = if map.width >= 3 { map.width - 3 } else { 1 };
    create_corridor(map, 2, mid_y as u32, end_x as u32, mid_y as u32);
}

fn create_safe_clearings(map: &mut Map) {
    // Create large open areas
    let clearings = vec![(5, 5, 8, 8), (30, 25, 10, 10)];
    for (cx, cy, cw, ch) in clearings {
        for y in cy..(cy + ch) {
            for x in cx..(cx + cw) {
                if x < map.width && y < map.height {
                    let idx = (y * map.width + x) as usize;
                    map.tiles[idx] = Tile::Floor { id: "clearing".to_string() };
                }
            }
        }
    }
}

fn remove_blocking_obstacles(map: &mut Map) {
    // Remove some walls to create more paths
    for y in 1..(map.height - 1) {
        for x in 1..(map.width - 1) {
            let idx = (y * map.width + x) as usize;
            if !map.tiles[idx].walkable() && (x + y) % 4 == 0 {
                map.tiles[idx] = Tile::Floor { id: "opened_path".to_string() };
            }
        }
    }
}

fn widen_streets(map: &mut Map) {
    // Make streets wider by removing building edges
    for y in 0..map.height {
        for x in 0..map.width {
            let idx = (y * map.width + x) as usize;
            if !map.tiles[idx].walkable() {
                let adjacent_floors = count_adjacent_floors(map, x as i32, y as i32);
                if adjacent_floors >= 3 {
                    map.tiles[idx] = Tile::Floor { id: "widened_street".to_string() };
                }
            }
        }
    }
}

fn add_street_connections(map: &mut Map) {
    // Add diagonal street connections
    for y in 5..(map.height - 5) {
        for x in 5..(map.width - 5) {
            if (x + y) % 8 == 0 {
                let idx = (y * map.width + x) as usize;
                map.tiles[idx] = Tile::Floor { id: "connection".to_string() };
            }
        }
    }
}

fn adjust_cover_placement(map: &mut Map) {
    // Redistribute cover for better tactical positioning
    // Remove some central cover
    for y in 8..12 {
        for x in 10..15 {
            let idx = (y * map.width + x) as usize;
            map.tiles[idx] = Tile::Floor { id: "arena_floor".to_string() };
        }
    }
    
    // Add corner cover
    let new_cover = vec![(3, 3, 2, 2), (20, 3, 2, 2), (3, 16, 2, 2), (20, 16, 2, 2)];
    for (cx, cy, cw, ch) in new_cover {
        for y in cy..(cy + ch) {
            for x in cx..(cx + cw) {
                let idx = (y * map.width + x) as usize;
                map.tiles[idx] = Tile::Wall { id: "tactical_cover".to_string(), hp: 75 };
            }
        }
    }
}

fn ensure_arena_exits(map: &mut Map) {
    // Create exit points in walls
    let exits = vec![(0, map.height / 2), (map.width - 1, map.height / 2)];
    for (ex, ey) in exits {
        let idx = (ey * map.width + ex) as usize;
        map.tiles[idx] = Tile::Floor { id: "exit".to_string() };
    }
}

fn count_adjacent_floors(map: &Map, x: i32, y: i32) -> u32 {
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < map.width as i32 && ny >= 0 && ny < map.height as i32 {
                let idx = (ny * map.width as i32 + nx) as usize;
                if map.tiles[idx].walkable() {
                    count += 1;
                }
            }
        }
    }
    count
}

fn print_constraint_results(map_type: &str, results: &[crate::game::generation::constraints::ConstraintResult]) {
    println!("  {} Results:", map_type);
    let mut critical_failed = 0;
    let mut warning_failed = 0;
    
    for result in results {
        let status = if result.passed { "✓" } else { "✗" };
        let severity = match result.severity {
            crate::game::generation::constraints::ConstraintSeverity::Critical => "CRIT",
            crate::game::generation::constraints::ConstraintSeverity::Warning => "WARN",
            crate::game::generation::constraints::ConstraintSeverity::Suggestion => "SUGG",
        };
        
        println!("    {} [{}] {}: {} (score: {:.2})", 
            status, severity, result.rule_id, result.message, result.score);
        
        if !result.passed {
            match result.severity {
                crate::game::generation::constraints::ConstraintSeverity::Critical => critical_failed += 1,
                crate::game::generation::constraints::ConstraintSeverity::Warning => warning_failed += 1,
                _ => {}
            }
        }
    }
    
    let overall_score = crate::game::generation::constraints::ConstraintSystem::calculate_satisfaction_score(results);
    println!("    Overall Score: {:.2} | Critical Failed: {} | Warnings Failed: {}", 
        overall_score, critical_failed, warning_failed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_maps() {
        super::test_constraint_maps();
    }
}
