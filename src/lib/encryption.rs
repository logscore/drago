use aes_gcm::{
    AeadCore, Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use anyhow::{Context, Result};
use std::env;

pub struct EncryptionResult {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}

fn get_key() -> Result<[u8; 32]> {
    let key_hex = env::var("ENCRYPTION_KEY").context("ENCRYPTION_KEY not set")?;
    let key_bytes = hex::decode(key_hex).context("Failed to decode key hex")?;

    if key_bytes.len() != 32 {
        return Err(anyhow::anyhow!(
            "Key must be exactly 32 bytes (64 hex chars)"
        ));
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    Ok(key_array)
}

pub fn encrypt(plain_text: &str) -> Result<EncryptionResult> {
    let key = get_key()?;
    let cipher = Aes256Gcm::new(&key.into());

    // 1. Generate random 96-bit (12-byte) nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

    // 2. Encrypt. Note: aes-gcm returns [ciphertext + tag] combined
    let encrypted_combined = cipher
        .encrypt(&nonce, plain_text.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failure: {}", e))?;

    // 3. Split the Tag (last 16 bytes) from Ciphertext for your DB schema
    let tag_len = 16;
    if encrypted_combined.len() < tag_len {
        return Err(anyhow::anyhow!("Encryption output too short"));
    }

    let split_index = encrypted_combined.len() - tag_len;
    let ciphertext = encrypted_combined[..split_index].to_vec();
    let tag = encrypted_combined[split_index..].to_vec();

    Ok(EncryptionResult {
        nonce: nonce.to_vec(),
        ciphertext,
        tag,
    })
}

pub fn decrypt(nonce_vec: &[u8], ciphertext: &[u8], tag: &[u8]) -> Result<String> {
    let key = get_key()?;
    let cipher = Aes256Gcm::new(&key.into());

    // 1. Reconstruct the Nonce object
    let nonce = Nonce::from_slice(nonce_vec);

    // 2. Reconstruct [ciphertext + tag] payload
    // Rust's aes-gcm expects the tag appended to the end
    let mut combined_payload = ciphertext.to_vec();
    combined_payload.extend_from_slice(tag);

    // 3. Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, combined_payload.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failure: {}", e))?;

    let plaintext = String::from_utf8(plaintext_bytes)?;

    Ok(plaintext)
}
