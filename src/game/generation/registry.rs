use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

use super::algorithm::{GenerationAlgorithm, GenerationError, AlgorithmParameters, ParameterValue};

/// Registry for managing procedural generation algorithms
pub struct AlgorithmRegistry {
    algorithms: RwLock<HashMap<String, Arc<dyn GenerationAlgorithm>>>,
    configurations: RwLock<HashMap<String, AlgorithmConfig>>,
}

/// Configuration for an algorithm including default parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    /// Algorithm identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Algorithm description
    pub description: String,
    
    /// Default parameters for this algorithm
    pub default_parameters: AlgorithmParameters,
    
    /// Algorithm category (e.g., "terrain", "structure", "ecosystem")
    pub category: String,
    
    /// Performance characteristics
    pub performance_profile: PerformanceProfile,
    
    /// Supported input/output layer types
    pub supported_layers: SupportedLayers,
}

/// Performance characteristics of an algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Typical execution time category
    pub speed: SpeedCategory,
    
    /// Memory usage category
    pub memory_usage: MemoryCategory,
    
    /// CPU intensity category
    pub cpu_intensity: CpuCategory,
    
    /// Whether algorithm can be parallelized
    pub parallelizable: bool,
}

/// Algorithm speed categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeedCategory {
    VeryFast,   // < 1ms
    Fast,       // 1-10ms
    Medium,     // 10-100ms
    Slow,       // 100ms-1s
    VerySlow,   // > 1s
}

/// Memory usage categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryCategory {
    Low,        // < 1MB
    Medium,     // 1-10MB
    High,       // 10-100MB
    VeryHigh,   // > 100MB
}

/// CPU intensity categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CpuCategory {
    Light,      // Simple operations
    Medium,     // Moderate computation
    Heavy,      // Complex algorithms
    VeryHeavy,  // Intensive computation
}

/// Supported input/output layers for an algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedLayers {
    /// Layer types this algorithm can consume
    pub input_layers: Vec<String>,
    
    /// Layer types this algorithm can produce
    pub output_layers: Vec<String>,
    
    /// Required input layers (must be present)
    pub required_inputs: Vec<String>,
    
    /// Optional input layers (can be used if present)
    pub optional_inputs: Vec<String>,
}

impl AlgorithmRegistry {
    /// Create a new algorithm registry
    pub fn new() -> Self {
        Self {
            algorithms: RwLock::new(HashMap::new()),
            configurations: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new algorithm with the registry
    pub fn register_algorithm(&self, algorithm: Arc<dyn GenerationAlgorithm>, config: AlgorithmConfig) -> Result<(), GenerationError> {
        let id = algorithm.algorithm_id().to_string();
        
        // Validate algorithm ID matches config
        if id != config.id {
            return Err(GenerationError::InvalidParameters(
                format!("Algorithm ID mismatch: {} != {}", id, config.id)
            ));
        }
        
        // Validate parameters
        config.default_parameters.validate()
            .map_err(|e| GenerationError::InvalidParameters(e.to_string()))?;
        
        // Register algorithm and configuration
        {
            let mut algorithms = self.algorithms.write().unwrap();
            let mut configurations = self.configurations.write().unwrap();
            
            algorithms.insert(id.clone(), algorithm);
            configurations.insert(id, config);
        }
        
        Ok(())
    }
    
    /// Get an algorithm by ID
    pub fn get_algorithm(&self, id: &str) -> Option<Arc<dyn GenerationAlgorithm>> {
        let algorithms = self.algorithms.read().unwrap();
        algorithms.get(id).cloned()
    }
    
    /// Get algorithm configuration by ID
    pub fn get_config(&self, id: &str) -> Option<AlgorithmConfig> {
        let configurations = self.configurations.read().unwrap();
        configurations.get(id).cloned()
    }
    
    /// List all registered algorithm IDs
    pub fn list_algorithms(&self) -> Vec<String> {
        let algorithms = self.algorithms.read().unwrap();
        algorithms.keys().cloned().collect()
    }
    
    /// List algorithms by category
    pub fn list_by_category(&self, category: &str) -> Vec<String> {
        let configurations = self.configurations.read().unwrap();
        configurations.iter()
            .filter(|(_, config)| config.category == category)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Find algorithms that can consume specific input layers
    pub fn find_by_input_layers(&self, required_layers: &[String]) -> Vec<String> {
        let configurations = self.configurations.read().unwrap();
        configurations.iter()
            .filter(|(_, config)| {
                required_layers.iter().all(|layer| 
                    config.supported_layers.input_layers.contains(layer) ||
                    config.supported_layers.optional_inputs.contains(layer)
                )
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Find algorithms that can produce specific output layers
    pub fn find_by_output_layers(&self, required_layers: &[String]) -> Vec<String> {
        let configurations = self.configurations.read().unwrap();
        configurations.iter()
            .filter(|(_, config)| {
                required_layers.iter().all(|layer| 
                    config.supported_layers.output_layers.contains(layer)
                )
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Get algorithms with specific performance characteristics
    pub fn find_by_performance(&self, max_speed: SpeedCategory, max_memory: MemoryCategory) -> Vec<String> {
        let configurations = self.configurations.read().unwrap();
        configurations.iter()
            .filter(|(_, config)| {
                self.speed_level(&config.performance_profile.speed) <= self.speed_level(&max_speed) &&
                self.memory_level(&config.performance_profile.memory_usage) <= self.memory_level(&max_memory)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Create algorithm parameters with defaults and overrides
    pub fn create_parameters(&self, algorithm_id: &str, overrides: HashMap<String, ParameterValue>) -> Result<AlgorithmParameters, GenerationError> {
        let config = self.get_config(algorithm_id)
            .ok_or_else(|| GenerationError::InvalidParameters(format!("Unknown algorithm: {}", algorithm_id)))?;
        
        let mut parameters = config.default_parameters.clone();
        
        // Apply overrides
        for (key, value) in overrides {
            parameters.set(key, value);
        }
        
        // Validate final parameters
        parameters.validate()
            .map_err(|e| GenerationError::InvalidParameters(e.to_string()))?;
        
        Ok(parameters)
    }
    
    /// Validate that an algorithm chain is compatible
    pub fn validate_chain(&self, algorithm_ids: &[String]) -> Result<(), GenerationError> {
        if algorithm_ids.is_empty() {
            return Ok(());
        }
        
        let configurations = self.configurations.read().unwrap();
        
        // Check that all algorithms exist
        for id in algorithm_ids {
            if !configurations.contains_key(id) {
                return Err(GenerationError::InvalidParameters(format!("Unknown algorithm: {}", id)));
            }
        }
        
        // Check layer compatibility between adjacent algorithms
        for window in algorithm_ids.windows(2) {
            let producer_config = &configurations[&window[0]];
            let consumer_config = &configurations[&window[1]];
            
            // Check if consumer can use producer's outputs
            let compatible = consumer_config.supported_layers.required_inputs.iter()
                .all(|required| producer_config.supported_layers.output_layers.contains(required));
            
            if !compatible {
                return Err(GenerationError::InvalidParameters(format!(
                    "Incompatible algorithms: {} cannot consume outputs from {}",
                    window[1], window[0]
                )));
            }
        }
        
        Ok(())
    }
    
    /// Get registry statistics
    pub fn get_statistics(&self) -> RegistryStatistics {
        let algorithms = self.algorithms.read().unwrap();
        let configurations = self.configurations.read().unwrap();
        
        let mut categories = HashMap::new();
        let mut performance_distribution = HashMap::new();
        
        for config in configurations.values() {
            *categories.entry(config.category.clone()).or_insert(0) += 1;
            *performance_distribution.entry(format!("{:?}", config.performance_profile.speed)).or_insert(0) += 1;
        }
        
        RegistryStatistics {
            total_algorithms: algorithms.len(),
            categories,
            performance_distribution,
        }
    }
    
    // Helper methods for performance comparison
    fn speed_level(&self, speed: &SpeedCategory) -> u8 {
        match speed {
            SpeedCategory::VeryFast => 0,
            SpeedCategory::Fast => 1,
            SpeedCategory::Medium => 2,
            SpeedCategory::Slow => 3,
            SpeedCategory::VerySlow => 4,
        }
    }
    
    fn memory_level(&self, memory: &MemoryCategory) -> u8 {
        match memory {
            MemoryCategory::Low => 0,
            MemoryCategory::Medium => 1,
            MemoryCategory::High => 2,
            MemoryCategory::VeryHigh => 3,
        }
    }
}

/// Registry statistics for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    pub total_algorithms: usize,
    pub categories: HashMap<String, usize>,
    pub performance_distribution: HashMap<String, usize>,
}

impl Default for AlgorithmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe singleton registry instance
use std::sync::{Once, OnceLock};

static GLOBAL_REGISTRY: OnceLock<AlgorithmRegistry> = OnceLock::new();

/// Get the global algorithm registry instance
pub fn get_global_registry() -> &'static AlgorithmRegistry {
    GLOBAL_REGISTRY.get_or_init(|| AlgorithmRegistry::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::generation::algorithm::*;
    
    // Mock algorithm for testing
    struct MockAlgorithm {
        id: String,
        parameters: AlgorithmParameters,
    }
    
    impl GenerationAlgorithm for MockAlgorithm {
        fn generate(&self, _context: &AlgorithmContext) -> Result<GenerationResult, GenerationError> {
            unimplemented!("Mock algorithm")
        }
        
        fn parameters(&self) -> &AlgorithmParameters {
            &self.parameters
        }
        
        fn validate_context(&self, _context: &AlgorithmContext) -> Result<(), ValidationError> {
            Ok(())
        }
        
        fn algorithm_id(&self) -> &str {
            &self.id
        }
        
        fn display_name(&self) -> &str {
            "Mock Algorithm"
        }
        
        fn description(&self) -> &str {
            "A mock algorithm for testing"
        }
    }
    
    #[test]
    fn test_registry_basic_operations() {
        let registry = AlgorithmRegistry::new();
        
        let algorithm = Arc::new(MockAlgorithm {
            id: "test_algorithm".to_string(),
            parameters: AlgorithmParameters::new(),
        });
        
        let config = AlgorithmConfig {
            id: "test_algorithm".to_string(),
            name: "Test Algorithm".to_string(),
            description: "A test algorithm".to_string(),
            default_parameters: AlgorithmParameters::new(),
            category: "test".to_string(),
            performance_profile: PerformanceProfile {
                speed: SpeedCategory::Fast,
                memory_usage: MemoryCategory::Low,
                cpu_intensity: CpuCategory::Light,
                parallelizable: true,
            },
            supported_layers: SupportedLayers {
                input_layers: vec!["heightmap".to_string()],
                output_layers: vec!["terrain".to_string()],
                required_inputs: vec!["heightmap".to_string()],
                optional_inputs: vec![],
            },
        };
        
        // Register algorithm
        assert!(registry.register_algorithm(algorithm.clone(), config).is_ok());
        
        // Retrieve algorithm
        assert!(registry.get_algorithm("test_algorithm").is_some());
        assert!(registry.get_config("test_algorithm").is_some());
        
        // List algorithms
        let algorithms = registry.list_algorithms();
        assert!(algorithms.contains(&"test_algorithm".to_string()));
    }
}
