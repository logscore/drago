use aes_gcm::aead::OsRng;
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use diesel::{ExpressionMethods, MysqlConnection, OptionalExtension, QueryDsl, RunQueryDsl};

use crate::{db::schema::dns_token, lib::encryption::decrypt};

pub fn hash_raw_string(raw_string: &String) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string 97 bytes
    let password_hash = argon2
        .hash_password(raw_string.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

pub fn get_user_token(conn: &mut MysqlConnection, user_id: &String) -> Result<String, String> {
    // Simplified to just return the decrypted token
    // Get the user's dns access token from our db
    let token_data = dns_token::table
        .filter(dns_token::user_id.eq(user_id))
        .select((dns_token::token_encrypted, dns_token::nonce, dns_token::tag))
        .first::<(Vec<u8>, Vec<u8>, Vec<u8>)>(conn)
        .optional();

    // Handle DB error or Missing Token
    let (ciphertext, nonce, tag) = match token_data {
        Ok(Some(data)) => data,
        Ok(None) => {
            return Err("No token found for account".to_string());
        }
        Err(e) => {
            eprintln!("Token Query failed: {:?}", e);
            return Err(format!("Database error: {}", e));
        }
    };

    // Decrypt the token
    let decrypted_token = decrypt(&nonce, &ciphertext, &tag).map_err(|e| {
        eprintln!("Decryption failed: {:?}", e);
        format!("Decryption error: {}", e)
    })?;

    Ok(decrypted_token)
}
