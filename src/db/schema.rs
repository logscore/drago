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
        #[max_length = 64]
        key_hash -> Varchar,
        last_used -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    dns_record (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 255]
        record_name -> Varchar,
        #[max_length = 255]
        zone_id -> Varchar,
        #[max_length = 255]
        record_id -> Varchar,
        #[max_length = 45]
        current_ip -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    dns_token (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 36]
        user_id -> Varchar,
        token_encrypted -> Text,
        #[max_length = 255]
        account_email -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        name -> Text,
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
        identifier -> Text,
        value -> Text,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(account -> user (user_id));
diesel::joinable!(api_keys -> user (user_id));
diesel::joinable!(dns_record -> user (user_id));
diesel::joinable!(dns_token -> user (user_id));
diesel::joinable!(session -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    api_keys,
    dns_record,
    dns_token,
    session,
    user,
    verification,
);
