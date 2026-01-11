//! Skills and Abilities Menu Interface

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::game::{
    skills::{
        SkillCategory, calculate_skill_cost, get_abilities_by_category, get_skill_def,
        get_skills_by_category,
    },
    state::GameState,
};

#[derive(Clone, Debug)]
pub enum SkillsMenuMode {
    Skills,
    Abilities,
}

#[derive(Clone, Debug)]
pub struct SkillsMenu {
    pub active: bool,
    pub mode: SkillsMenuMode,
    pub selected_category: SkillCategory,
    pub selected_index: usize,
    pub list_state: ListState,
}

impl Default for SkillsMenu {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            active: false,
            mode: SkillsMenuMode::Skills,
            selected_category: SkillCategory::Combat,
            selected_index: 0,
            list_state,
        }
    }
}

impl SkillsMenu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the skills menu
    pub fn open(&mut self) {
        self.active = true;
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    /// Close the skills menu
    pub fn close(&mut self) {
        self.active = false;
    }

    /// Navigate up in the current list
    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Navigate down in the current list
    pub fn navigate_down(&mut self, max_items: usize) {
        if self.selected_index < max_items.saturating_sub(1) {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Switch between skills and abilities view
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            SkillsMenuMode::Skills => SkillsMenuMode::Abilities,
            SkillsMenuMode::Abilities => SkillsMenuMode::Skills,
        };
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    /// Switch to next category
    pub fn next_category(&mut self) {
        self.selected_category = match self.selected_category {
            SkillCategory::Combat => SkillCategory::Athletics,
            SkillCategory::Athletics => SkillCategory::Survival,
            SkillCategory::Survival => SkillCategory::Crafting,
            SkillCategory::Crafting => SkillCategory::Combat,
        };
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    /// Switch to previous category
    pub fn prev_category(&mut self) {
        self.selected_category = match self.selected_category {
            SkillCategory::Combat => SkillCategory::Crafting,
            SkillCategory::Athletics => SkillCategory::Combat,
            SkillCategory::Survival => SkillCategory::Athletics,
            SkillCategory::Crafting => SkillCategory::Survival,
        };
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    /// Get currently selected skill ID
    pub fn get_selected_skill(&self) -> Option<String> {
        if !matches!(self.mode, SkillsMenuMode::Skills) {
            return None;
        }

        let skills = get_skills_by_category(&self.selected_category);
        skills.get(self.selected_index).map(|def| def.id.clone())
    }

    /// Get currently selected ability ID
    pub fn get_selected_ability(&self) -> Option<String> {
        if !matches!(self.mode, SkillsMenuMode::Abilities) {
            return None;
        }

        let abilities = get_abilities_by_category(&self.selected_category);
        abilities.get(self.selected_index).map(|def| def.id.clone())
    }

    /// Upgrade selected skill
    pub fn upgrade_skill(&self, game_state: &mut GameState) -> Result<(), String> {
        let skill_id = self.get_selected_skill().ok_or("No skill selected")?;

        game_state.skills.upgrade_skill(&skill_id)
    }

    /// Use selected ability
    pub fn use_ability(&self, game_state: &mut GameState) -> Result<(), String> {
        let ability_id = self.get_selected_ability().ok_or("No ability selected")?;

        match game_state.skills.use_ability(&ability_id) {
            Ok(effect_id) => {
                // Apply the effect
                apply_ability_effect(game_state, &effect_id);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

/// Apply ability effect to game state
fn apply_ability_effect(game_state: &mut GameState, effect_id: &str) {
    match effect_id {
        "power_strike" => {
            game_state.apply_status_effect("power_strike", 1);
            game_state.log("You prepare a devastating strike!");
        }
        "whirlwind_attack" => {
            // Attack all adjacent enemies
            let mut hit_count = 0;
            let adjacent_positions = [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ];

            for (dx, dy) in adjacent_positions {
                let x = game_state.player_x + dx;
                let y = game_state.player_y + dy;
                if game_state.enemy_at(x, y).is_some() {
                    if game_state.attack_melee(x, y) {
                        hit_count += 1;
                    }
                }
            }

            if hit_count > 0 {
                game_state.log(format!("Whirlwind hits {} enemies!", hit_count));
            } else {
                game_state.log("Whirlwind hits nothing.");
            }
        }
        "precise_shot" => {
            game_state.apply_status_effect("precise_shot", 1);
            game_state.log("You take careful aim...");
        }
        "defensive_stance" => {
            game_state.apply_status_effect("defensive_stance", 5);
            game_state.log("You adopt a defensive stance.");
        }
        "sprint" => {
            game_state.apply_status_effect("sprint", 1);
            game_state.log("You prepare to sprint!");
        }
        "dodge_roll" => {
            game_state.apply_status_effect("dodge_roll", 2);
            game_state.log("You ready yourself to dodge!");
        }
        "field_medicine" => {
            let heal = (game_state.player_max_hp / 4).max(5);
            game_state.player_hp = (game_state.player_hp + heal).min(game_state.player_max_hp);
            game_state.log(format!("You quickly treat your wounds. (+{} HP)", heal));
        }
        "antidote" => {
            game_state.status_effects.retain(|e| e.id != "poison");
            game_state.log("You cure harmful effects.");
        }
        "vanish" => {
            game_state.apply_status_effect("invisible", 3);
            game_state.log("You fade from sight...");
        }
        _ => {
            game_state.log("Effect not implemented.");
        }
    }
}

/// Render the skills menu
pub fn render_skills_menu(f: &mut Frame, game_state: &GameState, menu: &SkillsMenu) {
    let size = f.area();

    // Center the menu
    let popup_area = centered_rect(80, 80, size);

    // Clear the area
    f.render_widget(Clear, popup_area);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(popup_area);

    // Header
    let mode_text = match menu.mode {
        SkillsMenuMode::Skills => "Skills",
        SkillsMenuMode::Abilities => "Abilities",
    };
    let category_text = format!("{:?}", menu.selected_category);

    let header = Paragraph::new(format!("{} - {}", mode_text, category_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Character Development"),
        )
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(header, chunks[0]);

    // Content layout
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // List
            Constraint::Percentage(50), // Details
        ])
        .split(chunks[1]);

    // Render list and details based on mode
    match menu.mode {
        SkillsMenuMode::Skills => {
            render_skills_list(f, game_state, menu, content_chunks[0]);
            render_skill_details(f, game_state, menu, content_chunks[1]);
        }
        SkillsMenuMode::Abilities => {
            render_abilities_list(f, game_state, menu, content_chunks[0]);
            render_ability_details(f, game_state, menu, content_chunks[1]);
        }
    }

    // Footer with controls
    let footer_text = match menu.mode {
        SkillsMenuMode::Skills => {
            "↑↓: Navigate | ←→: Category | Tab: Abilities | Enter: Upgrade | Esc: Close"
        }
        SkillsMenuMode::Abilities => {
            "↑↓: Navigate | ←→: Category | Tab: Skills | Enter: Use | Esc: Close"
        }
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[2]);
}

fn render_skills_list(f: &mut Frame, game_state: &GameState, menu: &SkillsMenu, area: Rect) {
    let skills = get_skills_by_category(&menu.selected_category);

    let items: Vec<ListItem> = skills
        .iter()
        .map(|def| {
            let level = game_state.skills.get_skill_level(&def.id);
            let cost = calculate_skill_cost(&def.id, level);

            let color = if game_state.skills.skill_points >= cost && level < def.max_level {
                Color::Green
            } else if level >= def.max_level {
                Color::Yellow
            } else {
                Color::Gray
            };

            let text = format!(
                "{} (Lv.{}/{}) - {} SP",
                def.name, level, def.max_level, cost
            );
            ListItem::new(text).style(Style::default().fg(color))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Skills"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, area, &mut menu.list_state.clone());
}

fn render_abilities_list(f: &mut Frame, game_state: &GameState, menu: &SkillsMenu, area: Rect) {
    let abilities = get_abilities_by_category(&menu.selected_category);

    let items: Vec<ListItem> = abilities
        .iter()
        .map(|def| {
            let unlocked = game_state.skills.unlocked_abilities.contains(&def.id);
            let on_cooldown = game_state.skills.cooldowns.get(&def.id).unwrap_or(&0) > &0;
            let can_afford = game_state.skills.stamina >= def.stamina_cost;

            let color = if !unlocked {
                Color::DarkGray
            } else if on_cooldown {
                Color::Red
            } else if !can_afford {
                Color::Yellow
            } else {
                Color::Green
            };

            let status = if !unlocked {
                " [LOCKED]"
            } else if on_cooldown {
                " [COOLDOWN]"
            } else if !can_afford {
                " [NO STAMINA]"
            } else {
                ""
            };

            let text = format!("{} ({}){}", def.name, def.stamina_cost, status);
            ListItem::new(text).style(Style::default().fg(color))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Abilities"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, area, &mut menu.list_state.clone());
}

fn render_skill_details(f: &mut Frame, game_state: &GameState, menu: &SkillsMenu, area: Rect) {
    let skills = get_skills_by_category(&menu.selected_category);

    let content = if let Some(def) = skills.get(menu.selected_index) {
        let level = game_state.skills.get_skill_level(&def.id);
        let cost = calculate_skill_cost(&def.id, level);

        vec![
            Line::from(vec![Span::styled(
                &def.name,
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(""),
            Line::from(def.description.clone()),
            Line::from(""),
            Line::from(format!("Current Level: {}/{}", level, def.max_level)),
            Line::from(format!("Upgrade Cost: {} SP", cost)),
            Line::from(format!("Available SP: {}", game_state.skills.skill_points)),
        ]
    } else {
        vec![Line::from("No skill selected")]
    };

    let details = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });

    f.render_widget(details, area);
}

fn render_ability_details(f: &mut Frame, game_state: &GameState, menu: &SkillsMenu, area: Rect) {
    let abilities = get_abilities_by_category(&menu.selected_category);

    let content = if let Some(def) = abilities.get(menu.selected_index) {
        let unlocked = game_state.skills.unlocked_abilities.contains(&def.id);
        let cooldown = game_state.skills.cooldowns.get(&def.id).unwrap_or(&0);
        let skill_level = game_state.skills.get_skill_level(&def.required_skill);

        let mut lines = vec![
            Line::from(vec![Span::styled(
                &def.name,
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(""),
            Line::from(def.description.clone()),
            Line::from(""),
            Line::from(format!("Stamina Cost: {}", def.stamina_cost)),
            Line::from(format!("Cooldown: {} turns", def.cooldown)),
            Line::from(""),
        ];

        if let Some(skill_def) = get_skill_def(&def.required_skill) {
            lines.push(Line::from(format!(
                "Requires: {} Lv.{}",
                skill_def.name, def.required_level
            )));
            lines.push(Line::from(format!("Your Level: {}", skill_level)));
        }

        if !unlocked {
            lines.push(Line::from(vec![Span::styled(
                "LOCKED",
                Style::default().fg(Color::Red),
            )]));
        } else if *cooldown > 0 {
            lines.push(Line::from(vec![Span::styled(
                format!("Cooldown: {} turns", cooldown),
                Style::default().fg(Color::Red),
            )]));
        }

        lines
    } else {
        vec![Line::from("No ability selected")]
    };

    let details = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });

    f.render_widget(details, area);
}

/// Helper function to create a centered rectangle
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
