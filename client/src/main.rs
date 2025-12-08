extern crate daemonize;

use daemonize::Daemonize;
use reqwest::blocking::Client;
use serde_json::json; // NOTE: Using blocking client
use std::{
    fs::{self, *},
    path::Path,
    thread,
    time::Duration,
};

fn main() {
    let stdout = File::create("/tmp/drago.out").unwrap();
    let stderr = File::create("/tmp/drago.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/drago.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .user("nobody")
        .group("daemon")
        .umask(0o777)
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            let client = Client::new();
            let response = get_network_ip(&client);

            // Log the result to the stdout/stderr files we created
            dbg!(&response);

            if response.success {
                println!("Success: {}", response.message);
                if let Ok(ip) = response.data {
                    println!("IP is: {}", ip);
                    // Send to Drago sync endpoint here...
                    let api_key = match read_api_key_from_file() {
                        Ok(key) => key,
                        Err(e) => {
                            eprintln!("Error reading API key from .conf file: {}", e);
                            String::new()
                        }
                    };

                    let sync_response = send_sync(&client, &ip, &api_key);
                    match sync_response {
                        Ok(_) => println!("Sync successful"),
                        Err(e) => eprintln!("Sync failed: {}", e),
                    }
                }
            } else {
                eprintln!("Failure: {}", response.message);
            }
            // SET THIS DURATION TO 300 FOR PROD
            thread::sleep(Duration::from_secs(10));
        }
        Err(e) => eprintln!("Error starting daemon: {}", e),
    }
}

// Fixed: Explicit return type. Changed T to String because you want the IP text.
fn get_network_ip(client: &Client) -> ApiResponse<String> {
    // 1. Send Request
    match client.get("https://api.ipify.org").send() {
        Ok(resp) => {
            // 2. Check HTTP status
            if !resp.status().is_success() {
                return ApiResponse {
                    success: false,
                    data: Err(format!("HTTP Error: {}", resp.status())),
                    message: "Server returned error code".to_string(),
                };
            }

            // 3. Extract Body (IP Address)
            match resp.text() {
                Ok(ip_text) => ApiResponse {
                    success: true,
                    data: Ok(ip_text),
                    message: "Successfully retrieved network IP address".to_string(),
                },
                Err(e) => ApiResponse {
                    success: false,
                    data: Err(e.to_string()),
                    message: "Failed to read response body".to_string(),
                },
            }
        }
        Err(error) => ApiResponse {
            success: false,
            data: Err(error.to_string()),
            message: "Failed to connect to network IP service".to_string(),
        },
    }
}

fn read_api_key_from_file() -> Result<String, Box<dyn std::error::Error>> {
    let key_path = "/etc/drago/app.conf";

    // Check file
    if !Path::new(key_path).exists() {
        return Err(format!("API key file not found at {}", key_path).into());
    }

    // Read file
    let api_key = fs::read_to_string(key_path)?.trim().to_string();

    Ok(api_key)
}

fn send_sync(client: &Client, ip_address: &str, api_key: &str) -> Result<(), String> {
    let payload = json!({
        "ip": ip_address,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let response = client
        .post("http://127.0.0.1:8080/sync")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .map_err(|e| format!("Failed to send sync request: {}", e))?;

    // Check response status
    if response.status().is_success() {
        println!("Sync request successful");
        Ok(())
    } else {
        Err(format!(
            "Sync request failed with status: {}",
            response.status()
        ))
    }
}

// ==========TYPES=========
#[derive(Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Result<T, String>,
    pub message: String,
}
