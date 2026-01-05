use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Core trait that all procedural generation algorithms must implement
pub trait GenerationAlgorithm: Send + Sync {
    /// Generate content based on the provided context
    fn generate(&self, context: &AlgorithmContext) -> Result<GenerationResult, GenerationError>;
    
    /// Get algorithm-specific parameters and their current values
    fn parameters(&self) -> &AlgorithmParameters;
    
    /// Validate that the context is suitable for this algorithm
    fn validate_context(&self, context: &AlgorithmContext) -> Result<(), ValidationError>;
    
    /// Unique identifier for this algorithm
    fn algorithm_id(&self) -> &str;
    
    /// Human-readable name for this algorithm
    fn display_name(&self) -> &str;
    
    /// Description of what this algorithm does
    fn description(&self) -> &str;
}

/// Context passed to generation algorithms containing all necessary input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmContext {
    /// Map dimensions
    pub width: usize,
    pub height: usize,
    
    /// Random seed for deterministic generation
    pub seed: u64,
    
    /// Biome type for this generation
    pub biome: String,
    
    /// Point of Interest type (if any)
    pub poi_type: Option<String>,
    
    /// Input layers from previous generation passes
    pub input_layers: HashMap<String, GenerationLayer>,
    
    /// Algorithm-specific parameters
    pub parameters: AlgorithmParameters,
    
    /// Quest IDs that need to be supported
    pub quest_ids: Vec<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result returned by generation algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    /// Output layers for use by subsequent algorithms
    pub output_layers: HashMap<String, GenerationLayer>,
    
    /// Generation metadata and statistics
    pub metadata: GenerationMetadata,
    
    /// Any warnings or issues encountered during generation
    pub warnings: Vec<String>,
}

/// A layer of generation data that can be passed between algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationLayer {
    /// Layer name/identifier
    pub name: String,
    
    /// Layer data type
    pub layer_type: LayerType,
    
    /// Raw layer data
    pub data: Vec<Vec<f64>>,
    
    /// Layer metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of generation layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerType {
    /// Height/elevation data
    Heightmap,
    
    /// Moisture/humidity data
    Moisture,
    
    /// Temperature data
    Temperature,
    
    /// Density/probability data
    Density,
    
    /// Binary mask data
    Mask,
    
    /// Flow/direction data
    Flow,
    
    /// Custom layer type
    Custom(String),
}

/// Algorithm parameters with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmParameters {
    /// Parameter values
    pub values: HashMap<String, ParameterValue>,
    
    /// Parameter definitions and constraints
    pub definitions: HashMap<String, ParameterDefinition>,
}

/// A parameter value with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    String(String),
    Array(Vec<ParameterValue>),
}

/// Parameter definition with validation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter name
    pub name: String,
    
    /// Parameter description
    pub description: String,
    
    /// Parameter type
    pub param_type: ParameterType,
    
    /// Default value
    pub default_value: ParameterValue,
    
    /// Validation constraints
    pub constraints: Option<ParameterConstraints>,
    
    /// Whether this parameter is required
    pub required: bool,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Float,
    Integer,
    Boolean,
    String,
    Array(Box<ParameterType>),
}

/// Parameter validation constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraints {
    /// Minimum value (for numeric types)
    pub min_value: Option<f64>,
    
    /// Maximum value (for numeric types)
    pub max_value: Option<f64>,
    
    /// Valid string values (for string types)
    pub valid_values: Option<Vec<String>>,
    
    /// Array length constraints
    pub array_length: Option<(usize, usize)>, // (min, max)
}

/// Metadata about the generation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// Algorithm that generated this result
    pub algorithm_id: String,
    
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    
    /// Random seed used
    pub seed: u64,
    
    /// Quality metrics
    pub quality_metrics: HashMap<String, f64>,
    
    /// Performance metrics
    pub performance_metrics: HashMap<String, f64>,
    
    /// Additional algorithm-specific metadata
    pub algorithm_metadata: HashMap<String, serde_json::Value>,
}

/// Generation error types
#[derive(Debug, Clone)]
pub enum GenerationError {
    /// Invalid context provided
    InvalidContext(String),
    
    /// Parameter validation failed
    InvalidParameters(String),
    
    /// Algorithm execution failed
    ExecutionFailed(String),
    
    /// Resource constraints exceeded
    ResourceExhausted(String),
    
    /// Generic error with message
    Other(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::InvalidContext(msg) => write!(f, "Invalid context: {}", msg),
            GenerationError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            GenerationError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            GenerationError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            GenerationError::Other(msg) => write!(f, "Generation error: {}", msg),
        }
    }
}

impl Error for GenerationError {}

/// Validation error for context/parameter validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "Validation error in {}: {}", field, self.message),
            None => write!(f, "Validation error: {}", self.message),
        }
    }
}

impl Error for ValidationError {}

impl AlgorithmParameters {
    /// Create new empty parameters
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            definitions: HashMap::new(),
        }
    }
    
    /// Get a parameter value by name
    pub fn get<T>(&self, name: &str) -> Result<T, GenerationError>
    where
        T: TryFrom<ParameterValue>,
        T::Error: fmt::Display,
    {
        let value = self.values.get(name)
            .ok_or_else(|| GenerationError::InvalidParameters(format!("Missing parameter: {}", name)))?;
        
        T::try_from(value.clone())
            .map_err(|e| GenerationError::InvalidParameters(format!("Parameter conversion failed for {}: {}", name, e)))
    }
    
    /// Set a parameter value
    pub fn set(&mut self, name: String, value: ParameterValue) {
        self.values.insert(name, value);
    }
    
    /// Validate all parameters against their definitions
    pub fn validate(&self) -> Result<(), ValidationError> {
        for (name, definition) in &self.definitions {
            if definition.required && !self.values.contains_key(name) {
                return Err(ValidationError {
                    message: format!("Required parameter missing: {}", name),
                    field: Some(name.clone()),
                });
            }
            
            if let Some(value) = self.values.get(name) {
                self.validate_parameter_value(name, value, definition)?;
            }
        }
        Ok(())
    }
    
    fn validate_parameter_value(&self, name: &str, value: &ParameterValue, definition: &ParameterDefinition) -> Result<(), ValidationError> {
        if let Some(constraints) = &definition.constraints {
            match value {
                ParameterValue::Float(f) => {
                    if let Some(min) = constraints.min_value {
                        if *f < min {
                            return Err(ValidationError {
                                message: format!("Value {} below minimum {}", f, min),
                                field: Some(name.to_string()),
                            });
                        }
                    }
                    if let Some(max) = constraints.max_value {
                        if *f > max {
                            return Err(ValidationError {
                                message: format!("Value {} above maximum {}", f, max),
                                field: Some(name.to_string()),
                            });
                        }
                    }
                }
                ParameterValue::Integer(i) => {
                    if let Some(min) = constraints.min_value {
                        if (*i as f64) < min {
                            return Err(ValidationError {
                                message: format!("Value {} below minimum {}", i, min),
                                field: Some(name.to_string()),
                            });
                        }
                    }
                    if let Some(max) = constraints.max_value {
                        if (*i as f64) > max {
                            return Err(ValidationError {
                                message: format!("Value {} above maximum {}", i, max),
                                field: Some(name.to_string()),
                            });
                        }
                    }
                }
                ParameterValue::String(s) => {
                    if let Some(valid_values) = &constraints.valid_values {
                        if !valid_values.contains(s) {
                            return Err(ValidationError {
                                message: format!("Invalid value '{}', must be one of: {:?}", s, valid_values),
                                field: Some(name.to_string()),
                            });
                        }
                    }
                }
                ParameterValue::Array(arr) => {
                    if let Some((min_len, max_len)) = constraints.array_length {
                        if arr.len() < min_len || arr.len() > max_len {
                            return Err(ValidationError {
                                message: format!("Array length {} not in range [{}, {}]", arr.len(), min_len, max_len),
                                field: Some(name.to_string()),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

// Conversion implementations for ParameterValue
impl TryFrom<ParameterValue> for f64 {
    type Error = String;
    
    fn try_from(value: ParameterValue) -> Result<Self, Self::Error> {
        match value {
            ParameterValue::Float(f) => Ok(f),
            ParameterValue::Integer(i) => Ok(i as f64),
            _ => Err("Cannot convert to f64".to_string()),
        }
    }
}

impl TryFrom<ParameterValue> for i64 {
    type Error = String;
    
    fn try_from(value: ParameterValue) -> Result<Self, Self::Error> {
        match value {
            ParameterValue::Integer(i) => Ok(i),
            ParameterValue::Float(f) => Ok(f as i64),
            _ => Err("Cannot convert to i64".to_string()),
        }
    }
}

impl TryFrom<ParameterValue> for bool {
    type Error = String;
    
    fn try_from(value: ParameterValue) -> Result<Self, Self::Error> {
        match value {
            ParameterValue::Boolean(b) => Ok(b),
            _ => Err("Cannot convert to bool".to_string()),
        }
    }
}

impl TryFrom<ParameterValue> for String {
    type Error = String;
    
    fn try_from(value: ParameterValue) -> Result<Self, Self::Error> {
        match value {
            ParameterValue::String(s) => Ok(s),
            _ => Err("Cannot convert to String".to_string()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestAlgorithm {
        id: String,
        parameters: AlgorithmParameters,
    }

    impl TestAlgorithm {
        fn new(id: &str) -> Self {
            let mut parameters = AlgorithmParameters::new();
            parameters.set("test_param".to_string(), ParameterValue::Float(1.0));
            
            Self {
                id: id.to_string(),
                parameters,
            }
        }
    }

    impl GenerationAlgorithm for TestAlgorithm {
        fn generate(&self, context: &AlgorithmContext) -> Result<GenerationResult, GenerationError> {
            let mut output_layers = HashMap::new();
            
            // Create a simple test layer
            let data = vec![vec![0.5; context.width]; context.height];
            let layer = GenerationLayer {
                name: "test_layer".to_string(),
                layer_type: LayerType::Heightmap,
                data,
                metadata: HashMap::new(),
            };
            output_layers.insert("test_layer".to_string(), layer);
            
            Ok(GenerationResult {
                output_layers,
                metadata: GenerationMetadata {
                    algorithm_id: self.id.clone(),
                    generation_time_ms: 10,
                    seed: context.seed,
                    quality_metrics: HashMap::new(),
                    performance_metrics: HashMap::new(),
                    algorithm_metadata: HashMap::new(),
                },
                warnings: Vec::new(),
            })
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
            "Test Algorithm"
        }

        fn description(&self) -> &str {
            "A test algorithm for unit testing"
        }
    }

    #[test]
    fn test_algorithm_context_creation() {
        let context = AlgorithmContext {
            width: 10,
            height: 10,
            seed: 12345,
            biome: "test".to_string(),
            poi_type: None,
            input_layers: HashMap::new(),
            parameters: AlgorithmParameters::new(),
            quest_ids: Vec::new(),
            metadata: HashMap::new(),
        };

        assert_eq!(context.width, 10);
        assert_eq!(context.height, 10);
        assert_eq!(context.seed, 12345);
        assert_eq!(context.biome, "test");
    }

    #[test]
    fn test_algorithm_parameters() {
        let mut params = AlgorithmParameters::new();
        params.set("float_param".to_string(), ParameterValue::Float(3.14));
        params.set("int_param".to_string(), ParameterValue::Integer(42));
        params.set("bool_param".to_string(), ParameterValue::Boolean(true));
        params.set("string_param".to_string(), ParameterValue::String("test".to_string()));

        // Test using the generic get method
        assert_eq!(params.get::<f64>("float_param").unwrap(), 3.14);
        assert_eq!(params.get::<i64>("int_param").unwrap(), 42);
        assert_eq!(params.get::<bool>("bool_param").unwrap(), true);
        assert_eq!(params.get::<String>("string_param").unwrap(), "test".to_string());
        
        // Test missing parameter
        assert!(params.get::<f64>("nonexistent").is_err());
    }

    #[test]
    fn test_generation_layer() {
        let data = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let layer = GenerationLayer {
            name: "test".to_string(),
            layer_type: LayerType::Heightmap,
            data: data.clone(),
            metadata: HashMap::new(),
        };

        assert_eq!(layer.name, "test");
        assert_eq!(layer.data, data);
        assert!(matches!(layer.layer_type, LayerType::Heightmap));
    }

    #[test]
    fn test_test_algorithm_generation() {
        let algorithm = TestAlgorithm::new("test_algo");
        let context = AlgorithmContext {
            width: 5,
            height: 5,
            seed: 12345,
            biome: "test".to_string(),
            poi_type: None,
            input_layers: HashMap::new(),
            parameters: AlgorithmParameters::new(),
            quest_ids: Vec::new(),
            metadata: HashMap::new(),
        };

        let result = algorithm.generate(&context).unwrap();
        
        assert!(result.output_layers.contains_key("test_layer"));
        assert_eq!(result.metadata.algorithm_id, "test_algo");
        
        let layer = &result.output_layers["test_layer"];
        assert_eq!(layer.data.len(), 5); // height
        assert_eq!(layer.data[0].len(), 5); // width
        assert_eq!(layer.data[0][0], 0.5);
    }
}
