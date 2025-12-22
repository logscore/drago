use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use jsonwebtoken::decode_header;
use rand::{distr::Alphanumeric, Rng};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

// Base64 engine for URL-safe decoding
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

// 1. The Claims struct (What is inside the token)
#[derive(Debug, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // This is the User ID
    pub exp: usize,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// 2. JWKS Response Structures
#[derive(Deserialize, Debug)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Deserialize, Debug, Clone)]
struct JwkKey {
    #[serde(rename = "kty")]
    key_type: String,
    #[serde(rename = "crv")]
    curve: Option<String>,
    #[serde(rename = "x")]
    x: String, // Ed25519 public key (base64url encoded)
    #[serde(rename = "kid")]
    kid: String,
}

// 3. Shared State for caching Keys (store raw public key bytes for Ed25519)
#[derive(Clone)]
pub struct AuthState {
    pub keystore: Arc<RwLock<HashMap<String, ed25519_dalek::VerifyingKey>>>,
    pub jwks_url: String,
    pub http_client: Client,
}

impl AuthState {
    pub fn new(base_url: &str) -> Self {
        Self {
            keystore: Arc::new(RwLock::new(HashMap::new())),
            jwks_url: format!("{}/api/auth/jwks", base_url),
            http_client: Client::new(),
        }
    }

    pub async fn get_key(&self, kid: &str) -> Result<ed25519_dalek::VerifyingKey, String> {
        // Check cache first
        {
            let store = self.keystore.read().await;
            if let Some(key) = store.get(kid) {
                return Ok(*key);
            }
        }

        // Fetch from Better Auth if missing
        let resp = self
            .http_client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|_| "Failed to send JWKS request")?;

        if !resp.status().is_success() {
            return Err(format!("JWKS endpoint returned status: {}", resp.status()));
        }

        let jwks: JwksResponse = resp.json().await.map_err(|_| "Failed to parse JWKS")?;

        let mut store = self.keystore.write().await;
        for key in jwks.keys {
            // Only process Ed25519 keys (EdDSA with Ed25519 curve)
            if key.key_type == "OKP" && key.curve.as_deref() == Some("Ed25519") {
                // Decode base64url public key
                let key_bytes = URL_SAFE_NO_PAD
                    .decode(&key.x)
                    .map_err(|_| "Failed to decode public key")?;

                if key_bytes.len() == 32 {
                    let key_array: [u8; 32] = key_bytes[..32]
                        .try_into()
                        .map_err(|_| "Invalid key length")?;

                    if let Ok(verifying_key) = ed25519_dalek::VerifyingKey::from_bytes(&key_array) {
                        store.insert(key.kid.clone(), verifying_key);
                    }
                }
            }
        }

        store
            .get(kid)
            .copied()
            .ok_or_else(|| format!("Key ID '{}' not found in JWKS", kid))
    }
}

// 4. The Extractor (Middleware)
pub struct User(pub Claims);

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    AuthState: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_state = AuthState::from_ref(state);

        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization".into()))?
            .to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid header".into()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid format".into()));
        }

        let token = &auth_header[7..];
        let header = decode_header(token)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token header".into()))?;

        let kid = header
            .kid
            .ok_or((StatusCode::UNAUTHORIZED, "Missing kid".into()))?;

        let verifying_key = auth_state
            .get_key(&kid)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        // Parse JWT manually for Ed25519 verification
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err((StatusCode::UNAUTHORIZED, "Invalid auth token format".into()));
        }

        let header_b64 = parts[0];
        let payload_b64 = parts[1];
        let signature_b64 = parts[2];

        // Decode signature
        let signature_bytes = URL_SAFE_NO_PAD.decode(signature_b64).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Failed to decode signature".into(),
            )
        })?;

        if signature_bytes.len() != 64 {
            return Err((StatusCode::UNAUTHORIZED, "Invalid signature length".into()));
        }

        let signature_array: [u8; 64] = signature_bytes[..64]
            .try_into()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid signature format".into()))?;
        let signature = ed25519_dalek::Signature::from_bytes(&signature_array);

        // Verify signature
        let message = format!("{}.{}", header_b64, payload_b64);
        verifying_key
            .verify_strict(message.as_bytes(), &signature)
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Signature verification failed".into(),
                )
            })?;

        // Decode payload
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(payload_b64)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Failed to decode payload".into()))?;

        let payload: Value = serde_json::from_slice(&payload_bytes)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Failed to parse payload".into()))?;

        // Convert to Claims
        let mut claims: Claims = serde_json::from_value(payload.clone())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Failed to parse claims".into()))?;

        // Extract extra fields
        if let Ok(extra) = serde_json::from_value::<HashMap<String, Value>>(payload) {
            claims.extra = extra;
        }

        // Validate expiration
        let now = chrono::Utc::now().timestamp() as usize;
        if claims.exp < now {
            return Err((StatusCode::UNAUTHORIZED, "Token has expired".into()));
        }

        Ok(User(claims))
    }
}

pub fn generate_api_key() -> (String, String, String) {
    // Public id isnt hashed, but is used for quick api lookups
    let public_id: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    // Secret is hashed
    let secret: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let full_key = format!("dgo_{}_{}", public_id, secret);

    (full_key, public_id, secret)
}
