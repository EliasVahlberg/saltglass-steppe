pub mod action;
pub mod adaptation;
pub mod auto_explore;
pub mod book;
pub mod chest;
pub mod combat;
pub mod combat_actions;
pub mod constants;
pub mod crafting;
pub mod crystal_resonance;
pub mod des_testing;
pub mod dialogue;
pub mod effect;
pub mod enemy;
pub mod entity;
pub mod equipment;
pub mod event;
pub mod fov;
pub mod generation;
pub mod inspect;
pub mod interactable;
pub mod item;
pub mod light;
pub mod light_defs;
pub mod lighting;
pub mod map;
pub mod map_features;
pub mod meta;
pub mod npc;
pub mod progression;
pub mod psychic;
pub mod qa_tools;
pub mod quest;
pub mod ritual;
pub mod sanity;
pub mod skills;
pub mod state;
pub mod status;
pub mod storm;
pub mod structure_templates;
pub mod systems;
pub mod trading;
pub mod tutorial;
pub mod void_energy;
pub mod world_map;

#[cfg(test)]
pub mod tests;

pub use action::{action_cost, default_enemy_ap, default_player_ap};
pub use adaptation::Adaptation;
pub use auto_explore::{AutoExploreConfig, get_auto_explore_config};
pub use chest::{Chest, ChestDef, get_chest_def};
pub use combat::{
    CombatResult, WeaponDef, calc_damage, calc_hit_chance, default_weapon, get_weapon_def,
    roll_attack,
};
pub use constants::{FOV_RANGE, MAP_HEIGHT, MAP_WIDTH};
pub use effect::{
    EffectContext, VisualEffect, get_active_effects, get_enemy_effects, parse_effect,
};
pub use enemy::{Enemy, EnemyDef, all_enemy_ids, get_enemy_def};
pub use entity::{Entity, EntityType};
pub use equipment::{EquipSlot, Equipment};
pub use fov::FieldOfView;
pub use generation::{
    AlgorithmContext,
    AlgorithmParameters,
    AlgorithmRegistry,
    AtmosphericElement,
    BiomeGenerationContext,
    BiomeHazard,
    BiomeProfile,
    BiomeSystem,
    ConfigurationLoader,
    ConstraintContext,
    ConstraintResult,
    ConstraintRule,
    ConstraintSeverity,
    ConstraintSystem,
    ConstraintType,
    ContentTemplate,
    EntityPlacement,
    EnvironmentalFeature,
    EventType,
    // New algorithm system
    GenerationAlgorithm,
    GenerationConfig,
    GenerationConfiguration,
    GenerationError,
    GenerationLayer,
    GenerationPipeline,
    GenerationResult,
    Grammar,
    GrammarContext,
    HistoricalEvent,
    LootEntry,
    LootTable,
    MechanicalEffects,
    MicroStructureDef,
    NarrativeGenerator,
    NarrativeTemplate,
    ObjectivePlacement,
    PerlinNoiseAlgorithm,
    PlacedMicroStructure,
    PoissonSampler,
    Relationship,
    RelationshipType,
    ResourceModifiers,
    ResourcePlacement,
    StoryCharacter,
    StoryEvent,
    StoryModel,
    TemplateContext,
    TemplateLibrary,
    distribute_points_grid,
    events::{DynamicEvent, EventContext, EventSystem},
    generate_loot,
    get_loot_table,
    get_microstructure_def,
    load_generation_config,
    load_grammars_from_directory,
    narrative::{NarrativeContext, NarrativeIntegration, NarrativeState, StoryFragment},
    place_microstructures,
};
pub use interactable::{Interactable, InteractableDef, get_interactable_def};
pub use item::{Item, ItemDef, all_item_ids, get_item_def};
pub use status::{StatusEffect, StatusType, is_stunned, slow_penalty};

pub use map::{Map, Tile, compute_fov};
pub use map_features::MapFeatures;

pub use npc::{Npc, NpcDef, get_npc_def};
pub use state::{DamageNumber, GameMessage, GameState, MsgType, ProjectileTrail, TriggeredEffect};

pub use crafting::{
    Recipe, all_recipe_ids, available_recipes, can_craft, can_craft_advanced,
    crafting_success_chance, get_recipe,
};
pub use light_defs::get_light_def;
pub use lighting::{LightMap, LightSource, compute_lighting, is_lit};
pub use map::MapLight;
pub use quest::{ActiveQuest, QuestLog, QuestReward, get_quest_def};
pub use sanity::{MentalEffect, MentalEffectType, SanitySystem};
pub use skills::{
    SkillCategory, SkillsState, all_ability_ids, all_skill_ids, calculate_skill_cost,
    get_abilities_by_category, get_ability_def, get_skill_def, get_skills_by_category,
};
pub use storm::Storm;
pub use world_map::{Biome, POI, Terrain, WORLD_SIZE, WorldMap};

pub use des_testing::{
    DesTest, DesTestResult, create_sample_des_test, list_des_tests, run_des_test_file,
    save_sample_des_test,
};
pub use dialogue::{DialogueState, continue_dialogue, get_dialogue_tree, start_dialogue};
pub use meta::{ClassDef, MetaProgress, all_classes, get_class};
pub use qa_tools::{DebugInfo, IssueCategory, IssueReport, IssueSeverity};
pub use trading::{
    AvailableTradeItem, TradeInterface, calculate_area_tier, execute_sell, execute_trade,
    get_trade_interface, get_trader,
};
pub use tutorial::{TutorialProgress, get_tutorial_data};
