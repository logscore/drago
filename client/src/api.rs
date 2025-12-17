use crate::config;
use chrono::Utc;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

#[derive(Serialize)]
struct SyncRequest {
    ip_address: String,
    time_synced: chrono::NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct SyncResponse {
    pub success: bool,
    pub updated: bool,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Serialize)]
struct DeviceTokenRequest {
    grant_type: String,
    device_code: String,
    client_id: String,
}

#[derive(Deserialize, Debug)]
pub struct DeviceTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct DeviceErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

#[derive(Serialize)]
struct DeviceCodeRequest {
    client_id: String,
    scope: Option<String>,
}

pub fn get_public_ip() -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let ip = client
        .get("https://api.ipify.org")
        .timeout(Duration::from_secs(10))
        .send()?
        .text()?;
    Ok(ip.trim().to_string())
}

pub fn sync_ip_to_api(ip: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = config::load_config()?;
    let client = Client::new();

    let api_url = option_env!("DRAGO_API_URL").unwrap_or("http://127.0.0.1:8080");

    let time_synced = Utc::now().naive_utc();
    let req_body = SyncRequest {
        ip_address: ip.to_string(),
        time_synced,
    };

    let resp = client
        .put(&format!("{}/sync", api_url))
        .bearer_auth(&config.access_token)
        .json(&req_body)
        .timeout(Duration::from_secs(30))
        .send()?;

    let url = resp.url().to_string();
    let status = resp.status();
    let text = resp.text()?;

    if !status.is_success() {
        return Err(format!("{} HTTP {}: {}", url, status, text).into());
    }

    // Try to parse the structured API response
    match serde_json::from_str::<SyncResponse>(&text) {
        Ok(api_resp) => {
            if api_resp.success {
                Ok(format!(
                    "Sync succeeded (record updated: {}): {}",
                    api_resp.updated, api_resp.message
                ))
            } else {
                Err(format!("API error: {}", api_resp.message).into())
            }
        }
        Err(_) => {
            // fallback if unexpected body
            Ok(format!("Sync completed but unexpected response: {}", text))
        }
    }
}

pub fn start_device_authorization() -> Result<DeviceCodeResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let frontend_url = option_env!("DRAGO_FRONTEND_URL").unwrap_or("http://localhost:5173");

    let request = DeviceCodeRequest {
        client_id: "drago-dns-cli".to_string(),
        scope: Some("openid profile email dns:read dns:write".to_string()),
    };

    let resp = client
        .post(&format!("{}/api/auth/device/code", frontend_url))
        .json(&request)
        .timeout(Duration::from_secs(10))
        .send()?;

    if !resp.status().is_success() {
        return Err(format!("Device code request failed: {}", resp.status()).into());
    }

    let response: DeviceCodeResponse = resp.json()?;
    Ok(response)
}

pub fn poll_for_device_token(device_code: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let frontend_url = option_env!("DRAGO_FRONTEND_URL").unwrap_or("http://localhost:5173");

    let mut polling_interval = 5; // Start with 5 seconds
    let max_attempts = 600; // 30 minutes max

    for attempt in 0..max_attempts {
        let request = DeviceTokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            device_code: device_code.clone(),
            client_id: "drago-dns-cli".to_string(),
        };

        let resp = client
            .post(&format!("{}/api/auth/device/token", frontend_url))
            .json(&request)
            .timeout(Duration::from_secs(10))
            .send();

        match resp {
            Ok(response) => {
                let status = response.status();
                let body = response.text()?;

                if status.is_success() {
                    let token_response: DeviceTokenResponse = serde_json::from_str(&body)?;
                    return Ok(token_response.access_token);
                } else {
                    let error_resp = serde_json::from_str::<DeviceErrorResponse>(&body).unwrap_or(
                        DeviceErrorResponse {
                            error: "unknown_error".to_string(),
                            error_description: Some(body),
                        },
                    );

                    match error_resp.error.as_str() {
                        "authorization_pending" => {
                            // continue polling
                        }
                        "slow_down" => {
                            polling_interval += 5;
                            eprintln!("⚠️  Slowing down polling to {}s", polling_interval);
                        }
                        "access_denied" => {
                            return Err("Access was denied by the user".into());
                        }
                        "expired_token" => {
                            return Err("The device code has expired. Please try again.".into());
                        }
                        _ => {
                            let desc = error_resp.error_description.unwrap_or(error_resp.error);

                            return Err(format!("Device authorization failed: {}", desc).into());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Network error: {}", e);
                // continue polling
            }
        }

        thread::sleep(Duration::from_secs(polling_interval));

        if attempt % 12 == 0 {
            // Every minute
            print!(".");
            io::stdout().flush().ok();
        }
    }

    println!(); // New line after progress dots
    Err("Device authorization timed out".into())
}

pub fn authenticate_with_device_flow() -> Result<String, Box<dyn std::error::Error>> {
    println!("🔐 DragoDNS Device Authorization");
    println!("⏳ Requesting device authorization...");

    // Request device code
    let device_response = start_device_authorization()?;

    println!("\n📱 Device Authorization in Progress");
    println!("Please visit: {}", device_response.verification_uri);
    println!("Enter code: {}", device_response.user_code);

    // Try to open browser automatically
    let url_to_open = device_response
        .verification_uri_complete
        .unwrap_or(device_response.verification_uri);

    let _ = std::process::Command::new("xdg-open")
        .arg(&url_to_open)
        .spawn();

    println!("🌐 Opening browser...");
    println!(
        "⏳ Waiting for authorization... (polling every {}s)",
        device_response.interval
    );
    print!("Progress:");
    io::stdout().flush().ok();

    // Poll for token
    let access_token = poll_for_device_token(device_response.device_code)?;

    println!("\n✅ Authorization Successful!");
    println!("Access token received and ready to use.");

    Ok(access_token)
}
