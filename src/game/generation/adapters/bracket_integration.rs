use bracket_pathfinding::prelude::*;
use bracket_geometry::prelude::Point;
use crate::game::map::Map;

/// Test basic pathfinding functionality with bracket-lib
pub fn test_pathfinding(map: &Map, start: Point, end: Point) -> Option<NavigationPath> {
    let start_idx = map.point2d_to_index(start);
    let end_idx = map.point2d_to_index(end);
    
    let path = a_star_search(start_idx, end_idx, map);
    
    if path.success {
        Some(path)
    } else {
        None
    }
}

/// Test field of view functionality with bracket-lib
pub fn test_fov(map: &Map, center: Point, range: i32) -> Vec<Point> {
    field_of_view(center, range, map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::{Map, Tile};
    
    #[test]
    fn test_bracket_pathfinding_integration() {
        let mut map = Map::new(20, 20);
        
        // Create a simple corridor
        for x in 5..15 {
            map.set_tile(x, 10, Tile::default_floor());
        }
        
        let start = Point::new(5, 10);
        let end = Point::new(14, 10);
        
        let path = test_pathfinding(&map, start, end);
        assert!(path.is_some(), "Should find a path");
        
        let path = path.unwrap();
        assert!(path.steps.len() > 0, "Path should have steps");
    }
    
    #[test]
    fn test_bracket_fov_integration() {
        let mut map = Map::new(20, 20);
        
        // Create open area
        for x in 5..15 {
            for y in 5..15 {
                map.set_tile(x, y, Tile::default_floor());
            }
        }
        
        let center = Point::new(10, 10);
        let visible = test_fov(&map, center, 5);
        
        assert!(visible.len() > 0, "Should see some tiles");
        assert!(visible.contains(&center), "Should see center tile");
    }
}
