use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::GameState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssueReport {
    pub id: String,
    pub timestamp: String,
    pub description: String,
    pub reproduction_steps: Vec<String>,
    pub expected_behavior: String,
    pub actual_behavior: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub gamestate_file: Option<String>,
    pub system_info: SystemInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum IssueSeverity {
    Critical,
    High,
    #[default]
    Medium,
    Low,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum IssueCategory {
    #[default]
    Gameplay,
    UI,
    Performance,
    Save,
    Combat,
    AI,
    Map,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub version: String,
    pub seed: u64,
    pub turn: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    pub player_pos: (i32, i32),
    pub player_hp: (i32, i32),
    pub turn: u32,
    pub seed: u64,
    pub enemies_count: usize,
    pub items_count: usize,
    pub storm_intensity: u8,
    pub storm_turns: u32,
    pub memory_usage: String,
    pub performance_metrics: HashMap<String, f64>,
}

impl GameState {
    pub fn create_issue_report(&self, description: String, steps: Vec<String>, expected: String, actual: String, severity: IssueSeverity, category: IssueCategory) -> IssueReport {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let id = format!("issue_{}", timestamp);
        
        IssueReport {
            id: id.clone(),
            timestamp,
            description,
            reproduction_steps: steps,
            expected_behavior: expected,
            actual_behavior: actual,
            severity,
            category,
            gamestate_file: Some(format!("{}.ron", id)),
            system_info: SystemInfo {
                os: std::env::consts::OS.to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                seed: self.seed,
                turn: self.turn,
            },
        }
    }

    pub fn save_debug_state(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all("debug_states")?;
        let path = format!("debug_states/{}", filename);
        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        fs::write(&path, serialized)?;
        Ok(())
    }

    pub fn load_debug_state(filename: &str) -> Result<GameState, Box<dyn std::error::Error>> {
        let path = if filename.starts_with("debug_states/") {
            filename.to_string()
        } else {
            format!("debug_states/{}", filename)
        };
        
        let content = fs::read_to_string(&path)?;
        let state: GameState = ron::from_str(&content)?;
        Ok(state)
    }

    pub fn get_debug_info(&self) -> DebugInfo {
        let mut metrics = HashMap::new();
        metrics.insert("fov_calculation_time".to_string(), 0.0); // Placeholder
        metrics.insert("ai_processing_time".to_string(), 0.0);   // Placeholder
        
        DebugInfo {
            player_pos: (self.player_x, self.player_y),
            player_hp: (self.player_hp, self.player_max_hp),
            turn: self.turn,
            seed: self.seed,
            enemies_count: self.enemies.len(),
            items_count: self.items.len(),
            storm_intensity: self.storm.intensity,
            storm_turns: self.storm.turns_until,
            memory_usage: format!("{}KB", std::mem::size_of_val(self) / 1024),
            performance_metrics: metrics,
        }
    }

    pub fn save_issue_report(&self, report: &IssueReport) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all("issue_reports")?;
        fs::create_dir_all("debug_states")?;
        
        // Save the issue report
        let report_path = format!("issue_reports/{}.json", report.id);
        let report_json = serde_json::to_string_pretty(report)?;
        fs::write(&report_path, report_json)?;
        
        // Save the gamestate if specified
        if let Some(gamestate_file) = &report.gamestate_file {
            self.save_debug_state(gamestate_file)?;
        }
        
        Ok(())
    }

    pub fn list_debug_states() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut states = Vec::new();
        
        if Path::new("debug_states").exists() {
            for entry in fs::read_dir("debug_states")? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".ron") {
                        states.push(name.to_string());
                    }
                }
            }
        }
        
        states.sort();
        Ok(states)
    }

    pub fn list_issue_reports() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut reports = Vec::new();
        
        if Path::new("issue_reports").exists() {
            for entry in fs::read_dir("issue_reports")? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        reports.push(name.to_string());
                    }
                }
            }
        }
        
        reports.sort();
        Ok(reports)
    }
}
