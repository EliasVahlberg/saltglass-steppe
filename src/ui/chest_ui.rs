use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::game::{Chest, get_chest_def};

pub struct ChestUI {
    pub chest_index: usize,
    pub chest_list_state: ListState,
    pub inventory_list_state: ListState,
    pub selected_panel: ChestPanel,
}

#[derive(PartialEq)]
pub enum ChestPanel {
    ChestInventory,
    PlayerInventory,
}

impl ChestUI {
    pub fn new(chest_index: usize) -> Self {
        let mut chest_list_state = ListState::default();
        chest_list_state.select(Some(0));

        let mut inventory_list_state = ListState::default();
        inventory_list_state.select(Some(0));

        Self {
            chest_index,
            chest_list_state,
            inventory_list_state,
            selected_panel: ChestPanel::ChestInventory,
        }
    }

    pub fn move_selection(&mut self, direction: i32) {
        match self.selected_panel {
            ChestPanel::ChestInventory => {
                let current = self.chest_list_state.selected().unwrap_or(0);
                self.chest_list_state
                    .select(Some(current.saturating_add_signed(direction as isize)));
            }
            ChestPanel::PlayerInventory => {
                let current = self.inventory_list_state.selected().unwrap_or(0);
                self.inventory_list_state
                    .select(Some(current.saturating_add_signed(direction as isize)));
            }
        }
    }

    pub fn switch_panel(&mut self) {
        self.selected_panel = match self.selected_panel {
            ChestPanel::ChestInventory => ChestPanel::PlayerInventory,
            ChestPanel::PlayerInventory => ChestPanel::ChestInventory,
        };
    }

    pub fn get_selected_chest_item(&self) -> Option<usize> {
        if self.selected_panel == ChestPanel::ChestInventory {
            self.chest_list_state.selected()
        } else {
            None
        }
    }

    pub fn get_selected_inventory_item(&self) -> Option<usize> {
        if self.selected_panel == ChestPanel::PlayerInventory {
            self.inventory_list_state.selected()
        } else {
            None
        }
    }
}

pub fn render_chest_ui(
    frame: &mut Frame,
    area: Rect,
    chest: &Chest,
    player_inventory: &[String],
    ui_state: &mut ChestUI,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Instructions
        ])
        .split(area);

    // Title
    let chest_def = get_chest_def(&chest.id);
    let title = if let Some(def) = chest_def {
        format!("{} - {}", def.name, def.description)
    } else {
        "Chest".to_string()
    };

    let title_paragraph = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(title_paragraph, chunks[0]);

    // Main content - split into chest and inventory
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Chest inventory
    let chest_items: Vec<ListItem> = chest
        .inventory
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let item_def = crate::game::get_item_def(&item.id);
            let name = item_def.map(|def| def.name.as_str()).unwrap_or(&item.id);
            ListItem::new(format!("{}. {}", i + 1, name))
        })
        .collect();

    let chest_title = format!(
        "Chest ({}/{})",
        chest.inventory.len(),
        chest_def.map(|d| d.capacity).unwrap_or(10)
    );

    let chest_style = if ui_state.selected_panel == ChestPanel::ChestInventory {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let chest_list = List::new(chest_items)
        .block(Block::default().borders(Borders::ALL).title(chest_title))
        .style(chest_style)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    frame.render_stateful_widget(chest_list, main_chunks[0], &mut ui_state.chest_list_state);

    // Player inventory
    let inventory_items: Vec<ListItem> = player_inventory
        .iter()
        .enumerate()
        .map(|(i, item_id)| {
            let item_def = crate::game::get_item_def(item_id);
            let name = item_def.map(|def| def.name.as_str()).unwrap_or(item_id);
            ListItem::new(format!("{}. {}", i + 1, name))
        })
        .collect();

    let inventory_style = if ui_state.selected_panel == ChestPanel::PlayerInventory {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let inventory_list = List::new(inventory_items)
        .block(Block::default().borders(Borders::ALL).title("Inventory"))
        .style(inventory_style)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    frame.render_stateful_widget(
        inventory_list,
        main_chunks[1],
        &mut ui_state.inventory_list_state,
    );

    // Instructions
    let instructions = vec![Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" Switch Panel  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Transfer  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" Close"),
    ])];

    let instructions_paragraph =
        Paragraph::new(instructions).block(Block::default().borders(Borders::ALL));
    frame.render_widget(instructions_paragraph, chunks[2]);
}
