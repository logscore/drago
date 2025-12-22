use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub api_key: Option<String>,
}

fn get_home_dir() -> io::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })
}

pub fn get_config_path() -> PathBuf {
    let home = get_home_dir().unwrap_or_else(|_| PathBuf::from("."));
    let config_path = home.join(".config").join("drago").join("config.json");
    config_path
}

pub fn store_device_token() -> io::Result<()> {
    println!("Starting device authorization flow...");

    let access_token = match crate::api::authenticate_with_device_flow() {
        Ok(token) => token,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Device authorization failed: {}", e),
            ));
        }
    };

    let config_path = get_config_path();
    let parent_dir = config_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid config path"))?;

    // Ensure directory exists
    if !parent_dir.exists() {
        eprintln!("Creating config directory: {}", parent_dir.display());
        fs::create_dir_all(parent_dir)?;
    }

    // Load existing config or create new one
    let api_key = load_config().ok().and_then(|c| c.api_key);

    // Write config JSON with access token
    let config = Config {
        access_token,
        token_type: "Bearer".to_string(),
        expires_at: None, // Could implement token refresh in future
        api_key,
    };

    let data = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, data)?;

    // Ensure secure file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&config_path, perms)?;
    }

    // Verification and helpful output
    if config_path.exists() {
        println!("✅ Access token saved to {}", config_path.display());
    } else {
        eprintln!(
            "❌ Failed to verify config creation at {}",
            config_path.display()
        );
    }

    Ok(())
}

pub fn load_config() -> io::Result<Config> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Config file not found at {}. Run 'drago init' first.",
                config_path.display()
            ),
        ));
    }

    let contents = fs::read_to_string(&config_path)?;

    let config: Config = serde_json::from_str(&contents)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(config)
}

/// Save an API key to the config file (preserves existing access token)
pub fn save_api_key(api_key: &str) -> io::Result<()> {
    let config_path = get_config_path();

    // Load existing config
    let mut config = load_config()?;

    // Update API key
    config.api_key = Some(api_key.to_string());

    // Write back
    let data = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, data)?;

    // Ensure secure file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&config_path, perms)?;
    }

    Ok(())
}
