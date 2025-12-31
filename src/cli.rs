use clap::{Arg, Command};

#[derive(Debug, Clone)]
pub enum LaunchMode {
    MainGame,
    LogUi,
    GameLogUi,
    StatusUi,
    InventoryUi,
    DebugUi,
}

pub fn parse_args() -> LaunchMode {
    let matches = Command::new("saltglass-steppe")
        .version("0.1.0")
        .about("Saltglass Steppe - A TUI RPG")
        .arg(
            Arg::new("log-ui")
                .long("log-ui")
                .help("Launch as log viewer terminal")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("status-ui")
                .long("status-ui")
                .help("Launch as status viewer terminal")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("inventory-ui")
                .long("inventory-ui")
                .help("Launch as inventory viewer terminal")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("game-log-ui")
                .long("game-log-ui")
                .help("Launch as game log viewer terminal (in-game messages only)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("debug-ui")
                .long("debug-ui")
                .help("Launch as debug console terminal")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("log-ui") {
        LaunchMode::LogUi
    } else if matches.get_flag("game-log-ui") {
        LaunchMode::GameLogUi
    } else if matches.get_flag("status-ui") {
        LaunchMode::StatusUi
    } else if matches.get_flag("inventory-ui") {
        LaunchMode::InventoryUi
    } else if matches.get_flag("debug-ui") {
        LaunchMode::DebugUi
    } else {
        LaunchMode::MainGame
    }
}
