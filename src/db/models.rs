use crate::db::schema;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::dns_token)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct DnsAccessToken {
    id: String,
    user_id: String,
    token_encrypted: String,
    account_email: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = schema::dns_token)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ZoneRecord {
    
}