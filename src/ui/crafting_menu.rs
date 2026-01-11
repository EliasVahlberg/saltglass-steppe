//! Crafting menu UI

use crate::game::{GameState, all_recipe_ids, can_craft, get_item_def, get_recipe};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

#[derive(Default)]
pub struct CraftingMenu {
    pub active: bool,
    pub selected: usize,
}

impl CraftingMenu {
    pub fn open(&mut self) {
        self.active = true;
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn navigate(&mut self, delta: i32, max: usize) {
        if max == 0 {
            return;
        }
        let new = self.selected as i32 + delta;
        self.selected = new.clamp(0, max.saturating_sub(1) as i32) as usize;
    }

    pub fn selected_recipe_id(&self) -> Option<&'static str> {
        all_recipe_ids().get(self.selected).copied()
    }
}

pub fn render_crafting_menu(frame: &mut Frame, menu: &CraftingMenu, state: &GameState) {
    let area = frame.area();
    let block = Block::default()
        .title(" Crafting (Esc to close, Enter to craft) ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    // Recipe list
    let recipe_ids = all_recipe_ids();
    let items: Vec<ListItem> = recipe_ids
        .iter()
        .map(|id| {
            let recipe = get_recipe(id).unwrap();
            let craftable = can_craft(recipe, &state.inventory);
            let style = if craftable {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(Line::from(Span::styled(&recipe.name, style)))
        })
        .collect();

    let list_block = Block::default().title(" Recipes ").borders(Borders::ALL);
    let mut list_state = ListState::default();
    list_state.select(Some(menu.selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(list_block)
            .highlight_style(Style::default().bg(Color::DarkGray)),
        chunks[0],
        &mut list_state,
    );

    // Recipe details
    let detail_block = Block::default().title(" Details ").borders(Borders::ALL);
    let detail_inner = detail_block.inner(chunks[1]);
    frame.render_widget(detail_block, chunks[1]);

    if let Some(recipe_id) = menu.selected_recipe_id() {
        if let Some(recipe) = get_recipe(recipe_id) {
            let craftable = can_craft(recipe, &state.inventory);

            let mut lines = vec![
                Line::from(Span::styled(
                    &recipe.name,
                    Style::default().fg(Color::Yellow).bold(),
                )),
                Line::from(""),
                Line::from(&recipe.description[..]),
                Line::from(""),
                Line::from(Span::styled("Materials:", Style::default().bold())),
            ];

            for (item_id, &required) in &recipe.materials {
                let have = state.inventory.iter().filter(|id| *id == item_id).count() as u32;
                let item_name = get_item_def(item_id)
                    .map(|d| d.name.as_str())
                    .unwrap_or(item_id);
                let color = if have >= required {
                    Color::Green
                } else {
                    Color::Red
                };
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{}/{}", have, required), Style::default().fg(color)),
                    Span::raw(format!(" {}", item_name)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Output:", Style::default().bold())));
            let output_name = get_item_def(&recipe.output)
                .map(|d| d.name.as_str())
                .unwrap_or(&recipe.output);
            lines.push(Line::from(format!(
                "  {}x {}",
                recipe.output_count, output_name
            )));

            lines.push(Line::from(""));
            if craftable {
                lines.push(Line::from(Span::styled(
                    "Press Enter to craft",
                    Style::default().fg(Color::Green),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "Missing materials",
                    Style::default().fg(Color::Red),
                )));
            }

            frame.render_widget(Paragraph::new(lines), detail_inner);
        }
    }
}
