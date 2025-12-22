use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AddDnsRecord {
    pub zone_id: String,
    pub zone_name: String,
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: i32,
    pub proxied: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteDnsRecord {
    pub record_id: String,
    pub zone_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddAccessToken {
    pub name: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteAccessToken {
    pub token_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub ttl: i32,
    pub proxied: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeletedDnsRecord {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct DnsAccessToken {
    pub name: String,
    pub id: String,
    pub created_on: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiKey {
    pub id: String,
    pub created_on: NaiveDateTime,
    pub last_used: Option<NaiveDateTime>,
    pub name: String,
    pub record_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteApiKeyParams {
    pub key_id: String,
}

// From Cloudflare
#[derive(Debug, Deserialize, Serialize)]
pub struct DnsZonesResponse {
    pub result: Vec<Zone>,
    pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateRecordResponse {
    pub result: DnsRecord,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct RecordResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct PutRecordResponse {
    pub result: RecordResponse,
    pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteRecordResponse {
    pub result: DeletedDnsRecord,
    pub success: bool,
}

#[derive(Serialize)]
pub struct DnsRecordPayload<'a> {
    pub r#type: &'a String,
    pub name: &'a String,
    pub content: &'a String,
    pub ttl: &'a i32,
    pub proxied: &'a bool,
}

#[derive(Serialize, Debug)]
pub struct PutDnsRecordPayload<'a> {
    pub r#type: &'a String,
    pub name: &'a String,
    pub content: &'a str,
    pub ttl: &'a i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddApiKey {
    pub name: String,
    pub scope: String,
}

// API key is sent in the authorization header
#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub ip_address: String,
    // Used as the new time updated and time api key used
    pub time_synced: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub updated: bool,
    pub message: String,
}
