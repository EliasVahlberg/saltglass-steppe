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
    game_state: Option<GameStateData>,
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

impl SatelliteApp {
    pub fn new(socket_path: &str) -> Result<Self> {
        let client = IpcClient::connect(socket_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::ConnectionRefused, 
                format!("Failed to connect to game: {}", e)))?;
        
        Ok(Self {
            client,
            messages: Vec::new(),
            game_state: None,
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
                    .title("Game Log")
                    .borders(Borders::ALL);
                
                let items: Vec<ListItem> = self.messages
                    .iter()
                    .rev()
                    .take(area.height as usize - 2)
                    .map(|msg| ListItem::new(msg.as_str()))
                    .collect();
                
                let list = List::new(items).block(block);
                f.render_widget(list, area);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
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
                    .title("Player Status")
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
                    "Waiting for game data...".to_string()
                };
                
                let paragraph = Paragraph::new(content)
                    .block(block)
                    .wrap(Wrap { trim: true });
                
                f.render_widget(paragraph, area);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
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
                let area = f.area();
                let block = Block::default()
                    .title("Inventory")
                    .borders(Borders::ALL);
                
                let content = "Inventory data not yet implemented";
                let paragraph = Paragraph::new(content)
                    .block(block)
                    .wrap(Wrap { trim: true });
                
                f.render_widget(paragraph, area);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
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
                    "Waiting for game data...".to_string()
                };
                
                let debug_paragraph = Paragraph::new(debug_content)
                    .block(Block::default().title("Debug Info").borders(Borders::ALL))
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
                    .block(Block::default().title("Debug Command").borders(Borders::ALL));
                f.render_widget(input_paragraph, chunks[2]);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
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
                    IpcMessage::GameState { hp, max_hp, refraction, turn, storm_countdown, adaptations } => {
                        self.game_state = Some(GameStateData {
                            hp, max_hp, refraction, turn, storm_countdown, adaptations
                        });
                    }
                    IpcMessage::LogEntry { message, .. } => {
                        self.messages.push(message);
                        if self.messages.len() > 1000 {
                            self.messages.remove(0);
                        }
                    }
                    IpcMessage::DebugInfo { player_pos, enemies_count, items_count, storm_intensity, seed, god_view, phase_mode } => {
                        self.messages.push(format!("Debug: Pos({},{}) Enemies:{} Items:{} Storm:{} Seed:{} God:{} Phase:{}", 
                            player_pos.0, player_pos.1, enemies_count, items_count, storm_intensity, seed, god_view, phase_mode));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
