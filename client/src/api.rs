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

    // API key is required for sync
    let api_key = config
        .api_key
        .ok_or("No API key configured. Run 'drago setup' first.")?;

    let client = Client::new();
    let api_url = option_env!("DRAGO_API_URL").unwrap_or("http://127.0.0.1:8080");

    let time_synced = Utc::now().naive_utc();
    let req_body = SyncRequest {
        ip_address: ip.to_string(),
        time_synced,
    };

    let resp = client
        .put(&format!("{}/sync", api_url))
        .bearer_auth(&api_key)
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
    // let url_to_open = device_response
    //     .verification_uri_complete
    //     .unwrap_or(device_response.verification_uri);

    // // Platform-specific browser open
    // #[cfg(target_os = "macos")]
    // let _ = std::process::Command::new("open")
    //     .arg(&url_to_open)
    //     .spawn();
    // #[cfg(target_os = "linux")]
    // let _ = std::process::Command::new("xdg-open")
    //     .arg(&url_to_open)
    //     .spawn();
    // #[cfg(target_os = "windows")]
    // let _ = std::process::Command::new("cmd")
    //     .args(["/C", "start", &url_to_open])
    //     .spawn();

    // println!("🌐 Opening browser...");
    // println!(
    //     "⏳ Waiting for authorization... (polling every {}s)",
    //     device_response.interval
    // );
    // print!("Progress:");
    // io::stdout().flush().ok();

    // Poll for session token
    let session_token = poll_for_device_token(device_response.device_code)?;

    println!("\n✅ Device authorization successful!");
    println!("🔄 Retrieving auth token...");

    // Exchange session token for JWT
    let jwt_token = exchange_session_for_jwt(&session_token)?;

    println!("✅ Auth token obtained!");

    Ok(jwt_token)
}

/// Exchange a session token for a JWT token
fn exchange_session_for_jwt(session_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let frontend_url = option_env!("DRAGO_FRONTEND_URL").unwrap_or("http://localhost:5173");

    let resp = client
        .get(&format!("{}/api/auth/token", frontend_url))
        .bearer_auth(session_token)
        .timeout(Duration::from_secs(10))
        .send()?;

    if !resp.status().is_success() {
        return Err(format!("Failed to get JWT token: HTTP {}", resp.status()).into());
    }

    #[derive(Deserialize)]
    struct JwtResponse {
        token: String,
    }

    let jwt_response: JwtResponse = resp.json()?;
    Ok(jwt_response.token)
}

// ============================================================================
// CRUD Operations (use JWT token for authentication)
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub ttl: i32,
    pub proxied: bool,
}

#[derive(Deserialize, Debug)]
struct ZoneRecordsResponse(Vec<(Zone, Vec<DnsRecord>)>);

#[derive(Deserialize, Debug)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub record_name: String,
}

#[derive(Serialize)]
struct AddRecordRequest {
    zone_id: String,
    zone_name: String,
    record_type: String,
    name: String,
    content: String,
    ttl: i32,
    proxied: bool,
}

#[derive(Serialize)]
struct AddApiKeyRequest {
    name: String,
    scope: String, // record_id
}

fn get_api_url() -> String {
    option_env!("DRAGO_API_URL")
        .unwrap_or("http://127.0.0.1:8080")
        .to_string()
}

fn get_jwt_token() -> Result<String, Box<dyn std::error::Error>> {
    let config = config::load_config()?;
    Ok(config.access_token)
}

/// List all DNS zones for the authenticated user
pub fn list_zones() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();
    let token = get_jwt_token()?;

    let resp = client
        .get(&format!("{}/records", api_url))
        .bearer_auth(&token)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        return Err(format!("Failed to list zones: HTTP {}", resp.status()).into());
    }

    let data: Vec<(Zone, Vec<DnsRecord>)> = resp.json()?;
    let zones: Vec<(String, String)> = data.into_iter().map(|(z, _)| (z.id, z.name)).collect();

    Ok(zones)
}

/// List all DNS records for the authenticated user
pub fn list_records() -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();
    let token = get_jwt_token()?;

    let resp = client
        .get(&format!("{}/records", api_url))
        .bearer_auth(&token)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        return Err(format!("Failed to list records: HTTP {}", resp.status()).into());
    }

    let data: Vec<(Zone, Vec<DnsRecord>)> = resp.json()?;
    let records: Vec<DnsRecord> = data.into_iter().flat_map(|(_, records)| records).collect();

    Ok(records)
}

/// Add a new DNS record
pub fn add_record(
    zone_id: &str,
    subdomain: &str,
    ttl: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();
    let token = get_jwt_token()?;

    // Get zone name first
    let zones = list_zones()?;
    let zone_name = zones
        .iter()
        .find(|(id, _)| id == zone_id)
        .map(|(_, name)| name.clone())
        .ok_or("Zone not found")?;

    // Get current public IP for initial content
    let ip = get_public_ip().unwrap_or_else(|_| "0.0.0.0".to_string());

    let request = AddRecordRequest {
        zone_id: zone_id.to_string(),
        zone_name,
        record_type: "A".to_string(),
        name: subdomain.to_string(),
        content: ip,
        ttl: ttl,
        proxied: false,
    };

    let resp = client
        .post(&format!("{}/record", api_url))
        .bearer_auth(&token)
        .json(&request)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        let text = resp.text()?;
        return Err(format!("Failed to add record: {}", text).into());
    }

    Ok(format!("{}.{}", subdomain, request.zone_name))
}

/// Delete a DNS record
pub fn delete_record(record_id: &str, zone_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();
    let token = get_jwt_token()?;

    let resp = client
        .delete(&format!(
            "{}/record?record_id={}&zone_id={}",
            api_url, record_id, zone_id
        ))
        .bearer_auth(&token)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        let text = resp.text()?;
        return Err(format!("Failed to delete record: {}", text).into());
    }

    Ok(())
}

/// List all API keys for the authenticated user
pub fn list_api_keys() -> Result<Vec<ApiKeyInfo>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();
    let token = get_jwt_token()?;

    let resp = client
        .get(&format!("{}/api_keys", api_url))
        .bearer_auth(&token)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        return Err(format!("Failed to list API keys: HTTP {}", resp.status()).into());
    }

    let keys: Vec<ApiKeyInfo> = resp.json()?;
    Ok(keys)
}

/// Create an API key for a record
fn add_api_key(name: &str, record_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();
    let token = get_jwt_token()?;

    let request = AddApiKeyRequest {
        name: name.to_string(),
        scope: record_id.to_string(),
    };

    let resp = client
        .post(&format!("{}/api_key", api_url))
        .bearer_auth(&token)
        .json(&request)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        let text = resp.text()?;
        return Err(format!("Failed to create API key: {}", text).into());
    }

    // The response is the API key as a string
    let api_key: String = resp.json()?;
    Ok(api_key)
}

#[derive(Deserialize, Debug)]
pub struct DnsAccessToken {
    pub id: String,
    pub name: String,
    pub created_on: chrono::NaiveDateTime,
}

/// Get existing Cloudflare access tokens from the API
pub fn get_cloudflare_tokens(
    jwt_token: &str,
) -> Result<Vec<DnsAccessToken>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();

    let resp = client
        .get(&format!("{}/access_tokens", api_url))
        .bearer_auth(jwt_token)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        let text = resp.text()?;
        return Err(format!("Failed to get Cloudflare tokens: {}", text).into());
    }

    let tokens: Vec<DnsAccessToken> = resp.json()?;
    Ok(tokens)
}

/// Store Cloudflare access token in the API
pub fn store_cloudflare_token(
    jwt_token: &str,
    token_name: &str,
    cloudflare_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_url = get_api_url();

    #[derive(Serialize)]
    struct StoreTokenRequest {
        name: String,
        token: String,
    }

    let request = StoreTokenRequest {
        name: token_name.to_string(),
        token: cloudflare_token.to_string(),
    };

    let resp = client
        .post(&format!("{}/access_token", api_url))
        .bearer_auth(jwt_token)
        .json(&request)
        .timeout(Duration::from_secs(30))
        .send()?;

    if !resp.status().is_success() {
        let text = resp.text()?;
        return Err(format!("Failed to store Cloudflare token: {}", text).into());
    }

    Ok(())
}

/// Setup: create a DNS record and API key (does NOT auto-save the key)
pub fn setup_record_with_key(
    zone_id: &str,
    subdomain: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    println!("Creating DNS record...");
    let ttl = 300;

    // First, create the record
    let record_name = add_record(zone_id, subdomain, ttl)?;
    println!("   Record created: {}", record_name);

    // Find the record ID we just created
    let records = list_records()?;
    let record = records
        .iter()
        .find(|r| r.name == record_name)
        .ok_or("Could not find the record we just created")?;

    println!("🔑 Creating API key...");

    // Create an API key for this record
    let api_key = add_api_key(&format!("drago-{}", subdomain), &record.id)?;
    println!("   API key created");

    Ok((record_name, api_key))
}
