mod db;
mod lib;

use axum::{
    Json, Router,
    extract::Query,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use reqwest::Client;

use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::{
    db::schema::{api_keys, dns_record, dns_token, dns_zone},
    lib::{
        auth::{AuthState, User},
        encryption::{decrypt, encrypt},
        types::*,
        utils::{get_user_token, hash_raw_string},
    },
};
use crate::{
    db::{models::NewDnsAccessToken, *},
    lib::auth::generate_api_key,
};

#[tokio::main]
async fn main() {
    // TODO: replace hard coded url with the env variable
    let auth_state = AuthState::new("http://localhost:5173");

    let cors = CorsLayer::new()
        .allow_origin(Any)
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
        .layer(cors)
        .with_state(auth_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json("Drago is running. Let it rip.".to_string()),
    )
}

// DNS Record Controls
async fn add_dns_record(User(claims): User, Json(body): Json<AddDnsRecord>) -> impl IntoResponse {
    let user_id = claims.sub;
    let zone_id = body.zone_id;
    let zone_name = body.zone_name;
    let record_type = body.record_type;
    let name = body.name;
    let content = body.content;
    let ttl = body.ttl;
    let proxied = body.proxied;

    let conn = &mut establish_connection();
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

    // ... (Cloudflare Logic) ...
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
                dns_record::user_id.eq(&user_id), // Secure ID
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

async fn list_dns_records(User(claims): User) -> impl IntoResponse {
    let curr_user_id = claims.sub;
    let mut conn = establish_connection();

    let zones_result: Result<Vec<(String, String)>, DieselError> = dns_zone::table
        .filter(dns_zone::user_id.eq(&curr_user_id))
        .select((dns_zone::id, dns_zone::zone_name))
        .load::<(String, String)>(&mut conn);

    let zones = match zones_result {
        Ok(rows) => rows,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Zone DB Error").into_response(),
    };

    let mut zone_dns_data: Vec<(Zone, Vec<DnsRecord>)> = Vec::new();

    if zones.is_empty() {
        let token_data = dns_token::table
            .filter(dns_token::user_id.eq(&curr_user_id))
            .select((
                dns_token::id,
                dns_token::token_encrypted,
                dns_token::nonce,
                dns_token::tag,
            ))
            .first::<(String, Vec<u8>, Vec<u8>, Vec<u8>)>(&mut conn)
            .optional();

        let (token_id, ciphertext, nonce, tag) = match token_data {
            Ok(Some(data)) => data,
            Ok(None) => {
                return (StatusCode::NOT_FOUND, "No DNS Token found").into_response();
            }
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response(),
        };

        let decrypted_token = match decrypt(&nonce, &ciphertext, &tag) {
            Ok(t) => t,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Security Error").into_response(),
        };

        zone_dns_data =
            match initialize_zones(&mut conn, &curr_user_id, &decrypted_token, &token_id).await {
                Ok(data) => data,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error fetching DNS Zones",
                    )
                        .into_response();
                }
            };

        return (StatusCode::OK, Json(&zone_dns_data)).into_response();
    }

    // Existing DB logic
    let raw_zones = match dns_zone::table
        .filter(dns_zone::user_id.eq(&curr_user_id))
        .select((dns_zone::id, dns_zone::zone_name))
        .load::<(String, String)>(&mut conn)
    {
        Ok(z) => z,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response(),
    };

    for (z_id, z_name) in raw_zones {
        let raw_records = dns_record::table
            .filter(dns_record::zone_id.eq(&z_id))
            .select((
                dns_record::id,
                dns_record::record_name,
                dns_record::content,
                dns_record::ttl,
                dns_record::record_type,
                dns_record::proxied,
            ))
            .load::<(String, String, String, i32, String, bool)>(&mut conn)
            .unwrap_or_default();

        let records_structs: Vec<DnsRecord> = raw_records
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

        let zone_struct = Zone {
            id: z_id,
            name: z_name,
            status: "active".to_string(),
        };

        zone_dns_data.push((zone_struct, records_structs));
    }

    (StatusCode::OK, Json(&zone_dns_data)).into_response()
}

async fn delete_dns_record(
    User(claims): User,
    Query(params): Query<DeleteDnsRecord>,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let record_id = params.record_id;
    let zone_id = params.zone_id;

    let client = reqwest::Client::new();
    let conn = &mut establish_connection();

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
    User(claims): User, // AUTHENTICATED
                        // Params removed
) -> impl IntoResponse {
    let user_id = claims.sub; // Secure ID
    let mut conn = establish_connection();

    let tokens = match dns_token::table
        .filter(dns_token::user_id.eq(&user_id))
        .select((dns_token::name, dns_token::id, dns_token::created_on))
        .load::<(String, String, NaiveDateTime)>(&mut conn)
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
    User(claims): User, // AUTHENTICATED
    Json(body): Json<AddAccessToken>,
) -> impl IntoResponse {
    let name = body.name;
    let user_id = claims.sub; // Secure ID
    let dns_token_str = body.token;
    let id = Uuid::now_v7().to_string();
    let mut conn = establish_connection();

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
    User(claims): User,
    Query(params): Query<DeleteAccessToken>,
) -> impl IntoResponse {
    let dns_token_id = params.token_id;
    let user_id = claims.sub;
    let mut conn = establish_connection();

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
async fn get_api_keys(User(claims): User) -> impl IntoResponse {
    let user_id = claims.sub;

    let mut conn: MysqlConnection = establish_connection();

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
        .load::<(String, NaiveDateTime, Option<NaiveDateTime>, String, String)>(&mut conn)
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

async fn add_api_key(User(claims): User, Json(body): Json<AddApiKey>) -> impl IntoResponse {
    let user_id = &claims.sub;
    let key_name = &body.name;
    let key_scope = &body.scope;

    let mut conn = establish_connection();

    // Create an api key, hash it
    let api_key = generate_api_key();

    let hashed_key = match hash_raw_string(&api_key) {
        Ok(hashed_key) => hashed_key,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Failed to hash key: {}", err.to_string())),
            )
                .into_response();
        }
    };

    // Save the name, hash, scope id and user id to the database
    let result = conn.transaction(|conn| {
        diesel::insert_into(api_keys::table)
            .values((
                api_keys::id.eq(Uuid::now_v7().to_string()),
                api_keys::name.eq(key_name),
                api_keys::key_hash.eq(hashed_key),
                api_keys::dns_record_id.eq(key_scope),
                api_keys::user_id.eq(user_id),
            ))
            .execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(_) => (StatusCode::CREATED, Json(&api_key)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }

    // Frontend recieves the response value, and display it to the user to copy
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
) -> Result<Vec<(Zone, Vec<DnsRecord>)>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let zones_resp = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .bearer_auth(dns_access_token)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let data = zones_resp.json::<DnsZonesResponse>().await?;
    let zones = data.result;

    let mut account_dns_records: Vec<(Zone, Vec<DnsRecord>)> = Vec::new();

    for zone in zones {
        let zone_records = fetch_zone_records(conn, curr_user_id, &client, zone).await;
        if zone_records.success {
            account_dns_records.push(zone_records.data);
        }
    }

    if let Err(e) = conn.transaction(|conn| {
        for (zone, records) in &account_dns_records {
            diesel::insert_into(dns_zone::table)
                .values((
                    dns_zone::id.eq(&zone.id),
                    dns_zone::user_id.eq(curr_user_id),
                    dns_zone::zone_name.eq(&zone.name),
                    dns_zone::token_id.eq(&dns_access_token_id),
                    dns_zone::last_synced_on.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)?;

            for record in records {
                diesel::insert_into(dns_record::table)
                    .values((
                        dns_record::id.eq(&record.id),
                        dns_record::user_id.eq(curr_user_id),
                        dns_record::record_name.eq(&record.name),
                        dns_record::zone_id.eq(&zone.id),
                        dns_record::content.eq(&record.content),
                        dns_record::ttl.eq(&record.ttl),
                        dns_record::record_type.eq(&record.record_type),
                        dns_record::proxied.eq(&record.proxied),
                    ))
                    .execute(conn)?;
            }
        }
        Ok::<_, diesel::result::Error>(())
    }) {
        eprintln!("Failed to add DNS records to account: {:?}", e);
    }

    Ok(account_dns_records)
}

async fn fetch_zone_records(
    conn: &mut MysqlConnection,
    user_id: &String,
    client: &Client,
    zone: Zone,
) -> ApiResponse<(Zone, Vec<DnsRecord>)> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone.id
    );

    let user_dns_token = match get_user_token(conn, &user_id) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error fetching user DNS token: {:?}", e);
            return ApiResponse {
                success: false,
                message: "No DNS access token found".to_string(),
                data: (zone, Vec::new()),
            };
        }
    };

    let resp = match client
        .get(&url)
        .bearer_auth(&user_dns_token)
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("DNS provider request failed: {:?}", e);
            return ApiResponse {
                success: false,
                message: "DNS provider request failed".to_string(),
                data: (zone, Vec::new()),
            };
        }
    };

    match resp.json::<DnsRecordsResponse>().await {
        Ok(d) => ApiResponse {
            success: true,
            message: "DNS records fetched successfully".to_string(),
            data: (zone, d.result),
        },
        Err(e) => {
            eprintln!("JSON parse error: {:?}", e);
            ApiResponse {
                success: false,
                message: "Failed to parse DNS records".to_string(),
                data: (zone, Vec::new()),
            }
        }
    }
}
