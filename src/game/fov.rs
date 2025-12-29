use crate::game::map::{Map, Tile};
use std::collections::HashSet;

/// Field of View calculation using simple line-of-sight algorithm
/// Temporary replacement for broken shadow casting implementation
#[derive(Debug, Clone)]
pub struct FieldOfView {
    pub visible_tiles: HashSet<(i32, i32)>,
    pub range: i32,
    pub dirty: bool,
}

impl Default for FieldOfView {
    fn default() -> Self {
        Self::new(super::constants::FOV_RANGE)
    }
}

impl FieldOfView {
    pub fn new(range: i32) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            range,
            dirty: true,
        }
    }

    pub fn is_visible(&self, pos: (i32, i32)) -> bool {
        self.visible_tiles.contains(&pos)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn calculate(&mut self, map: &Map, start_pos: (i32, i32)) {
        if !self.dirty {
            return;
        }

        self.visible_tiles.clear();
        
        // Always see the starting position
        self.visible_tiles.insert(start_pos);

        // Simple circular FOV with line-of-sight checks
        for dy in -self.range..=self.range {
            for dx in -self.range..=self.range {
                let x = start_pos.0 + dx;
                let y = start_pos.1 + dy;
                
                // Check if within range (circular)
                let distance_sq = dx * dx + dy * dy;
                if distance_sq > self.range * self.range {
                    continue;
                }
                
                // Check if position is valid
                if !map.is_valid_position(x, y) {
                    continue;
                }
                
                // Simple line-of-sight check
                if self.has_line_of_sight(map, start_pos, (x, y)) {
                    self.visible_tiles.insert((x, y));
                }
            }
        }

        self.dirty = false;
    }
    
    // Simple line-of-sight using Bresenham's line algorithm
    fn has_line_of_sight(&self, map: &Map, start: (i32, i32), end: (i32, i32)) -> bool {
        let mut x0 = start.0;
        let mut y0 = start.1;
        let x1 = end.0;
        let y1 = end.1;
        
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        
        loop {
            // Check if current position blocks vision (but allow seeing the blocking tile itself)
            if (x0, y0) != end {
                let tile = map.get_tile(x0, y0);
                if matches!(tile, Tile::Wall { .. }) {
                    return false;
                }
            }
            
            if x0 == x1 && y0 == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
        
        true
    }
}
