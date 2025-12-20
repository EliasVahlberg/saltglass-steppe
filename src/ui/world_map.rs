//! World Map UI - displays the 64x64 world grid with biome colors and POI markers

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::game::world_map::{Biome, POI, Terrain, WorldMap, WORLD_WIDTH, WORLD_HEIGHT};

/// World map view state
#[derive(Default)]
pub struct WorldMapView {
    pub open: bool,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl WorldMapView {
    pub fn toggle(&mut self, player_wx: usize, player_wy: usize) {
        self.open = !self.open;
        if self.open {
            self.cursor_x = player_wx;
            self.cursor_y = player_wy;
        }
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        self.cursor_x = (self.cursor_x as i32 + dx).clamp(0, WORLD_WIDTH as i32 - 1) as usize;
        self.cursor_y = (self.cursor_y as i32 + dy).clamp(0, WORLD_HEIGHT as i32 - 1) as usize;
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

/// Render the world map view
pub fn render_world_map(
    frame: &mut Frame,
    area: Rect,
    world_map: &WorldMap,
    player_wx: usize,
    player_wy: usize,
    view: &WorldMapView,
) {
    let block = Block::default()
        .title(" World Map [M to close, arrows to move cursor] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate viewport (center on cursor)
    let view_width = inner.width as usize;
    let view_height = inner.height.saturating_sub(2) as usize; // Leave room for info
    
    let half_w = view_width / 2;
    let half_h = view_height / 2;
    
    let start_x = view.cursor_x.saturating_sub(half_w);
    let start_y = view.cursor_y.saturating_sub(half_h);
    let end_x = (start_x + view_width).min(WORLD_WIDTH);
    let end_y = (start_y + view_height).min(WORLD_HEIGHT);

    // Render map tiles
    for (screen_y, world_y) in (start_y..end_y).enumerate() {
        for (screen_x, world_x) in (start_x..end_x).enumerate() {
            let (biome, terrain, _elev, poi, resources, connected) = world_map.get(world_x, world_y);
            
            let (ch, fg) = if world_x == player_wx && world_y == player_wy {
                ('@', Color::White)
            } else if world_x == view.cursor_x && world_y == view.cursor_y {
                ('X', Color::LightYellow)
            } else if let Some((poi_ch, poi_color)) = poi_glyph(poi) {
                (poi_ch, poi_color)
            } else if connected.road {
                ('=', Color::Gray)
            } else if resources.water {
                ('~', Color::Blue)
            } else {
                (terrain_glyph(terrain), biome_color(biome))
            };

            let x = inner.x + screen_x as u16;
            let y = inner.y + screen_y as u16;
            if x < inner.x + inner.width && y < inner.y + inner.height.saturating_sub(2) {
                frame.render_widget(
                    Paragraph::new(ch.to_string()).style(Style::default().fg(fg)),
                    Rect::new(x, y, 1, 1),
                );
            }
        }
    }

    // Render info bar at bottom
    let (biome, terrain, _elev, poi, resources, _connected) = world_map.get(view.cursor_x, view.cursor_y);
    let poi_str = match poi {
        POI::None => "",
        POI::Town => " [Town]",
        POI::Dungeon => " [Dungeon]",
        POI::Landmark => " [Landmark]",
        POI::Shrine => " [Shrine]",
    };
    let res_str = if resources.water { " Water" } else { "" };
    let info = format!(
        "({},{}) {:?} {:?}{}{} | @ = You, X = Cursor",
        view.cursor_x, view.cursor_y, biome, terrain, poi_str, res_str
    );
    let info_y = inner.y + inner.height.saturating_sub(1);
    frame.render_widget(
        Paragraph::new(info).style(Style::default().fg(Color::Gray)),
        Rect::new(inner.x, info_y, inner.width, 1),
    );
}
