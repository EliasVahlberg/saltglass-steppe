use std::process::Command;
use std::env;

pub fn spawn_terminal_window(ui_type: &str) -> std::io::Result<()> {
    let current_exe = env::current_exe()?;
    let exe_path = current_exe.to_string_lossy();
    let flag = format!("--{}", ui_type);
    
    // Try different terminal emulators in order of preference
    let terminals = [
        ("gnome-terminal", vec!["--".to_string(), exe_path.to_string(), flag.clone()]),
        ("konsole", vec!["-e".to_string(), exe_path.to_string(), flag.clone()]),
        ("xterm", vec!["-e".to_string(), exe_path.to_string(), flag.clone()]),
        ("alacritty", vec!["-e".to_string(), exe_path.to_string(), flag.clone()]),
        ("kitty", vec!["--".to_string(), exe_path.to_string(), flag.clone()]),
    ];
    
    for (terminal, args) in &terminals {
        if which::which(terminal).is_ok() {
            return Command::new(terminal)
                .args(args)
                .spawn()
                .map(|_| ());
        }
    }
    
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No supported terminal emulator found"
    ))
}

pub fn get_available_terminals() -> Vec<String> {
    let terminals = ["gnome-terminal", "konsole", "xterm", "alacritty", "kitty"];
    terminals
        .iter()
        .filter(|&term| which::which(term).is_ok())
        .map(|&term| term.to_string())
        .collect()
}
