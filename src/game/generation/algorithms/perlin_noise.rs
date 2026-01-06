use bracket_noise::prelude::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

use super::super::algorithm::{
    GenerationAlgorithm, AlgorithmContext, GenerationResult, GenerationError, ValidationError,
    AlgorithmParameters, ParameterValue, ParameterDefinition, ParameterType, ParameterConstraints,
    GenerationLayer, LayerType, GenerationMetadata
};

/// Simple Perlin noise terrain generation algorithm
pub struct PerlinNoiseAlgorithm {
    parameters: AlgorithmParameters,
}

impl PerlinNoiseAlgorithm {
    pub fn new() -> Self {
        let mut parameters = AlgorithmParameters::new();
        
        // Define parameters with defaults and constraints
        parameters.definitions.insert("scale".to_string(), ParameterDefinition {
            name: "scale".to_string(),
            description: "Noise scale factor".to_string(),
            param_type: ParameterType::Float,
            default_value: ParameterValue::Float(0.1),
            constraints: Some(ParameterConstraints {
                min_value: Some(0.001),
                max_value: Some(1.0),
                valid_values: None,
                array_length: None,
            }),
            required: true,
        });
        
        parameters.definitions.insert("octaves".to_string(), ParameterDefinition {
            name: "octaves".to_string(),
            description: "Number of noise octaves".to_string(),
            param_type: ParameterType::Integer,
            default_value: ParameterValue::Integer(4),
            constraints: Some(ParameterConstraints {
                min_value: Some(1.0),
                max_value: Some(8.0),
                valid_values: None,
                array_length: None,
            }),
            required: true,
        });
        
        parameters.definitions.insert("persistence".to_string(), ParameterDefinition {
            name: "persistence".to_string(),
            description: "Amplitude persistence between octaves".to_string(),
            param_type: ParameterType::Float,
            default_value: ParameterValue::Float(0.5),
            constraints: Some(ParameterConstraints {
                min_value: Some(0.1),
                max_value: Some(1.0),
                valid_values: None,
                array_length: None,
            }),
            required: true,
        });
        
        // Set default values
        parameters.set("scale".to_string(), ParameterValue::Float(0.1));
        parameters.set("octaves".to_string(), ParameterValue::Integer(4));
        parameters.set("persistence".to_string(), ParameterValue::Float(0.5));
        
        Self { parameters }
    }
}

impl GenerationAlgorithm for PerlinNoiseAlgorithm {
    fn generate(&self, context: &AlgorithmContext) -> Result<GenerationResult, GenerationError> {
        let start_time = std::time::Instant::now();
        
        // Extract parameters
        let scale: f64 = self.parameters.get("scale")?;
        let octaves: i64 = self.parameters.get("octaves")?;
        let persistence: f64 = self.parameters.get("persistence")?;
        
        // Create RNG from context seed  
        let mut rng = ChaCha8Rng::seed_from_u64(context.seed);
        let noise_seed: u64 = rng.next_u64();
        let mut noise = FastNoise::seeded(noise_seed);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(scale as f32);
        
        // Generate heightmap
        let mut heightmap = vec![vec![0.0; context.height]; context.width];
        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;
        
        for x in 0..context.width {
            for y in 0..context.height {
                let mut value = 0.0;
                let mut amplitude = 1.0;
                let mut frequency = scale;
                
                // Multi-octave noise
                for _ in 0..octaves {
                    value += (noise.get_noise((x as f64 * frequency) as f32, (y as f64 * frequency) as f32) as f64) * amplitude;
                    amplitude *= persistence;
                    frequency *= 2.0;
                }
                
                heightmap[x][y] = value;
                min_value = min_value.min(value);
                max_value = max_value.max(value);
            }
        }
        
        // Normalize to [0, 1] using actual min/max
        let range = max_value - min_value;
        if range > 0.0 {
            for x in 0..context.width {
                for y in 0..context.height {
                    heightmap[x][y] = (heightmap[x][y] - min_value) / range;
                }
            }
        }
        
        // Create output layer
        let mut output_layers = HashMap::new();
        output_layers.insert("heightmap".to_string(), GenerationLayer {
            name: "heightmap".to_string(),
            layer_type: LayerType::Heightmap,
            data: heightmap,
            metadata: HashMap::new(),
        });
        
        // Generate metadata
        let generation_time = start_time.elapsed().as_millis() as u64;
        let metadata = GenerationMetadata {
            algorithm_id: self.algorithm_id().to_string(),
            generation_time_ms: generation_time,
            seed: context.seed,
            quality_metrics: HashMap::new(),
            performance_metrics: {
                let mut perf = HashMap::new();
                perf.insert("generation_time_ms".to_string(), generation_time as f64);
                perf
            },
            algorithm_metadata: {
                let mut meta = HashMap::new();
                meta.insert("scale".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(scale).unwrap()));
                meta.insert("octaves".to_string(), serde_json::Value::Number(serde_json::Number::from(octaves)));
                meta.insert("persistence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(persistence).unwrap()));
                meta
            },
        };
        
        Ok(GenerationResult {
            output_layers,
            metadata,
            warnings: Vec::new(),
        })
    }
    
    fn parameters(&self) -> &AlgorithmParameters {
        &self.parameters
    }
    
    fn validate_context(&self, context: &AlgorithmContext) -> Result<(), ValidationError> {
        if context.width == 0 || context.height == 0 {
            return Err(ValidationError {
                message: "Width and height must be greater than 0".to_string(),
                field: Some("dimensions".to_string()),
            });
        }
        
        if context.width > 1000 || context.height > 1000 {
            return Err(ValidationError {
                message: "Dimensions too large (max 1000x1000)".to_string(),
                field: Some("dimensions".to_string()),
            });
        }
        
        Ok(())
    }
    
    fn algorithm_id(&self) -> &str {
        "perlin_noise"
    }
    
    fn display_name(&self) -> &str {
        "Perlin Noise Terrain"
    }
    
    fn description(&self) -> &str {
        "Generates natural-looking terrain using multi-octave Perlin noise"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_perlin_noise_algorithm() {
        let algorithm = PerlinNoiseAlgorithm::new();
        
        let context = AlgorithmContext {
            width: 50,
            height: 50,
            seed: 12345,
            biome: "desert".to_string(),
            poi_type: None,
            input_layers: HashMap::new(),
            parameters: algorithm.parameters().clone(),
            quest_ids: Vec::new(),
            metadata: HashMap::new(),
        };
        
        // Validate context
        assert!(algorithm.validate_context(&context).is_ok());
        
        // Generate terrain
        let result = algorithm.generate(&context).unwrap();
        
        // Check output
        assert!(result.output_layers.contains_key("heightmap"));
        assert_eq!(result.metadata.algorithm_id, "perlin_noise");
        assert!(result.metadata.generation_time_ms > 0);
        
        // Check heightmap dimensions
        let heightmap = &result.output_layers["heightmap"];
        assert_eq!(heightmap.data.len(), 50);
        assert_eq!(heightmap.data[0].len(), 50);
        
        // Check values are in valid range [0, 1]
        for row in &heightmap.data {
            for &value in row {
                assert!(value >= 0.0 && value <= 1.0);
            }
        }
    }
    
    #[test]
    fn test_parameter_validation() {
        let algorithm = PerlinNoiseAlgorithm::new();
        
        // Test invalid dimensions
        let invalid_context = AlgorithmContext {
            width: 0,
            height: 50,
            seed: 12345,
            biome: "desert".to_string(),
            poi_type: None,
            input_layers: HashMap::new(),
            parameters: algorithm.parameters().clone(),
            quest_ids: Vec::new(),
            metadata: HashMap::new(),
        };
        
        assert!(algorithm.validate_context(&invalid_context).is_err());
    }
}
