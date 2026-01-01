use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
};
use crate::GameState;

#[derive(Default)]
pub struct DebugMenu {
    pub active: bool,
    pub tab: DebugTab,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum DebugTab {
    #[default]
    Info,
    Performance,
    States,
    Commands,
}

impl DebugMenu {
    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            DebugTab::Info => DebugTab::Performance,
            DebugTab::Performance => DebugTab::States,
            DebugTab::States => DebugTab::Commands,
            DebugTab::Commands => DebugTab::Info,
        };
    }

    pub fn prev_tab(&mut self) {
        self.tab = match self.tab {
            DebugTab::Info => DebugTab::Commands,
            DebugTab::Performance => DebugTab::Info,
            DebugTab::States => DebugTab::Performance,
            DebugTab::Commands => DebugTab::States,
        };
    }
}

pub fn render_debug_menu(f: &mut Frame, menu: &DebugMenu, state: &GameState) {
    if !menu.active {
        return;
    }

    let area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title("Debug Menu")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Tab headers
    let tab_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let tab_titles = ["Info", "Performance", "States", "Commands"];
    let tab_spans: Vec<Span> = tab_titles
        .iter()
        .enumerate()
        .map(|(i, &title)| {
            let style = if i == menu.tab as usize {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Span::styled(format!(" {} ", title), style)
        })
        .collect();

    let tabs_paragraph = Paragraph::new(Line::from(tab_spans))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(tabs_paragraph, tab_layout[0]);

    // Tab content
    let content_area = tab_layout[1];
    let current_tab = menu.tab;
    match current_tab {
        DebugTab::Info => render_debug_info(f, content_area, state),
        DebugTab::Performance => render_performance_info(f, content_area, state),
        DebugTab::States => render_debug_states(f, content_area),
        DebugTab::Commands => render_debug_commands(f, content_area),
    }

    // Controls
    let controls = Paragraph::new("[Tab] Switch | [F12] Close | [Enter] Execute")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    let controls_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(controls, controls_area);
}

fn render_debug_info(f: &mut Frame, area: Rect, state: &GameState) {
    let info = state.get_debug_info();
    
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    // Basic info
    let basic_info = format!(
        "Turn: {}\nPlayer Position: ({}, {})\nPlayer HP: {}/{}\nWorld Seed: {}\nEnemies: {}\nItems: {}",
        info.turn,
        info.player_pos.0, info.player_pos.1,
        info.player_hp.0, info.player_hp.1,
        info.seed,
        info.enemies_count,
        info.items_count
    );
    
    let basic_paragraph = Paragraph::new(basic_info)
        .block(Block::default().title("Game State").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(basic_paragraph, layout[0]);

    // Storm info
    let storm_info = format!(
        "Storm Intensity: {}\nTurns Until Storm: {}",
        info.storm_intensity,
        info.storm_turns
    );
    
    let storm_paragraph = Paragraph::new(storm_info)
        .block(Block::default().title("Storm").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(storm_paragraph, layout[1]);

    // Memory usage
    let memory_paragraph = Paragraph::new(format!("Memory Usage: {}", info.memory_usage))
        .block(Block::default().title("System").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(memory_paragraph, layout[2]);
}

fn render_performance_info(f: &mut Frame, area: Rect, state: &GameState) {
    let info = state.get_debug_info();
    
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    // FPS/Performance metrics (placeholder)
    let fps_gauge = Gauge::default()
        .block(Block::default().title("FPS (Simulated)").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(85); // Placeholder
    f.render_widget(fps_gauge, layout[0]);

    let memory_gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(45); // Placeholder
    f.render_widget(memory_gauge, layout[1]);

    // Performance metrics
    let mut metrics_text = "Performance Metrics:\n".to_string();
    for (key, value) in &info.performance_metrics {
        metrics_text.push_str(&format!("{}: {:.2}ms\n", key, value));
    }
    
    let metrics_paragraph = Paragraph::new(metrics_text)
        .block(Block::default().title("Timing").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(metrics_paragraph, layout[2]);
}

fn render_debug_states(f: &mut Frame, area: Rect) {
    let states = match crate::GameState::list_debug_states() {
        Ok(states) => states,
        Err(_) => vec!["Error loading debug states".to_string()],
    };

    let items: Vec<ListItem> = states
        .iter()
        .map(|state| ListItem::new(state.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Saved Debug States").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(list, area);
}

fn render_debug_commands(f: &mut Frame, area: Rect) {
    let commands = vec![
        "show tile - Enable god view",
        "hide tile - Disable god view", 
        "sturdy - Set HP to 9999",
        "phase - Toggle wall phasing",
        "save_debug [name] - Save debug state",
        "load_debug <name> - Load debug state",
        "list_debug - List saved states",
        "debug_info - Show debug info",
        "report_issue - Open issue reporter",
        "help - Show all commands",
    ];

    let items: Vec<ListItem> = commands
        .iter()
        .map(|cmd| ListItem::new(*cmd))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Available Commands").borders(Borders::ALL));

    f.render_widget(list, area);
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
