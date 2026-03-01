use super::{SettlementConfig, SettlementTier};
use rand::Rng;
use terrain_forge::{
    Algorithm, Grid, Rng as ForgeRng, SemanticExtractor,
    algorithms::{Bsp, BspConfig, Voronoi, VoronoiConfig},
};

/// Calculate settlement dimensions based on tier
pub fn calculate_dimensions(config: &SettlementConfig) -> (usize, usize) {
    match config.tier {
        SettlementTier::Village => (80, 60),
        SettlementTier::Town => (120, 90),
        SettlementTier::City => (180, 120),
    }
}

/// Generate a settlement layout and return building placement positions (region centroids).
/// Village uses Voronoi (organic scattered plots), Town/City use BSP (structured blocks).
pub fn generate_layout<R: Rng>(
    config: &SettlementConfig,
    width: usize,
    height: usize,
    rng: &mut R,
) -> Vec<(i32, i32)> {
    let seed = rng.next_u64();
    let mut grid = Grid::new(width, height);

    match config.tier {
        SettlementTier::Village => {
            let num_points = if width * height > 5000 { 12 } else { 8 };
            Voronoi::new(VoronoiConfig { num_points, floor_chance: 0.65 })
                .generate(&mut grid, seed);
        }
        SettlementTier::Town => {
            Bsp::new(BspConfig { min_room_size: 8, max_depth: 4, room_padding: 2 })
                .generate(&mut grid, seed);
        }
        SettlementTier::City => {
            Bsp::new(BspConfig { min_room_size: 8, max_depth: 5, room_padding: 2 })
                .generate(&mut grid, seed);
        }
    }

    let mut forge_rng = ForgeRng::new(seed);
    let semantic = SemanticExtractor::for_rooms().extract(&grid, &mut forge_rng);

    // Sort regions largest-first, compute centroids as placement positions
    let mut regions: Vec<_> = semantic.regions.iter().filter(|r| r.area() >= 20).collect();
    regions.sort_by(|a, b| b.area().cmp(&a.area()));

    regions
        .iter()
        .filter_map(|r| {
            if r.cells.is_empty() {
                return None;
            }
            let sum_x: u32 = r.cells.iter().map(|(x, _)| x).sum();
            let sum_y: u32 = r.cells.iter().map(|(_, y)| y).sum();
            let count = r.cells.len() as u32;
            Some(((sum_x / count) as i32, (sum_y / count) as i32))
        })
        .collect()
}
