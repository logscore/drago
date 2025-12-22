// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        #[max_length = 36]
        id -> Varchar,
        account_id -> Text,
        provider_id -> Text,
        #[max_length = 36]
        user_id -> Varchar,
        access_token -> Nullable<Text>,
        refresh_token -> Nullable<Text>,
        id_token -> Nullable<Text>,
        access_token_expires_at -> Nullable<Timestamp>,
        refresh_token_expires_at -> Nullable<Timestamp>,
        scope -> Nullable<Text>,
        password -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    api_keys (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 20]
        prefix_id -> Varchar,
        #[max_length = 97]
        key_hash -> Varchar,
        #[max_length = 255]
        dns_record_id -> Varchar,
        last_used -> Nullable<Timestamp>,
        created_on -> Timestamp,
        updated_on -> Timestamp,
    }
}

diesel::table! {
    device_code (id) {
        #[max_length = 36]
        id -> Varchar,
        #[sql_name = "device_code"]
        device_code_value -> Text,
        user_code -> Text,
        user_id -> Nullable<Text>,
        expires_at -> Timestamp,
        status -> Text,
        last_polled_at -> Nullable<Timestamp>,
        polling_interval -> Nullable<Integer>,
        client_id -> Nullable<Text>,
        scope -> Nullable<Text>,
    }
}

diesel::table! {
    dns_record (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 255]
        zone_id -> Varchar,
        #[max_length = 255]
        record_name -> Varchar,
        #[max_length = 16]
        record_type -> Varchar,
        content -> Text,
        ttl -> Integer,
        proxied -> Bool,
        last_synced_on -> Timestamp,
    }
}

diesel::table! {
    dns_token (id) {
        #[max_length = 225]
        id -> Varchar,
        #[max_length = 225]
        name -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 12]
        nonce -> Varbinary,
        #[max_length = 1024]
        token_encrypted -> Varbinary,
        #[max_length = 16]
        tag -> Varbinary,
        created_on -> Timestamp,
        updated_on -> Timestamp,
    }
}

diesel::table! {
    dns_zone (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 225]
        token_id -> Varchar,
        #[max_length = 255]
        zone_name -> Varchar,
        last_synced_on -> Timestamp,
        meta -> Nullable<Text>,
    }
}

diesel::table! {
    jwks (id) {
        #[max_length = 36]
        id -> Varchar,
        public_key -> Text,
        private_key -> Text,
        created_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    session (id) {
        #[max_length = 36]
        id -> Varchar,
        expires_at -> Timestamp,
        #[max_length = 255]
        token -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        #[max_length = 36]
        user_id -> Varchar,
    }
}

diesel::table! {
    user (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        email_verified -> Bool,
        image -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    verification (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 255]
        identifier -> Varchar,
        value -> Text,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(account -> user (user_id));
diesel::joinable!(api_keys -> dns_record (dns_record_id));
diesel::joinable!(api_keys -> user (user_id));
diesel::joinable!(dns_record -> dns_zone (zone_id));
diesel::joinable!(dns_record -> user (user_id));
diesel::joinable!(dns_token -> user (user_id));
diesel::joinable!(dns_zone -> dns_token (token_id));
diesel::joinable!(dns_zone -> user (user_id));
diesel::joinable!(session -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    api_keys,
    device_code,
    dns_record,
    dns_token,
    dns_zone,
    jwks,
    session,
    user,
    verification,
);
