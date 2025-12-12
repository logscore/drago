use crate::config;
use chrono::Utc;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Serialize)]
struct SyncRequest {
    ip_address: String,
    time_synced: chrono::NaiveDateTime,
}

#[derive(Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
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

    let api_url = env::var("DRAGO_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let time_synced = Utc::now().naive_utc();
    let req_body = SyncRequest {
        ip_address: ip.to_string(),
        time_synced,
    };

    let resp = client
        .put(&format!("{}/sync", api_url))
        .bearer_auth(config.api_key)
        .json(&req_body)
        .timeout(Duration::from_secs(30))
        .send()?;

    let status = resp.status();
    let text = resp.text()?;

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text).into());
    }

    // Try to parse the structured API response
    match serde_json::from_str::<ApiResponse<serde_json::Value>>(&text) {
        Ok(api_resp) => {
            if api_resp.success {
                Ok(format!("✅ Sync succeeded: {}", api_resp.message))
            } else {
                Err(format!("❌ API error: {}", api_resp.message).into())
            }
        }
        Err(_) => {
            // fallback if unexpected body
            Ok(format!(
                "⚠️ Sync completed but unexpected response: {}",
                text
            ))
        }
    }
}
