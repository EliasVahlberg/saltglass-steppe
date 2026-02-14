//! Visual effects system — damage numbers, projectile trails, light beams, hit flashes.
//!
//! Extracted from state.rs to separate rendering-only state from gameplay state.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Floating damage number for combat feedback
#[derive(Clone, Serialize, Deserialize)]
pub struct DamageNumber {
    pub x: i32,
    pub y: i32,
    pub value: i32,
    pub frames: u32,
    pub is_heal: bool,
}

/// Projectile trail for ranged attack animation
#[derive(Clone)]
pub struct ProjectileTrail {
    pub path: Vec<(i32, i32)>,
    pub current_idx: usize,
    pub frames_per_tile: u32,
    pub frame_counter: u32,
    pub char: char,
}

/// Light beam for tactical visualization
#[derive(Clone)]
pub struct LightBeam {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub path: Vec<(i32, i32)>,
    pub frames_remaining: u32,
    pub beam_type: BeamType,
}

#[derive(Clone)]
pub enum BeamType {
    Laser,      // Red beam, damage
    Light,      // Yellow beam, illumination
    Reflection, // Cyan beam, mirror reflection
    Arrow,      // Green beam, ranged attack
}

/// All visual-only state, separated from gameplay state.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualEffects {
    #[serde(skip)]
    pub hit_flash_positions: Vec<(i32, i32, u32)>,
    #[serde(skip)]
    pub damage_numbers: Vec<DamageNumber>,
    #[serde(skip)]
    pub projectile_trails: Vec<ProjectileTrail>,
    #[serde(skip)]
    pub light_beams: Vec<LightBeam>,
    #[serde(skip)]
    pub animation_frame: u32,
    #[serde(skip)]
    pub storm_changed_tiles: HashSet<usize>,
}

impl VisualEffects {
    /// Trigger a hit flash effect at position
    pub fn trigger_hit_flash(&mut self, x: i32, y: i32) {
        self.hit_flash_positions.push((x, y, 6)); // 6 frames
    }

    /// Tick hit flash animations (call each frame)
    pub fn tick_hit_flash(&mut self) {
        self.hit_flash_positions.retain_mut(|(_, _, frames)| {
            *frames = frames.saturating_sub(1);
            *frames > 0
        });
    }

    /// Check if position has active hit flash
    pub fn has_hit_flash(&self, x: i32, y: i32) -> bool {
        self.hit_flash_positions
            .iter()
            .any(|(fx, fy, _)| *fx == x && *fy == y)
    }

    /// Spawn a floating damage number
    pub fn spawn_damage_number(&mut self, x: i32, y: i32, value: i32, is_heal: bool) {
        self.damage_numbers.push(DamageNumber {
            x,
            y,
            value,
            frames: 12,
            is_heal,
        });
    }

    /// Tick damage number animations
    pub fn tick_damage_numbers(&mut self) {
        self.damage_numbers.retain_mut(|dn| {
            dn.frames = dn.frames.saturating_sub(1);
            dn.frames > 0
        });
    }

    /// Tick animation frame for ambient tile animations
    pub fn tick_animation(&mut self) {
        self.animation_frame = self.animation_frame.wrapping_add(1);
    }

    /// Spawn a projectile trail from source to target
    pub fn spawn_projectile(&mut self, from: (i32, i32), to: (i32, i32), ch: char) {
        let path = line_path(from, to);
        if path.len() > 1 {
            self.projectile_trails.push(ProjectileTrail {
                path,
                current_idx: 0,
                frames_per_tile: 2,
                frame_counter: 0,
                char: ch,
            });
        }
    }

    /// Tick projectile trail animations
    pub fn tick_projectile_trails(&mut self) {
        self.projectile_trails.retain_mut(|pt| {
            pt.frame_counter += 1;
            if pt.frame_counter >= pt.frames_per_tile {
                pt.frame_counter = 0;
                pt.current_idx += 1;
            }
            pt.current_idx < pt.path.len()
        });
    }

    /// Get current projectile position if any
    pub fn get_projectile_at(&self, x: i32, y: i32) -> Option<char> {
        for pt in &self.projectile_trails {
            if pt.current_idx < pt.path.len() {
                let (px, py) = pt.path[pt.current_idx];
                if px == x && py == y {
                    return Some(pt.char);
                }
            }
        }
        None
    }

    /// Spawn a light beam from source to target
    pub fn spawn_beam(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        beam_type: BeamType,
        duration: u32,
    ) {
        let path = line_path(from, to);
        if path.len() > 1 {
            self.light_beams.push(LightBeam {
                start_x: from.0,
                start_y: from.1,
                end_x: to.0,
                end_y: to.1,
                path,
                frames_remaining: duration,
                beam_type,
            });
        }
    }

    /// Tick light beam animations
    pub fn tick_light_beams(&mut self) {
        self.light_beams.retain_mut(|beam| {
            beam.frames_remaining = beam.frames_remaining.saturating_sub(1);
            beam.frames_remaining > 0
        });
    }

    /// Get beam character at position if any
    pub fn get_beam_at(&self, x: i32, y: i32) -> Option<(char, BeamType)> {
        for beam in &self.light_beams {
            for &(bx, by) in &beam.path {
                if bx == x && by == y {
                    let dx = beam.end_x - beam.start_x;
                    let dy = beam.end_y - beam.start_y;
                    let ch = if dx.abs() > dy.abs() {
                        '-'
                    } else if dy.abs() > dx.abs() {
                        '|'
                    } else if (dx > 0 && dy > 0) || (dx < 0 && dy < 0) {
                        '\\'
                    } else {
                        '/'
                    };
                    return Some((ch, beam.beam_type.clone()));
                }
            }
        }
        None
    }
}

/// Bresenham line algorithm for projectile/beam paths
fn line_path(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let mut path = Vec::new();
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        path.push((x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    path
}
