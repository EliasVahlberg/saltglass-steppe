use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use rand_chacha::ChaCha8Rng;
use crate::game::Map;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PassType {
    Terrain,
    Features,
    Entities,
    Narrative,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenerationPass {
    pub id: String,
    pub pass_type: PassType,
    pub config: Value,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenerationConfig {
    pub passes: Vec<GenerationPass>,
}

pub struct GenerationContext {
    pub map: Map,
    pub rng: ChaCha8Rng,
    pub metadata: HashMap<String, Value>,
}

pub struct GenerationPipeline {
    config: GenerationConfig,
}

impl GenerationPipeline {
    pub fn new(config: GenerationConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, mut context: GenerationContext) -> Result<GenerationContext, String> {
        let sorted_passes = self.sort_passes_by_dependencies()?;
        
        for pass in sorted_passes {
            context = self.execute_pass(&pass, context)?;
        }
        
        Ok(context)
    }

    pub fn sort_passes_by_dependencies(&self) -> Result<Vec<&GenerationPass>, String> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for pass in &self.config.passes {
            if !visited.contains(&pass.id) {
                self.visit_pass(pass, &mut sorted, &mut visited, &mut visiting)?;
            }
        }

        Ok(sorted)
    }

    fn visit_pass<'a>(
        &'a self,
        pass: &'a GenerationPass,
        sorted: &mut Vec<&'a GenerationPass>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
    ) -> Result<(), String> {
        if visiting.contains(&pass.id) {
            return Err(format!("Circular dependency detected involving pass: {}", pass.id));
        }

        if visited.contains(&pass.id) {
            return Ok(());
        }

        visiting.insert(pass.id.clone());

        for dep_id in &pass.dependencies {
            if let Some(dep_pass) = self.config.passes.iter().find(|p| p.id == *dep_id) {
                self.visit_pass(dep_pass, sorted, visited, visiting)?;
            } else {
                return Err(format!("Dependency not found: {}", dep_id));
            }
        }

        visiting.remove(&pass.id);
        visited.insert(pass.id.clone());
        sorted.push(pass);

        Ok(())
    }

    fn execute_pass(&self, pass: &GenerationPass, context: GenerationContext) -> Result<GenerationContext, String> {
        match pass.pass_type {
            PassType::Terrain => self.execute_terrain_pass(pass, context),
            PassType::Features => self.execute_features_pass(pass, context),
            PassType::Entities => self.execute_entities_pass(pass, context),
            PassType::Narrative => self.execute_narrative_pass(pass, context),
        }
    }

    fn execute_terrain_pass(&self, _pass: &GenerationPass, context: GenerationContext) -> Result<GenerationContext, String> {
        // Placeholder - will integrate existing terrain generation
        Ok(context)
    }

    fn execute_features_pass(&self, _pass: &GenerationPass, context: GenerationContext) -> Result<GenerationContext, String> {
        // Placeholder - will add feature placement logic
        Ok(context)
    }

    fn execute_entities_pass(&self, _pass: &GenerationPass, context: GenerationContext) -> Result<GenerationContext, String> {
        // Placeholder - will add entity spawning logic
        Ok(context)
    }

    fn execute_narrative_pass(&self, _pass: &GenerationPass, context: GenerationContext) -> Result<GenerationContext, String> {
        // Placeholder - will add narrative element placement
        Ok(context)
    }
}

pub fn load_generation_config() -> Result<GenerationConfig, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("data/generation_config.json")?;
    let config: GenerationConfig = serde_json::from_str(&data)?;
    Ok(config)
}
