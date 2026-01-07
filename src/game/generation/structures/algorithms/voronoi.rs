use crate::game::generation::structures::Rectangle;
use rand_chacha::ChaCha8Rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoronoiParams {
    pub num_sites: u32,
    pub relaxation_iterations: u32,
    pub cell_type_distribution: HashMap<String, f32>,
    pub border_thickness: u32,
}

impl Default for VoronoiParams {
    fn default() -> Self {
        let mut distribution = HashMap::new();
        distribution.insert("floor".to_string(), 0.7);
        distribution.insert("wall".to_string(), 0.2);
        distribution.insert("special".to_string(), 0.1);
        
        Self {
            num_sites: 20,
            relaxation_iterations: 2,
            cell_type_distribution: distribution,
            border_thickness: 1,
        }
    }
}

#[derive(Debug, Clone)]
struct VoronoiSite {
    x: f32,
    y: f32,
    cell_type: String,
}

pub struct VoronoiGenerator {
    params: VoronoiParams,
}

impl VoronoiGenerator {
    pub fn new(params: VoronoiParams) -> Self {
        Self { params }
    }

    pub fn generate(&self, bounds: Rectangle, rng: &mut ChaCha8Rng) -> HashMap<String, Vec<(u32, u32)>> {
        // Generate initial sites
        let sites = self.generate_sites(&bounds, rng);
        
        // Generate Voronoi diagram (skip relaxation for now to avoid complexity)
        self.generate_voronoi_cells(&sites, &bounds)
    }

    fn generate_sites(&self, bounds: &Rectangle, rng: &mut ChaCha8Rng) -> Vec<VoronoiSite> {
        let mut sites = Vec::new();
        
        for _ in 0..self.params.num_sites {
            let rand_x = rng.r#gen::<f32>();
            let rand_y = rng.r#gen::<f32>();
            let x = bounds.x as f32 + rand_x * bounds.width as f32;
            let y = bounds.y as f32 + rand_y * bounds.height as f32;
            let cell_type = self.select_cell_type(rng);
            
            sites.push(VoronoiSite { x, y, cell_type });
        }
        
        sites
    }

    fn select_cell_type(&self, rng: &mut ChaCha8Rng) -> String {
        let mut cumulative = 0.0;
        let roll = rng.r#gen::<f32>();
        
        for (cell_type, probability) in &self.params.cell_type_distribution {
            cumulative += probability;
            if roll <= cumulative {
                return cell_type.clone();
            }
        }
        
        "floor".to_string() // Default fallback
    }

    fn closest_site_index(&self, x: f32, y: f32, sites: &[VoronoiSite]) -> usize {
        let mut closest_idx = 0;
        let mut min_distance = f32::INFINITY;
        
        for (i, site) in sites.iter().enumerate() {
            let distance = ((x - site.x).powi(2) + (y - site.y).powi(2)).sqrt();
            if distance < min_distance {
                min_distance = distance;
                closest_idx = i;
            }
        }
        
        closest_idx
    }

    fn generate_voronoi_cells(&self, sites: &[VoronoiSite], bounds: &Rectangle) -> HashMap<String, Vec<(u32, u32)>> {
        let mut cells: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
        
        for y in bounds.y..bounds.y + bounds.width {
            for x in bounds.x..bounds.x + bounds.height {
                let closest_idx = self.closest_site_index(x as f32, y as f32, sites);
                let cell_type = &sites[closest_idx].cell_type;
                
                // Add border thickness
                let is_border = self.is_border_point(x, y, sites, bounds);
                let final_type = if is_border && self.params.border_thickness > 0 {
                    "wall".to_string()
                } else {
                    cell_type.clone()
                };
                
                cells.entry(final_type).or_insert_with(Vec::new).push((x, y));
            }
        }
        
        cells
    }

    fn is_border_point(&self, x: u32, y: u32, sites: &[VoronoiSite], bounds: &Rectangle) -> bool {
        let current_site = self.closest_site_index(x as f32, y as f32, sites);
        
        // Check neighboring points
        for dy in 0..=self.params.border_thickness {
            for dx in 0..=self.params.border_thickness {
                if dx == 0 && dy == 0 { continue; }
                
                let nx = x.saturating_add(dx);
                let ny = y.saturating_add(dy);
                
                if nx < bounds.x + bounds.width && ny < bounds.y + bounds.height {
                    let neighbor_site = self.closest_site_index(nx as f32, ny as f32, sites);
                    if neighbor_site != current_site {
                        return true;
                    }
                }
            }
        }
        
        false
    }
}
