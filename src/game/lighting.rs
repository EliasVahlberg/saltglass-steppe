use super::constants::{MAP_WIDTH, MAP_HEIGHT};

/// Light level per tile (0 = dark, 255 = bright)
pub type LightMap = Vec<u8>;

/// A light source with position and intensity
#[derive(Clone, Debug)]
pub struct LightSource {
    pub x: i32,
    pub y: i32,
    pub radius: i32,
    pub intensity: u8,
}

/// Compute light levels for all tiles given light sources
pub fn compute_lighting(sources: &[LightSource], ambient: u8) -> LightMap {
    let mut light = vec![ambient; MAP_WIDTH * MAP_HEIGHT];
    
    for src in sources {
        for dy in -src.radius..=src.radius {
            for dx in -src.radius..=src.radius {
                let x = src.x + dx;
                let y = src.y + dy;
                if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
                    continue;
                }
                let dist_sq = dx * dx + dy * dy;
                let radius_sq = src.radius * src.radius;
                if dist_sq <= radius_sq {
                    let idx = y as usize * MAP_WIDTH + x as usize;
                    // Linear falloff
                    let falloff = 1.0 - (dist_sq as f32 / radius_sq as f32).sqrt();
                    let add = (src.intensity as f32 * falloff) as u8;
                    light[idx] = light[idx].saturating_add(add);
                }
            }
        }
    }
    light
}

/// Check if a tile is lit enough to see (above threshold)
pub fn is_lit(light_map: &LightMap, x: i32, y: i32, threshold: u8) -> bool {
    if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
        return false;
    }
    let idx = y as usize * MAP_WIDTH + x as usize;
    light_map[idx] >= threshold
}

/// Check if a tile has glare (too bright, causes damage/blindness)
pub fn has_glare(light_map: &LightMap, x: i32, y: i32, glare_threshold: u8) -> bool {
    if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
        return false;
    }
    let idx = y as usize * MAP_WIDTH + x as usize;
    light_map[idx] >= glare_threshold
}

/// Get light level at specific position
pub fn get_light_level(light_map: &LightMap, x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
        return 0;
    }
    let idx = y as usize * MAP_WIDTH + x as usize;
    light_map[idx]
}

/// Calculate visibility modifier based on light level
pub fn visibility_modifier(light_level: u8) -> f32 {
    match light_level {
        0..=30 => 0.1,      // Very dark - hard to see
        31..=60 => 0.5,     // Dim - reduced visibility  
        61..=120 => 1.0,    // Normal - full visibility
        121..=200 => 1.2,   // Bright - enhanced visibility
        201..=255 => 0.8,   // Glare - reduced visibility from brightness
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_source_illuminates_area() {
        let sources = vec![LightSource { x: 10, y: 10, radius: 5, intensity: 200 }];
        let light = compute_lighting(&sources, 0);
        // Center should be bright
        assert!(light[10 * MAP_WIDTH + 10] >= 200);
        // Edge should be dimmer
        assert!(light[10 * MAP_WIDTH + 14] > 0);
        assert!(light[10 * MAP_WIDTH + 14] < 200);
        // Outside radius should be ambient (0)
        assert_eq!(light[10 * MAP_WIDTH + 20], 0);
    }

    #[test]
    fn ambient_light_baseline() {
        let sources = vec![];
        let light = compute_lighting(&sources, 50);
        assert!(light.iter().all(|&l| l == 50));
    }
}
