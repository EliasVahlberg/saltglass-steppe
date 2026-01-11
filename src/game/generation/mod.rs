pub mod adapters;
pub mod algorithm;
pub mod algorithms;
pub mod biomes;
pub mod config;
pub mod connectivity;
pub mod constraint_tests;
pub mod constraints;
pub mod events;
pub mod grammar;
pub mod layered_generation;
pub mod loot;
pub mod microstructures;
pub mod narrative;
pub mod narrative_templates;
pub mod pipeline;
pub mod quest_constraints;
pub mod registry;
pub mod spatial;
pub mod spawn;
pub mod feature_registry;
pub mod feature_materializer;
pub mod story;
pub mod structures;
pub mod templates;
pub mod terrain_forge_adapter;
pub mod weighted_table;
pub mod world_gen;

#[cfg(test)]
mod tests;

pub use algorithm::{
    AlgorithmContext, AlgorithmParameters, GenerationAlgorithm, GenerationError, GenerationLayer,
    GenerationMetadata, GenerationResult, ValidationError,
};
pub use algorithms::*;
pub use biomes::*;
pub use config::{ConfigurationLoader, GenerationConfiguration, GenerationPassConfig};
pub use connectivity::*;
pub use constraints::*;
pub use grammar::*;
pub use layered_generation::*;
pub use loot::*;
pub use microstructures::*;
pub use narrative::*;
pub use narrative_templates::{
    HistoricalEvent, NarrativeContext, NarrativeGenerator, NarrativeTemplate,
};
pub use pipeline::*;
pub use registry::*;
pub use spatial::*;
pub use spawn::*;
pub use story::*;
pub use structures::*;
pub use templates::*;
pub use terrain_forge_adapter::*;
pub use weighted_table::*;
pub use world_gen::*;
