use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::algorithm::{AlgorithmParameters, ParameterValue, GenerationError};

/// Configuration for a generation pass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationPassConfig {
    /// Pass identifier
    pub id: String,
    
    /// Algorithm to use for this pass
    pub algorithm: String,
    
    /// Algorithm parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Dependencies (other passes that must complete first)
    pub dependencies: Vec<String>,
    
    /// Condition for when this pass should run (optional)
    pub condition: Option<String>,
    
    /// Input layers this pass consumes
    pub input_layers: Vec<String>,
    
    /// Output layers this pass produces
    pub output_layers: Vec<String>,
}

/// Complete generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfiguration {
    /// List of generation passes
    pub passes: Vec<GenerationPassConfig>,
    
    /// Global configuration metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Configuration loader for generation systems
pub struct ConfigurationLoader;

impl ConfigurationLoader {
    /// Load generation configuration from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<GenerationConfiguration, GenerationError> {
        let content = fs::read_to_string(path)
            .map_err(|e| GenerationError::Other(format!("Failed to read config file: {}", e)))?;
        
        let config: GenerationConfiguration = serde_json::from_str(&content)
            .map_err(|e| GenerationError::Other(format!("Failed to parse config: {}", e)))?;
        
        // Validate configuration
        Self::validate_configuration(&config)?;
        
        Ok(config)
    }
    
    /// Convert JSON parameters to AlgorithmParameters
    pub fn create_algorithm_parameters(
        json_params: &HashMap<String, serde_json::Value>
    ) -> Result<AlgorithmParameters, GenerationError> {
        let mut parameters = AlgorithmParameters::new();
        
        for (key, value) in json_params {
            let param_value = Self::json_to_parameter_value(value)?;
            parameters.set(key.clone(), param_value);
        }
        
        Ok(parameters)
    }
    
    /// Validate that configuration is well-formed
    fn validate_configuration(config: &GenerationConfiguration) -> Result<(), GenerationError> {
        // Check for duplicate pass IDs
        let mut pass_ids = std::collections::HashSet::new();
        for pass in &config.passes {
            if !pass_ids.insert(&pass.id) {
                return Err(GenerationError::InvalidParameters(
                    format!("Duplicate pass ID: {}", pass.id)
                ));
            }
        }
        
        // Check dependencies exist
        for pass in &config.passes {
            for dep in &pass.dependencies {
                if !pass_ids.contains(dep) {
                    return Err(GenerationError::InvalidParameters(
                        format!("Pass '{}' depends on non-existent pass '{}'", pass.id, dep)
                    ));
                }
            }
        }
        
        // Check for circular dependencies (simple check)
        for pass in &config.passes {
            if pass.dependencies.contains(&pass.id) {
                return Err(GenerationError::InvalidParameters(
                    format!("Pass '{}' cannot depend on itself", pass.id)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Convert JSON value to ParameterValue
    fn json_to_parameter_value(value: &serde_json::Value) -> Result<ParameterValue, GenerationError> {
        match value {
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ParameterValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(ParameterValue::Float(f))
                } else {
                    Err(GenerationError::InvalidParameters("Invalid number format".to_string()))
                }
            }
            serde_json::Value::Bool(b) => Ok(ParameterValue::Boolean(*b)),
            serde_json::Value::String(s) => Ok(ParameterValue::String(s.clone())),
            serde_json::Value::Array(arr) => {
                let mut param_array = Vec::new();
                for item in arr {
                    param_array.push(Self::json_to_parameter_value(item)?);
                }
                Ok(ParameterValue::Array(param_array))
            }
            _ => Err(GenerationError::InvalidParameters("Unsupported parameter type".to_string())),
        }
    }
}

/// Default configuration for testing
impl Default for GenerationConfiguration {
    fn default() -> Self {
        Self {
            passes: vec![
                GenerationPassConfig {
                    id: "terrain_base".to_string(),
                    algorithm: "perlin_noise".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("scale".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.1).unwrap()));
                        params.insert("octaves".to_string(), serde_json::Value::Number(serde_json::Number::from(4)));
                        params.insert("persistence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()));
                        params
                    },
                    dependencies: Vec::new(),
                    condition: None,
                    input_layers: Vec::new(),
                    output_layers: vec!["heightmap".to_string()],
                }
            ],
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_configuration() {
        let config_json = r#"
        {
            "passes": [
                {
                    "id": "terrain",
                    "algorithm": "perlin_noise",
                    "parameters": {
                        "scale": 0.1,
                        "octaves": 4
                    },
                    "dependencies": [],
                    "input_layers": [],
                    "output_layers": ["heightmap"]
                }
            ],
            "metadata": {}
        }
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_json.as_bytes()).unwrap();
        
        let config = ConfigurationLoader::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.passes.len(), 1);
        assert_eq!(config.passes[0].id, "terrain");
        assert_eq!(config.passes[0].algorithm, "perlin_noise");
    }
    
    #[test]
    fn test_parameter_conversion() {
        let mut json_params = HashMap::new();
        json_params.insert("scale".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.1).unwrap()));
        json_params.insert("octaves".to_string(), serde_json::Value::Number(serde_json::Number::from(4)));
        json_params.insert("enabled".to_string(), serde_json::Value::Bool(true));
        
        let params = ConfigurationLoader::create_algorithm_parameters(&json_params).unwrap();
        
        let scale: f64 = params.get("scale").unwrap();
        let octaves: i64 = params.get("octaves").unwrap();
        let enabled: bool = params.get("enabled").unwrap();
        
        assert_eq!(scale, 0.1);
        assert_eq!(octaves, 4);
        assert_eq!(enabled, true);
    }
    
    #[test]
    fn test_validation_duplicate_ids() {
        let config = GenerationConfiguration {
            passes: vec![
                GenerationPassConfig {
                    id: "duplicate".to_string(),
                    algorithm: "test".to_string(),
                    parameters: HashMap::new(),
                    dependencies: Vec::new(),
                    condition: None,
                    input_layers: Vec::new(),
                    output_layers: Vec::new(),
                },
                GenerationPassConfig {
                    id: "duplicate".to_string(),
                    algorithm: "test2".to_string(),
                    parameters: HashMap::new(),
                    dependencies: Vec::new(),
                    condition: None,
                    input_layers: Vec::new(),
                    output_layers: Vec::new(),
                }
            ],
            metadata: HashMap::new(),
        };
        
        assert!(ConfigurationLoader::validate_configuration(&config).is_err());
    }
}
