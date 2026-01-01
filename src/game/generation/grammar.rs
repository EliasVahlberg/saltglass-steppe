use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Grammar {
    pub rules: HashMap<String, GrammarRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrammarRule {
    pub expansions: Vec<String>,
    pub weights: Option<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct GrammarContext {
    pub variables: HashMap<String, String>,
}

impl Grammar {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let grammar: Grammar = serde_json::from_str(&data)?;
        Ok(grammar)
    }

    pub fn generate<R: Rng>(
        &self,
        start_rule: &str,
        context: &GrammarContext,
        rng: &mut R,
    ) -> Result<String, String> {
        self.expand_rule(start_rule, context, rng, 0)
    }

    fn expand_rule<R: Rng>(
        &self,
        rule_name: &str,
        context: &GrammarContext,
        rng: &mut R,
        depth: usize,
    ) -> Result<String, String> {
        // Prevent infinite recursion
        if depth > 10 {
            return Err("Maximum recursion depth exceeded".to_string());
        }

        // Check if it's a variable reference
        if let Some(value) = context.variables.get(rule_name) {
            return Ok(value.clone());
        }

        // Get the rule
        let rule = self.rules.get(rule_name)
            .ok_or_else(|| format!("Rule not found: {}", rule_name))?;

        // Select expansion
        let expansion = self.select_expansion(rule, rng)?;

        // Process the expansion
        self.process_expansion(expansion, context, rng, depth + 1)
    }

    fn select_expansion<'a, R: Rng>(&self, rule: &'a GrammarRule, rng: &mut R) -> Result<&'a str, String> {
        if rule.expansions.is_empty() {
            return Err("Rule has no expansions".to_string());
        }

        if let Some(weights) = &rule.weights {
            if weights.len() != rule.expansions.len() {
                return Err("Weights count doesn't match expansions count".to_string());
            }

            let total_weight: f32 = weights.iter().sum();
            if total_weight <= 0.0 {
                return Ok(&rule.expansions[0]);
            }

            let mut roll = rng.gen_range(0.0..total_weight);
            for (i, &weight) in weights.iter().enumerate() {
                if roll < weight {
                    return Ok(&rule.expansions[i]);
                }
                roll -= weight;
            }

            Ok(rule.expansions.last().unwrap())
        } else {
            // Uniform selection
            let index = rng.gen_range(0..rule.expansions.len());
            Ok(&rule.expansions[index])
        }
    }

    fn process_expansion<R: Rng>(
        &self,
        expansion: &str,
        context: &GrammarContext,
        rng: &mut R,
        depth: usize,
    ) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = expansion.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '<' {
                // Find the closing >
                let mut rule_name = String::new();
                let mut found_close = false;

                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '>' {
                        found_close = true;
                        break;
                    }
                    rule_name.push(inner_ch);
                }

                if found_close && !rule_name.is_empty() {
                    // Recursively expand the rule
                    let expanded = self.expand_rule(&rule_name, context, rng, depth)?;
                    result.push_str(&expanded);
                } else {
                    // Malformed rule reference, treat as literal
                    result.push('<');
                    result.push_str(&rule_name);
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

pub fn load_grammars_from_directory(dir_path: &str) -> Result<HashMap<String, Grammar>, Box<dyn std::error::Error>> {
    let mut grammars = HashMap::new();
    
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(grammar) = Grammar::load_from_file(&path.to_string_lossy()) {
                            grammars.insert(file_stem.to_string(), grammar);
                        }
                    }
                }
            }
        }
    }
    
    Ok(grammars)
}
