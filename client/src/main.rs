use clap::{Parser, Subcommand};
use std::process;

mod api;
mod config;
mod daemon;
mod processes;

#[derive(Parser)]
#[command(name = "drago")]
#[command(about = "Dynamic DNS client for hobbyists")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize device authorization
    Init,
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Show daemon status
    Status,
    /// Restart the daemon
    Restart,
    /// Internal command: runs the daemon (do not call directly)
    Daemon,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match config::store_device_token() {
            Ok(()) => println!("✅ Device authorization completed successfully"),
            Err(e) => {
                eprintln!("❌ Failed to complete device authorization: {}", e);
                process::exit(1);
            }
        },
        Commands::Start => match processes::start_daemon() {
            Ok(()) => println!("✅ Daemon started"),
            Err(e) => {
                eprintln!("❌ Failed to start daemon: {}", e);
                process::exit(1);
            }
        },
        Commands::Stop => match processes::stop_daemon() {
            Ok(()) => println!("✅ Daemon stopped"),
            Err(e) => {
                eprintln!("❌ Failed to stop daemon: {}", e);
                process::exit(1);
            }
        },
        Commands::Status => match processes::daemon_status() {
            Ok(status) => println!("{}", status),
            Err(e) => {
                eprintln!("❌ Failed to get daemon status: {}", e);
                process::exit(1);
            }
        },
        Commands::Restart => match processes::restart_daemon() {
            Ok(()) => println!("✅ Daemon restarted"),
            Err(e) => {
                eprintln!("❌ Failed to restart daemon: {}", e);
                process::exit(1);
            }
        },
        Commands::Daemon => {
            if let Err(e) = daemon::run() {
                eprintln!("❌ Daemon error: {}", e);
                process::exit(1);
            }
        }
    }
}
