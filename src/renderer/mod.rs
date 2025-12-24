//! Modular rendering system for TUI RPG
//! 
//! This module provides a data-driven, modular rendering system that separates
//! concerns between lighting, visual effects, entity rendering, and tile rendering.

pub mod config;
pub mod lighting;
pub mod effects;
pub mod entities;
pub mod tiles;
pub mod camera;
pub mod performance;
pub mod particles;
pub mod animations;
pub mod themes;

use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use crate::GameState;
use self::{
    config::{RenderConfig, parse_color},
    lighting::LightingRenderer,
    effects::EffectsRenderer,
    entities::EntityRenderer,
    tiles::TileRenderer,
    camera::Camera,
    performance::{FrameLimiter, ViewportCuller},
    particles::{ParticleSystem, ParticleType},
    animations::AnimationSystem,
    themes::ThemeManager,
};

/// Main renderer that coordinates all rendering subsystems
pub struct Renderer {
    config: RenderConfig,
    lighting: LightingRenderer,
    effects: EffectsRenderer,
    entities: EntityRenderer,
    tiles: TileRenderer,
    camera: Camera,
    frame_limiter: FrameLimiter,
    viewport_culler: ViewportCuller,
    particle_system: ParticleSystem,
    animation_system: AnimationSystem,
    theme_manager: ThemeManager,
}

impl Renderer {
    /// Create a new renderer with configuration loaded from JSON
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = RenderConfig::load("data/render_config.json")?;
        
        // Load theme manager
        let theme_manager = ThemeManager::load_from_file("data/themes.json")
            .unwrap_or_else(|_| ThemeManager::new());
        
        Ok(Self {
            lighting: LightingRenderer::new(&config),
            effects: EffectsRenderer::new(&config),
            entities: EntityRenderer::new(&config),
            tiles: TileRenderer::new(&config),
            camera: Camera::new(),
            frame_limiter: FrameLimiter::new(config.performance.target_fps),
            viewport_culler: ViewportCuller::new(),
            particle_system: ParticleSystem::new(config.particles.clone()),
            animation_system: AnimationSystem::new(config.visual_animations.clone()),
            theme_manager,
            config,
        })
    }

    /// Render the main game view
    pub fn render_game(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        state: &GameState,
        frame_count: u64,
        look_cursor: Option<(i32, i32)>,
    ) {
        let title = Line::from(format!(" Turn {} ", state.turn));
        let block = Block::default().title(title).borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Update camera position
        self.camera.update(state.player_x, state.player_y, inner.width as i32, inner.height as i32);
        let (cam_x, cam_y) = self.camera.position();

        // Update particle system
        self.particle_system.update(1.0 / 60.0); // Assume 60 FPS for now

        // Update animation system
        self.animation_system.update();

        // Get screen shake offset
        let (shake_x, shake_y) = self.animation_system.get_screen_offset();
        let adjusted_cam_x = cam_x + shake_x as i32;
        let adjusted_cam_y = cam_y + shake_y as i32;

        // Get viewport bounds for culling
        let _viewport_bounds = self.viewport_culler.get_bounds(
            adjusted_cam_x, adjusted_cam_y, inner.width as i32, inner.height as i32
        );

        // Calculate lighting if enabled (only for visible area)
        let light_map = if self.config.lighting.enabled {
            self.lighting.calculate_lighting(state)
        } else {
            vec![255; (state.map.width * state.map.height) as usize]
        };

        // Render tiles first (background layer)
        let tile_spans = self.tiles.render_tiles(
            state,
            &light_map,
            frame_count,
            adjusted_cam_x,
            adjusted_cam_y,
            inner.width as i32,
            inner.height as i32,
        );

        // Render entities (foreground layer)
        let entity_spans = self.entities.render_entities(
            state,
            &light_map,
            frame_count,
            adjusted_cam_x,
            adjusted_cam_y,
            inner.width as i32,
            inner.height as i32,
        );

        // Apply visual effects
        let mut final_spans = self.effects.apply_effects(
            state,
            tile_spans,
            entity_spans,
            frame_count,
            adjusted_cam_x,
            adjusted_cam_y,
            inner.width as i32,
            inner.height as i32,
        );

        // Render particles on top of everything else
        self.render_particles(&mut final_spans, adjusted_cam_x, adjusted_cam_y, inner.width as i32, inner.height as i32);

        // Apply animation effects to all spans
        for row in &mut final_spans {
            for span in row {
                span.style = self.animation_system.get_combined_style(span.style);
            }
        }

        // Apply look cursor highlighting
        let final_spans = if let Some((lx, ly)) = look_cursor {
            let screen_x = lx - adjusted_cam_x;
            let screen_y = ly - adjusted_cam_y;
            
            if screen_y >= 0 && screen_y < final_spans.len() as i32 &&
               screen_x >= 0 && screen_x < final_spans[screen_y as usize].len() as i32 {
                let mut spans = final_spans;
                let content = spans[screen_y as usize][screen_x as usize].content.clone();
                let style = spans[screen_y as usize][screen_x as usize].style;
                spans[screen_y as usize][screen_x as usize] = Span::styled(
                    content,
                    style.bg(parse_color(&self.config.colors.ui.look_cursor.bg))
                         .fg(parse_color(&self.config.colors.ui.look_cursor.fg))
                );
                spans
            } else {
                final_spans
            }
        } else {
            final_spans
        };

        // Convert to lines and render
        let lines: Vec<Line> = final_spans.into_iter().map(Line::from).collect();
        frame.render_widget(Paragraph::new(lines), inner);
    }

    /// Reload configuration from file
    pub fn reload_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = RenderConfig::load("data/render_config.json")?;
        self.lighting.update_config(&self.config);
        self.effects.update_config(&self.config);
        self.entities.update_config(&self.config);
        self.tiles.update_config(&self.config);
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// Limit frame rate to prevent excessive CPU usage
    pub fn limit_frame_rate(&mut self) {
        self.frame_limiter.limit();
    }

    /// Set target FPS for frame rate limiting
    pub fn set_fps(&mut self, fps: u32) {
        self.frame_limiter.set_fps(fps);
    }

    /// Get current target FPS
    pub fn fps(&self) -> u32 {
        self.frame_limiter.fps()
    }

    /// Render particles onto the span grid
    fn render_particles(&self, spans: &mut Vec<Vec<Span<'static>>>, cam_x: i32, cam_y: i32, view_width: i32, view_height: i32) {
        for particle in self.particle_system.particles() {
            let screen_x = particle.position.0 as i32 - cam_x;
            let screen_y = particle.position.1 as i32 - cam_y;
            
            // Check if particle is within viewport
            if screen_x >= 0 && screen_x < view_width && 
               screen_y >= 0 && screen_y < view_height {
                let x = screen_x as usize;
                let y = screen_y as usize;
                
                if y < spans.len() && x < spans[y].len() {
                    // Apply brightness to particle color
                    let mut color = particle.color;
                    if particle.brightness < 1.0 {
                        color = self.dim_particle_color(color, particle.brightness);
                    }
                    
                    // Create particle span
                    let particle_span = Span::styled(
                        particle.character.to_string(),
                        Style::default().fg(color)
                    );
                    
                    spans[y][x] = particle_span;
                }
            }
        }
    }

    /// Dim particle color based on brightness
    fn dim_particle_color(&self, color: Color, brightness: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => Color::Rgb(
                (r as f32 * brightness) as u8,
                (g as f32 * brightness) as u8,
                (b as f32 * brightness) as u8,
            ),
            _ => if brightness < 0.5 { Color::DarkGray } else { color },
        }
    }

    /// Add a particle effect at a specific location
    pub fn add_particle_effect(&mut self, x: f32, y: f32, effect_type: ParticleType) {
        self.particle_system.add_particle(x, y, effect_type);
    }

    /// Clear all particles
    pub fn clear_particles(&mut self) {
        self.particle_system.clear();
    }

    /// Trigger a blink animation effect
    pub fn add_blink_effect(&mut self) {
        self.animation_system.add_blink();
    }

    /// Trigger a glow animation effect
    pub fn add_glow_effect(&mut self) {
        self.animation_system.add_glow();
    }

    /// Trigger a screen shake effect
    pub fn add_screen_shake(&mut self) {
        self.animation_system.add_screen_shake();
    }

    /// Set the active theme
    pub fn set_theme(&mut self, theme_name: &str) -> bool {
        if let Some(theme) = self.theme_manager.get_theme(theme_name) {
            let theme_colors = theme.colors.clone();
            if self.theme_manager.set_active_theme(theme_name) {
                self.apply_theme_to_config(&theme_colors);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get list of available themes
    pub fn list_themes(&self) -> Vec<String> {
        self.theme_manager.list_themes().iter().map(|t| t.name.clone()).collect()
    }

    /// Get current active theme name
    pub fn get_active_theme(&self) -> String {
        self.theme_manager.get_active_theme()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "classic".to_string())
    }

    /// Apply theme colors to the render config
    fn apply_theme_to_config(&mut self, theme_colors: &themes::ThemeColors) {
        // Update entity colors
        self.config.colors.entities.player.base = theme_colors.entities.player.clone();
        
        // Update tile colors
        self.config.colors.tiles.floor = theme_colors.tiles.floor.clone();
        self.config.colors.tiles.wall = theme_colors.tiles.wall.clone();
        self.config.colors.tiles.glass.base = theme_colors.tiles.glass.clone();
        self.config.colors.tiles.stairs_down = theme_colors.tiles.stairs.clone();
        self.config.colors.tiles.stairs_up = theme_colors.tiles.stairs.clone();
        self.config.colors.tiles.world_exit = theme_colors.tiles.world_exit.clone();
        
        // Update UI colors
        self.config.colors.ui.revealed_tile = theme_colors.ui.revealed_tile.clone();
        self.config.colors.ui.look_cursor.bg = theme_colors.ui.look_cursor_bg.clone();
        self.config.colors.ui.look_cursor.fg = theme_colors.ui.look_cursor_fg.clone();
        self.config.colors.ui.hit_flash.bg = theme_colors.ui.hit_flash_bg.clone();
        self.config.colors.ui.hit_flash.fg = theme_colors.ui.hit_flash_fg.clone();
    }
}

/// Render context passed to all rendering subsystems
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub frame_count: u64,
    pub camera_x: i32,
    pub camera_y: i32,
    pub view_width: i32,
    pub view_height: i32,
    pub god_view: bool,
}

impl RenderContext {
    pub fn new(frame_count: u64, cam_x: i32, cam_y: i32, view_w: i32, view_h: i32, god_view: bool) -> Self {
        Self {
            frame_count,
            camera_x: cam_x,
            camera_y: cam_y,
            view_width: view_w,
            view_height: view_h,
            god_view,
        }
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let screen_x = x - self.camera_x;
        let screen_y = y - self.camera_y;
        
        if screen_x >= 0 && screen_x < self.view_width && 
           screen_y >= 0 && screen_y < self.view_height {
            Some((screen_x, screen_y))
        } else {
            None
        }
    }
}
