//! Glass Seam Bridging Algorithm
//!
//! Ensures map connectivity by finding optimal tunnels between disconnected regions.
//! See docs/development/GLASS_SEAM_BRIDGING_ALGORITHM.md for full documentation.

use std::collections::{HashMap, HashSet, VecDeque};
use rand_chacha::ChaCha8Rng;

use crate::game::map::{Map, Tile};
use crate::game::constants::{MAP_WIDTH, MAP_HEIGHT};

// ============================================================================
// Configuration
// ============================================================================

/// Parameters for the Glass Seam Bridging algorithm
#[derive(Debug, Clone)]
pub struct GSBParams {
    /// Connectivity threshold (0.0-1.0) - fraction of floor that must be reachable
    pub connectivity_threshold: f32,
    /// Minimum area ratio to consider a region (filters tiny pockets)
    pub min_area_ratio: f32,
    /// Use Delaunay triangulation for edge filtering
    pub use_delaunay: bool,
    /// Number of angular sectors for pruning
    pub angular_sectors: usize,
    /// Occlusion factor for indirect path pruning
    pub occlusion_factor: f32,
    /// Maximum centroid distance for edge consideration
    pub max_edge_distance: f32,
    /// Use Perimeter Gradient Descent optimization
    pub use_pgd: bool,
    /// PGD skew limit
    pub pgd_skew: usize,
    /// PGD max iterations
    pub pgd_max_iter: usize,
    /// Use Frustum Ray Refinement
    pub use_frr: bool,
    /// FRR visibility cone angle (degrees)
    pub frr_theta_max: f32,
    /// FRR refinement depth
    pub frr_depth: usize,
    /// Tunnel width
    pub tunnel_width: usize,
}

impl Default for GSBParams {
    fn default() -> Self {
        Self {
            connectivity_threshold: 0.75,
            min_area_ratio: 0.05,
            use_delaunay: true,
            angular_sectors: 6,
            occlusion_factor: 1.2,
            max_edge_distance: 150.0,
            use_pgd: true,
            pgd_skew: 2,
            pgd_max_iter: 20,
            use_frr: false,
            frr_theta_max: 45.0,
            frr_depth: 3,
            tunnel_width: 2,
        }
    }
}

impl GSBParams {
    /// Fast profile for real-time generation
    pub fn fast() -> Self {
        Self {
            connectivity_threshold: 0.60,
            min_area_ratio: 0.10,
            angular_sectors: 4,
            occlusion_factor: 1.0,
            pgd_skew: 1,
            use_frr: false,
            ..Default::default()
        }
    }
    
    /// Quality profile for pre-computed maps
    pub fn quality() -> Self {
        Self {
            connectivity_threshold: 0.85,
            min_area_ratio: 0.02,
            angular_sectors: 8,
            occlusion_factor: 1.5,
            pgd_skew: 4,
            use_frr: true,
            frr_depth: 4,
            ..Default::default()
        }
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// A connected region of floor tiles
#[derive(Debug, Clone)]
pub struct Region {
    pub index: usize,
    pub tiles: Vec<(i32, i32)>,
    pub perimeter: Vec<(i32, i32)>,
    pub centroid: (f32, f32),
    pub size: usize,
    pub weight: f32,
}

/// A potential tunnel between two regions
#[derive(Debug, Clone)]
pub struct TunnelEdge {
    pub region_a: usize,
    pub region_b: usize,
    pub exit_a: (i32, i32),
    pub exit_b: (i32, i32),
    pub cost: usize,
}

/// Result of connectivity analysis
#[derive(Debug)]
pub struct ConnectivityAnalysis {
    pub regions: Vec<Region>,
    pub spawn_region: usize,
    pub spawn_coverage: f32,
    pub total_floor: usize,
}

// ============================================================================
// Core Algorithm
// ============================================================================

/// Main entry point: ensure map connectivity from spawn point
pub fn ensure_connectivity(
    map: &mut Map,
    spawn: (i32, i32),
    params: &GSBParams,
    rng: &mut ChaCha8Rng,
) -> Vec<TunnelEdge> {
    // Step 1-3: Analyze connectivity
    let analysis = analyze_connectivity(map, spawn, params);
    
    // Check if already connected enough
    if analysis.spawn_coverage >= params.connectivity_threshold {
        return Vec::new();
    }
    
    // Step 4-5: Compute and prune edges
    let mut edges = compute_all_edges(&analysis.regions, map, params);
    prune_edges(&mut edges, &analysis.regions, params);
    
    // Step 6: Optimize edge costs
    if params.use_pgd || params.use_frr {
        optimize_edges(&mut edges, &analysis.regions, map, params);
    }
    
    // Step 7: Select optimal tunnels
    let selected = select_tunnels(
        &analysis.regions,
        &edges,
        analysis.spawn_region,
        params.connectivity_threshold,
    );
    
    // Step 8: Carve tunnels
    for edge in &selected {
        carve_tunnel(map, edge, params.tunnel_width, rng);
    }
    
    selected
}

// ============================================================================
// Step 1-3: Region Identification
// ============================================================================

/// Analyze map connectivity and identify regions
pub fn analyze_connectivity(map: &Map, spawn: (i32, i32), params: &GSBParams) -> ConnectivityAnalysis {
    let regions = identify_regions(map, params.min_area_ratio);
    let total_floor: usize = regions.iter().map(|r| r.size).sum();
    
    // Find spawn region
    let spawn_region = regions.iter()
        .position(|r| r.tiles.contains(&spawn))
        .unwrap_or(0);
    
    let spawn_coverage = if total_floor > 0 {
        regions.get(spawn_region).map(|r| r.weight).unwrap_or(0.0)
    } else {
        0.0
    };
    
    ConnectivityAnalysis {
        regions,
        spawn_region,
        spawn_coverage,
        total_floor,
    }
}

/// Flood-fill to identify connected regions
fn identify_regions(map: &Map, min_area_ratio: f32) -> Vec<Region> {
    let mut visited = vec![vec![false; MAP_HEIGHT]; MAP_WIDTH];
    let mut regions = Vec::new();
    let mut total_floor = 0usize;
    
    // First pass: count total floor
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            if is_walkable(map, x as i32, y as i32) {
                total_floor += 1;
            }
        }
    }
    
    // Second pass: identify regions
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            if !visited[x][y] && is_walkable(map, x as i32, y as i32) {
                let tiles = flood_fill(map, x as i32, y as i32, &mut visited);
                if !tiles.is_empty() {
                    let size = tiles.len();
                    let weight = size as f32 / total_floor as f32;
                    
                    // Filter by minimum area
                    if weight >= min_area_ratio {
                        let perimeter = extract_perimeter(&tiles, map);
                        let centroid = compute_centroid(&tiles);
                        
                        regions.push(Region {
                            index: regions.len(),
                            tiles,
                            perimeter,
                            centroid,
                            size,
                            weight,
                        });
                    }
                }
            }
        }
    }
    
    // Update indices after filtering
    for (i, region) in regions.iter_mut().enumerate() {
        region.index = i;
    }
    
    regions
}

/// Flood-fill from a starting point
fn flood_fill(map: &Map, start_x: i32, start_y: i32, visited: &mut Vec<Vec<bool>>) -> Vec<(i32, i32)> {
    let mut tiles = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));
    
    while let Some((x, y)) = queue.pop_front() {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            continue;
        }
        let (ux, uy) = (x as usize, y as usize);
        if visited[ux][uy] || !is_walkable(map, x, y) {
            continue;
        }
        
        visited[ux][uy] = true;
        tiles.push((x, y));
        
        queue.push_back((x + 1, y));
        queue.push_back((x - 1, y));
        queue.push_back((x, y + 1));
        queue.push_back((x, y - 1));
    }
    
    tiles
}

/// Extract perimeter tiles (ordered)
fn extract_perimeter(tiles: &[(i32, i32)], _map: &Map) -> Vec<(i32, i32)> {
    let tile_set: HashSet<_> = tiles.iter().copied().collect();
    let mut perimeter: Vec<(i32, i32)> = tiles.iter()
        .filter(|&&(x, y)| {
            // Is on perimeter if adjacent to non-region tile
            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                !tile_set.contains(&(x + dx, y + dy))
            })
        })
        .copied()
        .collect();
    
    // Sort by angle from centroid for ordered traversal
    if !perimeter.is_empty() {
        let centroid = compute_centroid(tiles);
        perimeter.sort_by(|a, b| {
            let angle_a = (a.1 as f32 - centroid.1).atan2(a.0 as f32 - centroid.0);
            let angle_b = (b.1 as f32 - centroid.1).atan2(b.0 as f32 - centroid.0);
            angle_a.partial_cmp(&angle_b).unwrap()
        });
    }
    
    perimeter
}

/// Compute centroid of tiles
fn compute_centroid(tiles: &[(i32, i32)]) -> (f32, f32) {
    if tiles.is_empty() {
        return (0.0, 0.0);
    }
    let sum_x: i32 = tiles.iter().map(|t| t.0).sum();
    let sum_y: i32 = tiles.iter().map(|t| t.1).sum();
    let n = tiles.len() as f32;
    (sum_x as f32 / n, sum_y as f32 / n)
}

fn is_walkable(map: &Map, x: i32, y: i32) -> bool {
    map.get(x, y).map(|t| t.walkable()).unwrap_or(false)
}

fn is_wall(map: &Map, x: i32, y: i32) -> bool {
    map.get(x, y).map(|t| matches!(t, Tile::Wall { .. })).unwrap_or(true)
}


// ============================================================================
// Step 4: Edge Cost Computation
// ============================================================================

/// Compute edges between all region pairs
fn compute_all_edges(regions: &[Region], map: &Map, params: &GSBParams) -> Vec<TunnelEdge> {
    let mut edges = Vec::new();
    
    for i in 0..regions.len() {
        for j in (i + 1)..regions.len() {
            let r1 = &regions[i];
            let r2 = &regions[j];
            
            // Distance filter
            let dist = centroid_distance(r1, r2);
            if dist > params.max_edge_distance {
                continue;
            }
            
            // Compute edge using centroid line method
            if let Some(edge) = compute_edge(r1, r2, map) {
                edges.push(edge);
            }
        }
    }
    
    edges
}

/// Compute edge between two regions using centroid line
fn compute_edge(r1: &Region, r2: &Region, map: &Map) -> Option<TunnelEdge> {
    // Find exit points along centroid line
    let exit_a = find_exit_point(&r1.perimeter, r1.centroid, r2.centroid)?;
    let exit_b = find_exit_point(&r2.perimeter, r2.centroid, r1.centroid)?;
    
    // Count walls using Bresenham
    let cost = count_walls_bresenham(map, exit_a, exit_b);
    
    Some(TunnelEdge {
        region_a: r1.index,
        region_b: r2.index,
        exit_a,
        exit_b,
        cost,
    })
}

/// Find exit point on perimeter closest to target direction
fn find_exit_point(perimeter: &[(i32, i32)], from: (f32, f32), to: (f32, f32)) -> Option<(i32, i32)> {
    if perimeter.is_empty() {
        return None;
    }
    
    let dir = (to.0 - from.0, to.1 - from.1);
    
    perimeter.iter()
        .max_by(|&&a, &&b| {
            let da = (a.0 as f32 - from.0, a.1 as f32 - from.1);
            let db = (b.0 as f32 - from.0, b.1 as f32 - from.1);
            let dot_a = da.0 * dir.0 + da.1 * dir.1;
            let dot_b = db.0 * dir.0 + db.1 * dir.1;
            dot_a.partial_cmp(&dot_b).unwrap()
        })
        .copied()
}

/// Bresenham line algorithm to count wall tiles
fn count_walls_bresenham(map: &Map, from: (i32, i32), to: (i32, i32)) -> usize {
    let mut count = 0;
    for (x, y) in bresenham_line(from, to) {
        if is_wall(map, x, y) {
            count += 1;
        }
    }
    count
}

/// Bresenham line iterator
fn bresenham_line(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;
    
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        points.push((x0, y0));
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
    
    points
}

fn centroid_distance(r1: &Region, r2: &Region) -> f32 {
    let dx = r1.centroid.0 - r2.centroid.0;
    let dy = r1.centroid.1 - r2.centroid.1;
    (dx * dx + dy * dy).sqrt()
}

// ============================================================================
// Step 5: Edge Pruning
// ============================================================================

/// Apply pruning pipeline to edges
fn prune_edges(edges: &mut Vec<TunnelEdge>, regions: &[Region], params: &GSBParams) {
    // 5.1: Delaunay filter (simplified: keep edges to nearest neighbors)
    if params.use_delaunay {
        prune_delaunay(edges, regions);
    }
    
    // 5.2: Angular sector pruning
    prune_angular(edges, regions, params.angular_sectors);
    
    // 5.3: Occlusion pruning
    prune_occlusion(edges, params.occlusion_factor);
}

/// Simplified Delaunay: keep only edges to k nearest neighbors
fn prune_delaunay(edges: &mut Vec<TunnelEdge>, regions: &[Region]) {
    const MAX_NEIGHBORS: usize = 5;
    
    let mut keep = HashSet::new();
    
    for region in regions {
        // Find nearest neighbors by centroid distance
        let mut neighbors: Vec<_> = regions.iter()
            .filter(|r| r.index != region.index)
            .map(|r| (r.index, centroid_distance(region, r)))
            .collect();
        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        for (neighbor_idx, _) in neighbors.into_iter().take(MAX_NEIGHBORS) {
            let key = if region.index < neighbor_idx {
                (region.index, neighbor_idx)
            } else {
                (neighbor_idx, region.index)
            };
            keep.insert(key);
        }
    }
    
    edges.retain(|e| {
        let key = if e.region_a < e.region_b {
            (e.region_a, e.region_b)
        } else {
            (e.region_b, e.region_a)
        };
        keep.contains(&key)
    });
}

/// Keep only shortest edge per angular sector from each region
fn prune_angular(edges: &mut Vec<TunnelEdge>, regions: &[Region], sectors: usize) {
    use std::f32::consts::PI;
    
    let mut best_per_sector: HashMap<(usize, usize), TunnelEdge> = HashMap::new();
    
    for edge in edges.iter() {
        let r1 = &regions[edge.region_a];
        let r2 = &regions[edge.region_b];
        
        // Compute sector from r1's perspective
        let angle = (r2.centroid.1 - r1.centroid.1).atan2(r2.centroid.0 - r1.centroid.0);
        let sector = ((angle + PI) / (2.0 * PI) * sectors as f32) as usize % sectors;
        
        let key = (edge.region_a, sector);
        if let Some(existing) = best_per_sector.get(&key) {
            if edge.cost < existing.cost {
                best_per_sector.insert(key, edge.clone());
            }
        } else {
            best_per_sector.insert(key, edge.clone());
        }
    }
    
    let kept: HashSet<_> = best_per_sector.values()
        .map(|e| (e.region_a, e.region_b))
        .collect();
    
    edges.retain(|e| kept.contains(&(e.region_a, e.region_b)) || kept.contains(&(e.region_b, e.region_a)));
}

/// Remove edges with better indirect paths
fn prune_occlusion(edges: &mut Vec<TunnelEdge>, occlusion_factor: f32) {
    // Build cost lookup
    let mut costs: HashMap<(usize, usize), usize> = HashMap::new();
    for edge in edges.iter() {
        let key = if edge.region_a < edge.region_b {
            (edge.region_a, edge.region_b)
        } else {
            (edge.region_b, edge.region_a)
        };
        costs.insert(key, edge.cost);
    }
    
    let mut to_remove = HashSet::new();
    
    for edge in edges.iter() {
        let direct_cost = edge.cost as f32;
        let (a, b) = (edge.region_a, edge.region_b);
        
        // Check for better indirect paths through any intermediate region
        for &(k1, k2) in costs.keys() {
            if k1 == a && k2 != b {
                // Path a -> k2 -> b
                let key_b = if k2 < b { (k2, b) } else { (b, k2) };
                if let (Some(&c1), Some(&c2)) = (costs.get(&(k1, k2)), costs.get(&key_b)) {
                    let indirect_cost = (c1 + c2) as f32;
                    if indirect_cost < direct_cost * occlusion_factor {
                        to_remove.insert((a.min(b), a.max(b)));
                    }
                }
            }
        }
    }
    
    edges.retain(|e| {
        let key = (e.region_a.min(e.region_b), e.region_a.max(e.region_b));
        !to_remove.contains(&key)
    });
}


// ============================================================================
// Step 6: Edge Optimization (PGD and FRR)
// ============================================================================

/// Optimize edge costs using PGD and/or FRR
fn optimize_edges(edges: &mut Vec<TunnelEdge>, regions: &[Region], map: &Map, params: &GSBParams) {
    for edge in edges.iter_mut() {
        let r1 = &regions[edge.region_a];
        let r2 = &regions[edge.region_b];
        
        // Try FRR first (global search)
        if params.use_frr {
            if let Some((exit_a, exit_b, cost)) = frustum_ray_refinement(r1, r2, map, params) {
                edge.exit_a = exit_a;
                edge.exit_b = exit_b;
                edge.cost = cost;
            }
        }
        
        // Then PGD (local refinement)
        if params.use_pgd {
            if let Some((exit_a, exit_b, cost)) = perimeter_gradient_descent(
                r1, r2, edge.exit_a, edge.exit_b, map, params
            ) {
                edge.exit_a = exit_a;
                edge.exit_b = exit_b;
                edge.cost = cost;
            }
        }
    }
}

/// Perimeter Gradient Descent optimization
fn perimeter_gradient_descent(
    r1: &Region,
    r2: &Region,
    initial_a: (i32, i32),
    initial_b: (i32, i32),
    map: &Map,
    params: &GSBParams,
) -> Option<((i32, i32), (i32, i32), usize)> {
    if r1.perimeter.is_empty() || r2.perimeter.is_empty() {
        return None;
    }
    
    // Find initial indices
    let mut idx_a = r1.perimeter.iter().position(|&p| p == initial_a)
        .unwrap_or(0);
    let mut idx_b = r2.perimeter.iter().position(|&p| p == initial_b)
        .unwrap_or(0);
    
    let mut best_cost = count_walls_bresenham(map, r1.perimeter[idx_a], r2.perimeter[idx_b]);
    
    let deltas: [(i32, i32); 6] = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1)];
    
    for _ in 0..params.pgd_max_iter {
        let mut improved = false;
        
        for &(da, db) in &deltas {
            // Check skew limit
            if (da - db).unsigned_abs() as usize > params.pgd_skew {
                continue;
            }
            
            let new_a = (idx_a as i32 + da).rem_euclid(r1.perimeter.len() as i32) as usize;
            let new_b = (idx_b as i32 + db).rem_euclid(r2.perimeter.len() as i32) as usize;
            
            let cost = count_walls_bresenham(map, r1.perimeter[new_a], r2.perimeter[new_b]);
            
            if cost < best_cost {
                best_cost = cost;
                idx_a = new_a;
                idx_b = new_b;
                improved = true;
                break;
            }
        }
        
        if !improved {
            break;
        }
    }
    
    Some((r1.perimeter[idx_a], r2.perimeter[idx_b], best_cost))
}

/// Frustum Ray Refinement optimization
fn frustum_ray_refinement(
    r1: &Region,
    r2: &Region,
    map: &Map,
    params: &GSBParams,
) -> Option<((i32, i32), (i32, i32), usize)> {
    use std::f32::consts::PI;
    
    if r1.perimeter.is_empty() || r2.perimeter.is_empty() {
        return None;
    }
    
    let theta_max = params.frr_theta_max * PI / 180.0;
    
    // Axis direction
    let axis = (
        r2.centroid.0 - r1.centroid.0,
        r2.centroid.1 - r1.centroid.1,
    );
    let axis_len = (axis.0 * axis.0 + axis.1 * axis.1).sqrt();
    if axis_len < 0.001 {
        return None;
    }
    let axis_norm = (axis.0 / axis_len, axis.1 / axis_len);
    
    // Filter perimeter points by visibility cone
    let visible_a: Vec<_> = r1.perimeter.iter()
        .filter(|&&(x, y)| {
            let to_point = (x as f32 - r1.centroid.0, y as f32 - r1.centroid.1);
            let len = (to_point.0 * to_point.0 + to_point.1 * to_point.1).sqrt();
            if len < 0.001 { return false; }
            let dot = (to_point.0 * axis_norm.0 + to_point.1 * axis_norm.1) / len;
            dot.acos() <= theta_max
        })
        .copied()
        .collect();
    
    let visible_b: Vec<_> = r2.perimeter.iter()
        .filter(|&&(x, y)| {
            let to_point = (r2.centroid.0 - x as f32, r2.centroid.1 - y as f32);
            let len = (to_point.0 * to_point.0 + to_point.1 * to_point.1).sqrt();
            if len < 0.001 { return false; }
            let dot = (to_point.0 * axis_norm.0 + to_point.1 * axis_norm.1) / len;
            dot.acos() <= theta_max
        })
        .copied()
        .collect();
    
    if visible_a.is_empty() || visible_b.is_empty() {
        return None;
    }
    
    // Hierarchical refinement
    frr_refine(&visible_a, &visible_b, map, params.frr_depth, 4)
}

/// Recursive FRR refinement
fn frr_refine(
    candidates_a: &[(i32, i32)],
    candidates_b: &[(i32, i32)],
    map: &Map,
    depth: usize,
    bins: usize,
) -> Option<((i32, i32), (i32, i32), usize)> {
    if candidates_a.is_empty() || candidates_b.is_empty() {
        return None;
    }
    
    if depth == 0 || candidates_a.len() <= bins || candidates_b.len() <= bins {
        // Base case: find best pair
        let mut best: Option<((i32, i32), (i32, i32), usize)> = None;
        for &a in candidates_a {
            for &b in candidates_b {
                let cost = count_walls_bresenham(map, a, b);
                if best.is_none() || cost < best.unwrap().2 {
                    best = Some((a, b, cost));
                }
            }
        }
        return best;
    }
    
    // Partition into bins and sample
    let chunk_a = (candidates_a.len() + bins - 1) / bins;
    let chunk_b = (candidates_b.len() + bins - 1) / bins;
    
    let mut best_bin = (0, 0, usize::MAX);
    
    for i in 0..bins.min(candidates_a.len()) {
        for j in 0..bins.min(candidates_b.len()) {
            let start_a = i * chunk_a;
            let start_b = j * chunk_b;
            
            if start_a < candidates_a.len() && start_b < candidates_b.len() {
                let sample_a = candidates_a[start_a];
                let sample_b = candidates_b[start_b];
                let cost = count_walls_bresenham(map, sample_a, sample_b);
                
                if cost < best_bin.2 {
                    best_bin = (i, j, cost);
                }
            }
        }
    }
    
    // Recurse into best bin
    let start_a = best_bin.0 * chunk_a;
    let end_a = ((best_bin.0 + 1) * chunk_a).min(candidates_a.len());
    let start_b = best_bin.1 * chunk_b;
    let end_b = ((best_bin.1 + 1) * chunk_b).min(candidates_b.len());
    
    frr_refine(&candidates_a[start_a..end_a], &candidates_b[start_b..end_b], map, depth - 1, bins)
}


// ============================================================================
// Step 7: Graph Selection (Greedy)
// ============================================================================

/// Select optimal tunnels using greedy algorithm
fn select_tunnels(
    regions: &[Region],
    edges: &[TunnelEdge],
    spawn_region: usize,
    threshold: f32,
) -> Vec<TunnelEdge> {
    let mut selected_regions: HashSet<usize> = HashSet::new();
    selected_regions.insert(spawn_region);
    
    let mut selected_edges: Vec<TunnelEdge> = Vec::new();
    let mut coverage = regions.get(spawn_region).map(|r| r.weight).unwrap_or(0.0);
    
    // Build edge lookup by region
    let mut edges_by_region: HashMap<usize, Vec<&TunnelEdge>> = HashMap::new();
    for edge in edges {
        edges_by_region.entry(edge.region_a).or_default().push(edge);
        edges_by_region.entry(edge.region_b).or_default().push(edge);
    }
    
    while coverage < threshold {
        // Find best edge to add (highest efficiency: weight / cost)
        let mut best: Option<(&TunnelEdge, usize, f32)> = None;
        
        for &region_idx in &selected_regions {
            if let Some(region_edges) = edges_by_region.get(&region_idx) {
                for edge in region_edges {
                    let other = if edge.region_a == region_idx {
                        edge.region_b
                    } else {
                        edge.region_a
                    };
                    
                    if selected_regions.contains(&other) {
                        continue;
                    }
                    
                    let weight = regions.get(other).map(|r| r.weight).unwrap_or(0.0);
                    let efficiency = if edge.cost > 0 {
                        weight / edge.cost as f32
                    } else {
                        weight * 1000.0 // Free connection
                    };
                    
                    if best.is_none() || efficiency > best.unwrap().2 {
                        best = Some((edge, other, efficiency));
                    }
                }
            }
        }
        
        match best {
            Some((edge, new_region, _)) => {
                selected_edges.push(edge.clone());
                selected_regions.insert(new_region);
                coverage += regions.get(new_region).map(|r| r.weight).unwrap_or(0.0);
            }
            None => break, // No more edges available
        }
    }
    
    selected_edges
}

// ============================================================================
// Step 8: Tunnel Carving
// ============================================================================

/// Carve a tunnel between two points
fn carve_tunnel(map: &mut Map, edge: &TunnelEdge, width: usize, _rng: &mut ChaCha8Rng) {
    let points = bresenham_line(edge.exit_a, edge.exit_b);
    
    let half_width = width as i32 / 2;
    
    for (x, y) in points {
        // Carve with width
        for dx in -half_width..=half_width {
            for dy in -half_width..=half_width {
                let nx = x + dx;
                let ny = y + dy;
                
                if let Some(idx) = map.pos_to_idx(nx, ny) {
                    if matches!(map.tiles[idx], Tile::Wall { .. }) {
                        map.tiles[idx] = Tile::Floor;
                    }
                }
            }
        }
    }
}

// ============================================================================
// Multi-Terminal Variant
// ============================================================================

/// Connect multiple required regions (Steiner tree variant)
pub fn ensure_multi_terminal_connectivity(
    map: &mut Map,
    required_points: &[(i32, i32)],
    params: &GSBParams,
    rng: &mut ChaCha8Rng,
) -> Vec<TunnelEdge> {
    if required_points.is_empty() {
        return Vec::new();
    }
    
    let analysis = analyze_connectivity(map, required_points[0], params);
    
    // Find regions containing required points
    let required_regions: HashSet<usize> = required_points.iter()
        .filter_map(|&point| {
            analysis.regions.iter()
                .position(|r| r.tiles.contains(&point))
        })
        .collect();
    
    if required_regions.len() <= 1 {
        // All required points already connected
        return Vec::new();
    }
    
    // Compute edges
    let mut edges = compute_all_edges(&analysis.regions, map, params);
    prune_edges(&mut edges, &analysis.regions, params);
    
    if params.use_pgd || params.use_frr {
        optimize_edges(&mut edges, &analysis.regions, map, params);
    }
    
    // Phase 1: Connect required regions (simplified Steiner)
    let mut selected = connect_required_regions(&analysis.regions, &edges, &required_regions);
    
    // Phase 2: Expand for coverage if needed
    let connected: HashSet<usize> = selected.iter()
        .flat_map(|e| [e.region_a, e.region_b])
        .collect();
    
    let coverage: f32 = connected.iter()
        .filter_map(|&idx| analysis.regions.get(idx))
        .map(|r| r.weight)
        .sum();
    
    if coverage < params.connectivity_threshold {
        let spawn_region = *required_regions.iter().next().unwrap_or(&0);
        let additional = select_tunnels_from_connected(
            &analysis.regions,
            &edges,
            &connected,
            spawn_region,
            params.connectivity_threshold,
            coverage,
        );
        selected.extend(additional);
    }
    
    // Carve tunnels
    for edge in &selected {
        carve_tunnel(map, edge, params.tunnel_width, rng);
    }
    
    selected
}

/// Connect required regions using greedy Steiner approach
fn connect_required_regions(
    _regions: &[Region],
    edges: &[TunnelEdge],
    required: &HashSet<usize>,
) -> Vec<TunnelEdge> {
    if required.len() <= 1 {
        return Vec::new();
    }
    
    // Union-Find for component tracking
    let mut parent: HashMap<usize, usize> = required.iter().map(|&r| (r, r)).collect();
    
    fn find(parent: &mut HashMap<usize, usize>, x: usize) -> usize {
        if parent.get(&x) != Some(&x) {
            let p = *parent.get(&x).unwrap_or(&x);
            let root = find(parent, p);
            parent.insert(x, root);
            root
        } else {
            x
        }
    }
    
    fn union(parent: &mut HashMap<usize, usize>, a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent.insert(ra, rb);
        }
    }
    
    // Sort edges by cost
    let mut sorted_edges: Vec<_> = edges.iter().collect();
    sorted_edges.sort_by_key(|e| e.cost);
    
    let mut selected = Vec::new();
    
    for edge in sorted_edges {
        let in_a = required.contains(&edge.region_a);
        let in_b = required.contains(&edge.region_b);
        
        // Only consider edges that connect to required regions
        if !in_a && !in_b {
            continue;
        }
        
        // Add intermediate regions to parent map
        if !parent.contains_key(&edge.region_a) {
            parent.insert(edge.region_a, edge.region_a);
        }
        if !parent.contains_key(&edge.region_b) {
            parent.insert(edge.region_b, edge.region_b);
        }
        
        let root_a = find(&mut parent, edge.region_a);
        let root_b = find(&mut parent, edge.region_b);
        
        if root_a != root_b {
            union(&mut parent, edge.region_a, edge.region_b);
            selected.push(edge.clone());
            
            // Check if all required are connected
            let roots: HashSet<_> = required.iter()
                .map(|&r| find(&mut parent, r))
                .collect();
            if roots.len() == 1 {
                break;
            }
        }
    }
    
    selected
}

/// Continue selecting tunnels from already-connected set
fn select_tunnels_from_connected(
    regions: &[Region],
    edges: &[TunnelEdge],
    connected: &HashSet<usize>,
    _spawn_region: usize,
    threshold: f32,
    initial_coverage: f32,
) -> Vec<TunnelEdge> {
    let mut selected_regions = connected.clone();
    let mut selected_edges = Vec::new();
    let mut coverage = initial_coverage;
    
    while coverage < threshold {
        let mut best: Option<(&TunnelEdge, usize, f32)> = None;
        
        for edge in edges {
            let (in_a, in_b) = (
                selected_regions.contains(&edge.region_a),
                selected_regions.contains(&edge.region_b),
            );
            
            if in_a == in_b {
                continue; // Both in or both out
            }
            
            let other = if in_a { edge.region_b } else { edge.region_a };
            let weight = regions.get(other).map(|r| r.weight).unwrap_or(0.0);
            let efficiency = if edge.cost > 0 {
                weight / edge.cost as f32
            } else {
                weight * 1000.0
            };
            
            if best.is_none() || efficiency > best.unwrap().2 {
                best = Some((edge, other, efficiency));
            }
        }
        
        match best {
            Some((edge, new_region, _)) => {
                selected_edges.push(edge.clone());
                selected_regions.insert(new_region);
                coverage += regions.get(new_region).map(|r| r.weight).unwrap_or(0.0);
            }
            None => break,
        }
    }
    
    selected_edges
}

// ============================================================================
// Public API
// ============================================================================

/// Quick connectivity check without modification
pub fn check_connectivity(map: &Map, spawn: (i32, i32)) -> f32 {
    let params = GSBParams::default();
    let analysis = analyze_connectivity(map, spawn, &params);
    analysis.spawn_coverage
}

/// Get detailed connectivity analysis
pub fn get_connectivity_analysis(map: &Map, spawn: (i32, i32)) -> ConnectivityAnalysis {
    let params = GSBParams::default();
    analyze_connectivity(map, spawn, &params)
}
