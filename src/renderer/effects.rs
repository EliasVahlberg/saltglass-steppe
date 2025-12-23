//! Visual effects processing pipeline

use ratatui::prelude::*;
use crate::GameState;
use super::config::RenderConfig;

/// Handles visual effects processing and application
pub struct EffectsRenderer {
    _config: RenderConfig,
}

impl EffectsRenderer {
    pub fn new(config: &RenderConfig) -> Self {
        Self {
            _config: config.clone(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: &RenderConfig) {
        self._config = config.clone();
    }

    /// Apply visual effects to rendered spans
    pub fn apply_effects(
        &self,
        _state: &GameState,
        tile_spans: Vec<Vec<Span>>,
        entity_spans: Vec<Vec<Option<Span>>>,
        _frame_count: u64,
        _cam_x: i32,
        _cam_y: i32,
        view_width: i32,
        view_height: i32,
    ) -> Vec<Vec<Span<'static>>> {
        // Convert to owned spans to avoid lifetime issues
        let height = view_height.min(tile_spans.len() as i32) as usize;
        let mut result = Vec::with_capacity(height);
        
        for y in 0..height {
            let width = view_width.min(tile_spans[y].len() as i32) as usize;
            let mut row = Vec::with_capacity(width);
            
            for x in 0..width {
                let span = if let Some(ref entity_span) = entity_spans[y][x] {
                    // Create owned span from entity
                    Span::styled(
                        entity_span.content.to_string(),
                        entity_span.style
                    )
                } else {
                    // Create owned span from tile
                    Span::styled(
                        tile_spans[y][x].content.to_string(),
                        tile_spans[y][x].style
                    )
                };
                row.push(span);
            }
            result.push(row);
        }

        result
    }
}
