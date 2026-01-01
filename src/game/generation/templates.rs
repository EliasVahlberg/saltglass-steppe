use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use rand::Rng;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentTemplate {
    pub id: String,
    pub category: String,
    pub parameters: HashMap<String, Value>,
    pub variants: Vec<TemplateVariant>,
    pub inheritance: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateVariant {
    pub id: String,
    pub weight: f32,
    pub conditions: Vec<String>,
    pub overrides: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub variables: HashMap<String, Value>,
}

pub struct TemplateLibrary {
    templates: HashMap<String, ContentTemplate>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let templates: Vec<ContentTemplate> = serde_json::from_str(&data)?;
        
        let mut library = Self::new();
        for template in templates {
            library.add_template(template);
        }
        
        Ok(library)
    }

    pub fn add_template(&mut self, template: ContentTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn get_template(&self, id: &str) -> Option<&ContentTemplate> {
        self.templates.get(id)
    }

    pub fn instantiate<R: Rng>(
        &self,
        template_id: &str,
        context: &TemplateContext,
        rng: &mut R,
    ) -> Result<HashMap<String, Value>, String> {
        let template = self.get_template(template_id)
            .ok_or_else(|| format!("Template not found: {}", template_id))?;

        let mut result = self.resolve_inheritance(template)?;
        
        // Select variant if any exist
        if !template.variants.is_empty() {
            let variant = self.select_variant(&template.variants, context, rng)?;
            // Apply variant overrides
            for (key, value) in &variant.overrides {
                result.insert(key.clone(), value.clone());
            }
        }

        // Apply context variable substitution
        self.substitute_variables(&mut result, context)?;

        Ok(result)
    }

    fn resolve_inheritance(&self, template: &ContentTemplate) -> Result<HashMap<String, Value>, String> {
        let mut result = HashMap::new();

        // Apply parent template first if inheritance exists
        if let Some(parent_id) = &template.inheritance {
            let parent = self.get_template(parent_id)
                .ok_or_else(|| format!("Parent template not found: {}", parent_id))?;
            
            let parent_params = self.resolve_inheritance(parent)?;
            result.extend(parent_params);
        }

        // Apply current template parameters (overrides parent)
        result.extend(template.parameters.clone());

        Ok(result)
    }

    fn select_variant<'a, R: Rng>(
        &self,
        variants: &'a [TemplateVariant],
        context: &TemplateContext,
        rng: &mut R,
    ) -> Result<&'a TemplateVariant, String> {
        // Filter variants by conditions
        let valid_variants: Vec<&TemplateVariant> = variants
            .iter()
            .filter(|v| self.check_conditions(&v.conditions, context))
            .collect();

        if valid_variants.is_empty() {
            return Err("No valid variants found".to_string());
        }

        // Select by weight
        let total_weight: f32 = valid_variants.iter().map(|v| v.weight).sum();
        if total_weight <= 0.0 {
            return Ok(valid_variants[0]);
        }

        let mut roll = rng.gen_range(0.0..total_weight);
        for &variant in &valid_variants {
            if roll < variant.weight {
                return Ok(variant);
            }
            roll -= variant.weight;
        }

        Ok(valid_variants.last().unwrap())
    }

    fn check_conditions(&self, conditions: &[String], context: &TemplateContext) -> bool {
        for condition in conditions {
            if !self.evaluate_condition(condition, context) {
                return false;
            }
        }
        true
    }

    fn evaluate_condition(&self, condition: &str, context: &TemplateContext) -> bool {
        // Simple condition evaluation: "key=value" or "key"
        if let Some((key, expected)) = condition.split_once('=') {
            if let Some(value) = context.variables.get(key) {
                return value.as_str().map_or(false, |v| v == expected);
            }
            false
        } else {
            // Check if key exists and is truthy
            context.variables.get(condition)
                .map_or(false, |v| !v.is_null() && v != &Value::Bool(false))
        }
    }

    fn substitute_variables(&self, params: &mut HashMap<String, Value>, context: &TemplateContext) -> Result<(), String> {
        for (_, value) in params.iter_mut() {
            self.substitute_value(value, context)?;
        }
        Ok(())
    }

    fn substitute_value(&self, value: &mut Value, context: &TemplateContext) -> Result<(), String> {
        match value {
            Value::String(s) => {
                *s = self.substitute_string(s, context)?;
            }
            Value::Object(obj) => {
                for (_, v) in obj.iter_mut() {
                    self.substitute_value(v, context)?;
                }
            }
            Value::Array(arr) => {
                for v in arr.iter_mut() {
                    self.substitute_value(v, context)?;
                }
            }
            _ => {} // No substitution needed for other types
        }
        Ok(())
    }

    fn substitute_string(&self, s: &str, context: &TemplateContext) -> Result<String, String> {
        let mut result = s.to_string();
        
        // Simple variable substitution: ${variable_name}
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 2..start + end];
                let replacement = context.variables.get(var_name)
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                result.replace_range(start..start + end + 1, replacement);
            } else {
                break; // Malformed variable reference
            }
        }
        
        Ok(result)
    }
}
