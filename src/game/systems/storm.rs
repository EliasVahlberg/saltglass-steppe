use crate::game::state::{GameState, MsgType};
use crate::game::event::GameEvent;
use crate::game::map::Tile;
use crate::game::enemy::Enemy;
use crate::game::storm::{Storm, StormEditType, refraction_multiplier, wraith_spawn_max, storm_glass_drop_chance};
use super::System;
use rand::Rng;

/// Handles glass storm events and map transformations
pub struct StormSystem;

impl System for StormSystem {
    fn update(&self, _state: &mut GameState) {
        // Storm ticking is handled in end_turn, this is for event reactions
    }
    
    fn on_event(&self, state: &mut GameState, event: &GameEvent) {
        if let GameEvent::StormArrived { intensity } = event {
            state.log_typed(
                format!("The storm settles. Intensity was {}.", intensity),
                MsgType::System
            );
        }
    }
}

impl StormSystem {
    /// Apply a storm to the game state
    pub fn apply_storm(state: &mut GameState) {
        state.log(format!("⚡ GLASS STORM! Intensity {}", state.storm.intensity));
        
        let refraction_gain = state.storm.intensity as u32 * refraction_multiplier();
        state.refraction += refraction_gain;
        state.check_adaptation_threshold();
        
        // Clear previous storm changes
        state.storm_changed_tiles.clear();
        
        // Apply each edit type
        let edit_types = state.storm.edit_types.clone();
        for edit_type in &edit_types {
            match edit_type {
                StormEditType::Glass => Self::apply_glass_edit(state),
                StormEditType::Rotate => Self::apply_rotate_edit(state),
                StormEditType::Swap => Self::apply_swap_edit(state),
                StormEditType::Mirror => Self::apply_mirror_edit(state),
                StormEditType::Fracture => Self::apply_fracture_edit(state),
                StormEditType::Crystallize => Self::apply_crystallize_edit(state),
                StormEditType::Vortex => Self::apply_vortex_edit(state),
            }
        }
        
        // Spawn storm enemies on glass tiles
        Self::spawn_storm_enemies(state);
        
        // Emit event and forecast next storm
        let intensity = state.storm.intensity;
        state.emit(GameEvent::StormArrived { intensity });
        state.storm = Storm::forecast(&mut state.rng);
        state.update_fov();
        state.update_lighting();
    }
    
    /// Spawn wraiths on glass tiles after storm
    fn spawn_storm_enemies(state: &mut GameState) {
        let glass_tiles: Vec<(i32, i32)> = (0..state.map.tiles.len())
            .filter(|&i| state.map.tiles[i] == Tile::Glass)
            .map(|i| ((i % state.map.width) as i32, (i / state.map.width) as i32))
            .filter(|&(x, y)| state.enemy_at(x, y).is_none() && !(x == state.player_x && y == state.player_y))
            .collect();
            
        if !glass_tiles.is_empty() {
            let spawn_count = (state.storm.intensity as usize).min(wraith_spawn_max());
            for _ in 0..spawn_count {
                let idx = state.rng.gen_range(0..glass_tiles.len());
                let (x, y) = glass_tiles[idx];
                let enemy_idx = state.enemies.len();
                state.enemies.push(Enemy::new(x, y, "refraction_wraith"));
                state.enemy_positions.insert((x, y), enemy_idx);
                state.log("A wraith coalesces from the storm's edge.");
            }
        }
    }
    
    /// Convert walls to glass and potentially drop storm_glass items
    fn apply_glass_edit(state: &mut GameState) {
        let intensity = state.storm.intensity as usize;
        for _ in 0..(intensity * 5) {
            let x = state.rng.gen_range(1..state.map.width - 1);
            let y = state.rng.gen_range(1..state.map.height - 1);
            let idx = y * state.map.width + x;
            
            if matches!(state.map.tiles[idx], Tile::Wall { .. }) {
                state.map.tiles[idx] = Tile::Glass;
                state.storm_changed_tiles.insert(idx);
                
                // Chance to spawn storm_glass item
                let roll: f32 = state.rng.gen_range(0.0..1.0);
                if roll < storm_glass_drop_chance() {
                    if !state.items.iter().any(|item| item.x == x as i32 && item.y == y as i32) {
                        state.items.push(crate::game::item::Item::new(x as i32, y as i32, "storm_glass"));
                    }
                }
            }
        }
    }
    
    /// Rotate small 3x3 sections of the map
    fn apply_rotate_edit(state: &mut GameState) {
        let intensity = state.storm.intensity as usize;
        for _ in 0..(intensity * 2) {
            let center_x = state.rng.gen_range(2..state.map.width - 2);
            let center_y = state.rng.gen_range(2..state.map.height - 2);
            
            // Extract 3x3 area
            let mut area = vec![vec![Tile::Floor; 3]; 3];
            for dy in 0..3 {
                for dx in 0..3 {
                    let x = center_x + dx - 1;
                    let y = center_y + dy - 1;
                    area[dy][dx] = state.map.tiles[y * state.map.width + x].clone();
                }
            }
            
            // Rotate 90 degrees clockwise
            let mut rotated = vec![vec![Tile::Floor; 3]; 3];
            for dy in 0..3 {
                for dx in 0..3 {
                    rotated[dx][2 - dy] = area[dy][dx].clone();
                }
            }
            
            // Place back
            for dy in 0..3 {
                for dx in 0..3 {
                    let x = center_x + dx - 1;
                    let y = center_y + dy - 1;
                    let idx = y * state.map.width + x;
                    if state.map.tiles[idx] != rotated[dy][dx] {
                        state.map.tiles[idx] = rotated[dy][dx].clone();
                        state.storm_changed_tiles.insert(idx);
                    }
                }
            }
        }
    }
    
    /// Swap terrain types in small areas
    fn apply_swap_edit(state: &mut GameState) {
        let intensity = state.storm.intensity as usize;
        for _ in 0..(intensity * 3) {
            let x = state.rng.gen_range(1..state.map.width - 1);
            let y = state.rng.gen_range(1..state.map.height - 1);
            let idx = y * state.map.width + x;
            
            let new_tile = match &state.map.tiles[idx] {
                Tile::Floor => {
                    let roll = state.rng.gen_range(0..100);
                    if roll < 20 { Tile::Glass }
                    else if roll < 25 { Tile::Glare }
                    else { Tile::Floor }
                },
                Tile::Glass => if state.rng.gen_bool(0.5) { Tile::Floor } else { Tile::Glass },
                Tile::Wall { .. } => if state.rng.gen_bool(0.2) { Tile::Floor } else { state.map.tiles[idx].clone() },
                other => other.clone(),
            };
            
            if state.map.tiles[idx] != new_tile {
                state.map.tiles[idx] = new_tile;
                state.storm_changed_tiles.insert(idx);
            }
        }
    }
    
    /// Mirror sections of the map horizontally or vertically
    fn apply_mirror_edit(state: &mut GameState) {
        let intensity = state.storm.intensity as usize;
        for _ in 0..intensity {
            let size = state.rng.gen_range(3..8);
            let x = state.rng.gen_range(1..state.map.width - size);
            let y = state.rng.gen_range(1..state.map.height - size);
            let horizontal = state.rng.gen_bool(0.5);
            
            if horizontal {
                for dy in 0..size {
                    for dx in 0..size/2 {
                        let left_idx = (y + dy) * state.map.width + (x + dx);
                        let right_idx = (y + dy) * state.map.width + (x + size - 1 - dx);
                        
                        let left_tile = state.map.tiles[left_idx].clone();
                        state.map.tiles[left_idx] = state.map.tiles[right_idx].clone();
                        state.map.tiles[right_idx] = left_tile;
                        
                        state.storm_changed_tiles.insert(left_idx);
                        state.storm_changed_tiles.insert(right_idx);
                    }
                }
            } else {
                for dy in 0..size/2 {
                    for dx in 0..size {
                        let top_idx = (y + dy) * state.map.width + (x + dx);
                        let bottom_idx = (y + size - 1 - dy) * state.map.width + (x + dx);
                        
                        let top_tile = state.map.tiles[top_idx].clone();
                        state.map.tiles[top_idx] = state.map.tiles[bottom_idx].clone();
                        state.map.tiles[bottom_idx] = top_tile;
                        
                        state.storm_changed_tiles.insert(top_idx);
                        state.storm_changed_tiles.insert(bottom_idx);
                    }
                }
            }
        }
    }
    
    /// Create glass seams/cracks through terrain
    fn apply_fracture_edit(state: &mut GameState) {
        let intensity = state.storm.intensity as usize;
        for _ in 0..(intensity * 2) {
            let start_x = state.rng.gen_range(1..state.map.width - 1);
            let start_y = state.rng.gen_range(1..state.map.height - 1);
            let length = state.rng.gen_range(5..15);
            let angle = state.rng.gen_range(0..8);
            
            let (dx, dy) = match angle {
                0 => (1, 0), 1 => (1, 1), 2 => (0, 1), 3 => (-1, 1),
                4 => (-1, 0), 5 => (-1, -1), 6 => (0, -1), _ => (1, -1),
            };
            
            let mut x = start_x as i32;
            let mut y = start_y as i32;
            
            for _ in 0..length {
                if x >= 1 && x < (state.map.width - 1) as i32 && 
                   y >= 1 && y < (state.map.height - 1) as i32 {
                    let idx = (y as usize) * state.map.width + (x as usize);
                    if !matches!(state.map.tiles[idx], Tile::Glass) {
                        state.map.tiles[idx] = Tile::Glass;
                        state.storm_changed_tiles.insert(idx);
                    }
                }
                x += dx;
                y += dy;
            }
        }
    }
    
    /// Convert floor tiles to crystal formations (glare tiles)
    fn apply_crystallize_edit(state: &mut GameState) {
        let intensity = state.storm.intensity as usize;
        for _ in 0..(intensity * 4) {
            let center_x = state.rng.gen_range(2..state.map.width - 2);
            let center_y = state.rng.gen_range(2..state.map.height - 2);
            let radius = state.rng.gen_range(1..4);
            
            for dy in -(radius as i32)..=(radius as i32) {
                for dx in -(radius as i32)..=(radius as i32) {
                    if dx * dx + dy * dy <= (radius * radius) as i32 {
                        let x = (center_x as i32 + dx) as usize;
                        let y = (center_y as i32 + dy) as usize;
                        
                        if x < state.map.width && y < state.map.height {
                            let idx = y * state.map.width + x;
                            if matches!(state.map.tiles[idx], Tile::Floor) {
                                state.map.tiles[idx] = Tile::Glare;
                                state.storm_changed_tiles.insert(idx);
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Spiral rearrangement of map sections
    fn apply_vortex_edit(state: &mut GameState) {
        let intensity = state.storm.intensity as usize;
        for _ in 0..intensity {
            let center_x = state.rng.gen_range(3..state.map.width - 3);
            let center_y = state.rng.gen_range(3..state.map.height - 3);
            let radius = 3;
            
            let mut tiles = Vec::new();
            let mut positions = Vec::new();
            
            for r in 1..=radius {
                for angle in 0..(r * 8) {
                    let theta = (angle as f32) * std::f32::consts::PI * 2.0 / (r * 8) as f32;
                    let x = center_x as i32 + (r as f32 * theta.cos()) as i32;
                    let y = center_y as i32 + (r as f32 * theta.sin()) as i32;
                    
                    if x >= 0 && x < state.map.width as i32 && y >= 0 && y < state.map.height as i32 {
                        let idx = (y as usize) * state.map.width + (x as usize);
                        tiles.push(state.map.tiles[idx].clone());
                        positions.push((x as usize, y as usize));
                    }
                }
            }
            
            // Rotate tiles by one position
            if !tiles.is_empty() {
                let first_tile = tiles[0].clone();
                let len = tiles.len();
                for i in 0..len - 1 {
                    tiles[i] = tiles[i + 1].clone();
                }
                tiles[len - 1] = first_tile;
                
                for (i, &(x, y)) in positions.iter().enumerate() {
                    let idx = y * state.map.width + x;
                    if state.map.tiles[idx] != tiles[i] {
                        state.map.tiles[idx] = tiles[i].clone();
                        state.storm_changed_tiles.insert(idx);
                    }
                }
            }
        }
    }
}
