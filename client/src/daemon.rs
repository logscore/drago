// extern crate daemonize;

// use daemonize::Daemonize;
// use reqwest::blocking::Client;
// use serde_json::json; // NOTE: Using blocking client
// use std::{
//     fs::{self, *},
//     path::Path,
//     thread,
//     time::Duration,
// };

// fn main() {
//     let stdout = File::create("/tmp/drago.out").unwrap();
//     let stderr = File::create("/tmp/drago.err").unwrap();

//     let daemonize = Daemonize::new()
//         .pid_file("/tmp/drago.pid")
//         .chown_pid_file(true)
//         .working_directory("/tmp")
//         .user("nobody")
//         .group("daemon")
//         .umask(0o777)
//         .stdout(stdout)
//         .stderr(stderr);

//     match daemonize.start() {
//         Ok(_) => {
//             println!(
//                 "Drago daemon started successfully (PID {}).",
//                 std::process::id()
//             );
//             loop {
//                 let client = Client::new();
//                 let response = get_network_ip(&client);

//                 // Log the result to the stdout/stderr files we created
//                 dbg!(&response);

//                 if response.success {
//                     println!("Success: {}", response.message);
//                     if let Ok(ip) = response.data {
//                         println!("IP is: {}", ip);
//                         // Send to Drago sync endpoint here...
//                         let api_key = match read_api_key_from_file() {
//                             Ok(key) => key,
//                             Err(e) => {
//                                 eprintln!("Error reading API key from .conf file: {}", e);
//                                 String::new()
//                             }
//                         };

//                         let sync_response = send_sync(&client, &ip, &api_key);
//                         match sync_response {
//                             Ok(_) => println!("Sync successful"),
//                             Err(e) => eprintln!("Sync failed: {}", e),
//                         }
//                     }
//                 } else {
//                     eprintln!("Failure: {}", response.message);
//                 }
//                 // SET THIS DURATION TO 300 FOR PROD
//                 thread::sleep(Duration::from_secs(10));
//             }
//         }
//         Err(e) => eprintln!("Error starting daemon: {}", e),
//     }
// }

// // Fixed: Explicit return type. Changed T to String because you want the IP text.
// fn get_network_ip(client: &Client) -> ApiResponse<String> {
//     // 1. Send Request
//     match client.get("https://api.ipify.org").send() {
//         Ok(resp) => {
//             // 2. Check HTTP status
//             if !resp.status().is_success() {
//                 return ApiResponse {
//                     success: false,
//                     data: Err(format!("HTTP Error: {}", resp.status())),
//                     message: "Server returned error code".to_string(),
//                 };
//             }

//             // 3. Extract Body (IP Address)
//             match resp.text() {
//                 Ok(ip_text) => ApiResponse {
//                     success: true,
//                     data: Ok(ip_text),
//                     message: "Successfully retrieved network IP address".to_string(),
//                 },
//                 Err(e) => ApiResponse {
//                     success: false,
//                     data: Err(e.to_string()),
//                     message: "Failed to read response body".to_string(),
//                 },
//             }
//         }
//         Err(error) => ApiResponse {
//             success: false,
//             data: Err(error.to_string()),
//             message: "Failed to connect to network IP service".to_string(),
//         },
//     }
// }

// fn read_api_key_from_file() -> Result<String, Box<dyn std::error::Error>> {
//     let key_path = "/etc/drago/app.conf";

//     // Check file
//     if !Path::new(key_path).exists() {
//         return Err(format!("API key file not found at {}", key_path).into());
//     }

//     // Read file
//     let api_key = fs::read_to_string(key_path)?.trim().to_string();

//     Ok(api_key)
// }

// fn send_sync(client: &Client, ip_address: &str, api_key: &str) -> Result<(), String> {
//     let drago_api_url: &str = option_env!("DRAGO_API_URL").unwrap_or("http://localhost:8080");

//     let payload = json!({
//         "ip": ip_address,
//         "timestamp": chrono::Utc::now().to_rfc3339()
//     });

//     let response = client
//         .post(format!("{}/sync", drago_api_url))
//         .header("Authorization", format!("Bearer {}", api_key))
//         .header("Content-Type", "application/json")
//         .body(payload.to_string())
//         .send()
//         .map_err(|e| format!("Failed to send sync request: {}", e))?;

//     // Check response status
//     if response.status().is_success() {
//         println!("Sync request successful");
//         Ok(())
//     } else {
//         Err(format!(
//             "Sync request failed with status: {}",
//             response.status()
//         ))
//     }
// }

// // ==========TYPES=========
// #[derive(Debug)]
// pub struct ApiResponse<T> {
//     pub success: bool,
//     pub data: Result<T, String>,
//     pub message: String,
// }

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

            // Main daemon loop
            loop {
                match sync_ip() {
                    Ok(()) => {
                        // Success, continue
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

    // Get public IP
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

    // Sync to API
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
