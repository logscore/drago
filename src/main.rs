mod db;
mod lib;

use crate::{
    db::schema::{api_keys, dns_record, dns_token, dns_zone},
    lib::{
        auth::User,
        encryption::{decrypt, encrypt},
        types::*,
        utils::{get_user_token, hash_raw_string},
    },
};
use crate::{
    db::{models::NewDnsAccessToken, *},
    lib::auth::{generate_api_key, AuthState},
};
use axum::{
    extract::{FromRef, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    auth: AuthState,
}

impl FromRef<AppState> for AuthState {
    fn from_ref(input: &AppState) -> Self {
        input.auth.clone()
    }
}

impl FromRef<AppState> for Pool<ConnectionManager<MysqlConnection>> {
    fn from_ref(input: &AppState) -> Self {
        input.pool.clone()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
    let api_url = env::var("API_URL").expect("API_URL must be set");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let addr: SocketAddr = api_url.parse().expect("Invalid API_URL format");

    let manager = ConnectionManager::<MysqlConnection>::new(db_url);
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool");

    let auth_state = crate::lib::auth::AuthState::new(&frontend_url);

    let state = AppState {
        pool,
        auth: auth_state,
    };

    let cors = CorsLayer::new()
        .allow_origin(frontend_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::PUT,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .expose_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/records", get(list_dns_records))
        .route("/record", post(add_dns_record))
        .route("/record", delete(delete_dns_record))
        .route("/access_tokens", get(get_dns_access_tokens))
        .route("/access_token", post(add_dns_access_token))
        .route("/access_token", delete(delete_access_token))
        .route("/api_keys", get(get_api_keys))
        .route("/api_key", post(add_api_key))
        .route("/api_key", delete(delete_api_key))
        .route("/sync", put(sync_record))
        .with_state(state)
        .layer(cors);

    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json("Drago is running. Let it rip.".to_string()),
    )
}

async fn sync_record(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SyncRequest>,
) -> impl IntoResponse {
    let ip_addr = body.ip_address;
    let time_synced = body.time_synced;

    let api_key = match headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer").map(str::trim))
    {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SyncResponse {
                    success: false,
                    updated: false,
                    message: "Missing or invalid Authorization header (expected Bearer token)"
                        .to_string(),
                }),
            )
                .into_response();
        }
    };

    // Only accept API keys (format: dgo_<prefix>_<secret>)
    if !api_key.starts_with("dgo_") {
        return (
            StatusCode::UNAUTHORIZED,
            Json(SyncResponse {
                success: false,
                updated: false,
                message: "Invalid API key format. Expected dgo_<prefix>_<secret>".to_string(),
            }),
        )
            .into_response();
    }

    sync_with_api_key(&state, api_key, &ip_addr, &time_synced).await
}

/// Sync using API key - updates the specific record linked to the key
async fn sync_with_api_key(
    state: &AppState,
    api_key: &str,
    ip_addr: &str,
    time_synced: &NaiveDateTime,
) -> axum::response::Response {
    let key_parts: Vec<&str> = api_key.split("_").collect();

    let public_prefix = match key_parts.get(1) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SyncResponse {
                    success: false,
                    updated: false,
                    message: "Malformed API key.".to_string(),
                }),
            )
                .into_response();
        }
    };

    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    // query the db for that value
    let api_key_query_response = api_keys::table
        .select(api_keys::id)
        .filter(api_keys::prefix_id.eq(public_prefix))
        .first::<String>(conn);

    // Well use this id to update the api key entry for last used
    let api_key_id = match api_key_query_response {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SyncResponse {
                    success: false,
                    updated: false,
                    message: "Invalid authorization".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Now grab the dns record associated with the key hash by querying for the dns id and joining the dns record table and returning the contents
    let connected_record = dns_record::table
        .inner_join(api_keys::table)
        .filter(dns_record::id.eq(api_keys::dns_record_id))
        .filter(api_keys::id.eq(&api_key_id))
        .select((
            api_keys::user_id,
            dns_record::id,
            dns_record::zone_id,
            dns_record::content,
            dns_record::record_name,
            dns_record::ttl,
            dns_record::record_type,
        ))
        .first::<models::PutDnsRecord>(conn);

    let connected_record_data = match connected_record {
        Ok(data) => data,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(SyncResponse {
                    success: false,
                    updated: false,
                    message: "No record accociated to API key".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Check if the incoming ip is the same as the existing one.
    if connected_record_data.content == ip_addr {
        (
            StatusCode::OK,
            Json(SyncResponse {
                success: true,
                updated: false,
                message: "Record unchanged".to_string(),
            }),
        )
            .into_response()
    } else {
        // If not, post to cloudlfare with the new ip
        let decrypted_token = match get_user_token(conn, &connected_record_data.user_id) {
            Ok(token) => token,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SyncResponse {
                        success: false,
                        updated: false,
                        message: "Failed to get user token".to_string(),
                    }),
                )
                    .into_response();
            }
        };

        let client = reqwest::Client::new();
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            connected_record_data.zone_id, connected_record_data.id
        );

        let payload = PutDnsRecordPayload {
            // We only want to change the content, but type, name and ttl are required on the endpoint
            r#type: &connected_record_data.record_type,
            name: &connected_record_data.record_name,
            // The content is all that really changes in this call
            content: ip_addr,
            ttl: &connected_record_data.ttl,
        };

        let resp = match client
            .put(&url)
            .bearer_auth(&decrypted_token)
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SyncResponse {
                        success: false,
                        updated: false,
                        message: e.to_string(),
                    }),
                )
                    .into_response();
            }
        };

        let response = resp.json().await;

        dbg!(&response);
        let updated_dns_record_response_data: PutRecordResponse = match response {
            Ok(result) => result,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SyncResponse {
                        success: false,
                        updated: false,
                        message: e.to_string(),
                    }),
                )
                    .into_response();
            }
        };

        if updated_dns_record_response_data.success {
            let update_result = diesel::update(dns_record::table)
                .set((
                    dns_record::content.eq(ip_addr),
                    dns_record::last_synced_on.eq(time_synced),
                ))
                .filter(dns_record::user_id.eq(&connected_record_data.user_id))
                .filter(dns_record::id.eq(&updated_dns_record_response_data.result.id))
                .execute(conn);

            match update_result {
                Ok(_) => (
                    // Return success
                    StatusCode::OK,
                    Json(SyncResponse {
                        success: true,
                        updated: true,
                        message: "Record synced successfully".to_string(),
                    }),
                )
                    .into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SyncResponse {
                        success: false,
                        updated: false,
                        message: format!("DB update failed: {}", err),
                    }),
                )
                    .into_response(),
            }
        } else {
            (
                StatusCode::OK,
                Json(SyncResponse {
                    success: false,
                    updated: false,
                    message: "DNS provider error".to_string(),
                }),
            )
                .into_response()
        }
    }
}

// DNS Record Controls
async fn add_dns_record(
    State(state): State<AppState>,
    User(claims): User,
    Json(body): Json<AddDnsRecord>,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let zone_id = body.zone_id;
    let zone_name = body.zone_name;
    let record_type = body.record_type;
    let name = body.name;
    let content = body.content;
    let ttl = body.ttl;
    let proxied = body.proxied;

    let conn = &mut state.pool.get().expect("Failed to get DB connection"); // TODO: Dont save the full url as we reference the zone name in the record by id
    let subdomain = format!("{}.{}", name, zone_name);

    let result = dns_record::table
        .select(dns_record::id)
        .filter(dns_record::user_id.eq(&user_id))
        .filter(dns_record::zone_id.eq(&zone_id))
        .filter(dns_record::record_name.eq(&subdomain))
        .first::<String>(conn)
        .optional();

    let existing_record_id = match result {
        Ok(v) => v,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    if existing_record_id.is_some() {
        return (StatusCode::CONFLICT, Json("Record already exists")).into_response();
    }

    let decrypted_token = match get_user_token(conn, &user_id) {
        Ok(token) => token,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed to get user token"),
            )
                .into_response();
        }
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone_id
    );

    let payload = DnsRecordPayload {
        r#type: &record_type,
        name: &name,
        content: &content,
        ttl: &ttl,
        proxied: &proxied,
    };

    let resp = match client
        .post(&url)
        .bearer_auth(&decrypted_token)
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    };

    let response = resp.json::<CreateRecordResponse>().await;
    let new_token = match response {
        Ok(r) => r.result,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    };

    let result = conn.transaction(|conn| {
        diesel::insert_into(dns_record::table)
            .values((
                dns_record::id.eq(&new_token.id),
                dns_record::user_id.eq(&user_id),
                dns_record::record_name.eq(&new_token.name),
                dns_record::zone_id.eq(&zone_id),
                dns_record::content.eq(&new_token.content),
                dns_record::ttl.eq(&new_token.ttl),
                dns_record::record_type.eq(&new_token.record_type),
                dns_record::proxied.eq(&new_token.proxied),
            ))
            .execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(_) => (StatusCode::CREATED, Json("Record added successfully")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

async fn list_dns_records(State(state): State<AppState>, User(claims): User) -> impl IntoResponse {
    let curr_user_id = claims.sub;
    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    let zones: Vec<(String, String)> = dns_zone::table
        .filter(dns_zone::user_id.eq(&curr_user_id))
        .select((dns_zone::id, dns_zone::zone_name))
        .load(conn)
        .unwrap_or_default();

    if zones.is_empty() {
        let token_data = dns_token::table
            .filter(dns_token::user_id.eq(&curr_user_id))
            .select((
                dns_token::id,
                dns_token::token_encrypted,
                dns_token::nonce,
                dns_token::tag,
            ))
            .first::<(String, Vec<u8>, Vec<u8>, Vec<u8>)>(conn)
            .optional();

        let (token_id, ciphertext, nonce, tag) = match token_data {
            Ok(Some(data)) => data,
            Ok(None) => return (StatusCode::NOT_FOUND, "No DNS Token found").into_response(),
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response(),
        };

        let decrypted_token = match decrypt(&nonce, &ciphertext, &tag) {
            Ok(t) => t,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Security Error").into_response(),
        };

        let zones = match initialize_zones(conn, &curr_user_id, &decrypted_token, &token_id).await {
            Ok(z) => z,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error fetching DNS Zones",
                )
                    .into_response()
            }
        };

        let zone_dns_data: Vec<(Zone, Vec<DnsRecord>)> =
            zones.into_iter().map(|z| (z, Vec::new())).collect();

        return (StatusCode::OK, Json(&zone_dns_data)).into_response();
    }

    let zone_dns_data: Vec<(Zone, Vec<DnsRecord>)> = zones
        .into_iter()
        .map(|(z_id, z_name)| {
            let records = dns_record::table
                .filter(dns_record::zone_id.eq(&z_id))
                .filter(dns_record::user_id.eq(&curr_user_id))
                .select((
                    dns_record::id,
                    dns_record::record_name,
                    dns_record::content,
                    dns_record::ttl,
                    dns_record::record_type,
                    dns_record::proxied,
                ))
                .load::<(String, String, String, i32, String, bool)>(conn)
                .unwrap_or_default()
                .into_iter()
                .map(
                    |(r_id, r_name, r_content, r_ttl, r_type, r_proxied)| DnsRecord {
                        id: r_id,
                        name: r_name,
                        content: r_content,
                        ttl: r_ttl,
                        record_type: r_type,
                        proxied: r_proxied,
                    },
                )
                .collect();

            (
                Zone {
                    id: z_id,
                    name: z_name,
                    status: "active".to_string(),
                },
                records,
            )
        })
        .collect();

    (StatusCode::OK, Json(&zone_dns_data)).into_response()
}

async fn delete_dns_record(
    State(state): State<AppState>,
    User(claims): User,
    Query(params): Query<DeleteDnsRecord>,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let record_id = params.record_id;
    let zone_id = params.zone_id;

    let client = reqwest::Client::new();
    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    let decrypted_token = match get_user_token(conn, &user_id) {
        Ok(token) => token,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    };

    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );

    let resp = match client
        .delete(&url)
        .bearer_auth(&decrypted_token)
        .send()
        .await
    {
        Ok(r) => r,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    let response = resp.json::<DeleteRecordResponse>().await;

    let deleted_token = match response {
        Ok(r) => r.result,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    let result = conn.transaction(|conn| {
        diesel::delete(
            dns_record::table
                .filter(dns_record::user_id.eq(&user_id))
                .filter(dns_record::zone_id.eq(&zone_id))
                .filter(dns_record::id.eq(&deleted_token.id)),
        )
        .execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(_) => (StatusCode::OK, "Deleted DNS record from account").into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

// DNS Token Controls
async fn get_dns_access_tokens(
    State(state): State<AppState>,
    User(claims): User,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    let tokens = match dns_token::table
        .filter(dns_token::user_id.eq(&user_id))
        .select((dns_token::name, dns_token::id, dns_token::created_on))
        .load::<(String, String, NaiveDateTime)>(conn)
    {
        Ok(tokens) => tokens
            .into_iter()
            .map(|(name, id, created_on)| DnsAccessToken {
                name,
                id,
                created_on,
            })
            .collect::<Vec<_>>(),
        Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    };

    (StatusCode::OK, Json(&tokens)).into_response()
}

async fn add_dns_access_token(
    State(state): State<AppState>,
    User(claims): User,
    Json(body): Json<AddAccessToken>,
) -> impl IntoResponse {
    let name = body.name;
    let user_id = claims.sub;
    let dns_token_str = body.token;
    let id = Uuid::now_v7().to_string();
    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    let encrypted = match encrypt(&dns_token_str) {
        Ok(enc) => enc,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response();
        }
    };

    let new_token = NewDnsAccessToken {
        id: &id,
        name: &name,
        user_id: &user_id,
        nonce: &encrypted.nonce,
        token_encrypted: &encrypted.ciphertext,
        tag: &encrypted.tag,
    };

    let result = conn.transaction(|conn| {
        diesel::insert_into(dns_token::table)
            .values(&new_token)
            .execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(_) => (StatusCode::OK, Json("Added DNS token to account")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

async fn delete_access_token(
    State(state): State<AppState>,
    User(claims): User,
    Query(params): Query<DeleteAccessToken>,
) -> impl IntoResponse {
    let dns_token_id = params.token_id;
    let user_id = claims.sub;
    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    let result = conn.transaction(|conn| {
        diesel::delete(
            dns_token::table
                .filter(dns_token::id.eq(dns_token_id))
                .filter(dns_token::user_id.eq(user_id)),
        )
        .execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(_) => (StatusCode::OK, Json("Deleted DNS token from account")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

// API Key Controls
async fn get_api_keys(State(state): State<AppState>, User(claims): User) -> impl IntoResponse {
    let user_id = claims.sub;

    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    match api_keys::table
        .filter(api_keys::user_id.eq(&user_id))
        .inner_join(dns_record::table.on(api_keys::dns_record_id.eq(dns_record::id)))
        .select((
            api_keys::id,
            api_keys::created_on,
            api_keys::last_used,
            api_keys::name,
            dns_record::record_name,
        ))
        .load::<(String, NaiveDateTime, Option<NaiveDateTime>, String, String)>(conn)
    {
        Ok(response) => (
            StatusCode::OK,
            Json(
                response
                    .into_iter()
                    .map(|(id, created_on, last_used, name, record_name)| ApiKey {
                        id,
                        created_on,
                        last_used,
                        name,
                        record_name,
                    })
                    .collect::<Vec<_>>(),
            ),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(format!("Error: {}", err)),
        )
            .into_response(),
    }
}

async fn add_api_key(
    State(state): State<AppState>,
    User(claims): User,
    Json(body): Json<AddApiKey>,
) -> impl IntoResponse {
    let user_id = &claims.sub;
    let key_name = &body.name;
    let key_scope = &body.scope;

    let conn = &mut state.pool.get().expect("Failed to get DB connection");

    // Create an api key, hash it
    let (full_api_key, public_id, _secret) = generate_api_key();

    // Hash the FULL key (so verification is simple later)
    let hashed_key = hash_raw_string(&full_api_key).expect("Hash failed");

    let result = conn.transaction(|conn| {
        diesel::insert_into(api_keys::table)
            .values((
                api_keys::id.eq(Uuid::now_v7().to_string()),
                api_keys::name.eq(&key_name),
                api_keys::prefix_id.eq(&public_id),
                api_keys::key_hash.eq(&hashed_key),
                api_keys::dns_record_id.eq(&key_scope),
                api_keys::user_id.eq(&user_id),
            ))
            .execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    // Return full key to user
    match result {
        Ok(_) => (StatusCode::CREATED, Json(&full_api_key)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

async fn delete_api_key(
    User(claims): User,
    Query(params): Query<DeleteApiKeyParams>,
) -> impl IntoResponse {
    let api_key_id = params.key_id;
    let user_id = claims.sub;
    let mut conn = establish_connection();

    let result = conn.transaction(|conn| {
        diesel::delete(
            api_keys::table
                .filter(api_keys::id.eq(api_key_id))
                .filter(api_keys::user_id.eq(user_id)),
        )
        .execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(_) => (StatusCode::OK, Json("API key deleted from account")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}

// Helper functions
// TODO: Move these to a utils file
async fn initialize_zones(
    conn: &mut MysqlConnection,
    curr_user_id: &String,
    dns_access_token: &String,
    dns_access_token_id: &String,
) -> Result<Vec<Zone>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let zones_resp = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .bearer_auth(dns_access_token)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let data = zones_resp.json::<DnsZonesResponse>().await?;
    let zones = data.result;

    conn.transaction(|conn| {
        for zone in &zones {
            diesel::insert_into(dns_zone::table)
                .values((
                    dns_zone::id.eq(&zone.id),
                    dns_zone::user_id.eq(curr_user_id),
                    dns_zone::zone_name.eq(&zone.name),
                    dns_zone::token_id.eq(&dns_access_token_id),
                    dns_zone::last_synced_on.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)?;
        }
        Ok::<_, diesel::result::Error>(())
    })?;

    Ok(zones)
}
