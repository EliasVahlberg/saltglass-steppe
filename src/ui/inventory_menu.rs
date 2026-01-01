//! Fullscreen inventory menu with equipment display

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::game::{equipment::EquipSlot, get_item_def, inspect::inspect_item, Equipment};

/// Menu focus panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuPanel {
    Inventory,
    Equipment,
}

/// Inventory menu state
#[derive(Debug)]
pub struct InventoryMenu {
    pub active: bool,
    pub panel: MenuPanel,
    pub inv_index: usize,
    pub equip_index: usize,
    pub inspect_item: Option<String>,
}

impl Default for InventoryMenu {
    fn default() -> Self {
        Self { active: false, panel: MenuPanel::Inventory, inv_index: 0, equip_index: 0, inspect_item: None }
    }
}

impl InventoryMenu {
    pub fn open(&mut self) {
        self.active = true;
        self.panel = MenuPanel::Inventory;
        self.inv_index = 0;
        self.equip_index = 0;
        self.inspect_item = None;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.inspect_item = None;
    }

    pub fn navigate(&mut self, dy: i32, inventory_len: usize) {
        if self.inspect_item.is_some() { return; }
        match self.panel {
            MenuPanel::Inventory => {
                if inventory_len > 0 {
                    self.inv_index = (self.inv_index as i32 + dy).rem_euclid(inventory_len as i32) as usize;
                }
            }
            MenuPanel::Equipment => {
                let slots = EquipSlot::all().len();
                self.equip_index = (self.equip_index as i32 + dy).rem_euclid(slots as i32) as usize;
            }
        }
    }

    pub fn switch_panel(&mut self) {
        if self.inspect_item.is_some() { return; }
        self.panel = match self.panel {
            MenuPanel::Inventory => MenuPanel::Equipment,
            MenuPanel::Equipment => MenuPanel::Inventory,
        };
    }

    pub fn inspect(&mut self, inventory: &[String], equipment: &Equipment) {
        if self.inspect_item.is_some() {
            self.inspect_item = None;
            return;
        }
        let item_id = match self.panel {
            MenuPanel::Inventory => inventory.get(self.inv_index).cloned(),
            MenuPanel::Equipment => {
                let slot = EquipSlot::all()[self.equip_index];
                equipment.get(slot).cloned()
            }
        };
        self.inspect_item = item_id;
    }

    /// Get selected inventory index (for equip action)
    pub fn selected_inv_index(&self) -> Option<usize> {
        if self.panel == MenuPanel::Inventory { Some(self.inv_index) } else { None }
    }

    /// Get selected equipment slot (for unequip action)
    pub fn selected_equip_slot(&self) -> Option<EquipSlot> {
        if self.panel == MenuPanel::Equipment { Some(EquipSlot::all()[self.equip_index]) } else { None }
    }
}

/// Render the inventory menu
pub fn render_inventory_menu(
    frame: &mut Frame,
    menu: &InventoryMenu,
    inventory: &[String],
    equipment: &Equipment,
) {
    let area = frame.area();
    
    // Main layout: left (inventory) | right (equipment)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Inventory panel
    let inv_block = Block::default()
        .title(if menu.panel == MenuPanel::Inventory { " Inventory [*] " } else { " Inventory " })
        .borders(Borders::ALL)
        .border_style(if menu.panel == MenuPanel::Inventory { Style::default().fg(Color::Yellow) } else { Style::default() });
    
    let inv_items: Vec<ListItem> = inventory.iter().enumerate().map(|(i, id)| {
        let name = get_item_def(id).map(|d| d.name.as_str()).unwrap_or(id);
        let style = if menu.panel == MenuPanel::Inventory && i == menu.inv_index {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default()
        };
        ListItem::new(format!(" {} ", name)).style(style)
    }).collect();
    
    let inv_list = if inv_items.is_empty() {
        List::new(vec![ListItem::new(" (empty)").style(Style::default().fg(Color::DarkGray))])
    } else {
        List::new(inv_items)
    }.block(inv_block);
    
    frame.render_widget(inv_list, chunks[0]);

    // Equipment panel
    let equip_block = Block::default()
        .title(if menu.panel == MenuPanel::Equipment { " Equipment [*] " } else { " Equipment " })
        .borders(Borders::ALL)
        .border_style(if menu.panel == MenuPanel::Equipment { Style::default().fg(Color::Yellow) } else { Style::default() });
    
    let equip_items: Vec<ListItem> = EquipSlot::all().iter().enumerate().map(|(i, &slot)| {
        let item_name = equipment.get(slot)
            .and_then(|id| get_item_def(id))
            .map(|d| d.name.as_str())
            .unwrap_or("---");
        let style = if menu.panel == MenuPanel::Equipment && i == menu.equip_index {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        ListItem::new(format!(" {}: {} ", slot.display_name(), item_name)).style(style)
    }).collect();
    
    let equip_list = List::new(equip_items).block(equip_block);
    frame.render_widget(equip_list, chunks[1]);

    // Inspect overlay
    if let Some(ref item_id) = menu.inspect_item {
        render_inspect_overlay(frame, item_id, area);
    }

    // Help bar at bottom
    let help = " [h/l] Switch panel | [j/k] Navigate | [x] Inspect | [u] Use | [Enter] Equip/Unequip | [Esc/i] Close ";
    let help_area = Rect::new(area.x, area.y + area.height.saturating_sub(1), area.width, 1);
    frame.render_widget(Paragraph::new(help).style(Style::default().fg(Color::DarkGray)), help_area);
}

fn render_inspect_overlay(frame: &mut Frame, item_id: &str, area: Rect) {
    use ratatui::widgets::Wrap;
    
    let info = inspect_item(item_id);
    let width = 50u16.min(area.width.saturating_sub(4));
    
    let mut lines: Vec<Line> = Vec::new();
    let mut desc_len = 0usize;
    
    if let Some(ref info) = info {
        lines.push(Line::from(Span::styled(info.name.clone(), Style::default().fg(Color::Yellow).bold())));
        lines.push(Line::from(""));
        desc_len = info.description.len();
        lines.push(Line::from(Span::styled(info.description.clone(), Style::default().fg(Color::White))));
        lines.push(Line::from(""));
        for (stat, value) in &info.stats {
            lines.push(Line::from(format!("  {}: {}", stat, value)));
        }
    } else {
        lines.push(Line::from(Span::styled("Unknown item", Style::default().fg(Color::Red))));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[x] Close", Style::default().fg(Color::DarkGray))));

    // Estimate height based on description length + wrapping
    let inner_width = width.saturating_sub(2) as usize;
    let wrapped_lines = if inner_width > 0 { (desc_len / inner_width) + 1 } else { 1 };
    let height = ((lines.len() + wrapped_lines + 2) as u16).min(area.height.saturating_sub(4));
    
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect::new(x, y, width, height);

    let block = Block::default().title(" Inspect ").borders(Borders::ALL).style(Style::default().bg(Color::Black));
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }).block(block), popup_area);
}
