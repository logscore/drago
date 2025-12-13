use crate::api;
use daemonize::Daemonize;
use std::fs::OpenOptions;
use std::io::Write;
use std::process;
use std::thread;
use std::time::Duration;

const PID_FILE: &str = "/tmp/drago.pid";
const LOG_OUT: &str = "/tmp/drago.out";
const LOG_ERR: &str = "/tmp/drago.err";
const SYNC_INTERVAL: u64 = 300; // 5 minutes

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Create log files if they don't exist
    let stdout = OpenOptions::new().create(true).append(true).open(LOG_OUT)?;

    let stderr = OpenOptions::new().create(true).append(true).open(LOG_ERR)?;

    // Daemonize the process
    let daemonize = Daemonize::new()
        .pid_file(PID_FILE)
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            log_message("Daemon started successfully");
            loop {
                match sync_ip() {
                    Ok(()) => {
                        // If successful, just continue
                    }
                    Err(e) => {
                        log_error(&format!("Sync failed: {}", e));
                    }
                }

                thread::sleep(Duration::from_secs(SYNC_INTERVAL));
            }
        }
        Err(e) => {
            eprintln!("Failed to daemonize: {}", e);
            process::exit(1);
        }
    }
}

fn sync_ip() -> Result<(), Box<dyn std::error::Error>> {
    log_message("Starting IP sync");

    // Get public IP (usually the router IP)
    let ip = match api::get_public_ip() {
        Ok(ip) => {
            log_message(&format!("Public IP: {}", ip));
            ip
        }
        Err(e) => {
            log_error(&format!("Failed to get public IP: {}", e));
            return Err(e);
        }
    };

    // Sync to Drago API
    match api::sync_ip_to_api(&ip) {
        Ok(resp_msg) => {
            log_message(&format!("Sync completed successfully: {}", resp_msg));
        }
        Err(e) => {
            log_error(&format!("Sync failed: {}", e));
            return Err(e);
        }
    }

    Ok(())
}

fn log_message(message: &str) {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let log_line = format!("[{}] {}\n", timestamp, message);

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_OUT)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

fn log_error(message: &str) {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let log_line = format!("[{}] {}\n", timestamp, message);

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_ERR)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}
