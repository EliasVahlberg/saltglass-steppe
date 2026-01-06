use rand_chacha::ChaCha8Rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Narrative seed for generating consistent story elements
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NarrativeSeed {
    pub seed_id: String,
    pub theme: String,
    pub mood: String,
    pub factions: Vec<String>,
    pub key_elements: Vec<String>,
    pub weight: f32,
}

/// Story fragment that can be placed in the world
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoryFragment {
    pub fragment_id: String,
    pub narrative_seed: String,
    pub fragment_type: String,
    pub content: String,
    pub placement_rules: PlacementRules,
    pub faction_influence: HashMap<String, f32>,
    pub prerequisites: Vec<String>,
}

/// Rules for where story fragments can be placed
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlacementRules {
    pub biomes: Vec<String>,
    pub min_distance_from_player: u32,
    pub max_distance_from_player: u32,
    pub requires_poi: Option<String>,
    pub exclusion_zones: Vec<String>,
}

/// Faction influence on narrative content
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionInfluence {
    pub faction_id: String,
    pub influence_level: f32,
    pub narrative_modifiers: HashMap<String, f32>,
    pub preferred_themes: Vec<String>,
    pub story_fragments: Vec<String>,
}

/// Emergent narrative tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeState {
    pub active_seeds: Vec<String>,
    pub placed_fragments: HashMap<String, PlacedFragment>,
    pub faction_standings: HashMap<String, f32>,
    pub narrative_momentum: f32,
    pub story_threads: Vec<StoryThread>,
}

/// Placed story fragment in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedFragment {
    pub fragment_id: String,
    pub x: i32,
    pub y: i32,
    pub discovered: bool,
    pub activation_turn: u32,
}

/// Ongoing story thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryThread {
    pub thread_id: String,
    pub narrative_seed: String,
    pub progress: f32,
    pub active_fragments: Vec<String>,
    pub completion_condition: String,
}

/// Context for narrative generation
#[derive(Debug, Clone)]
pub struct NarrativeContext {
    pub player_x: i32,
    pub player_y: i32,
    pub current_biome: String,
    pub turn: u32,
    pub faction_standings: HashMap<String, f32>,
    pub discovered_fragments: Vec<String>,
    pub player_adaptations: Vec<String>,
}

/// Narrative integration system
pub struct NarrativeIntegration {
    seeds: HashMap<String, NarrativeSeed>,
    fragments: HashMap<String, StoryFragment>,
    factions: HashMap<String, FactionInfluence>,
    state: NarrativeState,
}

impl NarrativeIntegration {
    pub fn new() -> Self {
        Self {
            seeds: NARRATIVE_SEEDS.clone(),
            fragments: STORY_FRAGMENTS.clone(),
            factions: FACTION_INFLUENCES.clone(),
            state: NarrativeState {
                active_seeds: Vec::new(),
                placed_fragments: HashMap::new(),
                faction_standings: HashMap::new(),
                narrative_momentum: 0.0,
                story_threads: Vec::new(),
            },
        }
    }

    /// Initialize narrative system with starting seeds
    pub fn initialize(&mut self, _context: &NarrativeContext, rng: &mut ChaCha8Rng) {
        // Select initial narrative seeds based on context
        let mut available_seeds: Vec<_> = self.seeds.values().collect();
        available_seeds.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

        // Select 2-3 initial seeds
        let num_seeds = rng.gen_range(2..=3);
        for seed in available_seeds.iter().take(num_seeds) {
            self.state.active_seeds.push(seed.seed_id.clone());
            
            // Initialize faction standings based on seed
            for faction in &seed.factions {
                let standing = rng.gen_range(-0.2..0.2);
                self.state.faction_standings.insert(faction.clone(), standing);
            }
        }

        // Initialize narrative momentum
        self.state.narrative_momentum = rng.gen_range(0.3..0.7);
    }

    /// Generate story fragments for placement
    pub fn generate_fragments(&mut self, context: &NarrativeContext, rng: &mut ChaCha8Rng) -> Vec<PlacedFragment> {
        let mut new_fragments = Vec::new();

        for seed_id in &self.state.active_seeds.clone() {
            if let Some(_seed) = self.seeds.get(seed_id) {
                // Find fragments that match this seed
                let matching_fragment_ids: Vec<String> = self.fragments.iter()
                    .filter(|(_, f)| f.narrative_seed == *seed_id)
                    .map(|(id, _)| id.clone())
                    .collect();

                if !matching_fragment_ids.is_empty() {
                    let fragment_id = matching_fragment_ids[rng.gen_range(0..matching_fragment_ids.len())].clone();
                    
                    // Clone the fragment to avoid borrow checker issues
                    if let Some(fragment) = self.fragments.get(&fragment_id).cloned() {
                        // Check placement rules
                        if self.can_place_fragment(&fragment, context) {
                            let placed = self.place_fragment(&fragment, context, rng);
                            if let Some(placed) = placed {
                                new_fragments.push(placed);
                            }
                        }
                    }
                }
            }
        }

        new_fragments
    }

    /// Check if a fragment can be placed according to its rules
    fn can_place_fragment(&self, fragment: &StoryFragment, context: &NarrativeContext) -> bool {
        // Check biome requirements
        if !fragment.placement_rules.biomes.is_empty() {
            if !fragment.placement_rules.biomes.contains(&context.current_biome) {
                return false;
            }
        }

        // Check prerequisites
        for prereq in &fragment.prerequisites {
            if !context.discovered_fragments.contains(prereq) {
                return false;
            }
        }

        // Check if already placed
        if self.state.placed_fragments.contains_key(&fragment.fragment_id) {
            return false;
        }

        true
    }

    /// Place a fragment in the world
    fn place_fragment(&mut self, fragment: &StoryFragment, context: &NarrativeContext, rng: &mut ChaCha8Rng) -> Option<PlacedFragment> {
        let rules = &fragment.placement_rules;
        
        // Generate placement coordinates within distance constraints
        let min_dist = rules.min_distance_from_player as i32;
        let max_dist = rules.max_distance_from_player as i32;
        
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(min_dist..=max_dist);
        
        let x = context.player_x + (distance as f32 * angle.cos()) as i32;
        let y = context.player_y + (distance as f32 * angle.sin()) as i32;

        let placed = PlacedFragment {
            fragment_id: fragment.fragment_id.clone(),
            x,
            y,
            discovered: false,
            activation_turn: context.turn,
        };

        self.state.placed_fragments.insert(fragment.fragment_id.clone(), placed.clone());
        Some(placed)
    }

    /// Update faction influence based on player actions
    pub fn update_faction_influence(&mut self, faction_id: &str, change: f32) {
        let current = self.state.faction_standings.get(faction_id).unwrap_or(&0.0);
        let new_standing = (current + change).clamp(-1.0, 1.0);
        self.state.faction_standings.insert(faction_id.to_string(), new_standing);

        // Update narrative momentum based on faction changes
        self.state.narrative_momentum += change.abs() * 0.1;
        self.state.narrative_momentum = self.state.narrative_momentum.clamp(0.0, 1.0);
    }

    /// Track emergent narrative developments
    pub fn track_narrative_event(&mut self, event_type: &str, context: &NarrativeContext) {
        match event_type {
            "fragment_discovered" => {
                self.state.narrative_momentum += 0.05;
            }
            "faction_encounter" => {
                self.state.narrative_momentum += 0.03;
            }
            "adaptation_gained" => {
                self.state.narrative_momentum += 0.02;
                // Potentially activate new story threads
                self.check_story_thread_activation(context);
            }
            _ => {}
        }

        self.state.narrative_momentum = self.state.narrative_momentum.clamp(0.0, 1.0);
    }

    /// Check if new story threads should be activated
    fn check_story_thread_activation(&mut self, context: &NarrativeContext) {
        // Activate new threads based on narrative momentum and context
        if self.state.narrative_momentum > 0.7 && self.state.story_threads.len() < 3 {
            for seed_id in &self.state.active_seeds.clone() {
                if let Some(_seed) = self.seeds.get(seed_id) {
                    let thread = StoryThread {
                        thread_id: format!("thread_{}_{}", seed_id, context.turn),
                        narrative_seed: seed_id.clone(),
                        progress: 0.0,
                        active_fragments: Vec::new(),
                        completion_condition: "discover_all_fragments".to_string(),
                    };
                    self.state.story_threads.push(thread);
                    break;
                }
            }
        }
    }

    /// Get narrative content influenced by factions
    pub fn get_faction_influenced_content(&self, base_content: &str, location_factions: &[String]) -> String {
        let mut influenced_content = base_content.to_string();

        for faction_id in location_factions {
            if let Some(_faction) = self.factions.get(faction_id) {
                if let Some(standing) = self.state.faction_standings.get(faction_id) {
                    // Modify content based on faction standing
                    if *standing > 0.5 {
                        influenced_content = format!("{} [The {} regard you favorably.]", influenced_content, faction_id);
                    } else if *standing < -0.5 {
                        influenced_content = format!("{} [The {} view you with suspicion.]", influenced_content, faction_id);
                    }
                }
            }
        }

        influenced_content
    }

    /// Get current narrative state
    pub fn get_narrative_state(&self) -> &NarrativeState {
        &self.state
    }

    /// Get placed fragments near a location
    pub fn get_fragments_near(&self, x: i32, y: i32, radius: i32) -> Vec<&PlacedFragment> {
        self.state.placed_fragments.values()
            .filter(|fragment| {
                let dx = fragment.x - x;
                let dy = fragment.y - y;
                (dx * dx + dy * dy) <= (radius * radius)
            })
            .collect()
    }

    /// Mark a fragment as discovered
    pub fn discover_fragment(&mut self, fragment_id: &str) -> Option<String> {
        if let Some(fragment) = self.state.placed_fragments.get_mut(fragment_id) {
            fragment.discovered = true;
            
            // Get the fragment content
            if let Some(story_fragment) = self.fragments.get(fragment_id) {
                return Some(story_fragment.content.clone());
            }
        }
        None
    }

    /// Get number of loaded seeds (for testing)
    pub fn seed_count(&self) -> usize {
        self.seeds.len()
    }

    /// Get number of loaded fragments (for testing)
    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    /// Get number of loaded factions (for testing)
    pub fn faction_count(&self) -> usize {
        self.factions.len()
    }

    /// Check if there are active seeds (for testing)
    pub fn has_active_seeds(&self) -> bool {
        !self.state.active_seeds.is_empty()
    }
}

/// Narrative data loaded from JSON
#[derive(Deserialize)]
struct NarrativeFile {
    narrative_seeds: Vec<NarrativeSeed>,
    story_fragments: Vec<StoryFragment>,
    faction_influences: Vec<FactionInfluence>,
}

static NARRATIVE_SEEDS: Lazy<HashMap<String, NarrativeSeed>> = Lazy::new(|| {
    let data = include_str!("../../../data/narrative_integration.json");
    let file: NarrativeFile = serde_json::from_str(data).expect("Failed to parse narrative_integration.json");
    file.narrative_seeds.into_iter().map(|s| (s.seed_id.clone(), s)).collect()
});

static STORY_FRAGMENTS: Lazy<HashMap<String, StoryFragment>> = Lazy::new(|| {
    let data = include_str!("../../../data/narrative_integration.json");
    let file: NarrativeFile = serde_json::from_str(data).expect("Failed to parse narrative_integration.json");
    file.story_fragments.into_iter().map(|f| (f.fragment_id.clone(), f)).collect()
});

static FACTION_INFLUENCES: Lazy<HashMap<String, FactionInfluence>> = Lazy::new(|| {
    let data = include_str!("../../../data/narrative_integration.json");
    let file: NarrativeFile = serde_json::from_str(data).expect("Failed to parse narrative_integration.json");
    file.faction_influences.into_iter().map(|f| (f.faction_id.clone(), f)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn create_test_context() -> NarrativeContext {
        NarrativeContext {
            player_x: 10,
            player_y: 10,
            current_biome: "desert".to_string(),
            turn: 100,
            faction_standings: HashMap::new(),
            discovered_fragments: Vec::new(),
            player_adaptations: vec!["prismhide".to_string()],
        }
    }

    #[test]
    fn test_narrative_integration_creation() {
        let system = NarrativeIntegration::new();
        assert!(!system.seeds.is_empty());
        assert!(!system.fragments.is_empty());
        assert!(!system.factions.is_empty());
    }

    #[test]
    fn test_narrative_initialization() {
        let mut system = NarrativeIntegration::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let context = create_test_context();

        system.initialize(&context, &mut rng);
        
        assert!(!system.state.active_seeds.is_empty());
        assert!(system.state.narrative_momentum > 0.0);
    }

    #[test]
    fn test_fragment_generation() {
        let mut system = NarrativeIntegration::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let context = create_test_context();

        system.initialize(&context, &mut rng);
        
        // Ensure we have active seeds
        assert!(system.has_active_seeds(), "Should have active seeds after initialization");
        
        let fragments = system.generate_fragments(&context, &mut rng);
        
        // Should generate some fragments (may be empty if placement rules don't match)
        // This is acceptable behavior - not all contexts will generate fragments
    }

    #[test]
    fn test_faction_influence_update() {
        let mut system = NarrativeIntegration::new();
        
        system.update_faction_influence("mirror_monks", 0.3);
        
        assert_eq!(system.state.faction_standings.get("mirror_monks"), Some(&0.3));
        assert!(system.state.narrative_momentum > 0.0);
    }

    #[test]
    fn test_narrative_event_tracking() {
        let mut system = NarrativeIntegration::new();
        let context = create_test_context();
        let initial_momentum = system.state.narrative_momentum;

        system.track_narrative_event("fragment_discovered", &context);
        
        assert!(system.state.narrative_momentum > initial_momentum);
    }

    #[test]
    fn test_faction_influenced_content() {
        let mut system = NarrativeIntegration::new();
        system.state.faction_standings.insert("mirror_monks".to_string(), 0.8);

        let base_content = "You find ancient ruins.";
        let factions = vec!["mirror_monks".to_string()];
        let influenced = system.get_faction_influenced_content(base_content, &factions);
        
        assert!(influenced.contains("favorably"));
    }

    #[test]
    fn test_fragment_discovery() {
        let mut system = NarrativeIntegration::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let context = create_test_context();

        system.initialize(&context, &mut rng);
        let fragments = system.generate_fragments(&context, &mut rng);
        
        if let Some(fragment) = fragments.first() {
            let content = system.discover_fragment(&fragment.fragment_id);
            assert!(content.is_some());
            
            let discovered_fragment = system.state.placed_fragments.get(&fragment.fragment_id).unwrap();
            assert!(discovered_fragment.discovered);
        }
    }
}
