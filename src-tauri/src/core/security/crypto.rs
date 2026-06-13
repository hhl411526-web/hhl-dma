use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use crate::errors::AppError;

const PBKDF2_ITERATIONS: u32 = 600_000;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;

pub fn encrypt_password(plaintext: &str, master_password: Option<&str>) -> Result<String, AppError> {
    let rng = SystemRandom::new();
    let mut salt = [0u8; SALT_LEN];
    rng.fill(&mut salt).map_err(|e| AppError::Connection(format!("Failed to generate salt: {}", e)))?;
    let key = derive_key(master_password.unwrap_or("hhl-dma-default-key"), &salt)?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes).map_err(|e| AppError::Connection(format!("Failed to generate nonce: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Connection(format!("Cipher init error: {}", e)))?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Connection(format!("Encryption error: {}", e)))?;
    let mut combined = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&combined))
}

pub fn decrypt_password(encrypted: &str, master_password: Option<&str>) -> Result<String, AppError> {
    let combined = BASE64.decode(encrypted)
        .map_err(|e| AppError::Connection(format!("Base64 decode error: {}", e)))?;
    if combined.len() < SALT_LEN + NONCE_LEN {
        return Err(AppError::Connection("Invalid encrypted data".to_string()));
    }
    let salt = &combined[..SALT_LEN];
    let nonce_bytes = &combined[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &combined[SALT_LEN + NONCE_LEN..];
    let key = derive_key(master_password.unwrap_or("hhl-dma-default-key"), salt)?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Connection(format!("Cipher init error: {}", e)))?;
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Connection(format!("Decryption error: {}", e)))?;
    String::from_utf8(plaintext)
        .map_err(|e| AppError::Connection(format!("UTF-8 decode error: {}", e)))
}

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], AppError> {
    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
        salt,
        password.as_bytes(),
        &mut key,
    );
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = "my_secret_password";
        let encrypted = encrypt_password(plaintext, Some("master")).unwrap();
        let decrypted = decrypt_password(&encrypted, Some("master")).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_default_key() {
        let plaintext = "my_secret_password";
        let encrypted = encrypt_password(plaintext, None).unwrap();
        let decrypted = decrypt_password(&encrypted, None).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_master_password_fails() {
        let plaintext = "my_secret_password";
        let encrypted = encrypt_password(plaintext, Some("correct")).unwrap();
        let result = decrypt_password(&encrypted, Some("wrong"));
        assert!(result.is_err());
    }
}
