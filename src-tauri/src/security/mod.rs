use crate::errors::{AppError, AppResult};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn hash_password(password: &str) -> AppResult<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::PasswordHash)?
        .to_string();

    Ok((hash, salt.as_str().to_string()))
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|_| AppError::PasswordHash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_secret_hex() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hmac_lookup(secret_hex: &str, value: &str) -> AppResult<String> {
    let key = hex::decode(secret_hex).map_err(|_| AppError::Crypto)?;
    let mut mac = <HmacSha256 as Mac>::new_from_slice(&key).map_err(|_| AppError::Crypto)?;
    mac.update(value.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn encrypt_sensitive(secret_hex: &str, plaintext: &str) -> AppResult<Vec<u8>> {
    let key = hex::decode(secret_hex).map_err(|_| AppError::Crypto)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::Crypto)?;
    let mut nonce_bytes = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_bytes())
        .map_err(|_| AppError::Crypto)?;

    let mut encrypted = nonce_bytes.to_vec();
    encrypted.extend(ciphertext);
    Ok(encrypted)
}

pub fn decrypt_sensitive(secret_hex: &str, encrypted: &[u8]) -> AppResult<String> {
    if encrypted.len() <= 12 {
        return Err(AppError::Crypto);
    }

    let key = hex::decode(secret_hex).map_err(|_| AppError::Crypto)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AppError::Crypto)?;
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| AppError::Crypto)?;
    String::from_utf8(plaintext).map_err(|_| AppError::Crypto)
}
