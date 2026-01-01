pub mod action;
pub mod adaptation;
pub mod book;
pub mod chest;
pub mod combat;
pub mod combat_actions;
pub mod constants;
pub mod crafting;
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
pub mod item;
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
pub mod systems;
pub mod trading;
pub mod tutorial;
pub mod world_map;

pub use action::{action_cost, default_enemy_ap, default_player_ap};
pub use adaptation::Adaptation;
pub use chest::{get_chest_def, Chest, ChestDef};
pub use combat::{calc_damage, calc_hit_chance, default_weapon, get_weapon_def, roll_attack, CombatResult, WeaponDef};
pub use constants::{FOV_RANGE, MAP_HEIGHT, MAP_WIDTH};
pub use equipment::{Equipment, EquipSlot};
pub use fov::FieldOfView;
pub use generation::{
    GenerationPipeline, GenerationConfig, GenerationContext, load_generation_config, 
    TemplateLibrary, TemplateContext, ContentTemplate, 
    Grammar, GrammarContext, load_grammars_from_directory,
    BiomeSystem, BiomeProfile, BiomeGenerationContext, EnvironmentalFeature,
    AtmosphericElement, BiomeHazard, ResourceModifiers, MechanicalEffects,
    ConstraintSystem, ConstraintRule, ConstraintType, ConstraintSeverity,
    ConstraintResult, ConstraintContext, EntityPlacement, ResourcePlacement, ObjectivePlacement,
    get_microstructure_def, place_microstructures, PlacedMicroStructure, MicroStructureDef,
    PoissonSampler, distribute_points_grid,
    get_loot_table, generate_loot, LootTable, LootEntry,
    StoryModel, StoryEvent, StoryCharacter, EventType, Relationship, RelationshipType,
    NarrativeGenerator, NarrativeTemplate, HistoricalEvent,
    events::{EventSystem, EventContext, DynamicEvent},
    narrative::{NarrativeIntegration, NarrativeContext, NarrativeState, StoryFragment},
};
pub use status::{is_stunned, slow_penalty, StatusEffect, StatusType};
pub use effect::{get_active_effects, get_enemy_effects, parse_effect, EffectContext, VisualEffect};
pub use enemy::{all_enemy_ids, get_enemy_def, Enemy, EnemyDef};
pub use entity::{Entity, EntityType};
pub use item::{all_item_ids, get_item_def, Item, ItemDef};

pub use map::{compute_fov, Map, Tile};
pub use map_features::MapFeatures;


pub use npc::{get_npc_def, Npc, NpcDef};
pub use state::{DamageNumber, GameMessage, GameState, MsgType, ProjectileTrail, TriggeredEffect};

pub use storm::Storm;
pub use world_map::{Biome, POI, Terrain, WorldMap, WORLD_SIZE};
pub use lighting::{compute_lighting, is_lit, LightMap, LightSource};
pub use light_defs::get_light_def;
pub use map::MapLight;
pub use quest::{get_quest_def, ActiveQuest, QuestLog, QuestReward};
pub use crafting::{get_recipe, all_recipe_ids, can_craft, available_recipes, can_craft_advanced, crafting_success_chance, Recipe};
pub use sanity::{SanitySystem, MentalEffect, MentalEffectType};
pub use skills::{get_skill_def, get_ability_def, all_skill_ids, all_ability_ids, SkillsState, SkillCategory, calculate_skill_cost, get_skills_by_category, get_abilities_by_category};

pub use trading::{get_trader, get_trade_interface, execute_trade, execute_sell, TradeInterface, AvailableTradeItem, calculate_area_tier};
pub use dialogue::{get_dialogue_tree, start_dialogue, continue_dialogue, DialogueState};
pub use meta::{all_classes, get_class, ClassDef, MetaProgress};
pub use tutorial::{get_tutorial_data, TutorialProgress};
pub use qa_tools::{DebugInfo, IssueCategory, IssueReport, IssueSeverity};
pub use des_testing::{DesTest, DesTestResult, run_des_test_file, list_des_tests, create_sample_des_test, save_sample_des_test};
