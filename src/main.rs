mod db;
mod encryption;

use axum::{
    Json, Router,
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::db::{models::NewDnsAccessToken, *};
use crate::encryption::encrypt;
use crate::{
    db::schema::{dns_record, dns_token, dns_zone},
    encryption::decrypt,
};

const TOKEN: &str = "hZ2ar7s3edEQWIDmoxpzwvH5HIVL-m5pn3ouQScJ";

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/records", get(list_dns_records))
        .route("/record", post(add_dns_record))
        .route("/access_token", post(add_dns_access_token))
        .route("/access_token", delete(delete_access_token))
        .route("/access_tokens", get(get_dns_access_tokens))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json("Drago is running".to_string()))
}

async fn add_dns_record(Json(body): Json<AddDnsRecord>) {}

async fn list_dns_records(Query(params): Query<GetRecords>) -> impl IntoResponse {
    let curr_user_id = params.user_id;

    let mut conn = establish_connection();
    use crate::db::schema::dns_token;

    // Fetch the zones with their dns records from DNS provider
    let zones_result: Result<Vec<(String, String)>, DieselError> = dns_zone::table
        .filter(dns_zone::user_id.eq(&curr_user_id))
        .select((dns_zone::id, dns_zone::zone_name))
        .load::<(String, String)>(&mut conn);

    let zones = match zones_result {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Zone Query failed: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Zone DB Error").into_response();
        }
    };

    let mut zone_dns_data: Vec<(Zone, Vec<DnsRecord>)> = Vec::new();

    if zones.is_empty() {
        // We dont have the DNS records and zones from the DNS provider
        // Fetch the dns access token
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

        // Handle DB error or Missing Token
        let (token_id, ciphertext, nonce, tag) = match token_data {
            Ok(Some(data)) => data,
            Ok(None) => {
                return (StatusCode::NOT_FOUND, "No DNS Token found for user").into_response();
            }
            Err(e) => {
                eprintln!("Token Query failed: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response();
            }
        };

        // Decrypt the token
        let decrypted_token = match decrypt(&nonce, &ciphertext, &tag) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Decryption failed: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Security Error").into_response();
            }
        };

        zone_dns_data =
            match initialize_zones(&mut conn, &curr_user_id, &decrypted_token, &token_id).await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error fetching DNS records: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error fetching DNS Zones",
                    )
                        .into_response();
                }
            };

        // Return the zone dns data
        (StatusCode::OK, Json(&zone_dns_data)).into_response()
    } else {
        // We have records in the db, we use those
        // Query the DB for the users dns zones
        let raw_zones = match dns_zone::table
            .filter(dns_zone::user_id.eq(&curr_user_id))
            .select((dns_zone::id, dns_zone::zone_name))
            .load::<(String, String)>(&mut conn)
        {
            Ok(z) => z,
            Err(e) => {
                eprintln!("Zone fetch error: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response();
            }
        };

        // Loop through zones and fetch their records
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
                // Figure out how to make this tuple just a model struct in models.rs
                .load::<(String, String, String, i32, String, bool)>(&mut conn)
                .unwrap_or_default();

            // Map to DnsRecord struct
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

            // Map to Zone Struct
            let zone_struct = Zone {
                id: z_id,
                name: z_name,
                status: "active".to_string(),
            };

            zone_dns_data.push((zone_struct, records_structs));
        }

        (StatusCode::OK, Json(&zone_dns_data)).into_response()
    };

    // Not sure why this part is needed, but Rust yells at me if i dont have it. But we should never get to this point in the logic
    (StatusCode::OK, Json(&zone_dns_data)).into_response()
}

async fn get_dns_access_tokens(Query(params): Query<GetAccessTokens>) -> impl IntoResponse {
    let user_id = params.user_id;
    let mut conn = establish_connection();

    let tokens = match dns_token::table
        .filter(dns_token::user_id.eq(&user_id))
        .select((dns_token::id, dns_token::created_on))
        .load::<(String, NaiveDateTime)>(&mut conn)
    {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Failed to retrieve access tokens: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response();
        }
    };

    (StatusCode::OK, Json(&tokens)).into_response()
}

async fn add_dns_access_token(Json(body): Json<AddAccessToken>) -> impl IntoResponse {
    // TODO: Add in a dns token here.
    let user_id = body.user_id;
    let dns_token = body.token;
    let id = Uuid::now_v7().to_string();
    let mut conn = establish_connection();

    // Encrypts the dns access token
    let encrypted = match encrypt(&dns_token) {
        Ok(enc) => enc,
        Err(e) => {
            eprintln!("Encryption failed: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Encryption error").into_response();
        }
    };

    let new_token = NewDnsAccessToken {
        id: &id,
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
        Ok(_) => (StatusCode::OK, "Added DNS token to account").into_response(),
        Err(err) => {
            eprintln!("DB error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response()
        }
    }
}

async fn delete_access_token(Json(body): Json<DeleteAccessToken>) -> impl IntoResponse {
    let dns_token_id = body.token_id;

    let conn = &mut establish_connection();

    let result = conn.transaction(|conn| {
        diesel::delete(dns_token::table.filter(dns_token::id.eq(dns_token_id))).execute(conn)?;
        Ok::<_, diesel::result::Error>(())
    });

    match result {
        Ok(_) => (StatusCode::OK, "Deleted DNS token from account").into_response(),
        Err(err) => {
            eprintln!("DB error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response()
        }
    }
}

async fn initialize_zones(
    conn: &mut MysqlConnection,
    curr_user_id: &String,
    dns_access_token: &String,
    dns_access_token_id: &String,
) -> Result<Vec<(Zone, Vec<DnsRecord>)>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let zones_resp = client
        .get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", dns_access_token))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let data = zones_resp.json::<DnsZonesResponse>().await?;
    let zones = data.result;

    let mut account_dns_records: Vec<(Zone, Vec<DnsRecord>)> = Vec::new();

    for zone in zones {
        let zone_records = fetch_zone_records(&client, zone).await;
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

async fn fetch_zone_records(client: &Client, zone: Zone) -> ApiResponse<(Zone, Vec<DnsRecord>)> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone.id
    );

    let resp = match client
        .get(&url)
        .header("Authorization", format!("Bearer {}", TOKEN))
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("DNS provider request failed: {:?}", e);
            return ApiResponse {
                success: false,
                data: (zone, Vec::new()),
            };
        }
    };

    match resp.json::<DnsRecordsResponse>().await {
        Ok(d) => ApiResponse {
            success: true,
            data: (zone, d.result),
        },
        Err(e) => {
            eprintln!("JSON parse error: {:?}", e);
            ApiResponse {
                success: false,
                data: (zone, Vec::new()),
            }
        }
    }
}

// Types
#[derive(Debug, Deserialize, Serialize)]
struct AddDnsRecord {
    user_id: String,
    name: String,
    content: String,
    ttl: i32,
    proxied: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetRecords {
    user_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetAccessTokens {
    user_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AddAccessToken {
    user_id: String,
    token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeleteAccessToken {
    token_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DnsRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub ttl: i32,
    pub proxied: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Zone {
    id: String,
    name: String,
    status: String,
}

// From Cloudflare
#[derive(Debug, Deserialize, Serialize)]
struct DnsZonesResponse {
    result: Vec<Zone>,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct DnsRecordsResponse {
    result: Vec<DnsRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}
