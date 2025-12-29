use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
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
        equipped: Vec<String>,
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
    clients: Arc<Mutex<Vec<UnixStream>>>,
    receiver: mpsc::Receiver<IpcMessage>,
}

impl IpcServer {
    pub fn new(socket_path: &str) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        
        // Remove existing socket if it exists
        if Path::new(socket_path).exists() {
            std::fs::remove_file(socket_path)?;
        }
        
        let clients = Arc::new(Mutex::new(Vec::new()));
        let clients_clone = clients.clone();
        let socket_path_clone = socket_path.to_string();
        
        // Start server thread
        thread::spawn(move || {
            if let Ok(listener) = UnixListener::bind(&socket_path_clone) {
                for stream in listener.incoming() {
                    if let Ok(stream) = stream {
                        if let Ok(mut clients) = clients_clone.lock() {
                            clients.push(stream);
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            socket_path: socket_path.to_string(),
            clients,
            receiver,
        })
    }
    
    pub fn start(&self) -> std::io::Result<()> {
        // Server is already started in new()
        Ok(())
    }
    
    pub fn send_message(&self, message: IpcMessage) {
        if let Ok(mut clients) = self.clients.lock() {
            let json = serde_json::to_string(&message).unwrap_or_default();
            let data = format!("{}\n", json);
            
            // Send to all clients, remove disconnected ones
            clients.retain_mut(|client| {
                client.write_all(data.as_bytes()).is_ok() && client.flush().is_ok()
            });
        }
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
        use std::io::Read;
        let mut buffer = [0; 4096];
        let mut messages = Vec::new();
        
        // Non-blocking read
        self.stream.set_nonblocking(true)?;
        match self.stream.read(&mut buffer) {
            Ok(0) => {}, // No data available
            Ok(n) => {
                let data = String::from_utf8_lossy(&buffer[..n]);
                for line in data.lines() {
                    if let Ok(message) = serde_json::from_str::<IpcMessage>(line.trim()) {
                        messages.push(message);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}, // No data available
            Err(e) => return Err(e),
        }
        self.stream.set_nonblocking(false)?;
        
        Ok(messages)
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
