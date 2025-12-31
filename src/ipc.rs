#[cfg(unix)]
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::sync::{mpsc, Arc, Mutex};
#[cfg(unix)]
use std::thread;

#[cfg(unix)]
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

#[cfg(unix)]
pub struct IpcServer {
    socket_path: String,
    clients: Arc<Mutex<Vec<UnixStream>>>,
    receiver: mpsc::Receiver<IpcMessage>,
}

#[cfg(unix)]
impl IpcServer {
    pub fn new(socket_path: &str) -> std::io::Result<Self> {
        let (_sender, receiver) = mpsc::channel();
        
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
    
    pub fn send_message(&self, message: IpcMessage) -> std::io::Result<()> {
        if let Ok(mut clients) = self.clients.lock() {
            let json = serde_json::to_string(&message).unwrap_or_default();
            let data = format!("{}\n", json);
            
            // Send to all clients, remove disconnected ones
            clients.retain_mut(|client| {
                client.write_all(data.as_bytes()).is_ok() && client.flush().is_ok()
            });
        }
        Ok(())
    }
    
    pub fn try_recv_message(&self) -> Option<IpcMessage> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(unix)]
#[allow(dead_code)]
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

#[cfg(unix)]
pub struct IpcClient {
    stream: UnixStream,
}

#[cfg(unix)]
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

#[cfg(unix)]
impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

// Windows stubs - IPC not supported on Windows
#[cfg(not(unix))]
pub struct IpcServer;

#[cfg(not(unix))]
impl IpcServer {
    pub fn new(_socket_path: &str) -> std::io::Result<Self> {
        Ok(IpcServer)
    }
    
    pub fn start(&self) -> std::io::Result<()> {
        Ok(())
    }
    
    pub fn send_message(&self, _message: IpcMessage) -> std::io::Result<()> {
        Ok(())
    }
    
    pub fn broadcast_message(&self, _message: &IpcMessage) {}
    
    pub fn try_recv_message(&self) -> Option<IpcMessage> {
        None
    }
}

#[cfg(not(unix))]
pub struct IpcClient;

#[cfg(not(unix))]
impl IpcClient {
    pub fn connect(_socket_path: &str) -> std::io::Result<Self> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "IPC not supported on Windows"))
    }
    
    pub fn send_message(&mut self, _message: &IpcMessage) -> std::io::Result<()> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "IPC not supported on Windows"))
    }
    
    pub fn read_messages(&mut self) -> std::io::Result<Vec<IpcMessage>> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "IPC not supported on Windows"))
    }
}

#[cfg(not(unix))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IpcMessage {
    GameState {
        hp: i32,
        max_hp: i32,
        refraction: i32,
        turn: u32,
        storm_countdown: i32,
        adaptations: Vec<String>,
        god_view: bool,
        phase_mode: bool,
    },
    LogEntry {
        message: String,
        msg_type: String,
        turn: u32,
    },
    InventoryUpdate {
        items: Vec<String>,
        equipped: Vec<String>,
    },
    DebugInfo {
        player_pos: (i32, i32),
        enemies_count: usize,
        items_count: usize,
        storm_intensity: i32,
        seed: u64,
        god_view: bool,
        phase_mode: bool,
    },
    Command {
        action: String,
    },
}
