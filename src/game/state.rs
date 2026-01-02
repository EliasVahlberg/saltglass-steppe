use bracket_pathfinding::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::{
    action::{action_cost, default_player_ap},
    adaptation::Adaptation,
    chest::Chest,
    enemy::Enemy,
    equipment::{EquipSlot, Equipment},
    event::GameEvent,
    fov::FieldOfView,
    item::{get_item_def, Item},
    lighting::{compute_lighting, LightMap, LightSource},

    map::{Map, Tile},
    map_features::MapFeatures,
    generation::{place_microstructures, PlacedMicroStructure},

    npc::Npc,
    quest::QuestLog,
    sanity::SanitySystem,
    generation::{weighted_pick_by_level_and_tier, get_biome_spawn_table, distribute_points_grid, generate_loot, StoryModel, EventType, NarrativeGenerator, NarrativeContext},
    storm::Storm,

    systems::movement::MovementSystem,
    tutorial::TutorialProgress,
    world_map::WorldMap,
    generation::{events::{EventSystem, EventContext}, narrative::{NarrativeIntegration}, BiomeSystem, Grammar, GrammarContext, TemplateLibrary, TemplateContext, ConstraintSystem, GenerationPipeline, GenerationConfig},
};

mod rng_serde {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct RngState([u8; 32]);

    pub fn serialize<S: Serializer>(rng: &ChaCha8Rng, s: S) -> Result<S::Ok, S::Error> {
        let bytes: [u8; 32] = rng.get_seed();
        RngState(bytes).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<ChaCha8Rng, D::Error> {
        let state = RngState::deserialize(d)?;
        Ok(ChaCha8Rng::from_seed(state.0))
    }
}

/// Message types for color-coded log display
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MsgType {
    #[default]
    System,
    Combat,
    Social,
    Loot,
    Status,
    Dialogue,
    Warning,
}

/// Game message with type for color-coding
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameMessage {
    pub text: String,
    pub msg_type: MsgType,
    pub turn: u32,
}

impl GameMessage {
    pub fn new(text: impl Into<String>, msg_type: MsgType, turn: u32) -> Self {
        Self { text: text.into(), msg_type, turn }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggeredEffect {
    pub effect: String,
    pub turns_remaining: u32,
}

fn default_ambient_light() -> u8 { 100 }
fn default_time_of_day() -> u8 { 8 } // Start at 8 AM

/// Weather conditions affecting visibility and lighting
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Weather {
    #[default]
    Clear,
    Dusty,      // Reduced visibility
    Sandstorm,  // Severely reduced visibility
}

impl Weather {
    pub fn visibility_modifier(&self) -> i32 {
        match self {
            Weather::Clear => 0,
            Weather::Dusty => -2,
            Weather::Sandstorm => -4,
        }
    }
    
    pub fn ambient_modifier(&self) -> i32 {
        match self {
            Weather::Clear => 0,
            Weather::Dusty => -20,
            Weather::Sandstorm => -50,
        }
    }
}

/// Decoy left by mirage_step adaptation
#[derive(Clone, Serialize, Deserialize)]
pub struct Decoy {
    pub x: i32,
    pub y: i32,
    pub turns_remaining: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub player_x: i32, pub player_y: i32, pub player_hp: i32, pub player_max_hp: i32,
    #[serde(default = "default_player_ap")]
    pub player_ap: i32,
    #[serde(default = "default_player_ap")]
    pub player_max_ap: i32,
    #[serde(default)]
    pub player_reflex: i32,
    #[serde(default)]
    pub player_armor: i32,
    #[serde(default)]
    pub player_xp: u32,
    #[serde(default)]
    pub player_level: u32,
    #[serde(default)]
    pub pending_stat_points: i32,
    /// Currency (salt scrip)
    #[serde(default)]
    pub salt_scrip: u32,
    /// Faction reputation system (-100 to +100 per faction)
    #[serde(default)]
    pub faction_reputation: HashMap<String, i32>,
    /// Track last damage dealt for reflection behaviors
    #[serde(default)]
    pub last_damage_dealt: u32,
    /// Completed rituals
    #[serde(default)]
    pub completed_rituals: Vec<super::ritual::CompletedRitual>,
    #[serde(default)]
    pub equipped_weapon: Option<String>,
    #[serde(default)]
    pub equipment: Equipment,
    #[serde(default)]
    pub status_effects: Vec<super::status::StatusEffect>,
    pub map: Map, pub enemies: Vec<Enemy>,
    pub npcs: Vec<Npc>,
    pub items: Vec<Item>,
    pub chests: Vec<Chest>,
    pub inventory: Vec<String>,
    pub visible: HashSet<usize>, pub revealed: HashSet<usize>,
    #[serde(skip)]
    pub player_fov: FieldOfView,
    #[serde(skip)]
    pub light_map: LightMap,
    #[serde(default = "default_ambient_light")]
    pub ambient_light: u8,
    pub messages: Vec<GameMessage>, pub turn: u32,
    #[serde(with = "rng_serde")]
    pub rng: ChaCha8Rng, pub storm: Storm,
    pub refraction: u32,
    pub adaptations: Vec<Adaptation>,
    #[serde(default)]
    pub adaptations_hidden_turns: u32,
    #[serde(default)]
    pub quest_log: QuestLog,
    #[serde(default)]
    pub triggered_effects: Vec<TriggeredEffect>,
    /// Decoys left by mirage_step adaptation
    #[serde(default)]
    pub decoys: Vec<Decoy>,
    #[serde(skip)]
    pub enemy_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub npc_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    pub item_positions: HashMap<(i32, i32), Vec<usize>>,
    #[serde(skip)]
    pub chest_positions: HashMap<(i32, i32), usize>,
    #[serde(skip)]
    spatial_dirty: bool,
    #[serde(skip)]
    pub event_queue: Vec<GameEvent>,
    #[serde(skip)]
    pub hit_flash_positions: Vec<(i32, i32, u32)>,
    #[serde(skip)]
    pub damage_numbers: Vec<DamageNumber>,
    #[serde(skip)]
    pub projectile_trails: Vec<ProjectileTrail>,
    #[serde(skip)]
    pub light_beams: Vec<LightBeam>,
    #[serde(skip)]
    pub mock_combat_hit: Option<bool>,
    #[serde(skip)]
    pub mock_combat_damage: Option<i32>,
    #[serde(skip)]
    pub pending_book_open: Option<String>,
    #[serde(skip)]
    pub meta: super::meta::MetaProgress,
    // World map for lazy tile generation
    #[serde(default)]
    pub world_map: Option<WorldMap>,
    #[serde(default)]
    pub world_x: usize,
    #[serde(default)]
    pub world_y: usize,
    /// Current layer: 0 = surface, negative = underground
    #[serde(default)]
    pub layer: i32,
    /// Time of day (0-23 hours)
    #[serde(default = "default_time_of_day")]
    pub time_of_day: u8,
    /// Current weather condition
    #[serde(default)]
    pub weather: Weather,
    /// Consecutive turns waited (for auto-rest)
    #[serde(default)]
    pub wait_counter: u32,
    /// Tutorial system progress tracking
    #[serde(default)]
    pub tutorial_progress: TutorialProgress,
    /// Advanced map features (hidden locations, safe routes, etc.)
    #[serde(default)]
    pub map_features: MapFeatures,
    /// Sanity/Mental health system
    #[serde(default)]
    pub sanity: SanitySystem,
    /// Psychic/Quantum Consciousness system
    #[serde(default)]
    pub psychic: super::psychic::PsychicState,
    /// Pending trade interface (for UI)
    #[serde(skip)]
    pub pending_trade: Option<String>,
    /// Animation frame counter for ambient tile animations
    #[serde(skip)]
    pub animation_frame: u32,
    /// Tiles changed by the last storm (for diff highlighting)
    #[serde(skip)]
    pub storm_changed_tiles: HashSet<usize>,
    /// Pending dialogue to show in UI (speaker, text)
    #[serde(skip)]
    pub pending_dialogue: Option<(String, String)>,
    // Debug flags
    #[serde(skip)]
    pub debug_god_view: bool,
    #[serde(skip)]
    pub debug_phase: bool,
    #[serde(skip)]
    pub debug_disable_glare: bool,
    /// Original seed for reproducibility
    #[serde(default)]
    pub seed: u64,
    /// Procedural narrative generator
    #[serde(skip)]
    pub narrative_generator: Option<NarrativeGenerator>,
    /// Generated world history
    #[serde(default)]
    pub world_history: Vec<String>,
    /// Persistent story model with characters and relationships
    #[serde(default)]
    pub story_model: Option<StoryModel>,
    /// Physical skills and abilities system
    #[serde(default)]
    pub skills: super::skills::SkillsState,
    /// Placed micro-structures on the current tile
    #[serde(default)]
    pub microstructures: Vec<PlacedMicroStructure>,
    /// Dynamic event system for procedural events
    #[serde(skip)]
    pub event_system: Option<EventSystem>,
    /// Narrative integration system for story fragments
    #[serde(skip)]
    pub narrative_integration: Option<NarrativeIntegration>,
    /// Biome system for enhanced terrain generation
    #[serde(skip)]
    pub biome_system: Option<BiomeSystem>,
    /// Grammar system for dynamic text generation
    #[serde(skip)]
    pub grammar_system: Option<Grammar>,
    /// Content template system for procedural content creation
    #[serde(skip)]
    pub template_library: Option<TemplateLibrary>,
    /// Constraint system for map validation and generation rules
    #[serde(skip)]
    pub constraint_system: Option<ConstraintSystem>,
    /// Generation pipeline to coordinate all procedural systems
    #[serde(skip)]
    pub generation_pipeline: Option<GenerationPipeline>,
    /// Light manipulation system for beam mechanics and refraction
    #[serde(default)]
    pub light_system: super::light::LightSystem,
    /// Void energy system for reality distortion and void abilities
    #[serde(default)]
    pub void_system: super::void_energy::VoidSystem,
    /// Crystal resonance system for frequency tracking and harmonic effects
    #[serde(default)]
    pub crystal_system: super::crystal_resonance::CrystalSystem,
}

/// Floating damage number for visual feedback
#[derive(Clone)]
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

impl GameState {
    pub fn new(seed: u64) -> Self {
        // Generate world map
        let world_map = WorldMap::generate(seed);
        let world_x = super::world_map::WORLD_WIDTH / 2;
        let world_y = super::world_map::WORLD_HEIGHT / 2;
        
        // Get world context for starting tile
        let (biome, terrain, elevation, poi, _resources, _connected, level) = world_map.get(world_x, world_y);
        
        // Generate tile map using world context
        let tile_seed = world_map.tile_seed(world_x, world_y);
        let mut rng = ChaCha8Rng::seed_from_u64(tile_seed);
        let (mut map, rooms) = Map::generate_from_world_with_poi(&mut rng, biome, terrain, elevation, poi);
        let (px, py) = rooms[0];
        
        // Add world exit to starting tile (near spawn point)
        let exit_x = (px + 1).min(map.width as i32 - 1) as usize;
        let exit_y = py as usize;
        map.tiles[exit_y * map.width + exit_x] = Tile::WorldExit;
        
        let visible = {
            let mut fov = FieldOfView::new(super::constants::FOV_RANGE);
            fov.calculate(&map, (px, py));
            let mut vis = HashSet::new();
            for &(x, y) in &fov.visible_tiles {
                if let Some(idx) = map.pos_to_idx(x, y) {
                    vis.insert(idx);
                }
            }
            vis
        };
        let table = get_biome_spawn_table(&biome);

        // Spawn enemies (fewer on starting tile for hospitable start)
        let mut enemies = Vec::new();
        let max_enemies = 8; // Limit total enemies regardless of clearing count
        let safe_distance = 15; // Minimum distance from player spawn
        let (px, py) = rooms[0]; // Player spawn position
        
        let safe_rooms: Vec<_> = rooms.iter()
            .filter(|&&(rx, ry)| {
                let dx = (rx - px).abs();
                let dy = (ry - py).abs();
                dx >= safe_distance || dy >= safe_distance
            })
            .cloned()
            .collect();
            
        // Use spatial distribution to spread out enemy spawns
        let distributed_positions = distribute_points_grid(
            &safe_rooms, 
            max_enemies, 
            20, // Minimum distance between enemies
            &mut rng
        );
            
        for (rx, ry) in distributed_positions {
            if let Some(id) = weighted_pick_by_level_and_tier(&table.enemies, level, &mut rng, false) {
                enemies.push(Enemy::new(rx, ry, id));
            }
        }

        // Spawn NPCs
        let mut npcs = Vec::new();
        let late_room = rooms.len().saturating_sub(2);
        for spawn in &table.npcs {
            let room_idx = match spawn.room.as_deref() {
                Some("late") => Some(late_room),
                Some("last") => Some(rooms.len() - 1),
                Some("first") => Some(0),
                _ => {
                    if rng.gen_ratio(spawn.weight.min(10), 10) {
                        Some(rng.gen_range(1..rooms.len()))
                    } else { None }
                }
            };
            if let Some(idx) = room_idx {
                if idx < rooms.len() {
                    let (rx, ry) = rooms[idx];
                    // If spawning in first room (where player is), find adjacent position
                    let (npc_x, npc_y) = if idx == 0 {
                        // Try adjacent positions around the room center
                        let offsets = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)];
                        let mut spawn_pos = (rx, ry);
                        for &(dx, dy) in &offsets {
                            let test_x = rx + dx;
                            let test_y = ry + dy;
                            if test_x >= 0 && test_y >= 0 && 
                               test_x < map.width as i32 && test_y < map.height as i32 {
                                let test_idx = map.idx(test_x, test_y);
                                if map.tiles[test_idx].walkable() && 
                                   (test_x != px || test_y != py) { // Don't spawn on player
                                    spawn_pos = (test_x, test_y);
                                    break;
                                }
                            }
                        }
                        spawn_pos
                    } else {
                        (rx, ry)
                    };
                    npcs.push(Npc::new(npc_x, npc_y, &spawn.id));
                }
            }
        }

        // Spawn items (more on starting tile for hospitable start)
        let mut items = Vec::new();
        let mut used_positions = HashSet::new();
        
        // Always spawn hand torch near player start
        items.push(Item::new(px + 1, py, "hand_torch"));
        used_positions.insert((px + 1, py));
        
        // Always spawn glass pick (wall break tool) near player start
        items.push(Item::new(px - 1, py, "glass_pick"));
        used_positions.insert((px - 1, py));
        
        for spawn in &table.items {
            if let Some("last") = spawn.room.as_deref() {
                if let Some(&(rx, ry)) = rooms.last() {
                    if !used_positions.contains(&(rx, ry)) {
                        used_positions.insert((rx, ry));
                        // Check tier eligibility for last room items
                        if let Some(item_def) = super::item::get_item_def(&spawn.id) {
                            let tier_threshold = match level {
                                1 => 1, 2..=3 => 2, 4..=6 => 3, 7..=8 => 4, 9..=10 => 5, _ => 1,
                            };
                            if item_def.tier <= tier_threshold {
                                items.push(Item::new(rx, ry, &spawn.id));
                            }
                        }
                    }
                }
                continue;
            }
            for _ in 0..(spawn.weight + 1) { // +1 for hospitable start
                if let Some(&(rx, ry)) = rooms.get(rng.gen_range(1..rooms.len())) {
                    let ix = rx + rng.gen_range(-1..=1);
                    let iy = ry + rng.gen_range(-1..=1);
                    if !used_positions.contains(&(ix, iy)) {
                        used_positions.insert((ix, iy));
                        // Check tier eligibility for regular items
                        if let Some(item_def) = super::item::get_item_def(&spawn.id) {
                            let tier_threshold = match level {
                                1 => 1, 2..=3 => 2, 4..=6 => 3, 7..=8 => 4, 9..=10 => 5, _ => 1,
                            };
                            if item_def.tier <= tier_threshold {
                                items.push(Item::new(ix, iy, &spawn.id));
                            }
                        }
                    }
                }
            }
        }

        // Spawn chests in some rooms
        let mut chests = Vec::new();
        let chest_rooms: Vec<_> = rooms.iter()
            .skip(2) // Skip first two rooms (player start and adjacent)
            .take(3) // Limit to 3 chests per tile
            .collect();
            
        for &(rx, ry) in chest_rooms {
            if rng.gen_ratio(3, 10) { // 30% chance for chest in each room
                let chest_types = ["wooden_chest", "supply_crate", "glass_cache"];
                let chest_id = chest_types[rng.gen_range(0..chest_types.len())];
                
                // Generate loot for the chest
                let mut chest = Chest::new(rx, ry, chest_id);
                if let Some(def) = super::chest::get_chest_def(chest_id) {
                    if let Some(loot_table) = &def.loot_table {
                        let loot = generate_loot(loot_table, rx, ry, &mut rng);
                        for item in loot {
                            chest.add_item(item);
                        }
                    }
                }
                chests.push(chest);
            }
        }

        // Place micro-structures
        let biome_str = match biome {
            super::world_map::Biome::Saltflat => "saltflat",
            super::world_map::Biome::Oasis => "oasis", 
            super::world_map::Biome::Ruins => "ruins",
            super::world_map::Biome::Scrubland => "scrubland",
            _ => "saltflat",
        };
        
        let (microstructures, mut structure_npcs, mut structure_chests, mut structure_items) = 
            place_microstructures(&mut map, biome_str, &rooms, (px, py), &mut rng);
        
        // Add structure entities to main collections
        npcs.append(&mut structure_npcs);
        chests.append(&mut structure_chests);
        items.append(&mut structure_items);

        let ambient = 100u8;
        let light_sources = vec![LightSource { x: px, y: py, radius: 8, intensity: 120 }]; // Reduced from 150 to avoid glare
        let light_map = compute_lighting(&light_sources, ambient);

        let mut state = Self {
            player_x: px, player_y: py, player_hp: 20, player_max_hp: 20,
            player_ap: default_player_ap(), player_max_ap: default_player_ap(),
            player_reflex: 5, player_armor: 0, player_xp: 0, player_level: 0,
            pending_stat_points: 0,
            salt_scrip: 50,
            equipped_weapon: None,
            // Faction reputation system (-100 to +100 per faction)
            faction_reputation: HashMap::new(),
            last_damage_dealt: 0,
            completed_rituals: Vec::new(),
            equipment: Equipment::default(),
            status_effects: Vec::new(),
            map, enemies, npcs, items, chests, inventory: Vec::new(),
            visible: visible.clone(), revealed: visible,
            player_fov: FieldOfView::new(super::constants::FOV_RANGE),
            light_map, ambient_light: ambient,
            messages: vec![GameMessage::new("Welcome to the Saltglass Steppe.", MsgType::System, 0)],
            turn: 0, rng, storm: Storm::forecast(&mut ChaCha8Rng::seed_from_u64(seed + 1)),
            refraction: 0, adaptations: Vec::new(), adaptations_hidden_turns: 0,
            quest_log: QuestLog::default(),
            triggered_effects: Vec::new(),
            decoys: Vec::new(),
            enemy_positions: HashMap::new(),
            npc_positions: HashMap::new(),
            item_positions: HashMap::new(),
            chest_positions: HashMap::new(),
            spatial_dirty: true,
            event_queue: Vec::new(),
            hit_flash_positions: Vec::new(),
            damage_numbers: Vec::new(),
            projectile_trails: Vec::new(),
            light_beams: Vec::new(),
            mock_combat_hit: None,
            mock_combat_damage: None,
            meta: super::meta::MetaProgress::load(),
            world_map: Some(world_map),
            world_x,
            world_y,
            layer: 0,
            time_of_day: 8,
            weather: Weather::Clear,
            wait_counter: 0,
            tutorial_progress: TutorialProgress::default(),
            map_features: MapFeatures::new(),
            sanity: SanitySystem::new(),
            psychic: super::psychic::PsychicState::default(),
            pending_trade: None,
            animation_frame: 0,
            storm_changed_tiles: HashSet::new(),
            pending_dialogue: None,
            debug_god_view: false,
            debug_phase: false,
            debug_disable_glare: false,
            seed,
            narrative_generator: None,
            world_history: Vec::new(),
            story_model: None,
            pending_book_open: None,
            skills: super::skills::SkillsState::default(),
            microstructures,
            event_system: None,
            narrative_integration: None,
            biome_system: None,
            grammar_system: None,
            template_library: None,
            constraint_system: None,
            generation_pipeline: None,
            light_system: super::light::LightSystem::default(),
            void_system: super::void_energy::VoidSystem::new(),
            crystal_system: super::crystal_resonance::CrystalSystem::new(),
        };
        
        // Initialize dynamic event system
        let event_system = EventSystem::new();
        let _event_context = EventContext {
            player_hp: state.player_hp,
            player_max_hp: state.player_max_hp,
            player_x: state.player_x,
            player_y: state.player_y,
            turn: state.turn,
            biome: biome.as_str().to_string(),
            storm_intensity: state.storm.intensity,
            refraction_level: state.refraction,
            variables: std::collections::HashMap::new(),
        };
        // Don't trigger events on initialization, just set up the system
        state.event_system = Some(event_system);
        
        // Initialize narrative integration system
        let mut narrative_integration = NarrativeIntegration::new();
        let narrative_context = super::generation::narrative::NarrativeContext {
            player_x: state.player_x,
            player_y: state.player_y,
            current_biome: biome.as_str().to_string(),
            turn: state.turn,
            faction_standings: std::collections::HashMap::new(),
            discovered_fragments: Vec::new(),
            player_adaptations: Vec::new(),
        };
        narrative_integration.initialize(&narrative_context, &mut state.rng);
        state.narrative_integration = Some(narrative_integration);
        
        // Initialize biome system
        let biome_system = BiomeSystem;
        state.biome_system = Some(biome_system);
        
        // Initialize grammar system
        let grammar_system = match Grammar::load_from_file("data/grammars/descriptions.json") {
            Ok(grammar) => Some(grammar),
            Err(_e) => {
                // Silently fail and use fallback descriptions
                None
            }
        };
        state.grammar_system = grammar_system;
        
        // Initialize template library
        let template_library = match TemplateLibrary::load_from_file("data/templates/content_templates.json") {
            Ok(library) => Some(library),
            Err(_e) => {
                // Silently fail and use fallback content generation
                None
            }
        };
        state.template_library = template_library;
        
        // Initialize constraint system
        let constraint_system = ConstraintSystem;
        state.constraint_system = Some(constraint_system);
        
        // Initialize generation pipeline
        let pipeline_config = GenerationConfig {
            passes: vec![], // Minimal config for now
        };
        let generation_pipeline = GenerationPipeline::new(pipeline_config);
        state.generation_pipeline = Some(generation_pipeline);
        
        // Initialize narrative generator and generate world history
        if let Ok(generator) = NarrativeGenerator::new() {
            let mut history_rng = ChaCha8Rng::seed_from_u64(seed + 2);
            let history = generator.generate_world_history(&mut history_rng, 5);
            state.world_history = history;
            state.narrative_generator = Some(generator);
        }
        
        // Initialize story model
        let story_model = StoryModel::new(seed + 3);
        state.story_model = Some(story_model);
        
        // Generate backstories for NPCs now that story model is available
        state.generate_npc_backstories();
        
        state.rebuild_spatial_index();
        state
    }

    /// Create a new game with a specific character class
    pub fn new_with_class(seed: u64, class_id: &str) -> Self {
        let mut state = Self::new(seed);
        
        if let Some(class) = super::meta::get_class(class_id) {
            state.player_hp = class.starting_hp;
            state.player_max_hp = class.starting_hp;
            state.player_ap = class.starting_ap;
            state.player_max_ap = class.starting_ap;
            
            // Add starting items
            for item_id in &class.starting_items {
                state.inventory.push(item_id.clone());
            }
            
            // Add starting adaptations
            for adapt_id in &class.starting_adaptations {
                if let Some(adapt) = Adaptation::from_id(adapt_id) {
                    state.adaptations.push(adapt);
                }
            }
            
            state.log(format!("You begin as a {}.", class.name));
        }
        
        state
    }

    /// Mark spatial index as dirty, requiring rebuild on next query
    pub fn mark_spatial_dirty(&mut self) {
        self.spatial_dirty = true;
    }
    
    /// Ensure spatial index is up to date before querying
    fn ensure_spatial_index(&mut self) {
        if self.spatial_dirty {
            self.rebuild_spatial_index_internal();
        }
    }
    
    /// Internal rebuild that clears the dirty flag
    fn rebuild_spatial_index_internal(&mut self) {
        self.enemy_positions.clear();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.hp > 0 {
                self.enemy_positions.insert((e.x, e.y), i);
            }
        }
        self.npc_positions.clear();
        for (i, n) in self.npcs.iter().enumerate() {
            self.npc_positions.insert((n.x, n.y), i);
        }
        self.item_positions.clear();
        for (i, item) in self.items.iter().enumerate() {
            self.item_positions.entry((item.x, item.y)).or_default().push(i);
        }
        self.chest_positions.clear();
        for (i, chest) in self.chests.iter().enumerate() {
            self.chest_positions.insert((chest.x, chest.y), i);
        }
        self.spatial_dirty = false;
    }

    /// Rebuild spatial index (public, for backwards compatibility)
    pub fn rebuild_spatial_index(&mut self) {
        self.rebuild_spatial_index_internal();
    }

    /// Travel to a new world tile (lazy generation)
    pub fn travel_to_tile(&mut self, new_wx: usize, new_wy: usize) {
        let world_map = match &self.world_map {
            Some(wm) => wm,
            None => return,
        };
        
        let (biome, terrain, elevation, poi, _resources, _connected, level) = world_map.get(new_wx, new_wy);
        let tile_seed = world_map.tile_seed(new_wx, new_wy);
        let mut rng = ChaCha8Rng::seed_from_u64(tile_seed);
        
        // Generate new tile map
        let (map, rooms) = Map::generate_from_world_with_poi(&mut rng, biome, terrain, elevation, poi);
        let (px, py) = rooms[0];
        
        // Spawn enemies based on POI
        let table = get_biome_spawn_table(&biome);
        let mut enemies = Vec::new();
        let enemy_count = match poi {
            super::world_map::POI::Town => 0,
            super::world_map::POI::Shrine => 1,
            _ => 6.min(rooms.len().saturating_sub(2)), // Cap at 6 enemies
        };
        
        let safe_distance = 15; // Minimum distance from player spawn
        let safe_rooms: Vec<_> = rooms.iter()
            .filter(|&&(rx, ry)| {
                let dx = (rx - px).abs();
                let dy = (ry - py).abs();
                dx >= safe_distance || dy >= safe_distance
            })
            .cloned()
            .collect();
            
        // Use spatial distribution to spread out enemy spawns
        let distributed_positions = distribute_points_grid(
            &safe_rooms, 
            enemy_count, 
            20, // Minimum distance between enemies
            &mut rng
        );
            
        for (rx, ry) in distributed_positions {
            if let Some(id) = weighted_pick_by_level_and_tier(&table.enemies, level, &mut rng, false) {
                enemies.push(Enemy::new(rx, ry, id));
            }
        }
        
        // Spawn items
        let mut items = Vec::new();
        let mut used_positions = HashSet::new();
        for spawn in &table.items {
            for _ in 0..spawn.weight {
                if let Some(&(rx, ry)) = rooms.get(rng.gen_range(1..rooms.len())) {
                    let ix = rx + rng.gen_range(-1..=1);
                    let iy = ry + rng.gen_range(-1..=1);
                    if !used_positions.contains(&(ix, iy)) {
                        used_positions.insert((ix, iy));
                        // Check tier eligibility for travel items
                        if let Some(item_def) = super::item::get_item_def(&spawn.id) {
                            let tier_threshold = match level {
                                1 => 1, 2..=3 => 2, 4..=6 => 3, 7..=8 => 4, 9..=10 => 5, _ => 1,
                            };
                            if item_def.tier <= tier_threshold {
                                items.push(Item::new(ix, iy, &spawn.id));
                            }
                        }
                    }
                }
            }
        }
        
        // Update state
        self.world_x = new_wx;
        self.world_y = new_wy;
        self.map = map;
        self.enemies = enemies;
        self.items = items;
        self.npcs = Vec::new(); // NPCs are tile-specific
        self.player_x = px;
        self.player_y = py;
        self.update_fov();
        self.rebuild_spatial_index();
        self.update_lighting();
        
        // Generate narrative fragments for new tile
        self.generate_narrative_fragments(biome.as_str());
        
        // Generate biome-specific environmental content
        self.generate_biome_content(&biome, level as u8);
        
        // Generate crystal formations for appropriate biomes
        self.generate_crystal_formations(&biome, &rooms, &mut rng);
        
        // Generate template-based procedural content
        let mut template_context = std::collections::HashMap::new();
        template_context.insert("biome".to_string(), serde_json::Value::String(biome.as_str().to_string()));
        template_context.insert("level".to_string(), serde_json::Value::Number(serde_json::Number::from(level)));
        template_context.insert("storm_intensity".to_string(), serde_json::Value::String(
            if self.storm.intensity <= 2 { "low" } else { "high" }.to_string()
        ));
        self.generate_template_content("encounter", template_context);
        
        self.log(format!("You enter a new area ({:?} {:?}).", biome, terrain));
    }

    /// Travel to a world tile with safe spawn (not on wall/enemy/glass)
    pub fn travel_to_tile_safe(&mut self, new_wx: usize, new_wy: usize) {
        self.travel_to_tile(new_wx, new_wy);
        
        // Find safe spawn position (not wall, glass, or enemy)
        let (mut px, mut py) = (self.player_x, self.player_y);
        
        // Check if current position is safe
        let is_safe = |map: &Map, enemies: &[Enemy], x: i32, y: i32| -> bool {
            if let Some(tile) = map.get(x, y) {
                if matches!(*tile, Tile::Floor { .. }) { return false; }
            } else { return false; }
            !enemies.iter().any(|e| e.x == x && e.y == y && e.hp > 0)
        };
        
        if !is_safe(&self.map, &self.enemies, px, py) {
            // Search for safe position in expanding squares
            'search: for radius in 1i32..20 {
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        if dx.abs() == radius || dy.abs() == radius {
                            let nx = px + dx;
                            let ny = py + dy;
                            if is_safe(&self.map, &self.enemies, nx, ny) {
                                px = nx;
                                py = ny;
                                break 'search;
                            }
                        }
                    }
                }
            }
        }
        
        self.player_x = px;
        self.player_y = py;
        self.update_fov();
        self.update_lighting();
    }

    /// Enter subterranean layer (go down stairs)
    pub fn enter_subterranean(&mut self) -> bool {
        // Check if standing on stairs down
        if let Some(tile) = self.map.get(self.player_x, self.player_y) {
            if *tile != Tile::StairsDown { return false; }
        } else { return false; }

        self.layer -= 1;
        let seed = self.world_map.as_ref()
            .map(|wm| wm.tile_seed(self.world_x, self.world_y))
            .unwrap_or(42)
            .wrapping_add(self.layer.unsigned_abs() as u64 * 1000);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        
        let (map, rooms) = Map::generate_subterranean(&mut rng, self.layer);
        let (px, py) = rooms[0];
        
        self.map = map;
        self.enemies = Vec::new();
        self.items = Vec::new();
        self.npcs = Vec::new();
        self.player_x = px;
        self.player_y = py;
        self.update_fov();
        self.rebuild_spatial_index();
        self.update_lighting();
        
        self.log(format!("You descend to level {}.", -self.layer));
        true
    }

    /// Exit subterranean layer (go up stairs)
    pub fn exit_subterranean(&mut self) -> bool {
        // Check if standing on stairs up
        if let Some(tile) = self.map.get(self.player_x, self.player_y) {
            if *tile != Tile::StairsUp { return false; }
        } else { return false; }

        if self.layer >= 0 { return false; } // Already on surface

        self.layer += 1;
        
        if self.layer == 0 {
            // Return to surface - regenerate surface tile
            self.travel_to_tile(self.world_x, self.world_y);
            self.log("You return to the surface.");
        } else {
            // Go up one underground level
            let seed = self.world_map.as_ref()
                .map(|wm| wm.tile_seed(self.world_x, self.world_y))
                .unwrap_or(42)
                .wrapping_add(self.layer.unsigned_abs() as u64 * 1000);
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            
            let (map, rooms) = Map::generate_subterranean(&mut rng, self.layer);
            let (px, py) = rooms.last().copied().unwrap_or((5, 5));
            
            self.map = map;
            self.enemies = Vec::new();
            self.items = Vec::new();
            self.npcs = Vec::new();
            self.player_x = px;
            self.player_y = py;
            self.update_fov();
            self.rebuild_spatial_index();
            self.update_lighting();
            
            self.log(format!("You ascend to level {}.", -self.layer));
        }
        true
    }

    pub fn update_lighting(&mut self) {
        let mut sources = Vec::new();
        
        // Always add default player light
        sources.push(LightSource { x: self.player_x, y: self.player_y, radius: 5, intensity: 100 });
        
        // Equipped light sources (check all slots)
        for (_, slot_item) in self.equipment.iter() {
            if let Some(id) = slot_item {
                if let Some(def) = get_item_def(id) {
                    if let Some(ref ls) = def.light_source {
                        sources.push(LightSource { x: self.player_x, y: self.player_y, radius: ls.radius, intensity: ls.intensity });
                    }
                }
            }
        }
        // Map lights
        for ml in &self.map.lights {
            if let Some(def) = super::light_defs::get_light_def(&ml.id) {
                sources.push(LightSource { x: ml.x, y: ml.y, radius: def.radius, intensity: def.intensity });
            }
        }
        // Items on ground with light_source property
        for item in &self.items {
            if let Some(def) = get_item_def(&item.id) {
                if let Some(ref ls) = def.light_source {
                    sources.push(LightSource { x: item.x, y: item.y, radius: ls.radius, intensity: ls.intensity });
                }
            }
        }
        self.light_map = compute_lighting(&sources, self.effective_ambient_light());
    }

    /// Update player field of view using shadow casting algorithm
    pub fn update_fov(&mut self) {
        self.player_fov.mark_dirty();
        self.player_fov.calculate(&self.map, (self.player_x, self.player_y));
        
        // Update legacy visible set for compatibility
        self.visible.clear();
        for &(x, y) in &self.player_fov.visible_tiles {
            if let Some(idx) = self.map.pos_to_idx(x, y) {
                self.visible.insert(idx);
            }
        }
        
        // Update revealed tiles
        self.revealed.extend(&self.visible);
    }

    /// Generate procedural item lore using narrative templates
    pub fn generate_item_lore(&mut self, item_category: &str) -> Option<String> {
        if let Some(ref generator) = self.narrative_generator {
            let context = self.create_narrative_context();
            generator.generate_contextual_item_lore(item_category, &context, &mut self.rng)
        } else {
            None
        }
    }

    /// Generate procedural location description
    pub fn generate_location_description(&mut self, location_type: &str) -> Option<String> {
        if let Some(ref generator) = self.narrative_generator {
            generator.generate_location_description(location_type, &mut self.rng)
        } else {
            None
        }
    }

    /// Generate contextual description based on current game state
    pub fn generate_contextual_description(&mut self) -> Option<String> {
        if let Some(ref generator) = self.narrative_generator {
            let context = self.create_narrative_context();
            
            // Try story-based description first
            if let Some(ref story_model) = self.story_model {
                if let Some(faction_lore) = story_model.get_faction_lore("Mirror Monks") {
                    if context.faction_reputation.get("Mirror Monks").unwrap_or(&0) > &50 {
                        return Some(format!("Your understanding of the Mirror Monks deepens: {}", faction_lore));
                    }
                }
            }
            
            generator.generate_contextual_description(&context, &mut self.rng)
        } else {
            None
        }
    }

    /// Generate environmental storytelling text
    pub fn generate_environmental_text(&mut self, environment_type: &str) -> Option<String> {
        if let Some(ref generator) = self.narrative_generator {
            generator.generate_environmental_text(environment_type, &mut self.rng)
        } else {
            None
        }
    }

    /// Generate markov chain text for flavor
    pub fn generate_flavor_text(&mut self, max_words: usize) -> String {
        if let Some(ref generator) = self.narrative_generator {
            generator.generate_markov_text(&mut self.rng, max_words)
        } else {
            "The glass whispers secrets.".to_string()
        }
    }

    /// Get area description for current map
    pub fn get_area_description(&self) -> Option<String> {
        self.map.area_description.clone()
    }
    
    /// Generate NPC backstory using story model
    pub fn generate_npc_backstory(&mut self, npc_id: &str, story_model: &StoryModel) -> Option<String> {
        if let Some(ref generator) = self.narrative_generator {
            // Get NPC definition to understand their faction
            if let Some(npc_def) = super::npc::get_npc_def(npc_id) {
                // Try to find a character from the story model that matches this NPC's faction
                let matching_character = story_model.characters.iter()
                    .find(|(_, c)| c.faction == npc_def.faction);
                
                if let Some((_, character)) = matching_character {
                    // Generate backstory based on the character's history
                    let achievements = if character.achievements.is_empty() {
                        "made their mark on history".to_string()
                    } else {
                        character.achievements.join(", ")
                    };
                    
                    let backstory = format!(
                        "{} has connections to {}. They remember when {}.",
                        npc_def.name,
                        character.name,
                        achievements
                    );
                    Some(backstory)
                } else {
                    // Generate generic backstory using narrative templates
                    generator.generate_environmental_text("npc_backstory", &mut self.rng)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Generate backstories for all NPCs using story model
    pub fn generate_npc_backstories(&mut self) {
        if let Some(ref story_model) = self.story_model.clone() {
            // Collect NPC IDs that need backstories
            let npc_ids: Vec<(usize, String)> = self.npcs.iter().enumerate()
                .filter(|(_, npc)| npc.backstory.is_none())
                .map(|(i, npc)| (i, npc.id.clone()))
                .collect();
            
            // Generate backstories
            for (index, npc_id) in npc_ids {
                let backstory = self.generate_npc_backstory(&npc_id, &story_model);
                self.npcs[index].backstory = backstory;
            }
        }
    }

    /// Create narrative context from current game state
    fn create_narrative_context(&self) -> NarrativeContext {
        let biome = if let Some(ref world_map) = self.world_map {
            let (biome, _, _, _, _, _, _) = world_map.get(self.world_x, self.world_y);
            Some(format!("{:?}", biome))
        } else {
            None
        };

        let adaptations = self.adaptations.iter()
            .map(|a| format!("{:?}", a).to_lowercase())
            .collect();

        NarrativeContext {
            biome,
            terrain: None,
            adaptations,
            faction_reputation: self.faction_reputation.clone(),
            refraction_level: self.refraction,
            location_type: None,
        }
    }

    /// Get the generated world history
    pub fn get_world_history(&self) -> &[String] {
        &self.world_history
    }

    /// Get artifact inscription from story model
    pub fn get_artifact_inscription(&self, artifact_name: &str) -> Option<String> {
        self.story_model.as_ref()?.get_artifact_inscription(artifact_name)
    }

    /// Get shrine text from story model
    pub fn get_shrine_text(&self, location: &str) -> Option<String> {
        self.story_model.as_ref()?.get_shrine_text(location)
    }

    /// Get character relationships from story model
    pub fn get_character_relationships(&self, character_id: &str) -> Vec<String> {
        self.story_model.as_ref()
            .map(|sm| sm.get_character_relationships(character_id))
            .unwrap_or_default()
    }

    /// Get faction lore from story model
    pub fn get_faction_lore(&self, faction_name: &str) -> Option<String> {
        self.story_model.as_ref()?.get_faction_lore(faction_name)
    }

    /// Generate crystal formations for appropriate biomes
    fn generate_crystal_formations(&mut self, biome: &super::world_map::Biome, rooms: &[(i32, i32)], rng: &mut ChaCha8Rng) {
        use super::crystal_resonance::CrystalFrequency;
        
        let formation_chance = match biome {
            super::world_map::Biome::Ruins => 0.6,
            super::world_map::Biome::Oasis => 0.4,
            super::world_map::Biome::Saltflat => 0.3,
            super::world_map::Biome::Scrubland => 0.2,
            super::world_map::Biome::Desert => 0.1,
        };
        
        if !rng.gen_bool(formation_chance as f64) {
            return;
        }
        
        let formation_count = match biome {
            super::world_map::Biome::Ruins => rng.gen_range(2..=4),
            super::world_map::Biome::Oasis => rng.gen_range(1..=3),
            _ => rng.gen_range(1..=2),
        };
        
        let frequencies = CrystalFrequency::all();
        
        for _ in 0..formation_count {
            if let Some(&(rx, ry)) = rooms.get(rng.gen_range(0..rooms.len())) {
                let x = rx + rng.gen_range(-2..=2);
                let y = ry + rng.gen_range(-2..=2);
                
                // Don't place on player spawn or too close to enemies
                if (x - self.player_x).abs() < 5 && (y - self.player_y).abs() < 5 {
                    continue;
                }
                
                let frequency = frequencies[rng.gen_range(0..frequencies.len())];
                self.crystal_system.add_crystal(x, y, frequency);
                
                self.log_typed(format!("A {} crystal formation glimmers nearby.", frequency.name().to_lowercase()), MsgType::Loot);
            }
        }
    }

    /// Add player event to story model
    pub fn add_story_event(&mut self, event_type: EventType, description: String) {
        if let Some(ref mut story_model) = self.story_model {
            story_model.add_player_event(event_type, description);
        }
    }

    /// Calculate effective ambient light based on time of day and weather
    pub fn effective_ambient_light(&self) -> u8 {
        // Underground has fixed low ambient
        if self.layer < 0 {
            return 30;
        }
        
        // Base ambient from time of day (0-23 hours)
        let time_ambient = match self.time_of_day {
            0..=4 => 30,    // Night
            5..=6 => 60,    // Dawn
            7..=17 => 120,  // Day
            18..=19 => 80,  // Dusk
            20..=23 => 40,  // Night
            _ => 100,
        };
        
        // Apply weather modifier
        let weather_mod = self.weather.ambient_modifier();
        (time_ambient as i32 + weather_mod).clamp(10, 200) as u8
    }

    /// Advance time by one turn (10 turns = 1 hour)
    pub fn tick_time(&mut self) {
        if self.turn % 10 == 0 {
            self.time_of_day = (self.time_of_day + 1) % 24;
            
            // Random weather changes at dawn/dusk
            if self.time_of_day == 6 || self.time_of_day == 18 {
                let roll = self.rng.gen_range(0..10);
                self.weather = match roll {
                    0..=6 => Weather::Clear,
                    7..=8 => Weather::Dusty,
                    9 => Weather::Sandstorm,
                    _ => Weather::Clear,
                };
            }
        }
    }

    pub fn get_light_level(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 { return 0; }
        let idx = y as usize * self.map.width + x as usize;
        self.light_map.get(idx).copied().unwrap_or(0)
    }

    pub fn trigger_effect(&mut self, effect: &str, duration: u32) {
        self.triggered_effects.push(TriggeredEffect {
            effect: effect.to_string(),
            turns_remaining: duration,
        });
    }

    pub fn emit(&mut self, event: GameEvent) {
        self.event_queue.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.event_queue)
    }

    pub fn visible_adaptation_count(&self) -> usize {
        if self.adaptations_hidden_turns > 0 { 0 } else { self.adaptations.len() }
    }

    /// Gain XP and check for level up
    pub fn gain_xp(&mut self, amount: u32) {
        use super::progression::{xp_for_level, stat_points_per_level, max_level};
        
        self.player_xp += amount;
        self.log(format!("+{} XP", amount));
        
        // Check for level up
        while self.player_level < max_level() {
            let next_threshold = xp_for_level(self.player_level + 1);
            if self.player_xp >= next_threshold {
                self.player_level += 1;
                let points = stat_points_per_level();
                self.pending_stat_points += points;
                // Also gain skill points
                self.skills.skill_points += 2;
                self.log(format!("⬆ LEVEL {}! (+{} stat points, +2 skill points)", self.player_level, points));
                self.emit(GameEvent::LevelUp { level: self.player_level });
            } else {
                break;
            }
        }
    }

    /// Allocate a stat point to a specific stat
    pub fn allocate_stat(&mut self, stat: &str) -> bool {
        if self.pending_stat_points <= 0 { return false; }
        
        match stat {
            "max_hp" => {
                self.player_max_hp += 1;
                self.player_hp += 1; // Also heal
            }
            "max_ap" => self.player_max_ap += 1,
            "reflex" => self.player_reflex += 1,
            _ => return false,
        }
        
        self.pending_stat_points -= 1;
        self.log(format!("+1 {}", stat));
        true
    }

    /// End turn: reset AP, tick status effects, run enemy turns, tick storm, tick time
    pub fn end_turn(&mut self) {
        use super::systems::{System, StatusEffectSystem, StormSystem};
        
        // Ensure spatial index is up to date before AI/systems run
        self.ensure_spatial_index();
        
        self.player_ap = self.player_max_ap;
        StatusEffectSystem.update(self);
        self.psychic.tick();
        self.skills.tick();
        self.light_system.update(&mut self.rng);
        self.void_system.update(&mut self.rng);
        self.crystal_system.update(&mut self.rng);
        self.tick_turn();
        self.update_enemies();
        if self.storm.tick() { StormSystem::apply_storm(self); }
        self.tick_time();
        self.update_lighting();
        self.update_fov();
        
        // Check for dynamic events
        self.check_dynamic_events();
        
        // Process queued events
        self.process_events();
    }
    
    /// Generate narrative fragments for the current tile
    fn generate_narrative_fragments(&mut self, biome: &str) {
        if let Some(ref mut narrative) = self.narrative_integration {
            let context = super::generation::narrative::NarrativeContext {
                player_x: self.player_x,
                player_y: self.player_y,
                current_biome: biome.to_string(),
                turn: self.turn,
                faction_standings: std::collections::HashMap::new(),
                discovered_fragments: Vec::new(),
                player_adaptations: self.adaptations.iter().map(|a| a.name().to_string()).collect(),
            };
            
            let fragments = narrative.generate_fragments(&context, &mut self.rng);
            let fragment_count = fragments.len();
            
            if fragment_count > 0 {
                // Track narrative momentum
                narrative.track_narrative_event("fragments_generated", &context);
            }
            
            // Log after releasing the borrow
            let _ = narrative;
            if fragment_count > 0 {
                self.log(format!("You sense {} story fragments in this area.", fragment_count));
            }
        }
    }

    /// Generate biome-specific environmental content
    fn generate_biome_content(&mut self, biome: &super::world_map::Biome, _level: u8) {
        if let Some(ref _biome_system) = self.biome_system {
            let context = super::generation::BiomeGenerationContext {
                biome: *biome,
                storm_intensity: self.storm.intensity,
                time_of_day: "day".to_string(), // Could be enhanced with day/night cycle
                weather_conditions: "clear".to_string(), // Could be enhanced with weather system
                player_adaptations: self.adaptations.iter().map(|a| a.name().to_string()).collect(),
            };
            
            // Generate environmental description using Grammar system
            let description = if let Some(ref grammar) = self.grammar_system {
                let grammar_context = GrammarContext {
                    variables: std::collections::HashMap::new(),
                };
                
                match grammar.generate("description", &grammar_context, &mut self.rng) {
                    Ok(generated_desc) => generated_desc,
                    Err(_) => {
                        // Fallback to BiomeSystem description
                        super::generation::BiomeSystem::generate_environment_description(
                            *biome, &context, &mut self.rng
                        )
                    }
                }
            } else {
                // Fallback to BiomeSystem description
                super::generation::BiomeSystem::generate_environment_description(
                    *biome, &context, &mut self.rng
                )
            };
            
            // Generate environmental features (1-3 features per tile)
            let feature_count = self.rng.gen_range(1..=3);
            let features = super::generation::BiomeSystem::generate_environmental_features(
                *biome, feature_count, &mut self.rng
            );
            
            // Check for hazards
            let hazards = super::generation::BiomeSystem::check_hazards(
                *biome, &context, &mut self.rng
            );
            
            // Log environmental content
            if !description.is_empty() {
                self.log(description);
            }
            
            for feature in &features {
                if self.rng.gen_range(0.0..1.0) < 0.3 { // 30% chance to notice each feature
                    self.log(format!("You notice: {}", feature.description_template));
                }
            }
            
            for hazard in &hazards {
                if hazard.severity >= 5 { // Only log significant hazards
                    self.log(format!("Warning: {}", hazard.description));
                }
            }
        }
    }

    /// Generate procedural content using templates
    fn generate_template_content(&mut self, category: &str, context_vars: std::collections::HashMap<String, serde_json::Value>) {
        if let Some(ref template_library) = self.template_library {
            let template_context = TemplateContext {
                variables: context_vars,
            };
            
            // Find templates in the specified category by trying known template IDs
            let template_candidates = match category {
                "encounter" => vec!["encounter_basic", "storm_encounter"],
                "room" => vec!["basic_room", "glass_room"],
                _ => vec![],
            };
            
            for template_id in template_candidates {
                if let Some(_template) = template_library.get_template(template_id) {
                    match template_library.instantiate(template_id, &template_context, &mut self.rng) {
                        Ok(result) => {
                            // Log the generated content
                            if let Some(description) = result.get("description") {
                                if let Some(desc_str) = description.as_str() {
                                    self.log(format!("Template content: {}", desc_str));
                                }
                            }
                            break; // Only generate one template per category
                        }
                        Err(_) => {
                            // Try next template
                            continue;
                        }
                    }
                }
            }
        }
    }

    /// Check for dynamic events based on current game state
    fn check_dynamic_events(&mut self) {
        if let Some(ref mut event_system) = self.event_system {
            let current_biome = if let Some(ref world_map) = self.world_map {
                world_map.get(self.world_x, self.world_y).0.as_str().to_string()
            } else {
                "desert".to_string()
            };
            
            let context = EventContext {
                player_hp: self.player_hp,
                player_max_hp: self.player_max_hp,
                player_x: self.player_x,
                player_y: self.player_y,
                turn: self.turn,
                biome: current_biome.clone(),
                storm_intensity: self.storm.intensity,
                refraction_level: self.refraction,
                variables: std::collections::HashMap::new(),
            };
            
            let triggered_events = event_system.check_triggers(&context, &mut self.rng);
            let has_events = !triggered_events.is_empty();
            let mut messages_to_log = Vec::new();
            
            for event_id in triggered_events {
                let mut event_context = context.clone();
                let messages = event_system.apply_consequences(&event_id, &mut event_context);
                
                // Apply consequences to game state
                if let Some(damage) = event_context.variables.get("damage_taken") {
                    if let Some(damage) = damage.as_i64() {
                        self.player_hp = (self.player_hp - damage as i32).max(0);
                    }
                }
                
                if let Some(healing) = event_context.variables.get("healing_received") {
                    if let Some(healing) = healing.as_i64() {
                        self.player_hp = (self.player_hp + healing as i32).min(self.player_max_hp);
                    }
                }
                
                if let Some(refraction) = event_context.variables.get("refraction_gained") {
                    if let Some(refraction) = refraction.as_u64() {
                        self.refraction += refraction as u32;
                    }
                }
                
                // Collect messages to log later
                messages_to_log.extend(messages);
            }
            
            // Track narrative events
            if has_events {
                if let Some(ref mut narrative) = self.narrative_integration {
                    narrative.track_narrative_event("dynamic_event", &super::generation::narrative::NarrativeContext {
                        player_x: self.player_x,
                        player_y: self.player_y,
                        current_biome: current_biome,
                        turn: self.turn,
                        faction_standings: std::collections::HashMap::new(),
                        discovered_fragments: Vec::new(),
                        player_adaptations: self.adaptations.iter().map(|a| a.name().to_string()).collect(),
                    });
                }
            }
            
            // Log messages after releasing borrows
            for message in messages_to_log {
                self.log(message);
            }
        }
    }

    /// Process all queued game events
    /// This enables decoupled communication between systems
    fn process_events(&mut self) {
        use super::systems::{System, LootSystem, QuestSystem};
        
        let events = self.drain_events();
        for event in events {
            // Dispatch to systems
            LootSystem.on_event(self, &event);
            QuestSystem.on_event(self, &event);
            
            // Internal logging/handling
            self.handle_event(&event);
        }
    }
    
    /// Handle a single game event - internal logging and state updates
    fn handle_event(&mut self, event: &GameEvent) {
        match event {
            GameEvent::EnemyKilled { enemy_id, x, y } => {
                self.log_typed(
                    format!("[Event] Enemy '{}' killed at ({}, {})", enemy_id, x, y),
                    MsgType::System
                );
            }
            GameEvent::LevelUp { level } => {
                self.log_typed(
                    format!("[Event] Player reached level {}!", level),
                    MsgType::Status
                );
            }
            GameEvent::ItemPickedUp { .. } => {
                // Handled by QuestSystem
            }
            GameEvent::AdaptationGained { name } => {
                self.log_typed(
                    format!("[Event] Gained adaptation: {}", name),
                    MsgType::Status
                );
            }
            GameEvent::StormArrived { intensity } => {
                self.log_typed(
                    format!("[Event] Storm arrived with intensity {}", intensity),
                    MsgType::Warning
                );
            }
            _ => {}
        }
    }

    /// Apply a status effect to the player
    pub fn apply_status(&mut self, effect: super::status::StatusEffect) {
        self.log_typed(format!("You are {}! ({} turns)", effect.name, effect.duration), MsgType::System);
        self.status_effects.push(effect);
    }

    /// Wait in place (costs 0 AP, ends turn). Auto-heals after 10 consecutive waits with no enemies nearby.
    pub fn wait_turn(&mut self) {
        // Check for nearby enemies (within 8 tiles, not FOV range)
        let enemies_nearby = self.enemies.iter().any(|e| {
            if e.hp <= 0 { return false; } // Ignore dead enemies
            let dx = (e.x - self.player_x).abs();
            let dy = (e.y - self.player_y).abs();
            dx <= 8 && dy <= 8 // Much smaller range for healing
        });
        
        if enemies_nearby {
            self.wait_counter = 0;
        } else {
            self.wait_counter += 1;
            // Auto-rest after 10 consecutive waits
            if self.wait_counter >= 10 && self.player_hp < self.player_max_hp {
                let heal = (self.player_max_hp / 20).max(1); // 5% instead of 10%
                self.player_hp = (self.player_hp + heal).min(self.player_max_hp);
                self.log_typed(format!("You rest and recover {} HP.", heal), MsgType::Status);
                self.wait_counter = 0;
            }
        }
        self.end_turn();
    }

    /// Rest to recover HP (50% max HP). Requires no nearby enemies and costs 10 turns.
    pub fn rest(&mut self) -> Result<(), String> {
        // Check for nearby enemies (within FOV range)
        for enemy in &self.enemies {
            let dx = (enemy.x - self.player_x).abs();
            let dy = (enemy.y - self.player_y).abs();
            if dx <= super::constants::FOV_RANGE && dy <= super::constants::FOV_RANGE {
                return Err("You cannot rest with enemies nearby!".to_string());
            }
        }

        // Heal 50% max HP
        let heal_amount = (self.player_max_hp as f32 * 0.5) as i32;
        let old_hp = self.player_hp;
        self.player_hp = (self.player_hp + heal_amount).min(self.player_max_hp);
        let actual_heal = self.player_hp - old_hp;

        if actual_heal > 0 {
            self.log_typed(
                format!("You rest and recover {} HP.", actual_heal),
                MsgType::Status,
            );
        } else {
            self.log_typed("You rest but are already at full health.", MsgType::Status);
        }

        // Advance 10 turns
        for _ in 0..10 {
            self.tick_turn();
        }

        // Process enemy turns (they get to act while you rest)
        self.update_enemies();

        Ok(())
    }

    /// Auto-end turn if player has no AP left
    pub(crate) fn check_auto_end_turn(&mut self) {
        if self.player_ap <= 0 {
            self.end_turn();
        }
    }

    fn tick_turn(&mut self) {
        self.turn += 1;
        if self.adaptations_hidden_turns > 0 {
            self.adaptations_hidden_turns -= 1;
            if self.adaptations_hidden_turns == 0 {
                self.log_typed("The tincture wears off. Your glow returns.", MsgType::Status);
            }
        }
        // Tick down triggered effects
        self.triggered_effects.retain_mut(|e| {
            e.turns_remaining = e.turns_remaining.saturating_sub(1);
            e.turns_remaining > 0
        });
        // Tick down decoys
        self.decoys.retain_mut(|d| {
            d.turns_remaining = d.turns_remaining.saturating_sub(1);
            d.turns_remaining > 0
        });
        
        // Apply light-based effects
        self.apply_light_effects();
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.log_typed(msg, MsgType::System);
    }

    pub fn log_typed(&mut self, msg: impl Into<String>, msg_type: MsgType) {
        self.messages.push(GameMessage::new(msg, msg_type, self.turn));
        if self.messages.len() > 5 { self.messages.remove(0); }
    }

    /// Execute a debug command
    pub fn debug_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        match parts.first().map(|s| *s) {
            Some("show") if parts.get(1) == Some(&"tile") => {
                self.debug_god_view = true;
                self.log("Debug: God view enabled");
            }
            Some("hide") if parts.get(1) == Some(&"tile") => {
                self.debug_god_view = false;
                self.log("Debug: God view disabled");
            }
            Some("sturdy") => {
                self.player_hp = 9999;
                self.player_max_hp = 9999;
                self.log("Debug: HP set to 9999/9999");
            }
            Some("phase") => {
                self.debug_phase = !self.debug_phase;
                self.log(format!("Debug: Phase {}", if self.debug_phase { "enabled" } else { "disabled" }));
            }
            Some("save_debug") => {
                let filename = if parts.len() > 1 {
                    format!("{}.ron", parts[1])
                } else {
                    format!("debug_{}.ron", chrono::Utc::now().format("%Y%m%d_%H%M%S"))
                };
                match self.save_debug_state(&filename) {
                    Ok(_) => self.log(format!("Debug state saved: {}", filename)),
                    Err(e) => self.log(format!("Failed to save debug state: {}", e)),
                }
            }
            Some("load_debug") => {
                if let Some(filename) = parts.get(1) {
                    match Self::load_debug_state(filename) {
                        Ok(state) => {
                            *self = state;
                            self.log(format!("Debug state loaded: {}", filename));
                        }
                        Err(e) => self.log(format!("Failed to load debug state: {}", e)),
                    }
                } else {
                    self.log("Usage: load_debug <filename>");
                }
            }
            Some("list_debug") => {
                match Self::list_debug_states() {
                    Ok(states) => {
                        if states.is_empty() {
                            self.log("No debug states found");
                        } else {
                            self.log("Debug states:");
                            for state in states {
                                self.log(format!("  {}", state));
                            }
                        }
                    }
                    Err(e) => self.log(format!("Failed to list debug states: {}", e)),
                }
            }
            Some("debug_info") => {
                let info = self.get_debug_info();
                self.log(format!("Turn: {} | Pos: ({},{}) | HP: {}/{}", 
                    info.turn, info.player_pos.0, info.player_pos.1, 
                    info.player_hp.0, info.player_hp.1));
                self.log(format!("Enemies: {} | Items: {} | Storm: {}/{}", 
                    info.enemies_count, info.items_count, 
                    info.storm_intensity, info.storm_turns));
                self.log(format!("Seed: {} | Memory: {}", info.seed, info.memory_usage));
            }
            Some("run_des") => {
                if let Some(filename) = parts.get(1) {
                    match super::des_testing::run_des_test_file(filename) {
                        Ok(result) => {
                            self.log(format!("DES Test '{}': {}", result.test_name, 
                                if result.passed { "PASSED" } else { "FAILED" }));
                            for log_entry in result.execution_log {
                                self.log(format!("  {}", log_entry));
                            }
                            if !result.failed_expectations.is_empty() {
                                self.log("Failed expectations:");
                                for failure in result.failed_expectations {
                                    self.log(format!("  - {}", failure));
                                }
                            }
                        }
                        Err(e) => self.log(format!("DES test failed: {}", e)),
                    }
                } else {
                    self.log("Usage: run_des <filename>");
                }
            }
            Some("list_des") => {
                match super::des_testing::list_des_tests() {
                    Ok(tests) => {
                        if tests.is_empty() {
                            self.log("No DES test files found");
                        } else {
                            self.log("Available DES tests:");
                            for test in tests {
                                self.log(format!("  {}", test));
                            }
                        }
                    }
                    Err(e) => self.log(format!("Failed to list DES tests: {}", e)),
                }
            }
            Some("create_sample_des") => {
                match super::des_testing::save_sample_des_test() {
                    Ok(()) => self.log("Sample DES test created: tests/sample_test.des"),
                    Err(e) => self.log(format!("Failed to create sample DES test: {}", e)),
                }
            }
            Some("spawn") => {
                if let Some(ui_type) = parts.get(1) {
                    match ui_type {
                        &"log" => {
                            match crate::terminal_spawn::spawn_terminal_window("log-ui") {
                                Ok(()) => self.log("Spawned log terminal"),
                                Err(e) => self.log(format!("Failed to spawn log terminal: {}", e)),
                            }
                        }
                        &"status" => {
                            match crate::terminal_spawn::spawn_terminal_window("status-ui") {
                                Ok(()) => self.log("Spawned status terminal"),
                                Err(e) => self.log(format!("Failed to spawn status terminal: {}", e)),
                            }
                        }
                        &"inventory" => {
                            match crate::terminal_spawn::spawn_terminal_window("inventory-ui") {
                                Ok(()) => self.log("Spawned inventory terminal"),
                                Err(e) => self.log(format!("Failed to spawn inventory terminal: {}", e)),
                            }
                        }
                        &"gamelog" => {
                            match crate::terminal_spawn::spawn_terminal_window("game-log-ui") {
                                Ok(()) => self.log("Spawned game log terminal"),
                                Err(e) => self.log(format!("Failed to spawn game log terminal: {}", e)),
                            }
                        }
                        &"debug" => {
                            match crate::terminal_spawn::spawn_terminal_window("debug-ui") {
                                Ok(()) => self.log("Spawned debug terminal"),
                                Err(e) => self.log(format!("Failed to spawn debug terminal: {}", e)),
                            }
                        }
                        _ => self.log("Usage: spawn <log|gamelog|status|inventory|debug>"),
                    }
                } else {
                    self.log("Usage: spawn <log|gamelog|status|inventory|debug>");
                }
            }
            Some("terminals") => {
                let available = crate::terminal_spawn::get_available_terminals();
                if available.is_empty() {
                    self.log("No supported terminal emulators found");
                } else {
                    self.log(format!("Available terminals: {}", available.join(", ")));
                }
            }
            Some("report_issue") => {
                self.log("Issue reporting mode activated. Use UI to file report.");
                // This will be handled by the UI
            }
            Some("add_adaptation") => {
                if let Some(id) = parts.get(1) {
                    if let Some(adaptation) = super::adaptation::Adaptation::from_id(id) {
                        if !self.adaptations.contains(&adaptation) {
                            self.adaptations.push(adaptation);
                            self.log(format!("Added adaptation: {}", adaptation.name()));
                        } else {
                            self.log(format!("Already have adaptation: {}", adaptation.name()));
                        }
                    } else {
                        self.log(format!("Unknown adaptation: {}", id));
                    }
                } else {
                    self.log("Usage: add_adaptation <id>");
                }
            }
            Some("list_adaptations") => {
                self.log("Available adaptations:");
                for id in super::adaptation::all_adaptation_ids() {
                    if let Some(def) = super::adaptation::get_adaptation_def(id) {
                        let has = self.adaptations.iter().any(|a| a.id() == id);
                        self.log(format!("  {} - {} {}", id, def.name, if has { "[HAVE]" } else { "" }));
                    }
                }
            }
            Some("add_psychic") => {
                if let Some(id) = parts.get(1) {
                    if super::psychic::get_ability_def(id).is_some() {
                        if !self.psychic.unlocked_abilities.contains(&id.to_string()) {
                            self.psychic.unlocked_abilities.push(id.to_string());
                            self.log(format!("Added psychic ability: {}", id));
                        } else {
                            self.log(format!("Already have psychic ability: {}", id));
                        }
                    } else {
                        self.log(format!("Unknown psychic ability: {}", id));
                    }
                } else {
                    self.log("Usage: add_psychic <id>");
                }
            }
            Some("list_psychic") => {
                self.log("Available psychic abilities:");
                for id in super::psychic::all_ability_ids() {
                    if let Some(def) = super::psychic::get_ability_def(id) {
                        let has = self.psychic.unlocked_abilities.contains(&id.to_string());
                        self.log(format!("  {} - {} (Cost: {}) {}", id, def.name, def.coherence_cost, if has { "[HAVE]" } else { "" }));
                    }
                }
            }
            Some("set_coherence") => {
                if let Some(amount_str) = parts.get(1) {
                    if let Ok(amount) = amount_str.parse::<u32>() {
                        self.psychic.coherence = amount;
                        self.psychic.max_coherence = amount.max(self.psychic.max_coherence);
                        self.log(format!("Set coherence to: {}", amount));
                    } else {
                        self.log("Invalid amount. Use a number.");
                    }
                } else {
                    self.log("Usage: set_coherence <amount>");
                }
            }
            Some("add_skill_points") => {
                if let Some(amount_str) = parts.get(1) {
                    if let Ok(amount) = amount_str.parse::<u32>() {
                        self.skills.skill_points += amount;
                        self.log(format!("Added {} skill points (total: {})", amount, self.skills.skill_points));
                    } else {
                        self.log("Invalid amount. Use a number.");
                    }
                } else {
                    self.skills.skill_points += 10;
                    self.log(format!("Added 10 skill points (total: {})", self.skills.skill_points));
                }
            }
            Some("set_stamina") => {
                if let Some(amount_str) = parts.get(1) {
                    if let Ok(amount) = amount_str.parse::<u32>() {
                        self.skills.stamina = amount;
                        self.skills.max_stamina = amount.max(self.skills.max_stamina);
                        self.log(format!("Set stamina to: {}", amount));
                    } else {
                        self.log("Invalid amount. Use a number.");
                    }
                } else {
                    self.log("Usage: set_stamina <amount>");
                }
            }
            Some("unlock_skill") => {
                if let Some(skill_id) = parts.get(1) {
                    if super::skills::get_skill_def(skill_id).is_some() {
                        self.skills.skills.insert(skill_id.to_string(), 1);
                        self.skills.check_ability_unlocks();
                        self.log(format!("Unlocked skill: {}", skill_id));
                    } else {
                        self.log(format!("Unknown skill: {}", skill_id));
                    }
                } else {
                    self.log("Usage: unlock_skill <skill_id>");
                }
            }
            Some("list_skills") => {
                self.log("Available skills:");
                for id in super::skills::all_skill_ids() {
                    if let Some(def) = super::skills::get_skill_def(id) {
                        let level = self.skills.get_skill_level(id);
                        self.log(format!("  {} - {} (Lv.{})", id, def.name, level));
                    }
                }
            }
            Some("list_abilities") => {
                self.log("Available abilities:");
                for id in super::skills::all_ability_ids() {
                    if let Some(def) = super::skills::get_ability_def(id) {
                        let unlocked = self.skills.unlocked_abilities.contains(&id.to_string());
                        self.log(format!("  {} - {} {}", id, def.name, if unlocked { "[UNLOCKED]" } else { "" }));
                    }
                }
            }
            Some("focus_beam") => {
                if parts.len() >= 2 {
                    let direction = match parts[1] {
                        "n" | "north" => super::light::Direction::North,
                        "s" | "south" => super::light::Direction::South,
                        "e" | "east" => super::light::Direction::East,
                        "w" | "west" => super::light::Direction::West,
                        "ne" => super::light::Direction::NorthEast,
                        "nw" => super::light::Direction::NorthWest,
                        "se" => super::light::Direction::SouthEast,
                        "sw" => super::light::Direction::SouthWest,
                        _ => {
                            self.log("Invalid direction. Use: n, s, e, w, ne, nw, se, sw");
                            return;
                        }
                    };
                    
                    if self.light_system.focus_beam(self.player_x, self.player_y, direction, 10) {
                        self.log("You focus a beam of light!");
                    } else {
                        self.log("Not enough light energy!");
                    }
                } else {
                    self.log("Usage: focus_beam <direction>");
                }
            }
            Some("create_prism") => {
                if parts.len() >= 3 {
                    if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                        if self.light_system.create_prism(x, y, 20) {
                            self.log(format!("Created light prism at ({}, {})", x, y));
                        } else {
                            self.log("Not enough light energy!");
                        }
                    } else {
                        self.log("Invalid coordinates");
                    }
                } else {
                    self.log("Usage: create_prism <x> <y>");
                }
            }
            Some("add_light_energy") => {
                let amount = if parts.len() >= 2 {
                    parts[1].parse::<u32>().unwrap_or(50)
                } else {
                    50
                };
                self.light_system.light_energy += amount;
                self.log(format!("Added {} light energy (total: {})", amount, self.light_system.light_energy));
            }
            Some("absorb_light") => {
                let absorbed = self.light_system.absorb_light(self.player_x, self.player_y, &self.map);
                if absorbed > 0 {
                    self.log(format!("Absorbed {} light energy", absorbed));
                } else {
                    self.log("No light to absorb here");
                }
            }
            Some("add_void_exposure") => {
                let amount = if parts.len() >= 2 {
                    parts[1].parse::<u32>().unwrap_or(10)
                } else {
                    10
                };
                let level_changed = self.void_system.add_exposure(amount);
                self.log(format!("Added {} void exposure (total: {})", amount, self.void_system.void_exposure));
                if level_changed {
                    self.log(format!("Void exposure level: {:?}", self.void_system.exposure_level()));
                }
            }
            Some("add_void_energy") => {
                let amount = if parts.len() >= 2 {
                    parts[1].parse::<u32>().unwrap_or(25)
                } else {
                    25
                };
                self.void_system.gain_energy(amount);
                self.log(format!("Added {} void energy (total: {}/{})", 
                    amount, self.void_system.void_energy, self.void_system.max_void_energy));
            }
            Some("void_step") => {
                if parts.len() >= 3 {
                    if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                        if self.void_system.void_step(self.player_x, self.player_y, x, y) {
                            self.player_x = x;
                            self.player_y = y;
                            self.log("You step through the void!");
                        } else {
                            self.log("Cannot void step (insufficient energy or too far)");
                        }
                    } else {
                        self.log("Invalid coordinates");
                    }
                } else {
                    self.log("Usage: void_step <x> <y>");
                }
            }
            Some("reality_rend") => {
                if parts.len() >= 3 {
                    if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                        if let Some(damage) = self.void_system.reality_rend(x, y) {
                            self.log(format!("Reality rend deals {} void damage!", damage));
                        } else {
                            self.log("Cannot use reality rend (insufficient energy or not unlocked)");
                        }
                    } else {
                        self.log("Invalid coordinates");
                    }
                } else {
                    self.log("Usage: reality_rend <x> <y>");
                }
            }
            Some("create_crystal") => {
                if parts.len() >= 4 {
                    if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                        let frequency = match parts[3] {
                            "alpha" => super::crystal_resonance::CrystalFrequency::Alpha,
                            "beta" => super::crystal_resonance::CrystalFrequency::Beta,
                            "gamma" => super::crystal_resonance::CrystalFrequency::Gamma,
                            "delta" => super::crystal_resonance::CrystalFrequency::Delta,
                            "epsilon" => super::crystal_resonance::CrystalFrequency::Epsilon,
                            _ => {
                                self.log("Invalid frequency. Use: alpha, beta, gamma, delta, epsilon");
                                return;
                            }
                        };
                        
                        if self.crystal_system.create_crystal_seed(x, y, frequency, 20) {
                            self.log(format!("Created {} crystal at ({}, {})", frequency.name(), x, y));
                        } else {
                            self.log("Not enough resonance energy!");
                        }
                    } else {
                        self.log("Invalid coordinates");
                    }
                } else {
                    self.log("Usage: create_crystal <x> <y> <frequency>");
                }
            }
            Some("resonate") => {
                let energy = self.crystal_system.resonate(self.player_x, self.player_y);
                if energy > 0 {
                    self.log(format!("Resonated with crystals, gained {} energy", energy));
                } else {
                    self.log("No crystals to resonate with here");
                }
            }
            Some("add_resonance_energy") => {
                let amount = if parts.len() >= 2 {
                    parts[1].parse::<u32>().unwrap_or(30)
                } else {
                    30
                };
                self.crystal_system.resonance_energy = (self.crystal_system.resonance_energy + amount)
                    .min(self.crystal_system.max_resonance_energy);
                self.log(format!("Added {} resonance energy (total: {}/{})", 
                    amount, self.crystal_system.resonance_energy, self.crystal_system.max_resonance_energy));
            }
            Some("harmonize") => {
                if self.crystal_system.harmonize(self.player_x, self.player_y, 3, 40) {
                    self.log("Created harmonic resonance!");
                } else {
                    self.log("Cannot harmonize (insufficient energy or crystals)");
                }
            }
            Some("spawn_enemy") => {
                if let Some(id) = parts.get(1) {
                    if super::enemy::get_enemy_def(id).is_some() {
                        let x = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(self.player_x + 1);
                        let y = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(self.player_y);
                        
                        if self.map.get(x, y).map(|t| t.walkable()).unwrap_or(false) {
                            let enemy = super::enemy::Enemy::new(x, y, id);
                            self.enemies.push(enemy);
                            self.log(format!("Spawned {} at ({}, {})", id, x, y));
                        } else {
                            self.log("Cannot spawn at that location (not walkable)");
                        }
                    } else {
                        self.log(format!("Unknown enemy: {}", id));
                    }
                } else {
                    self.log("Usage: spawn_enemy <id> [x] [y]");
                }
            }
            Some("spawn_swarm") => {
                if let Some(id) = parts.get(1) {
                    if let Some(count_str) = parts.get(2) {
                        if let Ok(count) = count_str.parse::<u32>() {
                            if super::enemy::get_enemy_def(id).is_some() {
                                let swarm_id = format!("swarm_{}", self.turn);
                                let mut spawned = 0;
                                
                                for dx in -2..=2 {
                                    for dy in -2..=2 {
                                        if spawned >= count { break; }
                                        let x = self.player_x + dx;
                                        let y = self.player_y + dy;
                                        
                                        if self.map.get(x, y).map(|t| t.walkable()).unwrap_or(false) {
                                            let enemy = super::enemy::Enemy::new_swarm_member(x, y, id, swarm_id.clone(), spawned == 0);
                                            self.enemies.push(enemy);
                                            spawned += 1;
                                        }
                                    }
                                    if spawned >= count { break; }
                                }
                                
                                self.log(format!("Spawned swarm of {} {} (count: {})", spawned, id, count));
                            } else {
                                self.log(format!("Unknown enemy: {}", id));
                            }
                        } else {
                            self.log("Invalid count. Use a number.");
                        }
                    } else {
                        self.log("Usage: spawn_swarm <id> <count>");
                    }
                } else {
                    self.log("Usage: spawn_swarm <id> <count>");
                }
            }
            Some("spawn_npc") => {
                if let Some(id) = parts.get(1) {
                    if super::npc::get_npc_def(id).is_some() {
                        let x = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(self.player_x + 1);
                        let y = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(self.player_y);
                        
                        if self.map.get(x, y).map(|t| t.walkable()).unwrap_or(false) {
                            let npc = super::npc::Npc::new(x, y, id);
                            self.npcs.push(npc);
                            self.spatial_dirty = true; // Mark spatial index as dirty
                            self.log(format!("Spawned NPC {} at ({}, {}) - Total NPCs: {}", id, x, y, self.npcs.len()));
                        } else {
                            self.log("Cannot spawn at that location (not walkable)");
                        }
                    } else {
                        self.log(format!("Unknown NPC: {}", id));
                    }
                } else {
                    self.log("Usage: spawn_npc <id> [x] [y]");
                }
            }
            Some("list_npcs") => {
                self.log("Available NPCs:");
                for id in super::npc::all_npc_ids() {
                    if let Some(def) = super::npc::get_npc_def(id) {
                        self.log(format!("  {} - {} ({})", id, def.name, def.glyph));
                    }
                }
            }
            Some("show_npcs") => {
                if self.npcs.is_empty() {
                    self.log("No NPCs currently spawned");
                } else {
                    let npc_info: Vec<String> = self.npcs.iter().enumerate()
                        .map(|(i, npc)| format!("  {}: {} at ({}, {})", i, npc.id, npc.x, npc.y))
                        .collect();
                    self.log(format!("Current NPCs ({}): ", self.npcs.len()));
                    for info in npc_info {
                        self.log(info);
                    }
                }
            }
            Some("give_item") => {
                if let Some(id) = parts.get(1) {
                    if super::item::get_item_def(id).is_some() {
                        let count = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
                        for _ in 0..count {
                            self.inventory.push(id.to_string());
                        }
                        self.log(format!("Added {} x{} to inventory", id, count));
                    } else {
                        self.log(format!("Unknown item: {}", id));
                    }
                } else {
                    self.log("Usage: give_item <id> [count]");
                }
            }
            Some("show_level") => {
                let level = self.get_current_tile_level();
                let threat_desc = match level {
                    1 => "Safe",
                    2..=3 => "Low Threat",
                    4..=6 => "Medium Threat",
                    7..=8 => "High Threat",
                    9..=10 => "EXTREME THREAT",
                    _ => "Unknown Threat",
                };
                self.log(format!("Current tile level: {} ({})", level, threat_desc));
            }
            Some("show_item_tiers") => {
                self.log("Items by tier:");
                for tier in 1..=5 {
                    self.log(format!("Tier {} items:", tier));
                    for id in super::item::all_item_ids() {
                        if let Some(def) = super::item::get_item_def(id) {
                            if def.tier == tier {
                                self.log(format!("  {} - {} (value: {})", id, def.name, def.value));
                            }
                        }
                    }
                }
            }
            Some("help") => {
                self.log("Debug Commands:");
                self.log("  show tile, hide tile - Toggle god view");
                self.log("  sturdy - Set HP to 9999");
                self.log("  phase - Toggle wall phasing");
                self.log("  save_debug [name] - Save debug state");
                self.log("  load_debug <name> - Load debug state");
                self.log("  list_debug - List saved debug states");
                self.log("  debug_info - Show debug information");
                self.log("  report_issue - Open issue reporter");
                self.log("  run_des <file> - Run DES test");
                self.log("  list_des - List DES test files");
                self.log("  create_sample_des - Create sample DES test");
                self.log("  spawn <log|gamelog|status|inventory|debug> - Spawn satellite terminal");
                self.log("  terminals - List available terminal emulators");
                self.log("  add_adaptation <id> - Add adaptation");
                self.log("  list_adaptations - List available adaptations");
                self.log("  add_psychic <id> - Add psychic ability");
                self.log("  list_psychic - List available psychic abilities");
                self.log("  set_coherence <amount> - Set psychic coherence");
                self.log("  add_skill_points [amount] - Add skill points");
                self.log("  set_stamina <amount> - Set stamina");
                self.log("  unlock_skill <id> - Unlock a skill");
                self.log("  list_skills - List available skills");
                self.log("  list_abilities - List available abilities");
                self.log("  focus_beam <direction> - Create light beam (costs 10 energy)");
                self.log("  create_prism <x> <y> - Create refraction surface (costs 20 energy)");
                self.log("  add_light_energy [amount] - Add light energy");
                self.log("  absorb_light - Absorb light from nearby sources");
                self.log("  add_void_exposure [amount] - Add void exposure");
                self.log("  add_void_energy [amount] - Add void energy");
                self.log("  void_step <x> <y> - Teleport through void (costs 15 energy)");
                self.log("  reality_rend <x> <y> - Void damage attack (costs 25 energy)");
                self.log("  create_crystal <x> <y> <frequency> - Create crystal (costs 20 energy)");
                self.log("  resonate - Resonate with nearby crystals");
                self.log("  add_resonance_energy [amount] - Add resonance energy");
                self.log("  harmonize - Create harmonic effect (costs 40 energy)");
                self.log("  spawn_enemy <id> [x] [y] - Spawn enemy at position");
                self.log("  spawn_swarm <id> <count> - Spawn enemy swarm");
                self.log("  spawn_npc <id> [x] [y] - Spawn NPC at position");
                self.log("  list_npcs - List available NPCs");
                self.log("  show_npcs - Show currently spawned NPCs");
                self.log("  give_item <id> [count] - Add item to inventory");
                self.log("  show_level - Show current tile threat level");
                self.log("  show_item_tiers - Show items organized by tier");
                self.log("");
                self.log("Console Controls:");
                self.log("  ` - Toggle debug console");
                self.log("  Up/Down - Navigate command history");
                self.log("  Tab - Accept current suggestion");
                self.log("  Left/Right - Navigate suggestions");
                self.log("  Esc - Close console");
            }
            _ => self.log(format!("Unknown command: {}. Type 'help' for commands.", cmd)),
        }
    }

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
        self.hit_flash_positions.iter().any(|(fx, fy, _)| *fx == x && *fy == y)
    }

    /// Spawn a floating damage number
    pub fn spawn_damage_number(&mut self, x: i32, y: i32, value: i32, is_heal: bool) {
        self.damage_numbers.push(DamageNumber { x, y, value, frames: 12, is_heal });
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
    pub fn spawn_beam(&mut self, from: (i32, i32), to: (i32, i32), beam_type: BeamType, duration: u32) {
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
                    // Determine beam character based on direction
                    let dx = beam.end_x - beam.start_x;
                    let dy = beam.end_y - beam.start_y;
                    let char = if dx.abs() > dy.abs() {
                        '-' // Horizontal beam
                    } else if dy.abs() > dx.abs() {
                        '|' // Vertical beam
                    } else if (dx > 0 && dy > 0) || (dx < 0 && dy < 0) {
                        '\\' // Diagonal beam
                    } else {
                        '/' // Other diagonal
                    };
                    return Some((char, beam.beam_type.clone()));
                }
            }
        }
        None
    }

    /// Generate visual effects based on player adaptations
    pub fn get_adaptation_visual_effects(&self) -> Vec<super::effect::VisualEffect> {
        use super::effect::VisualEffect;
        let mut effects = Vec::new();
        
        for adaptation in &self.adaptations {
            match adaptation.name() {
                "Prismhide" => {
                    // Crystalline shimmer effect
                    effects.push(VisualEffect::Shimmer {
                        speed: 6,
                        colors: vec![Color::Cyan, Color::LightCyan, Color::White],
                    });
                }
                "Sunveins" => {
                    // Pulsing inner light
                    effects.push(VisualEffect::Pulse {
                        speed: 4,
                        color: Color::Yellow,
                    });
                }
                "Mirage Step" => {
                    // Flickering/fading effect
                    effects.push(VisualEffect::Fade {
                        speed: 8,
                        color: Color::LightBlue,
                    });
                }
                "Saltblood" => {
                    // Subtle white glow
                    effects.push(VisualEffect::Glow {
                        color: Color::White,
                    });
                }
                "Quantum Entanglement" => {
                    // Rainbow psychic aura
                    effects.push(VisualEffect::Rainbow {
                        speed: 5,
                        colors: vec![Color::Magenta, Color::Cyan, Color::Yellow, Color::Green],
                    });
                }
                "Phase Walking" => {
                    // Drifting translucent effect
                    effects.push(VisualEffect::Drift {
                        speed: 7,
                        color: Color::LightMagenta,
                    });
                }
                "Storm Affinity" => {
                    // Storm-like wave effect
                    effects.push(VisualEffect::Wave {
                        speed: 3,
                        color: Color::LightCyan,
                    });
                }
                "Crystalline Consciousness" => {
                    // Complex multi-effect for transcendent adaptation
                    effects.push(VisualEffect::Rainbow {
                        speed: 2,
                        colors: vec![Color::White, Color::LightCyan, Color::LightMagenta, Color::LightYellow],
                    });
                    effects.push(VisualEffect::Pulse {
                        speed: 3,
                        color: Color::White,
                    });
                }
                _ => {} // No visual effect for other adaptations
            }
        }
        
        effects
    }


    pub fn check_adaptation_threshold(&mut self) {
        // Get all available adaptations sorted by threshold
        let mut available: Vec<(&str, u32)> = super::adaptation::all_adaptation_ids()
            .iter()
            .filter_map(|&id| {
                super::adaptation::get_adaptation_def(id).map(|def| (id, def.threshold))
            })
            .filter(|(id, _)| {
                !self.adaptations.iter().any(|a| a.id() == *id)
            })
            .collect();
        
        available.sort_by_key(|(_, threshold)| *threshold);
        
        // Find first unlockable adaptation
        if let Some(&(adaptation_id, _threshold)) = available.iter().find(|(_, t)| self.refraction >= *t) {
            if let Some(adaptation) = super::adaptation::Adaptation::from_id(adaptation_id) {
                self.adaptations.push(adaptation);
                self.emit(GameEvent::AdaptationGained { name: adaptation.name().to_string() });
                self.log(format!("🧬 You gain {}!", adaptation.name()));
            }
        }
    }

    pub fn has_adaptation(&self, a: Adaptation) -> bool {
        self.adaptations.contains(&a)
    }

    pub fn enemy_at(&self, x: i32, y: i32) -> Option<usize> {
        self.enemy_positions.get(&(x, y)).copied()
    }

    pub fn npc_at(&self, x: i32, y: i32) -> Option<usize> {
        self.npc_positions.get(&(x, y)).copied()
    }

    /// Check if there's a decoy at position
    pub fn decoy_at(&self, x: i32, y: i32) -> bool {
        self.decoys.iter().any(|d| d.x == x && d.y == y)
    }

    /// Auto-explore: find nearest unexplored walkable tile and move toward it
    /// Enhanced with item pickup, danger avoidance, and enemy detection
    pub fn auto_explore(&mut self) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;
        
        let config = get_auto_explore_config();
        
        // Check for nearby enemies first
        if config.stop_on_enemies && self.has_nearby_enemies(config.enemy_detection_range) {
            self.messages.push(crate::game::GameMessage::new(
                "Auto-explore stopped: enemy detected nearby".to_string(),
                crate::game::MsgType::System,
                self.turn
            ));
            return false;
        }
        
        // Pick up items at current position if configured
        if config.pickup_items {
            self.pickup_filtered_items();
        }
        
        let start = self.map.idx(self.player_x, self.player_y);
        
        // BFS to find nearest item or unexplored walkable tile
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, vec![start]));
        visited.insert(start);
        
        let target = loop {
            let (idx, path) = match queue.pop_front() {
                Some(p) => p,
                None => return false, // No items or unexplored tiles reachable
            };
            
            let mut found_target = false;
            
            // Check if this tile has items we want to pick up
            if config.pickup_items {
                let x = (idx % self.map.width) as i32;
                let y = (idx / self.map.width) as i32;
                
                for item in &self.items {
                    if item.x == x && item.y == y && config.should_pickup_item(&item.id) {
                        found_target = true;
                        break;
                    }
                }
            }
            
            // Check if this tile has an NPC we haven't talked to yet
            if !found_target {
                let x = (idx % self.map.width) as i32;
                let y = (idx / self.map.width) as i32;
                
                for npc in &self.npcs {
                    if npc.x == x && npc.y == y && !npc.talked {
                        found_target = true;
                        break;
                    }
                }
            }
            
            // Check if this tile is unexplored
            if !found_target && !self.revealed.contains(&idx) {
                found_target = true;
            }
            
            if found_target {
                // Return the first step toward this tile
                if path.len() > 1 {
                    break Some(path[1]);
                }
                return false;
            }
            
            // Add neighbors, but avoid dangerous tiles if configured
            for (next_idx, _) in self.map.get_available_exits(idx) {
                if !visited.contains(&next_idx) {
                    // Check if we should avoid this tile due to dangers
                    if config.avoid_dangers && self.is_dangerous_tile(next_idx) {
                        continue;
                    }
                    
                    // Check if there's an NPC we've already talked to on this tile
                    if self.has_talked_npc_at_idx(next_idx) {
                        continue;
                    }
                    
                    visited.insert(next_idx);
                    let mut new_path = path.clone();
                    new_path.push(next_idx);
                    queue.push_back((next_idx, new_path));
                }
            }
        };
        
        if let Some(next_idx) = target {
            let nx = (next_idx % self.map.width) as i32;
            let ny = (next_idx / self.map.width) as i32;
            let dx = nx - self.player_x;
            let dy = ny - self.player_y;
            
            // Final check: don't move to dangerous tile
            let target_x = self.player_x + dx;
            let target_y = self.player_y + dy;
            let target_idx = self.map.idx(target_x, target_y);
            
            if config.avoid_dangers && self.is_dangerous_tile(target_idx) {
                return false;
            }
            
            self.try_move(dx, dy)
        } else {
            false
        }
    }
    
    /// Check if there are enemies nearby within the given range
    fn has_nearby_enemies(&self, range: i32) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();
        
        for enemy in &self.enemies {
            let distance = (enemy.x - self.player_x).abs() + (enemy.y - self.player_y).abs();
            if distance <= range {
                // If ignoring weak enemies, check enemy HP
                if config.ignore_weak_enemies {
                    if enemy.hp <= config.weak_enemy_threshold {
                        continue;
                    }
                }
                return true;
            }
        }
        false
    }
    
    /// Check if a tile is dangerous based on configuration
    fn is_dangerous_tile(&self, idx: usize) -> bool {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();
        
        let tile = &self.map.tiles[idx];
        
        // Check for glass tiles
        if config.is_danger_type("glass") && matches!(tile, crate::game::map::Tile::Glass) {
            return true;
        }
        
        // Check for glare tiles
        if config.is_danger_type("glare") && matches!(tile, crate::game::map::Tile::Glare) {
            return true;
        }
        
        false
    }
    
    /// Check if there's an NPC we've already talked to at the given tile index
    fn has_talked_npc_at_idx(&self, idx: usize) -> bool {
        let x = (idx % self.map.width) as i32;
        let y = (idx / self.map.width) as i32;
        
        self.npcs.iter().any(|npc| npc.x == x && npc.y == y && npc.talked)
    }
    
    /// Pick up items at current position, filtered by configuration
    fn pickup_filtered_items(&mut self) {
        use crate::game::auto_explore::get_auto_explore_config;
        let config = get_auto_explore_config();
        
        let player_idx = self.map.idx(self.player_x, self.player_y);
        let mut items_to_remove = Vec::new();
        
        for (i, item) in self.items.iter().enumerate() {
            let item_idx = self.map.idx(item.x, item.y);
            if item_idx == player_idx && config.should_pickup_item(&item.id) {
                items_to_remove.push(i);
            }
        }
        
        // Remove items in reverse order to maintain indices
        for &i in items_to_remove.iter().rev() {
            let item = self.items.remove(i);
            self.inventory.push(item.id.clone());
            self.messages.push(crate::game::GameMessage::new(
                format!("Picked up {}", item.name()),
                crate::game::MsgType::System,
                self.turn
            ));
        }
    }

    /// Move player by delta - delegates to MovementSystem
    pub fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        MovementSystem::try_move(self, dx, dy)
    }

    /// Pickup items at player position - delegates to MovementSystem
    pub fn pickup_items(&mut self) {
        MovementSystem::pickup_items(self)
    }

    pub fn can_open_chest(&self, chest_index: usize) -> bool {
        if chest_index >= self.chests.len() {
            return false;
        }
        
        let chest = &self.chests[chest_index];
        let player_pos = (self.player_x, self.player_y);
        let chest_pos = (chest.x, chest.y);
        
        // Check if player is adjacent to chest
        let dx = (player_pos.0 - chest_pos.0).abs();
        let dy = (player_pos.1 - chest_pos.1).abs();
        dx <= 1 && dy <= 1 && (dx + dy) > 0 // Adjacent but not same position
    }

    pub fn open_chest(&mut self, chest_index: usize) -> bool {
        if !self.can_open_chest(chest_index) {
            return false;
        }
        
        // Check if chest is locked and handle unlocking
        let chest_id = self.chests[chest_index].id.clone();
        let is_locked = self.chests[chest_index].is_locked();
        
        if is_locked {
            if let Some(def) = super::chest::get_chest_def(&chest_id) {
                if let Some(key_id) = &def.key_required {
                    if self.inventory.contains(key_id) {
                        self.chests[chest_index].unlock();
                        self.log(format!("Unlocked {} with {}.", def.name, key_id));
                    } else {
                        self.log(format!("{} is locked. You need a {}.", def.name, key_id));
                        return false;
                    }
                }
            }
        }
        
        self.chests[chest_index].opened = true;
        let def = super::chest::get_chest_def(&chest_id);
        let name = def.map(|d| d.name.as_str()).unwrap_or("chest");
        self.log(format!("Opened {}.", name));
        true
    }

    pub fn transfer_to_chest(&mut self, chest_index: usize, inventory_index: usize) -> bool {
        if chest_index >= self.chests.len() || inventory_index >= self.inventory.len() {
            return false;
        }
        
        let chest = &mut self.chests[chest_index];
        if !chest.can_add_item() {
            self.log("Chest is full.");
            return false;
        }
        
        let item_id = self.inventory.remove(inventory_index);
        let item = Item::new(chest.x, chest.y, &item_id);
        chest.add_item(item);
        
        let item_def = super::item::get_item_def(&item_id);
        let name = item_def.map(|d| d.name.as_str()).unwrap_or(&item_id);
        self.log(format!("Stored {} in chest.", name));
        true
    }

    pub fn transfer_from_chest(&mut self, chest_index: usize, chest_item_index: usize) -> bool {
        if chest_index >= self.chests.len() {
            return false;
        }
        
        let chest = &mut self.chests[chest_index];
        if let Some(item) = chest.remove_item(chest_item_index) {
            self.inventory.push(item.id.clone());
            
            let item_def = super::item::get_item_def(&item.id);
            let name = item_def.map(|d| d.name.as_str()).unwrap_or(&item.id);
            self.log(format!("Took {} from chest.", name));
            true
        } else {
            false
        }
    }

    pub fn use_item(&mut self, idx: usize) -> bool {
        if idx >= self.inventory.len() { return false; }
        let cost = action_cost("use_item");
        if self.player_ap < cost { return false; }
        let id = &self.inventory[idx];
        let def = match get_item_def(id) {
            Some(d) => d,
            None => return false,
        };
        if !def.usable {
            self.log(format!("You can't use {} right now.", def.name));
            return false;
        }
        
        // Check if it's a book
        if let Some(book_id) = &def.book_id {
            self.pending_book_open = Some(book_id.clone());
            self.log(format!("You read {}.", def.name));
            return true;
        }

        self.player_ap -= cost;
        if def.heal > 0 {
            let heal = def.heal.min(self.player_max_hp - self.player_hp);
            self.player_hp += heal;
            self.log_typed(format!("You use {}. (+{} HP)", def.name, heal), MsgType::Loot);
        }
        if def.reduces_refraction > 0 {
            let reduce = def.reduces_refraction.min(self.refraction);
            self.refraction -= reduce;
            self.log_typed(format!("Your glow fades slightly. (-{} Refraction)", reduce), MsgType::Status);
        }
        if def.suppresses_adaptations {
            self.adaptations_hidden_turns = 10;
            self.log_typed("Your glow dims. The tincture masks your changes.", MsgType::Status);
        }
        if def.reveals_map {
            self.log_typed(format!("The {} reveals hidden paths...", def.name), MsgType::Loot);
            for idx in 0..self.map.tiles.len() {
                self.revealed.insert(idx);
            }
        }
        if def.enables_aria_dialogue {
            self.log_typed("You interface with ARIA...", MsgType::System);
            self.quest_log.on_aria_interfaced(&def.id);
            // Trigger ARIA dialogue if we have a pending dialogue system
            // For now, we just log it.
        }
        
        // New system integrations
        if def.light_energy > 0 {
            self.light_system.light_energy += def.light_energy;
            self.log_typed(format!("Light energy surges through you! (+{} Light Energy)", def.light_energy), MsgType::Status);
        }
        if def.teaches_light_manipulation {
            self.log_typed("You learn to manipulate light! Use debug commands: focus_beam, create_prism", MsgType::System);
        }
        if def.void_exposure > 0 {
            let level_changed = self.void_system.add_exposure(def.void_exposure);
            self.log_typed(format!("Void corruption seeps into you! (+{} Void Exposure)", def.void_exposure), MsgType::Status);
            if level_changed {
                self.log_typed(format!("Void exposure level: {:?}", self.void_system.exposure_level()), MsgType::Status);
            }
        }
        if def.void_energy > 0 {
            self.void_system.gain_energy(def.void_energy);
            self.log_typed(format!("Void energy flows through you! (+{} Void Energy)", def.void_energy), MsgType::Status);
        }
        if def.teaches_crystal_resonance {
            self.log_typed("You learn crystal resonance! Use debug commands: create_crystal, resonate, harmonize", MsgType::System);
        }
        if def.resonance_energy > 0 {
            self.crystal_system.resonance_energy = (self.crystal_system.resonance_energy + def.resonance_energy)
                .min(self.crystal_system.max_resonance_energy);
            self.log_typed(format!("Crystal resonance fills you! (+{} Resonance Energy)", def.resonance_energy), MsgType::Status);
        }
        if let Some(frequency) = &def.crystal_frequency {
            let freq = match frequency.as_str() {
                "alpha" => super::crystal_resonance::CrystalFrequency::Alpha,
                "beta" => super::crystal_resonance::CrystalFrequency::Beta,
                "gamma" => super::crystal_resonance::CrystalFrequency::Gamma,
                "delta" => super::crystal_resonance::CrystalFrequency::Delta,
                "epsilon" => super::crystal_resonance::CrystalFrequency::Epsilon,
                _ => super::crystal_resonance::CrystalFrequency::Alpha,
            };
            self.crystal_system.add_crystal(self.player_x, self.player_y, freq);
            self.log_typed(format!("A {} crystal grows at your feet!", frequency), MsgType::Loot);
        }
        
        if def.consumable {
            self.inventory.remove(idx);
        }
        true
    }

    pub fn use_item_on_tile(&mut self, idx: usize, x: i32, y: i32) -> bool {
        if idx >= self.inventory.len() { return false; }
        
        // Check range (must be adjacent)
        let dx = (x - self.player_x).abs();
        let dy = (y - self.player_y).abs();
        if dx > 1 || dy > 1 {
            self.log("That is too far away.");
            return false;
        }

        let cost = action_cost("use_item");
        if self.player_ap < cost { return false; }

        let id = &self.inventory[idx];
        let def = match get_item_def(id) {
            Some(d) => d,
            None => return false,
        };

        if def.breaks_walls {
            let tile_idx = (y * self.map.width as i32 + x) as usize;
            if tile_idx >= self.map.tiles.len() { return false; }

            let is_wall = matches!(self.map.tiles[tile_idx], super::map::Tile::Wall { .. });
            if !is_wall {
                self.log("You can only use this on walls.");
                return false;
            }

            self.player_ap -= cost;
            let mut broken = false;
            if let super::map::Tile::Wall { id: _, hp } = &mut self.map.tiles[tile_idx] {
                *hp -= 10; // Arbitrary damage for now
                if *hp <= 0 {
                    broken = true;
                }
            }
            
            self.log_typed("You strike the wall. Cracks spread through the glass.", MsgType::Combat);

            if broken {
                self.map.tiles[tile_idx] = super::map::Tile::default_floor();
                self.log_typed("The wall shatters!", MsgType::Combat);
                self.update_lighting(); // Wall break changes lighting
            }

            // Consume item if consumable (or maybe always for now as per discussion)
            if def.consumable {
                self.inventory.remove(idx);
            }
            return true;
        }

        self.log(format!("You can't use {} on that.", def.name));
        false
    }

    pub fn use_psychic_ability(&mut self, ability_id: &str) {
        match self.psychic.use_ability(ability_id) {
            Ok(effect_id) => {
                self.log_typed(format!("You use {}.", ability_id), MsgType::Combat);
                // Apply effect
                match effect_id.as_str() {
                    "stun_aoe" => {
                        // Stun nearby enemies
                        let mut stunned_count = 0;
                        for enemy in &mut self.enemies {
                            let dist = ((enemy.x - self.player_x).pow(2) + (enemy.y - self.player_y).pow(2)) as f32;
                            if dist <= 25.0 { // Radius 5
                                enemy.apply_status("stun", 2);
                                stunned_count += 1;
                            }
                        }
                        self.log_typed(format!("Stunned {} enemies.", stunned_count), MsgType::Combat);
                    },
                    "guaranteed_hit" => {
                        self.apply_status_effect("guaranteed_hit", 1);
                    },
                    "phasing" => {
                        self.apply_status_effect("phasing", 5);
                        self.debug_phase = true; // Or handle via status effect check in movement
                    },
                    _ => self.log("Effect not implemented."),
                }
            },
            Err(e) => self.log(e),
        }
    }

    /// Equip an item from inventory to a slot
    pub fn equip_item(&mut self, inv_idx: usize, slot: EquipSlot) -> bool {
        if inv_idx >= self.inventory.len() { return false; }
        let item_id = self.inventory[inv_idx].clone();
        
        // Unequip current item in slot (returns to inventory)
        if let Some(old) = self.equipment.set(slot, Some(item_id)) {
            self.inventory.push(old);
        }
        self.inventory.remove(inv_idx);
        self.recalc_equipment_stats();
        true
    }

    /// Unequip item from slot back to inventory
    pub fn unequip_slot(&mut self, slot: EquipSlot) -> bool {
        if let Some(item) = self.equipment.set(slot, None) {
            self.inventory.push(item);
            self.recalc_equipment_stats();
            true
        } else {
            false
        }
    }

    /// Recalculate stats from equipment
    fn recalc_equipment_stats(&mut self) {
        // Sync equipped_weapon with equipment.weapon for backward compat
        self.equipped_weapon = self.equipment.weapon.clone();
        
        // Calculate armor from equipped jacket item
        self.player_armor = self.equipment.jacket.as_ref()
            .and_then(|id| get_item_def(id))
            .map(|def| def.armor_value)
            .unwrap_or(0);
    }

    /// Accept a quest by ID
    pub fn accept_quest(&mut self, quest_id: &str) -> bool {
        // Check if quest can be accepted (need to do this separately to avoid borrowing issues)
        let can_accept = self.quest_log.is_quest_available(quest_id, self);
        if !can_accept {
            return false;
        }
        
        // Create the quest
        if let Some(quest) = super::quest::ActiveQuest::new(quest_id) {
            self.quest_log.active.push(quest);
            
            if let Some(def) = super::quest::get_quest_def(quest_id) {
                self.log(format!("Quest accepted: {}", def.name));
                
                // Handle faction alignment for main questline
                if def.category == "main" && quest_id.starts_with("faction_choice_") {
                    let faction = if quest_id.contains("monks") {
                        "Mirror Monks"
                    } else if quest_id.contains("engineers") {
                        "Sand-Engineers"
                    } else if quest_id.contains("glassborn") {
                        "Glassborn"
                    } else {
                        return true;
                    };
                    
                    if self.quest_log.set_faction_alignment(faction) {
                        self.log(format!("You have aligned with the {}", faction));
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Complete a quest and receive rewards
    pub fn complete_quest(&mut self, quest_id: &str) -> bool {
        if let Some(reward) = self.quest_log.complete(quest_id) {
            if let Some(def) = super::quest::get_quest_def(quest_id) {
                self.log(format!("Quest completed: {}", def.name));
            }
            if reward.xp > 0 {
                self.gain_xp(reward.xp);
            }
            if reward.salt_scrip > 0 {
                self.salt_scrip += reward.salt_scrip;
                self.log(format!("Received {} salt scrip", reward.salt_scrip));
            }
            for item_id in &reward.items {
                self.inventory.push(item_id.clone());
            }
            // Log unlocked quests
            if !reward.unlocks_quests.is_empty() {
                for unlocked_id in &reward.unlocks_quests {
                    if let Some(unlocked_def) = super::quest::get_quest_def(unlocked_id) {
                        self.log(format!("New quest available: {}", unlocked_def.name));
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Craft an item using a recipe
    pub fn craft(&mut self, recipe_id: &str) -> bool {
        let recipe = match super::crafting::get_recipe(recipe_id) {
            Some(r) => r,
            None => return false,
        };
        
        if !super::crafting::can_craft(recipe, &self.inventory) {
            return false;
        }
        
        // Remove materials
        for (item_id, &count) in &recipe.materials {
            for _ in 0..count {
                if let Some(idx) = self.inventory.iter().position(|id| id == item_id) {
                    self.inventory.remove(idx);
                }
            }
        }
        
        // Add output
        for _ in 0..recipe.output_count {
            self.inventory.push(recipe.output.clone());
        }
        
        self.log(format!("Crafted {}.", recipe.name));
        true
    }

    /// Buy an item from an NPC shop
    pub fn buy_item(&mut self, item_id: &str, npc_id: &str) -> Result<(), String> {
        // Check if NPC exists and has the item in shop
        let npc_def = super::npc::get_npc_def(npc_id)
            .ok_or_else(|| format!("NPC '{}' not found", npc_id))?;
        
        if !npc_def.shop_inventory.contains(&item_id.to_string()) {
            return Err(format!("{} doesn't sell that item", npc_def.name));
        }

        // Get item value
        let item_def = get_item_def(item_id)
            .ok_or_else(|| format!("Item '{}' not found", item_id))?;
        
        let price = item_def.value;
        
        // Check if player has enough currency
        if self.salt_scrip < price {
            return Err(format!("Not enough salt scrip (need {}, have {})", price, self.salt_scrip));
        }

        // Execute transaction
        self.salt_scrip -= price;
        self.inventory.push(item_id.to_string());
        self.log(format!("Bought {} for {} salt scrip", item_def.name, price));
        Ok(())
    }

    /// Sell an item to an NPC
    pub fn sell_item(&mut self, item_id: &str) -> Result<(), String> {
        // Check if player has the item
        let item_idx = self.inventory.iter().position(|id| id == item_id)
            .ok_or_else(|| format!("You don't have that item"))?;

        // Get item value
        let item_def = get_item_def(item_id)
            .ok_or_else(|| format!("Item '{}' not found", item_id))?;
        
        // Sell for half value
        let sell_price = item_def.value / 2;
        
        // Execute transaction
        self.inventory.remove(item_idx);
        self.salt_scrip += sell_price;
        self.log(format!("Sold {} for {} salt scrip", item_def.name, sell_price));
        Ok(())
    }

    /// Get next tutorial message if conditions are met
    pub fn get_next_tutorial_message(&self) -> Option<&super::tutorial::TutorialMessage> {
        self.tutorial_progress.get_next_message(self)
    }

    /// Mark a tutorial message as shown
    pub fn dismiss_tutorial_message(&mut self, message_id: &str) {
        self.tutorial_progress.mark_shown(message_id);
    }

    /// Modify faction reputation (clamped to -100 to +100)
    pub fn modify_reputation(&mut self, faction: &str, delta: i32) {
        let current = self.faction_reputation.get(faction).copied().unwrap_or(0);
        let new_rep = (current + delta).clamp(-100, 100);
        self.faction_reputation.insert(faction.to_string(), new_rep);
        
        if delta != 0 {
            let change_desc = if delta > 0 { "improved" } else { "worsened" };
            self.log_typed(format!("Your reputation with {} has {}.", faction, change_desc), MsgType::Social);
        }
    }

    /// Get faction reputation (0 if not set)
    pub fn get_reputation(&self, faction: &str) -> i32 {
        self.faction_reputation.get(faction).copied().unwrap_or(0)
    }

    /// Add currency to player
    pub fn add_currency(&mut self, amount: u32) {
        self.salt_scrip += amount;
        if amount > 0 {
            self.log_typed(format!("Gained {} salt scrip.", amount), MsgType::Loot);
        }
    }

    /// Try to spend currency, returns true if successful
    pub fn spend_currency(&mut self, amount: u32) -> bool {
        if self.salt_scrip >= amount {
            self.salt_scrip -= amount;
            self.log_typed(format!("Spent {} salt scrip.", amount), MsgType::System);
            true
        } else {
            false
        }
    }

    /// Calculate item price with faction reputation modifier
    pub fn calculate_price(&self, base_price: u32, faction: &str, buying: bool) -> u32 {
        let reputation = self.get_reputation(faction);
        let modifier = 1.0 + (reputation as f32 * -0.002); // -0.2% per reputation point
        let price = (base_price as f32 * modifier) as u32;
        
        if buying {
            price.max(1) // Minimum 1 scrip when buying
        } else {
            (price * 7 / 10).max(1) // Sell for 70% of buy price
        }
    }

    /// Apply status effect to player
    pub fn apply_status_effect(&mut self, effect_id: &str, duration: i32) {
        // Check if effect already exists
        if let Some(existing) = self.status_effects.iter_mut().find(|e| e.id == effect_id) {
            existing.duration = existing.duration.max(duration); // Take longer duration
            existing.add_stack(5); // Max 5 stacks for most effects
        } else {
            self.status_effects.push(super::status::StatusEffect::new(effect_id, duration));
        }
        
        self.log_typed(format!("You are affected by {}.", effect_id), MsgType::Combat);
    }

    /// Check if player has specific status effect
    pub fn has_status_effect(&self, effect_id: &str) -> bool {
        self.status_effects.iter().any(|e| e.id == effect_id)
    }

    /// Process enemy behavior on attack
    pub fn process_enemy_behavior(&mut self, enemy_index: usize, behavior_type: &str, params: &super::enemy::Behavior) -> bool {
        match behavior_type {
            "reflect_damage" => {
                if let Some(percent) = params.percent {
                    let reflected = (self.last_damage_dealt * percent / 100) as i32;
                    if reflected > 0 {
                        self.player_hp -= reflected;
                        self.log_typed(format!("The enemy reflects {} damage back at you!", reflected), MsgType::Combat);
                        return true;
                    }
                }
            },
            "poison_sting" => {
                if let Some(duration) = params.value {
                    self.apply_status_effect("poison", duration as i32);
                    return true;
                }
            },
            "web_trap" => {
                if let Some(turns) = params.value {
                    self.apply_status_effect("immobilized", turns as i32);
                    self.log_typed("You are trapped in webbing!".to_string(), MsgType::Combat);
                    return true;
                }
            },
            "drain_sanity" => {
                if let Some(amount) = params.value {
                    // Placeholder for sanity system
                    self.log_typed(format!("Your mind reels from the encounter! (-{} sanity)", amount), MsgType::Combat);
                    return true;
                }
            },
            "teleport" => {
                if let Some(range) = params.value {
                    // Find valid teleport position
                    for _ in 0..10 {
                        let dx = self.rng.gen_range(-(range as i32)..=(range as i32));
                        let dy = self.rng.gen_range(-(range as i32)..=(range as i32));
                        let new_x = self.enemies[enemy_index].x + dx;
                        let new_y = self.enemies[enemy_index].y + dy;
                        
                        if let Some(tile) = self.map.get(new_x, new_y) {
                            if matches!(*tile, super::map::Tile::Floor { .. }) {
                                self.enemies[enemy_index].x = new_x;
                                self.enemies[enemy_index].y = new_y;
                                self.log_typed("The enemy teleports!".to_string(), MsgType::Combat);
                                self.rebuild_spatial_index();
                                return true;
                            }
                        }
                    }
                }
            },
            _ => return false,
        }
        false
    }

    /// Apply light-based effects (glare damage, visibility modifiers)
    pub fn apply_light_effects(&mut self) {
        if self.debug_disable_glare {
            return;
        }
        let light_level = super::lighting::get_light_level(&self.light_map, self.player_x, self.player_y);
        
        // Glare damage - disabled until lighting system is properly balanced
        // if super::lighting::has_glare(&self.light_map, self.player_x, self.player_y, 250) {
        //     if !self.has_status_effect("glare_protection") {
        //         self.player_hp -= 1;
        //         self.log_typed("The intense light burns your eyes!".to_string(), MsgType::Combat);
        //         self.apply_status_effect("blinded", 2);
        //     }
        // }
        
        // Light-based item effects
        for item_id in &self.inventory.clone() {
            if let Some(def) = super::item::get_item_def(item_id) {
                if def.reveals_storm_timing && light_level > 150 {
                    // Storm Chart works better in bright light
                    if self.rng.gen_range(0..100) < 10 {
                        self.log_typed("The Storm Chart glows, revealing storm patterns...".to_string(), MsgType::System);
                    }
                }
                
                if def.grants_invisibility && light_level < 50 {
                    // Refraction Oil works better in darkness
                    if !self.has_status_effect("invisible") {
                        self.apply_status_effect("invisible", 3);
                        self.log_typed("You blend into the shadows...".to_string(), MsgType::System);
                    }
                }
            }
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let data = ron::to_string(self).map_err(|e| e.to_string())?;
        fs::write(path, data).map_err(|e| e.to_string())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut state: Self = ron::from_str(&data).map_err(|e| e.to_string())?;
        state.rebuild_spatial_index();
        state.update_lighting(); // Recalculate lighting after loading
        Ok(state)
    }
}

/// Simple Bresenham line for projectile paths
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
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
    path
}

impl GameState {
    pub fn get_current_tile_level(&self) -> u32 {
        if let Some(ref world_map) = self.world_map {
            let (_, _, _, _, _, _, level) = world_map.get(self.world_x, self.world_y);
            level
        } else {
            1
        }
    }

    pub fn get_world_map(&self) -> Option<&WorldMap> {
        self.world_map.as_ref()
    }
}
