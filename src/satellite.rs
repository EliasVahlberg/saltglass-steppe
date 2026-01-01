use crate::ipc::{IpcClient, IpcMessage};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::io::{stdout, Result};
use std::time::{Duration, Instant};

pub struct SatelliteApp {
    client: IpcClient,
    messages: Vec<String>,
    game_messages: Vec<String>,
    game_state: Option<GameStateData>,
    inventory_data: Option<InventoryData>,
    log_scroll: usize,
    game_log_scroll: usize,
    #[allow(dead_code)]
    last_update: Instant,
}

#[derive(Debug, Clone)]
struct GameStateData {
    hp: i32,
    max_hp: i32,
    refraction: i32,
    turn: u32,
    storm_countdown: u32,
    adaptations: Vec<String>,
}

#[derive(Debug, Clone)]
struct InventoryData {
    items: Vec<String>,
    equipped: Vec<String>,
}

impl SatelliteApp {
    pub fn new(socket_path: &str) -> Result<Self> {
        let client = IpcClient::connect(socket_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::ConnectionRefused, 
                format!("Failed to connect to game: {}", e)))?;
        
        Ok(Self {
            client,
            messages: Vec::new(),
            game_messages: Vec::new(),
            game_state: None,
            inventory_data: None,
            log_scroll: 0,
            game_log_scroll: 0,
            last_update: Instant::now(),
        })
    }
    
    pub fn run_log_ui(&mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        
        loop {
            self.update_from_game()?;
            
            terminal.draw(|f| {
                let area = f.area();
                let block = Block::default()
                    .title("All Messages (Press 'q'/ESC to quit, ↑↓ to scroll)")
                    .borders(Borders::ALL);
                
                let available_height = area.height as usize - 2;
                let total_messages = self.messages.len();
                
                let items: Vec<ListItem> = if total_messages > 0 {
                    let start_idx = if self.log_scroll >= total_messages {
                        0
                    } else {
                        total_messages.saturating_sub(self.log_scroll + available_height)
                    };
                    let end_idx = total_messages.saturating_sub(self.log_scroll);
                    
                    self.messages[start_idx..end_idx]
                        .iter()
                        .map(|msg| ListItem::new(msg.as_str()))
                        .collect()
                } else {
                    vec![ListItem::new("No messages yet...")]
                };
                
                let list = List::new(items).block(block);
                f.render_widget(list, area);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Up => {
                                if self.log_scroll < self.messages.len().saturating_sub(1) {
                                    self.log_scroll += 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.log_scroll > 0 {
                                    self.log_scroll -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    
    pub fn run_game_log_ui(&mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        
        loop {
            self.update_from_game()?;
            
            terminal.draw(|f| {
                let area = f.area();
                let block = Block::default()
                    .title("Game Log (Press 'q'/ESC to quit, ↑↓ to scroll)")
                    .borders(Borders::ALL);
                
                let available_height = area.height as usize - 2;
                let total_messages = self.game_messages.len();
                
                let items: Vec<ListItem> = if total_messages > 0 {
                    let start_idx = if self.game_log_scroll >= total_messages {
                        0
                    } else {
                        total_messages.saturating_sub(self.game_log_scroll + available_height)
                    };
                    let end_idx = total_messages.saturating_sub(self.game_log_scroll);
                    
                    self.game_messages[start_idx..end_idx]
                        .iter()
                        .map(|msg| ListItem::new(msg.as_str()))
                        .collect()
                } else {
                    vec![ListItem::new("No game messages yet...")]
                };
                
                let list = List::new(items).block(block);
                f.render_widget(list, area);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Up => {
                                if self.game_log_scroll < self.game_messages.len().saturating_sub(1) {
                                    self.game_log_scroll += 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.game_log_scroll > 0 {
                                    self.game_log_scroll -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    
    pub fn run_status_ui(&mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        
        loop {
            self.update_from_game()?;
            
            terminal.draw(|f| {
                let area = f.area();
                let block = Block::default()
                    .title("Player Status (Press 'q' or ESC to quit)")
                    .borders(Borders::ALL);
                
                let content = if let Some(ref state) = self.game_state {
                    format!(
                        "HP: {}/{}\nRefraction: {}\nTurn: {}\nStorm: {} turns\nAdaptations: {}",
                        state.hp,
                        state.max_hp,
                        state.refraction,
                        state.turn,
                        state.storm_countdown,
                        state.adaptations.join(", ")
                    )
                } else {
                    "Waiting for game data...\n\nMake sure the main game is running first.\nSocket: /tmp/saltglass-steppe.sock".to_string()
                };
                
                let paragraph = Paragraph::new(content)
                    .block(block)
                    .wrap(Wrap { trim: true });
                
                f.render_widget(paragraph, area);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    
    pub fn run_inventory_ui(&mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        
        loop {
            self.update_from_game()?;
            
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ])
                    .split(f.area());
                
                let inventory_content = if let Some(ref inv) = self.inventory_data {
                    if inv.items.is_empty() {
                        "No items in inventory".to_string()
                    } else {
                        inv.items.join("\n")
                    }
                } else {
                    "Waiting for inventory data...\n\nMake sure the main game is running first.\nSocket: /tmp/saltglass-steppe.sock".to_string()
                };
                
                let inventory_paragraph = Paragraph::new(inventory_content)
                    .block(Block::default().title("Inventory (Press 'q' or ESC to quit)").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(inventory_paragraph, chunks[0]);
                
                let equipped_content = if let Some(ref inv) = self.inventory_data {
                    if inv.equipped.is_empty() {
                        "No items equipped".to_string()
                    } else {
                        inv.equipped.join("\n")
                    }
                } else {
                    "Waiting for equipment data...".to_string()
                };
                
                let equipped_paragraph = Paragraph::new(equipped_content)
                    .block(Block::default().title("Equipped").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(equipped_paragraph, chunks[1]);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    
    pub fn run_debug_ui(&mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let mut input = String::new();
        
        loop {
            self.update_from_game()?;
            
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(8),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ])
                    .split(f.area());
                
                let debug_content = if let Some(ref state) = self.game_state {
                    format!(
                        "HP: {}/{} | Refraction: {} | Turn: {}\nStorm: {} turns | Adaptations: {}",
                        state.hp, state.max_hp, state.refraction, state.turn,
                        state.storm_countdown, state.adaptations.join(", ")
                    )
                } else {
                    "Waiting for game data...\n\nMake sure the main game is running first.\nSocket: /tmp/saltglass-steppe.sock".to_string()
                };
                
                let debug_paragraph = Paragraph::new(debug_content)
                    .block(Block::default().title("Debug Info (Ctrl+Q/ESC to quit)").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(debug_paragraph, chunks[0]);
                
                let history_items: Vec<ListItem> = self.messages
                    .iter()
                    .rev()
                    .take(chunks[1].height as usize - 2)
                    .map(|msg| ListItem::new(msg.as_str()))
                    .collect();
                let history_list = List::new(history_items)
                    .block(Block::default().title("Command History").borders(Borders::ALL));
                f.render_widget(history_list, chunks[1]);
                
                let input_paragraph = Paragraph::new(input.as_str())
                    .block(Block::default().title("Debug Command (Enter to send)").borders(Borders::ALL));
                f.render_widget(input_paragraph, chunks[2]);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                            KeyCode::Esc => break,
                            KeyCode::Enter => {
                                if !input.is_empty() {
                                    let _ = self.client.send_message(&IpcMessage::Command {
                                        action: input.clone(),
                                    });
                                    self.messages.push(format!("> {}", input));
                                    input.clear();
                                }
                            }
                            KeyCode::Backspace => { input.pop(); }
                            KeyCode::Char(c) => input.push(c),
                            _ => {}
                        }
                    }
                }
            }
        }
        
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
    
    fn update_from_game(&mut self) -> Result<()> {
        if let Ok(messages) = self.client.read_messages() {
            for message in messages {
                match message {
                    IpcMessage::GameState { hp, max_hp, refraction, turn, storm_countdown, adaptations, .. } => {
                        self.game_state = Some(GameStateData {
                            hp, max_hp, refraction, turn, storm_countdown: storm_countdown as u32, adaptations
                        });
                    }
                    IpcMessage::LogEntry { message, .. } => {
                        if message.starts_with("Debug:") || message.starts_with(">") {
                            self.messages.push(message);
                        } else {
                            self.game_messages.push(message.clone());
                            self.messages.push(message);
                        }
                        
                        if self.messages.len() > 1000 {
                            self.messages.remove(0);
                        }
                        if self.game_messages.len() > 1000 {
                            self.game_messages.remove(0);
                        }
                    }
                    IpcMessage::InventoryUpdate { items, equipped } => {
                        self.inventory_data = Some(InventoryData { items, equipped });
                    }
                    IpcMessage::DebugInfo { player_pos, enemies_count, items_count, storm_intensity, seed, tile_seed, world_pos, god_view, phase_mode } => {
                        self.messages.push(format!("Debug: Pos({},{}) World({},{}) Enemies:{} Items:{} Storm:{} Seed:{} TileSeed:{} God:{} Phase:{}", 
                            player_pos.0, player_pos.1, world_pos.0, world_pos.1, enemies_count, items_count, storm_intensity, seed, tile_seed, god_view, phase_mode));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
