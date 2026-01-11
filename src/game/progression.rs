//! Data-driven character progression system

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct ProgressionFile {
    level_thresholds: Vec<u32>,
    #[serde(default = "default_stat_points")]
    stat_points_per_level: i32,
    stat_growth: HashMap<String, i32>,
}

fn default_stat_points() -> i32 {
    3
}

static PROGRESSION: Lazy<ProgressionFile> = Lazy::new(|| {
    let data = include_str!("../../data/progression.json");
    serde_json::from_str(data).expect("Failed to parse progression.json")
});

pub fn xp_for_level(level: u32) -> u32 {
    PROGRESSION
        .level_thresholds
        .get(level as usize)
        .copied()
        .unwrap_or(u32::MAX)
}

pub fn stat_points_per_level() -> i32 {
    PROGRESSION.stat_points_per_level
}

pub fn stat_growth(stat: &str) -> i32 {
    PROGRESSION.stat_growth.get(stat).copied().unwrap_or(0)
}

pub fn max_level() -> u32 {
    PROGRESSION.level_thresholds.len().saturating_sub(1) as u32
}
