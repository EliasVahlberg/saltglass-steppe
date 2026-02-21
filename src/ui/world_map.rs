//! World Map UI - displays the 64x64 world grid with biome colors and POI markers

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::game::GameState;
use crate::game::world_map::{Biome, POI, Terrain, WORLD_HEIGHT, WORLD_WIDTH, WorldMap};

/// Check if there's an active quest objective at the given world coordinates
fn has_quest_objective_at(state: &GameState, world_x: usize, world_y: usize) -> bool {
    state.player.quest_log.active.iter().any(|quest| {
        quest.objectives.iter().any(|obj| {
            !obj.completed &&
            quest.def().map_or(false, |def| {
                def.objectives.iter().any(|quest_obj| {
                    quest_obj.id == obj.objective_id &&
                    matches!(quest_obj.objective_type, crate::game::quest::ObjectiveType::Reach { x, y }
                        if x as usize == world_x && y as usize == world_y)
                })
            })
        })
    })
}

/// World map view state
#[derive(Default)]
pub struct WorldMapView {
    pub open: bool,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub inspect_mode: bool,
    pub show_faction_overlay: bool,
}

impl WorldMapView {
    pub fn toggle(&mut self, player_wx: usize, player_wy: usize) {
        self.open = !self.open;
        if self.open {
            self.cursor_x = player_wx;
            self.cursor_y = player_wy;
            self.inspect_mode = false;
        }
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        self.cursor_x = (self.cursor_x as i32 + dx).clamp(0, WORLD_WIDTH as i32 - 1) as usize;
        self.cursor_y = (self.cursor_y as i32 + dy).clamp(0, WORLD_HEIGHT as i32 - 1) as usize;
    }

    pub fn toggle_inspect(&mut self) {
        self.inspect_mode = !self.inspect_mode;
    }

    pub fn toggle_faction_overlay(&mut self) {
        self.show_faction_overlay = !self.show_faction_overlay;
    }
}

/// Get color for biome
fn biome_color(biome: Biome) -> Color {
    match biome {
        Biome::Desert => Color::Yellow,
        Biome::Saltflat => Color::White,
        Biome::Scrubland => Color::Green,
        Biome::Oasis => Color::Cyan,
        Biome::Ruins => Color::Magenta,
    }
}

/// Get color for faction
fn faction_color(faction_id: &str) -> Color {
    use crate::game::faction;
    if let Some(faction) = faction::get_faction(faction_id) {
        match faction.color.as_str() {
            "Cyan" => Color::Cyan,
            "Magenta" => Color::Magenta,
            "Yellow" => Color::Yellow,
            "White" => Color::White,
            "Red" => Color::Red,
            "Blue" => Color::Blue,
            "Green" => Color::Green,
            _ => Color::Gray,
        }
    } else {
        Color::Gray
    }
}

/// Get glyph for POI
fn poi_glyph(poi: POI) -> Option<(char, Color)> {
    match poi {
        POI::None => None,
        POI::Town => Some(('T', Color::Yellow)),
        POI::Dungeon => Some(('D', Color::Red)),
        POI::Landmark => Some(('L', Color::Blue)),
        POI::Shrine => Some(('S', Color::Cyan)),
    }
}

/// Get glyph for terrain
fn terrain_glyph(terrain: Terrain) -> char {
    match terrain {
        Terrain::Flat => '.',
        Terrain::Hills => '^',
        Terrain::Dunes => '~',
        Terrain::Canyon => 'v',
        Terrain::Mesa => '#',
    }
}

/// Get color intensity based on level (for background/border)
fn level_color(level: u32) -> Color {
    match level {
        1 => Color::DarkGray,     // Safe areas
        2..=3 => Color::Gray,     // Low threat
        4..=6 => Color::Yellow,   // Medium threat
        7..=8 => Color::LightRed, // High threat
        9..=10 => Color::Red,     // Extreme threat
        _ => Color::Red,
    }
}

/// Render the world map view
pub fn render_world_map(
    frame: &mut Frame,
    area: Rect,
    world_map: &WorldMap,
    player_wx: usize,
    player_wy: usize,
    view: &WorldMapView,
    state: &GameState,
) {
    let mode_str = if view.inspect_mode { "INSPECT" } else { "TRAVEL" };
    let faction_str = if view.show_faction_overlay { " [FACTION OVERLAY]" } else { "" };
    let title = if state.world.world_map_target.is_some() {
        format!(" World Map [{}]{} [Target Set - O auto-move, T clear, F factions] ", mode_str, faction_str)
    } else if view.inspect_mode {
        format!(" World Map [INSPECT]{} [X travel mode, T set target, F factions] ", faction_str)
    } else {
        format!(" World Map [TRAVEL]{} [X inspect mode, arrows move, F factions] ", faction_str)
    };
    
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate viewport (center on cursor)
    let view_width = inner.width as usize;
    let view_height = inner.height.saturating_sub(2) as usize; // Leave room for 2 info lines

    let half_w = view_width / 2;
    let half_h = view_height / 2;

    let start_x = view.cursor_x.saturating_sub(half_w);
    let start_y = view.cursor_y.saturating_sub(half_h);
    let end_x = (start_x + view_width).min(WORLD_WIDTH);
    let end_y = (start_y + view_height).min(WORLD_HEIGHT);

    // Render map tiles
    let path_set: std::collections::HashSet<(usize, usize)> = 
        state.world.world_map_path.iter().copied().collect();
    
    for (screen_y, world_y) in (start_y..end_y).enumerate() {
        for (screen_x, world_x) in (start_x..end_x).enumerate() {
            let (biome, terrain, _elev, poi, resources, connected, level) =
                world_map.get(world_x, world_y);

            let is_on_path = path_set.contains(&(world_x, world_y));
            let show_cursor = view.inspect_mode || state.world.world_map_target.is_some();
            
            // Get base character and color
            let (ch, base_fg) = if world_x == player_wx && world_y == player_wy {
                ('@', Color::White)
            } else if show_cursor && world_x == view.cursor_x && world_y == view.cursor_y {
                ('X', Color::LightYellow)
            } else if has_quest_objective_at(state, world_x, world_y) {
                // Highlight tiles with quest objectives
                let (base_ch, _) = if let Some((poi_ch, _)) = poi_glyph(poi) {
                    (poi_ch, Color::White)
                } else if connected.road {
                    ('=', Color::Gray)
                } else if resources.water {
                    ('~', Color::Blue)
                } else {
                    (terrain_glyph(terrain), biome_color(biome))
                };
                (base_ch, Color::LightGreen)
            } else if let Some((poi_ch, poi_color)) = poi_glyph(poi) {
                (poi_ch, poi_color)
            } else if connected.road {
                ('=', Color::Gray)
            } else if resources.water {
                ('~', Color::Blue)
            } else {
                (terrain_glyph(terrain), biome_color(biome))
            };

            // Override with faction color if overlay is active
            let fg = if view.show_faction_overlay {
                if let Some(faction_id) = world_map.get_faction_territory(world_x, world_y) {
                    faction_color(faction_id)
                } else {
                    Color::DarkGray // Neutral territory
                }
            } else {
                base_fg
            };

            // Determine background color (path takes priority over threat level)
            let bg = if is_on_path {
                Some(Color::DarkGray)
            } else if level > 1 && !view.show_faction_overlay {
                Some(level_color(level))
            } else {
                None
            };
            let style = if let Some(bg_color) = bg {
                Style::default().fg(fg).bg(bg_color)
            } else {
                Style::default().fg(fg)
            };

            let x = inner.x + screen_x as u16;
            let y = inner.y + screen_y as u16;
            if x < inner.x + inner.width && y < inner.y + inner.height.saturating_sub(2) {
                frame.render_widget(
                    Paragraph::new(ch.to_string()).style(style),
                    Rect::new(x, y, 1, 1),
                );
            }
        }
    }

    // Render info bar at bottom (2 lines)
    // Determine which position to show info for
    let (info_x, info_y_coord) = if view.inspect_mode {
        (view.cursor_x, view.cursor_y)
    } else {
        (player_wx, player_wy)
    };
    
    let (biome, terrain, _elev, poi, resources, _connected, level) =
        world_map.get(info_x, info_y_coord);
    let poi_str = match poi {
        POI::None => "",
        POI::Town => " Town",
        POI::Dungeon => " Dungeon",
        POI::Landmark => " Landmark",
        POI::Shrine => " Shrine",
    };
    let res_str = if resources.water { " Water" } else { "" };
    let threat_str = match level {
        1 => "Safe",
        2..=3 => "Low Threat",
        4..=6 => "Medium Threat",
        7..=8 => "High Threat",
        9..=10 => "EXTREME THREAT",
        _ => "Unknown Threat",
    };
    
    // Line 1: Legend and tile info
    let legend = if view.inspect_mode || state.world.world_map_target.is_some() {
        "@ = You, X = Cursor"
    } else {
        "@ = You"
    };
    let tile_info = format!(
        "({},{}) {:?} {:?}{}{} [{}] Level {}",
        info_x, info_y_coord, biome, terrain, poi_str, res_str, threat_str, level
    );
    let info_line = format!("{} | {}", legend, tile_info);
    let info_y = inner.y + inner.height.saturating_sub(2);
    frame.render_widget(
        Paragraph::new(info_line).style(Style::default().fg(Color::Gray)),
        Rect::new(inner.x, info_y, inner.width, 1),
    );
    
    // Line 2: Turn counter and recent log message
    let turn_info = format!("Turn: {} | ", state.turn);
    let recent_log = state.messages.last()
        .map(|msg| msg.text.as_str())
        .unwrap_or("");
    let status_line = format!("{}{}", turn_info, recent_log);
    let status_y = inner.y + inner.height.saturating_sub(1);
    frame.render_widget(
        Paragraph::new(status_line).style(Style::default().fg(Color::White)),
        Rect::new(inner.x, status_y, inner.width, 1),
    );
}
