mod db;

use self::models::*;
use axum::{Json, Router, extract::Query, http::StatusCode, response::IntoResponse, routing::get};
use diesel::prelude::*;

use serde::{Deserialize, Serialize};

use crate::db::*;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/health", get(health))
        // `POST /users` goes to `create_user`
        .route("/records", get(list_dns_records));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn health() -> (StatusCode, Json<String>) {
    (StatusCode::OK, Json("Drago is running".to_string()))
}

async fn list_dns_records(
    // This tells axum to parse the json in the body
    Query(params): Query<GetRecords>,
) {
    use self::schema::dns_token::dsl::*;
    // Recieve the user id, query the db for the cloudflare access token, and the zone name/id
    let curr_user_id = params.user_id;
    let conn = &mut establish_connection();
    let result = dns_token
        .filter(user_id.eq(curr_user_id))
        .select(DnsAccessToken::as_select())
        .load(conn);

    println!("Result: {:?}", result)

    // If dns zone not available, query cloudflare and insert the zone into the db. Dont insert dns records that arent managed by us, they arent our concern. We'll insert them when the user makes them in the dashboard

    // Return the dns zone with the records in our db, or none. Don't return cloudflare records.
    // Get the user dns zones
    // let client = reqwest::Client::new();
    // let zones = match client
    //     .get("https://api.cloudflare.com/client/v4/zones")
    //     .header(
    //         "Authorization",
    //         "Bearer hZ2ar7s3edEQWIDmoxpzwvH5HIVL-m5pn3ouQScJ",
    //     )
    //     .header("Content-Type", "application/json")
    //     .send()
    //     .await
    // {
    //     Ok(response) => match response.json::<DnsResponse>().await {
    //         Ok(data) => {
    //             println!("Response zone {:?}", data.result);
    //             data.result
    //         }
    //         Err(e) => {
    //             eprintln!("Failed to parse JSON: {:?}", e);
    //             return;
    //         }
    //     },
    //     Err(e) => {
    //         eprintln!("HTTP request failed: {:?}", e);
    //         return;
    //     }
    // };

    // let zone = match zones.first() {
    //     Some(zone) => zone,
    //     None => {
    //         initialize_zones(client)
    //         return;
    //     }
    // };

    // let dns_records = client
}

// async fn initialize_zones(client: reqwest::Client) {}

// Get the dns records for that single zone

// the input to our `create_user` handler
// #[derive(Deserialize)]
// struct CreateRecord {
//     record_name: String,
//     zone_id: String,
//     record_type: String,
// }

#[derive(Debug, Deserialize, Serialize)]
struct GetRecords {
    user_id: String,
}

// #[derive(Debug, Deserialize, Serialize)]
// struct ListRecords {
//     user_id: String,
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct RecordsResult {
//     record_type: String,
//     record_name: String,
//     record_ttl: u8,
//     // record_content is the ip address, domain name or other in a DNS record
//     record_content: String,
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct Zone {
//     id: String,
//     name: String,
//     status: String,
// }
