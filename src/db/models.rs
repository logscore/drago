use crate::db::schema;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::dns_token)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct DnsAccessToken {
    id: String,
    user_id: String,
    nonce: Vec<u8>,
    token_encrypted: Vec<u8>,
    tag: Vec<u8>,
    created_on: chrono::NaiveDateTime,
    updated_on: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::dns_token)]
pub struct NewDnsAccessToken<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub user_id: &'a str,
    pub nonce: &'a Vec<u8>,
    pub token_encrypted: &'a Vec<u8>,
    pub tag: &'a Vec<u8>,
}

#[derive(Debug, Queryable)]
#[diesel(table_name = schema::dns_record)]
pub struct PutDnsRecord {
    pub user_id: String,
    pub id: String,
    pub zone_id: String,
    pub content: String,
    pub record_name: String,
    pub ttl: i32,
    pub record_type: String,
}
