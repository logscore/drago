mod db;
mod encryption;

use axum::{
    Json, Router,
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use diesel::result::Error as DieselError;
use diesel::{prelude::*, sql_types::Binary};
use reqwest::{Client, dns};
use serde::{Deserialize, Serialize};
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
    let app = Router::new()
        .route("/health", get(health))
        .route("/records", get(list_dns_records))
        .route("/dns_access_key", post(add_dns_access_token));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json("Drago is running".to_string()))
}

async fn list_dns_records(Query(params): Query<GetRecords>) -> impl IntoResponse {
    let curr_user_id = params.user_id;

    let mut conn = establish_connection();
    use crate::db::schema::dns_token;

    // ---------------------------------------------------------
    // 1. Fetch the Token INDEPENDENTLY of zones
    // ---------------------------------------------------------
    let token_data = dns_token::table
        .filter(dns_token::user_id.eq(&curr_user_id))
        .select((dns_token::token_encrypted, dns_token::nonce, dns_token::tag))
        .first::<(Vec<u8>, Vec<u8>, Vec<u8>)>(&mut conn)
        .optional();

    // Handle DB error or Missing Token
    let (ciphertext, nonce, tag) = match token_data {
        Ok(Some(data)) => data,
        Ok(None) => return (StatusCode::NOT_FOUND, "No DNS Token found for user").into_response(),
        Err(e) => {
            eprintln!("Token Query failed: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response();
        }
    };

    // ---------------------------------------------------------
    // 2. Decrypt the token immediately
    // ---------------------------------------------------------
    let decrypted_token = match decrypt(&nonce, &ciphertext, &tag) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Decryption failed: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Security Error").into_response();
        }
    };

    // ---------------------------------------------------------
    // 3. Now fetch the zones
    // ---------------------------------------------------------
    let zones_result: Result<Vec<(String, String)>, DieselError> = dns_zone::table
        .filter(dns_zone::user_id.eq(&curr_user_id))
        .select((dns_zone::id, dns_zone::zone_name))
        .load::<(String, String)>(&mut conn);

    let mut zones = match zones_result {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Zone Query failed: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Zone DB Error").into_response();
        }
    };

    // ---------------------------------------------------------
    // 4. Logic: If empty, use the decrypted token to init
    // ---------------------------------------------------------
    if zones.is_empty() {
        if let Err(e) = initialize_zones(&mut conn, &curr_user_id, &decrypted_token).await {
            eprintln!("\nZone initialization failed: {:?}", e);
            return (StatusCode::NOT_FOUND, "Account has no domain names.").into_response();
        }
        // This is the actual query to see if the user has any domains after initializing the data in the users account. I could alternatively just have the initialize_zones() function return the zones data
        let new_zones = dns_zone::table
            .filter(dns_zone::user_id.eq(&curr_user_id))
            .select((dns_zone::id, dns_zone::zone_name))
            .load::<(String, String)>(&mut conn);

        match new_zones {
            Ok(z) => zones = z,
            Err(e) => {
                eprintln!("Re-query failed: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response();
            }
        }

        if zones.is_empty() {
            return (StatusCode::OK, "No domains found on DNS provider.").into_response();
        }
    } else {
        // TODO: Query the DB for the records and return the zone name, zone id, and dns records that cooresponds with each, in an API response
        let new_zones = dns_zone::table
            .filter(dns_zone::user_id.eq(&curr_user_id))
            .select((dns_zone::id, dns_zone::zone_name))
            .load::<(String, String)>(&mut conn);

        match new_zones {
            Ok(z) => zones = z,
            Err(e) => {
                eprintln!("Re-query failed: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response();
            }
        }
    }

    (StatusCode::OK, format!("\nZones: {:?}", zones)).into_response()
}

async fn initialize_zones(
    conn: &mut MysqlConnection,
    curr_user_id: &String,
    dns_access_token: &String,
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
                    dns_zone::token_id.eq("1"),
                    dns_zone::last_synced_at.eq(chrono::Utc::now().naive_utc()),
                ))
                .execute(conn)?;

            for record in records {
                println!("{:?}", &record.content);
                diesel::insert_into(dns_record::table)
                    .values((
                        dns_record::id.eq(&record.id),
                        dns_record::user_id.eq(curr_user_id),
                        dns_record::record_name.eq(&record.name),
                        dns_record::zone_id.eq(&zone.id),
                        dns_record::content.eq(&record.content),
                        dns_record::created_at.eq(chrono::Utc::now().naive_utc()),
                        dns_record::last_synced_at.eq(chrono::Utc::now().naive_utc()),
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
    println!("Fetching DNS records for zone: {}", zone.name);

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

async fn add_dns_access_token(Json(body): Json<AddAccessToken>) -> impl IntoResponse {
    // TODO: Add in a dns token here.
    let user_id = body.user_id;
    let dns_token = body.token;
    let id = Uuid::now_v7().to_string();
    let mut conn = establish_connection();

    // Encrypt the token here.
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

// --- Types ---

#[derive(Debug, Deserialize, Serialize)]
struct GetRecords {
    user_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AddAccessToken {
    user_id: String,
    token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DnsZonesResponse {
    result: Vec<Zone>,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct DnsRecordsResponse {
    result: Vec<DnsRecord>,
}

#[derive(Debug, Deserialize)]
struct DnsRecord {
    id: String,
    name: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Zone {
    id: String,
    name: String,
    status: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}
