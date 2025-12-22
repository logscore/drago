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
    /// Initialize device authorization (get JWT token)
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
    /// List available DNS zones
    Zones,
    /// List DNS records
    Records,
    /// Create a DNS record and get an API key for syncing
    Add {
        /// Zone ID to add the record to, found with drago zones
        #[arg(short, long)]
        zone: String,
        /// Subdomain name (e.g., "home" for home.example.com)
        #[arg(short, long)]
        name: String,
        /// TTL (We recommend excluding this and using the default 5 min)
        #[arg(short, long, default_value_t = 300)]
        ttl: i32,
    },
    /// Delete a DNS record
    Remove {
        /// Record ID to delete
        #[arg(short, long)]
        record: String,
        /// Zone ID the record belongs to
        #[arg(short, long)]
        zone: String,
    },
    /// List API keys
    Keys,
    /// Full setup: create record and configure for syncing
    Setup {
        /// Zone ID to add the record to
        #[arg(short, long)]
        zone: String,
        /// Subdomain name (e.g., "home" for home.example.com)
        #[arg(short, long)]
        name: String,
    },
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
        Commands::Zones => match api::list_zones() {
            Ok(zones) => {
                if zones.is_empty() {
                    println!("No DNS zones found. Make sure you have added a Cloudflare token.");
                } else {
                    println!("📋 Available DNS Zones:\n");
                    for (zone_id, zone_name) in zones {
                        println!("  {} - {}", zone_id, zone_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to list zones: {}", e);
                process::exit(1);
            }
        },
        Commands::Records => match api::list_records() {
            Ok(records) => {
                if records.is_empty() {
                    println!("No DNS records found.");
                } else {
                    println!("📋 DNS Records:\n");
                    for record in records {
                        println!(
                            "  {} ({}) -> {}",
                            record.name, record.record_type, record.content
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to list records: {}", e);
                process::exit(1);
            }
        },
        Commands::Add { zone, name, ttl } => match api::add_record(&zone, &name, ttl) {
            Ok(record_id) => {
                println!("✅ DNS record created: {}", record_id);
                println!("💡 Run 'drago setup' to also generate an API key for syncing.");
            }
            Err(e) => {
                eprintln!("❌ Failed to add record: {}", e);
                process::exit(1);
            }
        },
        Commands::Remove { record, zone } => match api::delete_record(&record, &zone) {
            Ok(()) => println!("✅ DNS record deleted"),
            Err(e) => {
                eprintln!("❌ Failed to delete record: {}", e);
                process::exit(1);
            }
        },
        Commands::Keys => match api::list_api_keys() {
            Ok(keys) => {
                if keys.is_empty() {
                    println!("No API keys found.");
                } else {
                    println!("🔑 API Keys:\n");
                    for key in keys {
                        println!("  {} - {} (record: {})", key.id, key.name, key.record_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to list API keys: {}", e);
                process::exit(1);
            }
        },
        Commands::Setup { zone, name } => match api::setup_record_with_key(&zone, &name) {
            Ok((record_name, api_key)) => {
                println!("✅ Setup complete!");
                println!("📍 DNS Record: {}", record_name);
                println!("\n🚀 Run 'drago start' to begin syncing your IP!");
                println!("   You will be prompted to enter the API key below.\n");

                // Show the API key once (user should save it)
                println!("⚠️  Your API key (save this, it won't be shown again):");
                println!("   {}", api_key);
            }
            Err(e) => {
                eprintln!("❌ Failed to setup: {}", e);
                process::exit(1);
            }
        },
    }
}
