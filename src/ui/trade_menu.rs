//! Trading menu UI

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
};

use crate::game::{
    trading::{TradeInterface},
    get_item_def,
};
use crate::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeMode {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct TradeMenu {
    pub active: bool,
    pub mode: TradeMode,
    pub selected_index: usize,
    pub trader_id: Option<String>,
    pub interface: Option<TradeInterface>,
}

impl Default for TradeMenu {
    fn default() -> Self {
        Self {
            active: false,
            mode: TradeMode::Buy,
            selected_index: 0,
            trader_id: None,
            interface: None,
        }
    }
}

impl TradeMenu {
    pub fn open(&mut self, trader_id: String, interface: TradeInterface) {
        self.active = true;
        self.mode = TradeMode::Buy;
        self.selected_index = 0;
        self.trader_id = Some(trader_id);
        self.interface = Some(interface);
    }
    
    pub fn close(&mut self) {
        self.active = false;
        self.trader_id = None;
        self.interface = None;
    }
    
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            TradeMode::Buy => TradeMode::Sell,
            TradeMode::Sell => TradeMode::Buy,
        };
        self.selected_index = 0;
    }
    
    pub fn navigate(&mut self, dy: i32, list_len: usize) {
        if list_len == 0 {
            self.selected_index = 0;
            return;
        }
        
        let new_index = self.selected_index as i32 + dy;
        self.selected_index = new_index.rem_euclid(list_len as i32) as usize;
    }
}

pub fn render_trade_menu(
    f: &mut Frame,
    menu: &TradeMenu,
    state: &GameState,
) {
    if !menu.active || menu.interface.is_none() {
        return;
    }
    
    let trade_interface = menu.interface.as_ref().unwrap();

    let area = f.area();
    let block = Block::default()
        .title(format!(" Trading with {} ", trade_interface.trader_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
        
    // Center popup
    let popup_area = centered_rect(80, 80, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header (Mode + Money)
            Constraint::Min(1),    // List
            Constraint::Length(3), // Footer (Controls)
        ])
        .split(popup_area);
        
    // Header
    let mode_str = match menu.mode {
        TradeMode::Buy => "BUY MODE (Buying from trader)",
        TradeMode::Sell => "SELL MODE (Selling to trader)",
    };
    
    let header_text = vec![
        Line::from(vec![
            Span::styled(mode_str, Style::default().fg(if menu.mode == TradeMode::Buy { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(format!("Salt Scrip: {}", state.salt_scrip), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Reputation: "),
            // TODO: Show reputation with this faction
            Span::raw("Neutral"), 
        ]),
    ];
    
    f.render_widget(Paragraph::new(header_text), chunks[0]);
    
    // List
    let items: Vec<ListItem> = match menu.mode {
        TradeMode::Buy => {
            trade_interface.available_items.iter().enumerate().map(|(i, item)| {
                let def = get_item_def(&item.item_id).unwrap();
                let style = if i == menu.selected_index {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default()
                };
                
                let stock_str = if item.stock < 0 { "∞".to_string() } else { item.stock.to_string() };
                let name = format!("{: <20}", def.name);
                let price = format!("{: >4} scrip", item.price);
                
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", def.glyph), Style::default().fg(Color::White)),
                    Span::styled(format!("{} | {} | Stock: {}", name, price, stock_str), style),
                ]))
            }).collect()
        },
        TradeMode::Sell => {
            state.inventory.iter().enumerate().map(|(i, item_id)| {
                let def = get_item_def(item_id).unwrap();
                let style = if i == menu.selected_index {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default()
                };
                
                let sell_price = (def.value as f32 * trade_interface.sell_price_multiplier) as u32;
                let name = format!("{: <20}", def.name);
                let price = format!("{: >4} scrip", sell_price);
                
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", def.glyph), Style::default().fg(Color::White)),
                    Span::styled(format!("{} | {}", name, price), style),
                ]))
            }).collect()
        }
    };
    
    f.render_widget(List::new(items).block(Block::default().borders(Borders::TOP)), chunks[1]);
    
    // Footer
    let controls = "Tab: Switch Mode | Enter: Trade | Esc: Close";
    f.render_widget(Paragraph::new(controls).alignment(Alignment::Center), chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
