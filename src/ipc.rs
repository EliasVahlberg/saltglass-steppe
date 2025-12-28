use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    GameState {
        hp: i32,
        max_hp: i32,
        refraction: i32,
        turn: u32,
        storm_countdown: u32,
        adaptations: Vec<String>,
    },
    LogEntry {
        message: String,
        timestamp: u64,
    },
    InventoryUpdate {
        items: Vec<String>,
    },
    DebugInfo {
        player_pos: (i32, i32),
        enemies_count: usize,
        items_count: usize,
        storm_intensity: u8,
        seed: u64,
        god_view: bool,
        phase_mode: bool,
    },
    Command {
        action: String,
    },
}

pub struct IpcServer {
    socket_path: String,
    sender: mpsc::Sender<IpcMessage>,
    receiver: mpsc::Receiver<IpcMessage>,
}

impl IpcServer {
    pub fn new(socket_path: &str) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        
        // Remove existing socket if it exists
        if Path::new(socket_path).exists() {
            std::fs::remove_file(socket_path)?;
        }
        
        Ok(Self {
            socket_path: socket_path.to_string(),
            sender,
            receiver,
        })
    }
    
    pub fn start(&self) -> std::io::Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;
        let sender = self.sender.clone();
        
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let sender = sender.clone();
                        thread::spawn(move || {
                            handle_client(stream, sender);
                        });
                    }
                    Err(err) => eprintln!("Connection failed: {}", err),
                }
            }
        });
        
        Ok(())
    }
    
    pub fn send_message(&self, message: IpcMessage) {
        let _ = self.sender.send(message);
    }
    
    pub fn try_recv_message(&self) -> Option<IpcMessage> {
        self.receiver.try_recv().ok()
    }
}

fn handle_client(stream: UnixStream, sender: mpsc::Sender<IpcMessage>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    
    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        if let Ok(message) = serde_json::from_str::<IpcMessage>(&line.trim()) {
            let _ = sender.send(message);
        }
        line.clear();
    }
}

pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    pub fn connect(socket_path: &str) -> std::io::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        Ok(Self { stream })
    }
    
    pub fn send_message(&mut self, message: &IpcMessage) -> std::io::Result<()> {
        let json = serde_json::to_string(message)?;
        writeln!(self.stream, "{}", json)?;
        self.stream.flush()?;
        Ok(())
    }
    
    pub fn read_messages(&mut self) -> std::io::Result<Vec<IpcMessage>> {
        let mut reader = BufReader::new(&self.stream);
        let mut messages = Vec::new();
        let mut line = String::new();
        
        while reader.read_line(&mut line)? > 0 {
            if let Ok(message) = serde_json::from_str::<IpcMessage>(&line.trim()) {
                messages.push(message);
            }
            line.clear();
        }
        
        Ok(messages)
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
